use anyhow::Result;
use image::DynamicImage;
use tokio::sync::mpsc;

use crate::events::{InstinctEvent, ObjectClass};
use crate::models::buffalo::BuffaloExtractor;
use crate::models::yolo::YoloDetector;
use crate::tracking::bytetrack::{ByteTrack, TrackState};

pub struct InstinctPipeline {
    yolo: YoloDetector,
    tracker: ByteTrack,
    event_tx: mpsc::Sender<InstinctEvent>,
    buffalo_tx: mpsc::Sender<(u64, [f32; 4], DynamicImage)>,
}

impl InstinctPipeline {
    /// Create a new [`InstinctPipeline`].
    pub fn new(event_tx: mpsc::Sender<InstinctEvent>) -> Result<Self> {
        let yolo = YoloDetector::new("../../models/yolo26n.onnx")?;
        let tracker = ByteTrack::new(0.5, 30, 0.8, 0.6);

        let (buffalo_tx, buffalo_rx) = mpsc::channel(10);
        Self::spawn_buffalo_worker(buffalo_rx, event_tx.clone());

        Ok(Self {
            yolo,
            tracker,
            event_tx,
            buffalo_tx,
        })
    }

    /// Processes a single frame.
    pub async fn process_frame(&mut self, image: DynamicImage) -> Result<()> {
        let detections = self.yolo.detect(&image)?;

        let tracking_inputs: Vec<_> = detections
            .iter()
            .map(|(obb, score, cls)| (obb.to_tlwh(), *score, *cls as i64))
            .collect();

        let active_tracks = self.tracker.update(tracking_inputs);

        for track in active_tracks {
            let class = ObjectClass::from(track.class_id);

            let obb = crate::events::OrientedBoundingBox {
                cx: track.tlwh[0] + track.tlwh[2] / 2.0,
                cy: track.tlwh[1] + track.tlwh[3] / 2.0,
                width: track.tlwh[2],
                height: track.tlwh[3],
                angle: 0.0,
            };

            let _ = self
                .event_tx
                .send(InstinctEvent::ObstacleDetected {
                    id: track.track_id,
                    class,
                    obb,
                    confidence: track.score,
                })
                .await;

            if class == ObjectClass::Person {
                let _ = self
                    .event_tx
                    .send(InstinctEvent::PersonTracked {
                        track_id: track.track_id,
                        obb,
                    })
                    .await;

                if track.state == TrackState::New && track.is_activated {
                    let _ = self
                        .buffalo_tx
                        .send((track.track_id, track.tlwh, image.clone()))
                        .await;
                }
            }
        }

        for lost_id in self.tracker.get_lost_track_ids() {
            let _ =
                self.event_tx.send(InstinctEvent::TrackLost(lost_id)).await;
        }

        Ok(())
    }

    /// Dedicated async worker for heavy Re-ID inference.
    fn spawn_buffalo_worker(
        mut rx: mpsc::Receiver<(u64, [f32; 4], DynamicImage)>,
        event_tx: mpsc::Sender<InstinctEvent>,
    ) {
        tokio::task::spawn_blocking(move || {
            // MODIFICATION ICI : On passe le chemin du dossier, pas un fichier
            // .onnx
            let mut buffalo =
                match BuffaloExtractor::new("../../models/buffalo_l") {
                    Ok(b) => b,
                    Err(err) => {
                        tracing::error!(?err, "failed to initialize buffalo");
                        return;
                    },
                };

            while let Some((track_id, bbox, image)) = rx.blocking_recv() {
                match buffalo.extract(&image, bbox) {
                    Ok(embedding) => {
                        let _ = event_tx.blocking_send(
                            InstinctEvent::PersonIdentityExtracted {
                                track_id,
                                embedding,
                            },
                        );
                    },
                    Err(err) => {
                        tracing::debug!(
                            ?track_id,
                            ?err,
                            "failed to extract embedding",
                        );
                    },
                }
            }
        });
    }
}

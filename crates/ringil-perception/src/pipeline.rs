use std::sync::mpsc;
use std::thread;

use eyre::{Context, Result};
use image::DynamicImage;

use crate::events::{InstinctEvent, ObjectClass, OrientedBoundingBox};
use crate::models::buffalo::BuffaloExtractor;
use crate::models::yolo::YoloDetector;
use crate::tracking::bytetrack::{ByteTrack, TrackState};

struct BuffaloJob {
    track_id: u64,
    cropped_image: DynamicImage,
}

pub struct InstinctPipeline {
    yolo: YoloDetector,
    tracker: ByteTrack,
    buffalo_tx: mpsc::SyncSender<BuffaloJob>,
    pub buffalo_rx: mpsc::Receiver<InstinctEvent>,
    tracking_cache: Vec<([f32; 4], f32, i64)>,
}

impl InstinctPipeline {
    /// Create a new [`InstinctPipeline`].
    pub fn new() -> Result<Self> {
        let yolo = YoloDetector::new("../../models/yolo26n.onnx")
            .context("Failed to load YOLO model")?;
        let tracker = ByteTrack::new(0.5, 30, 0.8, 0.6);

        let (buffalo_tx, rx) = mpsc::sync_channel(16);
        let (tx, buffalo_rx) = mpsc::channel();

        Self::spawn_buffalo_worker(rx, tx);

        Ok(Self {
            yolo,
            tracker,
            buffalo_tx,
            buffalo_rx,
            tracking_cache: Vec::with_capacity(32),
        })
    }

    /// Processes a single frame and returns a batch of immediate events.
    pub fn process_frame(
        &mut self,
        image: DynamicImage,
    ) -> Result<Vec<InstinctEvent>> {
        let detections = self.yolo.detect(&image)?;
        let mut events = Vec::with_capacity(detections.len() * 2); // Pre-allocate estimation

        self.tracking_cache.clear();
        for (obb, score, cls) in &detections {
            self.tracking_cache
                .push((obb.to_tlwh(), *score, *cls as i64));
        }

        let active_tracks = self.tracker.update(&self.tracking_cache);

        for track in active_tracks {
            let class = ObjectClass::from(track.class_id);

            let obb = OrientedBoundingBox {
                cx: track.tlwh[0] + track.tlwh[2] / 2.0,
                cy: track.tlwh[1] + track.tlwh[3] / 2.0,
                width: track.tlwh[2],
                height: track.tlwh[3],
                angle: 0.0,
            };

            events.push(InstinctEvent::ObstacleDetected {
                id: track.track_id,
                class,
                obb,
                confidence: track.score,
            });

            if class == ObjectClass::Person {
                events.push(InstinctEvent::PersonTracked {
                    track_id: track.track_id,
                    obb,
                });

                if track.state == TrackState::New && track.is_activated {
                    let x = track.tlwh[0].max(0.0) as u32;
                    let y = track.tlwh[1].max(0.0) as u32;
                    let w = track.tlwh[2].min(image.width() as f32) as u32;
                    let h = track.tlwh[3].min(image.height() as f32) as u32;

                    if w > 0 && h > 0 {
                        let cropped = image.crop_imm(x, y, w, h);
                        let _ = self.buffalo_tx.try_send(BuffaloJob {
                            track_id: track.track_id,
                            cropped_image: cropped,
                        });
                    }
                }
            }
        }

        for lost_id in self.tracker.get_lost_track_ids() {
            events.push(InstinctEvent::TrackLost(lost_id));
        }

        Ok(events)
    }

    /// Dedicated OS thread worker for heavy Re-ID inference (Buffalo).
    /// Removes reliance on Tokio runtime states.
    fn spawn_buffalo_worker(
        rx: mpsc::Receiver<BuffaloJob>,
        event_tx: mpsc::Sender<InstinctEvent>,
    ) {
        thread::spawn(move || {
            let mut buffalo = match BuffaloExtractor::new(
                "../../models/buffalo_l",
            ) {
                Ok(b) => b,
                Err(err) => {
                    tracing::error!(
                        ?err,
                        "failed to initialize buffalo extractor background worker"
                    );
                    return;
                },
            };

            while let Ok(job) = rx.recv() {
                // The background worker receives a tight, lightweight crop
                // wrapper containing only the localized person dimensions.
                let bbox_placeholder = [
                    0.0,
                    0.0,
                    job.cropped_image.width() as f32,
                    job.cropped_image.height() as f32,
                ];

                match buffalo.extract(&job.cropped_image, bbox_placeholder) {
                    Ok(embedding) => {
                        let _ = event_tx.send(
                            InstinctEvent::PersonIdentityExtracted {
                                track_id: job.track_id,
                                embedding,
                            },
                        );
                    },
                    Err(err) => {
                        tracing::debug!(
                            track_id = ?job.track_id,
                            ?err,
                            "failed to extract person re-id embedding"
                        );
                    },
                }
            }
        });
    }
}

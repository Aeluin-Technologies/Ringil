use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::thread;

use eyre::{Context, Result};
use flume;
use image::DynamicImage;

use crate::events::{InstinctEvent, ObjectClass, OrientedBoundingBox};
use crate::models::buffalo::BuffaloExtractor;
use crate::models::yolo::YoloDetector;
use crate::tracking::bytetrack::{ByteTrack, TrackState};

struct BuffaloJob {
    track_id: u64,
    full_image: Arc<DynamicImage>,
    bbox: [f32; 4],
}

pub struct InstinctPipeline {
    yolo: YoloDetector,
    tracker: ByteTrack,
    buffalo_tx: flume::Sender<BuffaloJob>,
    pub buffalo_rx: flume::Receiver<InstinctEvent>,
    tracking_cache: Vec<([f32; 4], f32, i64)>,
    buffalo_enabled: bool,
}

impl InstinctPipeline {
    /// Create a new [`InstinctPipeline`].
    pub fn new() -> Result<Self> {
        let yolo = YoloDetector::new(get_model_path("yolo26s.onnx"))
            .context("Failed to load YOLO model")?;
        let tracker = ByteTrack::new(0.5, 180, 0.8, 0.6);

        let (buffalo_tx, rx) = flume::bounded(16);
        let (tx, buffalo_rx) = flume::unbounded();

        Self::spawn_buffalo_worker(rx, tx);

        Ok(Self {
            yolo,
            tracker,
            buffalo_tx,
            buffalo_rx,
            tracking_cache: Vec::with_capacity(32),
            buffalo_enabled: true,
        })
    }

    /// Dynamically enable or disable Buffalo extractions at runtime.
    pub fn extract_embeddings(&mut self, enabled: bool) {
        self.buffalo_enabled = enabled;
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
        let shared_image = Arc::new(image);

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

                if self.buffalo_enabled
                    && track.state == TrackState::New
                    && track.is_activated
                {
                    let _ = self.buffalo_tx.try_send(BuffaloJob {
                        track_id: track.track_id,
                        full_image: Arc::clone(&shared_image),
                        bbox: track.tlwh,
                    });
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
        rx: flume::Receiver<BuffaloJob>,
        event_tx: flume::Sender<InstinctEvent>,
    ) {
        thread::spawn(move || {
            let mut buffalo = match BuffaloExtractor::new(get_model_path(
                "buffalo_s",
            )) {
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
                let x = job.bbox[0].max(0.0) as u32;
                let y = job.bbox[1].max(0.0) as u32;
                let w = job.bbox[2].min(job.full_image.width() as f32) as u32;
                let h = job.bbox[3].min(job.full_image.height() as f32) as u32;

                if w > 0 && h > 0 {
                    let cropped = job.full_image.crop_imm(x, y, w, h);

                    // The background worker receives a tight, lightweight crop
                    // wrapper containing only the localized person dimensions.
                    let bbox_placeholder = [
                        0.0,
                        0.0,
                        cropped.width() as f32,
                        cropped.height() as f32,
                    ];

                    match buffalo.extract(&cropped, bbox_placeholder) {
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
            }
        });
    }
}

fn get_model_path(model_name: &str) -> PathBuf {
    if let Ok(env_path) = std::env::var("MODEL_DIR") {
        return Path::new(&env_path).join(model_name);
    }

    if let Some(manifest_dir) = option_env!("CARGO_MANIFEST_DIR") {
        let crate_path = Path::new(manifest_dir);
        if let Some(workspace_root) =
            crate_path.parent().and_then(|p| p.parent())
        {
            let model_path = workspace_root.join("models").join(model_name);
            if model_path.exists() {
                return model_path;
            }
        }
    }

    Path::new("models").join(model_name)
}

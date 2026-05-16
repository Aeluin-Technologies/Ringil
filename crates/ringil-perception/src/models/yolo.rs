//! YOLO models handler.

use std::path::Path;

use crate::events::{ObjectClass, OrientedBoundingBox};
use eyre::{Context, Result};
use image::DynamicImage;
use ultralytics_inference::{Device, InferenceConfig, YOLOModel};

pub struct YoloDetector {
    model: YOLOModel,
}

impl YoloDetector {
    pub fn new(model_path: impl AsRef<Path>) -> Result<Self> {
        let config = InferenceConfig::new()
            .with_device(Device::Cuda(0))
            .with_device(Device::TensorRt(0))
            .with_threads(4)
            .with_batch(1)
            .with_confidence(0.0)
            .with_iou(0.40);

        let model = YOLOModel::load_with_config(&model_path, config)
            .with_context(|| {
                format!(
                    "Failed to load YOLO model from {:?}",
                    model_path.as_ref()
                )
            })?;

        tracing::info!(?model, "started yolo26 model");

        Ok(Self { model })
    }

    pub fn detect(
        &mut self,
        image: &DynamicImage,
    ) -> Result<Vec<(OrientedBoundingBox, f32, ObjectClass)>> {
        let results = self.model.predict_image(image, "stream".to_string())?;
        let mut detections = Vec::with_capacity(16);

        for result in &results {
            if let Some(ref boxes) = result.boxes {
                let xywh = boxes.xywh();
                let conf = boxes.conf();
                let cls = boxes.cls();

                for i in 0..boxes.len() {
                    detections.push((
                        OrientedBoundingBox {
                            cx: xywh[[i, 0]],
                            cy: xywh[[i, 1]],
                            width: xywh[[i, 2]],
                            height: xywh[[i, 3]],
                            angle: 0.0,
                        },
                        conf[i],
                        ObjectClass::from(cls[i] as i64),
                    ));
                }
            }
        }

        Ok(detections)
    }
}

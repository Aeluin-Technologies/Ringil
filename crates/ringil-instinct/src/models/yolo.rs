//! YOLO models handler.

use anyhow::{Context, Result};
use image::DynamicImage;
use ultralytics_inference::{Device, InferenceConfig, YOLOModel};

use crate::events::{ObjectClass, OrientedBoundingBox};

pub struct YoloDetector {
    model: YOLOModel,
}

impl YoloDetector {
    pub fn new(model_path: &str) -> Result<Self> {
        let config = InferenceConfig::new()
            .with_device(Device::Cuda(0))
            .with_threads(4)
            .with_confidence(0.4)
            .with_iou(0.45);

        let model = YOLOModel::load_with_config(model_path, config)
            .with_context(|| {
                format!("Failed to load YOLO model from {model_path}")
            })?;

        Ok(Self { model })
    }

    pub fn detect(
        &mut self,
        image: &DynamicImage,
    ) -> Result<Vec<(OrientedBoundingBox, f32, ObjectClass)>> {
        let results = self.model.predict_image(image, "stream".to_string())?;

        let mut detections = Vec::new();

        for result in &results {
            if let Some(ref boxes) = result.boxes {
                let xywh = boxes.xywh();
                let conf = boxes.conf();
                let cls = boxes.cls();

                for i in 0..boxes.len() {
                    let cx = xywh[[i, 0]];
                    let cy = xywh[[i, 1]];
                    let w = xywh[[i, 2]];
                    let h = xywh[[i, 3]];

                    let obb = OrientedBoundingBox {
                        cx,
                        cy,
                        width: w,
                        height: h,
                        angle: 0.0,
                    };

                    detections.push((
                        obb,
                        conf[i],
                        ObjectClass::from(cls[i] as i64),
                    ));
                }
            }
        }

        Ok(detections)
    }
}

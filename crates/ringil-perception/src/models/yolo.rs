//! YOLO models handler.

use std::path::Path;

use crate::events::{ObjectClass, OrientedBoundingBox};
use eyre::{Context, Result};
use image::{DynamicImage, RgbImage};
use ultralytics_inference::{Device, InferenceConfig, YOLOModel};

pub struct YoloDetector {
    model: YOLOModel,
}

impl YoloDetector {
    pub fn new(model_path: impl AsRef<Path>) -> Result<Self> {
        let config = InferenceConfig::new()
            .with_device(Device::Cuda(0))
            .with_device(Device::TensorRt(0))
            .with_device(Device::CoreMl)
            .with_device(Device::Cpu)
            .with_threads(4)
            .with_confidence(0.50)
            .with_iou(0.40);

        let model = YOLOModel::load_with_config(&model_path, config)
            .with_context(|| {
                format!(
                    "Failed to load YOLO model from {:?}",
                    model_path.as_ref()
                )
            })?;

        Ok(Self { model })
    }

    pub fn detect(
        &mut self,
        image: &DynamicImage,
    ) -> Result<Vec<(OrientedBoundingBox, f32, ObjectClass)>> {
        let rgb_img = image.as_rgb8().expect("Input must be RGB8");
        let orig_w = rgb_img.width();
        let orig_h = rgb_img.height();
        let target_dim = orig_w.max(orig_h);

        let mut padded_buffer =
            vec![0u8; (target_dim * target_dim * 3) as usize];
        let pad_x = (target_dim - orig_w) / 2;
        let pad_y = (target_dim - orig_h) / 2;

        let src_stride = (orig_w * 3) as usize;
        let dst_stride = (target_dim * 3) as usize;
        let src_raw = rgb_img.as_raw();

        for y in 0..orig_h {
            let src_start = (y as usize) * src_stride;
            let src_end = src_start + src_stride;

            let dst_y = y + pad_y;
            let dst_start =
                (dst_y as usize) * dst_stride + (pad_x as usize) * 3;
            let dst_end = dst_start + src_stride;

            padded_buffer[dst_start..dst_end]
                .copy_from_slice(&src_raw[src_start..src_end]);
        }

        let square_img = DynamicImage::ImageRgb8(
            RgbImage::from_raw(target_dim, target_dim, padded_buffer).unwrap(),
        );

        let results = self
            .model
            .predict_image(&square_img, "stream".to_string())?;
        let mut detections = Vec::new();

        for result in &results {
            if let Some(ref boxes) = result.boxes {
                let xywh = boxes.xywh();
                let conf = boxes.conf();
                let cls = boxes.cls();

                for i in 0..boxes.len() {
                    detections.push((
                        OrientedBoundingBox {
                            cx: xywh[[i, 0]] - pad_x as f32,
                            cy: xywh[[i, 1]] - pad_y as f32,
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

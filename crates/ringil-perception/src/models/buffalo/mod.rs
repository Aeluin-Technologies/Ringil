//! Buffalo model handler.

mod detection;
pub mod types;
mod utils;
mod warp;

use std::path::Path;

use detection::{distance2bbox, distance2kps, nms};
use eyre::{Context, Result};
use image::{DynamicImage, Rgba32FImage};
use ort::inputs;
use ort::session::Session;
use ort::session::builder::GraphOptimizationLevel;
use ort::value::Value;
use types::{ARCFACE_DST, Face};
use utils::{fast_resize, image_to_tensor_rgb, image_to_tensor_rgba32f};
use warp::{umeyama, warp_into};

pub struct BuffaloExtractor {
    det_10g: Session,
    w600k_r50: Session,
    _2d106det: Option<Session>,
    _1k3d68: Option<Session>,
}

impl BuffaloExtractor {
    // Initializes the ONNX sessions.
    pub fn new(models_dir: impl AsRef<Path>) -> Result<Self> {
        let path = models_dir.as_ref();
        let build = |file: &str| -> Result<Session> {
            Session::builder()?
                .with_optimization_level(GraphOptimizationLevel::Level3)
                .unwrap()
                .with_intra_threads(4)
                .unwrap()
                .commit_from_file(path.join(file))
                .with_context(|| format!("Failed to load model: {file}"))
        };

        Ok(Self {
            det_10g: build("det_10g.onnx")?,
            w600k_r50: build("w600k_r50.onnx")?,
            _2d106det: build("2d106det.onnx").ok(),
            _1k3d68: build("1k3d68.onnx").ok(),
        })
    }

    // Extracts the 512D facial embedding from an image, matching the provided
    // tracking bbox.
    pub fn extract(
        &mut self,
        image: &DynamicImage,
        bbox_tracked: [f32; 4],
    ) -> Result<Vec<f32>> {
        let orig_w = image.width() as f32;
        let orig_h = image.height() as f32;

        let img_640 = fast_resize(image, 640, 640)?;
        let det_tensor = image_to_tensor_rgb(&img_640);

        // Run detection and NMS filtering.
        let mut faces = self.run_detection(det_tensor, 0.5)?;
        faces = nms(faces, 0.4);

        if faces.is_empty() {
            return Err(eyre::eyre!("No faces detected"));
        }

        // Match detected faces with the tracker's bounding box using Euclidean
        // distance.
        let scale_x = orig_w / 640.0;
        let scale_y = orig_h / 640.0;
        let track_cx = bbox_tracked[0] + bbox_tracked[2] / 2.0;
        let track_cy = bbox_tracked[1] + bbox_tracked[3] / 2.0;

        let best_face = faces
            .into_iter()
            .min_by(|a, b| {
                let dist_a = ((((a.bbox.0 + a.bbox.2) / 2.0) * scale_x)
                    - track_cx)
                    .powi(2)
                    + ((((a.bbox.1 + a.bbox.3) / 2.0) * scale_y) - track_cy)
                        .powi(2);
                let dist_b = ((((b.bbox.0 + b.bbox.2) / 2.0) * scale_x)
                    - track_cx)
                    .powi(2)
                    + ((((b.bbox.1 + b.bbox.3) / 2.0) * scale_y) - track_cy)
                        .powi(2);
                dist_a.partial_cmp(&dist_b).unwrap()
            })
            .unwrap();

        // Map keypoints back to the original image dimensions.
        let scaled_kps =
            best_face.keypoints.map(|(x, y)| (x * scale_x, y * scale_y));

        // Align and crop the 112x112 facial region.
        let original_rgba = image.to_rgba32f();
        let face_crop = Self::crop_and_align(&original_rgba, &scaled_kps, 112);

        let emb_tensor = image_to_tensor_rgba32f(&face_crop);
        self.run_embedding(emb_tensor)
    }

    // Runs the det_10g ONNX model to find facial candidates.
    fn run_detection(
        &mut self,
        tensor: ndarray::Array4<f32>,
        thresh: f32,
    ) -> Result<Vec<Face>> {
        let outputs = self.det_10g.run(inputs![Value::from_array(tensor)?])?;
        let mut result = Vec::with_capacity(32);

        let strides = [8, 16, 32];
        let limits = [12800, 3200, 800]; // Anchors per stride grid

        for (i, (&stride, &limit)) in
            strides.iter().zip(limits.iter()).enumerate()
        {
            // Reconstruct 2D arrays from the flat slices returned by ONNX.
            let (s_shape, s_data) = outputs[i].try_extract_tensor::<f32>()?;
            let scores = ndarray::ArrayView2::from_shape(
                (s_shape[0] as usize, s_shape[1] as usize),
                s_data,
            )?;

            let (b_shape, b_data) =
                outputs[i + 3].try_extract_tensor::<f32>()?;
            let bboxes = ndarray::ArrayView2::from_shape(
                (b_shape[0] as usize, b_shape[1] as usize),
                b_data,
            )?
            .into_dyn();

            let (k_shape, k_data) =
                outputs[i + 6].try_extract_tensor::<f32>()?;
            let kpsses = ndarray::ArrayView2::from_shape(
                (k_shape[0] as usize, k_shape[1] as usize),
                k_data,
            )?
            .into_dyn();

            // Decode predictions mapping to bounding boxes and landmarks.
            for idx in 0..limit {
                let score = scores[[idx, 0]];
                if score > thresh {
                    result.push(Face {
                        score,
                        bbox: distance2bbox(idx, stride, &bboxes),
                        keypoints: distance2kps(idx, stride, &kpsses),
                    });
                }
            }
        }
        Ok(result)
    }

    // Runs the w600k_r50 model to calculate facial embeddings.
    fn run_embedding(
        &mut self,
        tensor: ndarray::Array4<f32>,
    ) -> Result<Vec<f32>> {
        let outputs =
            self.w600k_r50.run(inputs![Value::from_array(tensor)?])?;
        let (_, data) = outputs[0].try_extract_tensor::<f32>()?;

        Ok(data[..512].to_vec())
    }

    // Calculates the Kabsch-Umeyama affine matrix and warps the target face
    // image.
    fn crop_and_align(
        image: &Rgba32FImage,
        keypoints: &[(f32, f32); 5],
        size: u32,
    ) -> Rgba32FImage {
        let matrix = umeyama(keypoints, &ARCFACE_DST);
        let mut output = Rgba32FImage::new(size, size);
        warp_into(image, matrix, &mut output);
        output
    }
}

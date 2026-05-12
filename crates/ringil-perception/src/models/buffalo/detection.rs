//! Detection logic for Buffalo.

use ndarray::ArrayViewD;

use crate::models::buffalo::types::Face;

// Decodes network output distances into bounding box coordinates (x1, y1, x2,
// y2).
pub fn distance2bbox(
    index: usize,
    stride: usize,
    distance: &ArrayViewD<f32>,
) -> (f32, f32, f32, f32) {
    let m = 640 / stride;
    let cx = ((index / 2) * stride) % 640;
    let cy = (((index / 2) / m) * stride) % 640;

    let stride_f = stride as f32;
    let x1 = cx as f32 - distance[[index, 0]] * stride_f;
    let y1 = cy as f32 - distance[[index, 1]] * stride_f;
    let x2 = cx as f32 + distance[[index, 2]] * stride_f;
    let y2 = cy as f32 + distance[[index, 3]] * stride_f;

    (x1, y1, x2, y2)
}

// Decodes network output distances into 5 facial landmark coordinates.
pub fn distance2kps(
    index: usize,
    stride: usize,
    distance: &ArrayViewD<f32>,
) -> [(f32, f32); 5] {
    let m = 640 / stride;
    let cx = ((index / 2) * stride) % 640;
    let cy = (((index / 2) / m) * stride) % 640;

    let stride_f = stride as f32;
    let mut kps = [(0.0, 0.0); 5];

    for i in 0..5 {
        kps[i] = (
            cx as f32 + distance[[index, i * 2]] * stride_f,
            cy as f32 + distance[[index, i * 2 + 1]] * stride_f,
        );
    }

    kps
}

// O(N^2) Non-Maximum Suppression using an in-place suppression mask.
// Significantly faster than removing elements from a vector sequentially.
pub fn nms(mut faces: Vec<Face>, iou_threshold: f32) -> Vec<Face> {
    faces.sort_unstable_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

    let mut keep = Vec::with_capacity(faces.len());
    let mut suppressed = vec![false; faces.len()];

    for i in 0..faces.len() {
        if suppressed[i] {
            continue;
        }
        let f1 = &faces[i];
        keep.push(*f1);

        let area1 =
            (f1.bbox.2 - f1.bbox.0 + 1.0) * (f1.bbox.3 - f1.bbox.1 + 1.0);

        for j in (i + 1)..faces.len() {
            if suppressed[j] {
                continue;
            }
            let f2 = &faces[j];
            let area2 =
                (f2.bbox.2 - f2.bbox.0 + 1.0) * (f2.bbox.3 - f2.bbox.1 + 1.0);

            // Compute Intersection over Union (IoU)
            let xx1 = f1.bbox.0.max(f2.bbox.0);
            let yy1 = f1.bbox.1.max(f2.bbox.1);
            let xx2 = f1.bbox.2.min(f2.bbox.2);
            let yy2 = f1.bbox.3.min(f2.bbox.3);

            let w = (xx2 - xx1 + 1.0).max(0.0);
            let h = (yy2 - yy1 + 1.0).max(0.0);
            let inter = w * h;
            let iou = inter / (area1 + area2 - inter);

            if iou > iou_threshold {
                suppressed[j] = true;
            }
        }
    }

    keep
}

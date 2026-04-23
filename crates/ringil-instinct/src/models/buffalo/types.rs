//! Types for InsightFace.

// Standard ArcFace 112x112 destination points for alignment.
pub const ARCFACE_DST: [(f32, f32); 5] = [
    (38.2946, 51.6963),
    (73.5318, 51.5014),
    (56.0252, 71.7366),
    (41.5493, 92.3655),
    (70.7299, 92.2041),
];

#[derive(Debug, Clone, Copy)]
pub struct Face {
    pub score: f32,
    pub bbox: (f32, f32, f32, f32), // (x1, y1, x2, y2)
    pub keypoints: [(f32, f32); 5],
}

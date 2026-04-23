use anyhow::Result;
use image::DynamicImage;

/// Standard trait for camera inputs (GStreamer, V4L2, Dummy).
pub trait Camera: Send + Sync {
    /// Captures a single frame. Blocks or awaits until frame is ready.
    fn capture_frame(&mut self) -> Result<DynamicImage>;
}

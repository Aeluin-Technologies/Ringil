//! Image-related functions.

use anyhow::{Context, Result};
use fast_image_resize::images::Image;
use fast_image_resize::{PixelType, ResizeOptions, Resizer};
use image::{DynamicImage, RgbImage, Rgba32FImage};
use ndarray::Array4;

// High-performance image resize using CPU SIMD instructions via
// fast_image_resize.
pub fn fast_resize(
    img: &DynamicImage,
    width: u32,
    height: u32,
) -> Result<RgbImage> {
    let rgb = img.to_rgb8();
    let src_image = Image::from_vec_u8(
        rgb.width(),
        rgb.height(),
        rgb.into_raw(),
        PixelType::U8x3,
    )?;

    let mut dst_image = Image::new(width, height, PixelType::U8x3);
    let mut resizer = Resizer::new();

    resizer.resize(&src_image, &mut dst_image, &ResizeOptions::default())?;

    RgbImage::from_raw(width, height, dst_image.into_vec())
        .context("Failed to reconstruct RgbImage after resize")
}

// Converts RgbImage directly into a normalized NHWC Float32 tensor for ONNX.
// Avoids intermediate image allocations.
pub fn image_to_tensor_rgb(img: &RgbImage) -> Array4<f32> {
    let (width, height) = img.dimensions();
    let mut tensor =
        Array4::<f32>::zeros((1, 3, height as usize, width as usize));

    for (x, y, pixel) in img.enumerate_pixels() {
        for c in 0..3 {
            // Normalize to [-1.0, 1.0] expected by InsightFace models
            tensor[[0, c, y as usize, x as usize]] =
                (pixel[c] as f32 - 127.5) / 127.5;
        }
    }
    tensor
}

// Converts a cropped Rgba32FImage to an NHWC tensor.
pub fn image_to_tensor_rgba32f(img: &Rgba32FImage) -> Array4<f32> {
    let (width, height) = img.dimensions();
    let mut tensor =
        Array4::<f32>::zeros((1, 3, height as usize, width as usize));

    for (x, y, pixel) in img.enumerate_pixels() {
        for c in 0..3 {
            tensor[[0, c, y as usize, x as usize]] = (pixel[c] - 0.5) / 0.5;
        }
    }
    tensor
}

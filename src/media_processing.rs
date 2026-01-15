//! Image processing utilities for media uploads.
//!
//! Handles resizing, format conversion, and blurhash generation.

use image::{DynamicImage, GenericImageView, ImageFormat, imageops::FilterType};
use std::io::Cursor;

/// Target widths for image variants
pub const THUMB_WIDTH: u32 = 300;
pub const MEDIUM_WIDTH: u32 = 800;
pub const FULL_WIDTH: u32 = 1600;

/// Quality setting for WebP encoding (0-100)
const WEBP_QUALITY: u8 = 85;

/// Result of processing an uploaded image
#[derive(Debug)]
pub struct ProcessedImage {
    pub thumb: ImageVariant,
    pub medium: ImageVariant,
    pub full: ImageVariant,
    pub original: OriginalImage,
    pub blurhash: String,
}

/// A processed image variant
#[derive(Debug)]
pub struct ImageVariant {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

/// Original image info
#[derive(Debug)]
pub struct OriginalImage {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub mime: String,
}

/// Errors that can occur during image processing
#[derive(Debug, thiserror::Error)]
pub enum ProcessingError {
    #[error("Failed to decode image: {0}")]
    DecodeError(String),

    #[error("Failed to encode image: {0}")]
    EncodeError(String),

    #[error("Unsupported image format: {0}")]
    UnsupportedFormat(String),

    #[error("Image too small: minimum {min}px, got {actual}px")]
    TooSmall { min: u32, actual: u32 },

    #[error("BlurHash generation failed: {0}")]
    BlurHashError(String),
}

/// Detect image format from bytes and filename
pub fn detect_format(
    data: &[u8],
    filename: &str,
) -> Result<(ImageFormat, &'static str), ProcessingError> {
    // Try to detect from magic bytes first
    if let Ok(format) = image::guess_format(data) {
        let mime = match format {
            ImageFormat::Jpeg => "image/jpeg",
            ImageFormat::Png => "image/png",
            ImageFormat::Gif => "image/gif",
            ImageFormat::WebP => "image/webp",
            ImageFormat::Avif => "image/avif",
            _ => return Err(ProcessingError::UnsupportedFormat(format!("{format:?}"))),
        };
        return Ok((format, mime));
    }

    // Fall back to extension
    let ext = filename.rsplit('.').next().unwrap_or("").to_lowercase();

    match ext.as_str() {
        "jpg" | "jpeg" => Ok((ImageFormat::Jpeg, "image/jpeg")),
        "png" => Ok((ImageFormat::Png, "image/png")),
        "gif" => Ok((ImageFormat::Gif, "image/gif")),
        "webp" => Ok((ImageFormat::WebP, "image/webp")),
        "avif" => Ok((ImageFormat::Avif, "image/avif")),
        _ => Err(ProcessingError::UnsupportedFormat(ext)),
    }
}

/// Process an uploaded image into all required variants
pub fn process_image(data: &[u8], filename: &str) -> Result<ProcessedImage, ProcessingError> {
    // Detect format and decode
    let (format, mime) = detect_format(data, filename)?;
    let img = image::load_from_memory_with_format(data, format)
        .map_err(|e| ProcessingError::DecodeError(e.to_string()))?;

    let (orig_width, orig_height) = img.dimensions();

    // Minimum size check - at least thumbnail size
    let min_dim = orig_width.min(orig_height);
    if min_dim < THUMB_WIDTH {
        return Err(ProcessingError::TooSmall {
            min: THUMB_WIDTH,
            actual: min_dim,
        });
    }

    // Generate variants (only resize if larger than target)
    let thumb = resize_to_webp(&img, THUMB_WIDTH)?;
    let medium = resize_to_webp(&img, MEDIUM_WIDTH)?;
    let full = resize_to_webp(&img, FULL_WIDTH)?;

    // Generate blurhash from thumbnail for efficiency
    let blurhash = generate_blurhash(&img, 4, 3)?;

    // Keep original as-is (preserve format)
    let original = OriginalImage {
        data: data.to_vec(),
        width: orig_width,
        height: orig_height,
        mime: mime.to_string(),
    };

    Ok(ProcessedImage {
        thumb,
        medium,
        full,
        original,
        blurhash,
    })
}

/// Resize image to target width (maintaining aspect ratio) and encode as WebP
fn resize_to_webp(img: &DynamicImage, target_width: u32) -> Result<ImageVariant, ProcessingError> {
    let (orig_width, orig_height) = img.dimensions();

    // Only resize if larger than target
    let (resized, width, height) = if orig_width > target_width {
        let ratio = target_width as f64 / orig_width as f64;
        let target_height = (orig_height as f64 * ratio).round() as u32;
        let resized = img.resize(target_width, target_height, FilterType::Lanczos3);
        (resized, target_width, target_height)
    } else {
        (img.clone(), orig_width, orig_height)
    };

    // Encode as WebP
    let mut buf = Cursor::new(Vec::new());
    resized
        .write_to(&mut buf, ImageFormat::WebP)
        .map_err(|e| ProcessingError::EncodeError(e.to_string()))?;

    Ok(ImageVariant {
        data: buf.into_inner(),
        width,
        height,
    })
}

/// Generate a BlurHash string from an image
fn generate_blurhash(
    img: &DynamicImage,
    x_components: u32,
    y_components: u32,
) -> Result<String, ProcessingError> {
    // Resize to small size for efficient blurhash computation
    let small = img.resize(32, 32, FilterType::Triangle);
    let rgba = small.to_rgba8();
    let (w, h) = rgba.dimensions();

    let hash = blurhash::encode(x_components, y_components, w, h, rgba.as_raw())
        .map_err(|e| ProcessingError::BlurHashError(format!("{e:?}")))?;

    Ok(hash)
}

/// Check if a MIME type is a supported image format
pub fn is_supported_image(mime: &str) -> bool {
    matches!(
        mime,
        "image/jpeg" | "image/png" | "image/gif" | "image/webp" | "image/avif"
    )
}

/// Check if a MIME type is a supported video format
pub fn is_supported_video(mime: &str) -> bool {
    matches!(mime, "video/mp4" | "video/webm" | "video/quicktime")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_format_jpeg() {
        // JPEG magic bytes
        let data = [0xFF, 0xD8, 0xFF, 0xE0];
        let (format, mime) = detect_format(&data, "test.jpg").unwrap();
        assert_eq!(format, ImageFormat::Jpeg);
        assert_eq!(mime, "image/jpeg");
    }

    #[test]
    fn test_detect_format_png() {
        // PNG magic bytes
        let data = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        let (format, mime) = detect_format(&data, "test.png").unwrap();
        assert_eq!(format, ImageFormat::Png);
        assert_eq!(mime, "image/png");
    }

    #[test]
    fn test_is_supported_image() {
        assert!(is_supported_image("image/jpeg"));
        assert!(is_supported_image("image/png"));
        assert!(!is_supported_image("text/plain"));
        assert!(!is_supported_image("video/mp4"));
    }

    #[test]
    fn test_is_supported_video() {
        assert!(is_supported_video("video/mp4"));
        assert!(is_supported_video("video/webm"));
        assert!(!is_supported_video("image/jpeg"));
    }
}

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Image input and output types for vision models.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use anyhow::{Context, Result};

/// Image format types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ImageFormat {
    /// JPEG format
    Jpeg,
    /// PNG format
    Png,
    /// WebP format
    WebP,
    /// GIF format
    Gif,
    /// BMP format
    Bmp,
}

impl ImageFormat {
    /// Returns the MIME type for this format
    pub fn mime_type(&self) -> &'static str {
        match self {
            ImageFormat::Jpeg => "image/jpeg",
            ImageFormat::Png => "image/png",
            ImageFormat::WebP => "image/webp",
            ImageFormat::Gif => "image/gif",
            ImageFormat::Bmp => "image/bmp",
        }
    }

    /// Detects format from file extension
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "jpg" | "jpeg" => Some(ImageFormat::Jpeg),
            "png" => Some(ImageFormat::Png),
            "webp" => Some(ImageFormat::WebP),
            "gif" => Some(ImageFormat::Gif),
            "bmp" => Some(ImageFormat::Bmp),
            _ => None,
        }
    }
}

/// Image quality level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ImageQuality {
    /// Low quality (faster, smaller)
    Low,
    /// Medium quality
    Medium,
    /// High quality (slower, larger)
    High,
}

/// Image dimensions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImageDimensions {
    /// Width in pixels
    pub width: u32,
    /// Height in pixels
    pub height: u32,
}

impl ImageDimensions {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    /// Returns the total number of pixels
    pub fn pixels(&self) -> u64 {
        self.width as u64 * self.height as u64
    }

    /// Returns the aspect ratio (width/height)
    pub fn aspect_ratio(&self) -> f64 {
        self.width as f64 / self.height as f64
    }

    /// Checks if this is a square image
    pub fn is_square(&self) -> bool {
        self.width == self.height
    }

    /// Scales dimensions to fit within max size while preserving aspect ratio
    pub fn scale_to_fit(&self, max_width: u32, max_height: u32) -> Self {
        if self.width <= max_width && self.height <= max_height {
            return *self;
        }

        let width_scale = max_width as f64 / self.width as f64;
        let height_scale = max_height as f64 / self.height as f64;
        let scale = width_scale.min(height_scale);

        Self {
            width: (self.width as f64 * scale) as u32,
            height: (self.height as f64 * scale) as u32,
        }
    }
}

/// Image metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageMetadata {
    /// Image format
    pub format: ImageFormat,
    /// Image dimensions
    pub dimensions: ImageDimensions,
    /// File size in bytes
    pub size_bytes: usize,
    /// Optional description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Image input for vision models
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "source", rename_all = "lowercase")]
pub enum ImageInput {
    /// Base64-encoded image data
    Base64 {
        /// Base64-encoded image
        data: String,
        /// Media type (e.g., "image/jpeg")
        media_type: String,
        /// Optional metadata
        #[serde(skip_serializing_if = "Option::is_none")]
        metadata: Option<ImageMetadata>,
    },
    /// URL to an image
    Url {
        /// Image URL
        url: String,
        /// Optional metadata
        #[serde(skip_serializing_if = "Option::is_none")]
        metadata: Option<ImageMetadata>,
    },
    /// Local file path
    Path {
        /// File path
        path: PathBuf,
        /// Optional metadata
        #[serde(skip_serializing_if = "Option::is_none")]
        metadata: Option<ImageMetadata>,
    },
}

impl ImageInput {
    /// Creates an image input from a base64 string
    pub fn from_base64(data: String, format: ImageFormat) -> Self {
        Self::Base64 {
            data,
            media_type: format.mime_type().to_string(),
            metadata: None,
        }
    }

    /// Creates an image input from a URL
    pub fn from_url(url: impl Into<String>) -> Self {
        Self::Url {
            url: url.into(),
            metadata: None,
        }
    }

    /// Creates an image input from a file path
    pub async fn from_path(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();

        // Validate file exists
        if !path.exists() {
            anyhow::bail!("Image file does not exist: {}", path.display());
        }

        // Detect format from extension
        let format = path.extension()
            .and_then(|ext| ext.to_str())
            .and_then(ImageFormat::from_extension)
            .context("Could not determine image format from file extension")?;

        // Get file size
        let metadata = tokio::fs::metadata(path).await
            .context("Failed to read file metadata")?;

        let size_bytes = metadata.len() as usize;

        Ok(Self::Path {
            path: path.to_path_buf(),
            metadata: Some(ImageMetadata {
                format,
                dimensions: ImageDimensions::new(0, 0), // Unknown until we decode
                size_bytes,
                description: None,
            }),
        })
    }

    /// Reads the image data as bytes
    pub async fn read_bytes(&self) -> Result<Vec<u8>> {
        match self {
            ImageInput::Base64 { data, .. } => {
                use base64::{Engine as _, engine::general_purpose};
                general_purpose::STANDARD.decode(data)
                    .context("Failed to decode base64 image data")
            }
            ImageInput::Url { url, .. } => {
                let response = reqwest::get(url).await
                    .context("Failed to fetch image from URL")?;

                if !response.status().is_success() {
                    anyhow::bail!("Failed to fetch image: HTTP {}", response.status());
                }

                response.bytes().await
                    .map(|b| b.to_vec())
                    .context("Failed to read image bytes from response")
            }
            ImageInput::Path { path, .. } => {
                tokio::fs::read(path).await
                    .context("Failed to read image file")
            }
        }
    }

    /// Converts to base64 encoding
    pub async fn to_base64(&self) -> Result<String> {
        use base64::{Engine as _, engine::general_purpose};

        match self {
            ImageInput::Base64 { data, .. } => Ok(data.clone()),
            _ => {
                let bytes = self.read_bytes().await?;
                Ok(general_purpose::STANDARD.encode(&bytes))
            }
        }
    }

    /// Returns the media type
    pub fn media_type(&self) -> Option<&str> {
        match self {
            ImageInput::Base64 { media_type, .. } => Some(media_type),
            ImageInput::Url { metadata, .. } | ImageInput::Path { metadata, .. } => {
                metadata.as_ref().map(|m| m.format.mime_type())
            }
        }
    }
}

/// Image output from generation models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageOutput {
    /// Image data (base64 or URL)
    pub data: String,

    /// Format of the image
    pub format: ImageFormat,

    /// Dimensions if known
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dimensions: Option<ImageDimensions>,

    /// Generation parameters used
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<ImageGenerationParameters>,

    /// Revised prompt (if modified by safety)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revised_prompt: Option<String>,
}

impl ImageOutput {
    pub fn new(data: String, format: ImageFormat) -> Self {
        Self {
            data,
            format,
            dimensions: None,
            parameters: None,
            revised_prompt: None,
        }
    }

    /// Saves the image to a file
    pub async fn save_to_file(&self, path: impl AsRef<Path>) -> Result<()> {
        use base64::{Engine as _, engine::general_purpose};

        // Decode base64 if needed
        let bytes = if self.data.starts_with("http") {
            // Download from URL
            let response = reqwest::get(&self.data).await?;
            response.bytes().await?.to_vec()
        } else {
            // Decode base64
            general_purpose::STANDARD.decode(&self.data)?
        };

        tokio::fs::write(path.as_ref(), &bytes).await?;
        Ok(())
    }
}

/// Parameters for image generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageGenerationParameters {
    /// Prompt used for generation
    pub prompt: String,

    /// Negative prompt (what to avoid)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub negative_prompt: Option<String>,

    /// Image size
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<String>,

    /// Quality level
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quality: Option<ImageQuality>,

    /// Style preset
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<String>,

    /// Number of inference steps
    #[serde(skip_serializing_if = "Option::is_none")]
    pub steps: Option<u32>,

    /// Guidance scale
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guidance_scale: Option<f32>,

    /// Random seed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_format_mime() {
        assert_eq!(ImageFormat::Jpeg.mime_type(), "image/jpeg");
        assert_eq!(ImageFormat::Png.mime_type(), "image/png");
        assert_eq!(ImageFormat::WebP.mime_type(), "image/webp");
    }

    #[test]
    fn test_image_format_from_extension() {
        assert_eq!(ImageFormat::from_extension("jpg"), Some(ImageFormat::Jpeg));
        assert_eq!(ImageFormat::from_extension("PNG"), Some(ImageFormat::Png));
        assert_eq!(ImageFormat::from_extension("webp"), Some(ImageFormat::WebP));
        assert_eq!(ImageFormat::from_extension("unknown"), None);
    }

    #[test]
    fn test_image_dimensions() {
        let dims = ImageDimensions::new(1920, 1080);
        assert_eq!(dims.pixels(), 2073600);
        assert!((dims.aspect_ratio() - 16.0/9.0).abs() < 0.01);
        assert!(!dims.is_square());

        let square = ImageDimensions::new(512, 512);
        assert!(square.is_square());
    }

    #[test]
    fn test_image_dimensions_scale() {
        let dims = ImageDimensions::new(1920, 1080);
        let scaled = dims.scale_to_fit(960, 540);

        assert_eq!(scaled.width, 960);
        assert_eq!(scaled.height, 540);
    }

    #[test]
    fn test_image_input_from_base64() {
        let input = ImageInput::from_base64("abc123".to_string(), ImageFormat::Png);

        match input {
            ImageInput::Base64 { data, media_type, .. } => {
                assert_eq!(data, "abc123");
                assert_eq!(media_type, "image/png");
            }
            _ => panic!("Expected Base64 variant"),
        }
    }

    #[test]
    fn test_image_input_from_url() {
        let input = ImageInput::from_url("https://example.com/image.jpg");

        match input {
            ImageInput::Url { url, .. } => {
                assert_eq!(url, "https://example.com/image.jpg");
            }
            _ => panic!("Expected Url variant"),
        }
    }

    #[test]
    fn test_image_output() {
        let output = ImageOutput::new("base64data".to_string(), ImageFormat::Png);
        assert_eq!(output.format, ImageFormat::Png);
        assert_eq!(output.data, "base64data");
    }
}

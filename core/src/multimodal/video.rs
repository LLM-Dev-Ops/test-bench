// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Video input and output types for video understanding models.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use anyhow::{Context, Result};

use super::image::{ImageDimensions, ImageFormat};
use super::audio::AudioDuration;

/// Video format types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VideoFormat {
    /// MP4 format
    Mp4,
    /// WebM format
    WebM,
    /// AVI format
    Avi,
    /// MOV format
    Mov,
    /// MKV format
    Mkv,
}

impl VideoFormat {
    pub fn mime_type(&self) -> &'static str {
        match self {
            VideoFormat::Mp4 => "video/mp4",
            VideoFormat::WebM => "video/webm",
            VideoFormat::Avi => "video/x-msvideo",
            VideoFormat::Mov => "video/quicktime",
            VideoFormat::Mkv => "video/x-matroska",
        }
    }

    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "mp4" => Some(VideoFormat::Mp4),
            "webm" => Some(VideoFormat::WebM),
            "avi" => Some(VideoFormat::Avi),
            "mov" => Some(VideoFormat::Mov),
            "mkv" => Some(VideoFormat::Mkv),
            _ => None,
        }
    }
}

/// Video codec types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VideoCodec {
    /// H.264/AVC
    H264,
    /// H.265/HEVC
    H265,
    /// VP8
    Vp8,
    /// VP9
    Vp9,
    /// AV1
    Av1,
}

/// Video metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoMetadata {
    /// Video format
    pub format: VideoFormat,

    /// Video dimensions
    pub dimensions: ImageDimensions,

    /// Frame rate (FPS)
    pub fps: f32,

    /// Duration
    pub duration: AudioDuration,

    /// Video codec
    #[serde(skip_serializing_if = "Option::is_none")]
    pub codec: Option<VideoCodec>,

    /// Bitrate in kbps
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bitrate: Option<u32>,

    /// File size in bytes
    pub size_bytes: usize,

    /// Has audio track
    #[serde(default)]
    pub has_audio: bool,
}

/// A single frame from a video
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoFrame {
    /// Frame number (0-indexed)
    pub frame_number: u32,

    /// Timestamp in seconds
    pub timestamp: f64,

    /// Frame data (base64-encoded image)
    pub data: String,

    /// Image format
    pub format: ImageFormat,

    /// Frame dimensions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dimensions: Option<ImageDimensions>,
}

impl VideoFrame {
    pub fn new(frame_number: u32, timestamp: f64, data: String, format: ImageFormat) -> Self {
        Self {
            frame_number,
            timestamp,
            data,
            format,
            dimensions: None,
        }
    }
}

/// Video input for video understanding models
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "source", rename_all = "lowercase")]
pub enum VideoInput {
    /// Base64-encoded video data
    Base64 {
        /// Base64-encoded video
        data: String,
        /// Media type
        media_type: String,
        /// Optional metadata
        #[serde(skip_serializing_if = "Option::is_none")]
        metadata: Option<VideoMetadata>,
    },
    /// URL to a video
    Url {
        /// Video URL
        url: String,
        /// Optional metadata
        #[serde(skip_serializing_if = "Option::is_none")]
        metadata: Option<VideoMetadata>,
    },
    /// Local file path
    Path {
        /// File path
        path: PathBuf,
        /// Optional metadata
        #[serde(skip_serializing_if = "Option::is_none")]
        metadata: Option<VideoMetadata>,
    },
    /// Sequence of frames
    Frames {
        /// List of video frames
        frames: Vec<VideoFrame>,
        /// Frame rate
        fps: f32,
        /// Optional metadata
        #[serde(skip_serializing_if = "Option::is_none")]
        metadata: Option<VideoMetadata>,
    },
}

impl VideoInput {
    pub fn from_url(url: impl Into<String>) -> Self {
        Self::Url {
            url: url.into(),
            metadata: None,
        }
    }

    pub async fn from_path(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();

        if !path.exists() {
            anyhow::bail!("Video file does not exist: {}", path.display());
        }

        let format = path.extension()
            .and_then(|ext| ext.to_str())
            .and_then(VideoFormat::from_extension)
            .context("Could not determine video format from file extension")?;

        let metadata = tokio::fs::metadata(path).await?;
        let size_bytes = metadata.len() as usize;

        Ok(Self::Path {
            path: path.to_path_buf(),
            metadata: Some(VideoMetadata {
                format,
                dimensions: ImageDimensions::new(0, 0),
                fps: 0.0,
                duration: AudioDuration::new(0, 0),
                codec: None,
                bitrate: None,
                size_bytes,
                has_audio: false,
            }),
        })
    }

    pub fn from_frames(frames: Vec<VideoFrame>, fps: f32) -> Self {
        Self::Frames {
            frames,
            fps,
            metadata: None,
        }
    }
}

/// Video output from generation models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoOutput {
    /// Video data (base64 or URL)
    pub data: String,

    /// Format
    pub format: VideoFormat,

    /// Metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<VideoMetadata>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_video_format() {
        assert_eq!(VideoFormat::Mp4.mime_type(), "video/mp4");
        assert_eq!(VideoFormat::from_extension("mp4"), Some(VideoFormat::Mp4));
    }

    #[test]
    fn test_video_frame() {
        let frame = VideoFrame::new(0, 0.0, "data".to_string(), ImageFormat::Jpeg);
        assert_eq!(frame.frame_number, 0);
        assert_eq!(frame.timestamp, 0.0);
    }
}

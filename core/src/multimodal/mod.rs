// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Multi-modal support for vision, audio, and video inputs/outputs.
//!
//! This module provides comprehensive multi-modal capabilities including:
//! - Vision: Image understanding, generation, and evaluation
//! - Audio: Speech-to-text, text-to-speech, and audio analysis
//! - Video: Video understanding and frame analysis
//!
//! # Architecture
//!
//! The multi-modal system is built on several core abstractions:
//!
//! - **Media Types**: Image, Audio, Video representations
//! - **Multi-modal Requests**: Combined text + media inputs
//! - **Multi-modal Responses**: Text and/or media outputs
//! - **Evaluation Metrics**: Specialized metrics for each modality
//!
//! # Examples
//!
//! ## Vision Understanding
//!
//! ```no_run
//! use llm_test_bench_core::multimodal::{ImageInput, MultiModalRequest};
//! use llm_test_bench_core::providers::Provider;
//!
//! # async fn example(provider: Box<dyn Provider>) -> Result<(), Box<dyn std::error::Error>> {
//! // Load image
//! let image = ImageInput::from_path("cat.jpg").await?;
//!
//! // Create multi-modal request
//! let request = MultiModalRequest::new()
//!     .with_text("What's in this image?")
//!     .with_image(image);
//!
//! // Process with provider
//! let response = provider.complete_multimodal(request).await?;
//! println!("Response: {}", response.text);
//! # Ok(())
//! # }
//! ```
//!
//! ## Audio Transcription
//!
//! ```no_run
//! use llm_test_bench_core::multimodal::{AudioInput, MultiModalRequest};
//!
//! # async fn example(provider: Box<dyn Provider>) -> Result<(), Box<dyn std::error::Error>> {
//! // Load audio
//! let audio = AudioInput::from_path("speech.mp3").await?;
//!
//! // Create transcription request
//! let request = MultiModalRequest::new()
//!     .with_audio(audio);
//!
//! let response = provider.complete_multimodal(request).await?;
//! println!("Transcription: {}", response.text);
//! # Ok(())
//! # }
//! ```

pub mod types;
pub mod image;
pub mod audio;
pub mod video;
pub mod evaluation;
pub mod datasets;

// Re-export commonly used types
pub use types::{
    MediaType, MultiModalRequest, MultiModalResponse, MultiModalContent,
    ContentPart, ImagePart, AudioPart, VideoPart, TextPart,
};

pub use image::{
    ImageInput, ImageOutput, ImageFormat, ImageQuality,
    ImageMetadata, ImageDimensions,
};

pub use audio::{
    AudioInput, AudioOutput, AudioFormat, AudioCodec,
    AudioMetadata, AudioDuration,
};

pub use video::{
    VideoInput, VideoOutput, VideoFormat, VideoCodec,
    VideoMetadata, VideoFrame,
};

pub use evaluation::{
    VisionEvaluator, AudioEvaluator, MultiModalEvaluator,
    VisionMetrics, AudioMetrics, MultiModalMetrics,
};

pub use datasets::{
    MultiModalDataset, MultiModalExample, MultiModalTask,
    VisionTask, AudioTask,
};

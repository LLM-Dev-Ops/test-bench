// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Core types for multi-modal requests and responses.

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

use super::image::{ImageInput, ImageOutput};
use super::audio::{AudioInput, AudioOutput};
use super::video::{VideoInput, VideoOutput};

/// Media type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MediaType {
    /// Text content
    Text,
    /// Image content
    Image,
    /// Audio content
    Audio,
    /// Video content
    Video,
}

impl std::fmt::Display for MediaType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MediaType::Text => write!(f, "text"),
            MediaType::Image => write!(f, "image"),
            MediaType::Audio => write!(f, "audio"),
            MediaType::Video => write!(f, "video"),
        }
    }
}

/// A single content part in a multi-modal message
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ContentPart {
    /// Plain text content
    Text(TextPart),
    /// Image content
    Image(ImagePart),
    /// Audio content
    Audio(AudioPart),
    /// Video content
    Video(VideoPart),
}

impl ContentPart {
    /// Returns the media type of this content part
    pub fn media_type(&self) -> MediaType {
        match self {
            ContentPart::Text(_) => MediaType::Text,
            ContentPart::Image(_) => MediaType::Image,
            ContentPart::Audio(_) => MediaType::Audio,
            ContentPart::Video(_) => MediaType::Video,
        }
    }

    /// Returns true if this is a text part
    pub fn is_text(&self) -> bool {
        matches!(self, ContentPart::Text(_))
    }

    /// Returns true if this is an image part
    pub fn is_image(&self) -> bool {
        matches!(self, ContentPart::Image(_))
    }

    /// Returns true if this is an audio part
    pub fn is_audio(&self) -> bool {
        matches!(self, ContentPart::Audio(_))
    }

    /// Returns true if this is a video part
    pub fn is_video(&self) -> bool {
        matches!(self, ContentPart::Video(_))
    }
}

/// Text content part
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TextPart {
    /// The text content
    pub text: String,
}

impl TextPart {
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }
}

/// Image content part
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImagePart {
    /// The image input
    #[serde(flatten)]
    pub image: ImageInput,
    /// Optional detail level for vision models
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<ImageDetail>,
}

impl ImagePart {
    pub fn new(image: ImageInput) -> Self {
        Self { image, detail: None }
    }

    pub fn with_detail(mut self, detail: ImageDetail) -> Self {
        self.detail = Some(detail);
        self
    }
}

/// Detail level for image processing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ImageDetail {
    /// Low detail (faster, cheaper)
    Low,
    /// High detail (slower, more expensive)
    High,
    /// Automatic selection
    Auto,
}

/// Audio content part
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioPart {
    /// The audio input
    #[serde(flatten)]
    pub audio: AudioInput,
    /// Optional language hint for transcription
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
}

impl AudioPart {
    pub fn new(audio: AudioInput) -> Self {
        Self { audio, language: None }
    }

    pub fn with_language(mut self, language: impl Into<String>) -> Self {
        self.language = Some(language.into());
        self
    }
}

/// Video content part
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoPart {
    /// The video input
    #[serde(flatten)]
    pub video: VideoInput,
    /// Optional frame sampling rate
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frame_rate: Option<f32>,
}

impl VideoPart {
    pub fn new(video: VideoInput) -> Self {
        Self { video, frame_rate: None }
    }

    pub fn with_frame_rate(mut self, frame_rate: f32) -> Self {
        self.frame_rate = Some(frame_rate);
        self
    }
}

/// Multi-modal content that can contain mixed media types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiModalContent {
    /// List of content parts
    pub parts: Vec<ContentPart>,
}

impl MultiModalContent {
    /// Creates a new empty multi-modal content
    pub fn new() -> Self {
        Self { parts: Vec::new() }
    }

    /// Adds a text part
    pub fn add_text(&mut self, text: impl Into<String>) {
        self.parts.push(ContentPart::Text(TextPart::new(text)));
    }

    /// Adds an image part
    pub fn add_image(&mut self, image: ImageInput) {
        self.parts.push(ContentPart::Image(ImagePart::new(image)));
    }

    /// Adds an audio part
    pub fn add_audio(&mut self, audio: AudioInput) {
        self.parts.push(ContentPart::Audio(AudioPart::new(audio)));
    }

    /// Adds a video part
    pub fn add_video(&mut self, video: VideoInput) {
        self.parts.push(ContentPart::Video(VideoPart::new(video)));
    }

    /// Returns all text parts concatenated
    pub fn extract_text(&self) -> String {
        self.parts.iter()
            .filter_map(|part| {
                if let ContentPart::Text(text_part) = part {
                    Some(text_part.text.as_str())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Returns all image parts
    pub fn extract_images(&self) -> Vec<&ImageInput> {
        self.parts.iter()
            .filter_map(|part| {
                if let ContentPart::Image(image_part) = part {
                    Some(&image_part.image)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Returns all audio parts
    pub fn extract_audio(&self) -> Vec<&AudioInput> {
        self.parts.iter()
            .filter_map(|part| {
                if let ContentPart::Audio(audio_part) = part {
                    Some(&audio_part.audio)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Returns the count of each media type
    pub fn media_type_counts(&self) -> (usize, usize, usize, usize) {
        let mut text_count = 0;
        let mut image_count = 0;
        let mut audio_count = 0;
        let mut video_count = 0;

        for part in &self.parts {
            match part.media_type() {
                MediaType::Text => text_count += 1,
                MediaType::Image => image_count += 1,
                MediaType::Audio => audio_count += 1,
                MediaType::Video => video_count += 1,
            }
        }

        (text_count, image_count, audio_count, video_count)
    }
}

impl Default for MultiModalContent {
    fn default() -> Self {
        Self::new()
    }
}

/// A multi-modal request combining text, images, audio, and video
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiModalRequest {
    /// The model to use
    pub model: String,

    /// Multi-modal content
    pub content: MultiModalContent,

    /// Optional system message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,

    /// Maximum tokens to generate
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<usize>,

    /// Sampling temperature
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,

    /// Nucleus sampling parameter
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,

    /// Stop sequences
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<String>>,

    /// Whether to stream the response
    #[serde(default)]
    pub stream: bool,
}

impl MultiModalRequest {
    /// Creates a new multi-modal request
    pub fn new(model: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            content: MultiModalContent::new(),
            system: None,
            max_tokens: None,
            temperature: None,
            top_p: None,
            stop: None,
            stream: false,
        }
    }

    /// Adds text to the request
    pub fn with_text(mut self, text: impl Into<String>) -> Self {
        self.content.add_text(text);
        self
    }

    /// Adds an image to the request
    pub fn with_image(mut self, image: ImageInput) -> Self {
        self.content.add_image(image);
        self
    }

    /// Adds audio to the request
    pub fn with_audio(mut self, audio: AudioInput) -> Self {
        self.content.add_audio(audio);
        self
    }

    /// Adds video to the request
    pub fn with_video(mut self, video: VideoInput) -> Self {
        self.content.add_video(video);
        self
    }

    /// Sets the system message
    pub fn with_system(mut self, system: impl Into<String>) -> Self {
        self.system = Some(system.into());
        self
    }

    /// Sets max tokens
    pub fn with_max_tokens(mut self, max_tokens: usize) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    /// Sets temperature
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }

    /// Sets top_p
    pub fn with_top_p(mut self, top_p: f32) -> Self {
        self.top_p = Some(top_p);
        self
    }

    /// Enables streaming
    pub fn with_streaming(mut self) -> Self {
        self.stream = true;
        self
    }

    /// Returns true if this request contains only text
    pub fn is_text_only(&self) -> bool {
        let (text, image, audio, video) = self.content.media_type_counts();
        text > 0 && image == 0 && audio == 0 && video == 0
    }

    /// Returns true if this request contains images
    pub fn has_images(&self) -> bool {
        let (_, image, _, _) = self.content.media_type_counts();
        image > 0
    }

    /// Returns true if this request contains audio
    pub fn has_audio(&self) -> bool {
        let (_, _, audio, _) = self.content.media_type_counts();
        audio > 0
    }

    /// Returns true if this request contains video
    pub fn has_video(&self) -> bool {
        let (_, _, _, video) = self.content.media_type_counts();
        video > 0
    }
}

/// A multi-modal response that may contain text, images, audio, or video
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiModalResponse {
    /// Response ID
    pub id: String,

    /// The model that generated this response
    pub model: String,

    /// Generated content
    pub content: MultiModalContent,

    /// Token usage information
    pub usage: MultiModalUsage,

    /// Finish reason
    pub finish_reason: FinishReason,

    /// Timestamp
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
}

impl MultiModalResponse {
    /// Extracts the text content from the response
    pub fn text(&self) -> String {
        self.content.extract_text()
    }

    /// Returns all generated images
    pub fn images(&self) -> Vec<ImageOutput> {
        self.content.parts.iter()
            .filter_map(|part| {
                if let ContentPart::Image(image_part) = part {
                    // Convert ImageInput to ImageOutput
                    // This is a simplified conversion
                    None // TODO: Implement proper conversion
                } else {
                    None
                }
            })
            .collect()
    }
}

/// Token usage for multi-modal requests
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct MultiModalUsage {
    /// Tokens in the prompt (text)
    pub prompt_tokens: usize,

    /// Tokens in the completion (text)
    pub completion_tokens: usize,

    /// Total text tokens
    pub total_tokens: usize,

    /// Image tokens (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_tokens: Option<usize>,

    /// Audio tokens (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio_tokens: Option<usize>,
}

impl MultiModalUsage {
    pub fn new(prompt_tokens: usize, completion_tokens: usize) -> Self {
        Self {
            prompt_tokens,
            completion_tokens,
            total_tokens: prompt_tokens + completion_tokens,
            image_tokens: None,
            audio_tokens: None,
        }
    }

    pub fn with_image_tokens(mut self, image_tokens: usize) -> Self {
        self.image_tokens = Some(image_tokens);
        self
    }

    pub fn with_audio_tokens(mut self, audio_tokens: usize) -> Self {
        self.audio_tokens = Some(audio_tokens);
        self
    }

    /// Calculates total cost based on pricing
    pub fn calculate_cost(
        &self,
        text_cost_per_1k: f64,
        image_cost_per_1k: f64,
        audio_cost_per_1k: f64,
    ) -> f64 {
        let text_cost = (self.total_tokens as f64 / 1000.0) * text_cost_per_1k;
        let image_cost = self.image_tokens.map(|t| (t as f64 / 1000.0) * image_cost_per_1k).unwrap_or(0.0);
        let audio_cost = self.audio_tokens.map(|t| (t as f64 / 1000.0) * audio_cost_per_1k).unwrap_or(0.0);

        text_cost + image_cost + audio_cost
    }
}

/// Finish reason for multi-modal responses
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FinishReason {
    /// Completed normally
    Stop,
    /// Reached token limit
    Length,
    /// Content filtered
    ContentFilter,
    /// Tool/function call
    ToolCalls,
    /// Error occurred
    Error,
}

impl std::fmt::Display for FinishReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FinishReason::Stop => write!(f, "stop"),
            FinishReason::Length => write!(f, "length"),
            FinishReason::ContentFilter => write!(f, "content_filter"),
            FinishReason::ToolCalls => write!(f, "tool_calls"),
            FinishReason::Error => write!(f, "error"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_media_type_display() {
        assert_eq!(MediaType::Text.to_string(), "text");
        assert_eq!(MediaType::Image.to_string(), "image");
        assert_eq!(MediaType::Audio.to_string(), "audio");
        assert_eq!(MediaType::Video.to_string(), "video");
    }

    #[test]
    fn test_content_part_types() {
        let text_part = ContentPart::Text(TextPart::new("Hello"));
        assert!(text_part.is_text());
        assert!(!text_part.is_image());
        assert_eq!(text_part.media_type(), MediaType::Text);
    }

    #[test]
    fn test_multimodal_content() {
        let mut content = MultiModalContent::new();
        content.add_text("Test text");

        let (text, image, audio, video) = content.media_type_counts();
        assert_eq!(text, 1);
        assert_eq!(image, 0);
        assert_eq!(audio, 0);
        assert_eq!(video, 0);

        assert_eq!(content.extract_text(), "Test text");
    }

    #[test]
    fn test_multimodal_request_builder() {
        let request = MultiModalRequest::new("gpt-4-vision")
            .with_text("What's in this image?")
            .with_max_tokens(100)
            .with_temperature(0.7);

        assert_eq!(request.model, "gpt-4-vision");
        assert_eq!(request.max_tokens, Some(100));
        assert_eq!(request.temperature, Some(0.7));
        assert!(request.is_text_only());
    }

    #[test]
    fn test_multimodal_usage() {
        let usage = MultiModalUsage::new(100, 50)
            .with_image_tokens(1000)
            .with_audio_tokens(500);

        assert_eq!(usage.total_tokens, 150);
        assert_eq!(usage.image_tokens, Some(1000));
        assert_eq!(usage.audio_tokens, Some(500));

        let cost = usage.calculate_cost(0.01, 0.02, 0.03);
        assert!(cost > 0.0);
    }
}

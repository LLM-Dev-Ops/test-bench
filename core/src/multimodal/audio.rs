// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Audio input and output types for speech models.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::Duration;
use anyhow::{Context, Result};

/// Audio format types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AudioFormat {
    /// MP3 format
    Mp3,
    /// WAV format
    Wav,
    /// FLAC format
    Flac,
    /// Opus format
    Opus,
    /// AAC format
    Aac,
    /// OGG format
    Ogg,
    /// M4A format
    M4a,
    /// WebM format
    WebM,
}

impl AudioFormat {
    /// Returns the MIME type for this format
    pub fn mime_type(&self) -> &'static str {
        match self {
            AudioFormat::Mp3 => "audio/mpeg",
            AudioFormat::Wav => "audio/wav",
            AudioFormat::Flac => "audio/flac",
            AudioFormat::Opus => "audio/opus",
            AudioFormat::Aac => "audio/aac",
            AudioFormat::Ogg => "audio/ogg",
            AudioFormat::M4a => "audio/m4a",
            AudioFormat::WebM => "audio/webm",
        }
    }

    /// Detects format from file extension
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "mp3" => Some(AudioFormat::Mp3),
            "wav" => Some(AudioFormat::Wav),
            "flac" => Some(AudioFormat::Flac),
            "opus" => Some(AudioFormat::Opus),
            "aac" => Some(AudioFormat::Aac),
            "ogg" => Some(AudioFormat::Ogg),
            "m4a" => Some(AudioFormat::M4a),
            "webm" => Some(AudioFormat::WebM),
            _ => None,
        }
    }
}

/// Audio codec types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AudioCodec {
    /// PCM (uncompressed)
    Pcm,
    /// MP3
    Mp3,
    /// AAC
    Aac,
    /// Opus
    Opus,
    /// FLAC
    Flac,
    /// Vorbis
    Vorbis,
}

/// Audio duration wrapper
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct AudioDuration {
    /// Duration in seconds
    pub seconds: u64,
    /// Milliseconds component
    pub millis: u32,
}

impl AudioDuration {
    pub fn new(seconds: u64, millis: u32) -> Self {
        Self { seconds, millis }
    }

    pub fn from_seconds(seconds: f64) -> Self {
        let secs = seconds.floor() as u64;
        let millis = ((seconds - secs as f64) * 1000.0) as u32;
        Self::new(secs, millis)
    }

    pub fn from_duration(duration: Duration) -> Self {
        Self::new(duration.as_secs(), duration.subsec_millis())
    }

    pub fn to_seconds(&self) -> f64 {
        self.seconds as f64 + (self.millis as f64 / 1000.0)
    }

    pub fn to_std_duration(&self) -> Duration {
        Duration::new(self.seconds, self.millis * 1_000_000)
    }
}

/// Audio metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioMetadata {
    /// Audio format
    pub format: AudioFormat,

    /// Sample rate in Hz (e.g., 44100, 48000)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sample_rate: Option<u32>,

    /// Number of channels (1 = mono, 2 = stereo)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channels: Option<u8>,

    /// Bit depth (e.g., 16, 24, 32)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bit_depth: Option<u8>,

    /// Duration of the audio
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<AudioDuration>,

    /// Bitrate in kbps
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bitrate: Option<u32>,

    /// File size in bytes
    pub size_bytes: usize,

    /// Optional title/description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

/// Audio input for speech models
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "source", rename_all = "lowercase")]
pub enum AudioInput {
    /// Base64-encoded audio data
    Base64 {
        /// Base64-encoded audio
        data: String,
        /// Media type (e.g., "audio/mpeg")
        media_type: String,
        /// Optional metadata
        #[serde(skip_serializing_if = "Option::is_none")]
        metadata: Option<AudioMetadata>,
    },
    /// URL to an audio file
    Url {
        /// Audio URL
        url: String,
        /// Optional metadata
        #[serde(skip_serializing_if = "Option::is_none")]
        metadata: Option<AudioMetadata>,
    },
    /// Local file path
    Path {
        /// File path
        path: PathBuf,
        /// Optional metadata
        #[serde(skip_serializing_if = "Option::is_none")]
        metadata: Option<AudioMetadata>,
    },
}

impl AudioInput {
    /// Creates an audio input from a base64 string
    pub fn from_base64(data: String, format: AudioFormat) -> Self {
        Self::Base64 {
            data,
            media_type: format.mime_type().to_string(),
            metadata: None,
        }
    }

    /// Creates an audio input from a URL
    pub fn from_url(url: impl Into<String>) -> Self {
        Self::Url {
            url: url.into(),
            metadata: None,
        }
    }

    /// Creates an audio input from a file path
    pub async fn from_path(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();

        // Validate file exists
        if !path.exists() {
            anyhow::bail!("Audio file does not exist: {}", path.display());
        }

        // Detect format from extension
        let format = path.extension()
            .and_then(|ext| ext.to_str())
            .and_then(AudioFormat::from_extension)
            .context("Could not determine audio format from file extension")?;

        // Get file size
        let metadata = tokio::fs::metadata(path).await
            .context("Failed to read file metadata")?;

        let size_bytes = metadata.len() as usize;

        Ok(Self::Path {
            path: path.to_path_buf(),
            metadata: Some(AudioMetadata {
                format,
                sample_rate: None,
                channels: None,
                bit_depth: None,
                duration: None,
                bitrate: None,
                size_bytes,
                title: None,
            }),
        })
    }

    /// Reads the audio data as bytes
    pub async fn read_bytes(&self) -> Result<Vec<u8>> {
        match self {
            AudioInput::Base64 { data, .. } => {
                use base64::{Engine as _, engine::general_purpose};
                general_purpose::STANDARD.decode(data)
                    .context("Failed to decode base64 audio data")
            }
            AudioInput::Url { url, .. } => {
                let response = reqwest::get(url).await
                    .context("Failed to fetch audio from URL")?;

                if !response.status().is_success() {
                    anyhow::bail!("Failed to fetch audio: HTTP {}", response.status());
                }

                response.bytes().await
                    .map(|b| b.to_vec())
                    .context("Failed to read audio bytes from response")
            }
            AudioInput::Path { path, .. } => {
                tokio::fs::read(path).await
                    .context("Failed to read audio file")
            }
        }
    }

    /// Converts to base64 encoding
    pub async fn to_base64(&self) -> Result<String> {
        use base64::{Engine as _, engine::general_purpose};

        match self {
            AudioInput::Base64 { data, .. } => Ok(data.clone()),
            _ => {
                let bytes = self.read_bytes().await?;
                Ok(general_purpose::STANDARD.encode(&bytes))
            }
        }
    }

    /// Returns the media type
    pub fn media_type(&self) -> Option<&str> {
        match self {
            AudioInput::Base64 { media_type, .. } => Some(media_type),
            AudioInput::Url { metadata, .. } | AudioInput::Path { metadata, .. } => {
                metadata.as_ref().map(|m| m.format.mime_type())
            }
        }
    }
}

/// Audio output from generation/transcription models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioOutput {
    /// Audio data (base64 or URL)
    pub data: String,

    /// Format of the audio
    pub format: AudioFormat,

    /// Transcription text (if this is from STT)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transcription: Option<String>,

    /// Language detected/used
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,

    /// Duration if known
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<AudioDuration>,

    /// Generation parameters used
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<AudioGenerationParameters>,
}

impl AudioOutput {
    pub fn new(data: String, format: AudioFormat) -> Self {
        Self {
            data,
            format,
            transcription: None,
            language: None,
            duration: None,
            parameters: None,
        }
    }

    /// Creates an audio output with transcription
    pub fn with_transcription(mut self, transcription: String, language: Option<String>) -> Self {
        self.transcription = Some(transcription);
        self.language = language;
        self
    }

    /// Saves the audio to a file
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

/// Parameters for audio generation (TTS)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioGenerationParameters {
    /// Text input for TTS
    pub text: String,

    /// Voice to use
    #[serde(skip_serializing_if = "Option::is_none")]
    pub voice: Option<String>,

    /// Language code
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,

    /// Speed/rate (0.25 to 4.0, 1.0 = normal)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speed: Option<f32>,

    /// Pitch adjustment (-20.0 to 20.0, 0.0 = normal)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pitch: Option<f32>,

    /// Sample rate in Hz
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sample_rate: Option<u32>,
}

/// Transcription options for STT
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionOptions {
    /// Language hint (e.g., "en", "es", "fr")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,

    /// Prompt to guide transcription
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,

    /// Temperature for sampling (0.0 to 1.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,

    /// Response format (json, text, srt, vtt)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<String>,

    /// Include timestamps
    #[serde(default)]
    pub timestamps: bool,

    /// Include word-level timestamps
    #[serde(default)]
    pub word_timestamps: bool,
}

impl Default for TranscriptionOptions {
    fn default() -> Self {
        Self {
            language: None,
            prompt: None,
            temperature: Some(0.0),
            response_format: Some("json".to_string()),
            timestamps: false,
            word_timestamps: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_format_mime() {
        assert_eq!(AudioFormat::Mp3.mime_type(), "audio/mpeg");
        assert_eq!(AudioFormat::Wav.mime_type(), "audio/wav");
        assert_eq!(AudioFormat::Flac.mime_type(), "audio/flac");
    }

    #[test]
    fn test_audio_format_from_extension() {
        assert_eq!(AudioFormat::from_extension("mp3"), Some(AudioFormat::Mp3));
        assert_eq!(AudioFormat::from_extension("WAV"), Some(AudioFormat::Wav));
        assert_eq!(AudioFormat::from_extension("flac"), Some(AudioFormat::Flac));
        assert_eq!(AudioFormat::from_extension("unknown"), None);
    }

    #[test]
    fn test_audio_duration() {
        let duration = AudioDuration::from_seconds(65.5);
        assert_eq!(duration.seconds, 65);
        assert_eq!(duration.millis, 500);
        assert!((duration.to_seconds() - 65.5).abs() < 0.001);

        let std_dur = duration.to_std_duration();
        assert_eq!(std_dur.as_secs(), 65);
        assert_eq!(std_dur.subsec_millis(), 500);
    }

    #[test]
    fn test_audio_input_from_base64() {
        let input = AudioInput::from_base64("abc123".to_string(), AudioFormat::Mp3);

        match input {
            AudioInput::Base64 { data, media_type, .. } => {
                assert_eq!(data, "abc123");
                assert_eq!(media_type, "audio/mpeg");
            }
            _ => panic!("Expected Base64 variant"),
        }
    }

    #[test]
    fn test_audio_input_from_url() {
        let input = AudioInput::from_url("https://example.com/audio.mp3");

        match input {
            AudioInput::Url { url, .. } => {
                assert_eq!(url, "https://example.com/audio.mp3");
            }
            _ => panic!("Expected Url variant"),
        }
    }

    #[test]
    fn test_audio_output() {
        let output = AudioOutput::new("base64data".to_string(), AudioFormat::Mp3)
            .with_transcription("Hello world".to_string(), Some("en".to_string()));

        assert_eq!(output.format, AudioFormat::Mp3);
        assert_eq!(output.transcription, Some("Hello world".to_string()));
        assert_eq!(output.language, Some("en".to_string()));
    }

    #[test]
    fn test_transcription_options_default() {
        let options = TranscriptionOptions::default();
        assert_eq!(options.temperature, Some(0.0));
        assert_eq!(options.response_format, Some("json".to_string()));
        assert!(!options.timestamps);
        assert!(!options.word_timestamps);
    }
}

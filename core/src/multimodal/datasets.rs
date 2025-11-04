// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Multi-modal dataset types and utilities.

use serde::{Deserialize, Serialize};
use std::path::Path;
use anyhow::Result;

use super::types::{MultiModalRequest, MultiModalContent};
use super::image::ImageInput;
use super::audio::AudioInput;

/// Multi-modal task types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MultiModalTask {
    /// Image captioning
    ImageCaptioning,
    /// Visual question answering
    VisualQuestionAnswering,
    /// Image classification
    ImageClassification,
    /// Object detection
    ObjectDetection,
    /// OCR (text extraction)
    Ocr,
    /// Speech-to-text transcription
    SpeechToText,
    /// Text-to-speech synthesis
    TextToSpeech,
    /// Audio classification
    AudioClassification,
    /// Video understanding
    VideoUnderstanding,
    /// Image generation
    ImageGeneration,
    /// Audio generation
    AudioGeneration,
}

/// Vision-specific task types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VisionTask {
    /// Describe what's in an image
    Captioning,
    /// Answer questions about an image
    QuestionAnswering,
    /// Classify the image into categories
    Classification,
    /// Detect and locate objects
    ObjectDetection,
    /// Extract text from image
    Ocr,
    /// Understand spatial relationships
    SpatialReasoning,
}

/// Audio-specific task types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AudioTask {
    /// Transcribe speech to text
    Transcription,
    /// Synthesize text to speech
    Synthesis,
    /// Classify audio (music, speech, etc.)
    Classification,
    /// Separate speakers
    Diarization,
    /// Translate speech
    Translation,
}

/// A single multi-modal example
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiModalExample {
    /// Unique identifier
    pub id: String,

    /// Task type
    pub task: MultiModalTask,

    /// Input content
    pub input: MultiModalContent,

    /// Expected output/ground truth
    pub expected_output: String,

    /// Optional metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

impl MultiModalExample {
    pub fn new(
        id: impl Into<String>,
        task: MultiModalTask,
        input: MultiModalContent,
        expected_output: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            task,
            input,
            expected_output: expected_output.into(),
            metadata: None,
        }
    }

    /// Creates a vision captioning example
    pub async fn vision_captioning(
        id: impl Into<String>,
        image_path: impl AsRef<Path>,
        caption: impl Into<String>,
    ) -> Result<Self> {
        let image = ImageInput::from_path(image_path).await?;
        let mut content = MultiModalContent::new();
        content.add_image(image);

        Ok(Self::new(
            id,
            MultiModalTask::ImageCaptioning,
            content,
            caption,
        ))
    }

    /// Creates a VQA example
    pub async fn visual_qa(
        id: impl Into<String>,
        image_path: impl AsRef<Path>,
        question: impl Into<String>,
        answer: impl Into<String>,
    ) -> Result<Self> {
        let image = ImageInput::from_path(image_path).await?;
        let mut content = MultiModalContent::new();
        content.add_text(question);
        content.add_image(image);

        Ok(Self::new(
            id,
            MultiModalTask::VisualQuestionAnswering,
            content,
            answer,
        ))
    }

    /// Creates a speech transcription example
    pub async fn speech_transcription(
        id: impl Into<String>,
        audio_path: impl AsRef<Path>,
        transcription: impl Into<String>,
    ) -> Result<Self> {
        let audio = AudioInput::from_path(audio_path).await?;
        let mut content = MultiModalContent::new();
        content.add_audio(audio);

        Ok(Self::new(
            id,
            MultiModalTask::SpeechToText,
            content,
            transcription,
        ))
    }

    /// Converts to a multi-modal request
    pub fn to_request(&self, model: impl Into<String>) -> MultiModalRequest {
        MultiModalRequest::new(model)
            .with_text("")  // Will be populated from content
    }
}

/// A collection of multi-modal examples
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiModalDataset {
    /// Dataset name
    pub name: String,

    /// Dataset description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Dataset version
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,

    /// Examples in the dataset
    pub examples: Vec<MultiModalExample>,

    /// Task distribution
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_distribution: Option<std::collections::HashMap<String, usize>>,
}

impl MultiModalDataset {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            version: None,
            examples: Vec::new(),
            task_distribution: None,
        }
    }

    /// Adds an example to the dataset
    pub fn add_example(&mut self, example: MultiModalExample) {
        self.examples.push(example);
    }

    /// Returns the number of examples
    pub fn len(&self) -> usize {
        self.examples.len()
    }

    /// Returns true if the dataset is empty
    pub fn is_empty(&self) -> bool {
        self.examples.is_empty()
    }

    /// Filters examples by task
    pub fn filter_by_task(&self, task: MultiModalTask) -> Vec<&MultiModalExample> {
        self.examples.iter().filter(|e| e.task == task).collect()
    }

    /// Calculates task distribution
    pub fn calculate_task_distribution(&mut self) {
        let mut distribution = std::collections::HashMap::new();

        for example in &self.examples {
            let task_name = format!("{:?}", example.task);
            *distribution.entry(task_name).or_insert(0) += 1;
        }

        self.task_distribution = Some(distribution);
    }

    /// Loads a dataset from JSON file
    pub async fn from_json_file(path: impl AsRef<Path>) -> Result<Self> {
        let content = tokio::fs::read_to_string(path).await?;
        let dataset: Self = serde_json::from_str(&content)?;
        Ok(dataset)
    }

    /// Saves the dataset to a JSON file
    pub async fn to_json_file(&self, path: impl AsRef<Path>) -> Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        tokio::fs::write(path, json).await?;
        Ok(())
    }
}

/// Builder for creating multi-modal datasets
pub struct DatasetBuilder {
    name: String,
    description: Option<String>,
    version: Option<String>,
    examples: Vec<MultiModalExample>,
}

impl DatasetBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            version: None,
            examples: Vec::new(),
        }
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }

    pub fn add_example(mut self, example: MultiModalExample) -> Self {
        self.examples.push(example);
        self
    }

    pub fn build(self) -> MultiModalDataset {
        MultiModalDataset {
            name: self.name,
            description: self.description,
            version: self.version,
            examples: self.examples,
            task_distribution: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multimodal_dataset() {
        let mut dataset = MultiModalDataset::new("test-dataset");
        assert_eq!(dataset.name, "test-dataset");
        assert_eq!(dataset.len(), 0);
        assert!(dataset.is_empty());
    }

    #[test]
    fn test_dataset_builder() {
        let dataset = DatasetBuilder::new("test")
            .with_description("Test dataset")
            .with_version("1.0")
            .build();

        assert_eq!(dataset.name, "test");
        assert_eq!(dataset.description, Some("Test dataset".to_string()));
        assert_eq!(dataset.version, Some("1.0".to_string()));
    }
}

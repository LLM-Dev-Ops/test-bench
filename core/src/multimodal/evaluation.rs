// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Evaluation metrics for multi-modal models.

use serde::{Deserialize, Serialize};
use anyhow::Result;

use super::image::ImageOutput;
use super::audio::AudioOutput;
use super::types::{MultiModalRequest, MultiModalResponse};

/// Vision evaluation metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisionMetrics {
    /// Image description accuracy (0.0-1.0)
    pub description_accuracy: f64,

    /// Object detection F1 score (0.0-1.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub object_detection_f1: Option<f64>,

    /// OCR accuracy (0.0-1.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ocr_accuracy: Option<f64>,

    /// Spatial reasoning score (0.0-1.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spatial_reasoning: Option<f64>,

    /// Visual question answering accuracy (0.0-1.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vqa_accuracy: Option<f64>,

    /// CLIP similarity score (if applicable, -1.0 to 1.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clip_similarity: Option<f64>,
}

/// Audio evaluation metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioMetrics {
    /// Word Error Rate (WER) for transcription (0.0-1.0, lower is better)
    pub wer: f64,

    /// Character Error Rate (CER) (0.0-1.0, lower is better)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cer: Option<f64>,

    /// Audio quality score (1.0-5.0 for generated audio)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio_quality: Option<f64>,

    /// Speaker diarization accuracy (0.0-1.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diarization_accuracy: Option<f64>,

    /// Prosody naturalness (1.0-5.0 for TTS)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prosody_score: Option<f64>,
}

/// Combined multi-modal metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiModalMetrics {
    /// Vision metrics (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vision: Option<VisionMetrics>,

    /// Audio metrics (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio: Option<AudioMetrics>,

    /// Cross-modal alignment score (0.0-1.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cross_modal_alignment: Option<f64>,

    /// Overall multi-modal score (0.0-1.0)
    pub overall_score: f64,
}

/// Vision evaluator
pub struct VisionEvaluator {
    // Configuration for vision evaluation
}

impl VisionEvaluator {
    pub fn new() -> Self {
        Self {}
    }

    /// Evaluates vision model output against ground truth
    pub async fn evaluate(
        &self,
        request: &MultiModalRequest,
        response: &MultiModalResponse,
        ground_truth: Option<&str>,
    ) -> Result<VisionMetrics> {
        // Extract response text
        let response_text = response.text();

        // Calculate description accuracy if ground truth provided
        let description_accuracy = if let Some(truth) = ground_truth {
            self.calculate_description_similarity(&response_text, truth)
        } else {
            0.0
        };

        Ok(VisionMetrics {
            description_accuracy,
            object_detection_f1: None,
            ocr_accuracy: None,
            spatial_reasoning: None,
            vqa_accuracy: None,
            clip_similarity: None,
        })
    }

    fn calculate_description_similarity(&self, response: &str, truth: &str) -> f64 {
        // Simple word overlap metric (production would use BLEU, ROUGE, etc.)
        let response_words: std::collections::HashSet<_> = response
            .to_lowercase()
            .split_whitespace()
            .collect();

        let truth_words: std::collections::HashSet<_> = truth
            .to_lowercase()
            .split_whitespace()
            .collect();

        let intersection = response_words.intersection(&truth_words).count();
        let union = response_words.union(&truth_words).count();

        if union == 0 {
            0.0
        } else {
            intersection as f64 / union as f64
        }
    }
}

impl Default for VisionEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

/// Audio evaluator
pub struct AudioEvaluator {
    // Configuration for audio evaluation
}

impl AudioEvaluator {
    pub fn new() -> Self {
        Self {}
    }

    /// Evaluates audio transcription against reference
    pub async fn evaluate(
        &self,
        transcription: &str,
        reference: &str,
    ) -> Result<AudioMetrics> {
        let wer = self.calculate_wer(transcription, reference);
        let cer = Some(self.calculate_cer(transcription, reference));

        Ok(AudioMetrics {
            wer,
            cer,
            audio_quality: None,
            diarization_accuracy: None,
            prosody_score: None,
        })
    }

    /// Calculates Word Error Rate
    fn calculate_wer(&self, hypothesis: &str, reference: &str) -> f64 {
        let hyp_words: Vec<&str> = hypothesis.split_whitespace().collect();
        let ref_words: Vec<&str> = reference.split_whitespace().collect();

        if ref_words.is_empty() {
            return if hyp_words.is_empty() { 0.0 } else { 1.0 };
        }

        // Levenshtein distance at word level
        let distance = self.levenshtein_distance(&hyp_words, &ref_words);
        distance as f64 / ref_words.len() as f64
    }

    /// Calculates Character Error Rate
    fn calculate_cer(&self, hypothesis: &str, reference: &str) -> f64 {
        let hyp_chars: Vec<char> = hypothesis.chars().collect();
        let ref_chars: Vec<char> = reference.chars().collect();

        if ref_chars.is_empty() {
            return if hyp_chars.is_empty() { 0.0 } else { 1.0 };
        }

        let distance = self.levenshtein_distance_chars(&hyp_chars, &ref_chars);
        distance as f64 / ref_chars.len() as f64
    }

    fn levenshtein_distance<T: PartialEq>(&self, a: &[T], b: &[T]) -> usize {
        let len_a = a.len();
        let len_b = b.len();

        if len_a == 0 {
            return len_b;
        }
        if len_b == 0 {
            return len_a;
        }

        let mut matrix = vec![vec![0; len_b + 1]; len_a + 1];

        for i in 0..=len_a {
            matrix[i][0] = i;
        }
        for j in 0..=len_b {
            matrix[0][j] = j;
        }

        for i in 1..=len_a {
            for j in 1..=len_b {
                let cost = if a[i - 1] == b[j - 1] { 0 } else { 1 };
                matrix[i][j] = std::cmp::min(
                    std::cmp::min(matrix[i - 1][j] + 1, matrix[i][j - 1] + 1),
                    matrix[i - 1][j - 1] + cost,
                );
            }
        }

        matrix[len_a][len_b]
    }

    fn levenshtein_distance_chars(&self, a: &[char], b: &[char]) -> usize {
        self.levenshtein_distance(a, b)
    }
}

impl Default for AudioEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

/// Multi-modal evaluator that combines vision and audio evaluation
pub struct MultiModalEvaluator {
    vision_evaluator: VisionEvaluator,
    audio_evaluator: AudioEvaluator,
}

impl MultiModalEvaluator {
    pub fn new() -> Self {
        Self {
            vision_evaluator: VisionEvaluator::new(),
            audio_evaluator: AudioEvaluator::new(),
        }
    }

    /// Evaluates a multi-modal response
    pub async fn evaluate(
        &self,
        request: &MultiModalRequest,
        response: &MultiModalResponse,
        ground_truth: Option<&str>,
    ) -> Result<MultiModalMetrics> {
        let mut vision_metrics = None;
        let mut audio_metrics = None;

        // Evaluate vision if request contains images
        if request.has_images() {
            vision_metrics = Some(
                self.vision_evaluator
                    .evaluate(request, response, ground_truth)
                    .await?,
            );
        }

        // Calculate overall score
        let overall_score = self.calculate_overall_score(
            vision_metrics.as_ref(),
            audio_metrics.as_ref(),
        );

        Ok(MultiModalMetrics {
            vision: vision_metrics,
            audio: audio_metrics,
            cross_modal_alignment: None,
            overall_score,
        })
    }

    fn calculate_overall_score(
        &self,
        vision: Option<&VisionMetrics>,
        audio: Option<&AudioMetrics>,
    ) -> f64 {
        let mut total_score = 0.0;
        let mut count = 0;

        if let Some(v) = vision {
            total_score += v.description_accuracy;
            count += 1;
        }

        if let Some(a) = audio {
            total_score += 1.0 - a.wer; // Invert WER (lower is better)
            count += 1;
        }

        if count > 0 {
            total_score / count as f64
        } else {
            0.0
        }
    }
}

impl Default for MultiModalEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_evaluator_wer() {
        let evaluator = AudioEvaluator::new();

        // Perfect match
        let wer = evaluator.calculate_wer("hello world", "hello world");
        assert_eq!(wer, 0.0);

        // One substitution
        let wer = evaluator.calculate_wer("hello world", "hello earth");
        assert!(wer > 0.0 && wer < 1.0);

        // Complete mismatch
        let wer = evaluator.calculate_wer("foo bar", "hello world");
        assert_eq!(wer, 1.0);
    }

    #[test]
    fn test_audio_evaluator_cer() {
        let evaluator = AudioEvaluator::new();

        // Perfect match
        let cer = evaluator.calculate_cer("hello", "hello");
        assert_eq!(cer, 0.0);

        // One character difference
        let cer = evaluator.calculate_cer("hello", "hallo");
        assert!(cer > 0.0);
    }

    #[test]
    fn test_vision_evaluator_similarity() {
        let evaluator = VisionEvaluator::new();

        // Perfect match
        let sim = evaluator.calculate_description_similarity("a cat", "a cat");
        assert_eq!(sim, 1.0);

        // Partial match
        let sim = evaluator.calculate_description_similarity("a black cat", "a cat");
        assert!(sim > 0.5 && sim < 1.0);

        // No match
        let sim = evaluator.calculate_description_similarity("dog", "cat");
        assert!(sim < 0.5);
    }
}

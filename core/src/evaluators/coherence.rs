// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Coherence metric implementation
//!
//! This module provides a comprehensive coherence evaluator that combines:
//! - Readability metrics (Flesch Reading Ease, Flesch-Kincaid Grade Level)
//! - Discourse analysis (markers, topic shifts, logical flow)
//! - LLM-as-judge scoring for grammatical correctness and consistency
//!
//! The coherence score is a multi-dimensional assessment of text quality.

use super::text_analysis::{
    average_sentence_length, count_words, detect_discourse_markers, flesch_kincaid_grade,
    flesch_reading_ease, split_sentences, DiscourseMarker,
};
use super::{EvaluationResult, Evaluator, EvaluatorError};
use crate::providers::{CompletionRequest, Provider};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Detailed coherence evaluation score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoherenceScore {
    /// Overall coherence score (0.0-1.0, higher is better)
    pub overall_score: f64,

    /// Logical flow score from LLM judge (0.0-1.0)
    pub logical_flow: f64,

    /// Grammatical correctness from LLM judge (0.0-1.0)
    pub grammatical_correctness: f64,

    /// Consistency score from LLM judge (0.0-1.0)
    pub consistency: f64,

    /// Readability score (0.0-1.0, normalized from metrics)
    pub readability: f64,

    /// Flesch Reading Ease (0-100+, higher is easier)
    pub flesch_reading_ease: f64,

    /// Flesch-Kincaid Grade Level (U.S. grade level)
    pub flesch_kincaid_grade: f64,

    /// Average sentence length in words
    pub avg_sentence_length: f64,

    /// Detected discourse markers
    pub discourse_markers: Vec<DiscourseMarker>,

    /// List of coherence violations found
    pub coherence_violations: Vec<CoherenceViolation>,
}

/// A detected coherence violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoherenceViolation {
    /// Location in text (line number or position description)
    pub location: String,

    /// Type of violation
    pub violation_type: ViolationType,

    /// Human-readable description of the issue
    pub description: String,

    /// Severity level
    pub severity: Severity,
}

/// Types of coherence violations
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ViolationType {
    /// Abrupt topic shift without transition
    TopicShift,

    /// Logical contradiction or inconsistency
    Contradiction,

    /// Grammatical error
    Grammar,

    /// Missing logical connection
    MissingConnection,

    /// Excessive repetition
    Repetition,

    /// Poor sentence structure
    Structure,
}

/// Severity levels for violations
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    /// Minor issue, doesn't significantly impact understanding
    Low,

    /// Moderate issue, may cause confusion
    Medium,

    /// Major issue, significantly impairs understanding
    High,
}

/// Coherence evaluator - measures output fluency and logical consistency
///
/// This evaluator combines multiple approaches:
/// 1. **Readability Metrics**: Flesch-Kincaid formulas for objective readability
/// 2. **Discourse Analysis**: Detection of logical connectors and transitions
/// 3. **LLM-as-Judge**: Advanced analysis of grammar, logic, and consistency
///
/// # Examples
///
/// ```no_run
/// use llm_test_bench_core::evaluators::CoherenceEvaluator;
/// use llm_test_bench_core::providers::OpenAIProvider;
/// use std::sync::Arc;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let provider = OpenAIProvider::new("test_key".to_string())?;
/// let evaluator = CoherenceEvaluator::new(Arc::new(provider), "gpt-4".to_string());
///
/// let text = "The cat sat on the mat. The weather was sunny. The economy is growing.";
/// let result = evaluator.evaluate_detailed("", text).await?;
///
/// println!("Overall coherence: {:.2}", result.overall_score);
/// println!("Readability: {:.2}", result.flesch_reading_ease);
/// # Ok(())
/// # }
/// ```
pub struct CoherenceEvaluator {
    /// Optional LLM provider for advanced analysis
    provider: Option<Arc<dyn Provider>>,
    /// Model to use for LLM-based evaluation
    model: String,
}

impl CoherenceEvaluator {
    /// Create a new coherence evaluator without LLM-based analysis
    ///
    /// This will only use readability metrics and discourse analysis.
    pub fn new_basic() -> Self {
        Self {
            provider: None,
            model: String::new(),
        }
    }

    /// Create a new coherence evaluator with LLM-based analysis
    ///
    /// # Arguments
    ///
    /// * `provider` - LLM provider for advanced analysis
    /// * `model` - Model to use (e.g., "gpt-4", "gpt-3.5-turbo")
    pub fn new(provider: Arc<dyn Provider>, model: String) -> Self {
        Self {
            provider: Some(provider),
            model,
        }
    }

    /// Evaluate coherence with detailed breakdown
    ///
    /// # Arguments
    ///
    /// * `_prompt` - The original prompt (for context)
    /// * `response` - The text to evaluate
    pub async fn evaluate_detailed(
        &self,
        _prompt: &str,
        response: &str,
    ) -> Result<CoherenceScore, EvaluatorError> {
        // Calculate readability metrics
        let flesch_ease = flesch_reading_ease(response);
        let flesch_grade = flesch_kincaid_grade(response);
        let avg_sent_len = average_sentence_length(response);

        // Detect discourse markers
        let markers = detect_discourse_markers(response);

        // Normalize readability to 0-1 scale
        // Flesch Reading Ease: 0-100, we consider 60+ as good
        let readability_score = (flesch_ease / 100.0).clamp(0.0, 1.0);

        // Analyze basic coherence issues
        let violations = self.analyze_basic_violations(response);

        // Get LLM-based scores if provider is available
        let (logical_flow, grammar, consistency) = if let Some(ref provider) = self.provider {
            self.evaluate_with_llm(provider, response).await?
        } else {
            // Use heuristic scores based on basic analysis
            let base_score = self.calculate_heuristic_score(response, &violations);
            (base_score, base_score, base_score)
        };

        // Calculate overall score as weighted average
        let overall_score = self.calculate_overall_score(
            logical_flow,
            grammar,
            consistency,
            readability_score,
            &violations,
        );

        Ok(CoherenceScore {
            overall_score,
            logical_flow,
            grammatical_correctness: grammar,
            consistency,
            readability: readability_score,
            flesch_reading_ease: flesch_ease,
            flesch_kincaid_grade: flesch_grade,
            avg_sentence_length: avg_sent_len,
            discourse_markers: markers,
            coherence_violations: violations,
        })
    }

    /// Analyze basic coherence violations without LLM
    fn analyze_basic_violations(&self, text: &str) -> Vec<CoherenceViolation> {
        let mut violations = Vec::new();

        // Check for very short text
        let word_count = count_words(text);
        if word_count < 3 {
            violations.push(CoherenceViolation {
                location: "entire text".to_string(),
                violation_type: ViolationType::Structure,
                description: "Text is too short to assess coherence properly".to_string(),
                severity: Severity::Medium,
            });
            return violations;
        }

        // Check for excessively long sentences
        let sentences = split_sentences(text);
        for (i, sentence) in sentences.iter().enumerate() {
            let words = count_words(sentence);
            if words > 40 {
                violations.push(CoherenceViolation {
                    location: format!("sentence {}", i + 1),
                    violation_type: ViolationType::Structure,
                    description: format!(
                        "Very long sentence ({} words) may impair readability",
                        words
                    ),
                    severity: Severity::Low,
                });
            }
        }

        // Check for excessively short sentences (excluding valid short ones)
        for (i, sentence) in sentences.iter().enumerate() {
            let words = count_words(sentence);
            if words == 1 && !sentence.ends_with('?') && !sentence.ends_with('!') {
                violations.push(CoherenceViolation {
                    location: format!("sentence {}", i + 1),
                    violation_type: ViolationType::Structure,
                    description: "Single-word sentence may indicate incomplete thought".to_string(),
                    severity: Severity::Low,
                });
            }
        }

        // Check for repeated words (simple repetition detection)
        let words: Vec<&str> = text.split_whitespace().collect();
        for i in 0..words.len().saturating_sub(1) {
            if words[i].to_lowercase() == words[i + 1].to_lowercase()
                && words[i].len() > 3
                && !words[i].chars().all(|c| !c.is_alphabetic())
            {
                violations.push(CoherenceViolation {
                    location: format!("words {}-{}", i + 1, i + 2),
                    violation_type: ViolationType::Repetition,
                    description: format!("Repeated word: '{}'", words[i]),
                    severity: Severity::Low,
                });
            }
        }

        violations
    }

    /// Evaluate using LLM-as-judge
    async fn evaluate_with_llm(
        &self,
        provider: &Arc<dyn Provider>,
        text: &str,
    ) -> Result<(f64, f64, f64), EvaluatorError> {
        let prompt = format!(
            r#"Evaluate the following text for coherence. Provide scores from 0.0 to 1.0 for:
1. Logical flow (how well ideas connect)
2. Grammatical correctness (grammar, syntax, punctuation)
3. Consistency (no contradictions, maintains theme)

Text to evaluate:
"{}"

Respond ONLY with three numbers separated by commas, like: 0.8,0.9,0.7
Do not include any other text or explanation."#,
            text
        );

        let request = CompletionRequest::new(&self.model, prompt)
            .with_max_tokens(50)
            .with_temperature(0.1); // Low temperature for consistent scoring

        let response = provider
            .complete(request)
            .await
            .map_err(|e| EvaluatorError::EvaluationFailed(format!("LLM request failed: {}", e)))?;

        // Parse the response
        let scores = self.parse_llm_scores(&response.content)?;
        Ok(scores)
    }

    /// Parse LLM response into three scores
    fn parse_llm_scores(&self, response: &str) -> Result<(f64, f64, f64), EvaluatorError> {
        let cleaned = response.trim().replace(' ', "");

        let parts: Vec<&str> = cleaned.split(',').collect();
        if parts.len() != 3 {
            return Err(EvaluatorError::EvaluationFailed(format!(
                "Expected 3 scores, got {}: {}",
                parts.len(),
                response
            )));
        }

        let logical_flow: f64 = parts[0].parse().map_err(|_| {
            EvaluatorError::EvaluationFailed(format!("Invalid logical flow score: {}", parts[0]))
        })?;

        let grammar: f64 = parts[1].parse().map_err(|_| {
            EvaluatorError::EvaluationFailed(format!("Invalid grammar score: {}", parts[1]))
        })?;

        let consistency: f64 = parts[2].parse().map_err(|_| {
            EvaluatorError::EvaluationFailed(format!("Invalid consistency score: {}", parts[2]))
        })?;

        // Clamp scores to valid range
        Ok((
            logical_flow.clamp(0.0, 1.0),
            grammar.clamp(0.0, 1.0),
            consistency.clamp(0.0, 1.0),
        ))
    }

    /// Calculate heuristic score when LLM is not available
    fn calculate_heuristic_score(&self, text: &str, violations: &[CoherenceViolation]) -> f64 {
        let word_count = count_words(text);

        // Base score starts high
        let mut score = 0.85;

        // Penalize for violations
        for violation in violations {
            let penalty = match violation.severity {
                Severity::Low => 0.03,
                Severity::Medium => 0.08,
                Severity::High => 0.15,
            };
            score -= penalty;
        }

        // Penalize very short text
        if word_count < 10 {
            score -= 0.1;
        }

        // Check for discourse markers (good sign of coherence)
        let markers = detect_discourse_markers(text);
        let marker_density = markers.len() as f64 / word_count.max(1) as f64;

        // Good marker density is around 1 per 20-30 words
        if marker_density > 0.02 && marker_density < 0.15 {
            score += 0.05;
        }

        score.clamp(0.0, 1.0)
    }

    /// Calculate overall coherence score from components
    fn calculate_overall_score(
        &self,
        logical_flow: f64,
        grammar: f64,
        consistency: f64,
        readability: f64,
        violations: &[CoherenceViolation],
    ) -> f64 {
        // Weighted average
        let base_score = logical_flow * 0.35 // Logical flow is most important
            + grammar * 0.25
            + consistency * 0.25
            + readability * 0.15;

        // Apply penalty for high-severity violations
        let high_severity_count = violations
            .iter()
            .filter(|v| v.severity == Severity::High)
            .count();

        let penalty = (high_severity_count as f64) * 0.1;
        (base_score - penalty).clamp(0.0, 1.0)
    }
}

impl Default for CoherenceEvaluator {
    fn default() -> Self {
        Self::new_basic()
    }
}

#[async_trait::async_trait]
impl super::Evaluator for CoherenceEvaluator {
    async fn evaluate(
        &self,
        _prompt: &str,
        response: &str,
    ) -> Result<EvaluationResult, EvaluatorError> {
        // For sync trait, we use blocking evaluation without LLM
        let flesch_ease = flesch_reading_ease(response);
        let flesch_grade = flesch_kincaid_grade(response);
        let avg_sent_len = average_sentence_length(response);
        let markers = detect_discourse_markers(response);
        let violations = self.analyze_basic_violations(response);

        let readability_score = (flesch_ease / 100.0).clamp(0.0, 1.0);
        let heuristic_score = self.calculate_heuristic_score(response, &violations);

        let overall_score = self.calculate_overall_score(
            heuristic_score,
            heuristic_score,
            heuristic_score,
            readability_score,
            &violations,
        );

        let score = CoherenceScore {
            overall_score,
            logical_flow: heuristic_score,
            grammatical_correctness: heuristic_score,
            consistency: heuristic_score,
            readability: readability_score,
            flesch_reading_ease: flesch_ease,
            flesch_kincaid_grade: flesch_grade,
            avg_sentence_length: avg_sent_len,
            discourse_markers: markers,
            coherence_violations: violations,
        };

        Ok(EvaluationResult {
            metric: "coherence".to_string(),
            score: overall_score,
            details: serde_json::to_value(score)
                .map_err(|e| EvaluatorError::EvaluationFailed(e.to_string()))?,
        })
    }

    fn name(&self) -> &str {
        "Coherence"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coherence_evaluator_basic() {
        let evaluator = CoherenceEvaluator::new_basic();
        assert_eq!(evaluator.name(), "Coherence");
    }

    #[test]
    fn test_evaluate_well_structured_text() {
        let evaluator = CoherenceEvaluator::new_basic();
        let text = "The cat sat on the mat. It was a sunny day. The birds were singing cheerfully.";

        let result = evaluator.evaluate("", text).unwrap();
        assert!(result.score > 0.7, "Well-structured text should score highly");
    }

    #[test]
    fn test_evaluate_poor_structure() {
        let evaluator = CoherenceEvaluator::new_basic();
        let text = "Cat. Sunny. Birds birds birds. Very very very long sentence that goes on and on without any clear purpose or meaningful content just rambling continuously.";

        let result = evaluator.evaluate("", text).unwrap();
        assert!(result.score < 0.8, "Poor structure should lower score");
    }

    #[test]
    fn test_evaluate_empty_text() {
        let evaluator = CoherenceEvaluator::new_basic();
        let result = evaluator.evaluate("", "").unwrap();
        assert!(result.score < 0.5, "Empty text should score low");
    }

    #[test]
    fn test_evaluate_very_short_text() {
        let evaluator = CoherenceEvaluator::new_basic();
        let result = evaluator.evaluate("", "Yes.").unwrap();
        assert!(result.score < 0.9, "Very short text should be penalized");
    }

    #[test]
    fn test_detect_repetition_violations() {
        let evaluator = CoherenceEvaluator::new_basic();
        let violations = evaluator.analyze_basic_violations("The the cat sat on the mat.");

        assert!(!violations.is_empty());
        assert!(violations
            .iter()
            .any(|v| v.violation_type == ViolationType::Repetition));
    }

    #[test]
    fn test_detect_long_sentence_violations() {
        let evaluator = CoherenceEvaluator::new_basic();
        let long_sentence = "This is an extremely long sentence that contains far too many words and clauses and goes on and on without any clear structure or logical breaks which makes it very difficult to read and understand properly.";

        let violations = evaluator.analyze_basic_violations(long_sentence);
        assert!(violations
            .iter()
            .any(|v| v.violation_type == ViolationType::Structure));
    }

    #[test]
    fn test_discourse_markers_improve_score() {
        let evaluator = CoherenceEvaluator::new_basic();

        let without_markers = "The experiment was conducted. The results were analyzed. The conclusion was drawn.";
        let with_markers = "First, the experiment was conducted. Then, the results were analyzed. Finally, the conclusion was drawn.";

        let score1 = evaluator.evaluate("", without_markers).unwrap().score;
        let score2 = evaluator.evaluate("", with_markers).unwrap().score;

        // Text with discourse markers should generally score better
        assert!(
            score2 >= score1 - 0.05,
            "Discourse markers should help or maintain score"
        );
    }

    #[test]
    fn test_parse_llm_scores_valid() {
        let evaluator = CoherenceEvaluator::new_basic();

        let result = evaluator.parse_llm_scores("0.8,0.9,0.7");
        assert!(result.is_ok());
        let (flow, grammar, consistency) = result.unwrap();
        assert_eq!(flow, 0.8);
        assert_eq!(grammar, 0.9);
        assert_eq!(consistency, 0.7);
    }

    #[test]
    fn test_parse_llm_scores_with_spaces() {
        let evaluator = CoherenceEvaluator::new_basic();

        let result = evaluator.parse_llm_scores("0.8, 0.9, 0.7");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_llm_scores_clamping() {
        let evaluator = CoherenceEvaluator::new_basic();

        let result = evaluator.parse_llm_scores("1.5,0.5,-0.1");
        assert!(result.is_ok());
        let (flow, grammar, consistency) = result.unwrap();
        assert_eq!(flow, 1.0); // Clamped to max
        assert_eq!(grammar, 0.5);
        assert_eq!(consistency, 0.0); // Clamped to min
    }

    #[test]
    fn test_parse_llm_scores_invalid() {
        let evaluator = CoherenceEvaluator::new_basic();

        assert!(evaluator.parse_llm_scores("0.8,0.9").is_err());
        assert!(evaluator.parse_llm_scores("invalid,0.9,0.7").is_err());
        assert!(evaluator.parse_llm_scores("").is_err());
    }

    #[test]
    fn test_violation_severity_ordering() {
        assert!(Severity::High > Severity::Medium);
        assert!(Severity::Medium > Severity::Low);
    }

    #[test]
    fn test_calculate_overall_score() {
        let evaluator = CoherenceEvaluator::new_basic();

        let score = evaluator.calculate_overall_score(0.9, 0.9, 0.9, 0.9, &[]);
        assert!(score > 0.85, "High component scores should yield high overall");

        let score = evaluator.calculate_overall_score(0.5, 0.5, 0.5, 0.5, &[]);
        assert!(
            score < 0.6 && score > 0.4,
            "Medium component scores should yield medium overall"
        );

        // Test with violations
        let violations = vec![CoherenceViolation {
            location: "test".to_string(),
            violation_type: ViolationType::Contradiction,
            description: "test".to_string(),
            severity: Severity::High,
        }];

        let score_with_violation = evaluator.calculate_overall_score(0.9, 0.9, 0.9, 0.9, &violations);
        let score_without = evaluator.calculate_overall_score(0.9, 0.9, 0.9, 0.9, &[]);

        assert!(score_with_violation < score_without, "Violations should lower score");
    }

    #[test]
    fn test_heuristic_score_calculation() {
        let evaluator = CoherenceEvaluator::new_basic();

        let good_text = "The experiment was successful. However, further research is needed. Therefore, we recommend additional studies.";
        let score = evaluator.calculate_heuristic_score(good_text, &[]);
        assert!(score > 0.8, "Good text should score highly: {}", score);

        let violations = vec![
            CoherenceViolation {
                location: "test".to_string(),
                violation_type: ViolationType::Grammar,
                description: "test".to_string(),
                severity: Severity::Medium,
            },
            CoherenceViolation {
                location: "test".to_string(),
                violation_type: ViolationType::Structure,
                description: "test".to_string(),
                severity: Severity::High,
            },
        ];

        let score_with_violations = evaluator.calculate_heuristic_score(good_text, &violations);
        assert!(score_with_violations < score, "Violations should reduce score");
    }
}

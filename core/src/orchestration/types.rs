// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Shared types for orchestration module.
//!
//! This module defines common types used across comparison, ranking, and routing
//! functionality, including configuration structures, result types, and enums.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

use crate::evaluators::EvaluationResult;
use crate::providers::types::{CompletionResponse, TokenUsage};

/// Configuration for a specific model in a comparison.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModelConfig {
    /// Provider name (e.g., "openai", "anthropic")
    pub provider: String,

    /// Model identifier (e.g., "gpt-4", "claude-3-opus-20240229")
    pub model: String,

    /// Model-specific parameters
    #[serde(default)]
    pub parameters: HashMap<String, serde_json::Value>,
}

impl ModelConfig {
    /// Create a new model configuration.
    pub fn new(provider: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            provider: provider.into(),
            model: model.into(),
            parameters: HashMap::new(),
        }
    }

    /// Add a parameter to the configuration.
    pub fn with_parameter(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.parameters.insert(key.into(), value);
        self
    }

    /// Get a unique identifier for this model config.
    pub fn identifier(&self) -> String {
        format!("{}/{}", self.provider, self.model)
    }
}

/// Configuration for multi-model comparison.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonConfig {
    /// Models to compare
    pub models: Vec<ModelConfig>,

    /// Metrics to evaluate (e.g., ["faithfulness", "relevance", "coherence"])
    pub metrics: Vec<String>,

    /// Whether to perform statistical significance testing
    #[serde(default)]
    pub statistical_tests: bool,

    /// Timeout for the entire comparison operation
    #[serde(default = "default_comparison_timeout")]
    pub timeout_seconds: u64,

    /// Concurrency limit for parallel model execution
    #[serde(default = "default_concurrency_limit")]
    pub concurrency_limit: usize,
}

fn default_comparison_timeout() -> u64 {
    300
}

fn default_concurrency_limit() -> usize {
    10
}

impl ComparisonConfig {
    /// Create a new comparison configuration.
    pub fn new(models: Vec<ModelConfig>) -> Self {
        Self {
            models,
            metrics: vec![
                "faithfulness".to_string(),
                "relevance".to_string(),
                "coherence".to_string(),
            ],
            statistical_tests: false,
            timeout_seconds: default_comparison_timeout(),
            concurrency_limit: default_concurrency_limit(),
        }
    }

    /// Set the metrics to evaluate.
    pub fn with_metrics(mut self, metrics: Vec<String>) -> Self {
        self.metrics = metrics;
        self
    }

    /// Enable statistical significance testing.
    pub fn with_statistical_tests(mut self, enabled: bool) -> Self {
        self.statistical_tests = enabled;
        self
    }

    /// Set the timeout in seconds.
    pub fn with_timeout(mut self, timeout_seconds: u64) -> Self {
        self.timeout_seconds = timeout_seconds;
        self
    }

    /// Set the concurrency limit.
    pub fn with_concurrency_limit(mut self, limit: usize) -> Self {
        self.concurrency_limit = limit;
        self
    }
}

/// Result for a single model in a comparison.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelResult {
    /// Model configuration
    pub model_config: ModelConfig,

    /// Whether the model execution succeeded
    pub success: bool,

    /// The model's response (if successful)
    pub response: Option<CompletionResponse>,

    /// Evaluation scores by metric name
    pub evaluation_scores: HashMap<String, f64>,

    /// Latency in milliseconds
    pub latency_ms: u64,

    /// Estimated cost in USD
    pub cost: f64,

    /// Error message (if failed)
    pub error: Option<String>,
}

impl ModelResult {
    /// Calculate the average quality score across all evaluation metrics.
    pub fn average_quality_score(&self) -> f64 {
        if self.evaluation_scores.is_empty() {
            return 0.0;
        }

        let sum: f64 = self.evaluation_scores.values().sum();
        sum / self.evaluation_scores.len() as f64
    }

    /// Get a specific evaluation score by metric name.
    pub fn get_score(&self, metric: &str) -> Option<f64> {
        self.evaluation_scores.get(metric).copied()
    }
}

/// Ranking information for a model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelRanking {
    /// Model configuration
    pub model_config: ModelConfig,

    /// Overall rank (1 = best)
    pub rank: usize,

    /// Overall score (0.0 - 1.0)
    pub overall_score: f64,

    /// Individual component scores
    pub component_scores: ComponentScores,

    /// Key strengths of this model
    pub strengths: Vec<String>,

    /// Key weaknesses of this model
    pub weaknesses: Vec<String>,
}

/// Component scores used in ranking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentScores {
    /// Quality score (0.0 - 1.0)
    pub quality: f64,

    /// Performance score (0.0 - 1.0)
    pub performance: f64,

    /// Cost efficiency score (0.0 - 1.0)
    pub cost_efficiency: f64,
}

/// Complete comparison result with rankings and analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonResult {
    /// Individual model results
    pub models: Vec<ModelResult>,

    /// Rankings in order
    pub rankings: Vec<ModelRanking>,

    /// Winner (best overall model)
    pub winner: Option<String>,

    /// Comparative analysis
    pub comparative_analysis: ComparativeAnalysis,

    /// Statistical significance test results (if enabled)
    pub statistical_significance: Option<SignificanceTest>,

    /// Total comparison duration
    pub total_duration: Duration,
}

/// Comparative analysis across models.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparativeAnalysis {
    /// High-level summary
    pub summary: String,

    /// Key findings
    pub key_findings: Vec<Finding>,

    /// Model comparison matrix (pairwise scores)
    pub model_comparison_matrix: Vec<Vec<f64>>,

    /// Areas where models agree
    pub consensus_areas: Vec<String>,

    /// Areas where models diverge
    pub divergence_areas: Vec<String>,

    /// Recommendations
    pub recommendations: Vec<Recommendation>,
}

/// A single finding from the analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    /// Finding category
    pub category: FindingCategory,

    /// Finding description
    pub description: String,

    /// Supporting evidence
    pub evidence: String,
}

/// Category of finding.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum FindingCategory {
    /// Quality-related finding
    Quality,

    /// Performance-related finding
    Performance,

    /// Cost-related finding
    Cost,

    /// Reliability-related finding
    Reliability,
}

/// A recommendation from the analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    /// Type of recommendation
    pub recommendation_type: RecommendationType,

    /// Model being recommended
    pub model: String,

    /// Reasoning for the recommendation
    pub reasoning: String,

    /// Confidence level (0.0 - 1.0)
    pub confidence: f64,
}

/// Type of recommendation.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum RecommendationType {
    /// Best overall model
    BestOverall,

    /// Best for cost optimization
    BestForCost,

    /// Best for speed/latency
    BestForSpeed,

    /// Best for quality
    BestForQuality,

    /// Best for reliability
    BestForReliability,
}

/// Statistical significance test results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignificanceTest {
    /// Test method used (e.g., "t-test", "wilcoxon")
    pub test_method: String,

    /// p-values between pairs of models
    pub p_values: HashMap<String, f64>,

    /// Whether differences are statistically significant
    pub significant_differences: Vec<SignificantDifference>,
}

/// A statistically significant difference between models.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignificantDifference {
    /// First model
    pub model_a: String,

    /// Second model
    pub model_b: String,

    /// p-value
    pub p_value: f64,

    /// Effect size
    pub effect_size: f64,

    /// Which model performed better
    pub better_model: String,
}

/// Model profile for routing decisions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelProfile {
    /// Model name (provider/model format)
    pub name: String,

    /// Typical quality score (0.0 - 1.0)
    pub typical_quality: f64,

    /// Average latency in milliseconds
    pub avg_latency_ms: u64,

    /// Cost per 1,000 tokens
    pub cost_per_1k_tokens: f64,

    /// Maximum context length
    pub context_limit: usize,

    /// Task types this model excels at
    pub strengths: Vec<TaskType>,

    /// Number of samples used to calculate statistics
    pub sample_count: usize,
}

impl ModelProfile {
    /// Create a new model profile.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            typical_quality: 0.0,
            avg_latency_ms: 0,
            cost_per_1k_tokens: 0.0,
            context_limit: 0,
            strengths: Vec::new(),
            sample_count: 0,
        }
    }

    /// Update profile with new benchmark results.
    pub fn update_from_result(&mut self, result: &ModelResult, weight: f64) {
        // Rolling average for quality
        let new_quality = result.average_quality_score();
        self.typical_quality = self.typical_quality * (1.0 - weight) + new_quality * weight;

        // Rolling average for latency
        let new_latency = result.latency_ms as f64;
        self.avg_latency_ms =
            ((self.avg_latency_ms as f64 * (1.0 - weight)) + (new_latency * weight)) as u64;

        // Update sample count
        self.sample_count += 1;
    }
}

/// Task type classification for routing.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum TaskType {
    /// Reasoning and problem-solving tasks
    Reasoning,

    /// Code generation and analysis
    Coding,

    /// Creative writing and generation
    Creative,

    /// Text summarization
    Summarization,

    /// Language translation
    Translation,

    /// Text classification
    Classification,

    /// Question answering
    QuestionAnswering,

    /// General-purpose tasks
    General,
}

impl TaskType {
    /// Classify a prompt into a task type.
    pub fn classify_prompt(prompt: &str) -> Self {
        let prompt_lower = prompt.to_lowercase();

        // Simple keyword-based classification
        if prompt_lower.contains("code")
            || prompt_lower.contains("function")
            || prompt_lower.contains("implement")
            || prompt_lower.contains("debug")
        {
            TaskType::Coding
        } else if prompt_lower.contains("summarize")
            || prompt_lower.contains("summary")
            || prompt_lower.contains("tldr")
        {
            TaskType::Summarization
        } else if prompt_lower.contains("translate") || prompt_lower.contains("translation") {
            TaskType::Translation
        } else if prompt_lower.contains("classify")
            || prompt_lower.contains("categorize")
            || prompt_lower.contains("label")
        {
            TaskType::Classification
        } else if prompt_lower.contains("story")
            || prompt_lower.contains("poem")
            || prompt_lower.contains("creative")
            || prompt_lower.contains("imagine")
        {
            TaskType::Creative
        } else if prompt_lower.contains("reason")
            || prompt_lower.contains("solve")
            || prompt_lower.contains("calculate")
            || prompt_lower.contains("analyze")
        {
            TaskType::Reasoning
        } else if prompt_lower.contains("what")
            || prompt_lower.contains("how")
            || prompt_lower.contains("why")
            || prompt_lower.contains("?")
        {
            TaskType::QuestionAnswering
        } else {
            TaskType::General
        }
    }
}

/// Routing strategy for model selection.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum RoutingStrategy {
    /// Select highest quality model
    Quality,

    /// Select cheapest model meeting quality threshold
    CostOptimized,

    /// Select fastest model
    Latency,

    /// Balance quality, cost, and latency
    Balanced,
}

/// Constraints for model selection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConstraints {
    /// Maximum cost per request in USD
    pub max_cost: Option<f64>,

    /// Maximum latency in milliseconds
    pub max_latency_ms: Option<u64>,

    /// Minimum quality score (0.0 - 1.0)
    #[serde(default = "default_min_quality")]
    pub min_quality: f64,

    /// Minimum context length required
    pub min_context_length: Option<usize>,
}

fn default_min_quality() -> f64 {
    0.7
}

impl Default for ModelConstraints {
    fn default() -> Self {
        Self {
            max_cost: None,
            max_latency_ms: None,
            min_quality: default_min_quality(),
            min_context_length: None,
        }
    }
}

impl ModelConstraints {
    /// Create new constraints with defaults.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set maximum cost constraint.
    pub fn with_max_cost(mut self, max_cost: f64) -> Self {
        self.max_cost = Some(max_cost);
        self
    }

    /// Set maximum latency constraint.
    pub fn with_max_latency_ms(mut self, max_latency_ms: u64) -> Self {
        self.max_latency_ms = Some(max_latency_ms);
        self
    }

    /// Set minimum quality constraint.
    pub fn with_min_quality(mut self, min_quality: f64) -> Self {
        self.min_quality = min_quality;
        self
    }

    /// Set minimum context length constraint.
    pub fn with_min_context_length(mut self, min_context_length: usize) -> Self {
        self.min_context_length = Some(min_context_length);
        self
    }
}

/// Model selection result from router.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelSelection {
    /// Selected model configuration
    pub model_config: ModelConfig,

    /// Reasoning for the selection
    pub reasoning: String,

    /// Expected quality score
    pub expected_quality: f64,

    /// Expected latency in milliseconds
    pub expected_latency_ms: u64,

    /// Expected cost in USD
    pub expected_cost: f64,

    /// Alternative models that were considered
    pub alternatives: Vec<ModelAlternative>,
}

/// Alternative model that was considered but not selected.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelAlternative {
    /// Model configuration
    pub model_config: ModelConfig,

    /// Reason for not selecting
    pub reason: String,

    /// Score for this alternative
    pub score: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_config_identifier() {
        let config = ModelConfig::new("openai", "gpt-4");
        assert_eq!(config.identifier(), "openai/gpt-4");
    }

    #[test]
    fn test_model_config_with_parameter() {
        let config = ModelConfig::new("openai", "gpt-4")
            .with_parameter("temperature", serde_json::json!(0.7));

        assert_eq!(config.parameters.len(), 1);
        assert_eq!(config.parameters.get("temperature").unwrap(), &serde_json::json!(0.7));
    }

    #[test]
    fn test_comparison_config_defaults() {
        let models = vec![ModelConfig::new("openai", "gpt-4")];
        let config = ComparisonConfig::new(models);

        assert_eq!(config.timeout_seconds, 300);
        assert_eq!(config.concurrency_limit, 10);
        assert_eq!(config.metrics.len(), 3);
        assert!(!config.statistical_tests);
    }

    #[test]
    fn test_task_type_classification() {
        assert_eq!(TaskType::classify_prompt("Write a function to sort an array"), TaskType::Coding);
        assert_eq!(TaskType::classify_prompt("Summarize this article"), TaskType::Summarization);
        assert_eq!(TaskType::classify_prompt("Translate to French"), TaskType::Translation);
        assert_eq!(TaskType::classify_prompt("Classify this sentiment"), TaskType::Classification);
        assert_eq!(TaskType::classify_prompt("Write a creative story"), TaskType::Creative);
        assert_eq!(TaskType::classify_prompt("Solve this math problem"), TaskType::Reasoning);
        assert_eq!(TaskType::classify_prompt("What is Rust?"), TaskType::QuestionAnswering);
        assert_eq!(TaskType::classify_prompt("Hello there"), TaskType::General);
    }

    #[test]
    fn test_model_constraints_builder() {
        let constraints = ModelConstraints::new()
            .with_max_cost(0.01)
            .with_max_latency_ms(1000)
            .with_min_quality(0.8)
            .with_min_context_length(4096);

        assert_eq!(constraints.max_cost, Some(0.01));
        assert_eq!(constraints.max_latency_ms, Some(1000));
        assert_eq!(constraints.min_quality, 0.8);
        assert_eq!(constraints.min_context_length, Some(4096));
    }

    #[test]
    fn test_model_result_average_quality_score() {
        let mut scores = HashMap::new();
        scores.insert("faithfulness".to_string(), 0.9);
        scores.insert("relevance".to_string(), 0.8);
        scores.insert("coherence".to_string(), 0.85);

        let result = ModelResult {
            model_config: ModelConfig::new("openai", "gpt-4"),
            success: true,
            response: None,
            evaluation_scores: scores,
            latency_ms: 1000,
            cost: 0.01,
            error: None,
        };

        let avg = result.average_quality_score();
        assert!((avg - 0.85).abs() < 0.01);
    }

    #[test]
    fn test_model_profile_update() {
        let mut profile = ModelProfile::new("openai/gpt-4");
        profile.typical_quality = 0.8;
        profile.avg_latency_ms = 1000;

        let result = ModelResult {
            model_config: ModelConfig::new("openai", "gpt-4"),
            success: true,
            response: None,
            evaluation_scores: vec![("test".to_string(), 0.9)].into_iter().collect(),
            latency_ms: 1200,
            cost: 0.01,
            error: None,
        };

        profile.update_from_result(&result, 0.2);

        // Quality should have moved towards 0.9
        assert!(profile.typical_quality > 0.8);
        assert!(profile.typical_quality < 0.9);

        // Latency should have moved towards 1200
        assert!(profile.avg_latency_ms > 1000);
        assert!(profile.avg_latency_ms < 1200);

        assert_eq!(profile.sample_count, 1);
    }
}

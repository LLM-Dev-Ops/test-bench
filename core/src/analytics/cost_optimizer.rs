// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Cost optimization and analysis for LLM usage.
//!
//! This module provides tools to analyze and optimize the costs of LLM usage,
//! including model recommendations, savings calculations, and pattern detection.
//!
//! # Features
//!
//! - Model cost comparison and recommendations
//! - Savings calculations (per-request, monthly, annual)
//! - Detection of expensive usage patterns
//! - Prompt optimization suggestions
//!
//! # Examples
//!
//! ```
//! use llm_test_bench_core::analytics::CostOptimizer;
//!
//! let optimizer = CostOptimizer::new(0.95); // 95% quality threshold
//!
//! // Calculate potential savings
//! let savings = optimizer.calculate_savings(
//!     "gpt-4",
//!     "gpt-3.5-turbo",
//!     100_000, // monthly requests
//! );
//! println!("Monthly savings: ${:.2}", savings);
//! ```

use crate::benchmarks::results::BenchmarkResults;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Cost optimizer for finding cost-effective alternatives.
///
/// Analyzes benchmark results to identify opportunities for cost savings
/// while maintaining acceptable quality levels.
#[derive(Debug, Clone)]
pub struct CostOptimizer {
    /// Minimum acceptable quality threshold (0.0 - 1.0)
    pub quality_threshold: f64,
}

impl CostOptimizer {
    /// Creates a new cost optimizer with the specified quality threshold.
    ///
    /// # Arguments
    ///
    /// * `quality_threshold` - Minimum acceptable quality (0.0 - 1.0)
    ///
    /// # Examples
    ///
    /// ```
    /// use llm_test_bench_core::analytics::CostOptimizer;
    ///
    /// let optimizer = CostOptimizer::new(0.95); // Require 95% quality
    /// ```
    pub fn new(quality_threshold: f64) -> Self {
        Self { quality_threshold }
    }

    /// Creates a default optimizer with 95% quality threshold.
    pub fn default() -> Self {
        Self::new(0.95)
    }

    /// Recommends the most cost-effective model based on benchmark results.
    ///
    /// Analyzes all benchmarks and recommends a model that provides the best
    /// cost-to-quality ratio while maintaining the quality threshold.
    ///
    /// # Arguments
    ///
    /// * `results` - Vector of benchmark results to analyze
    ///
    /// # Returns
    ///
    /// A `CostRecommendation` with the recommended model and savings analysis.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use llm_test_bench_core::analytics::CostOptimizer;
    /// use llm_test_bench_core::benchmarks::results::BenchmarkResults;
    ///
    /// let optimizer = CostOptimizer::new(0.95);
    /// # let results = vec![BenchmarkResults::new("test".into(), "provider".into(), vec![])];
    /// let recommendation = optimizer.recommend_model(&results).unwrap();
    /// println!("Recommended: {}", recommendation.recommended_model);
    /// println!("Monthly savings: ${:.2}", recommendation.monthly_savings);
    /// ```
    pub fn recommend_model(&self, results: &[BenchmarkResults]) -> Result<CostRecommendation> {
        if results.is_empty() {
            return Err(anyhow!("No benchmark results provided"));
        }

        // Calculate cost and quality metrics for each model
        let mut model_metrics: Vec<ModelMetrics> = results
            .iter()
            .map(|r| {
                let quality = r.summary.success_rate;
                let cost_per_request = r.summary.total_cost / r.summary.total as f64;
                let avg_tokens = r.summary.total_tokens as f64 / r.summary.total as f64;

                ModelMetrics {
                    model_name: r.provider_name.clone(),
                    quality,
                    cost_per_request,
                    avg_tokens,
                    avg_latency_ms: r.summary.avg_duration_ms,
                }
            })
            .filter(|m| m.quality >= self.quality_threshold)
            .collect();

        if model_metrics.is_empty() {
            return Err(anyhow!(
                "No models meet the quality threshold of {:.1}%",
                self.quality_threshold * 100.0
            ));
        }

        // Sort by cost (ascending)
        model_metrics.sort_by(|a, b| {
            a.cost_per_request
                .partial_cmp(&b.cost_per_request)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Find current model (most expensive above threshold)
        let current = model_metrics.last().unwrap();
        let recommended = &model_metrics[0];

        // Calculate savings
        let cost_diff = current.cost_per_request - recommended.cost_per_request;
        let quality_delta = recommended.quality - current.quality;

        let monthly_savings = cost_diff * 30.0 * 24.0 * 100.0; // Assuming 100 req/hr
        let annual_savings = monthly_savings * 12.0;

        // Calculate confidence based on quality similarity and sample size
        let quality_similarity = 1.0 - (quality_delta.abs() / current.quality);
        let confidence = quality_similarity * 0.9; // Conservative estimate

        // Generate reasoning
        let reasoning = format!(
            "Model '{}' provides {:.1}% quality at ${:.6} per request, \
             compared to '{}' at {:.1}% quality and ${:.6} per request. \
             The cost is {:.1}% lower with {:.1}% quality difference.",
            recommended.model_name,
            recommended.quality * 100.0,
            recommended.cost_per_request,
            current.model_name,
            current.quality * 100.0,
            current.cost_per_request,
            (cost_diff / current.cost_per_request) * 100.0,
            quality_delta * 100.0
        );

        Ok(CostRecommendation {
            recommended_model: recommended.model_name.clone(),
            current_cost_per_request: current.cost_per_request,
            recommended_cost_per_request: recommended.cost_per_request,
            monthly_savings,
            annual_savings,
            quality_delta,
            reasoning,
            confidence,
        })
    }

    /// Calculates potential savings from switching models.
    ///
    /// # Arguments
    ///
    /// * `current_model` - Current model identifier
    /// * `recommended_model` - Recommended model identifier
    /// * `monthly_requests` - Estimated monthly request volume
    ///
    /// # Returns
    ///
    /// Monthly savings in USD.
    ///
    /// # Examples
    ///
    /// ```
    /// use llm_test_bench_core::analytics::CostOptimizer;
    ///
    /// let optimizer = CostOptimizer::new(0.95);
    /// let savings = optimizer.calculate_savings("gpt-4", "gpt-3.5-turbo", 100_000);
    /// println!("Monthly savings: ${:.2}", savings);
    /// ```
    pub fn calculate_savings(
        &self,
        current_model: &str,
        recommended_model: &str,
        monthly_requests: usize,
    ) -> f64 {
        let pricing = get_model_pricing();

        let current_cost = pricing
            .get(current_model)
            .copied()
            .unwrap_or((0.03, 0.06));
        let recommended_cost = pricing
            .get(recommended_model)
            .copied()
            .unwrap_or((0.01, 0.02));

        // Assume average token usage: 100 prompt, 50 completion
        let avg_prompt_tokens = 100.0;
        let avg_completion_tokens = 50.0;

        let current_cost_per_request = (avg_prompt_tokens / 1000.0 * current_cost.0)
            + (avg_completion_tokens / 1000.0 * current_cost.1);
        let recommended_cost_per_request = (avg_prompt_tokens / 1000.0 * recommended_cost.0)
            + (avg_completion_tokens / 1000.0 * recommended_cost.1);

        let cost_diff = current_cost_per_request - recommended_cost_per_request;
        cost_diff * monthly_requests as f64
    }

    /// Identifies expensive usage patterns in benchmark history.
    ///
    /// # Arguments
    ///
    /// * `history` - Historical benchmark results
    ///
    /// # Returns
    ///
    /// A vector of identified expensive patterns.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use llm_test_bench_core::analytics::CostOptimizer;
    /// use llm_test_bench_core::benchmarks::results::BenchmarkResults;
    ///
    /// let optimizer = CostOptimizer::new(0.95);
    /// # let history = vec![BenchmarkResults::new("test".into(), "provider".into(), vec![])];
    /// let patterns = optimizer.identify_expensive_patterns(&history);
    /// for pattern in patterns {
    ///     println!("{}: ${:.2} potential savings", pattern.description, pattern.potential_savings);
    /// }
    /// ```
    pub fn identify_expensive_patterns(
        &self,
        history: &[BenchmarkResults],
    ) -> Vec<ExpensivePattern> {
        let mut patterns = Vec::new();

        for results in history {
            // Pattern 1: Long prompts (>1000 tokens average)
            let avg_prompt_tokens: f64 = results
                .results
                .iter()
                .filter_map(|r| r.response.as_ref().map(|resp| resp.usage.prompt_tokens as f64))
                .sum::<f64>()
                / results.results.len().max(1) as f64;

            if avg_prompt_tokens > 1000.0 {
                let excess_cost = (avg_prompt_tokens - 800.0) / 1000.0 * 0.03 * results.total_tests as f64;
                patterns.push(ExpensivePattern {
                    pattern_type: PatternType::LongPrompts,
                    description: format!(
                        "Long prompts detected (avg {:.0} tokens). Consider compression or summarization.",
                        avg_prompt_tokens
                    ),
                    average_cost: results.summary.total_cost / results.total_tests.max(1) as f64,
                    frequency: results.total_tests,
                    potential_savings: excess_cost,
                });
            }

            // Pattern 2: Verbose responses (>500 completion tokens average)
            let avg_completion_tokens: f64 = results
                .results
                .iter()
                .filter_map(|r| {
                    r.response
                        .as_ref()
                        .map(|resp| resp.usage.completion_tokens as f64)
                })
                .sum::<f64>()
                / results.results.len().max(1) as f64;

            if avg_completion_tokens > 500.0 {
                let excess_cost = (avg_completion_tokens - 300.0) / 1000.0 * 0.06 * results.total_tests as f64;
                patterns.push(ExpensivePattern {
                    pattern_type: PatternType::VerboseResponses,
                    description: format!(
                        "Verbose responses detected (avg {:.0} tokens). Consider setting max_tokens limit.",
                        avg_completion_tokens
                    ),
                    average_cost: results.summary.total_cost / results.total_tests.max(1) as f64,
                    frequency: results.total_tests,
                    potential_savings: excess_cost,
                });
            }

            // Pattern 3: Expensive model for simple tasks
            let avg_cost_per_request = results.summary.total_cost / results.total_tests.max(1) as f64;
            if avg_cost_per_request > 0.05 && results.summary.success_rate > 0.98 {
                patterns.push(ExpensivePattern {
                    pattern_type: PatternType::ExpensiveModel,
                    description: format!(
                        "Using expensive model ({}) with {:.1}% success rate. \
                         Consider cheaper alternatives for high-success tasks.",
                        results.provider_name, results.summary.success_rate * 100.0
                    ),
                    average_cost: avg_cost_per_request,
                    frequency: results.total_tests,
                    potential_savings: avg_cost_per_request * 0.5 * results.total_tests as f64,
                });
            }
        }

        patterns
    }

    /// Suggests prompt optimizations based on benchmark results.
    ///
    /// # Arguments
    ///
    /// * `results` - Benchmark results to analyze
    ///
    /// # Returns
    ///
    /// A vector of optimization suggestions.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use llm_test_bench_core::analytics::CostOptimizer;
    /// use llm_test_bench_core::benchmarks::results::BenchmarkResults;
    ///
    /// let optimizer = CostOptimizer::new(0.95);
    /// # let results = BenchmarkResults::new("test".into(), "provider".into(), vec![]);
    /// let suggestions = optimizer.suggest_prompt_optimizations(&results);
    /// for suggestion in suggestions {
    ///     println!("{}: {}", suggestion.title, suggestion.description);
    /// }
    /// ```
    pub fn suggest_prompt_optimizations(
        &self,
        results: &BenchmarkResults,
    ) -> Vec<OptimizationSuggestion> {
        let mut suggestions = Vec::new();

        // Calculate average token usage
        let avg_prompt_tokens: f64 = results
            .results
            .iter()
            .filter_map(|r| r.response.as_ref().map(|resp| resp.usage.prompt_tokens as f64))
            .sum::<f64>()
            / results.results.len().max(1) as f64;

        let avg_completion_tokens: f64 = results
            .results
            .iter()
            .filter_map(|r| r.response.as_ref().map(|resp| resp.usage.completion_tokens as f64))
            .sum::<f64>()
            / results.results.len().max(1) as f64;

        // Suggestion 1: Prompt compression
        if avg_prompt_tokens > 1000.0 {
            suggestions.push(OptimizationSuggestion {
                title: "Compress Long Prompts".to_string(),
                description: format!(
                    "Your prompts average {:.0} tokens. Consider:\n\
                     - Removing redundant instructions\n\
                     - Using more concise language\n\
                     - Splitting into multiple focused prompts",
                    avg_prompt_tokens
                ),
                estimated_savings: (avg_prompt_tokens - 700.0) / 1000.0 * 0.03 * results.total_tests as f64,
                implementation_effort: ImplementationEffort::Medium,
            });
        }

        // Suggestion 2: Response length limits
        if avg_completion_tokens > 500.0 {
            suggestions.push(OptimizationSuggestion {
                title: "Set Response Length Limits".to_string(),
                description: format!(
                    "Responses average {:.0} tokens. Consider:\n\
                     - Setting max_tokens parameter\n\
                     - Requesting more concise responses\n\
                     - Using structured output formats",
                    avg_completion_tokens
                ),
                estimated_savings: (avg_completion_tokens - 300.0) / 1000.0 * 0.06 * results.total_tests as f64,
                implementation_effort: ImplementationEffort::Low,
            });
        }

        // Suggestion 3: Batch processing
        if results.total_tests > 100 {
            suggestions.push(OptimizationSuggestion {
                title: "Use Batch Processing".to_string(),
                description: "For high-volume requests, consider:\n\
                     - Batching similar requests\n\
                     - Caching common responses\n\
                     - Using async processing"
                    .to_string(),
                estimated_savings: results.summary.total_cost * 0.1, // ~10% savings
                implementation_effort: ImplementationEffort::High,
            });
        }

        // Suggestion 4: Temperature optimization
        if results.summary.success_rate > 0.95 {
            suggestions.push(OptimizationSuggestion {
                title: "Optimize Temperature Settings".to_string(),
                description: format!(
                    "With {:.1}% success rate, you might:\n\
                     - Use lower temperature for deterministic tasks\n\
                     - Reduce model variability\n\
                     - Improve response consistency",
                    results.summary.success_rate * 100.0
                ),
                estimated_savings: 0.0, // Indirect savings through improved consistency
                implementation_effort: ImplementationEffort::Low,
            });
        }

        suggestions
    }
}

/// Cost recommendation with detailed analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostRecommendation {
    /// Recommended model identifier
    pub recommended_model: String,
    /// Current cost per request (USD)
    pub current_cost_per_request: f64,
    /// Recommended model cost per request (USD)
    pub recommended_cost_per_request: f64,
    /// Estimated monthly savings (USD)
    pub monthly_savings: f64,
    /// Estimated annual savings (USD)
    pub annual_savings: f64,
    /// Quality difference (positive = improvement, negative = degradation)
    pub quality_delta: f64,
    /// Plain-language reasoning for recommendation
    pub reasoning: String,
    /// Confidence in recommendation (0.0 - 1.0)
    pub confidence: f64,
}

/// An identified expensive usage pattern.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpensivePattern {
    /// Type of pattern detected
    pub pattern_type: PatternType,
    /// Description of the pattern
    pub description: String,
    /// Average cost per occurrence (USD)
    pub average_cost: f64,
    /// Number of occurrences
    pub frequency: usize,
    /// Estimated potential savings (USD)
    pub potential_savings: f64,
}

/// Types of expensive usage patterns.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PatternType {
    /// Prompts that are longer than necessary
    LongPrompts,
    /// Responses that are more verbose than needed
    VerboseResponses,
    /// Using expensive models for simple tasks
    ExpensiveModel,
    /// Suboptimal temperature settings
    HighTemperature,
}

impl std::fmt::Display for PatternType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PatternType::LongPrompts => write!(f, "Long Prompts"),
            PatternType::VerboseResponses => write!(f, "Verbose Responses"),
            PatternType::ExpensiveModel => write!(f, "Expensive Model"),
            PatternType::HighTemperature => write!(f, "High Temperature"),
        }
    }
}

/// A suggestion for optimizing prompts or usage patterns.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationSuggestion {
    /// Title of the suggestion
    pub title: String,
    /// Detailed description and implementation steps
    pub description: String,
    /// Estimated cost savings (USD)
    pub estimated_savings: f64,
    /// Implementation effort required
    pub implementation_effort: ImplementationEffort,
}

/// Estimated effort to implement an optimization.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ImplementationEffort {
    /// Quick changes (< 1 hour)
    Low,
    /// Moderate changes (1-4 hours)
    Medium,
    /// Significant changes (> 4 hours)
    High,
}

impl std::fmt::Display for ImplementationEffort {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImplementationEffort::Low => write!(f, "Low"),
            ImplementationEffort::Medium => write!(f, "Medium"),
            ImplementationEffort::High => write!(f, "High"),
        }
    }
}

// Internal helper structures

#[derive(Debug, Clone)]
struct ModelMetrics {
    model_name: String,
    quality: f64,
    cost_per_request: f64,
    avg_tokens: f64,
    avg_latency_ms: f64,
}

/// Returns pricing information for common models.
/// Format: (prompt_cost_per_1k, completion_cost_per_1k)
fn get_model_pricing() -> HashMap<&'static str, (f64, f64)> {
    let mut pricing = HashMap::new();

    // OpenAI GPT-4 models
    pricing.insert("gpt-4", (0.03, 0.06));
    pricing.insert("gpt-4-turbo", (0.01, 0.03));
    pricing.insert("gpt-4o", (0.005, 0.015));
    pricing.insert("gpt-4o-mini", (0.00015, 0.0006));

    // OpenAI GPT-3.5 models
    pricing.insert("gpt-3.5-turbo", (0.0015, 0.002));
    pricing.insert("gpt-3.5-turbo-16k", (0.003, 0.004));

    // Anthropic Claude models
    pricing.insert("claude-3-opus", (0.015, 0.075));
    pricing.insert("claude-3-sonnet", (0.003, 0.015));
    pricing.insert("claude-3-haiku", (0.00025, 0.00125));
    pricing.insert("claude-3-5-sonnet", (0.003, 0.015));

    pricing
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::benchmarks::results::{BenchmarkResults, TestResult};
    use crate::providers::{CompletionResponse, FinishReason, TokenUsage};
    use chrono::Utc;
    use std::time::Duration;

    fn create_test_result(prompt_tokens: usize, completion_tokens: usize) -> TestResult {
        let response = CompletionResponse {
            id: "test".to_string(),
            model: "gpt-4".to_string(),
            content: "test".to_string(),
            usage: TokenUsage::new(prompt_tokens, completion_tokens),
            finish_reason: FinishReason::Stop,
            created_at: Utc::now(),
        };

        TestResult::success(
            "test-1".to_string(),
            None,
            response,
            Duration::from_millis(1000),
        )
    }

    #[test]
    fn test_cost_optimizer_new() {
        let optimizer = CostOptimizer::new(0.9);
        assert_eq!(optimizer.quality_threshold, 0.9);
    }

    #[test]
    fn test_cost_optimizer_default() {
        let optimizer = CostOptimizer::default();
        assert_eq!(optimizer.quality_threshold, 0.95);
    }

    #[test]
    fn test_calculate_savings() {
        let optimizer = CostOptimizer::new(0.95);
        let savings = optimizer.calculate_savings("gpt-4", "gpt-3.5-turbo", 100_000);
        assert!(savings > 0.0); // Should show savings
    }

    #[test]
    fn test_calculate_savings_same_model() {
        let optimizer = CostOptimizer::new(0.95);
        let savings = optimizer.calculate_savings("gpt-4", "gpt-4", 100_000);
        assert_eq!(savings, 0.0); // No savings for same model
    }

    #[test]
    fn test_identify_expensive_patterns_long_prompts() {
        let optimizer = CostOptimizer::new(0.95);

        // Create results with long prompts
        let results = vec![
            create_test_result(1500, 100), // Long prompt
            create_test_result(1200, 100),
            create_test_result(1600, 100),
        ];

        let mut benchmark = BenchmarkResults::new("test".to_string(), "gpt-4".to_string(), results);
        benchmark.calculate_summary();

        let patterns = optimizer.identify_expensive_patterns(&[benchmark]);
        assert!(!patterns.is_empty());

        let long_prompt_pattern = patterns
            .iter()
            .find(|p| matches!(p.pattern_type, PatternType::LongPrompts));
        assert!(long_prompt_pattern.is_some());
    }

    #[test]
    fn test_identify_expensive_patterns_verbose_responses() {
        let optimizer = CostOptimizer::new(0.95);

        // Create results with verbose responses
        let results = vec![
            create_test_result(100, 600), // Verbose response
            create_test_result(100, 700),
            create_test_result(100, 550),
        ];

        let mut benchmark = BenchmarkResults::new("test".to_string(), "gpt-4".to_string(), results);
        benchmark.calculate_summary();

        let patterns = optimizer.identify_expensive_patterns(&[benchmark]);
        assert!(!patterns.is_empty());

        let verbose_pattern = patterns
            .iter()
            .find(|p| matches!(p.pattern_type, PatternType::VerboseResponses));
        assert!(verbose_pattern.is_some());
    }

    #[test]
    fn test_suggest_prompt_optimizations_long_prompts() {
        let optimizer = CostOptimizer::new(0.95);

        let results = vec![create_test_result(1500, 100), create_test_result(1200, 100)];

        let mut benchmark = BenchmarkResults::new("test".to_string(), "gpt-4".to_string(), results);
        benchmark.calculate_summary();

        let suggestions = optimizer.suggest_prompt_optimizations(&benchmark);
        assert!(!suggestions.is_empty());

        let compress_suggestion = suggestions
            .iter()
            .find(|s| s.title.contains("Compress"));
        assert!(compress_suggestion.is_some());
    }

    #[test]
    fn test_suggest_prompt_optimizations_verbose_responses() {
        let optimizer = CostOptimizer::new(0.95);

        let results = vec![create_test_result(100, 600), create_test_result(100, 700)];

        let mut benchmark = BenchmarkResults::new("test".to_string(), "gpt-4".to_string(), results);
        benchmark.calculate_summary();

        let suggestions = optimizer.suggest_prompt_optimizations(&benchmark);
        assert!(!suggestions.is_empty());

        let length_limit_suggestion = suggestions.iter().find(|s| s.title.contains("Length"));
        assert!(length_limit_suggestion.is_some());
    }

    #[test]
    fn test_pattern_type_display() {
        assert_eq!(PatternType::LongPrompts.to_string(), "Long Prompts");
        assert_eq!(PatternType::VerboseResponses.to_string(), "Verbose Responses");
        assert_eq!(PatternType::ExpensiveModel.to_string(), "Expensive Model");
        assert_eq!(PatternType::HighTemperature.to_string(), "High Temperature");
    }

    #[test]
    fn test_implementation_effort_display() {
        assert_eq!(ImplementationEffort::Low.to_string(), "Low");
        assert_eq!(ImplementationEffort::Medium.to_string(), "Medium");
        assert_eq!(ImplementationEffort::High.to_string(), "High");
    }

    #[test]
    fn test_get_model_pricing() {
        let pricing = get_model_pricing();
        assert!(pricing.contains_key("gpt-4"));
        assert!(pricing.contains_key("gpt-3.5-turbo"));
        assert!(pricing.contains_key("claude-3-opus"));

        let gpt4_pricing = pricing.get("gpt-4").unwrap();
        assert_eq!(gpt4_pricing, &(0.03, 0.06));
    }

    #[test]
    fn test_recommend_model_empty_results() {
        let optimizer = CostOptimizer::new(0.95);
        let result = optimizer.recommend_model(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_recommend_model_below_threshold() {
        let optimizer = CostOptimizer::new(0.99); // Very high threshold

        // Create result with 95% success rate (below threshold)
        let results = vec![
            create_test_result(100, 50),
            create_test_result(100, 50),
        ];

        let mut benchmark = BenchmarkResults::new("test".to_string(), "gpt-4".to_string(), results);
        benchmark.calculate_summary();
        benchmark.summary.success_rate = 0.95; // Below 0.99 threshold

        let result = optimizer.recommend_model(&[benchmark]);
        assert!(result.is_err());
    }

    #[test]
    fn test_cost_recommendation_serialization() {
        let recommendation = CostRecommendation {
            recommended_model: "gpt-3.5-turbo".to_string(),
            current_cost_per_request: 0.06,
            recommended_cost_per_request: 0.002,
            monthly_savings: 500.0,
            annual_savings: 6000.0,
            quality_delta: -0.02,
            reasoning: "Test reasoning".to_string(),
            confidence: 0.85,
        };

        let json = serde_json::to_string(&recommendation).unwrap();
        let deserialized: CostRecommendation = serde_json::from_str(&json).unwrap();
        assert_eq!(recommendation.recommended_model, deserialized.recommended_model);
    }

    #[test]
    fn test_expensive_pattern_serialization() {
        let pattern = ExpensivePattern {
            pattern_type: PatternType::LongPrompts,
            description: "Test pattern".to_string(),
            average_cost: 0.05,
            frequency: 100,
            potential_savings: 5.0,
        };

        let json = serde_json::to_string(&pattern).unwrap();
        let deserialized: ExpensivePattern = serde_json::from_str(&json).unwrap();
        assert_eq!(pattern.pattern_type, deserialized.pattern_type);
    }

    #[test]
    fn test_optimization_suggestion_serialization() {
        let suggestion = OptimizationSuggestion {
            title: "Test Suggestion".to_string(),
            description: "Test description".to_string(),
            estimated_savings: 100.0,
            implementation_effort: ImplementationEffort::Medium,
        };

        let json = serde_json::to_string(&suggestion).unwrap();
        let deserialized: OptimizationSuggestion = serde_json::from_str(&json).unwrap();
        assert_eq!(suggestion.title, deserialized.title);
    }
}

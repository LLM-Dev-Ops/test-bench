// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Multi-model comparison engine.
//!
//! This module provides functionality for executing multiple LLM models in parallel
//! and comparing their responses using various evaluation metrics.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use futures::stream::{self, StreamExt};
use thiserror::Error;
use tokio::time::timeout;

use crate::evaluators::{Evaluator, EvaluatorError};
use crate::providers::error::ProviderError;
use crate::providers::traits::Provider;
use crate::providers::types::{CompletionRequest, CompletionResponse};

use super::ranking::RankingEngine;
use super::types::{
    ComparisonConfig, ComparisonResult, ComparativeAnalysis, ModelConfig, ModelResult,
};

/// Errors that can occur during comparison.
#[derive(Error, Debug)]
pub enum ComparisonError {
    /// No models provided for comparison
    #[error("No models provided for comparison")]
    NoModels,

    /// Provider not found
    #[error("Provider not found: {0}")]
    ProviderNotFound(String),

    /// Evaluation failed
    #[error("Evaluation failed: {0}")]
    EvaluationFailed(String),

    /// Comparison timed out
    #[error("Comparison timed out after {0} seconds")]
    Timeout(u64),

    /// All models failed
    #[error("All models failed to execute")]
    AllModelsFailed,

    /// Invalid configuration
    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),
}

/// Multi-model comparison engine.
///
/// Executes multiple models in parallel, evaluates their responses,
/// and generates comparative rankings and analysis.
pub struct ComparisonEngine {
    /// Available providers by name
    providers: HashMap<String, Arc<dyn Provider>>,

    /// Available evaluators by metric name
    evaluators: HashMap<String, Arc<dyn Evaluator>>,

    /// Ranking engine for generating rankings
    ranking_engine: RankingEngine,
}

impl ComparisonEngine {
    /// Create a new comparison engine.
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
            evaluators: HashMap::new(),
            ranking_engine: RankingEngine::new(),
        }
    }

    /// Register a provider with the engine.
    pub fn register_provider(&mut self, name: impl Into<String>, provider: Arc<dyn Provider>) {
        self.providers.insert(name.into(), provider);
    }

    /// Register an evaluator with the engine.
    pub fn register_evaluator(&mut self, metric: impl Into<String>, evaluator: Arc<dyn Evaluator>) {
        self.evaluators.insert(metric.into(), evaluator);
    }

    /// Run a comparison across multiple models.
    ///
    /// # Arguments
    ///
    /// * `prompt` - The prompt to send to all models
    /// * `config` - Comparison configuration
    ///
    /// # Returns
    ///
    /// A `ComparisonResult` containing model results, rankings, and analysis.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - No models are provided
    /// - Required providers are not registered
    /// - The operation times out
    /// - All models fail to execute
    pub async fn compare(
        &self,
        prompt: &str,
        config: ComparisonConfig,
    ) -> Result<ComparisonResult, ComparisonError> {
        let start_time = Instant::now();

        // Validate configuration
        self.validate_config(&config)?;

        // Execute all models in parallel with timeout
        let timeout_duration = Duration::from_secs(config.timeout_seconds);
        let model_results = timeout(timeout_duration, self.execute_models(prompt, &config))
            .await
            .map_err(|_| ComparisonError::Timeout(config.timeout_seconds))??;

        // Check if at least one model succeeded
        if !model_results.iter().any(|r| r.success) {
            return Err(ComparisonError::AllModelsFailed);
        }

        // Generate rankings
        let rankings = self
            .ranking_engine
            .calculate_rankings(&model_results)
            .map_err(|e| ComparisonError::EvaluationFailed(e.to_string()))?;

        // Determine winner
        let winner = rankings.first().map(|r| r.model_config.identifier());

        // Generate comparative analysis
        let comparative_analysis =
            self.ranking_engine
                .generate_comparative_analysis(&model_results, &rankings);

        // Statistical significance testing (if enabled)
        let statistical_significance = if config.statistical_tests {
            Some(self.ranking_engine.run_significance_tests(&model_results))
        } else {
            None
        };

        let total_duration = start_time.elapsed();

        Ok(ComparisonResult {
            models: model_results,
            rankings,
            winner,
            comparative_analysis,
            statistical_significance,
            total_duration,
        })
    }

    /// Validate the comparison configuration.
    fn validate_config(&self, config: &ComparisonConfig) -> Result<(), ComparisonError> {
        if config.models.is_empty() {
            return Err(ComparisonError::NoModels);
        }

        // Check that all required providers are registered
        for model_config in &config.models {
            if !self.providers.contains_key(&model_config.provider) {
                return Err(ComparisonError::ProviderNotFound(
                    model_config.provider.clone(),
                ));
            }
        }

        // Check that requested metrics have registered evaluators
        for metric in &config.metrics {
            if !self.evaluators.contains_key(metric) {
                return Err(ComparisonError::InvalidConfiguration(format!(
                    "Evaluator not registered for metric: {}",
                    metric
                )));
            }
        }

        Ok(())
    }

    /// Execute all models in parallel.
    async fn execute_models(
        &self,
        prompt: &str,
        config: &ComparisonConfig,
    ) -> Result<Vec<ModelResult>, ComparisonError> {
        // Create a stream of model execution futures
        let model_futures = config.models.iter().map(|model_config| {
            let prompt = prompt.to_string();
            let model_config = model_config.clone();
            let config = config.clone();

            async move {
                self.execute_single_model(&prompt, &model_config, &config)
                    .await
            }
        });

        // Execute with concurrency limit
        let results: Vec<ModelResult> = stream::iter(model_futures)
            .buffer_unordered(config.concurrency_limit)
            .collect()
            .await;

        Ok(results)
    }

    /// Execute a single model and evaluate its response.
    async fn execute_single_model(
        &self,
        prompt: &str,
        model_config: &ModelConfig,
        config: &ComparisonConfig,
    ) -> ModelResult {
        let start_time = Instant::now();

        // Get the provider
        let provider = match self.providers.get(&model_config.provider) {
            Some(p) => p,
            None => {
                return ModelResult {
                    model_config: model_config.clone(),
                    success: false,
                    response: None,
                    evaluation_scores: HashMap::new(),
                    latency_ms: 0,
                    cost: 0.0,
                    error: Some(format!("Provider not found: {}", model_config.provider)),
                };
            }
        };

        // Build completion request
        let mut request = CompletionRequest::new(&model_config.model, prompt);

        // Apply parameters from model config
        if let Some(temp) = model_config
            .parameters
            .get("temperature")
            .and_then(|v| v.as_f64())
        {
            request = request.with_temperature(temp as f32);
        }
        if let Some(max_tokens) = model_config
            .parameters
            .get("max_tokens")
            .and_then(|v| v.as_u64())
        {
            request = request.with_max_tokens(max_tokens as usize);
        }

        // Execute the request
        let response_result = provider.complete(request).await;
        let latency_ms = start_time.elapsed().as_millis() as u64;

        match response_result {
            Ok(response) => {
                // Calculate cost (estimated)
                let cost = self.estimate_cost(&response, model_config);

                // Evaluate the response
                let evaluation_scores = self.evaluate_response(prompt, &response.content, config).await;

                ModelResult {
                    model_config: model_config.clone(),
                    success: true,
                    response: Some(response),
                    evaluation_scores,
                    latency_ms,
                    cost,
                    error: None,
                }
            }
            Err(e) => ModelResult {
                model_config: model_config.clone(),
                success: false,
                response: None,
                evaluation_scores: HashMap::new(),
                latency_ms,
                cost: 0.0,
                error: Some(e.to_string()),
            },
        }
    }

    /// Evaluate a model's response using registered evaluators.
    async fn evaluate_response(
        &self,
        prompt: &str,
        response: &str,
        config: &ComparisonConfig,
    ) -> HashMap<String, f64> {
        let mut scores = HashMap::new();

        for metric in &config.metrics {
            if let Some(evaluator) = self.evaluators.get(metric) {
                match evaluator.evaluate(prompt, response).await {
                    Ok(result) => {
                        scores.insert(metric.clone(), result.score);
                    }
                    Err(e) => {
                        tracing::warn!("Evaluation failed for metric {}: {}", metric, e);
                        // Don't include failed metrics in scores
                    }
                }
            }
        }

        scores
    }

    /// Estimate the cost of a completion.
    ///
    /// Uses hardcoded pricing for common models. In production, this should
    /// be configurable or retrieved from a pricing service.
    fn estimate_cost(&self, response: &CompletionResponse, model_config: &ModelConfig) -> f64 {
        // Hardcoded pricing for common models (per 1K tokens)
        let (prompt_cost, completion_cost) = match model_config.model.as_str() {
            // OpenAI pricing
            "gpt-4" => (0.03, 0.06),
            "gpt-4-turbo" => (0.01, 0.03),
            "gpt-3.5-turbo" => (0.0005, 0.0015),
            // Anthropic pricing
            "claude-3-opus-20240229" => (0.015, 0.075),
            "claude-3-sonnet-20240229" => (0.003, 0.015),
            "claude-3-haiku-20240307" => (0.00025, 0.00125),
            // Default fallback
            _ => (0.001, 0.002),
        };

        response.usage.calculate_cost(prompt_cost, completion_cost)
    }

    /// Get the number of registered providers.
    pub fn provider_count(&self) -> usize {
        self.providers.len()
    }

    /// Get the number of registered evaluators.
    pub fn evaluator_count(&self) -> usize {
        self.evaluators.len()
    }
}

impl Default for ComparisonEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evaluators::{EvaluationResult, Evaluator, EvaluatorError};
    use crate::providers::types::{FinishReason, TokenUsage};
    use async_trait::async_trait;
    use chrono::Utc;

    // Mock Provider for testing
    struct MockProvider {
        name: String,
        latency_ms: u64,
        should_fail: bool,
    }

    #[async_trait]
    impl Provider for MockProvider {
        async fn complete(
            &self,
            request: CompletionRequest,
        ) -> Result<CompletionResponse, ProviderError> {
            if self.should_fail {
                return Err(ProviderError::ApiError {
                    status: 500,
                    message: "Mock error".to_string(),
                });
            }

            // Simulate latency
            tokio::time::sleep(Duration::from_millis(self.latency_ms)).await;

            Ok(CompletionResponse {
                id: format!("{}-completion", self.name),
                model: request.model.clone(),
                content: format!("Response from {}", self.name),
                usage: TokenUsage::new(50, 100),
                finish_reason: FinishReason::Stop,
                created_at: Utc::now(),
            })
        }

        async fn stream(
            &self,
            _request: CompletionRequest,
        ) -> Result<crate::providers::types::ResponseStream, ProviderError> {
            unimplemented!()
        }

        fn supported_models(&self) -> Vec<crate::providers::types::ModelInfo> {
            vec![]
        }

        fn max_context_length(&self, _model: &str) -> Option<usize> {
            Some(4096)
        }

        fn name(&self) -> &str {
            &self.name
        }

        async fn validate_config(&self) -> Result<(), ProviderError> {
            Ok(())
        }

        fn estimate_tokens(&self, text: &str, _model: &str) -> Result<usize, ProviderError> {
            Ok(text.split_whitespace().count())
        }
    }

    // Mock Evaluator for testing
    struct MockEvaluator {
        name: String,
        score: f64,
    }

    impl Evaluator for MockEvaluator {
        fn evaluate(&self, _prompt: &str, _response: &str) -> Result<EvaluationResult, EvaluatorError> {
            Ok(EvaluationResult {
                metric: self.name.clone(),
                score: self.score,
                details: serde_json::json!({}),
            })
        }

        fn name(&self) -> &str {
            &self.name
        }
    }

    #[tokio::test]
    async fn test_comparison_engine_basic() {
        let mut engine = ComparisonEngine::new();

        // Register providers
        engine.register_provider(
            "provider1",
            Arc::new(MockProvider {
                name: "provider1".to_string(),
                latency_ms: 100,
                should_fail: false,
            }),
        );
        engine.register_provider(
            "provider2",
            Arc::new(MockProvider {
                name: "provider2".to_string(),
                latency_ms: 200,
                should_fail: false,
            }),
        );

        // Register evaluators
        engine.register_evaluator(
            "quality",
            Arc::new(MockEvaluator {
                name: "quality".to_string(),
                score: 0.8,
            }),
        );

        let config = ComparisonConfig {
            models: vec![
                ModelConfig::new("provider1", "model1"),
                ModelConfig::new("provider2", "model2"),
            ],
            metrics: vec!["quality".to_string()],
            statistical_tests: false,
            timeout_seconds: 10,
            concurrency_limit: 2,
        };

        let result = engine.compare("Test prompt", config).await.unwrap();

        assert_eq!(result.models.len(), 2);
        assert!(result.models.iter().all(|m| m.success));
        assert_eq!(result.rankings.len(), 2);
        assert!(result.winner.is_some());
    }

    #[tokio::test]
    async fn test_comparison_with_failure() {
        let mut engine = ComparisonEngine::new();

        // One working, one failing provider
        engine.register_provider(
            "provider1",
            Arc::new(MockProvider {
                name: "provider1".to_string(),
                latency_ms: 100,
                should_fail: false,
            }),
        );
        engine.register_provider(
            "provider2",
            Arc::new(MockProvider {
                name: "provider2".to_string(),
                latency_ms: 200,
                should_fail: true,
            }),
        );

        engine.register_evaluator(
            "quality",
            Arc::new(MockEvaluator {
                name: "quality".to_string(),
                score: 0.8,
            }),
        );

        let config = ComparisonConfig {
            models: vec![
                ModelConfig::new("provider1", "model1"),
                ModelConfig::new("provider2", "model2"),
            ],
            metrics: vec!["quality".to_string()],
            statistical_tests: false,
            timeout_seconds: 10,
            concurrency_limit: 2,
        };

        let result = engine.compare("Test prompt", config).await.unwrap();

        assert_eq!(result.models.len(), 2);
        assert_eq!(result.models.iter().filter(|m| m.success).count(), 1);
        assert_eq!(result.models.iter().filter(|m| !m.success).count(), 1);
    }

    #[tokio::test]
    async fn test_all_models_failed() {
        let mut engine = ComparisonEngine::new();

        engine.register_provider(
            "provider1",
            Arc::new(MockProvider {
                name: "provider1".to_string(),
                latency_ms: 100,
                should_fail: true,
            }),
        );

        engine.register_evaluator(
            "quality",
            Arc::new(MockEvaluator {
                name: "quality".to_string(),
                score: 0.8,
            }),
        );

        let config = ComparisonConfig {
            models: vec![ModelConfig::new("provider1", "model1")],
            metrics: vec!["quality".to_string()],
            statistical_tests: false,
            timeout_seconds: 10,
            concurrency_limit: 1,
        };

        let result = engine.compare("Test prompt", config).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ComparisonError::AllModelsFailed));
    }

    #[tokio::test]
    async fn test_no_models_error() {
        let engine = ComparisonEngine::new();

        let config = ComparisonConfig {
            models: vec![],
            metrics: vec!["quality".to_string()],
            statistical_tests: false,
            timeout_seconds: 10,
            concurrency_limit: 1,
        };

        let result = engine.compare("Test prompt", config).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ComparisonError::NoModels));
    }

    #[tokio::test]
    async fn test_provider_not_found() {
        let engine = ComparisonEngine::new();

        let config = ComparisonConfig {
            models: vec![ModelConfig::new("nonexistent", "model1")],
            metrics: vec![],
            statistical_tests: false,
            timeout_seconds: 10,
            concurrency_limit: 1,
        };

        let result = engine.compare("Test prompt", config).await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ComparisonError::ProviderNotFound(_)
        ));
    }

    #[tokio::test]
    async fn test_parallel_execution() {
        let mut engine = ComparisonEngine::new();

        // Register 5 providers with different latencies
        for i in 1..=5 {
            engine.register_provider(
                format!("provider{}", i),
                Arc::new(MockProvider {
                    name: format!("provider{}", i),
                    latency_ms: 100,
                    should_fail: false,
                }),
            );
        }

        engine.register_evaluator(
            "quality",
            Arc::new(MockEvaluator {
                name: "quality".to_string(),
                score: 0.8,
            }),
        );

        let config = ComparisonConfig {
            models: (1..=5)
                .map(|i| ModelConfig::new(format!("provider{}", i), format!("model{}", i)))
                .collect(),
            metrics: vec!["quality".to_string()],
            statistical_tests: false,
            timeout_seconds: 10,
            concurrency_limit: 5,
        };

        let start = Instant::now();
        let result = engine.compare("Test prompt", config).await.unwrap();
        let duration = start.elapsed();

        assert_eq!(result.models.len(), 5);
        assert!(result.models.iter().all(|m| m.success));

        // With parallel execution, should take close to 100ms (the individual latency)
        // rather than 500ms (sum of all latencies)
        assert!(duration.as_millis() < 300);
    }

    #[test]
    fn test_engine_registration() {
        let mut engine = ComparisonEngine::new();

        assert_eq!(engine.provider_count(), 0);
        assert_eq!(engine.evaluator_count(), 0);

        engine.register_provider(
            "provider1",
            Arc::new(MockProvider {
                name: "provider1".to_string(),
                latency_ms: 100,
                should_fail: false,
            }),
        );

        engine.register_evaluator(
            "quality",
            Arc::new(MockEvaluator {
                name: "quality".to_string(),
                score: 0.8,
            }),
        );

        assert_eq!(engine.provider_count(), 1);
        assert_eq!(engine.evaluator_count(), 1);
    }
}

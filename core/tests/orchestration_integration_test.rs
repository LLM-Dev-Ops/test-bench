// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Integration tests for orchestration module.
//!
//! These tests verify that comparison, ranking, and routing work together correctly.

use llm_test_bench_core::evaluators::{EvaluationResult, Evaluator, EvaluatorError};
use llm_test_bench_core::orchestration::{
    ComparisonConfig, ComparisonEngine, ModelConfig, ModelConstraints, ModelRouter, RoutingStrategy,
};
use llm_test_bench_core::providers::error::ProviderError;
use llm_test_bench_core::providers::traits::Provider;
use llm_test_bench_core::providers::types::{CompletionRequest, CompletionResponse, FinishReason, ModelInfo, ResponseStream, TokenUsage};
use std::sync::Arc;
use async_trait::async_trait;
use chrono::Utc;

// Mock Provider for testing
struct TestProvider {
    name: String,
    latency_ms: u64,
    quality_multiplier: f64,
}

#[async_trait]
impl Provider for TestProvider {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse, ProviderError> {
        tokio::time::sleep(std::time::Duration::from_millis(self.latency_ms)).await;

        Ok(CompletionResponse {
            id: format!("{}-{}", self.name, std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()),
            model: request.model.clone(),
            content: format!(
                "Response from {} (quality: {}): {}",
                self.name, self.quality_multiplier, request.prompt
            ),
            usage: TokenUsage::new(100, 200),
            finish_reason: FinishReason::Stop,
            created_at: Utc::now(),
        })
    }

    async fn stream(&self, _request: CompletionRequest) -> Result<ResponseStream, ProviderError> {
        unimplemented!()
    }

    fn supported_models(&self) -> Vec<ModelInfo> {
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
struct TestEvaluator {
    name: String,
    base_score: f64,
}

impl Evaluator for TestEvaluator {
    fn evaluate(&self, _prompt: &str, response: &str) -> Result<EvaluationResult, EvaluatorError> {
        // Extract quality multiplier from response if present
        let score = if let Some(start) = response.find("quality: ") {
            let end = response[start..].find(")").unwrap_or(response.len());
            let quality_str = &response[start + 9..start + end];
            quality_str.parse::<f64>().unwrap_or(self.base_score) * self.base_score
        } else {
            self.base_score
        };

        Ok(EvaluationResult {
            metric: self.name.clone(),
            score: score.min(1.0),
            details: serde_json::json!({}),
        })
    }

    fn name(&self) -> &str {
        &self.name
    }
}

#[tokio::test]
async fn test_full_comparison_workflow() {
    let mut engine = ComparisonEngine::new();

    // Register providers with different characteristics
    engine.register_provider(
        "fast",
        Arc::new(TestProvider {
            name: "fast".to_string(),
            latency_ms: 50,
            quality_multiplier: 0.75,
        }),
    );

    engine.register_provider(
        "quality",
        Arc::new(TestProvider {
            name: "quality".to_string(),
            latency_ms: 200,
            quality_multiplier: 0.95,
        }),
    );

    engine.register_provider(
        "cheap",
        Arc::new(TestProvider {
            name: "cheap".to_string(),
            latency_ms: 100,
            quality_multiplier: 0.70,
        }),
    );

    // Register evaluators
    engine.register_evaluator(
        "faithfulness",
        Arc::new(TestEvaluator {
            name: "faithfulness".to_string(),
            base_score: 0.9,
        }),
    );

    engine.register_evaluator(
        "relevance",
        Arc::new(TestEvaluator {
            name: "relevance".to_string(),
            base_score: 0.85,
        }),
    );

    engine.register_evaluator(
        "coherence",
        Arc::new(TestEvaluator {
            name: "coherence".to_string(),
            base_score: 0.88,
        }),
    );

    let config = ComparisonConfig::new(vec![
        ModelConfig::new("fast", "fast-model"),
        ModelConfig::new("quality", "quality-model"),
        ModelConfig::new("cheap", "cheap-model"),
    ]);

    let result = engine
        .compare("Explain quantum computing", config)
        .await
        .unwrap();

    // Verify basic results
    assert_eq!(result.models.len(), 3);
    assert!(result.models.iter().all(|m| m.success));

    // Verify rankings
    assert_eq!(result.rankings.len(), 3);
    assert_eq!(result.rankings[0].rank, 1);
    assert_eq!(result.rankings[1].rank, 2);
    assert_eq!(result.rankings[2].rank, 3);

    // Quality model should win due to highest quality
    assert_eq!(result.rankings[0].model_config.provider, "quality");

    // Verify winner is set
    assert!(result.winner.is_some());
    assert_eq!(result.winner.unwrap(), "quality/quality-model");

    // Verify comparative analysis exists
    assert!(!result.comparative_analysis.summary.is_empty());
    assert!(!result.comparative_analysis.key_findings.is_empty());
    assert!(!result.comparative_analysis.recommendations.is_empty());
}

#[tokio::test]
async fn test_router_quality_strategy() {
    let mut router = ModelRouter::new(RoutingStrategy::Quality);

    // Simulate loading profiles from previous benchmarks
    let mut profile_high = llm_test_bench_core::orchestration::ModelProfile::new("provider1/model1");
    profile_high.typical_quality = 0.9;
    profile_high.avg_latency_ms = 1000;
    profile_high.cost_per_1k_tokens = 0.03;
    profile_high.context_limit = 8192;

    let mut profile_low = llm_test_bench_core::orchestration::ModelProfile::new("provider2/model2");
    profile_low.typical_quality = 0.7;
    profile_low.avg_latency_ms = 500;
    profile_low.cost_per_1k_tokens = 0.01;
    profile_low.context_limit = 4096;

    router.register_profile(profile_high);
    router.register_profile(profile_low);

    let available = vec![
        ModelConfig::new("provider1", "model1"),
        ModelConfig::new("provider2", "model2"),
    ];

    let constraints = ModelConstraints::default();

    let selection = router
        .select_model("Write a function", &available, &constraints)
        .unwrap();

    // Should select model1 for quality
    assert_eq!(selection.model_config.model, "model1");
    assert!(selection.reasoning.contains("quality"));
}

#[tokio::test]
async fn test_router_cost_strategy() {
    let mut router = ModelRouter::new(RoutingStrategy::CostOptimized);

    let mut profile_expensive = llm_test_bench_core::orchestration::ModelProfile::new("provider1/model1");
    profile_expensive.typical_quality = 0.9;
    profile_expensive.avg_latency_ms = 1000;
    profile_expensive.cost_per_1k_tokens = 0.03;
    profile_expensive.context_limit = 8192;

    let mut profile_cheap = llm_test_bench_core::orchestration::ModelProfile::new("provider2/model2");
    profile_cheap.typical_quality = 0.8;
    profile_cheap.avg_latency_ms = 500;
    profile_cheap.cost_per_1k_tokens = 0.01;
    profile_cheap.context_limit = 4096;

    router.register_profile(profile_expensive);
    router.register_profile(profile_cheap);

    let available = vec![
        ModelConfig::new("provider1", "model1"),
        ModelConfig::new("provider2", "model2"),
    ];

    let constraints = ModelConstraints::default();

    let selection = router
        .select_model("Summarize this", &available, &constraints)
        .unwrap();

    // Should select model2 for cost efficiency
    assert_eq!(selection.model_config.model, "model2");
}

#[tokio::test]
async fn test_router_latency_strategy() {
    let mut router = ModelRouter::new(RoutingStrategy::Latency);

    let mut profile_slow = llm_test_bench_core::orchestration::ModelProfile::new("provider1/model1");
    profile_slow.typical_quality = 0.9;
    profile_slow.avg_latency_ms = 2000;
    profile_slow.cost_per_1k_tokens = 0.02;
    profile_slow.context_limit = 8192;

    let mut profile_fast = llm_test_bench_core::orchestration::ModelProfile::new("provider2/model2");
    profile_fast.typical_quality = 0.8;
    profile_fast.avg_latency_ms = 300;
    profile_fast.cost_per_1k_tokens = 0.02;
    profile_fast.context_limit = 4096;

    router.register_profile(profile_slow);
    router.register_profile(profile_fast);

    let available = vec![
        ModelConfig::new("provider1", "model1"),
        ModelConfig::new("provider2", "model2"),
    ];

    let constraints = ModelConstraints::default();

    let selection = router
        .select_model("Quick question", &available, &constraints)
        .unwrap();

    // Should select model2 for speed
    assert_eq!(selection.model_config.model, "model2");
    assert!(selection.reasoning.contains("speed") || selection.reasoning.contains("latency"));
}

#[tokio::test]
async fn test_constraints_filtering() {
    let mut router = ModelRouter::new(RoutingStrategy::Quality);

    let mut profile_high_quality = llm_test_bench_core::orchestration::ModelProfile::new("provider1/model1");
    profile_high_quality.typical_quality = 0.95;
    profile_high_quality.avg_latency_ms = 1000;
    profile_high_quality.cost_per_1k_tokens = 0.03;
    profile_high_quality.context_limit = 8192;

    let mut profile_low_quality = llm_test_bench_core::orchestration::ModelProfile::new("provider2/model2");
    profile_low_quality.typical_quality = 0.65;
    profile_low_quality.avg_latency_ms = 500;
    profile_low_quality.cost_per_1k_tokens = 0.01;
    profile_low_quality.context_limit = 4096;

    router.register_profile(profile_high_quality);
    router.register_profile(profile_low_quality);

    let available = vec![
        ModelConfig::new("provider1", "model1"),
        ModelConfig::new("provider2", "model2"),
    ];

    // Set high quality threshold
    let constraints = ModelConstraints::new().with_min_quality(0.8);

    let selection = router
        .select_model("Important task", &available, &constraints)
        .unwrap();

    // Should only select model1 as model2 doesn't meet quality threshold
    assert_eq!(selection.model_config.model, "model1");
}

#[tokio::test]
async fn test_parallel_execution_performance() {
    let mut engine = ComparisonEngine::new();

    // Register 10 providers with 200ms latency each
    for i in 1..=10 {
        engine.register_provider(
            format!("provider{}", i),
            Arc::new(TestProvider {
                name: format!("provider{}", i),
                latency_ms: 200,
                quality_multiplier: 0.8 + (i as f64 * 0.01),
            }),
        );
    }

    engine.register_evaluator(
        "quality",
        Arc::new(TestEvaluator {
            name: "quality".to_string(),
            base_score: 0.8,
        }),
    );

    let models: Vec<_> = (1..=10)
        .map(|i| ModelConfig::new(format!("provider{}", i), format!("model{}", i)))
        .collect();

    let config = ComparisonConfig::new(models);

    let start = std::time::Instant::now();
    let result = engine.compare("Test prompt", config).await.unwrap();
    let duration = start.elapsed();

    assert_eq!(result.models.len(), 10);

    // With parallel execution (concurrency 10), should take ~200ms
    // Without it would take ~2000ms
    // Allow some overhead, check it's less than 500ms
    assert!(
        duration.as_millis() < 500,
        "Parallel execution took too long: {}ms",
        duration.as_millis()
    );
}

#[test]
fn test_model_config_builder() {
    let config = ModelConfig::new("openai", "gpt-4")
        .with_parameter("temperature", serde_json::json!(0.7))
        .with_parameter("max_tokens", serde_json::json!(1000));

    assert_eq!(config.provider, "openai");
    assert_eq!(config.model, "gpt-4");
    assert_eq!(config.parameters.len(), 2);
    assert_eq!(config.identifier(), "openai/gpt-4");
}

#[test]
fn test_model_constraints_builder() {
    let constraints = ModelConstraints::new()
        .with_max_cost(0.05)
        .with_max_latency_ms(1000)
        .with_min_quality(0.85)
        .with_min_context_length(8192);

    assert_eq!(constraints.max_cost, Some(0.05));
    assert_eq!(constraints.max_latency_ms, Some(1000));
    assert_eq!(constraints.min_quality, 0.85);
    assert_eq!(constraints.min_context_length, Some(8192));
}

#[test]
fn test_task_type_classification() {
    use llm_test_bench_core::orchestration::TaskType;

    assert_eq!(
        TaskType::classify_prompt("Write a function to sort an array"),
        TaskType::Coding
    );
    assert_eq!(
        TaskType::classify_prompt("Summarize this article"),
        TaskType::Summarization
    );
    assert_eq!(
        TaskType::classify_prompt("Translate to French"),
        TaskType::Translation
    );
    assert_eq!(
        TaskType::classify_prompt("What is quantum computing?"),
        TaskType::QuestionAnswering
    );
    assert_eq!(
        TaskType::classify_prompt("Write a creative story"),
        TaskType::Creative
    );
    assert_eq!(
        TaskType::classify_prompt("Solve this problem"),
        TaskType::Reasoning
    );
}

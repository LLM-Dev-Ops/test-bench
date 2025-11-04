// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! LLM-as-Judge evaluation framework
//!
//! This module provides a robust, production-ready framework for using LLMs
//! to evaluate other LLM outputs. It includes:
//! - Multiple judge model support (GPT-4, Claude 3 Opus, GPT-3.5 Turbo)
//! - Deterministic evaluation (temperature=0.0)
//! - Result caching by (prompt, response, metric) key
//! - Custom rubric support
//! - Cost tracking per evaluation
//! - Comprehensive error handling

use crate::providers::{
    CompletionRequest, CompletionResponse, Provider, ProviderError,
};
use chrono::{DateTime, Duration, Utc};
use lru::LruCache;
use serde::{Deserialize, Serialize};
use siphasher::sip::SipHasher13;
use std::hash::{Hash, Hasher};
use std::num::NonZeroUsize;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use thiserror::Error;

/// LLM-as-Judge errors
#[derive(Error, Debug)]
pub enum JudgeError {
    /// Provider error
    #[error("Provider error: {0}")]
    ProviderError(#[from] ProviderError),

    /// Invalid configuration
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    /// Evaluation failed
    #[error("Evaluation failed: {0}")]
    EvaluationFailed(String),

    /// Cache error
    #[error("Cache error: {0}")]
    CacheError(String),

    /// JSON parsing error
    #[error("JSON parsing error: {0}")]
    JsonError(#[from] serde_json::Error),

    /// Cost limit exceeded
    #[error("Cost limit exceeded: ${0:.4}")]
    CostLimitExceeded(f64),
}

/// Configuration for LLM-as-Judge
#[derive(Debug, Clone)]
pub struct JudgeConfig {
    /// Judge model to use
    pub model: String,

    /// Temperature for generation (default: 0.0 for deterministic evaluation)
    pub temperature: f32,

    /// Maximum tokens for judge response
    pub max_tokens: usize,

    /// Enable caching
    pub cache_enabled: bool,

    /// Cache directory
    pub cache_dir: Option<PathBuf>,

    /// Cache TTL in hours
    pub cache_ttl_hours: i64,

    /// Maximum cache size (number of entries)
    pub max_cache_size: usize,

    /// Maximum cost per evaluation (USD)
    pub max_cost_per_evaluation: Option<f64>,
}

impl Default for JudgeConfig {
    fn default() -> Self {
        Self {
            model: "gpt-4".to_string(),
            temperature: 0.0,
            max_tokens: 500,
            cache_enabled: true,
            cache_dir: dirs::cache_dir().map(|d| d.join("llm-test-bench").join("evaluations")),
            cache_ttl_hours: 168, // 7 days
            max_cache_size: 10_000,
            max_cost_per_evaluation: Some(0.10),
        }
    }
}

impl JudgeConfig {
    /// Create a new judge configuration
    pub fn new(model: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            ..Default::default()
        }
    }

    /// Set the temperature
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = temperature;
        self
    }

    /// Set the max tokens
    pub fn with_max_tokens(mut self, max_tokens: usize) -> Self {
        self.max_tokens = max_tokens;
        self
    }

    /// Disable caching
    pub fn without_cache(mut self) -> Self {
        self.cache_enabled = false;
        self
    }

    /// Set cache TTL in hours
    pub fn with_cache_ttl_hours(mut self, hours: i64) -> Self {
        self.cache_ttl_hours = hours;
        self
    }

    /// Set maximum cost per evaluation
    pub fn with_max_cost(mut self, max_cost: f64) -> Self {
        self.max_cost_per_evaluation = Some(max_cost);
        self
    }
}

/// A cached evaluation result
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CachedResult {
    /// The evaluation result
    result: String,

    /// When this was cached
    cached_at: DateTime<Utc>,

    /// Cost of this evaluation
    cost: f64,
}

impl CachedResult {
    /// Check if this cached result is still valid
    fn is_valid(&self, ttl_hours: i64) -> bool {
        let age = Utc::now().signed_duration_since(self.cached_at);
        age < Duration::hours(ttl_hours)
    }
}

/// Cache key for evaluation results
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct CacheKey {
    /// The prompt being evaluated
    prompt: String,

    /// The response being evaluated
    response: String,

    /// The metric name
    metric: String,

    /// The rubric or system prompt
    rubric: String,

    /// Judge model
    model: String,
}

impl CacheKey {
    /// Create a fast hash of this cache key
    fn fast_hash(&self) -> u64 {
        let mut hasher = SipHasher13::new();
        self.hash(&mut hasher);
        hasher.finish()
    }
}

/// Evaluation cache with LRU eviction and TTL
pub struct EvaluationCache {
    /// LRU cache for evaluation results
    cache: Arc<Mutex<LruCache<u64, CachedResult>>>,

    /// Cache hits counter
    hits: Arc<Mutex<usize>>,

    /// Cache misses counter
    misses: Arc<Mutex<usize>>,
}

impl EvaluationCache {
    /// Create a new evaluation cache
    pub fn new(max_size: usize) -> Self {
        Self {
            cache: Arc::new(Mutex::new(
                LruCache::new(NonZeroUsize::new(max_size).unwrap())
            )),
            hits: Arc::new(Mutex::new(0)),
            misses: Arc::new(Mutex::new(0)),
        }
    }

    /// Get a cached result
    pub fn get(&self, key: &CacheKey, ttl_hours: i64) -> Option<(String, f64)> {
        let hash = key.fast_hash();
        let mut cache = self.cache.lock().unwrap();

        if let Some(cached) = cache.get(&hash) {
            if cached.is_valid(ttl_hours) {
                *self.hits.lock().unwrap() += 1;
                return Some((cached.result.clone(), cached.cost));
            } else {
                // Expired, remove it
                cache.pop(&hash);
            }
        }

        *self.misses.lock().unwrap() += 1;
        None
    }

    /// Put a result into the cache
    pub fn put(&self, key: CacheKey, result: String, cost: f64) {
        let hash = key.fast_hash();
        let cached = CachedResult {
            result,
            cached_at: Utc::now(),
            cost,
        };

        let mut cache = self.cache.lock().unwrap();
        cache.put(hash, cached);
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        let hits = *self.hits.lock().unwrap();
        let misses = *self.misses.lock().unwrap();
        let size = self.cache.lock().unwrap().len();

        CacheStats {
            hits,
            misses,
            size,
            hit_rate: if hits + misses > 0 {
                hits as f64 / (hits + misses) as f64
            } else {
                0.0
            },
        }
    }

    /// Clear the cache
    pub fn clear(&self) {
        self.cache.lock().unwrap().clear();
        *self.hits.lock().unwrap() = 0;
        *self.misses.lock().unwrap() = 0;
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    /// Number of cache hits
    pub hits: usize,

    /// Number of cache misses
    pub misses: usize,

    /// Current cache size
    pub size: usize,

    /// Hit rate (0.0 - 1.0)
    pub hit_rate: f64,
}

/// An LLM-as-Judge evaluator
///
/// This provides a robust framework for using LLMs to evaluate other LLM outputs.
/// It includes caching, cost tracking, and deterministic evaluation.
///
/// # Examples
///
/// ```no_run
/// use llm_test_bench_core::evaluators::llm_judge::{LLMJudge, JudgeConfig};
/// use llm_test_bench_core::providers::OpenAIProvider;
/// use std::sync::Arc;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let provider = Arc::new(OpenAIProvider::new("key".to_string()));
/// let config = JudgeConfig::new("gpt-4");
/// let judge = LLMJudge::new(provider, config);
///
/// let result = judge.evaluate(
///     "What is 2+2?",
///     "4",
///     "correctness",
///     "Evaluate if the answer is mathematically correct.",
/// ).await?;
///
/// println!("Score: {}", result.score);
/// println!("Cost: ${:.4}", result.cost);
/// # Ok(())
/// # }
/// ```
pub struct LLMJudge {
    /// Provider to use for judge calls
    provider: Arc<dyn Provider>,

    /// Judge configuration
    config: JudgeConfig,

    /// Evaluation cache
    cache: Option<EvaluationCache>,

    /// Total cost across all evaluations
    total_cost: Arc<Mutex<f64>>,
}

impl LLMJudge {
    /// Create a new LLM-as-Judge evaluator
    pub fn new(provider: Arc<dyn Provider>, config: JudgeConfig) -> Self {
        let cache = if config.cache_enabled {
            Some(EvaluationCache::new(config.max_cache_size))
        } else {
            None
        };

        Self {
            provider,
            config,
            cache,
            total_cost: Arc::new(Mutex::new(0.0)),
        }
    }

    /// Evaluate a prompt/response pair with the given rubric
    ///
    /// # Arguments
    ///
    /// * `prompt` - The original prompt
    /// * `response` - The response to evaluate
    /// * `metric` - Name of the metric being evaluated
    /// * `rubric` - The evaluation rubric or system prompt
    ///
    /// # Returns
    ///
    /// An `EvaluationResult` containing the score, reasoning, and cost
    pub async fn evaluate(
        &self,
        prompt: &str,
        response: &str,
        metric: &str,
        rubric: &str,
    ) -> Result<EvaluationResult, JudgeError> {
        // Check cache first
        if let Some(ref cache) = self.cache {
            let key = CacheKey {
                prompt: prompt.to_string(),
                response: response.to_string(),
                metric: metric.to_string(),
                rubric: rubric.to_string(),
                model: self.config.model.clone(),
            };

            if let Some((cached_result, cost)) = cache.get(&key, self.config.cache_ttl_hours) {
                tracing::debug!("Cache hit for {} evaluation", metric);
                return self.parse_evaluation_result(&cached_result, cost);
            }
        }

        // Build the evaluation prompt
        let eval_prompt = self.build_evaluation_prompt(prompt, response, rubric);

        // Call the judge model
        let request = CompletionRequest::new(&self.config.model, eval_prompt)
            .with_temperature(self.config.temperature)
            .with_max_tokens(self.config.max_tokens);

        let judge_response = self.provider.complete(request).await?;

        // Calculate cost
        let cost = self.calculate_cost(&judge_response);

        // Check cost limit
        if let Some(max_cost) = self.config.max_cost_per_evaluation {
            if cost > max_cost {
                return Err(JudgeError::CostLimitExceeded(cost));
            }
        }

        // Update total cost
        {
            let mut total = self.total_cost.lock().unwrap();
            *total += cost;
        }

        // Cache the result
        if let Some(ref cache) = self.cache {
            let key = CacheKey {
                prompt: prompt.to_string(),
                response: response.to_string(),
                metric: metric.to_string(),
                rubric: rubric.to_string(),
                model: self.config.model.clone(),
            };
            cache.put(key, judge_response.content.clone(), cost);
        }

        // Parse and return the result
        self.parse_evaluation_result(&judge_response.content, cost)
    }

    /// Build the evaluation prompt
    fn build_evaluation_prompt(&self, prompt: &str, response: &str, rubric: &str) -> String {
        format!(
            r#"You are an expert evaluator for LLM responses. Your task is to evaluate the quality of a response according to the given rubric.

RUBRIC:
{rubric}

ORIGINAL PROMPT:
{prompt}

RESPONSE TO EVALUATE:
{response}

Please evaluate the response and provide your assessment in the following JSON format:
{{
    "score": <float between 0.0 and 1.0>,
    "reasoning": "<detailed explanation of your evaluation>",
    "confidence": <float between 0.0 and 1.0>
}}

Respond with ONLY the JSON object, no additional text."#
        )
    }

    /// Parse the evaluation result from JSON
    fn parse_evaluation_result(
        &self,
        content: &str,
        cost: f64,
    ) -> Result<EvaluationResult, JudgeError> {
        // Try to extract JSON from the response
        let json_str = self.extract_json(content)?;

        // Parse the JSON
        let parsed: serde_json::Value = serde_json::from_str(&json_str)?;

        let score = parsed["score"]
            .as_f64()
            .ok_or_else(|| JudgeError::EvaluationFailed("Missing or invalid score".to_string()))?;

        let reasoning = parsed["reasoning"]
            .as_str()
            .ok_or_else(|| {
                JudgeError::EvaluationFailed("Missing or invalid reasoning".to_string())
            })?
            .to_string();

        let confidence = parsed["confidence"].as_f64().unwrap_or(1.0);

        // Validate score range
        if !(0.0..=1.0).contains(&score) {
            return Err(JudgeError::EvaluationFailed(format!(
                "Score {} is outside valid range [0.0, 1.0]",
                score
            )));
        }

        Ok(EvaluationResult {
            score,
            reasoning,
            confidence,
            cost,
            model: self.config.model.clone(),
        })
    }

    /// Extract JSON from the response (handles cases where LLM adds extra text)
    fn extract_json(&self, content: &str) -> Result<String, JudgeError> {
        let trimmed = content.trim();

        // Try to find JSON object boundaries
        if let Some(start) = trimmed.find('{') {
            if let Some(end) = trimmed.rfind('}') {
                if start < end {
                    return Ok(trimmed[start..=end].to_string());
                }
            }
        }

        // If no braces found, try the whole content
        Ok(trimmed.to_string())
    }

    /// Calculate the cost of an evaluation
    fn calculate_cost(&self, response: &CompletionResponse) -> f64 {
        // Pricing as of 2024 (these should ideally be configurable)
        let (prompt_cost_per_1k, completion_cost_per_1k) = match self.config.model.as_str() {
            "gpt-4" | "gpt-4-0613" => (0.03, 0.06),
            "gpt-4-turbo" | "gpt-4-turbo-preview" | "gpt-4-1106-preview" => (0.01, 0.03),
            "gpt-3.5-turbo" | "gpt-3.5-turbo-0125" => (0.0005, 0.0015),
            "claude-3-opus-20240229" => (0.015, 0.075),
            "claude-3-sonnet-20240229" => (0.003, 0.015),
            "claude-3-haiku-20240307" => (0.00025, 0.00125),
            _ => (0.01, 0.03), // Default to GPT-4 Turbo pricing
        };

        response.usage.calculate_cost(prompt_cost_per_1k, completion_cost_per_1k)
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> Option<CacheStats> {
        self.cache.as_ref().map(|c| c.stats())
    }

    /// Get total cost of all evaluations
    pub fn total_cost(&self) -> f64 {
        *self.total_cost.lock().unwrap()
    }

    /// Clear the cache
    pub fn clear_cache(&self) {
        if let Some(ref cache) = self.cache {
            cache.clear();
        }
    }

    /// Get the judge configuration
    pub fn config(&self) -> &JudgeConfig {
        &self.config
    }
}

/// The result of an LLM-as-Judge evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationResult {
    /// The evaluation score (0.0 - 1.0)
    pub score: f64,

    /// Detailed reasoning for the score
    pub reasoning: String,

    /// Confidence in the evaluation (0.0 - 1.0)
    pub confidence: f64,

    /// Cost of this evaluation in USD
    pub cost: f64,

    /// Model used for evaluation
    pub model: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::{CompletionResponse, FinishReason, TokenUsage};
    use mockall::mock;
    use mockall::predicate::*;

    mock! {
        Provider {}

        #[async_trait::async_trait]
        impl Provider for Provider {
            async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse, ProviderError>;
            async fn stream(&self, request: CompletionRequest) -> Result<crate::providers::ResponseStream, ProviderError>;
            fn supported_models(&self) -> Vec<crate::providers::ModelInfo>;
            fn max_context_length(&self, model: &str) -> Option<usize>;
            fn name(&self) -> &str;
            async fn validate_config(&self) -> Result<(), ProviderError>;
            fn estimate_tokens(&self, text: &str, model: &str) -> Result<usize, ProviderError>;
        }
    }

    fn create_mock_response(content: String, prompt_tokens: usize, completion_tokens: usize) -> CompletionResponse {
        CompletionResponse {
            id: "test-123".to_string(),
            model: "gpt-4".to_string(),
            content,
            usage: TokenUsage::new(prompt_tokens, completion_tokens),
            finish_reason: FinishReason::Stop,
            created_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_judge_evaluation_success() {
        let mut mock = MockProvider::new();

        let response_json = r#"{"score": 0.85, "reasoning": "Good response", "confidence": 0.9}"#;
        mock.expect_complete()
            .times(1)
            .returning(move |_| Ok(create_mock_response(response_json.to_string(), 100, 50)));

        let config = JudgeConfig::new("gpt-4").without_cache();
        let judge = LLMJudge::new(Arc::new(mock), config);

        let result = judge
            .evaluate(
                "What is 2+2?",
                "4",
                "correctness",
                "Evaluate mathematical correctness.",
            )
            .await
            .unwrap();

        assert_eq!(result.score, 0.85);
        assert_eq!(result.reasoning, "Good response");
        assert_eq!(result.confidence, 0.9);
        assert!(result.cost > 0.0);
    }

    #[tokio::test]
    async fn test_judge_caching() {
        let mut mock = MockProvider::new();

        let response_json = r#"{"score": 0.75, "reasoning": "Test", "confidence": 0.8}"#;
        mock.expect_complete()
            .times(1) // Should only be called once due to caching
            .returning(move |_| Ok(create_mock_response(response_json.to_string(), 100, 50)));

        let config = JudgeConfig::new("gpt-4");
        let judge = LLMJudge::new(Arc::new(mock), config);

        // First call - cache miss
        let result1 = judge
            .evaluate("prompt", "response", "test", "rubric")
            .await
            .unwrap();

        // Second call - cache hit
        let result2 = judge
            .evaluate("prompt", "response", "test", "rubric")
            .await
            .unwrap();

        assert_eq!(result1.score, result2.score);
        assert_eq!(result1.reasoning, result2.reasoning);

        let stats = judge.cache_stats().unwrap();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.hit_rate, 0.5);
    }

    #[tokio::test]
    async fn test_cache_key_differentiation() {
        let mut mock = MockProvider::new();

        mock.expect_complete()
            .times(2) // Two different cache keys
            .returning(move |_| {
                Ok(create_mock_response(
                    r#"{"score": 0.5, "reasoning": "Test", "confidence": 0.5}"#.to_string(),
                    100,
                    50,
                ))
            });

        let config = JudgeConfig::new("gpt-4");
        let judge = LLMJudge::new(Arc::new(mock), config);

        // Different prompts should result in different cache keys
        judge.evaluate("prompt1", "response", "test", "rubric").await.unwrap();
        judge.evaluate("prompt2", "response", "test", "rubric").await.unwrap();

        let stats = judge.cache_stats().unwrap();
        assert_eq!(stats.misses, 2);
        assert_eq!(stats.hits, 0);
    }

    #[tokio::test]
    async fn test_cost_tracking() {
        let mut mock = MockProvider::new();

        mock.expect_complete()
            .times(3)
            .returning(move |_| {
                Ok(create_mock_response(
                    r#"{"score": 0.5, "reasoning": "Test", "confidence": 0.5}"#.to_string(),
                    100,
                    50,
                ))
            });

        let config = JudgeConfig::new("gpt-4").without_cache();
        let judge = LLMJudge::new(Arc::new(mock), config);

        judge.evaluate("p1", "r1", "test", "rubric").await.unwrap();
        judge.evaluate("p2", "r2", "test", "rubric").await.unwrap();
        judge.evaluate("p3", "r3", "test", "rubric").await.unwrap();

        let total_cost = judge.total_cost();
        assert!(total_cost > 0.0);

        // Cost should be approximately 3 * (100 * 0.03 + 50 * 0.06) / 1000
        let expected = 3.0 * (100.0 * 0.03 + 50.0 * 0.06) / 1000.0;
        assert!((total_cost - expected).abs() < 0.001);
    }

    #[tokio::test]
    async fn test_cost_limit_exceeded() {
        let mut mock = MockProvider::new();

        mock.expect_complete()
            .returning(move |_| {
                // Return a response with very high token count
                Ok(create_mock_response(
                    r#"{"score": 0.5, "reasoning": "Test", "confidence": 0.5}"#.to_string(),
                    10000,
                    10000,
                ))
            });

        let config = JudgeConfig::new("gpt-4")
            .without_cache()
            .with_max_cost(0.01); // Very low limit

        let judge = LLMJudge::new(Arc::new(mock), config);

        let result = judge.evaluate("prompt", "response", "test", "rubric").await;

        assert!(result.is_err());
        match result {
            Err(JudgeError::CostLimitExceeded(cost)) => {
                assert!(cost > 0.01);
            }
            _ => panic!("Expected CostLimitExceeded error"),
        }
    }

    #[tokio::test]
    async fn test_json_extraction_with_extra_text() {
        let mut mock = MockProvider::new();

        let response_with_extra = r#"Here's my evaluation:

        {"score": 0.65, "reasoning": "Decent response", "confidence": 0.7}

        Hope this helps!"#;

        mock.expect_complete()
            .returning(move |_| Ok(create_mock_response(response_with_extra.to_string(), 100, 50)));

        let config = JudgeConfig::new("gpt-4").without_cache();
        let judge = LLMJudge::new(Arc::new(mock), config);

        let result = judge
            .evaluate("prompt", "response", "test", "rubric")
            .await
            .unwrap();

        assert_eq!(result.score, 0.65);
        assert_eq!(result.reasoning, "Decent response");
    }

    #[tokio::test]
    async fn test_invalid_score_range() {
        let mut mock = MockProvider::new();

        let invalid_response = r#"{"score": 1.5, "reasoning": "Too high", "confidence": 0.9}"#;
        mock.expect_complete()
            .returning(move |_| Ok(create_mock_response(invalid_response.to_string(), 100, 50)));

        let config = JudgeConfig::new("gpt-4").without_cache();
        let judge = LLMJudge::new(Arc::new(mock), config);

        let result = judge.evaluate("prompt", "response", "test", "rubric").await;

        assert!(result.is_err());
        match result {
            Err(JudgeError::EvaluationFailed(msg)) => {
                assert!(msg.contains("outside valid range"));
            }
            _ => panic!("Expected EvaluationFailed error"),
        }
    }

    #[tokio::test]
    async fn test_cache_ttl_expiration() {
        let cache = EvaluationCache::new(100);

        let key = CacheKey {
            prompt: "test".to_string(),
            response: "test".to_string(),
            metric: "test".to_string(),
            rubric: "test".to_string(),
            model: "gpt-4".to_string(),
        };

        cache.put(key.clone(), "result".to_string(), 0.01);

        // Should be valid with long TTL
        assert!(cache.get(&key, 168).is_some());

        // Should be invalid with 0 TTL (expired)
        assert!(cache.get(&key, 0).is_none());
    }

    #[test]
    fn test_cache_clear() {
        let cache = EvaluationCache::new(100);

        let key = CacheKey {
            prompt: "test".to_string(),
            response: "test".to_string(),
            metric: "test".to_string(),
            rubric: "test".to_string(),
            model: "gpt-4".to_string(),
        };

        cache.put(key.clone(), "result".to_string(), 0.01);
        assert_eq!(cache.stats().size, 1);

        cache.clear();
        assert_eq!(cache.stats().size, 0);
        assert_eq!(cache.stats().hits, 0);
        assert_eq!(cache.stats().misses, 0);
    }

    #[test]
    fn test_judge_config_builder() {
        let config = JudgeConfig::new("gpt-3.5-turbo")
            .with_temperature(0.5)
            .with_max_tokens(1000)
            .without_cache()
            .with_cache_ttl_hours(24)
            .with_max_cost(0.05);

        assert_eq!(config.model, "gpt-3.5-turbo");
        assert_eq!(config.temperature, 0.5);
        assert_eq!(config.max_tokens, 1000);
        assert!(!config.cache_enabled);
        assert_eq!(config.cache_ttl_hours, 24);
        assert_eq!(config.max_cost_per_evaluation, Some(0.05));
    }

    #[test]
    fn test_cache_key_hash_consistency() {
        let key1 = CacheKey {
            prompt: "test".to_string(),
            response: "response".to_string(),
            metric: "metric".to_string(),
            rubric: "rubric".to_string(),
            model: "model".to_string(),
        };

        let key2 = key1.clone();

        assert_eq!(key1.fast_hash(), key2.fast_hash());
    }

    #[test]
    fn test_cache_key_hash_differentiation() {
        let key1 = CacheKey {
            prompt: "test1".to_string(),
            response: "response".to_string(),
            metric: "metric".to_string(),
            rubric: "rubric".to_string(),
            model: "model".to_string(),
        };

        let key2 = CacheKey {
            prompt: "test2".to_string(),
            response: "response".to_string(),
            metric: "metric".to_string(),
            rubric: "rubric".to_string(),
            model: "model".to_string(),
        };

        assert_ne!(key1.fast_hash(), key2.fast_hash());
    }
}

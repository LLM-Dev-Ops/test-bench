//! LLM Dev Ops Infra Integration for Providers
//!
//! This module provides enhanced provider functionality by integrating with
//! the LLM-Dev-Ops infra layer. When the `infra-llm` feature is enabled,
//! providers gain access to:
//!
//! - **Retry Logic**: Advanced retry policies from `infra-retry`
//! - **Caching**: Response caching from `infra-cache`
//! - **Rate Limiting**: Provider-specific rate limiters from `infra-rate-limit`
//!
//! # Example
//!
//! ```rust,ignore
//! use llm_test_bench_core::providers::{OpenAIProvider, InfraEnhancedProvider};
//! use llm_test_bench_core::providers::CompletionRequest;
//!
//! let provider = OpenAIProvider::new("api-key".to_string());
//! let enhanced = InfraEnhancedProvider::wrap(provider);
//!
//! // Requests now have automatic retry, caching, and rate limiting
//! let response = enhanced.complete(request).await?;
//! ```

use async_trait::async_trait;
use std::sync::Arc;

use super::error::ProviderError;
use super::traits::Provider;
use super::types::{CompletionRequest, CompletionResponse, ModelInfo, ResponseStream};

use crate::infra::cache::{response_cache, cache_key, Cache};
use crate::infra::rate_limit::{provider_limiter, ProviderType, ProviderLimiter, ProviderLimitResult};
use crate::infra::retry::{retry_with_policy, default_llm_retry_policy, RetryPolicy};

/// A wrapper that enhances any Provider with infra capabilities.
///
/// This wrapper adds:
/// - Automatic retry with exponential backoff
/// - Response caching with configurable TTL
/// - Provider-specific rate limiting
pub struct InfraEnhancedProvider<P: Provider> {
    inner: P,
    cache: Arc<Cache>,
    rate_limiter: Arc<ProviderLimiter>,
    retry_policy: RetryPolicy,
    enable_cache: bool,
    enable_rate_limit: bool,
}

impl<P: Provider> InfraEnhancedProvider<P> {
    /// Wrap an existing provider with infra enhancements.
    pub fn wrap(provider: P) -> Self {
        let provider_type = Self::detect_provider_type(&provider);

        Self {
            inner: provider,
            cache: Arc::new(response_cache()),
            rate_limiter: Arc::new(provider_limiter(provider_type)),
            retry_policy: default_llm_retry_policy(),
            enable_cache: true,
            enable_rate_limit: true,
        }
    }

    /// Create with custom configuration.
    pub fn with_config(
        provider: P,
        cache: Cache,
        rate_limiter: ProviderLimiter,
        retry_policy: RetryPolicy,
    ) -> Self {
        Self {
            inner: provider,
            cache: Arc::new(cache),
            rate_limiter: Arc::new(rate_limiter),
            retry_policy,
            enable_cache: true,
            enable_rate_limit: true,
        }
    }

    /// Disable response caching.
    pub fn without_cache(mut self) -> Self {
        self.enable_cache = false;
        self
    }

    /// Disable rate limiting.
    pub fn without_rate_limit(mut self) -> Self {
        self.enable_rate_limit = false;
        self
    }

    /// Set a custom retry policy.
    pub fn with_retry_policy(mut self, policy: RetryPolicy) -> Self {
        self.retry_policy = policy;
        self
    }

    /// Get a reference to the underlying provider.
    pub fn inner(&self) -> &P {
        &self.inner
    }

    /// Get cache statistics.
    pub fn cache_stats(&self) -> infra_cache::CacheStats {
        self.cache.stats()
    }

    /// Check remaining rate limit capacity.
    pub fn remaining_requests(&self) -> usize {
        self.rate_limiter.remaining_requests()
    }

    /// Detect the provider type from the provider name.
    fn detect_provider_type(provider: &P) -> ProviderType {
        match provider.name().to_lowercase().as_str() {
            "openai" => ProviderType::OpenAI,
            "anthropic" => ProviderType::Anthropic,
            "google" => ProviderType::Google,
            "azure" | "azure_openai" => ProviderType::Azure,
            "cohere" => ProviderType::Cohere,
            "mistral" => ProviderType::Mistral,
            "together" => ProviderType::Together,
            "replicate" => ProviderType::Replicate,
            "ollama" => ProviderType::Ollama,
            _ => ProviderType::Custom { rpm: 60, tpm: 100_000 },
        }
    }

    /// Generate a cache key for a request.
    fn generate_cache_key(&self, request: &CompletionRequest) -> String {
        cache_key(
            self.inner.name(),
            &request.model,
            &request.prompt,
            request.temperature,
            request.max_tokens.map(|t| t as u32),
        )
    }

    /// Estimate tokens for rate limiting.
    fn estimate_request_tokens(&self, request: &CompletionRequest) -> u32 {
        crate::infra::rate_limit::estimate_tokens(&request.prompt)
            + request.max_tokens.unwrap_or(100) as u32
    }
}

#[async_trait]
impl<P: Provider + 'static> Provider for InfraEnhancedProvider<P> {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse, ProviderError> {
        // Check cache first
        if self.enable_cache {
            let key = self.generate_cache_key(&request);
            if let Some(cached) = self.cache.get::<CompletionResponse>(&key) {
                return Ok(cached);
            }
        }

        // Check rate limit
        if self.enable_rate_limit {
            let estimated_tokens = self.estimate_request_tokens(&request);
            match self.rate_limiter.try_acquire(estimated_tokens) {
                ProviderLimitResult::Allowed => {}
                ProviderLimitResult::RequestLimitExceeded { retry_after } => {
                    return Err(ProviderError::RateLimitExceeded {
                        retry_after: Some(retry_after),
                    });
                }
                ProviderLimitResult::TokenLimitExceeded { retry_after } => {
                    return Err(ProviderError::RateLimitExceeded {
                        retry_after: Some(retry_after),
                    });
                }
                ProviderLimitResult::DailyLimitExceeded { retry_after } => {
                    return Err(ProviderError::RateLimitExceeded {
                        retry_after: Some(retry_after),
                    });
                }
            }
        }

        // Execute with retry
        let inner = &self.inner;
        let req = request.clone();
        let response = retry_with_policy(self.retry_policy.clone(), || async {
            inner.complete(req.clone()).await
        }).await?;

        // Cache successful response
        if self.enable_cache {
            let key = self.generate_cache_key(&request);
            self.cache.insert(key, response.clone(), None);
        }

        // Record actual token usage for rate limiting
        if self.enable_rate_limit {
            self.rate_limiter.record_usage(
                response.usage.total_tokens as u32,
                self.estimate_request_tokens(&request),
            );
        }

        Ok(response)
    }

    async fn stream(&self, request: CompletionRequest) -> Result<ResponseStream, ProviderError> {
        // Streaming bypasses cache but respects rate limits
        if self.enable_rate_limit {
            let estimated_tokens = self.estimate_request_tokens(&request);
            match self.rate_limiter.try_acquire(estimated_tokens) {
                ProviderLimitResult::Allowed => {}
                result => {
                    if let Some(retry_after) = result.retry_after() {
                        return Err(ProviderError::RateLimitExceeded {
                            retry_after: Some(retry_after),
                        });
                    }
                }
            }
        }

        self.inner.stream(request).await
    }

    fn supported_models(&self) -> Vec<ModelInfo> {
        self.inner.supported_models()
    }

    fn max_context_length(&self, model: &str) -> Option<usize> {
        self.inner.max_context_length(model)
    }

    fn name(&self) -> &str {
        self.inner.name()
    }

    async fn validate_config(&self) -> Result<(), ProviderError> {
        self.inner.validate_config().await
    }

    fn estimate_tokens(&self, text: &str, model: &str) -> Result<usize, ProviderError> {
        self.inner.estimate_tokens(text, model)
    }
}

/// Extension trait for easily wrapping providers with infra capabilities.
pub trait InfraProviderExt: Provider + Sized {
    /// Wrap this provider with infra enhancements (retry, cache, rate limiting).
    fn with_infra(self) -> InfraEnhancedProvider<Self> {
        InfraEnhancedProvider::wrap(self)
    }
}

// Blanket implementation for all providers
impl<P: Provider> InfraProviderExt for P {}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock provider for testing
    struct MockProvider;

    #[async_trait]
    impl Provider for MockProvider {
        async fn complete(&self, _request: CompletionRequest) -> Result<CompletionResponse, ProviderError> {
            Ok(CompletionResponse {
                content: "Mock response".to_string(),
                model: "mock-model".to_string(),
                usage: super::super::types::TokenUsage {
                    prompt_tokens: 10,
                    completion_tokens: 5,
                    total_tokens: 15,
                },
                finish_reason: super::super::types::FinishReason::Stop,
                latency_ms: 100,
            })
        }

        async fn stream(&self, _request: CompletionRequest) -> Result<ResponseStream, ProviderError> {
            Err(ProviderError::NotSupported {
                feature: "streaming".to_string(),
            })
        }

        fn supported_models(&self) -> Vec<ModelInfo> {
            vec![]
        }

        fn max_context_length(&self, _model: &str) -> Option<usize> {
            Some(4096)
        }

        fn name(&self) -> &str {
            "mock"
        }

        async fn validate_config(&self) -> Result<(), ProviderError> {
            Ok(())
        }

        fn estimate_tokens(&self, text: &str, _model: &str) -> Result<usize, ProviderError> {
            Ok(text.len() / 4)
        }
    }

    #[test]
    fn test_detect_provider_type() {
        let provider = MockProvider;
        let enhanced = InfraEnhancedProvider::wrap(provider);
        // Mock provider returns "mock" which maps to Custom
        assert!(enhanced.inner().name() == "mock");
    }

    #[test]
    fn test_cache_key_generation() {
        let provider = MockProvider;
        let enhanced = InfraEnhancedProvider::wrap(provider);

        let request = CompletionRequest {
            model: "gpt-4".to_string(),
            prompt: "Hello".to_string(),
            max_tokens: Some(100),
            temperature: Some(0.7),
            top_p: None,
            stop_sequences: vec![],
            stream: false,
        };

        let key = enhanced.generate_cache_key(&request);
        assert!(!key.is_empty());
    }
}

//! Cache integration bridge for infra-cache.
//!
//! This module provides caching utilities from the LLM-Dev-Ops infra layer,
//! optimized for caching LLM API responses to reduce costs and latency.
//!
//! # Example
//!
//! ```rust,ignore
//! use llm_test_bench_core::infra::cache::{response_cache, CacheKey};
//!
//! // Create a response cache
//! let cache = response_cache();
//!
//! // Check cache before making API call
//! let key = CacheKey::for_request(&provider, &model, &prompt);
//! if let Some(cached) = cache.get::<LlmResponse>(&key) {
//!     return Ok(cached);
//! }
//!
//! // Make API call and cache result
//! let response = provider.complete(request).await?;
//! cache.insert(key, response.clone(), Some(Duration::from_secs(3600)));
//! ```

pub use infra_cache::{
    Cache, CacheConfig, CacheEntry, CacheStats,
    llm_response_cache, embedding_cache, short_lived_cache,
    LlmCacheKey, CachedLlmResponse,
};

use std::time::Duration;

/// Create a cache optimized for LLM response caching.
///
/// Default configuration:
/// - Max entries: 1000
/// - Default TTL: 1 hour
/// - LRU eviction policy
pub fn response_cache() -> Cache {
    infra_cache::llm_response_cache()
}

/// Create a cache for embedding vectors.
///
/// Default configuration:
/// - Max entries: 10,000
/// - Default TTL: 24 hours
/// - LRU eviction policy
pub fn vector_cache() -> Cache {
    infra_cache::embedding_cache()
}

/// Create a short-lived cache for rate limiting checks.
///
/// Default configuration:
/// - Max entries: 100
/// - Default TTL: 1 minute
pub fn rate_limit_cache() -> Cache {
    infra_cache::short_lived_cache()
}

/// Create a custom cache with specified configuration.
pub fn custom_cache(max_entries: usize, default_ttl: Duration) -> Cache {
    Cache::new(
        CacheConfig::default()
            .max_entries(max_entries)
            .default_ttl(default_ttl)
    )
}

/// Generate a cache key for an LLM request.
///
/// The key is based on:
/// - Provider name
/// - Model name
/// - Full message content
/// - Temperature (if specified)
/// - Max tokens (if specified)
pub fn cache_key(
    provider: &str,
    model: &str,
    prompt: &str,
    temperature: Option<f32>,
    max_tokens: Option<u32>,
) -> String {
    LlmCacheKey::new(provider, model, prompt)
        .temperature(temperature)
        .max_tokens(max_tokens)
        .to_key()
}

/// Wrapper for cached LLM responses with metadata.
pub type CachedResponse = CachedLlmResponse;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_response_cache_creation() {
        let cache = response_cache();
        assert!(cache.stats().hits == 0);
    }

    #[test]
    fn test_cache_key_generation() {
        let key1 = cache_key("openai", "gpt-4", "Hello", Some(0.7), Some(100));
        let key2 = cache_key("openai", "gpt-4", "Hello", Some(0.7), Some(100));
        let key3 = cache_key("openai", "gpt-4", "Hello", Some(0.8), Some(100));

        assert_eq!(key1, key2); // Same parameters = same key
        assert_ne!(key1, key3); // Different temperature = different key
    }

    #[test]
    fn test_cache_insert_and_get() {
        let cache = response_cache();
        let key = "test-key".to_string();

        cache.insert(key.clone(), "test-value".to_string(), None);
        let retrieved: Option<String> = cache.get(&key);

        assert_eq!(retrieved, Some("test-value".to_string()));
    }
}

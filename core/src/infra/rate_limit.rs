//! Rate limiting integration bridge for infra-rate-limit.
//!
//! This module provides rate limiting utilities from the LLM-Dev-Ops infra layer,
//! with pre-configured limiters for major LLM providers.
//!
//! # Example
//!
//! ```rust,ignore
//! use llm_test_bench_core::infra::rate_limit::{provider_limiter, ProviderType};
//!
//! // Create a rate limiter for OpenAI
//! let limiter = provider_limiter(ProviderType::OpenAI);
//!
//! // Check rate limit before making request
//! match limiter.try_acquire(estimated_tokens) {
//!     ProviderLimitResult::Allowed => {
//!         // Make the request
//!     }
//!     ProviderLimitResult::RequestLimitExceeded { retry_after } => {
//!         tokio::time::sleep(retry_after).await;
//!     }
//!     // ... handle other cases
//! }
//! ```

pub use infra_rate_limit::{
    RateLimiter, RateLimitConfig, RateLimitError,
    TokenBucket, SlidingWindowLimiter,
    ProviderLimiter, ProviderRateLimits, ProviderLimitResult,
    openai_limiter, anthropic_limiter, google_limiter, custom_limiter,
};

/// Provider types for pre-configured rate limiters
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderType {
    OpenAI,
    Anthropic,
    Google,
    Azure,
    Cohere,
    Mistral,
    Together,
    Replicate,
    Ollama,
    Custom { rpm: u32, tpm: u32 },
}

/// Create a rate limiter for a specific provider.
pub fn provider_limiter(provider: ProviderType) -> ProviderLimiter {
    match provider {
        ProviderType::OpenAI => openai_limiter(),
        ProviderType::Anthropic => anthropic_limiter(),
        ProviderType::Google => google_limiter(),
        ProviderType::Azure => ProviderLimiter::new(ProviderRateLimits {
            requests_per_minute: 60,
            tokens_per_minute: 80_000,
            requests_per_day: None,
        }),
        ProviderType::Cohere => ProviderLimiter::new(ProviderRateLimits {
            requests_per_minute: 100,
            tokens_per_minute: 100_000,
            requests_per_day: None,
        }),
        ProviderType::Mistral => ProviderLimiter::new(ProviderRateLimits {
            requests_per_minute: 60,
            tokens_per_minute: 100_000,
            requests_per_day: None,
        }),
        ProviderType::Together => ProviderLimiter::new(ProviderRateLimits {
            requests_per_minute: 60,
            tokens_per_minute: 100_000,
            requests_per_day: None,
        }),
        ProviderType::Replicate => ProviderLimiter::new(ProviderRateLimits {
            requests_per_minute: 30,
            tokens_per_minute: 50_000,
            requests_per_day: None,
        }),
        ProviderType::Ollama => ProviderLimiter::new(ProviderRateLimits {
            // Local Ollama has no real limits, but we add sensible defaults
            requests_per_minute: 1000,
            tokens_per_minute: 1_000_000,
            requests_per_day: None,
        }),
        ProviderType::Custom { rpm, tpm } => ProviderLimiter::new(ProviderRateLimits {
            requests_per_minute: rpm,
            tokens_per_minute: tpm,
            requests_per_day: None,
        }),
    }
}

/// Create a simple requests-per-minute limiter.
pub fn rpm_limiter(rpm: u32) -> RateLimiter {
    RateLimiter::requests_per_minute(rpm)
}

/// Create a simple requests-per-second limiter.
pub fn rps_limiter(rps: u32) -> RateLimiter {
    RateLimiter::requests_per_second(rps)
}

/// Estimate token count for a request (rough approximation).
/// Assumes ~4 characters per token, which is typical for English text.
pub fn estimate_tokens(text: &str) -> u32 {
    (text.len() / 4).max(1) as u32
}

/// Check if rate limiting should be applied based on provider response headers.
pub struct RateLimitHeaders {
    /// Remaining requests in current window
    pub remaining_requests: Option<u32>,
    /// Remaining tokens in current window
    pub remaining_tokens: Option<u32>,
    /// Time until rate limit resets
    pub reset_after: Option<std::time::Duration>,
}

impl RateLimitHeaders {
    /// Parse rate limit headers from an HTTP response.
    /// Different providers use different header names.
    pub fn from_headers(headers: &[(String, String)], provider: ProviderType) -> Self {
        let get = |name: &str| -> Option<u32> {
            headers.iter()
                .find(|(k, _)| k.eq_ignore_ascii_case(name))
                .and_then(|(_, v)| v.parse().ok())
        };

        match provider {
            ProviderType::OpenAI | ProviderType::Azure => Self {
                remaining_requests: get("x-ratelimit-remaining-requests"),
                remaining_tokens: get("x-ratelimit-remaining-tokens"),
                reset_after: get("x-ratelimit-reset-requests")
                    .map(|s| std::time::Duration::from_secs(s as u64)),
            },
            ProviderType::Anthropic => Self {
                remaining_requests: get("x-ratelimit-remaining-requests"),
                remaining_tokens: get("x-ratelimit-remaining-tokens"),
                reset_after: get("x-ratelimit-reset")
                    .map(|s| std::time::Duration::from_secs(s as u64)),
            },
            _ => Self {
                remaining_requests: get("x-ratelimit-remaining").or_else(|| get("ratelimit-remaining")),
                remaining_tokens: None,
                reset_after: None,
            },
        }
    }

    /// Check if we're close to hitting the rate limit.
    pub fn is_near_limit(&self) -> bool {
        self.remaining_requests.map_or(false, |r| r < 5)
            || self.remaining_tokens.map_or(false, |t| t < 1000)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_limiter_creation() {
        let limiter = provider_limiter(ProviderType::OpenAI);
        assert_eq!(limiter.limits().requests_per_minute, 60);
    }

    #[test]
    fn test_custom_limiter() {
        let limiter = provider_limiter(ProviderType::Custom { rpm: 100, tpm: 50_000 });
        assert_eq!(limiter.limits().requests_per_minute, 100);
        assert_eq!(limiter.limits().tokens_per_minute, 50_000);
    }

    #[test]
    fn test_estimate_tokens() {
        let text = "Hello, world!"; // 13 characters
        let tokens = estimate_tokens(text);
        assert!(tokens >= 3 && tokens <= 4);
    }

    #[test]
    fn test_rpm_limiter() {
        let limiter = rpm_limiter(60);
        assert!(limiter.remaining() > 0);
    }
}

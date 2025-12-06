//! Retry integration bridge for infra-retry.
//!
//! This module provides retry policy utilities from the LLM-Dev-Ops infra layer,
//! pre-configured for LLM API interactions with appropriate backoff strategies.
//!
//! # Example
//!
//! ```rust,ignore
//! use llm_test_bench_core::infra::retry::{retry_llm_call, RetryPolicy};
//!
//! // Use default LLM retry policy
//! let result = retry_llm_call(|| async {
//!     provider.complete(request).await
//! }).await?;
//!
//! // Or customize the policy
//! let policy = RetryPolicy::exponential()
//!     .max_attempts(5)
//!     .retry_on_rate_limit(true);
//! let result = retry_with_policy(policy, || async { ... }).await?;
//! ```

pub use infra_retry::{
    RetryPolicy, BackoffStrategy, retry, retry_with_context,
    llm_default, conservative, aggressive,
};

use std::future::Future;
use std::time::Duration;

/// Default retry configuration for LLM API calls.
/// Uses exponential backoff with jitter, retrying on rate limits and timeouts.
pub fn default_llm_retry_policy() -> RetryPolicy {
    infra_retry::llm_default()
}

/// Conservative retry policy with longer delays and fewer attempts.
/// Suitable for production workloads where minimizing API calls is important.
pub fn conservative_retry_policy() -> RetryPolicy {
    infra_retry::conservative()
}

/// Aggressive retry policy with shorter delays and more attempts.
/// Suitable for development/testing where faster feedback is preferred.
pub fn aggressive_retry_policy() -> RetryPolicy {
    infra_retry::aggressive()
}

/// Retry an LLM API call with the default policy.
///
/// This is a convenience wrapper that uses the recommended retry settings
/// for LLM providers (exponential backoff, rate limit awareness, etc.).
pub async fn retry_llm_call<F, Fut, T, E>(operation: F) -> Result<T, E>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    E: std::fmt::Debug,
{
    retry(default_llm_retry_policy(), operation).await
}

/// Retry with a custom policy.
pub async fn retry_with_policy<F, Fut, T, E>(
    policy: RetryPolicy,
    operation: F,
) -> Result<T, E>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    E: std::fmt::Debug,
{
    retry(policy, operation).await
}

/// Create a retry policy for streaming requests.
/// Uses shorter timeouts since streaming should begin quickly.
pub fn streaming_retry_policy() -> RetryPolicy {
    RetryPolicy::exponential()
        .max_attempts(2)
        .base_delay(Duration::from_millis(500))
        .max_delay(Duration::from_secs(5))
        .retry_on_timeout(true)
        .retry_on_rate_limit(true)
}

/// Create a retry policy for batch operations.
/// Uses longer delays to avoid overwhelming the API.
pub fn batch_retry_policy() -> RetryPolicy {
    RetryPolicy::linear()
        .max_attempts(5)
        .base_delay(Duration::from_secs(2))
        .max_delay(Duration::from_secs(60))
        .jitter(true)
        .retry_on_rate_limit(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_policy() {
        let policy = default_llm_retry_policy();
        assert_eq!(policy.max_attempts(), 3);
    }

    #[test]
    fn test_streaming_policy() {
        let policy = streaming_retry_policy();
        assert_eq!(policy.max_attempts(), 2);
    }

    #[test]
    fn test_batch_policy() {
        let policy = batch_retry_policy();
        assert_eq!(policy.max_attempts(), 5);
    }
}

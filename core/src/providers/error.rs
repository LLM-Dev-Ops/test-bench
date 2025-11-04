// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Error types for LLM providers.
//!
//! This module defines comprehensive error types that can occur when interacting
//! with LLM providers. All provider implementations should map their specific
//! errors to these common error types.

use std::time::Duration;
use thiserror::Error;

/// Errors that can occur when interacting with LLM providers.
///
/// This enum provides a comprehensive set of error types that cover all common
/// failure modes when working with LLM APIs. Provider implementations should
/// map their API-specific errors to these variants.
///
/// # Examples
///
/// ```
/// use llm_test_bench_core::providers::error::ProviderError;
/// use std::time::Duration;
///
/// let error = ProviderError::RateLimitExceeded {
///     retry_after: Some(Duration::from_secs(60)),
/// };
/// assert!(error.to_string().contains("Rate limit exceeded"));
/// ```
#[derive(Debug, Error)]
pub enum ProviderError {
    /// Authentication with the provider failed.
    ///
    /// This typically indicates an invalid API key or expired credentials.
    #[error("Authentication failed: {0}")]
    AuthenticationError(String),

    /// The provided API key is invalid or missing.
    ///
    /// This is a specific case of authentication failure where the API key
    /// itself is the problem (e.g., wrong format, not set in environment).
    #[error("Invalid API key")]
    InvalidApiKey,

    /// The provider's rate limit has been exceeded.
    ///
    /// The `retry_after` field indicates when the request can be retried,
    /// if the provider supplies this information.
    ///
    /// # Examples
    ///
    /// ```
    /// use llm_test_bench_core::providers::error::ProviderError;
    /// use std::time::Duration;
    ///
    /// let error = ProviderError::RateLimitExceeded {
    ///     retry_after: Some(Duration::from_secs(30)),
    /// };
    /// ```
    #[error("Rate limit exceeded. Retry after {retry_after:?}")]
    RateLimitExceeded {
        /// Optional duration to wait before retrying
        retry_after: Option<Duration>,
    },

    /// The specified model was not found or is not available.
    ///
    /// This can happen if:
    /// - The model name is misspelled
    /// - The model has been deprecated
    /// - The account doesn't have access to the model
    #[error("Model not found: {model}")]
    ModelNotFound {
        /// The model identifier that was not found
        model: String,
    },

    /// The request parameters are invalid.
    ///
    /// This indicates a problem with the request structure or parameters,
    /// such as invalid temperature values, malformed prompts, etc.
    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    /// The prompt exceeds the model's maximum context length.
    ///
    /// # Examples
    ///
    /// ```
    /// use llm_test_bench_core::providers::error::ProviderError;
    ///
    /// let error = ProviderError::ContextLengthExceeded {
    ///     tokens: 10000,
    ///     max: 8192,
    /// };
    /// assert_eq!(error.to_string(), "Context length exceeded: 10000 > 8192");
    /// ```
    #[error("Context length exceeded: {tokens} > {max}")]
    ContextLengthExceeded {
        /// Number of tokens in the request
        tokens: usize,
        /// Maximum tokens supported by the model
        max: usize,
    },

    /// A network error occurred while communicating with the provider.
    ///
    /// This wraps underlying HTTP client errors from `reqwest`.
    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),

    /// Failed to parse the provider's response.
    ///
    /// This wraps JSON parsing errors from `serde_json`.
    #[error("JSON parsing error: {0}")]
    ParseError(#[from] serde_json::Error),

    /// The provider returned an API error.
    ///
    /// This is a catch-all for provider-specific API errors that don't
    /// fit into the other categories.
    ///
    /// # Examples
    ///
    /// ```
    /// use llm_test_bench_core::providers::error::ProviderError;
    ///
    /// let error = ProviderError::ApiError {
    ///     status: 500,
    ///     message: "Internal server error".to_string(),
    /// };
    /// ```
    #[error("API error: {status} - {message}")]
    ApiError {
        /// HTTP status code
        status: u16,
        /// Error message from the provider
        message: String,
    },

    /// The request timed out.
    ///
    /// The duration indicates how long we waited before timing out.
    #[error("Timeout after {0:?}")]
    Timeout(Duration),

    /// An internal provider error occurred.
    ///
    /// This is used for unexpected errors within the provider implementation
    /// itself, not errors from the remote API.
    #[error("Provider internal error: {0}")]
    InternalError(String),
}

impl ProviderError {
    /// Returns `true` if this error is retryable.
    ///
    /// Retryable errors are those that might succeed if the request is
    /// attempted again after some delay (e.g., rate limits, timeouts,
    /// network errors).
    ///
    /// # Examples
    ///
    /// ```
    /// use llm_test_bench_core::providers::error::ProviderError;
    /// use std::time::Duration;
    ///
    /// let rate_limit = ProviderError::RateLimitExceeded { retry_after: None };
    /// assert!(rate_limit.is_retryable());
    ///
    /// let invalid_key = ProviderError::InvalidApiKey;
    /// assert!(!invalid_key.is_retryable());
    /// ```
    pub fn is_retryable(&self) -> bool {
        match self {
            ProviderError::RateLimitExceeded { .. } => true,
            ProviderError::NetworkError(_) => true,
            ProviderError::Timeout(_) => true,
            ProviderError::ApiError { status, .. } if *status >= 500 => true,
            _ => false,
        }
    }

    /// Returns the suggested retry delay for retryable errors.
    ///
    /// Returns `None` if the error is not retryable or if no specific
    /// delay is suggested.
    ///
    /// # Examples
    ///
    /// ```
    /// use llm_test_bench_core::providers::error::ProviderError;
    /// use std::time::Duration;
    ///
    /// let error = ProviderError::RateLimitExceeded {
    ///     retry_after: Some(Duration::from_secs(60)),
    /// };
    /// assert_eq!(error.retry_delay(), Some(Duration::from_secs(60)));
    /// ```
    pub fn retry_delay(&self) -> Option<Duration> {
        match self {
            ProviderError::RateLimitExceeded { retry_after } => *retry_after,
            _ => None,
        }
    }

    /// Returns `true` if this is an authentication-related error.
    ///
    /// # Examples
    ///
    /// ```
    /// use llm_test_bench_core::providers::error::ProviderError;
    ///
    /// let error = ProviderError::InvalidApiKey;
    /// assert!(error.is_auth_error());
    ///
    /// let error = ProviderError::AuthenticationError("Invalid credentials".to_string());
    /// assert!(error.is_auth_error());
    /// ```
    pub fn is_auth_error(&self) -> bool {
        matches!(
            self,
            ProviderError::AuthenticationError(_) | ProviderError::InvalidApiKey
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let error = ProviderError::InvalidApiKey;
        assert_eq!(error.to_string(), "Invalid API key");

        let error = ProviderError::RateLimitExceeded {
            retry_after: Some(Duration::from_secs(60)),
        };
        assert!(error.to_string().contains("Rate limit exceeded"));

        let error = ProviderError::ContextLengthExceeded {
            tokens: 10000,
            max: 8192,
        };
        assert_eq!(error.to_string(), "Context length exceeded: 10000 > 8192");
    }

    #[test]
    fn test_is_retryable() {
        assert!(ProviderError::RateLimitExceeded { retry_after: None }.is_retryable());
        assert!(ProviderError::Timeout(Duration::from_secs(30)).is_retryable());
        assert!(ProviderError::ApiError {
            status: 500,
            message: "Internal error".to_string()
        }
        .is_retryable());

        assert!(!ProviderError::InvalidApiKey.is_retryable());
        assert!(!ProviderError::ModelNotFound {
            model: "gpt-5".to_string()
        }
        .is_retryable());
        assert!(!ProviderError::ApiError {
            status: 400,
            message: "Bad request".to_string()
        }
        .is_retryable());
    }

    #[test]
    fn test_retry_delay() {
        let error = ProviderError::RateLimitExceeded {
            retry_after: Some(Duration::from_secs(60)),
        };
        assert_eq!(error.retry_delay(), Some(Duration::from_secs(60)));

        let error = ProviderError::InvalidApiKey;
        assert_eq!(error.retry_delay(), None);
    }

    #[test]
    fn test_is_auth_error() {
        assert!(ProviderError::InvalidApiKey.is_auth_error());
        assert!(ProviderError::AuthenticationError("test".to_string()).is_auth_error());
        assert!(!ProviderError::RateLimitExceeded { retry_after: None }.is_auth_error());
    }

    #[tokio::test]
    async fn test_from_reqwest_error() {
        // Test that we can convert from reqwest errors
        // We need to create a real reqwest error by making a bad request
        let client = reqwest::Client::new();
        let result = client.get("http://invalid-url-that-does-not-exist-12345.com").send().await;

        if let Err(error) = result {
            let provider_error: ProviderError = error.into();
            assert!(matches!(provider_error, ProviderError::NetworkError(_)));
        }
    }

    #[test]
    fn test_from_serde_error() {
        // Test that we can convert from serde errors
        let json = r#"{"invalid": json"#;
        let error: Result<serde_json::Value, _> = serde_json::from_str(json);
        let serde_error = error.unwrap_err();
        let provider_error: ProviderError = serde_error.into();
        assert!(matches!(provider_error, ProviderError::ParseError(_)));
    }
}

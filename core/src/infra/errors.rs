//! Error handling integration with infra-errors.
//!
//! This module provides bridge types and conversions between
//! test-bench's error types and the unified `InfraError` from infra-errors.

use infra_errors::{InfraError, InfraResult, ErrorContext, RetryConfig};

/// Convert anyhow::Error to InfraError
pub fn from_anyhow(err: anyhow::Error) -> InfraError {
    InfraError::Internal {
        message: err.to_string(),
        context: Some(ErrorContext::new()),
    }
}

/// Convert test-bench error types to InfraError
pub trait IntoInfraError {
    /// Convert to InfraError
    fn into_infra_error(self) -> InfraError;
}

impl IntoInfraError for anyhow::Error {
    fn into_infra_error(self) -> InfraError {
        from_anyhow(self)
    }
}

impl IntoInfraError for std::io::Error {
    fn into_infra_error(self) -> InfraError {
        InfraError::Io {
            operation: infra_errors::IoOperation::Read,
            path: None,
            source: self.to_string(),
        }
    }
}

impl IntoInfraError for serde_json::Error {
    fn into_infra_error(self) -> InfraError {
        InfraError::Serialization {
            format: infra_errors::SerializationFormat::Json,
            message: self.to_string(),
            context: None,
        }
    }
}

impl IntoInfraError for reqwest::Error {
    fn into_infra_error(self) -> InfraError {
        let status = self.status().map(|s| s.as_u16());
        InfraError::Http {
            status_code: status,
            message: self.to_string(),
            url: self.url().map(|u| u.to_string()),
        }
    }
}

/// Extension trait for Result types to convert to InfraResult
pub trait ResultExt<T, E> {
    /// Convert Result<T, E> to InfraResult<T>
    fn into_infra(self) -> InfraResult<T>
    where
        E: IntoInfraError;
}

impl<T, E: IntoInfraError> ResultExt<T, E> for Result<T, E> {
    fn into_infra(self) -> InfraResult<T> {
        self.map_err(|e| e.into_infra_error())
    }
}

/// Create a configuration error
pub fn config_error(key: Option<&str>, message: impl Into<String>) -> InfraError {
    InfraError::Config {
        key: key.map(String::from),
        message: message.into(),
        context: None,
    }
}

/// Create a validation error
pub fn validation_error(field: &str, message: impl Into<String>) -> InfraError {
    InfraError::Validation {
        field: field.to_string(),
        message: message.into(),
        context: None,
    }
}

/// Create default retry configuration for LLM API calls
pub fn default_llm_retry_config() -> RetryConfig {
    RetryConfig {
        max_attempts: 3,
        initial_delay: std::time::Duration::from_millis(1000),
        max_delay: std::time::Duration::from_secs(30),
        multiplier: 2.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_anyhow_conversion() {
        let err = anyhow::anyhow!("test error");
        let infra_err = from_anyhow(err);

        match infra_err {
            InfraError::Internal { message, .. } => {
                assert!(message.contains("test error"));
            }
            _ => panic!("Expected Internal error"),
        }
    }

    #[test]
    fn test_config_error() {
        let err = config_error(Some("api_key"), "missing required field");

        match err {
            InfraError::Config { key, message, .. } => {
                assert_eq!(key, Some("api_key".to_string()));
                assert!(message.contains("missing"));
            }
            _ => panic!("Expected Config error"),
        }
    }

    #[test]
    fn test_retry_config() {
        let config = default_llm_retry_config();
        assert_eq!(config.max_attempts, 3);
    }
}

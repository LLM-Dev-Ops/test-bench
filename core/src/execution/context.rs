//! Execution context provided by the calling Core.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Execution context provided by the calling Core.
/// Every externally-invoked operation MUST receive this.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionContext {
    /// Unique identifier for this execution graph.
    pub execution_id: String,

    /// Span ID from the Core that parents this repo's span.
    pub parent_span_id: String,
}

/// Error returned when execution context is missing or invalid.
#[derive(Debug)]
pub struct ExecutionContextError {
    pub message: String,
}

impl fmt::Display for ExecutionContextError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ExecutionContextError: {}", self.message)
    }
}

impl std::error::Error for ExecutionContextError {}

impl From<ExecutionContextError> for anyhow::Error {
    fn from(e: ExecutionContextError) -> Self {
        anyhow::anyhow!("{}", e)
    }
}

/// Validate that an ExecutionContext is structurally valid.
/// Rejects execution if parent_span_id is missing.
pub fn validate_execution_context(ctx: &ExecutionContext) -> Result<(), ExecutionContextError> {
    if ctx.execution_id.is_empty() {
        return Err(ExecutionContextError {
            message: "execution_id is required and must be non-empty".to_string(),
        });
    }
    if ctx.parent_span_id.is_empty() {
        return Err(ExecutionContextError {
            message: "parent_span_id is required and must be non-empty. \
                     Every externally-invoked operation must provide a valid \
                     parent_span_id from the Core."
                .to_string(),
        });
    }
    Ok(())
}

/// Extract ExecutionContext from HTTP headers.
///
/// Expected headers:
/// - `X-Execution-Id`: the execution graph ID
/// - `X-Parent-Span-Id`: the Core's span ID
pub fn extract_from_headers(
    headers: &axum::http::HeaderMap,
) -> Result<ExecutionContext, ExecutionContextError> {
    let execution_id = headers
        .get("X-Execution-Id")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();

    let parent_span_id = headers
        .get("X-Parent-Span-Id")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();

    let ctx = ExecutionContext {
        execution_id,
        parent_span_id,
    };

    validate_execution_context(&ctx)?;
    Ok(ctx)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_context() {
        let ctx = ExecutionContext {
            execution_id: "exec-123".to_string(),
            parent_span_id: "core-span-456".to_string(),
        };
        assert!(validate_execution_context(&ctx).is_ok());
    }

    #[test]
    fn test_missing_execution_id() {
        let ctx = ExecutionContext {
            execution_id: "".to_string(),
            parent_span_id: "core-span-456".to_string(),
        };
        assert!(validate_execution_context(&ctx).is_err());
    }

    #[test]
    fn test_missing_parent_span_id() {
        let ctx = ExecutionContext {
            execution_id: "exec-123".to_string(),
            parent_span_id: "".to_string(),
        };
        let err = validate_execution_context(&ctx).unwrap_err();
        assert!(err.message.contains("parent_span_id"));
    }
}

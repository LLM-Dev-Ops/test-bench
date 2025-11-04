// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! API data models.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// Health check response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct HealthResponse {
    /// Service status
    pub status: String,
    /// API version
    pub version: String,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Service uptime in seconds
    pub uptime_seconds: u64,
}

/// Completion request
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CompletionRequest {
    /// Provider name
    pub provider: String,
    /// Model name
    pub model: String,
    /// Prompt text
    pub prompt: String,
    /// Maximum tokens to generate
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<usize>,
    /// Temperature (0.0 to 2.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    /// Enable streaming
    #[serde(default)]
    pub stream: bool,
}

/// Completion response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CompletionResponse {
    /// Generated text
    pub text: String,
    /// Model used
    pub model: String,
    /// Token usage
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<TokenUsage>,
    /// Request ID
    pub request_id: String,
    /// Timestamp
    pub created_at: DateTime<Utc>,
}

/// Token usage information
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TokenUsage {
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub total_tokens: u64,
}

/// Evaluation request
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct EvaluationRequest {
    /// Provider name
    pub provider: String,
    /// Model name
    pub model: String,
    /// Input text
    pub input: String,
    /// Output text
    pub output: String,
    /// Expected output (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected: Option<String>,
    /// Metrics to evaluate
    pub metrics: Vec<String>,
}

/// Evaluation response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct EvaluationResponse {
    /// Overall score
    pub score: f64,
    /// Individual metric scores
    pub metrics: HashMap<String, f64>,
    /// Evaluation ID
    pub evaluation_id: String,
    /// Timestamp
    pub created_at: DateTime<Utc>,
}

/// Benchmark request
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct BenchmarkRequest {
    /// Benchmark name
    pub name: String,
    /// Providers to benchmark
    pub providers: Vec<String>,
    /// Models to benchmark
    pub models: Vec<String>,
    /// Dataset to use
    pub dataset: String,
    /// Number of examples
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_examples: Option<usize>,
}

/// Benchmark response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct BenchmarkResponse {
    /// Benchmark ID
    pub benchmark_id: String,
    /// Status
    pub status: BenchmarkStatus,
    /// Progress (0-100)
    pub progress: f64,
    /// Results (when completed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub results: Option<BenchmarkResults>,
}

/// Benchmark status
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum BenchmarkStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

/// Benchmark results
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct BenchmarkResults {
    /// Results by provider
    pub by_provider: HashMap<String, ProviderResults>,
    /// Total duration in seconds
    pub duration_seconds: f64,
}

/// Provider benchmark results
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ProviderResults {
    /// Provider name
    pub provider: String,
    /// Model name
    pub model: String,
    /// Average score
    pub avg_score: f64,
    /// Average latency (seconds)
    pub avg_latency: f64,
    /// Total cost (USD)
    pub total_cost: f64,
    /// Error rate
    pub error_rate: f64,
}

/// Plugin information
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PluginInfo {
    /// Plugin ID
    pub id: String,
    /// Plugin name
    pub name: String,
    /// Plugin version
    pub version: String,
    /// Plugin type
    pub plugin_type: String,
    /// Status
    pub status: String,
    /// Execution count
    pub execution_count: u64,
}

/// Plugin execution request
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PluginExecuteRequest {
    /// Plugin ID
    pub plugin_id: String,
    /// Input data
    pub input: serde_json::Value,
}

/// Plugin execution response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PluginExecuteResponse {
    /// Output data
    pub output: serde_json::Value,
    /// Execution time (ms)
    pub execution_time_ms: u64,
    /// Request ID
    pub request_id: String,
}

/// Paginated response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PaginatedResponse<T> {
    /// Items
    pub items: Vec<T>,
    /// Total count
    pub total: usize,
    /// Page number (1-indexed)
    pub page: usize,
    /// Page size
    pub page_size: usize,
    /// Total pages
    pub total_pages: usize,
}

impl<T> PaginatedResponse<T> {
    pub fn new(items: Vec<T>, total: usize, page: usize, page_size: usize) -> Self {
        let total_pages = (total + page_size - 1) / page_size;
        Self {
            items,
            total,
            page,
            page_size,
            total_pages,
        }
    }
}

/// API error response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ErrorResponse {
    /// Error message
    pub error: String,
    /// Error code
    pub code: String,
    /// Detailed message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    /// Request ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

impl ErrorResponse {
    pub fn new(error: impl Into<String>, code: impl Into<String>) -> Self {
        Self {
            error: error.into(),
            code: code.into(),
            detail: None,
            request_id: None,
            timestamp: Utc::now(),
        }
    }

    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    pub fn with_request_id(mut self, request_id: impl Into<String>) -> Self {
        self.request_id = Some(request_id.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_paginated_response() {
        let items = vec![1, 2, 3, 4, 5];
        let response = PaginatedResponse::new(items.clone(), 15, 1, 5);

        assert_eq!(response.items, items);
        assert_eq!(response.total, 15);
        assert_eq!(response.page, 1);
        assert_eq!(response.total_pages, 3);
    }

    #[test]
    fn test_error_response() {
        let error = ErrorResponse::new("Test error", "TEST_ERROR")
            .with_detail("Detailed message")
            .with_request_id("req_123");

        assert_eq!(error.error, "Test error");
        assert_eq!(error.code, "TEST_ERROR");
        assert_eq!(error.detail, Some("Detailed message".to_string()));
        assert_eq!(error.request_id, Some("req_123".to_string()));
    }
}

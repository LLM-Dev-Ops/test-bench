// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Shared types for LLM provider interactions.
//!
//! This module defines common types used across all provider implementations,
//! including request/response structures, token usage, and model information.

use chrono::{DateTime, Utc};
use futures::Stream;
use serde::{Deserialize, Serialize};
use std::pin::Pin;

use super::error::ProviderError;

/// A completion request to send to an LLM provider.
///
/// This structure provides a common interface for making completion requests
/// across different providers. Provider implementations will translate these
/// fields to their provider-specific API formats.
///
/// # Examples
///
/// ```
/// use llm_test_bench_core::providers::types::CompletionRequest;
///
/// let request = CompletionRequest {
///     model: "gpt-4".to_string(),
///     prompt: "Explain Rust ownership".to_string(),
///     max_tokens: Some(100),
///     temperature: Some(0.7),
///     top_p: Some(0.9),
///     stop: Some(vec!["\n\n".to_string()]),
///     stream: false,
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CompletionRequest {
    /// The model identifier to use (e.g., "gpt-4", "claude-3-opus-20240229").
    pub model: String,

    /// The prompt or input text to send to the model.
    pub prompt: String,

    /// Maximum number of tokens to generate in the completion.
    ///
    /// If `None`, the provider's default will be used.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<usize>,

    /// Sampling temperature between 0.0 and 2.0.
    ///
    /// Higher values make output more random, lower values more deterministic.
    /// If `None`, the provider's default (typically 0.7 or 1.0) will be used.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,

    /// Nucleus sampling parameter between 0.0 and 1.0.
    ///
    /// An alternative to temperature sampling. If `None`, the provider's
    /// default will be used.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,

    /// Sequences where the model should stop generating.
    ///
    /// The model will stop generating when it encounters any of these sequences.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<String>>,

    /// Whether to stream the response as it's generated.
    ///
    /// If `true`, the provider should return a `ResponseStream`. If `false`,
    /// the provider should return a complete response.
    #[serde(default)]
    pub stream: bool,
}

impl CompletionRequest {
    /// Creates a new completion request with the specified model and prompt.
    ///
    /// All optional parameters are set to `None` and streaming is disabled.
    ///
    /// # Examples
    ///
    /// ```
    /// use llm_test_bench_core::providers::types::CompletionRequest;
    ///
    /// let request = CompletionRequest::new("gpt-4", "Hello, world!");
    /// assert_eq!(request.model, "gpt-4");
    /// assert_eq!(request.prompt, "Hello, world!");
    /// assert!(!request.stream);
    /// ```
    pub fn new(model: impl Into<String>, prompt: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            prompt: prompt.into(),
            max_tokens: None,
            temperature: None,
            top_p: None,
            stop: None,
            stream: false,
        }
    }

    /// Sets the maximum number of tokens to generate.
    pub fn with_max_tokens(mut self, max_tokens: usize) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    /// Sets the sampling temperature.
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }

    /// Sets the nucleus sampling parameter.
    pub fn with_top_p(mut self, top_p: f32) -> Self {
        self.top_p = Some(top_p);
        self
    }

    /// Sets the stop sequences.
    pub fn with_stop(mut self, stop: Vec<String>) -> Self {
        self.stop = Some(stop);
        self
    }

    /// Enables streaming mode.
    pub fn with_streaming(mut self) -> Self {
        self.stream = true;
        self
    }
}

/// A completion response from an LLM provider.
///
/// This structure provides a common format for responses across different
/// providers. Provider implementations translate their API-specific responses
/// to this format.
///
/// # Examples
///
/// ```
/// use llm_test_bench_core::providers::types::{CompletionResponse, TokenUsage, FinishReason};
/// use chrono::Utc;
///
/// let response = CompletionResponse {
///     id: "cmpl-123".to_string(),
///     model: "gpt-4".to_string(),
///     content: "Hello! How can I help you?".to_string(),
///     usage: TokenUsage {
///         prompt_tokens: 10,
///         completion_tokens: 8,
///         total_tokens: 18,
///     },
///     finish_reason: FinishReason::Stop,
///     created_at: Utc::now(),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CompletionResponse {
    /// Unique identifier for this completion.
    pub id: String,

    /// The model that generated this completion.
    pub model: String,

    /// The generated text content.
    pub content: String,

    /// Token usage information for this completion.
    pub usage: TokenUsage,

    /// The reason the model stopped generating.
    pub finish_reason: FinishReason,

    /// Timestamp when the completion was created.
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
}

/// Token usage information for a completion.
///
/// # Examples
///
/// ```
/// use llm_test_bench_core::providers::types::TokenUsage;
///
/// let usage = TokenUsage {
///     prompt_tokens: 50,
///     completion_tokens: 100,
///     total_tokens: 150,
/// };
/// assert_eq!(usage.total_tokens, 150);
/// ```
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct TokenUsage {
    /// Number of tokens in the prompt.
    pub prompt_tokens: usize,

    /// Number of tokens in the completion.
    pub completion_tokens: usize,

    /// Total tokens used (prompt + completion).
    pub total_tokens: usize,
}

impl TokenUsage {
    /// Creates a new `TokenUsage` from prompt and completion token counts.
    ///
    /// The total is automatically calculated.
    ///
    /// # Examples
    ///
    /// ```
    /// use llm_test_bench_core::providers::types::TokenUsage;
    ///
    /// let usage = TokenUsage::new(50, 100);
    /// assert_eq!(usage.total_tokens, 150);
    /// ```
    pub fn new(prompt_tokens: usize, completion_tokens: usize) -> Self {
        Self {
            prompt_tokens,
            completion_tokens,
            total_tokens: prompt_tokens + completion_tokens,
        }
    }

    /// Returns the total cost of this usage at the given rates.
    ///
    /// # Arguments
    ///
    /// * `prompt_cost_per_1k` - Cost per 1,000 prompt tokens
    /// * `completion_cost_per_1k` - Cost per 1,000 completion tokens
    ///
    /// # Examples
    ///
    /// ```
    /// use llm_test_bench_core::providers::types::TokenUsage;
    ///
    /// let usage = TokenUsage::new(1000, 2000);
    /// let cost = usage.calculate_cost(0.03, 0.06);
    /// assert_eq!(cost, 0.15); // (1000 * 0.03 / 1000) + (2000 * 0.06 / 1000)
    /// ```
    pub fn calculate_cost(&self, prompt_cost_per_1k: f64, completion_cost_per_1k: f64) -> f64 {
        let prompt_cost = (self.prompt_tokens as f64 / 1000.0) * prompt_cost_per_1k;
        let completion_cost = (self.completion_tokens as f64 / 1000.0) * completion_cost_per_1k;
        prompt_cost + completion_cost
    }
}

/// The reason a completion finished.
///
/// # Examples
///
/// ```
/// use llm_test_bench_core::providers::types::FinishReason;
///
/// let reason = FinishReason::Stop;
/// assert_eq!(reason.to_string(), "stop");
/// ```
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FinishReason {
    /// The model finished naturally (e.g., reached end of thought).
    Stop,

    /// The model stopped because it reached the maximum token limit.
    Length,

    /// The model stopped due to content filtering.
    ContentFilter,

    /// The model stopped to invoke a tool/function.
    ToolCalls,

    /// The model stopped due to an error.
    Error,
}

impl std::fmt::Display for FinishReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FinishReason::Stop => write!(f, "stop"),
            FinishReason::Length => write!(f, "length"),
            FinishReason::ContentFilter => write!(f, "content_filter"),
            FinishReason::ToolCalls => write!(f, "tool_calls"),
            FinishReason::Error => write!(f, "error"),
        }
    }
}

/// Information about a model supported by a provider.
///
/// # Examples
///
/// ```
/// use llm_test_bench_core::providers::types::ModelInfo;
///
/// let model = ModelInfo {
///     id: "gpt-4".to_string(),
///     name: "GPT-4".to_string(),
///     max_tokens: 8192,
///     supports_streaming: true,
///     supports_function_calling: true,
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModelInfo {
    /// Unique identifier for the model (e.g., "gpt-4", "claude-3-opus-20240229").
    pub id: String,

    /// Human-readable name for the model.
    pub name: String,

    /// Maximum number of tokens this model can process (context window).
    pub max_tokens: usize,

    /// Whether this model supports streaming responses.
    pub supports_streaming: bool,

    /// Whether this model supports function/tool calling.
    pub supports_function_calling: bool,
}

impl ModelInfo {
    /// Creates a new `ModelInfo` with the specified parameters.
    ///
    /// # Examples
    ///
    /// ```
    /// use llm_test_bench_core::providers::types::ModelInfo;
    ///
    /// let model = ModelInfo::new(
    ///     "gpt-4",
    ///     "GPT-4",
    ///     8192,
    ///     true,
    ///     true,
    /// );
    /// assert_eq!(model.id, "gpt-4");
    /// ```
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        max_tokens: usize,
        supports_streaming: bool,
        supports_function_calling: bool,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            max_tokens,
            supports_streaming,
            supports_function_calling,
        }
    }
}

/// A stream of response chunks from a provider.
///
/// This type represents a streaming response where tokens are yielded
/// incrementally as they're generated by the model.
///
/// Each item in the stream is either a chunk of text or an error.
pub type ResponseStream = Pin<Box<dyn Stream<Item = Result<String, ProviderError>> + Send>>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_completion_request_new() {
        let request = CompletionRequest::new("gpt-4", "Hello");
        assert_eq!(request.model, "gpt-4");
        assert_eq!(request.prompt, "Hello");
        assert_eq!(request.max_tokens, None);
        assert_eq!(request.temperature, None);
        assert!(!request.stream);
    }

    #[test]
    fn test_completion_request_builder() {
        let request = CompletionRequest::new("gpt-4", "Hello")
            .with_max_tokens(100)
            .with_temperature(0.8)
            .with_top_p(0.9)
            .with_stop(vec!["\n".to_string()])
            .with_streaming();

        assert_eq!(request.max_tokens, Some(100));
        assert_eq!(request.temperature, Some(0.8));
        assert_eq!(request.top_p, Some(0.9));
        assert_eq!(request.stop, Some(vec!["\n".to_string()]));
        assert!(request.stream);
    }

    #[test]
    fn test_token_usage_new() {
        let usage = TokenUsage::new(50, 100);
        assert_eq!(usage.prompt_tokens, 50);
        assert_eq!(usage.completion_tokens, 100);
        assert_eq!(usage.total_tokens, 150);
    }

    #[test]
    fn test_token_usage_calculate_cost() {
        let usage = TokenUsage::new(1000, 2000);
        let cost = usage.calculate_cost(0.03, 0.06);
        assert!((cost - 0.15).abs() < 0.001); // Account for floating point
    }

    #[test]
    fn test_finish_reason_display() {
        assert_eq!(FinishReason::Stop.to_string(), "stop");
        assert_eq!(FinishReason::Length.to_string(), "length");
        assert_eq!(FinishReason::ContentFilter.to_string(), "content_filter");
        assert_eq!(FinishReason::ToolCalls.to_string(), "tool_calls");
        assert_eq!(FinishReason::Error.to_string(), "error");
    }

    #[test]
    fn test_model_info_new() {
        let model = ModelInfo::new("gpt-4", "GPT-4", 8192, true, true);
        assert_eq!(model.id, "gpt-4");
        assert_eq!(model.name, "GPT-4");
        assert_eq!(model.max_tokens, 8192);
        assert!(model.supports_streaming);
        assert!(model.supports_function_calling);
    }

    #[test]
    fn test_completion_request_serialization() {
        let request = CompletionRequest::new("gpt-4", "Hello")
            .with_max_tokens(100)
            .with_temperature(0.7);

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: CompletionRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(request, deserialized);
    }

    #[test]
    fn test_completion_response_serialization() {
        let response = CompletionResponse {
            id: "test-123".to_string(),
            model: "gpt-4".to_string(),
            content: "Hello!".to_string(),
            usage: TokenUsage::new(10, 5),
            finish_reason: FinishReason::Stop,
            created_at: Utc::now(),
        };

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: CompletionResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(response.id, deserialized.id);
        assert_eq!(response.content, deserialized.content);
    }

    #[test]
    fn test_token_usage_serialization() {
        let usage = TokenUsage::new(50, 100);
        let json = serde_json::to_string(&usage).unwrap();
        let deserialized: TokenUsage = serde_json::from_str(&json).unwrap();
        assert_eq!(usage, deserialized);
    }

    #[test]
    fn test_finish_reason_serialization() {
        let reason = FinishReason::Stop;
        let json = serde_json::to_string(&reason).unwrap();
        assert_eq!(json, "\"stop\"");

        let deserialized: FinishReason = serde_json::from_str(&json).unwrap();
        assert_eq!(reason, deserialized);
    }

    #[test]
    fn test_model_info_serialization() {
        let model = ModelInfo::new("gpt-4", "GPT-4", 8192, true, true);
        let json = serde_json::to_string(&model).unwrap();
        let deserialized: ModelInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(model, deserialized);
    }
}

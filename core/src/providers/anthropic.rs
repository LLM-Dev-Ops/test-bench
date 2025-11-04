// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Anthropic Claude provider implementation
//!
//! This module provides integration with Anthropic's Claude API,
//! supporting Claude 3 Opus, Sonnet, and Haiku models with their
//! 200,000 token context windows.
//!
//! # Features
//!
//! - Non-streaming completions
//! - Streaming completions with SSE
//! - Support for all Claude 3 models
//! - Automatic retry with exponential backoff
//! - 200K context window support
//!
//! # Example
//!
//! ```no_run
//! use llm_test_bench_core::providers::{AnthropicProvider, CompletionRequest, Provider};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let provider = AnthropicProvider::new("your-api-key".to_string());
//!
//! let request = CompletionRequest {
//!     model: "claude-3-sonnet-20240229".to_string(),
//!     prompt: "Hello, Claude!".to_string(),
//!     temperature: 0.7,
//!     max_tokens: Some(1024),
//!     extra: serde_json::Value::Null,
//! };
//!
//! let response = provider.complete(&request).await?;
//! println!("Response: {}", response.content);
//! # Ok(())
//! # }
//! ```

use super::{CompletionRequest, CompletionResponse, FinishReason, ModelInfo, Provider, ProviderError, ResponseStream, TokenUsage};
use async_trait::async_trait;
use futures::StreamExt;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use reqwest_eventsource::{Event, EventSource};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{debug, trace, warn};

const ANTHROPIC_API_VERSION: &str = "2023-06-01";
const DEFAULT_TIMEOUT_SECS: u64 = 300; // 5 minutes for large context
const MAX_RETRIES: u32 = 3;
const BASE_RETRY_DELAY_MS: u64 = 1000;

/// Anthropic Claude provider
///
/// Implements the Provider trait for Anthropic's Claude API.
/// Supports streaming and non-streaming completions with automatic
/// retry logic and comprehensive error handling.
pub struct AnthropicProvider {
    client: reqwest::Client,
    api_key: String,
    base_url: String,
    max_retries: u32,
}

impl AnthropicProvider {
    /// Create a new Anthropic provider with default settings
    ///
    /// # Arguments
    ///
    /// * `api_key` - Anthropic API key
    pub fn new(api_key: String) -> Self {
        Self::with_config(api_key, "https://api.anthropic.com/v1".to_string(), MAX_RETRIES)
    }

    /// Create a new Anthropic provider with custom base URL
    ///
    /// Useful for testing with mock servers.
    ///
    /// # Arguments
    ///
    /// * `api_key` - Anthropic API key
    /// * `base_url` - Custom base URL (e.g., for testing)
    pub fn with_base_url(api_key: String, base_url: String) -> Self {
        Self::with_config(api_key, base_url, MAX_RETRIES)
    }

    /// Create a new Anthropic provider with full configuration
    ///
    /// # Arguments
    ///
    /// * `api_key` - Anthropic API key
    /// * `base_url` - API base URL
    /// * `max_retries` - Maximum number of retry attempts
    pub fn with_config(api_key: String, base_url: String, max_retries: u32) -> Self {
        let mut headers = HeaderMap::new();
        headers.insert("anthropic-version", HeaderValue::from_static(ANTHROPIC_API_VERSION));

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS))
            .default_headers(headers)
            .build()
            .expect("Failed to build HTTP client");

        Self {
            client,
            api_key,
            base_url,
            max_retries,
        }
    }

    /// Build the request payload for Claude Messages API
    fn build_request_body(&self, request: &CompletionRequest, stream: bool) -> ClaudeRequest {
        ClaudeRequest {
            model: request.model.clone(),
            messages: vec![ClaudeMessage {
                role: "user".to_string(),
                content: request.prompt.clone(),
            }],
            max_tokens: request.max_tokens.unwrap_or(1024) as u32,
            temperature: request.temperature,
            stream: Some(stream),
            top_p: request.top_p,
            top_k: None,
            system: None,
        }
    }

    /// Execute a completion request with retry logic
    async fn complete_with_retry(&self, request: &CompletionRequest) -> Result<CompletionResponse, ProviderError> {
        let mut attempts = 0;
        let mut last_error = None;

        while attempts <= self.max_retries {
            if attempts > 0 {
                let delay = Self::calculate_backoff(attempts - 1);
                debug!("Retrying after {:?} (attempt {}/{})", delay, attempts, self.max_retries);
                tokio::time::sleep(delay).await;
            }

            match self.complete_once(request).await {
                Ok(response) => return Ok(response),
                Err(e) if Self::is_retryable(&e) && attempts < self.max_retries => {
                    warn!("Retryable error: {}", e);
                    last_error = Some(e);
                    attempts += 1;
                }
                Err(e) => return Err(e),
            }
        }

        Err(last_error.unwrap_or_else(|| ProviderError::InvalidRequest("Max retries exceeded".to_string())))
    }

    /// Execute a single completion request without retry
    async fn complete_once(&self, request: &CompletionRequest) -> Result<CompletionResponse, ProviderError> {
        let url = format!("{}/messages", self.base_url);
        let body = self.build_request_body(request, false);

        trace!("Sending request to Claude API: {:?}", body);

        let response = self
            .client
            .post(&url)
            .header("x-api-key", &self.api_key)
            .header(CONTENT_TYPE, "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| ProviderError::InvalidRequest(format!("HTTP request failed: {}", e)))?;

        let status = response.status();
        let response_text = response
            .text()
            .await
            .map_err(|e| ProviderError::ApiError { status: 500, message: format!("Failed to read response: {}", e) })?;

        trace!("Received response (status {}): {}", status, response_text);

        if !status.is_success() {
            return Err(Self::parse_error(status.as_u16(), &response_text));
        }

        let claude_response: ClaudeResponse = serde_json::from_str(&response_text)
            .map_err(|e| ProviderError::ApiError { status: 500, message: format!("Failed to parse response: {}", e) })?;

        Ok(Self::convert_response(claude_response))
    }

    /// Stream completion with retry logic
    async fn stream_completion(&self, request: &CompletionRequest) -> Result<ResponseStream, ProviderError> {
        let url = format!("{}/messages", self.base_url);
        let body = self.build_request_body(request, true);

        debug!("Starting streaming request to Claude API");

        let request_builder = self
            .client
            .post(&url)
            .header("x-api-key", &self.api_key)
            .header(CONTENT_TYPE, "application/json")
            .json(&body);

        let event_source = EventSource::new(request_builder)
            .map_err(|e| ProviderError::InvalidRequest(format!("Failed to create event source: {}", e)))?;

        let stream = event_source.filter_map(|event| async move {
            match event {
                Ok(Event::Open) => {
                    debug!("SSE connection opened");
                    None
                }
                Ok(Event::Message(message)) => {
                    trace!("Received SSE message: {:?}", message);
                    Self::parse_streaming_event(&message.data)
                }
                Err(e) => {
                    warn!("SSE error: {}", e);
                    Some(Err(ProviderError::InternalError(format!("Streaming error: {}", e))))
                }
            }
        });

        Ok(Box::pin(stream))
    }

    /// Parse a streaming event from Claude's SSE format
    fn parse_streaming_event(data: &str) -> Option<Result<String, ProviderError>> {
        trace!("Parsing streaming event: {}", data);

        let event: Result<ClaudeStreamEvent, _> = serde_json::from_str(data);

        match event {
            Ok(ClaudeStreamEvent::ContentBlockDelta { delta, .. }) => {
                if let ClaudeDelta::TextDelta { text } = delta {
                    if !text.is_empty() {
                        Some(Ok(text))
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            Ok(ClaudeStreamEvent::MessageStop) => {
                debug!("Stream completed");
                None // Don't send empty final chunk
            }
            Ok(_) => None, // Other event types we don't need
            Err(e) => {
                warn!("Failed to parse streaming event: {}", e);
                None
            }
        }
    }

    /// Convert Claude response to our standard format
    fn convert_response(response: ClaudeResponse) -> CompletionResponse {
        let content = response
            .content
            .into_iter()
            .filter_map(|c| match c {
                ClaudeContent::Text { text } => Some(text),
            })
            .collect::<Vec<_>>()
            .join("");

        let finish_reason = match response.stop_reason.as_deref() {
            Some("end_turn") | Some("stop_sequence") => FinishReason::Stop,
            Some("max_tokens") => FinishReason::Length,
            Some("content_filter") => FinishReason::ContentFilter,
            Some("tool_use") => FinishReason::ToolCalls,
            _ => FinishReason::Error,
        };

        CompletionResponse {
            id: response.id.clone(),
            content,
            model: response.model,
            usage: TokenUsage {
                prompt_tokens: response.usage.input_tokens as usize,
                completion_tokens: response.usage.output_tokens as usize,
                total_tokens: (response.usage.input_tokens + response.usage.output_tokens) as usize,
            },
            finish_reason,
            created_at: chrono::Utc::now(),
        }
    }

    /// Parse error response from Claude API
    fn parse_error(status: u16, body: &str) -> ProviderError {
        if let Ok(error_response) = serde_json::from_str::<ClaudeErrorResponse>(body) {
            match error_response.error.error_type.as_str() {
                "authentication_error" => ProviderError::AuthenticationError(error_response.error.message),
                "invalid_request_error" => ProviderError::InvalidRequest(error_response.error.message),
                "rate_limit_error" => ProviderError::RateLimitExceeded { retry_after: None },
                _ => ProviderError::InvalidRequest(format!("API error: {}", error_response.error.message)),
            }
        } else {
            match status {
                401 => ProviderError::AuthenticationError("Invalid API key".to_string()),
                429 => ProviderError::RateLimitExceeded { retry_after: None },
                _ => ProviderError::InvalidRequest(format!("HTTP {}: {}", status, body)),
            }
        }
    }

    /// Check if an error is retryable
    fn is_retryable(error: &ProviderError) -> bool {
        matches!(error, ProviderError::RateLimitExceeded { .. } | ProviderError::InvalidRequest(_))
    }

    /// Calculate exponential backoff delay
    fn calculate_backoff(attempt: u32) -> Duration {
        let delay_ms = BASE_RETRY_DELAY_MS * 2_u64.pow(attempt);
        let max_delay_ms = 60_000; // 60 seconds max
        Duration::from_millis(delay_ms.min(max_delay_ms))
    }
}

#[async_trait]
impl Provider for AnthropicProvider {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse, ProviderError> {
        self.complete_with_retry(&request).await
    }

    async fn stream(&self, request: CompletionRequest) -> Result<ResponseStream, ProviderError> {
        self.stream_completion(&request).await
    }

    fn supported_models(&self) -> Vec<ModelInfo> {
        vec![
            ModelInfo {
                id: "claude-3-opus-20240229".to_string(),
                name: "Claude 3 Opus".to_string(),
                max_tokens: 200_000,
                supports_streaming: true,
                supports_function_calling: true,
            },
            ModelInfo {
                id: "claude-3-sonnet-20240229".to_string(),
                name: "Claude 3 Sonnet".to_string(),
                max_tokens: 200_000,
                supports_streaming: true,
                supports_function_calling: true,
            },
            ModelInfo {
                id: "claude-3-haiku-20240307".to_string(),
                name: "Claude 3 Haiku".to_string(),
                max_tokens: 200_000,
                supports_streaming: true,
                supports_function_calling: true,
            },
        ]
    }

    fn max_context_length(&self, model: &str) -> Option<usize> {
        self.supported_models()
            .iter()
            .find(|m| m.id == model)
            .map(|m| m.max_tokens)
    }

    fn name(&self) -> &str {
        "Anthropic"
    }

    async fn validate_config(&self) -> Result<(), ProviderError> {
        if self.api_key.is_empty() {
            return Err(ProviderError::InvalidApiKey);
        }
        // Could optionally make a test request here
        Ok(())
    }

    fn estimate_tokens(&self, text: &str, _model: &str) -> Result<usize, ProviderError> {
        // Rough approximation: 4 characters per token
        Ok((text.len() as f64 / 4.0).ceil() as usize)
    }
}

// Claude API request/response types

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ClaudeRequest {
    model: String,
    messages: Vec<ClaudeMessage>,
    max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_k: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ClaudeMessage {
    role: String,
    content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ClaudeResponse {
    id: String,
    #[serde(rename = "type")]
    response_type: String,
    role: String,
    content: Vec<ClaudeContent>,
    model: String,
    stop_reason: Option<String>,
    usage: ClaudeUsage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
enum ClaudeContent {
    #[serde(rename = "text")]
    Text { text: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ClaudeUsage {
    input_tokens: u32,
    output_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ClaudeErrorResponse {
    #[serde(rename = "type")]
    error_type: String,
    error: ClaudeError,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ClaudeError {
    #[serde(rename = "type")]
    error_type: String,
    message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
enum ClaudeStreamEvent {
    #[serde(rename = "message_start")]
    MessageStart { message: serde_json::Value },
    #[serde(rename = "content_block_start")]
    ContentBlockStart { index: usize, content_block: serde_json::Value },
    #[serde(rename = "content_block_delta")]
    ContentBlockDelta { index: usize, delta: ClaudeDelta },
    #[serde(rename = "content_block_stop")]
    ContentBlockStop { index: usize },
    #[serde(rename = "message_delta")]
    MessageDelta { delta: serde_json::Value, usage: serde_json::Value },
    #[serde(rename = "message_stop")]
    MessageStop,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
enum ClaudeDelta {
    #[serde(rename = "text_delta")]
    TextDelta { text: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_anthropic_provider_creation() {
        let provider = AnthropicProvider::new("test_key".to_string());
        assert_eq!(provider.name(), "Anthropic");
        assert_eq!(provider.supported_models().len(), 3);
        assert_eq!(provider.max_retries, MAX_RETRIES);
    }

    #[test]
    fn test_supported_models() {
        let provider = AnthropicProvider::new("test_key".to_string());
        let models = provider.supported_models();

        assert_eq!(models.len(), 3);
        assert!(models.iter().any(|m| m.id == "claude-3-opus-20240229"));
        assert!(models.iter().any(|m| m.id == "claude-3-sonnet-20240229"));
        assert!(models.iter().any(|m| m.id == "claude-3-haiku-20240307"));

        for model in models {
            assert_eq!(model.max_tokens, 200_000);
            assert!(model.supports_streaming);
        }
    }

    #[test]
    fn test_build_request_body() {
        let provider = AnthropicProvider::new("test_key".to_string());
        let request = CompletionRequest {
            model: "claude-3-sonnet-20240229".to_string(),
            prompt: "Hello, Claude!".to_string(),
            temperature: Some(0.7),
            max_tokens: Some(100),
            top_p: None,
            stop: None,
            stream: false,
        };

        let body = provider.build_request_body(&request, false);

        assert_eq!(body.model, "claude-3-sonnet-20240229");
        assert_eq!(body.messages.len(), 1);
        assert_eq!(body.messages[0].role, "user");
        assert_eq!(body.messages[0].content, "Hello, Claude!");
        assert_eq!(body.max_tokens, 100);
        assert_eq!(body.temperature, Some(0.7));
        assert_eq!(body.stream, Some(false));
    }

    #[test]
    fn test_build_request_body_streaming() {
        let provider = AnthropicProvider::new("test_key".to_string());
        let request = CompletionRequest {
            model: "claude-3-haiku-20240307".to_string(),
            prompt: "Test".to_string(),
            temperature: Some(0.5),
            max_tokens: None,
            top_p: None,
            stop: None,
            stream: false,
        };

        let body = provider.build_request_body(&request, true);

        assert_eq!(body.stream, Some(true));
        assert_eq!(body.max_tokens, 1024); // Default
    }

    #[test]
    fn test_convert_response() {
        let claude_response = ClaudeResponse {
            id: "msg_123".to_string(),
            response_type: "message".to_string(),
            role: "assistant".to_string(),
            content: vec![ClaudeContent::Text {
                text: "Hello, human!".to_string(),
            }],
            model: "claude-3-sonnet-20240229".to_string(),
            stop_reason: Some("end_turn".to_string()),
            usage: ClaudeUsage {
                input_tokens: 10,
                output_tokens: 5,
            },
        };

        let response = AnthropicProvider::convert_response(claude_response);

        assert_eq!(response.content, "Hello, human!");
        assert_eq!(response.model, "claude-3-sonnet-20240229");
        assert_eq!(response.usage.prompt_tokens, 10);
        assert_eq!(response.usage.completion_tokens, 5);
        assert_eq!(response.usage.total_tokens, 15);
    }

    #[test]
    fn test_convert_response_multiple_content_blocks() {
        let claude_response = ClaudeResponse {
            id: "msg_123".to_string(),
            response_type: "message".to_string(),
            role: "assistant".to_string(),
            content: vec![
                ClaudeContent::Text {
                    text: "Part 1. ".to_string(),
                },
                ClaudeContent::Text {
                    text: "Part 2.".to_string(),
                },
            ],
            model: "claude-3-opus-20240229".to_string(),
            stop_reason: Some("end_turn".to_string()),
            usage: ClaudeUsage {
                input_tokens: 20,
                output_tokens: 10,
            },
        };

        let response = AnthropicProvider::convert_response(claude_response);

        assert_eq!(response.content, "Part 1. Part 2.");
        assert_eq!(response.usage.total_tokens, 30);
    }

    #[test]
    fn test_parse_error_authentication() {
        let error_json = r#"{
            "type": "error",
            "error": {
                "type": "authentication_error",
                "message": "Invalid API key"
            }
        }"#;

        let error = AnthropicProvider::parse_error(401, error_json);

        match error {
            ProviderError::AuthenticationError(msg) => assert_eq!(msg, "Invalid API key"),
            _ => panic!("Expected AuthenticationError"),
        }
    }

    #[test]
    fn test_parse_error_rate_limit() {
        let error_json = r#"{
            "type": "error",
            "error": {
                "type": "rate_limit_error",
                "message": "Rate limit exceeded"
            }
        }"#;

        let error = AnthropicProvider::parse_error(429, error_json);

        assert!(matches!(error, ProviderError::RateLimitExceeded { .. }));
    }

    #[test]
    fn test_parse_error_invalid_request() {
        let error_json = r#"{
            "type": "error",
            "error": {
                "type": "invalid_request_error",
                "message": "Invalid model specified"
            }
        }"#;

        let error = AnthropicProvider::parse_error(400, error_json);

        match error {
            ProviderError::InvalidRequest(msg) => assert_eq!(msg, "Invalid model specified"),
            _ => panic!("Expected InvalidRequest"),
        }
    }

    #[test]
    fn test_is_retryable() {
        assert!(AnthropicProvider::is_retryable(&ProviderError::RateLimitExceeded { retry_after: None }));
        assert!(AnthropicProvider::is_retryable(&ProviderError::InvalidRequest("Network error".to_string())));
        assert!(!AnthropicProvider::is_retryable(&ProviderError::AuthenticationError("Invalid key".to_string())));
        assert!(!AnthropicProvider::is_retryable(&ProviderError::ModelNotFound { model: "model".to_string() }));
    }

    #[test]
    fn test_calculate_backoff() {
        let delay0 = AnthropicProvider::calculate_backoff(0);
        let delay1 = AnthropicProvider::calculate_backoff(1);
        let delay2 = AnthropicProvider::calculate_backoff(2);
        let delay10 = AnthropicProvider::calculate_backoff(10);

        assert_eq!(delay0, Duration::from_millis(1000));
        assert_eq!(delay1, Duration::from_millis(2000));
        assert_eq!(delay2, Duration::from_millis(4000));
        assert_eq!(delay10, Duration::from_millis(60_000)); // Capped at max
    }

    #[test]
    fn test_parse_streaming_event_text_delta() {
        let event_data = r#"{
            "type": "content_block_delta",
            "index": 0,
            "delta": {
                "type": "text_delta",
                "text": "Hello"
            }
        }"#;

        let result = AnthropicProvider::parse_streaming_event(event_data);

        assert!(result.is_some());
        let text = result.unwrap().unwrap();
        assert_eq!(text, "Hello");
    }

    #[test]
    fn test_parse_streaming_event_message_stop() {
        let event_data = r#"{
            "type": "message_stop"
        }"#;

        let result = AnthropicProvider::parse_streaming_event(event_data);

        assert!(result.is_none());
    }

    #[test]
    fn test_with_base_url() {
        let provider = AnthropicProvider::with_base_url(
            "test_key".to_string(),
            "http://localhost:8080".to_string(),
        );

        assert_eq!(provider.base_url, "http://localhost:8080");
        assert_eq!(provider.api_key, "test_key");
    }

    #[test]
    fn test_with_config() {
        let provider = AnthropicProvider::with_config(
            "test_key".to_string(),
            "http://test.com".to_string(),
            5,
        );

        assert_eq!(provider.max_retries, 5);
        assert_eq!(provider.base_url, "http://test.com");
    }
}

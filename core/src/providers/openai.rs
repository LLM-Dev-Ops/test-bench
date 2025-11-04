// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! OpenAI provider implementation

use super::{CompletionRequest, CompletionResponse, FinishReason, ModelInfo, Provider, ProviderError, ResponseStream, TokenUsage};
use async_trait::async_trait;
use futures::stream::{Stream, StreamExt};
use reqwest_eventsource::{Event, EventSource};
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use std::time::Duration;
use tracing::{debug, error, warn};

/// OpenAI provider configuration
#[derive(Debug, Clone)]
pub struct OpenAIConfig {
    /// Maximum number of retries for failed requests
    pub max_retries: u32,
    /// Request timeout
    pub timeout: Duration,
}

impl Default for OpenAIConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            timeout: Duration::from_secs(120),
        }
    }
}

/// OpenAI provider
pub struct OpenAIProvider {
    client: reqwest::Client,
    api_key: String,
    base_url: String,
    config: OpenAIConfig,
}

impl OpenAIProvider {
    /// Create a new OpenAI provider from environment variable
    pub fn new(api_key: String) -> Result<Self, ProviderError> {
        Self::with_config(api_key, "https://api.openai.com/v1".to_string(), OpenAIConfig::default())
    }

    /// Create a new OpenAI provider with custom base URL
    pub fn with_base_url(api_key: String, base_url: String) -> Result<Self, ProviderError> {
        Self::with_config(api_key, base_url, OpenAIConfig::default())
    }

    /// Create a new OpenAI provider with custom configuration
    pub fn with_config(api_key: String, base_url: String, config: OpenAIConfig) -> Result<Self, ProviderError> {
        if api_key.is_empty() {
            return Err(ProviderError::InvalidApiKey);
        }

        let client = reqwest::Client::builder()
            .timeout(config.timeout)
            .pool_max_idle_per_host(10)
            .pool_idle_timeout(Duration::from_secs(90))
            .use_rustls_tls()
            .build()
            .map_err(|e| ProviderError::InternalError(format!("Failed to build HTTP client: {}", e)))?;

        Ok(Self {
            client,
            api_key,
            base_url,
            config,
        })
    }

    /// Build request body for OpenAI API
    fn build_request_body(&self, request: &CompletionRequest, stream: bool) -> serde_json::Value {
        let mut body = serde_json::json!({
            "model": request.model,
            "messages": [
                {
                    "role": "user",
                    "content": request.prompt
                }
            ],
            "stream": stream,
        });

        if let Some(temp) = request.temperature {
            body["temperature"] = serde_json::json!(temp);
        }

        if let Some(max_tokens) = request.max_tokens {
            body["max_tokens"] = serde_json::json!(max_tokens);
        }

        if let Some(top_p) = request.top_p {
            body["top_p"] = serde_json::json!(top_p);
        }

        if let Some(ref stop) = request.stop {
            body["stop"] = serde_json::json!(stop);
        }

        body
    }

    /// Parse OpenAI error response
    fn parse_error_response(status: u16, text: &str) -> ProviderError {
        #[derive(Deserialize)]
        struct ErrorResponse {
            error: ErrorDetail,
        }

        #[derive(Deserialize)]
        struct ErrorDetail {
            message: String,
            #[serde(rename = "type")]
            error_type: Option<String>,
        }

        if let Ok(err_resp) = serde_json::from_str::<ErrorResponse>(text) {
            let error_type = err_resp.error.error_type.as_deref().unwrap_or("");

            match (status, error_type) {
                (401, _) => ProviderError::InvalidApiKey,
                (429, _) => ProviderError::RateLimitExceeded { retry_after: None },
                (_, "context_length_exceeded") => {
                    // Try to extract token counts from message
                    ProviderError::ContextLengthExceeded {
                        tokens: 0,
                        max: 0,
                    }
                }
                (_, "model_not_found") => ProviderError::ModelNotFound {
                    model: "unknown".to_string(),
                },
                _ => ProviderError::ApiError {
                    status,
                    message: err_resp.error.message,
                },
            }
        } else {
            ProviderError::ApiError {
                status,
                message: text.to_string(),
            }
        }
    }

    /// Parse non-streaming response
    fn parse_completion_response(&self, json: &str) -> Result<CompletionResponse, ProviderError> {
        #[derive(Deserialize)]
        struct OpenAIResponse {
            id: String,
            model: String,
            choices: Vec<Choice>,
            usage: Usage,
        }

        #[derive(Deserialize)]
        struct Choice {
            message: Message,
            finish_reason: String,
        }

        #[derive(Deserialize)]
        struct Message {
            content: String,
        }

        #[derive(Deserialize)]
        struct Usage {
            prompt_tokens: u32,
            completion_tokens: u32,
            total_tokens: u32,
        }

        let resp: OpenAIResponse = serde_json::from_str(json)?;

        let choice = resp.choices.first()
            .ok_or_else(|| ProviderError::ApiError { status: 500, message: "No choices in response".to_string() })?;

        let finish_reason = match choice.finish_reason.as_str() {
            "stop" => FinishReason::Stop,
            "length" => FinishReason::Length,
            "content_filter" => FinishReason::ContentFilter,
            "tool_calls" | "function_call" => FinishReason::ToolCalls,
            _ => FinishReason::Error,
        };

        Ok(CompletionResponse {
            id: resp.id,
            content: choice.message.content.clone(),
            model: resp.model,
            usage: TokenUsage {
                prompt_tokens: resp.usage.prompt_tokens as usize,
                completion_tokens: resp.usage.completion_tokens as usize,
                total_tokens: resp.usage.total_tokens as usize,
            },
            finish_reason,
            created_at: chrono::Utc::now(),
        })
    }

    /// Check if an error is retryable
    fn is_retryable(error: &ProviderError) -> bool {
        match error {
            ProviderError::NetworkError(_) => true,
            ProviderError::RateLimitExceeded { .. } => true,
            ProviderError::ApiError { status, .. } if *status >= 500 => true,
            ProviderError::Timeout(_) => true,
            _ => false,
        }
    }

    /// Calculate exponential backoff delay
    /// Formula: base_delay * 2^attempt, capped at 60 seconds
    fn calculate_backoff(attempt: u32) -> Duration {
        const BASE_DELAY_MS: u64 = 1000; // 1 second
        const MAX_DELAY_MS: u64 = 60000; // 60 seconds

        let delay_ms = BASE_DELAY_MS * 2_u64.pow(attempt);
        let delay_ms = delay_ms.min(MAX_DELAY_MS);
        Duration::from_millis(delay_ms)
    }

    /// Make a completion request with a single attempt (no retry)
    async fn complete_once(&self, request: &CompletionRequest) -> Result<CompletionResponse, ProviderError> {
        debug!("OpenAI completion request: model={}, prompt_len={}", request.model, request.prompt.len());

        let url = format!("{}/chat/completions", self.base_url);
        let body = self.build_request_body(request, false);

        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        let status = response.status();
        let text = response.text().await?;

        if !status.is_success() {
            error!("OpenAI API error: status={}, response={}", status, text);
            return Err(Self::parse_error_response(status.as_u16(), &text));
        }

        debug!("OpenAI completion response received, parsing...");
        self.parse_completion_response(&text)
    }

    /// Make a completion request with retry logic
    async fn complete_with_retry(&self, request: &CompletionRequest) -> Result<CompletionResponse, ProviderError> {
        let mut last_error = None;

        for attempt in 0..=self.config.max_retries {
            match self.complete_once(request).await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    if !Self::is_retryable(&e) || attempt >= self.config.max_retries {
                        return Err(e);
                    }

                    warn!(
                        "Request failed (attempt {}/{}): {}. Retrying...",
                        attempt + 1,
                        self.config.max_retries + 1,
                        e
                    );

                    last_error = Some(e);
                    let delay = Self::calculate_backoff(attempt);
                    tokio::time::sleep(delay).await;
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            ProviderError::InternalError("Retry loop completed without error".to_string())
        }))
    }
}

#[async_trait]
impl Provider for OpenAIProvider {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse, ProviderError> {
        self.complete_with_retry(&request).await
    }

    async fn stream(&self, request: CompletionRequest) -> Result<ResponseStream, ProviderError> {
        debug!("OpenAI streaming request: model={}, prompt_len={}", request.model, request.prompt.len());

        let url = format!("{}/chat/completions", self.base_url);
        let body = self.build_request_body(&request, true);

        let req_builder = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&body);

        let event_source = EventSource::new(req_builder)
            .map_err(|e| ProviderError::InternalError(format!("Failed to create event source: {}", e)))?;

        let stream = event_source.filter_map(move |event| async move {
            match event {
                Ok(Event::Message(msg)) => {
                    if msg.data == "[DONE]" {
                        return None;
                    }

                    #[derive(Deserialize)]
                    struct StreamResponse {
                        choices: Vec<StreamChoice>,
                    }

                    #[derive(Deserialize)]
                    struct StreamChoice {
                        delta: Delta,
                        finish_reason: Option<String>,
                    }

                    #[derive(Deserialize)]
                    struct Delta {
                        content: Option<String>,
                    }

                    match serde_json::from_str::<StreamResponse>(&msg.data) {
                        Ok(resp) => {
                            if let Some(choice) = resp.choices.first() {
                                let content = choice.delta.content.clone().unwrap_or_default();
                                if !content.is_empty() {
                                    Some(Ok(content))
                                } else {
                                    None
                                }
                            } else {
                                Some(Err(ProviderError::InternalError("No choices in stream chunk".to_string())))
                            }
                        }
                        Err(e) => {
                            error!("Failed to parse stream chunk: {}", e);
                            Some(Err(ProviderError::ParseError(e)))
                        }
                    }
                }
                Ok(Event::Open) => {
                    debug!("Stream opened");
                    None
                }
                Err(e) => {
                    error!("Stream error: {}", e);
                    Some(Err(ProviderError::InternalError(format!("{}", e))))
                }
            }
        });

        Ok(Box::pin(stream))
    }

    fn supported_models(&self) -> Vec<ModelInfo> {
        vec![
            ModelInfo {
                id: "gpt-4".to_string(),
                name: "GPT-4".to_string(),
                max_tokens: 8192,
                supports_streaming: true,
                supports_function_calling: true,
            },
            ModelInfo {
                id: "gpt-4-turbo".to_string(),
                name: "GPT-4 Turbo".to_string(),
                max_tokens: 128000,
                supports_streaming: true,
                supports_function_calling: true,
            },
            ModelInfo {
                id: "gpt-4-turbo-preview".to_string(),
                name: "GPT-4 Turbo Preview".to_string(),
                max_tokens: 128000,
                supports_streaming: true,
                supports_function_calling: true,
            },
            ModelInfo {
                id: "gpt-3.5-turbo".to_string(),
                name: "GPT-3.5 Turbo".to_string(),
                max_tokens: 16385,
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
        "OpenAI"
    }

    async fn validate_config(&self) -> Result<(), ProviderError> {
        if self.api_key.is_empty() {
            return Err(ProviderError::InvalidApiKey);
        }
        Ok(())
    }

    fn estimate_tokens(&self, text: &str, _model: &str) -> Result<usize, ProviderError> {
        // Simple estimation: ~4 characters per token
        Ok(text.len() / 4)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openai_provider_creation() {
        let provider = OpenAIProvider::new("test_key".to_string()).unwrap();
        assert_eq!(provider.name(), "OpenAI");
        assert_eq!(provider.supported_models().len(), 4);
    }

    #[test]
    fn test_max_context_length() {
        let provider = OpenAIProvider::new("test_key".to_string()).unwrap();
        assert_eq!(provider.max_context_length("gpt-4"), Some(8192));
        assert_eq!(provider.max_context_length("gpt-4-turbo"), Some(128000));
        assert_eq!(provider.max_context_length("unknown"), None);
    }

    #[test]
    fn test_build_request_body() {
        let provider = OpenAIProvider::new("test_key".to_string()).unwrap();
        let request = CompletionRequest {
            model: "gpt-4".to_string(),
            prompt: "Hello".to_string(),
            temperature: Some(0.7),
            max_tokens: Some(100),
            top_p: Some(0.9),
            stop: Some(vec!["STOP".to_string()]),
            stream: false,
        };

        let body = provider.build_request_body(&request, false);
        assert_eq!(body["model"], "gpt-4");
        assert_eq!(body["messages"][0]["content"], "Hello");
        assert_eq!(body["temperature"], 0.7);
        assert_eq!(body["max_tokens"], 100);
        assert_eq!(body["stream"], false);
    }

    #[test]
    fn test_is_retryable() {
        assert!(OpenAIProvider::is_retryable(&ProviderError::RateLimitExceeded {
            retry_after: None
        }));
        assert!(OpenAIProvider::is_retryable(&ProviderError::ApiError {
            status: 500,
            message: "Internal server error".to_string()
        }));
        assert!(OpenAIProvider::is_retryable(&ProviderError::ApiError {
            status: 503,
            message: "Service unavailable".to_string()
        }));
        assert!(OpenAIProvider::is_retryable(&ProviderError::Timeout(
            Duration::from_secs(30)
        )));
        assert!(!OpenAIProvider::is_retryable(&ProviderError::InvalidApiKey));
        assert!(!OpenAIProvider::is_retryable(&ProviderError::ApiError {
            status: 400,
            message: "Bad request".to_string()
        }));
        assert!(!OpenAIProvider::is_retryable(&ProviderError::ApiError {
            status: 404,
            message: "Not found".to_string()
        }));
    }

    #[test]
    fn test_calculate_backoff() {
        assert_eq!(
            OpenAIProvider::calculate_backoff(0),
            Duration::from_millis(1000)
        );
        assert_eq!(
            OpenAIProvider::calculate_backoff(1),
            Duration::from_millis(2000)
        );
        assert_eq!(
            OpenAIProvider::calculate_backoff(2),
            Duration::from_millis(4000)
        );
        assert_eq!(
            OpenAIProvider::calculate_backoff(3),
            Duration::from_millis(8000)
        );
        assert_eq!(
            OpenAIProvider::calculate_backoff(4),
            Duration::from_millis(16000)
        );
        // Should cap at 60 seconds
        assert_eq!(
            OpenAIProvider::calculate_backoff(10),
            Duration::from_millis(60000)
        );
    }

    #[test]
    fn test_parse_error_response() {
        // Test 401 error
        let error_json = r#"{"error":{"message":"Invalid API key","type":"invalid_request_error"}}"#;
        let error = OpenAIProvider::parse_error_response(401, error_json);
        assert!(matches!(error, ProviderError::InvalidApiKey));

        // Test 429 error
        let error_json = r#"{"error":{"message":"Rate limit exceeded","type":"rate_limit_error"}}"#;
        let error = OpenAIProvider::parse_error_response(429, error_json);
        assert!(matches!(error, ProviderError::RateLimitExceeded { .. }));

        // Test model not found
        let error_json = r#"{"error":{"message":"Model not found","type":"model_not_found"}}"#;
        let error = OpenAIProvider::parse_error_response(404, error_json);
        assert!(matches!(error, ProviderError::ModelNotFound { .. }));

        // Test generic error
        let error_json = r#"{"error":{"message":"Something went wrong","type":"server_error"}}"#;
        let error = OpenAIProvider::parse_error_response(500, error_json);
        assert!(matches!(error, ProviderError::ApiError { status: 500, .. }));
    }

    #[test]
    fn test_estimate_tokens() {
        let provider = OpenAIProvider::new("test_key".to_string()).unwrap();

        // "Hello, world!" is ~13 chars, should be ~4 tokens
        let count = provider.estimate_tokens("Hello, world!", "gpt-4").unwrap();
        assert_eq!(count, 4);

        // Empty string
        let count = provider.estimate_tokens("", "gpt-4").unwrap();
        assert_eq!(count, 0);

        // Longer text
        let long_text = "a".repeat(400); // 400 chars = ~100 tokens
        let count = provider.estimate_tokens(&long_text, "gpt-4").unwrap();
        assert_eq!(count, 100);
    }

    #[test]
    fn test_provider_config_creation() {
        let config = OpenAIConfig::default();
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.timeout, Duration::from_secs(120));

        let custom_config = OpenAIConfig {
            max_retries: 5,
            timeout: Duration::from_secs(60),
        };
        assert_eq!(custom_config.max_retries, 5);
        assert_eq!(custom_config.timeout, Duration::from_secs(60));
    }

    #[test]
    fn test_empty_api_key() {
        let result = OpenAIProvider::new("".to_string());
        assert!(matches!(result, Err(ProviderError::InvalidApiKey)));
    }
}

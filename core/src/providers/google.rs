// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Google AI (Gemini) provider implementation

use super::{CompletionRequest, CompletionResponse, FinishReason, ModelInfo, Provider, ProviderError, ResponseStream, TokenUsage};
use async_trait::async_trait;
use futures::stream::{Stream, StreamExt};
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use std::time::Duration;
use tracing::{debug, error, warn};

/// Google AI provider configuration
#[derive(Debug, Clone)]
pub struct GoogleConfig {
    /// Maximum number of retries for failed requests
    pub max_retries: u32,
    /// Request timeout
    pub timeout: Duration,
}

impl Default for GoogleConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            timeout: Duration::from_secs(120),
        }
    }
}

/// Google AI provider for Gemini models
pub struct GoogleProvider {
    client: reqwest::Client,
    api_key: String,
    base_url: String,
    config: GoogleConfig,
}

impl GoogleProvider {
    /// Create a new Google AI provider
    pub fn new(api_key: String) -> Result<Self, ProviderError> {
        Self::with_config(
            api_key,
            "https://generativelanguage.googleapis.com/v1beta".to_string(),
            GoogleConfig::default(),
        )
    }

    /// Create a new Google AI provider with custom base URL
    pub fn with_base_url(api_key: String, base_url: String) -> Result<Self, ProviderError> {
        Self::with_config(api_key, base_url, GoogleConfig::default())
    }

    /// Create a new Google AI provider with custom configuration
    pub fn with_config(api_key: String, base_url: String, config: GoogleConfig) -> Result<Self, ProviderError> {
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

    /// Build request body for Google AI API
    fn build_request_body(&self, request: &CompletionRequest) -> serde_json::Value {
        let mut body = serde_json::json!({
            "contents": [
                {
                    "parts": [
                        {
                            "text": request.prompt
                        }
                    ]
                }
            ]
        });

        // Build generation config
        let mut generation_config = serde_json::Map::new();

        if let Some(temp) = request.temperature {
            generation_config.insert("temperature".to_string(), serde_json::json!(temp));
        }

        if let Some(max_tokens) = request.max_tokens {
            generation_config.insert("maxOutputTokens".to_string(), serde_json::json!(max_tokens));
        }

        if let Some(top_p) = request.top_p {
            generation_config.insert("topP".to_string(), serde_json::json!(top_p));
        }

        if let Some(ref stop) = request.stop {
            generation_config.insert("stopSequences".to_string(), serde_json::json!(stop));
        }

        if !generation_config.is_empty() {
            body["generationConfig"] = serde_json::Value::Object(generation_config);
        }

        body
    }

    /// Parse Google AI error response
    fn parse_error_response(status: u16, text: &str) -> ProviderError {
        #[derive(Deserialize)]
        struct ErrorResponse {
            error: ErrorDetail,
        }

        #[derive(Deserialize)]
        struct ErrorDetail {
            message: String,
            code: Option<i32>,
            status: Option<String>,
        }

        if let Ok(err_resp) = serde_json::from_str::<ErrorResponse>(text) {
            match (status, err_resp.error.code) {
                (401, _) | (403, _) => ProviderError::InvalidApiKey,
                (429, _) => ProviderError::RateLimitExceeded { retry_after: None },
                (400, Some(code)) if err_resp.error.message.contains("token") => {
                    ProviderError::ContextLengthExceeded {
                        tokens: 0,
                        max: 0,
                    }
                }
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
        struct GoogleResponse {
            candidates: Vec<Candidate>,
            #[serde(rename = "usageMetadata")]
            usage_metadata: Option<UsageMetadata>,
        }

        #[derive(Deserialize)]
        struct Candidate {
            content: Content,
            #[serde(rename = "finishReason")]
            finish_reason: Option<String>,
        }

        #[derive(Deserialize)]
        struct Content {
            parts: Vec<Part>,
        }

        #[derive(Deserialize)]
        struct Part {
            text: String,
        }

        #[derive(Deserialize)]
        struct UsageMetadata {
            #[serde(rename = "promptTokenCount")]
            prompt_token_count: Option<u32>,
            #[serde(rename = "candidatesTokenCount")]
            candidates_token_count: Option<u32>,
            #[serde(rename = "totalTokenCount")]
            total_token_count: Option<u32>,
        }

        let resp: GoogleResponse = serde_json::from_str(json)
            .map_err(|e| ProviderError::InternalError(format!("Failed to parse response: {}", e)))?;

        let candidate = resp.candidates.first()
            .ok_or_else(|| ProviderError::ApiError {
                status: 500,
                message: "No candidates in response".to_string()
            })?;

        let content = candidate.content.parts.iter()
            .map(|p| p.text.as_str())
            .collect::<Vec<_>>()
            .join("");

        let finish_reason = match candidate.finish_reason.as_deref() {
            Some("STOP") | Some("FINISH_REASON_STOP") => FinishReason::Stop,
            Some("MAX_TOKENS") | Some("FINISH_REASON_MAX_TOKENS") => FinishReason::Length,
            Some("SAFETY") | Some("FINISH_REASON_SAFETY") => FinishReason::ContentFilter,
            _ => FinishReason::Stop,
        };

        // Extract token usage if available
        let usage = if let Some(metadata) = resp.usage_metadata {
            TokenUsage {
                prompt_tokens: metadata.prompt_token_count.unwrap_or(0) as usize,
                completion_tokens: metadata.candidates_token_count.unwrap_or(0) as usize,
                total_tokens: metadata.total_token_count.unwrap_or(0) as usize,
            }
        } else {
            // Estimate tokens if not provided
            let prompt_tokens = estimate_tokens(&content);
            let completion_tokens = estimate_tokens(&content);
            TokenUsage {
                prompt_tokens,
                completion_tokens,
                total_tokens: prompt_tokens + completion_tokens,
            }
        };

        Ok(CompletionResponse {
            id: format!("gemini-{}", chrono::Utc::now().timestamp()),
            content,
            model: "gemini".to_string(),
            usage,
            finish_reason,
            created_at: chrono::Utc::now(),
        })
    }
}

#[async_trait]
impl Provider for GoogleProvider {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse, ProviderError> {
        let url = format!(
            "{}/models/{}:generateContent?key={}",
            self.base_url, request.model, self.api_key
        );

        let body = self.build_request_body(&request);

        debug!("Sending request to Google AI: {}", url);

        let response = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| ProviderError::NetworkError(e.to_string()))?;

        let status = response.status().as_u16();

        if !response.status().is_success() {
            let text = response.text().await
                .unwrap_or_else(|_| "Failed to read error response".to_string());
            error!("Google AI API error ({}): {}", status, text);
            return Err(Self::parse_error_response(status, &text));
        }

        let text = response.text().await
            .map_err(|e| ProviderError::NetworkError(e.to_string()))?;

        self.parse_completion_response(&text)
    }

    async fn stream(&self, request: CompletionRequest) -> Result<ResponseStream, ProviderError> {
        let url = format!(
            "{}/models/{}:streamGenerateContent?key={}&alt=sse",
            self.base_url, request.model, self.api_key
        );

        let body = self.build_request_body(&request);

        debug!("Starting stream from Google AI: {}", url);

        let response = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| ProviderError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let text = response.text().await
                .unwrap_or_else(|_| "Failed to read error response".to_string());
            error!("Google AI streaming error ({}): {}", status, text);
            return Err(Self::parse_error_response(status, &text));
        }

        let stream = response.bytes_stream();

        let text_stream = stream.map(move |result| {
            match result {
                Ok(bytes) => {
                    let text = String::from_utf8_lossy(&bytes);
                    // Parse SSE format
                    for line in text.lines() {
                        if line.starts_with("data: ") {
                            let json_str = line.strip_prefix("data: ").unwrap_or("");
                            if json_str.trim().is_empty() || json_str == "[DONE]" {
                                continue;
                            }

                            // Extract text from response
                            if let Ok(json) = serde_json::from_str::<serde_json::Value>(json_str) {
                                if let Some(candidates) = json.get("candidates").and_then(|c| c.as_array()) {
                                    if let Some(candidate) = candidates.first() {
                                        if let Some(parts) = candidate.get("content")
                                            .and_then(|c| c.get("parts"))
                                            .and_then(|p| p.as_array()) {
                                            if let Some(text) = parts.first().and_then(|p| p.get("text")).and_then(|t| t.as_str()) {
                                                return Ok(text.to_string());
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Ok(String::new())
                }
                Err(e) => Err(ProviderError::NetworkError(e.to_string())),
            }
        });

        Ok(Box::pin(text_stream))
    }

    fn supported_models(&self) -> Vec<ModelInfo> {
        vec![
            ModelInfo::new("gemini-pro", "Gemini Pro", 30720, true, true),
            ModelInfo::new("gemini-pro-vision", "Gemini Pro Vision", 30720, true, false),
            ModelInfo::new("gemini-1.5-pro", "Gemini 1.5 Pro", 1048576, true, true),
            ModelInfo::new("gemini-1.5-flash", "Gemini 1.5 Flash", 1048576, true, true),
            ModelInfo::new("gemini-ultra", "Gemini Ultra", 30720, true, true),
        ]
    }

    fn max_context_length(&self, model: &str) -> Option<usize> {
        match model {
            "gemini-pro" | "gemini-pro-vision" | "gemini-ultra" => Some(30720),
            "gemini-1.5-pro" | "gemini-1.5-flash" => Some(1048576),
            _ => None,
        }
    }

    fn name(&self) -> &str {
        "google"
    }

    async fn validate_config(&self) -> Result<(), ProviderError> {
        if self.api_key.is_empty() {
            return Err(ProviderError::InvalidApiKey);
        }

        // Try a simple request to validate the API key
        let request = CompletionRequest::new("gemini-pro", "Hello")
            .with_max_tokens(5);

        match self.complete(request).await {
            Ok(_) => Ok(()),
            Err(ProviderError::InvalidApiKey) => Err(ProviderError::InvalidApiKey),
            Err(e) => {
                warn!("Config validation encountered error: {}", e);
                Ok(()) // Don't fail on other errors during validation
            }
        }
    }

    fn estimate_tokens(&self, text: &str, _model: &str) -> Result<usize, ProviderError> {
        Ok(estimate_tokens(text))
    }
}

/// Simple token estimation (roughly 4 chars per token)
fn estimate_tokens(text: &str) -> usize {
    (text.len() / 4).max(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_creation() {
        let result = GoogleProvider::new("test-key".to_string());
        assert!(result.is_ok());
    }

    #[test]
    fn test_provider_creation_empty_key() {
        let result = GoogleProvider::new(String::new());
        assert!(matches!(result, Err(ProviderError::InvalidApiKey)));
    }

    #[test]
    fn test_supported_models() {
        let provider = GoogleProvider::new("test-key".to_string()).unwrap();
        let models = provider.supported_models();
        assert!(!models.is_empty());
        assert!(models.iter().any(|m| m.id == "gemini-pro"));
        assert!(models.iter().any(|m| m.id == "gemini-1.5-pro"));
    }

    #[test]
    fn test_max_context_length() {
        let provider = GoogleProvider::new("test-key".to_string()).unwrap();
        assert_eq!(provider.max_context_length("gemini-pro"), Some(30720));
        assert_eq!(provider.max_context_length("gemini-1.5-pro"), Some(1048576));
        assert_eq!(provider.max_context_length("unknown-model"), None);
    }

    #[test]
    fn test_estimate_tokens() {
        let provider = GoogleProvider::new("test-key".to_string()).unwrap();
        let tokens = provider.estimate_tokens("Hello, world!", "gemini-pro").unwrap();
        assert!(tokens > 0);
    }

    #[test]
    fn test_provider_name() {
        let provider = GoogleProvider::new("test-key".to_string()).unwrap();
        assert_eq!(provider.name(), "google");
    }

    #[test]
    fn test_build_request_body() {
        let provider = GoogleProvider::new("test-key".to_string()).unwrap();
        let request = CompletionRequest::new("gemini-pro", "Test prompt")
            .with_temperature(0.7)
            .with_max_tokens(100);

        let body = provider.build_request_body(&request);

        assert!(body.get("contents").is_some());
        assert!(body.get("generationConfig").is_some());
    }
}

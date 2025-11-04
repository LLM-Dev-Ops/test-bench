// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Cohere provider implementation

use super::{CompletionRequest, CompletionResponse, FinishReason, ModelInfo, Provider, ProviderError, ResponseStream, TokenUsage};
use async_trait::async_trait;
use futures::stream::{Stream, StreamExt};
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use std::time::Duration;
use tracing::{debug, error, warn};

/// Cohere provider configuration
#[derive(Debug, Clone)]
pub struct CohereConfig {
    /// Maximum number of retries for failed requests
    pub max_retries: u32,
    /// Request timeout
    pub timeout: Duration,
}

impl Default for CohereConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            timeout: Duration::from_secs(120),
        }
    }
}

/// Cohere provider for Command models
pub struct CohereProvider {
    client: reqwest::Client,
    api_key: String,
    base_url: String,
    config: CohereConfig,
}

impl CohereProvider {
    /// Create a new Cohere provider
    pub fn new(api_key: String) -> Result<Self, ProviderError> {
        Self::with_config(
            api_key,
            "https://api.cohere.ai/v1".to_string(),
            CohereConfig::default(),
        )
    }

    /// Create a new Cohere provider with custom base URL
    pub fn with_base_url(api_key: String, base_url: String) -> Result<Self, ProviderError> {
        Self::with_config(api_key, base_url, CohereConfig::default())
    }

    /// Create a new Cohere provider with custom configuration
    pub fn with_config(api_key: String, base_url: String, config: CohereConfig) -> Result<Self, ProviderError> {
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

    /// Build request body for Cohere API
    fn build_request_body(&self, request: &CompletionRequest, stream: bool) -> serde_json::Value {
        let mut body = serde_json::json!({
            "model": request.model,
            "message": request.prompt,
            "stream": stream,
        });

        if let Some(temp) = request.temperature {
            body["temperature"] = serde_json::json!(temp);
        }

        if let Some(max_tokens) = request.max_tokens {
            body["max_tokens"] = serde_json::json!(max_tokens);
        }

        if let Some(top_p) = request.top_p {
            body["p"] = serde_json::json!(top_p);
        }

        if let Some(ref stop) = request.stop {
            body["stop_sequences"] = serde_json::json!(stop);
        }

        body
    }

    /// Parse Cohere error response
    fn parse_error_response(status: u16, text: &str) -> ProviderError {
        #[derive(Deserialize)]
        struct ErrorResponse {
            message: String,
        }

        if let Ok(err_resp) = serde_json::from_str::<ErrorResponse>(text) {
            match status {
                401 => ProviderError::InvalidApiKey,
                429 => ProviderError::RateLimitExceeded { retry_after: None },
                _ => ProviderError::ApiError {
                    status,
                    message: err_resp.message,
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
        struct CohereResponse {
            id: Option<String>,
            text: String,
            meta: Option<Meta>,
            finish_reason: Option<String>,
        }

        #[derive(Deserialize)]
        struct Meta {
            billed_units: Option<BilledUnits>,
        }

        #[derive(Deserialize)]
        struct BilledUnits {
            input_tokens: Option<u32>,
            output_tokens: Option<u32>,
        }

        let resp: CohereResponse = serde_json::from_str(json)
            .map_err(|e| ProviderError::InternalError(format!("Failed to parse response: {}", e)))?;

        let finish_reason = match resp.finish_reason.as_deref() {
            Some("COMPLETE") => FinishReason::Stop,
            Some("MAX_TOKENS") => FinishReason::Length,
            Some("ERROR") => FinishReason::Error,
            _ => FinishReason::Stop,
        };

        // Extract token usage from metadata
        let usage = if let Some(meta) = resp.meta {
            if let Some(billed) = meta.billed_units {
                let prompt_tokens = billed.input_tokens.unwrap_or(0) as usize;
                let completion_tokens = billed.output_tokens.unwrap_or(0) as usize;
                TokenUsage {
                    prompt_tokens,
                    completion_tokens,
                    total_tokens: prompt_tokens + completion_tokens,
                }
            } else {
                TokenUsage {
                    prompt_tokens: 0,
                    completion_tokens: 0,
                    total_tokens: 0,
                }
            }
        } else {
            TokenUsage {
                prompt_tokens: 0,
                completion_tokens: 0,
                total_tokens: 0,
            }
        };

        Ok(CompletionResponse {
            id: resp.id.unwrap_or_else(|| format!("cohere-{}", chrono::Utc::now().timestamp())),
            content: resp.text,
            model: "command".to_string(),
            usage,
            finish_reason,
            created_at: chrono::Utc::now(),
        })
    }
}

#[async_trait]
impl Provider for CohereProvider {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse, ProviderError> {
        let url = format!("{}/chat", self.base_url);

        let body = self.build_request_body(&request, false);

        debug!("Sending request to Cohere: {}", url);

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| ProviderError::NetworkError(e.to_string()))?;

        let status = response.status().as_u16();

        if !response.status().is_success() {
            let text = response.text().await
                .unwrap_or_else(|_| "Failed to read error response".to_string());
            error!("Cohere API error ({}): {}", status, text);
            return Err(Self::parse_error_response(status, &text));
        }

        let text = response.text().await
            .map_err(|e| ProviderError::NetworkError(e.to_string()))?;

        self.parse_completion_response(&text)
    }

    async fn stream(&self, request: CompletionRequest) -> Result<ResponseStream, ProviderError> {
        let url = format!("{}/chat", self.base_url);

        let body = self.build_request_body(&request, true);

        debug!("Starting stream from Cohere: {}", url);

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .header("Accept", "text/event-stream")
            .json(&body)
            .send()
            .await
            .map_err(|e| ProviderError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let text = response.text().await
                .unwrap_or_else(|_| "Failed to read error response".to_string());
            error!("Cohere streaming error ({}): {}", status, text);
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
                            if json_str.trim().is_empty() {
                                continue;
                            }

                            // Extract text from response
                            if let Ok(json) = serde_json::from_str::<serde_json::Value>(json_str) {
                                if let Some(text) = json.get("text").and_then(|t| t.as_str()) {
                                    return Ok(text.to_string());
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
            ModelInfo::new("command", "Command", 4096, true, false),
            ModelInfo::new("command-light", "Command Light", 4096, true, false),
            ModelInfo::new("command-nightly", "Command Nightly", 4096, true, false),
            ModelInfo::new("command-r", "Command R", 128000, true, true),
            ModelInfo::new("command-r-plus", "Command R+", 128000, true, true),
        ]
    }

    fn max_context_length(&self, model: &str) -> Option<usize> {
        match model {
            "command" | "command-light" | "command-nightly" => Some(4096),
            "command-r" | "command-r-plus" => Some(128000),
            _ => None,
        }
    }

    fn name(&self) -> &str {
        "cohere"
    }

    async fn validate_config(&self) -> Result<(), ProviderError> {
        if self.api_key.is_empty() {
            return Err(ProviderError::InvalidApiKey);
        }

        // Try a simple request to validate the API key
        let request = CompletionRequest::new("command-light", "Hello")
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
        // Cohere uses roughly 4 chars per token
        Ok((text.len() / 4).max(1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_creation() {
        let result = CohereProvider::new("test-key".to_string());
        assert!(result.is_ok());
    }

    #[test]
    fn test_provider_creation_empty_key() {
        let result = CohereProvider::new(String::new());
        assert!(matches!(result, Err(ProviderError::InvalidApiKey)));
    }

    #[test]
    fn test_supported_models() {
        let provider = CohereProvider::new("test-key".to_string()).unwrap();
        let models = provider.supported_models();
        assert!(!models.is_empty());
        assert!(models.iter().any(|m| m.id == "command"));
        assert!(models.iter().any(|m| m.id == "command-r"));
        assert!(models.iter().any(|m| m.id == "command-r-plus"));
    }

    #[test]
    fn test_max_context_length() {
        let provider = CohereProvider::new("test-key".to_string()).unwrap();
        assert_eq!(provider.max_context_length("command"), Some(4096));
        assert_eq!(provider.max_context_length("command-r"), Some(128000));
        assert_eq!(provider.max_context_length("unknown-model"), None);
    }

    #[test]
    fn test_estimate_tokens() {
        let provider = CohereProvider::new("test-key".to_string()).unwrap();
        let tokens = provider.estimate_tokens("Hello, world!", "command").unwrap();
        assert!(tokens > 0);
    }

    #[test]
    fn test_provider_name() {
        let provider = CohereProvider::new("test-key".to_string()).unwrap();
        assert_eq!(provider.name(), "cohere");
    }

    #[test]
    fn test_build_request_body() {
        let provider = CohereProvider::new("test-key".to_string()).unwrap();
        let request = CompletionRequest::new("command", "Test prompt")
            .with_temperature(0.7)
            .with_max_tokens(100);

        let body = provider.build_request_body(&request, false);

        assert!(body.get("model").is_some());
        assert!(body.get("message").is_some());
        assert_eq!(body.get("stream"), Some(&serde_json::json!(false)));
    }
}

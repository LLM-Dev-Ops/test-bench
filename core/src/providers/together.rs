// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Together AI provider implementation (OpenAI-compatible)

use super::{CompletionRequest, CompletionResponse, FinishReason, ModelInfo, Provider, ProviderError, ResponseStream, TokenUsage};
use async_trait::async_trait;
use futures::stream::StreamExt;
use serde::Deserialize;
use std::time::Duration;
use tracing::{debug, error};

/// Together AI provider
pub struct TogetherProvider {
    client: reqwest::Client,
    api_key: String,
    base_url: String,
}

impl TogetherProvider {
    pub fn new(api_key: String) -> Result<Self, ProviderError> {
        Self::with_base_url(api_key, "https://api.together.xyz/v1".to_string())
    }

    pub fn with_base_url(api_key: String, base_url: String) -> Result<Self, ProviderError> {
        if api_key.is_empty() {
            return Err(ProviderError::InvalidApiKey);
        }

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(120))
            .use_rustls_tls()
            .build()
            .map_err(|e| ProviderError::InternalError(format!("Failed to build HTTP client: {}", e)))?;

        Ok(Self { client, api_key, base_url })
    }

    fn build_request_body(&self, request: &CompletionRequest, stream: bool) -> serde_json::Value {
        let mut body = serde_json::json!({
            "model": request.model,
            "messages": [{"role": "user", "content": request.prompt}],
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

        body
    }
}

#[async_trait]
impl Provider for TogetherProvider {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse, ProviderError> {
        let url = format!("{}/chat/completions", self.base_url);
        let body = self.build_request_body(&request, false);

        debug!("Sending request to Together AI");

        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&body)
            .send()
            .await
            .map_err(|e| ProviderError::NetworkError(e))?;

        let status = response.status().as_u16();
        if !response.status().is_success() {
            let text = response.text().await.unwrap_or_default();
            error!("Together AI API error ({}): {}", status, text);
            return Err(ProviderError::ApiError { status, message: text });
        }

        let text = response.text().await.map_err(|e| ProviderError::NetworkError(e))?;

        #[derive(Deserialize)]
        struct TogetherResponse {
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

        let resp: TogetherResponse = serde_json::from_str(&text)?;
        let choice = resp.choices.first()
            .ok_or_else(|| ProviderError::ApiError { status: 500, message: "No choices".to_string() })?;

        Ok(CompletionResponse {
            id: resp.id,
            content: choice.message.content.clone(),
            model: resp.model,
            usage: TokenUsage {
                prompt_tokens: resp.usage.prompt_tokens as usize,
                completion_tokens: resp.usage.completion_tokens as usize,
                total_tokens: resp.usage.total_tokens as usize,
            },
            finish_reason: match choice.finish_reason.as_str() {
                "stop" | "eos" => FinishReason::Stop,
                "length" => FinishReason::Length,
                _ => FinishReason::Stop,
            },
            created_at: chrono::Utc::now(),
        })
    }

    async fn stream(&self, request: CompletionRequest) -> Result<ResponseStream, ProviderError> {
        let url = format!("{}/chat/completions", self.base_url);
        let body = self.build_request_body(&request, true);

        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&body)
            .send()
            .await
            .map_err(|e| ProviderError::NetworkError(e))?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let text = response.text().await.unwrap_or_default();
            return Err(ProviderError::ApiError { status, message: text });
        }

        let stream = response.bytes_stream().map(move |result| {
            match result {
                Ok(bytes) => {
                    let text = String::from_utf8_lossy(&bytes);
                    for line in text.lines() {
                        if line.starts_with("data: ") {
                            let json_str = line.strip_prefix("data: ").unwrap_or("");
                            if json_str == "[DONE]" {
                                continue;
                            }
                            if let Ok(json) = serde_json::from_str::<serde_json::Value>(json_str) {
                                if let Some(content) = json["choices"][0]["delta"]["content"].as_str() {
                                    return Ok(content.to_string());
                                }
                            }
                        }
                    }
                    Ok(String::new())
                }
                Err(e) => Err(ProviderError::NetworkError(e)),
            }
        });

        Ok(Box::pin(stream))
    }

    fn supported_models(&self) -> Vec<ModelInfo> {
        vec![
            ModelInfo::new("meta-llama/Llama-2-70b-chat-hf", "Llama 2 70B Chat", 4096, true, false),
            ModelInfo::new("meta-llama/Llama-3-70b-chat-hf", "Llama 3 70B Chat", 8192, true, false),
            ModelInfo::new("mistralai/Mixtral-8x7B-Instruct-v0.1", "Mixtral 8x7B Instruct", 32768, true, false),
            ModelInfo::new("mistralai/Mistral-7B-Instruct-v0.2", "Mistral 7B Instruct", 32768, true, false),
            ModelInfo::new("togethercomputer/CodeLlama-34b-Instruct", "Code Llama 34B", 16384, true, false),
        ]
    }

    fn max_context_length(&self, model: &str) -> Option<usize> {
        if model.contains("Llama-2") {
            Some(4096)
        } else if model.contains("Llama-3") {
            Some(8192)
        } else if model.contains("Mixtral") || model.contains("Mistral") {
            Some(32768)
        } else if model.contains("CodeLlama") {
            Some(16384)
        } else {
            Some(4096)
        }
    }

    fn name(&self) -> &str {
        "together"
    }

    async fn validate_config(&self) -> Result<(), ProviderError> {
        if self.api_key.is_empty() {
            return Err(ProviderError::InvalidApiKey);
        }
        Ok(())
    }

    fn estimate_tokens(&self, text: &str, _model: &str) -> Result<usize, ProviderError> {
        Ok((text.len() / 4).max(1))
    }
}

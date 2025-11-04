// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Azure OpenAI provider implementation

use super::{CompletionRequest, CompletionResponse, FinishReason, ModelInfo, Provider, ProviderError, ResponseStream, TokenUsage};
use async_trait::async_trait;
use futures::stream::StreamExt;
use serde::Deserialize;
use std::time::Duration;
use tracing::{debug, error};

/// Azure OpenAI provider
pub struct AzureOpenAIProvider {
    client: reqwest::Client,
    api_key: String,
    endpoint: String,
    deployment: String,
    api_version: String,
}

impl AzureOpenAIProvider {
    pub fn new(api_key: String, endpoint: String, deployment: String) -> Result<Self, ProviderError> {
        Self::with_api_version(api_key, endpoint, deployment, "2024-02-15-preview".to_string())
    }

    pub fn with_api_version(
        api_key: String,
        endpoint: String,
        deployment: String,
        api_version: String,
    ) -> Result<Self, ProviderError> {
        if api_key.is_empty() {
            return Err(ProviderError::InvalidApiKey);
        }

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(120))
            .use_rustls_tls()
            .build()
            .map_err(|e| ProviderError::InternalError(format!("Failed to build HTTP client: {}", e)))?;

        Ok(Self {
            client,
            api_key,
            endpoint,
            deployment,
            api_version,
        })
    }

    fn build_request_body(&self, request: &CompletionRequest, stream: bool) -> serde_json::Value {
        let mut body = serde_json::json!({
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
impl Provider for AzureOpenAIProvider {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse, ProviderError> {
        let url = format!(
            "{}/openai/deployments/{}/chat/completions?api-version={}",
            self.endpoint, self.deployment, self.api_version
        );

        let body = self.build_request_body(&request, false);

        debug!("Sending request to Azure OpenAI");

        let response = self.client
            .post(&url)
            .header("api-key", &self.api_key)
            .json(&body)
            .send()
            .await
            .map_err(|e| ProviderError::NetworkError(e))?;

        let status = response.status().as_u16();
        if !response.status().is_success() {
            let text = response.text().await.unwrap_or_default();
            error!("Azure OpenAI API error ({}): {}", status, text);
            return Err(ProviderError::ApiError { status, message: text });
        }

        let text = response.text().await.map_err(|e| ProviderError::NetworkError(e))?;

        #[derive(Deserialize)]
        struct AzureResponse {
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

        let resp: AzureResponse = serde_json::from_str(&text)?;
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
                "stop" => FinishReason::Stop,
                "length" => FinishReason::Length,
                "content_filter" => FinishReason::ContentFilter,
                _ => FinishReason::Stop,
            },
            created_at: chrono::Utc::now(),
        })
    }

    async fn stream(&self, request: CompletionRequest) -> Result<ResponseStream, ProviderError> {
        let url = format!(
            "{}/openai/deployments/{}/chat/completions?api-version={}",
            self.endpoint, self.deployment, self.api_version
        );

        let body = self.build_request_body(&request, true);

        let response = self.client
            .post(&url)
            .header("api-key", &self.api_key)
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
            ModelInfo::new("gpt-35-turbo", "GPT-3.5 Turbo", 4096, true, true),
            ModelInfo::new("gpt-35-turbo-16k", "GPT-3.5 Turbo 16K", 16384, true, true),
            ModelInfo::new("gpt-4", "GPT-4", 8192, true, true),
            ModelInfo::new("gpt-4-32k", "GPT-4 32K", 32768, true, true),
            ModelInfo::new("gpt-4-turbo", "GPT-4 Turbo", 128000, true, true),
        ]
    }

    fn max_context_length(&self, model: &str) -> Option<usize> {
        match model {
            "gpt-35-turbo" => Some(4096),
            "gpt-35-turbo-16k" => Some(16384),
            "gpt-4" => Some(8192),
            "gpt-4-32k" => Some(32768),
            "gpt-4-turbo" => Some(128000),
            _ => None,
        }
    }

    fn name(&self) -> &str {
        "azure-openai"
    }

    async fn validate_config(&self) -> Result<(), ProviderError> {
        if self.api_key.is_empty() {
            return Err(ProviderError::InvalidApiKey);
        }
        if self.endpoint.is_empty() || self.deployment.is_empty() {
            return Err(ProviderError::InternalError("Missing endpoint or deployment".to_string()));
        }
        Ok(())
    }

    fn estimate_tokens(&self, text: &str, _model: &str) -> Result<usize, ProviderError> {
        Ok((text.len() / 4).max(1))
    }
}

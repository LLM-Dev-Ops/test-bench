// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Hugging Face Inference API provider implementation

use super::{CompletionRequest, CompletionResponse, FinishReason, ModelInfo, Provider, ProviderError, ResponseStream, TokenUsage};
use async_trait::async_trait;
use futures::stream::StreamExt;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{debug, error};

/// Hugging Face provider
pub struct HuggingFaceProvider {
    client: reqwest::Client,
    api_key: String,
    base_url: String,
}

impl HuggingFaceProvider {
    pub fn new(api_key: String) -> Result<Self, ProviderError> {
        Self::with_base_url(api_key, "https://api-inference.huggingface.co/models".to_string())
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

    fn build_request_body(&self, request: &CompletionRequest) -> serde_json::Value {
        let mut params = serde_json::Map::new();

        if let Some(temp) = request.temperature {
            params.insert("temperature".to_string(), serde_json::json!(temp));
        }
        if let Some(max_tokens) = request.max_tokens {
            params.insert("max_new_tokens".to_string(), serde_json::json!(max_tokens));
        }
        if let Some(top_p) = request.top_p {
            params.insert("top_p".to_string(), serde_json::json!(top_p));
        }

        serde_json::json!({
            "inputs": request.prompt,
            "parameters": params,
        })
    }
}

#[async_trait]
impl Provider for HuggingFaceProvider {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse, ProviderError> {
        let url = format!("{}/{}", self.base_url, request.model);
        let body = self.build_request_body(&request);

        debug!("Sending request to Hugging Face");

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
            error!("Hugging Face API error ({}): {}", status, text);
            return Err(ProviderError::ApiError { status, message: text });
        }

        let text = response.text().await.map_err(|e| ProviderError::NetworkError(e))?;

        // HF API returns array of results
        #[derive(Deserialize)]
        struct HFResponse {
            generated_text: String,
        }

        let responses: Vec<HFResponse> = serde_json::from_str(&text)
            .map_err(|e| ProviderError::InternalError(format!("Failed to parse response: {}", e)))?;

        let result = responses.first()
            .ok_or_else(|| ProviderError::ApiError { status: 500, message: "No results".to_string() })?;

        // Remove the prompt from the generated text if it's included
        let content = result.generated_text
            .strip_prefix(&request.prompt)
            .unwrap_or(&result.generated_text)
            .to_string();

        // Estimate tokens (HF doesn't always provide usage)
        let prompt_tokens = (request.prompt.len() / 4).max(1);
        let completion_tokens = (content.len() / 4).max(1);

        Ok(CompletionResponse {
            id: format!("hf-{}", chrono::Utc::now().timestamp()),
            content,
            model: request.model.clone(),
            usage: TokenUsage {
                prompt_tokens,
                completion_tokens,
                total_tokens: prompt_tokens + completion_tokens,
            },
            finish_reason: FinishReason::Stop,
            created_at: chrono::Utc::now(),
        })
    }

    async fn stream(&self, request: CompletionRequest) -> Result<ResponseStream, ProviderError> {
        // Hugging Face Inference API doesn't support streaming in the same way
        // We'll return a single-item stream
        let response = self.complete(request).await?;
        let content = response.content.clone();

        let stream = futures::stream::once(async move {
            Ok(content)
        });

        Ok(Box::pin(stream))
    }

    fn supported_models(&self) -> Vec<ModelInfo> {
        vec![
            ModelInfo::new("meta-llama/Llama-2-7b-chat-hf", "Llama 2 7B Chat", 4096, false, false),
            ModelInfo::new("meta-llama/Llama-2-13b-chat-hf", "Llama 2 13B Chat", 4096, false, false),
            ModelInfo::new("mistralai/Mistral-7B-Instruct-v0.2", "Mistral 7B Instruct", 32768, false, false),
            ModelInfo::new("mistralai/Mixtral-8x7B-Instruct-v0.1", "Mixtral 8x7B", 32768, false, false),
            ModelInfo::new("google/flan-t5-xxl", "FLAN-T5 XXL", 512, false, false),
            ModelInfo::new("bigcode/starcoder", "StarCoder", 8192, false, false),
        ]
    }

    fn max_context_length(&self, model: &str) -> Option<usize> {
        if model.contains("Llama-2") {
            Some(4096)
        } else if model.contains("Mistral") || model.contains("Mixtral") {
            Some(32768)
        } else if model.contains("starcoder") {
            Some(8192)
        } else if model.contains("flan-t5") {
            Some(512)
        } else {
            Some(2048)
        }
    }

    fn name(&self) -> &str {
        "huggingface"
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

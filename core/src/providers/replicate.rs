// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Replicate provider implementation

use super::{CompletionRequest, CompletionResponse, FinishReason, ModelInfo, Provider, ProviderError, ResponseStream, TokenUsage};
use async_trait::async_trait;
use futures::stream::StreamExt;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{debug, error};

/// Replicate provider
pub struct ReplicateProvider {
    client: reqwest::Client,
    api_key: String,
    base_url: String,
}

impl ReplicateProvider {
    pub fn new(api_key: String) -> Result<Self, ProviderError> {
        Self::with_base_url(api_key, "https://api.replicate.com/v1".to_string())
    }

    pub fn with_base_url(api_key: String, base_url: String) -> Result<Self, ProviderError> {
        if api_key.is_empty() {
            return Err(ProviderError::InvalidApiKey);
        }

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(300))  // Replicate can be slow for cold starts
            .use_rustls_tls()
            .build()
            .map_err(|e| ProviderError::InternalError(format!("Failed to build HTTP client: {}", e)))?;

        Ok(Self { client, api_key, base_url })
    }

    fn build_request_body(&self, request: &CompletionRequest) -> serde_json::Value {
        let version = self.get_model_version(&request.model);

        let mut input = serde_json::json!({
            "prompt": request.prompt,
        });

        if let Some(temp) = request.temperature {
            input["temperature"] = serde_json::json!(temp);
        }
        if let Some(max_tokens) = request.max_tokens {
            input["max_tokens"] = serde_json::json!(max_tokens);
        }
        if let Some(top_p) = request.top_p {
            input["top_p"] = serde_json::json!(top_p);
        }

        serde_json::json!({
            "version": version,
            "input": input,
        })
    }

    fn get_model_version(&self, model: &str) -> String {
        // These are example versions - in practice, you'd manage these separately
        match model {
            "meta/llama-2-70b-chat" => "02e509c789964a7ea8736978a43525956ef40397be9033abf9fd2badfe68c9e3",
            "mistralai/mixtral-8x7b-instruct-v0.1" => "2b56576fcfbe32fa0526897d8385dd3fb3d36ba6fd0dbe033c72886b81ade14e",
            _ => model,
        }.to_string()
    }
}

#[async_trait]
impl Provider for ReplicateProvider {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse, ProviderError> {
        let url = format!("{}/predictions", self.base_url);
        let body = self.build_request_body(&request);

        debug!("Sending request to Replicate");

        // Start prediction
        let response = self.client
            .post(&url)
            .header("Authorization", format!("Token {}", self.api_key))
            .json(&body)
            .send()
            .await
            .map_err(|e| ProviderError::NetworkError(e.to_string()))?;

        let status = response.status().as_u16();
        if !response.status().is_success() {
            let text = response.text().await.unwrap_or_default();
            error!("Replicate API error ({}): {}", status, text);
            return Err(ProviderError::ApiError { status, message: text });
        }

        let text = response.text().await.map_err(|e| ProviderError::NetworkError(e.to_string()))?;

        #[derive(Deserialize)]
        struct PredictionResponse {
            id: String,
            urls: Urls,
        }

        #[derive(Deserialize)]
        struct Urls {
            get: String,
        }

        let pred: PredictionResponse = serde_json::from_str(&text)?;

        // Poll for result
        let mut attempts = 0;
        loop {
            if attempts > 60 {  // 5 minutes max
                return Err(ProviderError::InternalError("Prediction timeout".to_string()));
            }

            tokio::time::sleep(Duration::from_secs(5)).await;

            let poll_response = self.client
                .get(&pred.urls.get)
                .header("Authorization", format!("Token {}", self.api_key))
                .send()
                .await
                .map_err(|e| ProviderError::NetworkError(e.to_string()))?;

            let poll_text = poll_response.text().await.unwrap_or_default();

            #[derive(Deserialize)]
            struct PollResponse {
                status: String,
                output: Option<serde_json::Value>,
            }

            let poll_result: PollResponse = serde_json::from_str(&poll_text)?;

            match poll_result.status.as_str() {
                "succeeded" => {
                    let output = poll_result.output.ok_or_else(|| {
                        ProviderError::ApiError { status: 500, message: "No output".to_string() }
                    })?;

                    // Extract text from output (can be string or array)
                    let content = if let Some(text) = output.as_str() {
                        text.to_string()
                    } else if let Some(arr) = output.as_array() {
                        arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>().join("")
                    } else {
                        output.to_string()
                    };

                    let tokens = (content.len() / 4).max(1);

                    return Ok(CompletionResponse {
                        id: pred.id,
                        content,
                        model: request.model.clone(),
                        usage: TokenUsage {
                            prompt_tokens: 0,
                            completion_tokens: tokens,
                            total_tokens: tokens,
                        },
                        finish_reason: FinishReason::Stop,
                        created_at: chrono::Utc::now(),
                    });
                }
                "failed" | "canceled" => {
                    return Err(ProviderError::ApiError {
                        status: 500,
                        message: format!("Prediction {}", poll_result.status),
                    });
                }
                _ => {
                    attempts += 1;
                    continue;
                }
            }
        }
    }

    async fn stream(&self, request: CompletionRequest) -> Result<ResponseStream, ProviderError> {
        // Replicate doesn't support true streaming, return single result
        let response = self.complete(request).await?;
        let content = response.content.clone();

        let stream = futures::stream::once(async move {
            Ok(content)
        });

        Ok(Box::pin(stream))
    }

    fn supported_models(&self) -> Vec<ModelInfo> {
        vec![
            ModelInfo::new("meta/llama-2-70b-chat", "Llama 2 70B Chat", 4096, false, false),
            ModelInfo::new("meta/llama-2-13b-chat", "Llama 2 13B Chat", 4096, false, false),
            ModelInfo::new("mistralai/mixtral-8x7b-instruct-v0.1", "Mixtral 8x7B", 32768, false, false),
            ModelInfo::new("stability-ai/sdxl", "Stable Diffusion XL", 77, false, false),
        ]
    }

    fn max_context_length(&self, model: &str) -> Option<usize> {
        if model.contains("llama-2") {
            Some(4096)
        } else if model.contains("mixtral") {
            Some(32768)
        } else {
            Some(2048)
        }
    }

    fn name(&self) -> &str {
        "replicate"
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

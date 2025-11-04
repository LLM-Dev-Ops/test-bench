// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Ollama provider implementation for local models

use super::{CompletionRequest, CompletionResponse, FinishReason, ModelInfo, Provider, ProviderError, ResponseStream, TokenUsage};
use async_trait::async_trait;
use futures::stream::StreamExt;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{debug, error};

/// Ollama provider for local model hosting
pub struct OllamaProvider {
    client: reqwest::Client,
    base_url: String,
}

impl OllamaProvider {
    pub fn new() -> Result<Self, ProviderError> {
        Self::with_base_url("http://localhost:11434".to_string())
    }

    pub fn with_base_url(base_url: String) -> Result<Self, ProviderError> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(300))  // Local inference can be slow
            .build()
            .map_err(|e| ProviderError::InternalError(format!("Failed to build HTTP client: {}", e)))?;

        Ok(Self { client, base_url })
    }

    fn build_request_body(&self, request: &CompletionRequest, stream: bool) -> serde_json::Value {
        let mut body = serde_json::json!({
            "model": request.model,
            "prompt": request.prompt,
            "stream": stream,
        });

        let mut options = serde_json::Map::new();

        if let Some(temp) = request.temperature {
            options.insert("temperature".to_string(), serde_json::json!(temp));
        }
        if let Some(top_p) = request.top_p {
            options.insert("top_p".to_string(), serde_json::json!(top_p));
        }

        if !options.is_empty() {
            body["options"] = serde_json::Value::Object(options);
        }

        if let Some(ref stop) = request.stop {
            body["stop"] = serde_json::json!(stop);
        }

        body
    }
}

impl Default for OllamaProvider {
    fn default() -> Self {
        Self::new().expect("Failed to create Ollama provider")
    }
}

#[async_trait]
impl Provider for OllamaProvider {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse, ProviderError> {
        let url = format!("{}/api/generate", self.base_url);
        let body = self.build_request_body(&request, false);

        debug!("Sending request to Ollama");

        let response = self.client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| ProviderError::NetworkError(e))?;

        let status = response.status().as_u16();
        if !response.status().is_success() {
            let text = response.text().await.unwrap_or_default();
            error!("Ollama API error ({}): {}", status, text);
            return Err(ProviderError::ApiError { status, message: text });
        }

        let text = response.text().await.map_err(|e| ProviderError::NetworkError(e))?;

        #[derive(Deserialize)]
        struct OllamaResponse {
            model: String,
            response: String,
            done: bool,
            context: Option<Vec<i32>>,
            total_duration: Option<i64>,
            prompt_eval_count: Option<i32>,
            eval_count: Option<i32>,
        }

        let resp: OllamaResponse = serde_json::from_str(&text)
            .map_err(|e| ProviderError::InternalError(format!("Failed to parse response: {}", e)))?;

        let prompt_tokens = resp.prompt_eval_count.unwrap_or(0) as usize;
        let completion_tokens = resp.eval_count.unwrap_or(0) as usize;

        Ok(CompletionResponse {
            id: format!("ollama-{}", chrono::Utc::now().timestamp()),
            content: resp.response,
            model: resp.model,
            usage: TokenUsage {
                prompt_tokens,
                completion_tokens,
                total_tokens: prompt_tokens + completion_tokens,
            },
            finish_reason: if resp.done { FinishReason::Stop } else { FinishReason::Length },
            created_at: chrono::Utc::now(),
        })
    }

    async fn stream(&self, request: CompletionRequest) -> Result<ResponseStream, ProviderError> {
        let url = format!("{}/api/generate", self.base_url);
        let body = self.build_request_body(&request, true);

        let response = self.client
            .post(&url)
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
                        if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
                            if let Some(response) = json.get("response").and_then(|r| r.as_str()) {
                                return Ok(response.to_string());
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
            ModelInfo::new("llama2", "Llama 2", 4096, true, false),
            ModelInfo::new("llama2:13b", "Llama 2 13B", 4096, true, false),
            ModelInfo::new("llama2:70b", "Llama 2 70B", 4096, true, false),
            ModelInfo::new("mistral", "Mistral 7B", 32768, true, false),
            ModelInfo::new("mixtral", "Mixtral 8x7B", 32768, true, false),
            ModelInfo::new("codellama", "Code Llama", 16384, true, false),
            ModelInfo::new("phi", "Phi-2", 2048, true, false),
            ModelInfo::new("gemma:7b", "Gemma 7B", 8192, true, false),
        ]
    }

    fn max_context_length(&self, model: &str) -> Option<usize> {
        if model.starts_with("llama2") {
            Some(4096)
        } else if model.starts_with("mistral") || model.starts_with("mixtral") {
            Some(32768)
        } else if model.starts_with("codellama") {
            Some(16384)
        } else if model.starts_with("gemma") {
            Some(8192)
        } else if model.starts_with("phi") {
            Some(2048)
        } else {
            Some(2048)
        }
    }

    fn name(&self) -> &str {
        "ollama"
    }

    async fn validate_config(&self) -> Result<(), ProviderError> {
        // Try to connect to the Ollama server
        let url = format!("{}/api/tags", self.base_url);
        match self.client.get(&url).send().await {
            Ok(resp) if resp.status().is_success() => Ok(()),
            Ok(resp) => Err(ProviderError::NetworkError(format!("Ollama server returned status: {}", resp.status()))),
            Err(e) => Err(ProviderError::NetworkError(format!("Cannot connect to Ollama: {}", e))),
        }
    }

    fn estimate_tokens(&self, text: &str, _model: &str) -> Result<usize, ProviderError> {
        Ok((text.len() / 4).max(1))
    }
}

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! AWS Bedrock provider implementation

use super::{CompletionRequest, CompletionResponse, FinishReason, ModelInfo, Provider, ProviderError, ResponseStream, TokenUsage};
use async_trait::async_trait;
use futures::stream;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{debug, error, warn};

/// AWS Bedrock provider
/// Note: This is a simplified implementation. Full AWS Bedrock support requires AWS SDK.
pub struct BedrockProvider {
    client: reqwest::Client,
    region: String,
    access_key: String,
    secret_key: String,
}

impl BedrockProvider {
    pub fn new(region: String, access_key: String, secret_key: String) -> Result<Self, ProviderError> {
        if access_key.is_empty() || secret_key.is_empty() {
            return Err(ProviderError::InvalidApiKey);
        }

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(120))
            .use_rustls_tls()
            .build()
            .map_err(|e| ProviderError::InternalError(format!("Failed to build HTTP client: {}", e)))?;

        Ok(Self {
            client,
            region,
            access_key,
            secret_key,
        })
    }
}

#[async_trait]
impl Provider for BedrockProvider {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse, ProviderError> {
        // Note: Full Bedrock implementation requires AWS SigV4 signing
        // This is a placeholder that shows the structure
        warn!("AWS Bedrock provider requires AWS SDK for full implementation");

        Err(ProviderError::InternalError(
            "AWS Bedrock provider requires AWS SDK. Please use AWS SDK directly or use a wrapper service.".to_string()
        ))
    }

    async fn stream(&self, _request: CompletionRequest) -> Result<ResponseStream, ProviderError> {
        Err(ProviderError::InternalError(
            "AWS Bedrock streaming requires AWS SDK implementation".to_string()
        ))
    }

    fn supported_models(&self) -> Vec<ModelInfo> {
        vec![
            ModelInfo::new("anthropic.claude-3-sonnet-20240229-v1:0", "Claude 3 Sonnet", 200000, true, false),
            ModelInfo::new("anthropic.claude-3-opus-20240229-v1:0", "Claude 3 Opus", 200000, true, false),
            ModelInfo::new("anthropic.claude-v2:1", "Claude 2.1", 200000, true, false),
            ModelInfo::new("amazon.titan-text-express-v1", "Titan Text Express", 8000, true, false),
            ModelInfo::new("meta.llama2-70b-chat-v1", "Llama 2 70B", 4096, true, false),
            ModelInfo::new("cohere.command-text-v14", "Command", 4096, true, false),
            ModelInfo::new("ai21.j2-ultra-v1", "Jurassic-2 Ultra", 8191, true, false),
        ]
    }

    fn max_context_length(&self, model: &str) -> Option<usize> {
        if model.contains("claude-3") || model.contains("claude-v2") {
            Some(200000)
        } else if model.contains("titan") {
            Some(8000)
        } else if model.contains("llama2") || model.contains("command") {
            Some(4096)
        } else if model.contains("j2") {
            Some(8191)
        } else {
            Some(4096)
        }
    }

    fn name(&self) -> &str {
        "bedrock"
    }

    async fn validate_config(&self) -> Result<(), ProviderError> {
        if self.access_key.is_empty() || self.secret_key.is_empty() {
            return Err(ProviderError::InvalidApiKey);
        }
        Ok(())
    }

    fn estimate_tokens(&self, text: &str, _model: &str) -> Result<usize, ProviderError> {
        Ok((text.len() / 4).max(1))
    }
}

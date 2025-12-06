//! LLM client integration bridge for infra-llm-client.
//!
//! This module provides a unified LLM client from the LLM-Dev-Ops infra layer,
//! with built-in support for retry, caching, and rate limiting.
//!
//! # Example
//!
//! ```rust,ignore
//! use llm_test_bench_core::infra::llm_client::{create_client, ProviderType, LlmRequest};
//!
//! // Create a client for OpenAI
//! let client = create_client(ProviderType::OpenAI, "your-api-key")?;
//!
//! // Make a completion request
//! let response = client.complete(
//!     LlmRequest::new("What is the capital of France?")
//!         .model("gpt-4")
//!         .temperature(0.7)
//! ).await?;
//!
//! println!("Response: {}", response.content);
//! println!("Tokens used: {}", response.usage.total_tokens);
//! ```

pub use infra_llm_client::{
    LlmClient, LlmClientBuilder, LlmConfig, ProviderConfig,
    LlmRequest, LlmResponse, LlmError,
    Provider, Message, Role, Usage, FinishReason,
    openai, anthropic,
    DEFAULT_TIMEOUT, DEFAULT_MAX_RETRIES,
};

#[cfg(feature = "google")]
pub use infra_llm_client::google;

#[cfg(feature = "azure")]
pub use infra_llm_client::azure;

use std::time::Duration;

/// Provider types for client creation (mirrors rate_limit::ProviderType)
pub type ProviderType = Provider;

/// Create a client for a specific provider.
pub fn create_client(provider: Provider, api_key: impl Into<String>) -> Result<LlmClient, LlmError> {
    LlmClient::builder()
        .provider(provider)
        .api_key(api_key)
        .with_cache()
        .with_rate_limit()
        .build()
}

/// Create a client with custom configuration.
pub fn create_client_with_config(
    provider: Provider,
    api_key: impl Into<String>,
    model: impl Into<String>,
    timeout: Duration,
) -> Result<LlmClient, LlmError> {
    LlmClient::builder()
        .provider(provider)
        .api_key(api_key)
        .default_model(model)
        .timeout(timeout)
        .with_cache()
        .with_rate_limit()
        .build()
}

/// Create a minimal client without caching or rate limiting.
/// Useful for testing or when you want to manage these externally.
pub fn create_basic_client(
    provider: Provider,
    api_key: impl Into<String>,
) -> Result<LlmClient, LlmError> {
    LlmClient::builder()
        .provider(provider)
        .api_key(api_key)
        .build()
}

/// Get the API key from environment for a provider.
pub fn get_api_key_from_env(provider: Provider) -> Option<String> {
    std::env::var(provider.api_key_env()).ok()
}

/// Check if an API key is configured for a provider.
pub fn has_api_key(provider: Provider) -> bool {
    get_api_key_from_env(provider).is_some()
}

/// Builder for creating multiple clients with shared configuration.
pub struct ClientFactory {
    timeout: Duration,
    max_retries: u32,
    enable_cache: bool,
    enable_rate_limit: bool,
}

impl Default for ClientFactory {
    fn default() -> Self {
        Self {
            timeout: DEFAULT_TIMEOUT,
            max_retries: DEFAULT_MAX_RETRIES,
            enable_cache: true,
            enable_rate_limit: true,
        }
    }
}

impl ClientFactory {
    /// Create a new client factory with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the request timeout.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set the maximum retry attempts.
    pub fn max_retries(mut self, retries: u32) -> Self {
        self.max_retries = retries;
        self
    }

    /// Disable response caching.
    pub fn without_cache(mut self) -> Self {
        self.enable_cache = false;
        self
    }

    /// Disable rate limiting.
    pub fn without_rate_limit(mut self) -> Self {
        self.enable_rate_limit = false;
        self
    }

    /// Create a client for the specified provider.
    pub fn create(&self, provider: Provider, api_key: impl Into<String>) -> Result<LlmClient, LlmError> {
        let mut builder = LlmClient::builder()
            .provider(provider)
            .api_key(api_key)
            .timeout(self.timeout)
            .max_retries(self.max_retries);

        if self.enable_cache {
            builder = builder.with_cache();
        }
        if self.enable_rate_limit {
            builder = builder.with_rate_limit();
        }

        builder.build()
    }

    /// Create a client using the API key from environment.
    pub fn create_from_env(&self, provider: Provider) -> Result<LlmClient, LlmError> {
        let api_key = get_api_key_from_env(provider)
            .ok_or_else(|| LlmError::config(format!(
                "API key not found in environment: {}",
                provider.api_key_env()
            )))?;

        self.create(provider, api_key)
    }
}

/// Create a simple chat request.
pub fn chat(prompt: impl Into<String>) -> LlmRequest {
    LlmRequest::new(prompt)
}

/// Create a chat request with a system message.
pub fn chat_with_system(system: impl Into<String>, prompt: impl Into<String>) -> LlmRequest {
    LlmRequest::new(prompt).with_system(system)
}

/// Create a request from multiple messages.
pub fn chat_from_messages(messages: Vec<Message>) -> LlmRequest {
    LlmRequest::from_messages(messages)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_api_key_env() {
        assert_eq!(Provider::OpenAI.api_key_env(), "OPENAI_API_KEY");
        assert_eq!(Provider::Anthropic.api_key_env(), "ANTHROPIC_API_KEY");
    }

    #[test]
    fn test_chat_request() {
        let request = chat("Hello, world!")
            .model("gpt-4")
            .temperature(0.7);

        assert_eq!(request.model(), Some("gpt-4"));
        assert_eq!(request.temperature(), Some(0.7));
    }

    #[test]
    fn test_chat_with_system() {
        let request = chat_with_system(
            "You are a helpful assistant.",
            "What is 2+2?"
        );

        assert_eq!(request.messages().len(), 2);
        assert_eq!(request.messages()[0].role, Role::System);
        assert_eq!(request.messages()[1].role, Role::User);
    }

    #[test]
    fn test_client_factory() {
        let factory = ClientFactory::new()
            .timeout(Duration::from_secs(60))
            .max_retries(5)
            .without_cache();

        // Can't actually create a client without an API key
        assert!(factory.create_from_env(Provider::OpenAI).is_err());
    }
}

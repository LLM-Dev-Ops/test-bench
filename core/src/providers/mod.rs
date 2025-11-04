// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! LLM Provider implementations and abstractions.
//!
//! This module provides a unified interface for interacting with various LLM
//! providers through the [`Provider`] trait. It includes:
//!
//! - **Core abstractions**: The [`Provider`] trait and related types
//! - **Error handling**: Comprehensive error types for provider operations
//! - **Shared types**: Common request/response structures
//! - **Provider implementations**: OpenAI, Anthropic, and future providers
//! - **Factory pattern**: For creating provider instances from configuration
//!
//! # Architecture
//!
//! The provider system uses a trait-based abstraction layer to support
//! multiple LLM providers while presenting a consistent API:
//!
//! ```text
//! ┌─────────────────────────────────────┐
//! │       Provider Trait                │
//! │  (Unified API for all providers)    │
//! └─────────────────────────────────────┘
//!           ↓              ↓
//!    ┌──────────┐   ┌──────────────┐
//!    │  OpenAI  │   │   Anthropic  │
//!    │ Provider │   │   Provider   │
//!    └──────────┘   └──────────────┘
//! ```
//!
//! # Usage Examples
//!
//! ## Creating and using a provider
//!
//! ```no_run
//! use llm_test_bench_core::providers::{Provider, CompletionRequest, ProviderFactory};
//! use llm_test_bench_core::config::models::ProviderConfig;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create provider from configuration
//! let factory = ProviderFactory::new();
//! let config = ProviderConfig {
//!     name: "openai".to_string(),
//!     api_key_env: "OPENAI_API_KEY".to_string(),
//!     base_url: "https://api.openai.com/v1".to_string(),
//!     default_model: "gpt-4".to_string(),
//!     timeout_seconds: 30,
//!     max_retries: 3,
//!     enabled: true,
//! };
//!
//! let provider = factory.create("openai", &config)?;
//!
//! // Make a completion request
//! let request = CompletionRequest::new("gpt-4", "Explain Rust ownership")
//!     .with_max_tokens(100)
//!     .with_temperature(0.7);
//!
//! let response = provider.complete(request).await?;
//! println!("Response: {}", response.content);
//! println!("Tokens used: {}", response.usage.total_tokens);
//! # Ok(())
//! # }
//! ```
//!
//! ## Streaming responses
//!
//! ```no_run
//! use llm_test_bench_core::providers::{Provider, CompletionRequest, ProviderFactory};
//! use llm_test_bench_core::config::models::ProviderConfig;
//! use futures::StreamExt;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! # let factory = ProviderFactory::new();
//! # let config = ProviderConfig {
//! #     name: "openai".to_string(),
//! #     api_key_env: "OPENAI_API_KEY".to_string(),
//! #     base_url: "https://api.openai.com/v1".to_string(),
//! #     default_model: "gpt-4".to_string(),
//! #     timeout_seconds: 30,
//! #     max_retries: 3,
//! #     enabled: true,
//! # };
//! # let provider = factory.create("openai", &config)?;
//! let request = CompletionRequest::new("gpt-4", "Write a story")
//!     .with_streaming();
//!
//! let mut stream = provider.stream(request).await?;
//! while let Some(chunk) = stream.next().await {
//!     match chunk {
//!         Ok(text) => print!("{}", text),
//!         Err(e) => eprintln!("Error: {}", e),
//!     }
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## Handling errors
//!
//! ```no_run
//! use llm_test_bench_core::providers::{Provider, CompletionRequest, ProviderError};
//! # use llm_test_bench_core::providers::OpenAIProvider;
//!
//! # async fn example(provider: OpenAIProvider) {
//! let request = CompletionRequest::new("gpt-4", "Hello");
//!
//! match provider.complete(request).await {
//!     Ok(response) => println!("Success: {}", response.content),
//!     Err(ProviderError::RateLimitExceeded { retry_after }) => {
//!         if let Some(delay) = retry_after {
//!             println!("Rate limited. Retry after {:?}", delay);
//!         }
//!     }
//!     Err(ProviderError::InvalidApiKey) => {
//!         eprintln!("Invalid API key. Check your configuration.");
//!     }
//!     Err(e) => eprintln!("Error: {}", e),
//! }
//! # }
//! ```

// Core modules
pub mod error;
pub mod factory;
pub mod models;
pub mod traits;
pub mod types;

// Provider implementations
pub mod anthropic;
pub mod openai;
pub mod google;
pub mod cohere;
pub mod mistral;
pub mod groq;
pub mod together;
pub mod huggingface;
pub mod ollama;
pub mod azure_openai;
pub mod bedrock;
pub mod replicate;
pub mod perplexity;

// Re-export commonly used types
pub use error::ProviderError;
pub use factory::ProviderFactory;
pub use traits::{calculate_backoff, Provider, RetryableProvider};
pub use types::{
    CompletionRequest, CompletionResponse, FinishReason, ModelInfo, ResponseStream, TokenUsage,
};

// Re-export provider implementations
pub use anthropic::AnthropicProvider;
pub use azure_openai::AzureOpenAIProvider;
pub use bedrock::BedrockProvider;
pub use cohere::CohereProvider;
pub use google::GoogleProvider;
pub use groq::GroqProvider;
pub use huggingface::HuggingFaceProvider;
pub use mistral::MistralProvider;
pub use ollama::OllamaProvider;
pub use openai::OpenAIProvider;
pub use perplexity::PerplexityProvider;
pub use replicate::ReplicateProvider;
pub use together::TogetherProvider;

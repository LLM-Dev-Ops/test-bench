// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Provider factory for creating provider instances.
//!
//! This module provides a factory pattern for creating provider instances
//! from configuration. It handles provider registration and instantiation.

use std::collections::HashMap;
use std::sync::Arc;

use super::anthropic::AnthropicProvider;
use super::error::ProviderError;
use super::openai::OpenAIProvider;
use super::traits::Provider;
use crate::config::models::ProviderConfig;

/// A factory for creating provider instances.
///
/// The factory maintains a registry of available providers and creates
/// instances based on configuration.
///
/// # Examples
///
/// ```no_run
/// use llm_test_bench_core::providers::ProviderFactory;
/// use llm_test_bench_core::config::models::ProviderConfig;
///
/// let factory = ProviderFactory::new();
/// let config = ProviderConfig {
///     name: "openai".to_string(),
///     api_key_env: "OPENAI_API_KEY".to_string(),
///     base_url: "https://api.openai.com/v1".to_string(),
///     default_model: "gpt-4".to_string(),
///     timeout_seconds: 30,
///     max_retries: 3,
///     enabled: true,
/// };
///
/// let provider = factory.create("openai", &config).unwrap();
/// ```
pub struct ProviderFactory {
    _registry: HashMap<String, fn(&ProviderConfig) -> Result<Box<dyn Provider>, ProviderError>>,
}

impl ProviderFactory {
    /// Creates a new provider factory with default providers registered.
    ///
    /// By default, the following providers are registered:
    /// - `openai` - OpenAI API (GPT models)
    /// - `anthropic` - Anthropic API (Claude models)
    ///
    /// # Examples
    ///
    /// ```
    /// use llm_test_bench_core::providers::ProviderFactory;
    ///
    /// let factory = ProviderFactory::new();
    /// ```
    pub fn new() -> Self {
        let mut registry = HashMap::new();

        // Register built-in providers
        registry.insert("openai".to_string(), create_openai as _);
        registry.insert("anthropic".to_string(), create_anthropic as _);

        Self { _registry: registry }
    }

    /// Creates a provider instance from configuration.
    ///
    /// # Arguments
    ///
    /// * `provider_name` - The name of the provider (e.g., "openai", "anthropic")
    /// * `config` - The provider configuration
    ///
    /// # Returns
    ///
    /// A boxed provider instance that implements the `Provider` trait.
    ///
    /// # Errors
    ///
    /// - `ProviderError::InvalidRequest` - Unknown provider name
    /// - `ProviderError::InvalidApiKey` - API key is missing or invalid
    /// - Other provider-specific errors during initialization
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use llm_test_bench_core::providers::ProviderFactory;
    /// use llm_test_bench_core::config::models::ProviderConfig;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let factory = ProviderFactory::new();
    /// let config = ProviderConfig {
    ///     name: "openai".to_string(),
    ///     api_key_env: "OPENAI_API_KEY".to_string(),
    ///     base_url: "https://api.openai.com/v1".to_string(),
    ///     default_model: "gpt-4".to_string(),
    ///     timeout_seconds: 30,
    ///     max_retries: 3,
    ///     enabled: true,
    /// };
    ///
    /// let provider = factory.create("openai", &config)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn create(&self, provider_name: &str, config: &ProviderConfig) -> Result<Box<dyn Provider>, ProviderError> {
        match provider_name.to_lowercase().as_str() {
            "openai" => create_openai(config),
            "anthropic" => create_anthropic(config),
            _ => Err(ProviderError::InvalidRequest(format!(
                "Unknown provider: {}. Supported providers: openai, anthropic",
                provider_name
            ))),
        }
    }

    /// Creates a provider instance and wraps it in an Arc for shared ownership.
    ///
    /// This is useful when you need to share a provider across multiple tasks
    /// or components.
    ///
    /// # Arguments
    ///
    /// * `provider_name` - The name of the provider
    /// * `config` - The provider configuration
    ///
    /// # Returns
    ///
    /// An Arc-wrapped provider instance.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use llm_test_bench_core::providers::ProviderFactory;
    /// use llm_test_bench_core::config::models::ProviderConfig;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let factory = ProviderFactory::new();
    /// let config = ProviderConfig {
    ///     name: "openai".to_string(),
    ///     api_key_env: "OPENAI_API_KEY".to_string(),
    ///     base_url: "https://api.openai.com/v1".to_string(),
    ///     default_model: "gpt-4".to_string(),
    ///     timeout_seconds: 30,
    ///     max_retries: 3,
    ///     enabled: true,
    /// };
    ///
    /// let provider = factory.create_shared("openai", &config)?;
    /// // Can now clone and share the Arc across tasks
    /// let provider_clone = provider.clone();
    /// # Ok(())
    /// # }
    /// ```
    pub fn create_shared(
        &self,
        provider_name: &str,
        config: &ProviderConfig,
    ) -> Result<Arc<dyn Provider>, ProviderError> {
        let provider = self.create(provider_name, config)?;
        Ok(Arc::from(provider))
    }

    /// Returns a list of all registered provider names.
    ///
    /// # Examples
    ///
    /// ```
    /// use llm_test_bench_core::providers::ProviderFactory;
    ///
    /// let factory = ProviderFactory::new();
    /// let providers = factory.available_providers();
    /// assert!(providers.contains(&"openai".to_string()));
    /// assert!(providers.contains(&"anthropic".to_string()));
    /// ```
    pub fn available_providers(&self) -> Vec<String> {
        vec!["openai".to_string(), "anthropic".to_string()]
    }
}

impl Default for ProviderFactory {
    fn default() -> Self {
        Self::new()
    }
}

/// Creates an OpenAI provider instance from configuration.
fn create_openai(config: &ProviderConfig) -> Result<Box<dyn Provider>, ProviderError> {
    // Get API key from environment
    let api_key = std::env::var(&config.api_key_env)
        .map_err(|_| ProviderError::InvalidApiKey)?;

    // Create provider with custom base URL if specified
    let provider = if config.base_url.contains("openai.com") {
        OpenAIProvider::new(api_key)?
    } else {
        OpenAIProvider::with_base_url(api_key, config.base_url.clone())?
    };

    Ok(Box::new(provider))
}

/// Creates an Anthropic provider instance from configuration.
fn create_anthropic(config: &ProviderConfig) -> Result<Box<dyn Provider>, ProviderError> {
    // Get API key from environment
    let api_key = std::env::var(&config.api_key_env)
        .map_err(|_| ProviderError::InvalidApiKey)?;

    // Create provider with custom base URL if specified
    let provider = if config.base_url.contains("anthropic.com") {
        AnthropicProvider::new(api_key)
    } else {
        AnthropicProvider::with_base_url(api_key, config.base_url.clone())
    };

    Ok(Box::new(provider))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config(provider_name: &str) -> ProviderConfig {
        ProviderConfig {
            api_key_env: format!("{}_API_KEY", provider_name.to_uppercase()),
            base_url: format!("https://api.{}.com/v1", provider_name),
            default_model: "test-model".to_string(),
            timeout_seconds: 30,
            max_retries: 3,
            rate_limit_rpm: None,
        }
    }

    #[test]
    fn test_factory_creation() {
        let factory = ProviderFactory::new();
        assert!(!factory.available_providers().is_empty());
    }

    #[test]
    fn test_factory_default() {
        let factory = ProviderFactory::default();
        assert!(!factory.available_providers().is_empty());
    }

    #[test]
    fn test_available_providers() {
        let factory = ProviderFactory::new();
        let providers = factory.available_providers();
        assert!(providers.contains(&"openai".to_string()));
        assert!(providers.contains(&"anthropic".to_string()));
    }

    #[test]
    fn test_create_unknown_provider() {
        let factory = ProviderFactory::new();
        let config = test_config("unknown");
        let result = factory.create("unknown", &config);
        assert!(result.is_err());
        match result {
            Err(ProviderError::InvalidRequest(msg)) => {
                assert!(msg.contains("Unknown provider"));
            }
            _ => panic!("Expected InvalidRequest error"),
        }
    }

    #[test]
    fn test_create_openai_without_api_key() {
        let factory = ProviderFactory::new();
        let config = test_config("openai");

        // Ensure the env var is not set
        std::env::remove_var(&config.api_key_env);

        let result = factory.create("openai", &config);
        assert!(result.is_err());
        assert!(matches!(result, Err(ProviderError::InvalidApiKey)));
    }

    #[test]
    fn test_create_openai_with_api_key() {
        let factory = ProviderFactory::new();
        let config = test_config("openai");

        // Set the API key
        std::env::set_var(&config.api_key_env, "test-key");

        let result = factory.create("openai", &config);
        assert!(result.is_ok());

        let provider = result.unwrap();
        assert_eq!(provider.name(), "openai");

        // Clean up
        std::env::remove_var(&config.api_key_env);
    }

    #[test]
    fn test_create_anthropic_with_api_key() {
        let factory = ProviderFactory::new();
        let config = test_config("anthropic");

        // Set the API key
        std::env::set_var(&config.api_key_env, "test-key");

        let result = factory.create("anthropic", &config);
        assert!(result.is_ok());

        let provider = result.unwrap();
        assert_eq!(provider.name(), "anthropic");

        // Clean up
        std::env::remove_var(&config.api_key_env);
    }

    #[test]
    fn test_create_shared() {
        let factory = ProviderFactory::new();
        let config = test_config("openai");

        std::env::set_var(&config.api_key_env, "test-key");

        let result = factory.create_shared("openai", &config);
        assert!(result.is_ok());

        let provider = result.unwrap();
        let provider_clone = provider.clone();
        assert_eq!(provider.name(), "openai");
        assert_eq!(provider_clone.name(), "openai");

        std::env::remove_var(&config.api_key_env);
    }

    #[test]
    fn test_case_insensitive_provider_names() {
        let factory = ProviderFactory::new();
        let config = test_config("openai");

        std::env::set_var(&config.api_key_env, "test-key");

        // Test different casings
        assert!(factory.create("OpenAI", &config).is_ok());
        assert!(factory.create("OPENAI", &config).is_ok());
        assert!(factory.create("openai", &config).is_ok());

        std::env::remove_var(&config.api_key_env);
    }
}

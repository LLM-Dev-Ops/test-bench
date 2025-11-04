//! Configuration management for LLM Test Bench
//!
//! This module provides a hierarchical configuration system with the following precedence:
//! 1. CLI Arguments (highest priority)
//! 2. Environment Variables (prefixed with LLM_TEST_BENCH_)
//! 3. Config Files (~/.config/llm-test-bench/config.toml)
//! 4. Defaults (lowest priority)
//!
//! # Examples
//!
//! ```no_run
//! use llm_test_bench_core::config::ConfigLoader;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Load configuration with all sources
//! let config = ConfigLoader::new().load()?;
//!
//! // Load from specific file
//! let config = ConfigLoader::new()
//!     .with_file("/path/to/config.toml")
//!     .load()?;
//!
//! // Load with environment variable overrides
//! std::env::set_var("LLM_TEST_BENCH_BENCHMARKS__PARALLEL_REQUESTS", "10");
//! let config = ConfigLoader::new().load()?;
//! # Ok(())
//! # }
//! ```

pub mod models;

use anyhow::{Context, Result};
use config::{Config as ConfigBuilder, ConfigError, Environment, File};
use serde_valid::Validate;
use std::path::{Path, PathBuf};

// Re-export all public types from models module
pub use models::{
    AnalyticsConfig, BenchmarkConfig, Config, DashboardConfig, EvaluationConfig, Metric,
    OrchestrationConfig, ProviderConfig,
};

/// Default configuration file name
const CONFIG_FILE_NAME: &str = "config.toml";

/// Default configuration directory name
const CONFIG_DIR_NAME: &str = "llm-test-bench";

/// Environment variable prefix for configuration overrides
pub const ENV_PREFIX: &str = "LLM_TEST_BENCH";

/// Environment variable separator for nested configuration
/// Example: LLM_TEST_BENCH_PROVIDERS__OPENAI__API_KEY
const ENV_SEPARATOR: &str = "__";

/// Configuration loader with builder pattern
///
/// Provides a flexible way to load configuration from multiple sources
/// with proper precedence handling.
#[derive(Debug, Default)]
pub struct ConfigLoader {
    /// Optional custom configuration file path
    custom_file: Option<PathBuf>,
    /// Whether to skip loading from default config file
    skip_default_file: bool,
    /// Whether to skip loading from environment variables
    skip_env: bool,
}

impl ConfigLoader {
    /// Create a new configuration loader with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Specify a custom configuration file path
    ///
    /// This will be used instead of the default config file location.
    pub fn with_file<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.custom_file = Some(path.as_ref().to_path_buf());
        self
    }

    /// Skip loading from the default configuration file
    ///
    /// Useful for testing or when you want to use only environment variables
    pub fn skip_default_file(mut self) -> Self {
        self.skip_default_file = true;
        self
    }

    /// Skip loading from environment variables
    ///
    /// Useful for testing or when you want strict file-only configuration
    pub fn skip_env(mut self) -> Self {
        self.skip_env = true;
        self
    }

    /// Load the configuration from all sources
    ///
    /// Configuration is loaded in this order (later sources override earlier):
    /// 1. Defaults (from Config::default())
    /// 2. Config file (if exists)
    /// 3. Environment variables (if enabled)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Configuration file parsing fails
    /// - Environment variable format is invalid
    /// - Validation fails (required fields missing, invalid values, etc.)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use llm_test_bench_core::config::ConfigLoader;
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = ConfigLoader::new().load()?;
    /// println!("Parallel requests: {}", config.benchmarks.parallel_requests);
    /// # Ok(())
    /// # }
    /// ```
    pub fn load(&self) -> Result<Config> {
        let mut builder = ConfigBuilder::builder();

        // Start with defaults serialized to a map
        let defaults = Config::default();
        let defaults_map = config_to_map(&defaults)?;
        builder = builder.add_source(config::Config::try_from(&defaults_map)?);

        // Load from config file if not skipped
        if !self.skip_default_file {
            if let Some(config_path) = self.find_config_file() {
                tracing::debug!("Loading config from: {}", config_path.display());
                builder = builder.add_source(
                    File::from(config_path)
                        .required(false)
                        .format(config::FileFormat::Toml),
                );
            } else {
                tracing::debug!("No default config file found");
            }
        }

        // Load from custom file if specified
        if let Some(ref custom_path) = self.custom_file {
            tracing::info!("Loading custom config from: {}", custom_path.display());
            builder = builder.add_source(
                File::from(custom_path.as_ref())
                    .required(true)
                    .format(config::FileFormat::Toml),
            );
        }

        // Load from environment variables if not skipped
        if !self.skip_env {
            tracing::debug!("Loading config from environment variables");
            builder = builder.add_source(
                Environment::with_prefix(ENV_PREFIX)
                    .separator(ENV_SEPARATOR)
                    .try_parsing(true)
                    // Convert ALL_CAPS to lowercase for field matching
                    .with_list_parse_key("evaluation.metrics")
                    .list_separator(","),
            );
        }

        // Build the configuration
        let config_result = builder.build().context("Failed to build configuration")?;

        // Deserialize into our Config struct
        let config: Config = config_result
            .try_deserialize()
            .context("Failed to deserialize configuration")?;

        // Validate the configuration
        config
            .validate()
            .map_err(|e| anyhow::anyhow!("Configuration validation failed: {}", e))?;

        tracing::info!("Configuration loaded successfully");
        Ok(config)
    }

    /// Find the default configuration file
    ///
    /// Searches in the following locations:
    /// 1. $XDG_CONFIG_HOME/llm-test-bench/config.toml (Linux)
    /// 2. ~/Library/Application Support/llm-test-bench/config.toml (macOS)
    /// 3. %APPDATA%/llm-test-bench/config.toml (Windows)
    /// 4. ./config.toml (current directory fallback)
    fn find_config_file(&self) -> Option<PathBuf> {
        // Try platform-specific config directory
        if let Some(config_dir) = dirs::config_dir() {
            let config_path = config_dir.join(CONFIG_DIR_NAME).join(CONFIG_FILE_NAME);
            if config_path.exists() {
                return Some(config_path);
            }
        }

        // Fallback to current directory
        let local_config = PathBuf::from(CONFIG_FILE_NAME);
        if local_config.exists() {
            return Some(local_config);
        }

        None
    }

    /// Get the default configuration directory path
    ///
    /// Returns the platform-specific configuration directory where
    /// the config file should be placed.
    ///
    /// # Examples
    ///
    /// ```
    /// use llm_test_bench_core::config::ConfigLoader;
    ///
    /// let config_dir = ConfigLoader::default_config_dir();
    /// println!("Config directory: {:?}", config_dir);
    /// ```
    pub fn default_config_dir() -> Option<PathBuf> {
        dirs::config_dir().map(|dir| dir.join(CONFIG_DIR_NAME))
    }

    /// Get the default configuration file path
    ///
    /// Returns the full path where the default config file is expected.
    pub fn default_config_path() -> Option<PathBuf> {
        Self::default_config_dir().map(|dir| dir.join(CONFIG_FILE_NAME))
    }
}

/// Helper function to convert Config to a HashMap for config builder
fn config_to_map(config: &Config) -> Result<serde_json::Value, ConfigError> {
    serde_json::to_value(config).map_err(|e| ConfigError::Foreign(Box::new(e)))
}

/// Initialize a default configuration file at the standard location
///
/// Creates the configuration directory if it doesn't exist and writes
/// a default configuration file.
///
/// # Errors
///
/// Returns an error if:
/// - Cannot determine config directory
/// - Cannot create directories
/// - Cannot write file
///
/// # Examples
///
/// ```no_run
/// use llm_test_bench_core::config::init_config_file;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let config_path = init_config_file()?;
/// println!("Created config file at: {}", config_path.display());
/// # Ok(())
/// # }
/// ```
pub fn init_config_file() -> Result<PathBuf> {
    let config_dir = ConfigLoader::default_config_dir()
        .context("Could not determine config directory")?;

    // Create directory if it doesn't exist
    std::fs::create_dir_all(&config_dir)
        .context("Failed to create config directory")?;

    let config_path = config_dir.join(CONFIG_FILE_NAME);

    // Don't overwrite existing config
    if config_path.exists() {
        anyhow::bail!("Config file already exists at: {}", config_path.display());
    }

    // Generate default config and write it
    let default_config = Config::default();
    let toml_content = toml::to_string_pretty(&default_config)
        .context("Failed to serialize default config")?;

    std::fs::write(&config_path, toml_content)
        .context("Failed to write config file")?;

    Ok(config_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use tempfile::TempDir;

    #[test]
    fn test_load_default_config() {
        let loader = ConfigLoader::new()
            .skip_default_file()
            .skip_env();
        let config = loader.load().expect("Failed to load default config");

        assert!(config.providers.contains_key("openai"));
        assert!(config.providers.contains_key("anthropic"));
        assert_eq!(config.benchmarks.parallel_requests, 5);
    }

    #[test]
    fn test_load_from_custom_file() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test_config.toml");

        let custom_config = r#"
[providers.openai]
api_key_env = "CUSTOM_OPENAI_KEY"
base_url = "https://custom.openai.com"
default_model = "gpt-3.5-turbo"
timeout_seconds = 60
max_retries = 5

[benchmarks]
output_dir = "/tmp/custom-results"
save_responses = false
parallel_requests = 10
continue_on_failure = true

[evaluation]
metrics = ["latency", "relevance"]
llm_judge_model = "gpt-3.5-turbo"
confidence_threshold = 0.8
include_explanations = false
        "#;

        std::fs::write(&config_path, custom_config).unwrap();

        let loader = ConfigLoader::new()
            .with_file(&config_path)
            .skip_env();
        let config = loader.load().expect("Failed to load custom config");

        // Verify custom values were loaded
        let openai = config.providers.get("openai").unwrap();
        assert_eq!(openai.api_key_env, "CUSTOM_OPENAI_KEY");
        assert_eq!(openai.default_model, "gpt-3.5-turbo");
        assert_eq!(openai.timeout_seconds, 60);

        assert_eq!(config.benchmarks.parallel_requests, 10);
        assert_eq!(config.benchmarks.save_responses, false);

        assert_eq!(config.evaluation.metrics.len(), 2);
        assert_eq!(config.evaluation.confidence_threshold, 0.8);
    }

    #[test]
    fn test_environment_variable_override() {
        // Set environment variables
        env::set_var("LLM_TEST_BENCH_BENCHMARKS__PARALLEL_REQUESTS", "20");
        env::set_var("LLM_TEST_BENCH_BENCHMARKS__SAVE_RESPONSES", "false");
        env::set_var("LLM_TEST_BENCH_EVALUATION__LLM_JUDGE_MODEL", "claude-3-opus");

        let loader = ConfigLoader::new().skip_default_file();
        let config = loader.load().expect("Failed to load config with env vars");

        assert_eq!(config.benchmarks.parallel_requests, 20);
        assert_eq!(config.benchmarks.save_responses, false);
        assert_eq!(config.evaluation.llm_judge_model, "claude-3-opus");

        // Cleanup
        env::remove_var("LLM_TEST_BENCH_BENCHMARKS__PARALLEL_REQUESTS");
        env::remove_var("LLM_TEST_BENCH_BENCHMARKS__SAVE_RESPONSES");
        env::remove_var("LLM_TEST_BENCH_EVALUATION__LLM_JUDGE_MODEL");
    }

    #[test]
    fn test_nested_provider_env_override() {
        env::set_var("LLM_TEST_BENCH_PROVIDERS__OPENAI__DEFAULT_MODEL", "gpt-4");
        env::set_var("LLM_TEST_BENCH_PROVIDERS__OPENAI__TIMEOUT_SECONDS", "120");

        let loader = ConfigLoader::new().skip_default_file();
        let config = loader.load().expect("Failed to load config");

        let openai = config.providers.get("openai").unwrap();
        assert_eq!(openai.default_model, "gpt-4");
        assert_eq!(openai.timeout_seconds, 120);

        // Cleanup
        env::remove_var("LLM_TEST_BENCH_PROVIDERS__OPENAI__DEFAULT_MODEL");
        env::remove_var("LLM_TEST_BENCH_PROVIDERS__OPENAI__TIMEOUT_SECONDS");
    }

    #[test]
    fn test_validation_failure_invalid_timeout() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("invalid_config.toml");

        // Timeout exceeds maximum of 300 seconds
        let invalid_config = r#"
[providers.openai]
api_key_env = "OPENAI_API_KEY"
base_url = "https://api.openai.com/v1"
default_model = "gpt-4"
timeout_seconds = 500
max_retries = 3
        "#;

        std::fs::write(&config_path, invalid_config).unwrap();

        let loader = ConfigLoader::new()
            .with_file(&config_path)
            .skip_env();
        let result = loader.load();

        assert!(result.is_err());
    }

    #[test]
    fn test_validation_failure_empty_model() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("invalid_config.toml");

        let invalid_config = r#"
[providers.openai]
api_key_env = "OPENAI_API_KEY"
base_url = "https://api.openai.com/v1"
default_model = ""
timeout_seconds = 30
max_retries = 3
        "#;

        std::fs::write(&config_path, invalid_config).unwrap();

        let loader = ConfigLoader::new()
            .with_file(&config_path)
            .skip_env();
        let result = loader.load();

        assert!(result.is_err());
    }

    #[test]
    fn test_precedence_env_over_file() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        let file_config = r#"
[benchmarks]
parallel_requests = 5
        "#;

        std::fs::write(&config_path, file_config).unwrap();

        // Environment variable should override file
        env::set_var("LLM_TEST_BENCH_BENCHMARKS__PARALLEL_REQUESTS", "15");

        let loader = ConfigLoader::new().with_file(&config_path);
        let config = loader.load().expect("Failed to load config");

        assert_eq!(config.benchmarks.parallel_requests, 15);

        // Cleanup
        env::remove_var("LLM_TEST_BENCH_BENCHMARKS__PARALLEL_REQUESTS");
    }

    #[test]
    fn test_metrics_list_from_env() {
        env::set_var("LLM_TEST_BENCH_EVALUATION__METRICS", "latency,faithfulness");

        let loader = ConfigLoader::new().skip_default_file();
        let config = loader.load().expect("Failed to load config");

        assert_eq!(config.evaluation.metrics.len(), 2);
        assert!(config.evaluation.metrics.contains(&"latency".to_string()));
        assert!(config.evaluation.metrics.contains(&"faithfulness".to_string()));

        // Cleanup
        env::remove_var("LLM_TEST_BENCH_EVALUATION__METRICS");
    }

    #[test]
    fn test_default_config_dir() {
        let config_dir = ConfigLoader::default_config_dir();
        assert!(config_dir.is_some());

        if let Some(dir) = config_dir {
            assert!(dir.to_string_lossy().contains("llm-test-bench"));
        }
    }

    #[test]
    fn test_default_config_path() {
        let config_path = ConfigLoader::default_config_path();
        assert!(config_path.is_some());

        if let Some(path) = config_path {
            assert!(path.to_string_lossy().ends_with("config.toml"));
        }
    }
}

//! Configuration integration with infra-config.
//!
//! This module provides a bridge between test-bench's existing ConfigLoader
//! and the unified configuration system from infra-config.

use infra_config::{ConfigLoader as InfraConfigLoader, ConfigFormat, ConfigBuilder, EnvSource, FileSource};
use infra_errors::InfraResult;
use serde::de::DeserializeOwned;
use std::path::Path;

/// Environment variable prefix for test-bench configuration
pub const TEST_BENCH_ENV_PREFIX: &str = "LLM_TEST_BENCH";

/// Load test-bench configuration using infra-config
///
/// This function provides a unified way to load configuration that is
/// compatible with both the original test-bench ConfigLoader and the
/// shared infra-config system.
///
/// # Example
///
/// ```rust,ignore
/// use llm_test_bench_core::infra::config::load_config;
/// use llm_test_bench_core::config::Config;
///
/// let config: Config = load_config("config.toml")?;
/// ```
pub fn load_config<T: DeserializeOwned>(path: impl AsRef<Path>) -> InfraResult<T> {
    InfraConfigLoader::new()
        .add_source(FileSource::new(path))
        .add_source(EnvSource::with_prefix(TEST_BENCH_ENV_PREFIX))
        .load()
}

/// Load configuration from environment variables only
pub fn load_from_env<T: DeserializeOwned>() -> InfraResult<T> {
    InfraConfigLoader::new()
        .add_source(EnvSource::with_prefix(TEST_BENCH_ENV_PREFIX))
        .load()
}

/// Load configuration with a custom prefix
pub fn load_with_prefix<T: DeserializeOwned>(
    path: impl AsRef<Path>,
    prefix: &str,
) -> InfraResult<T> {
    InfraConfigLoader::new()
        .add_source(FileSource::new(path))
        .add_source(EnvSource::with_prefix(prefix))
        .load()
}

/// Parse configuration from a string
pub fn parse_config<T: DeserializeOwned>(content: &str, format: ConfigFormat) -> InfraResult<T> {
    infra_config::parse(content, format)
}

/// Create a config builder for programmatic configuration
pub fn builder() -> ConfigBuilder {
    ConfigBuilder::new()
}

/// Configuration format detection based on file extension
pub fn detect_format(path: impl AsRef<Path>) -> ConfigFormat {
    match path.as_ref().extension().and_then(|e| e.to_str()) {
        Some("json") => ConfigFormat::Json,
        Some("toml") | Some("tml") => ConfigFormat::Toml,
        _ => ConfigFormat::Toml, // Default to TOML
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Debug, Deserialize, PartialEq)]
    struct TestConfig {
        name: String,
        port: u16,
    }

    #[test]
    fn test_parse_json() {
        let json = r#"{"name": "test", "port": 8080}"#;
        let config: TestConfig = parse_config(json, ConfigFormat::Json).unwrap();
        assert_eq!(config.name, "test");
        assert_eq!(config.port, 8080);
    }

    #[test]
    fn test_parse_toml() {
        let toml = r#"
name = "test"
port = 8080
"#;
        let config: TestConfig = parse_config(toml, ConfigFormat::Toml).unwrap();
        assert_eq!(config.name, "test");
        assert_eq!(config.port, 8080);
    }

    #[test]
    fn test_detect_format() {
        assert!(matches!(detect_format("config.json"), ConfigFormat::Json));
        assert!(matches!(detect_format("config.toml"), ConfigFormat::Toml));
        assert!(matches!(detect_format("config.tml"), ConfigFormat::Toml));
        assert!(matches!(detect_format("config"), ConfigFormat::Toml)); // default
    }
}

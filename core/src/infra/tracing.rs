//! Tracing and observability integration with infra-otel.
//!
//! This module provides a bridge between test-bench's existing tracing
//! setup and the unified OpenTelemetry system from infra-otel.

use infra_otel::{OtelConfig, init_tracing, init_metrics, shutdown};
use infra_errors::InfraResult;

/// Default service name for test-bench
pub const SERVICE_NAME: &str = "llm-test-bench";

/// Default service version (pulled from Cargo.toml at compile time)
pub const SERVICE_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Initialize tracing with default test-bench configuration
///
/// This sets up OpenTelemetry tracing with sensible defaults for
/// LLM testing and benchmarking workloads.
///
/// # Example
///
/// ```rust,ignore
/// use llm_test_bench_core::infra::tracing::init_default;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     init_default()?;
///
///     // Your application code here
///
///     shutdown_tracing().await;
///     Ok(())
/// }
/// ```
pub fn init_default() -> InfraResult<()> {
    let config = OtelConfig::builder()
        .service_name(SERVICE_NAME)
        .service_version(SERVICE_VERSION)
        .build();

    init_tracing(&config)
}

/// Initialize tracing with a custom service name
pub fn init_with_name(service_name: &str) -> InfraResult<()> {
    let config = OtelConfig::builder()
        .service_name(service_name)
        .service_version(SERVICE_VERSION)
        .build();

    init_tracing(&config)
}

/// Initialize full observability (tracing + metrics)
pub fn init_full(service_name: &str) -> InfraResult<()> {
    let config = OtelConfig::builder()
        .service_name(service_name)
        .service_version(SERVICE_VERSION)
        .build();

    init_tracing(&config)?;
    init_metrics(&config)?;
    Ok(())
}

/// Shutdown the tracing system
///
/// Call this before your application exits to ensure all spans
/// are flushed to the collector.
pub async fn shutdown_tracing() {
    shutdown().await;
}

/// Create a custom OtelConfig for advanced use cases
pub fn custom_config() -> infra_otel::OtelConfig {
    OtelConfig::builder()
        .service_name(SERVICE_NAME)
        .service_version(SERVICE_VERSION)
        .build()
}

/// Span names for test-bench operations
pub mod spans {
    /// Span for provider API calls
    pub const PROVIDER_CALL: &str = "llm.provider.call";
    /// Span for benchmark execution
    pub const BENCHMARK_RUN: &str = "llm.benchmark.run";
    /// Span for evaluation
    pub const EVALUATION: &str = "llm.evaluation";
    /// Span for config loading
    pub const CONFIG_LOAD: &str = "llm.config.load";
    /// Span for response streaming
    pub const RESPONSE_STREAM: &str = "llm.response.stream";
}

/// Attribute keys for test-bench spans
pub mod attributes {
    /// Provider name (e.g., "openai", "anthropic")
    pub const PROVIDER: &str = "llm.provider";
    /// Model name (e.g., "gpt-4", "claude-3")
    pub const MODEL: &str = "llm.model";
    /// Input token count
    pub const INPUT_TOKENS: &str = "llm.tokens.input";
    /// Output token count
    pub const OUTPUT_TOKENS: &str = "llm.tokens.output";
    /// Request latency in milliseconds
    pub const LATENCY_MS: &str = "llm.latency_ms";
    /// Cost in USD
    pub const COST_USD: &str = "llm.cost_usd";
    /// Benchmark name
    pub const BENCHMARK_NAME: &str = "llm.benchmark.name";
    /// Evaluation metric name
    pub const METRIC_NAME: &str = "llm.metric.name";
    /// Evaluation score
    pub const METRIC_SCORE: &str = "llm.metric.score";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_constants() {
        assert_eq!(SERVICE_NAME, "llm-test-bench");
        assert!(!SERVICE_VERSION.is_empty());
    }

    #[test]
    fn test_span_names() {
        assert!(!spans::PROVIDER_CALL.is_empty());
        assert!(!spans::BENCHMARK_RUN.is_empty());
        assert!(!spans::EVALUATION.is_empty());
    }

    #[test]
    fn test_attribute_keys() {
        assert!(!attributes::PROVIDER.is_empty());
        assert!(!attributes::MODEL.is_empty());
        assert!(!attributes::LATENCY_MS.is_empty());
    }
}

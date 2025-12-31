// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Benchmarking logic and reporting
//!
//! This module provides functionality for running benchmarks on LLM providers,
//! including concurrent execution, progress reporting, and result aggregation.
//!
//! # Examples
//!
//! ```no_run
//! use llm_test_bench_core::benchmarks::{BenchmarkRunner, BenchmarkConfig};
//! use llm_test_bench_core::providers::OpenAIProvider;
//! use llm_test_bench_datasets::Dataset;
//! use std::sync::Arc;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a benchmark configuration
//! let config = BenchmarkConfig::new()
//!     .with_concurrency(10)
//!     .with_save_responses(true);
//!
//! // Create a benchmark runner
//! let runner = BenchmarkRunner::new(config);
//!
//! // Load a dataset
//! let dataset = Dataset::new("test".to_string(), "Test dataset".to_string());
//!
//! // Run the benchmark
//! let provider = Arc::new(OpenAIProvider::new("api-key".to_string()));
//! let results = runner.run(&dataset, provider).await?;
//!
//! println!("Success rate: {:.2}%", results.summary.success_rate * 100.0);
//! # Ok(())
//! # }
//! ```

use serde::{Deserialize, Serialize};
use std::time::Duration;
use thiserror::Error;

/// Benchmark errors
#[derive(Error, Debug)]
pub enum BenchmarkError {
    /// Benchmark execution failed
    #[error("Benchmark failed: {0}")]
    ExecutionFailed(String),

    /// Invalid benchmark configuration
    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),
}

/// Benchmark result (legacy - kept for compatibility)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    /// Name of the benchmark
    pub name: String,

    /// Provider name
    pub provider: String,

    /// Model used
    pub model: String,

    /// Latency metrics
    pub latency: LatencyMetrics,

    /// Token usage
    pub token_usage: TokenMetrics,

    /// Success rate (0.0 - 1.0)
    pub success_rate: f64,
}

/// Latency metrics (legacy - kept for compatibility)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyMetrics {
    /// Median latency (P50)
    pub p50: Duration,

    /// 95th percentile latency
    pub p95: Duration,

    /// 99th percentile latency
    pub p99: Duration,

    /// Average latency
    pub mean: Duration,
}

/// Token metrics (legacy - kept for compatibility)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenMetrics {
    /// Total prompt tokens
    pub total_prompt_tokens: u64,

    /// Total completion tokens
    pub total_completion_tokens: u64,

    /// Average tokens per request
    pub avg_tokens_per_request: f64,
}

pub mod config;
pub mod runner;
pub mod reporter;
pub mod results;
pub mod export;
pub mod storage;
pub mod fleet;
pub mod fleet_export;
pub mod fleet_api;
pub mod fleet_manifest;
pub mod fleet_adapters;
pub mod fleet_runner;

pub use config::BenchmarkConfig;
pub use reporter::BenchmarkReporter;
pub use runner::{BenchmarkResults, BenchmarkRunner, ResultSummary, TestResult, TestStatus};
pub use export::CsvExporter;
pub use storage::ResultStorage;
// Re-export the calculate_percentile utility function
pub use results::calculate_percentile;

// Fleet-level exports
pub use fleet::{
    FleetBenchmarkResults, RepositoryResults, FleetSummary,
    ProviderFleetStats, CategoryFleetStats, FleetMetadata,
};
pub use fleet_export::FleetCsvExporter;
pub use fleet_api::{
    FleetBenchmarkAPI, FleetConfig, FleetExecutionHandle, FleetError,
    FleetExecutionMetadata,
};

// Fleet orchestration exports (new manifest-based system)
pub use fleet_manifest::{
    FleetManifest, FleetManifestError, RepositoryConfig,
    ScenarioProfile, OutputConfig, GlobalSettings,
};
pub use fleet_adapters::{
    RepositoryAdapter, AdapterFactory, NativeAdapter, GenericAdapter,
    AdapterError,
};
pub use fleet_runner::{FleetRunner, FleetRunnerError};

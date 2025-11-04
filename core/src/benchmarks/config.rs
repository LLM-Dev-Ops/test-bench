// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Benchmark configuration structures

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Configuration for benchmark execution.
///
/// This structure controls how benchmarks are executed, including concurrency
/// limits, output options, and error handling behavior.
///
/// # Examples
///
/// ```
/// use llm_test_bench_core::benchmarks::BenchmarkConfig;
/// use std::path::PathBuf;
///
/// let config = BenchmarkConfig {
///     concurrency: 10,
///     save_responses: true,
///     output_dir: PathBuf::from("./results"),
///     continue_on_failure: true,
///     random_seed: Some(42),
///     request_delay_ms: Some(100),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkConfig {
    /// Maximum number of concurrent requests.
    ///
    /// Controls how many test cases can be executed in parallel. Higher values
    /// increase throughput but may hit rate limits. Default: 5
    pub concurrency: usize,

    /// Whether to save raw responses to disk.
    ///
    /// If true, each response will be saved to `{output_dir}/{test_id}.json`.
    /// Default: true
    pub save_responses: bool,

    /// Output directory for results and responses.
    ///
    /// All benchmark results and raw responses will be saved here.
    /// Default: "./bench-results"
    pub output_dir: PathBuf,

    /// Whether to continue on failure or stop at first error.
    ///
    /// If true, benchmark execution continues even if individual tests fail.
    /// If false, the entire benchmark stops on the first error. Default: true
    pub continue_on_failure: bool,

    /// Random seed for reproducible test ordering.
    ///
    /// If provided, tests will be shuffled in a deterministic way using this seed.
    /// This is useful for finding order-dependent issues. Default: None (no shuffling)
    pub random_seed: Option<u64>,

    /// Delay between requests in milliseconds.
    ///
    /// Adds a fixed delay between consecutive requests to avoid rate limiting.
    /// Default: None (no delay)
    pub request_delay_ms: Option<u64>,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            concurrency: 5,
            save_responses: true,
            output_dir: PathBuf::from("./bench-results"),
            continue_on_failure: true,
            random_seed: None,
            request_delay_ms: None,
        }
    }
}

impl BenchmarkConfig {
    /// Creates a new benchmark configuration with default values.
    ///
    /// # Examples
    ///
    /// ```
    /// use llm_test_bench_core::benchmarks::BenchmarkConfig;
    ///
    /// let config = BenchmarkConfig::new();
    /// assert_eq!(config.concurrency, 5);
    /// assert!(config.save_responses);
    /// assert!(config.continue_on_failure);
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the concurrency level.
    ///
    /// # Examples
    ///
    /// ```
    /// use llm_test_bench_core::benchmarks::BenchmarkConfig;
    ///
    /// let config = BenchmarkConfig::new().with_concurrency(10);
    /// assert_eq!(config.concurrency, 10);
    /// ```
    pub fn with_concurrency(mut self, concurrency: usize) -> Self {
        self.concurrency = concurrency;
        self
    }

    /// Sets whether to save raw responses.
    ///
    /// # Examples
    ///
    /// ```
    /// use llm_test_bench_core::benchmarks::BenchmarkConfig;
    ///
    /// let config = BenchmarkConfig::new().with_save_responses(false);
    /// assert!(!config.save_responses);
    /// ```
    pub fn with_save_responses(mut self, save: bool) -> Self {
        self.save_responses = save;
        self
    }

    /// Sets the output directory.
    ///
    /// # Examples
    ///
    /// ```
    /// use llm_test_bench_core::benchmarks::BenchmarkConfig;
    /// use std::path::PathBuf;
    ///
    /// let config = BenchmarkConfig::new()
    ///     .with_output_dir(PathBuf::from("./my-results"));
    /// assert_eq!(config.output_dir, PathBuf::from("./my-results"));
    /// ```
    pub fn with_output_dir(mut self, dir: PathBuf) -> Self {
        self.output_dir = dir;
        self
    }

    /// Sets whether to continue on failure.
    ///
    /// # Examples
    ///
    /// ```
    /// use llm_test_bench_core::benchmarks::BenchmarkConfig;
    ///
    /// let config = BenchmarkConfig::new().with_continue_on_failure(false);
    /// assert!(!config.continue_on_failure);
    /// ```
    pub fn with_continue_on_failure(mut self, continue_on_failure: bool) -> Self {
        self.continue_on_failure = continue_on_failure;
        self
    }

    /// Sets the random seed for test ordering.
    ///
    /// # Examples
    ///
    /// ```
    /// use llm_test_bench_core::benchmarks::BenchmarkConfig;
    ///
    /// let config = BenchmarkConfig::new().with_random_seed(42);
    /// assert_eq!(config.random_seed, Some(42));
    /// ```
    pub fn with_random_seed(mut self, seed: u64) -> Self {
        self.random_seed = Some(seed);
        self
    }

    /// Sets the delay between requests.
    ///
    /// # Examples
    ///
    /// ```
    /// use llm_test_bench_core::benchmarks::BenchmarkConfig;
    ///
    /// let config = BenchmarkConfig::new().with_request_delay_ms(100);
    /// assert_eq!(config.request_delay_ms, Some(100));
    /// ```
    pub fn with_request_delay_ms(mut self, delay_ms: u64) -> Self {
        self.request_delay_ms = Some(delay_ms);
        self
    }

    /// Validates the configuration.
    ///
    /// Returns an error if the configuration has invalid values.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Concurrency is 0
    /// - Output directory path is invalid
    pub fn validate(&self) -> Result<(), String> {
        if self.concurrency == 0 {
            return Err("Concurrency must be greater than 0".to_string());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = BenchmarkConfig::default();
        assert_eq!(config.concurrency, 5);
        assert!(config.save_responses);
        assert!(config.continue_on_failure);
        assert_eq!(config.output_dir, PathBuf::from("./bench-results"));
        assert_eq!(config.random_seed, None);
        assert_eq!(config.request_delay_ms, None);
    }

    #[test]
    fn test_new_config() {
        let config = BenchmarkConfig::new();
        assert_eq!(config.concurrency, 5);
    }

    #[test]
    fn test_builder_pattern() {
        let config = BenchmarkConfig::new()
            .with_concurrency(10)
            .with_save_responses(false)
            .with_output_dir(PathBuf::from("./custom"))
            .with_continue_on_failure(false)
            .with_random_seed(42)
            .with_request_delay_ms(100);

        assert_eq!(config.concurrency, 10);
        assert!(!config.save_responses);
        assert_eq!(config.output_dir, PathBuf::from("./custom"));
        assert!(!config.continue_on_failure);
        assert_eq!(config.random_seed, Some(42));
        assert_eq!(config.request_delay_ms, Some(100));
    }

    #[test]
    fn test_validate_valid_config() {
        let config = BenchmarkConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_zero_concurrency() {
        let config = BenchmarkConfig::new().with_concurrency(0);
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_serialization() {
        let config = BenchmarkConfig::new()
            .with_concurrency(10)
            .with_random_seed(42);

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: BenchmarkConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.concurrency, config.concurrency);
        assert_eq!(deserialized.random_seed, config.random_seed);
    }
}

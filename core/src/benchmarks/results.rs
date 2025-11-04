// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Result storage and aggregation for benchmark runs.
//!
//! This module provides comprehensive result tracking and analysis for benchmarks,
//! including individual test results, aggregated statistics, and percentile calculations.
//!
//! # Core Types
//!
//! - [`BenchmarkResults`] - Complete results from a benchmark run
//! - [`TestResult`] - Individual test case result with timing and status
//! - [`ResultSummary`] - Aggregated statistics across all tests
//! - [`TestStatus`] - Outcome of a test (Success, Failure, Timeout, Skipped)
//!
//! # Examples
//!
//! ```
//! use llm_test_bench_core::benchmarks::results::{BenchmarkResults, TestResult, TestStatus};
//! use llm_test_bench_core::providers::{CompletionResponse, TokenUsage, FinishReason};
//! use chrono::Utc;
//!
//! // Create a test result
//! let result = TestResult::success(
//!     "test-1".to_string(),
//!     Some("coding".to_string()),
//!     CompletionResponse {
//!         id: "resp-1".to_string(),
//!         model: "gpt-4".to_string(),
//!         content: "Hello world".to_string(),
//!         usage: TokenUsage::new(10, 5),
//!         finish_reason: FinishReason::Stop,
//!         created_at: Utc::now(),
//!     },
//!     std::time::Duration::from_millis(1234),
//! );
//!
//! // Create benchmark results
//! let mut results = BenchmarkResults::new(
//!     "my-dataset".to_string(),
//!     "openai".to_string(),
//!     vec![result],
//! );
//!
//! // Calculate summary statistics
//! results.calculate_summary();
//! println!("Success rate: {:.2}%", results.summary.success_rate * 100.0);
//! println!("P95 latency: {:.2}ms", results.summary.p95_duration_ms);
//! ```

use crate::providers::CompletionResponse;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Complete results from a benchmark run.
///
/// This structure aggregates all test results along with metadata about the
/// benchmark run and calculated statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResults {
    /// Name of the dataset used for this benchmark
    pub dataset_name: String,

    /// Name of the provider tested (e.g., "openai", "anthropic")
    pub provider_name: String,

    /// Total number of tests in the benchmark
    pub total_tests: usize,

    /// Individual test results
    pub results: Vec<TestResult>,

    /// Timestamp when the benchmark was executed
    pub timestamp: DateTime<Utc>,

    /// Aggregated summary statistics
    pub summary: ResultSummary,
}

impl BenchmarkResults {
    /// Creates a new `BenchmarkResults` with the given parameters.
    ///
    /// The summary is initialized with default values and should be calculated
    /// using [`calculate_summary`](Self::calculate_summary).
    pub fn new(
        dataset_name: String,
        provider_name: String,
        results: Vec<TestResult>,
    ) -> Self {
        let total_tests = results.len();
        Self {
            dataset_name,
            provider_name,
            total_tests,
            results,
            timestamp: Utc::now(),
            summary: ResultSummary::default(),
        }
    }

    /// Calculates and updates the summary statistics based on the test results.
    ///
    /// This method computes:
    /// - Count of each status type (succeeded, failed, timeout, skipped)
    /// - Success rate
    /// - Average duration
    /// - Percentiles (P50, P95, P99)
    /// - Total tokens used
    /// - Estimated cost
    ///
    /// # Examples
    ///
    /// ```
    /// # use llm_test_bench_core::benchmarks::results::{BenchmarkResults, TestResult};
    /// # let results = vec![];
    /// let mut benchmark = BenchmarkResults::new(
    ///     "dataset".to_string(),
    ///     "provider".to_string(),
    ///     results,
    /// );
    /// benchmark.calculate_summary();
    /// println!("Success rate: {:.2}%", benchmark.summary.success_rate * 100.0);
    /// ```
    pub fn calculate_summary(&mut self) {
        let total = self.results.len();
        if total == 0 {
            self.summary = ResultSummary::default();
            return;
        }

        // Count status types
        let mut succeeded = 0;
        let mut failed = 0;
        let mut timeout = 0;
        let mut skipped = 0;

        for result in &self.results {
            match result.status {
                TestStatus::Success => succeeded += 1,
                TestStatus::Failure => failed += 1,
                TestStatus::Timeout => timeout += 1,
                TestStatus::Skipped => skipped += 1,
            }
        }

        // Calculate success rate
        let success_rate = if total > 0 {
            succeeded as f64 / total as f64
        } else {
            0.0
        };

        // Calculate average duration
        let total_duration_ms: u64 = self.results.iter().map(|r| r.duration_ms).sum();
        let avg_duration_ms = if total > 0 {
            total_duration_ms as f64 / total as f64
        } else {
            0.0
        };

        // Calculate percentiles
        let mut durations: Vec<u64> = self.results.iter().map(|r| r.duration_ms).collect();
        let (p50, p95, p99) = if !durations.is_empty() {
            durations.sort_unstable();
            (
                calculate_percentile(&durations, 50.0),
                calculate_percentile(&durations, 95.0),
                calculate_percentile(&durations, 99.0),
            )
        } else {
            (0.0, 0.0, 0.0)
        };

        // Calculate total tokens
        let total_tokens: usize = self
            .results
            .iter()
            .filter_map(|r| r.response.as_ref())
            .map(|resp| resp.usage.total_tokens)
            .sum();

        // Estimate total cost (using average pricing)
        // GPT-4: ~$0.03/1K prompt, ~$0.06/1K completion
        // This is a rough estimate; real pricing varies by model
        let total_cost: f64 = self
            .results
            .iter()
            .filter_map(|r| r.response.as_ref())
            .map(|resp| {
                let prompt_cost = (resp.usage.prompt_tokens as f64 / 1000.0) * 0.03;
                let completion_cost = (resp.usage.completion_tokens as f64 / 1000.0) * 0.06;
                prompt_cost + completion_cost
            })
            .sum();

        self.summary = ResultSummary {
            total,
            succeeded,
            failed,
            timeout,
            skipped,
            success_rate,
            avg_duration_ms,
            p50_duration_ms: p50,
            p95_duration_ms: p95,
            p99_duration_ms: p99,
            total_tokens,
            total_cost,
        };
    }

    /// Returns the number of successful tests.
    pub fn success_count(&self) -> usize {
        self.results
            .iter()
            .filter(|r| matches!(r.status, TestStatus::Success))
            .count()
    }

    /// Returns the number of failed tests.
    pub fn failure_count(&self) -> usize {
        self.results
            .iter()
            .filter(|r| matches!(r.status, TestStatus::Failure))
            .count()
    }

    /// Returns tests that match the given status.
    pub fn filter_by_status(&self, status: TestStatus) -> Vec<&TestResult> {
        self.results.iter().filter(|r| r.status == status).collect()
    }
}

/// Result of an individual test case execution.
///
/// Contains all information about a single test, including its outcome,
/// response data, timing, and any error information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    /// Unique identifier for this test
    pub test_id: String,

    /// Optional category/tag for grouping tests
    pub category: Option<String>,

    /// Status of the test execution
    pub status: TestStatus,

    /// Response from the provider (if successful)
    pub response: Option<CompletionResponse>,

    /// Error message (if failed)
    pub error: Option<String>,

    /// Duration of the test in milliseconds
    pub duration_ms: u64,

    /// Timestamp when the test was executed
    pub timestamp: DateTime<Utc>,
}

impl TestResult {
    /// Creates a successful test result.
    ///
    /// # Examples
    ///
    /// ```
    /// use llm_test_bench_core::benchmarks::results::TestResult;
    /// use llm_test_bench_core::providers::{CompletionResponse, TokenUsage, FinishReason};
    /// use chrono::Utc;
    /// use std::time::Duration;
    ///
    /// let response = CompletionResponse {
    ///     id: "resp-1".to_string(),
    ///     model: "gpt-4".to_string(),
    ///     content: "Hello".to_string(),
    ///     usage: TokenUsage::new(5, 2),
    ///     finish_reason: FinishReason::Stop,
    ///     created_at: Utc::now(),
    /// };
    ///
    /// let result = TestResult::success(
    ///     "test-1".to_string(),
    ///     Some("coding".to_string()),
    ///     response,
    ///     Duration::from_millis(500),
    /// );
    ///
    /// assert!(result.is_success());
    /// ```
    pub fn success(
        test_id: String,
        category: Option<String>,
        response: CompletionResponse,
        duration: Duration,
    ) -> Self {
        Self {
            test_id,
            category,
            status: TestStatus::Success,
            response: Some(response),
            error: None,
            duration_ms: duration.as_millis() as u64,
            timestamp: Utc::now(),
        }
    }

    /// Creates a failed test result.
    ///
    /// # Examples
    ///
    /// ```
    /// use llm_test_bench_core::benchmarks::results::TestResult;
    /// use std::time::Duration;
    ///
    /// let result = TestResult::failure(
    ///     "test-2".to_string(),
    ///     Some("reasoning".to_string()),
    ///     "API rate limit exceeded".to_string(),
    ///     Duration::from_millis(100),
    /// );
    ///
    /// assert!(result.is_failure());
    /// assert_eq!(result.error.as_ref().unwrap(), "API rate limit exceeded");
    /// ```
    pub fn failure(
        test_id: String,
        category: Option<String>,
        error: String,
        duration: Duration,
    ) -> Self {
        Self {
            test_id,
            category,
            status: TestStatus::Failure,
            response: None,
            error: Some(error),
            duration_ms: duration.as_millis() as u64,
            timestamp: Utc::now(),
        }
    }

    /// Creates a timeout test result.
    pub fn timeout(test_id: String, category: Option<String>, duration: Duration) -> Self {
        Self {
            test_id,
            category,
            status: TestStatus::Timeout,
            response: None,
            error: Some("Request timed out".to_string()),
            duration_ms: duration.as_millis() as u64,
            timestamp: Utc::now(),
        }
    }

    /// Creates a skipped test result.
    pub fn skipped(test_id: String, category: Option<String>, reason: String) -> Self {
        Self {
            test_id,
            category,
            status: TestStatus::Skipped,
            response: None,
            error: Some(reason),
            duration_ms: 0,
            timestamp: Utc::now(),
        }
    }

    /// Returns `true` if the test was successful.
    pub fn is_success(&self) -> bool {
        matches!(self.status, TestStatus::Success)
    }

    /// Returns `true` if the test failed.
    pub fn is_failure(&self) -> bool {
        matches!(self.status, TestStatus::Failure)
    }

    /// Returns `true` if the test timed out.
    pub fn is_timeout(&self) -> bool {
        matches!(self.status, TestStatus::Timeout)
    }

    /// Returns `true` if the test was skipped.
    pub fn is_skipped(&self) -> bool {
        matches!(self.status, TestStatus::Skipped)
    }
}

/// Status of a test execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TestStatus {
    /// Test completed successfully
    Success,

    /// Test failed with an error
    Failure,

    /// Test exceeded time limit
    Timeout,

    /// Test was skipped
    Skipped,
}

impl std::fmt::Display for TestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TestStatus::Success => write!(f, "Success"),
            TestStatus::Failure => write!(f, "Failure"),
            TestStatus::Timeout => write!(f, "Timeout"),
            TestStatus::Skipped => write!(f, "Skipped"),
        }
    }
}

/// Aggregated statistics from a benchmark run.
///
/// Contains computed metrics including success rates, latency percentiles,
/// token usage, and estimated costs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultSummary {
    /// Total number of tests
    pub total: usize,

    /// Number of successful tests
    pub succeeded: usize,

    /// Number of failed tests
    pub failed: usize,

    /// Number of tests that timed out
    pub timeout: usize,

    /// Number of tests that were skipped
    pub skipped: usize,

    /// Success rate (0.0 - 1.0)
    pub success_rate: f64,

    /// Average duration across all tests (milliseconds)
    pub avg_duration_ms: f64,

    /// Median latency - 50th percentile (milliseconds)
    pub p50_duration_ms: f64,

    /// 95th percentile latency (milliseconds)
    pub p95_duration_ms: f64,

    /// 99th percentile latency (milliseconds)
    pub p99_duration_ms: f64,

    /// Total tokens used across all tests
    pub total_tokens: usize,

    /// Estimated total cost in USD
    pub total_cost: f64,
}

impl Default for ResultSummary {
    fn default() -> Self {
        Self {
            total: 0,
            succeeded: 0,
            failed: 0,
            timeout: 0,
            skipped: 0,
            success_rate: 0.0,
            avg_duration_ms: 0.0,
            p50_duration_ms: 0.0,
            p95_duration_ms: 0.0,
            p99_duration_ms: 0.0,
            total_tokens: 0,
            total_cost: 0.0,
        }
    }
}

impl ResultSummary {
    /// Returns a formatted string of the summary statistics.
    pub fn display(&self) -> String {
        format!(
            "Total: {}, Success: {}, Failed: {}, Timeout: {}, Skipped: {}\n\
             Success Rate: {:.2}%, Avg Duration: {:.2}ms\n\
             P50: {:.2}ms, P95: {:.2}ms, P99: {:.2}ms\n\
             Total Tokens: {}, Estimated Cost: ${:.4}",
            self.total,
            self.succeeded,
            self.failed,
            self.timeout,
            self.skipped,
            self.success_rate * 100.0,
            self.avg_duration_ms,
            self.p50_duration_ms,
            self.p95_duration_ms,
            self.p99_duration_ms,
            self.total_tokens,
            self.total_cost
        )
    }
}

/// Calculates the percentile value from a sorted list of durations.
///
/// # Arguments
///
/// * `durations` - Sorted vector of duration values in milliseconds
/// * `percentile` - Percentile to calculate (0.0 - 100.0)
///
/// # Returns
///
/// The duration value at the specified percentile.
///
/// # Examples
///
/// ```
/// # use llm_test_bench_core::benchmarks::results::calculate_percentile;
/// let mut durations = vec![100, 200, 300, 400, 500];
/// durations.sort_unstable();
///
/// let p50 = calculate_percentile(&durations, 50.0);
/// assert_eq!(p50, 300.0);
///
/// let p95 = calculate_percentile(&durations, 95.0);
/// assert_eq!(p95, 500.0);
/// ```
pub fn calculate_percentile(durations: &[u64], percentile: f64) -> f64 {
    if durations.is_empty() {
        return 0.0;
    }

    if durations.len() == 1 {
        return durations[0] as f64;
    }

    // Calculate the index for the percentile
    let index = (percentile / 100.0 * (durations.len() - 1) as f64).ceil() as usize;
    let index = index.min(durations.len() - 1);

    durations[index] as f64
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::{FinishReason, TokenUsage};

    fn create_test_response(tokens: (usize, usize)) -> CompletionResponse {
        CompletionResponse {
            id: "test-resp".to_string(),
            model: "test-model".to_string(),
            content: "test content".to_string(),
            usage: TokenUsage::new(tokens.0, tokens.1),
            finish_reason: FinishReason::Stop,
            created_at: Utc::now(),
        }
    }

    #[test]
    fn test_test_result_success() {
        let response = create_test_response((10, 20));
        let result = TestResult::success(
            "test-1".to_string(),
            Some("category".to_string()),
            response.clone(),
            Duration::from_millis(500),
        );

        assert_eq!(result.test_id, "test-1");
        assert_eq!(result.category, Some("category".to_string()));
        assert!(result.is_success());
        assert!(!result.is_failure());
        assert_eq!(result.duration_ms, 500);
        assert!(result.response.is_some());
        assert!(result.error.is_none());
    }

    #[test]
    fn test_test_result_failure() {
        let result = TestResult::failure(
            "test-2".to_string(),
            None,
            "API error".to_string(),
            Duration::from_millis(100),
        );

        assert!(result.is_failure());
        assert!(!result.is_success());
        assert_eq!(result.error, Some("API error".to_string()));
        assert!(result.response.is_none());
    }

    #[test]
    fn test_test_result_timeout() {
        let result = TestResult::timeout(
            "test-3".to_string(),
            Some("slow".to_string()),
            Duration::from_millis(30000),
        );

        assert!(result.is_timeout());
        assert_eq!(result.duration_ms, 30000);
    }

    #[test]
    fn test_test_result_skipped() {
        let result = TestResult::skipped(
            "test-4".to_string(),
            None,
            "Prerequisites not met".to_string(),
        );

        assert!(result.is_skipped());
        assert_eq!(result.error, Some("Prerequisites not met".to_string()));
    }

    #[test]
    fn test_calculate_percentile() {
        let durations = vec![100, 200, 300, 400, 500];

        assert_eq!(calculate_percentile(&durations, 0.0), 100.0);
        assert_eq!(calculate_percentile(&durations, 50.0), 300.0);
        assert_eq!(calculate_percentile(&durations, 95.0), 500.0);
        assert_eq!(calculate_percentile(&durations, 100.0), 500.0);
    }

    #[test]
    fn test_calculate_percentile_empty() {
        let durations: Vec<u64> = vec![];
        assert_eq!(calculate_percentile(&durations, 50.0), 0.0);
    }

    #[test]
    fn test_calculate_percentile_single() {
        let durations = vec![42];
        assert_eq!(calculate_percentile(&durations, 50.0), 42.0);
    }

    #[test]
    fn test_benchmark_results_new() {
        let response = create_test_response((10, 20));
        let result = TestResult::success(
            "test-1".to_string(),
            None,
            response,
            Duration::from_millis(500),
        );

        let results = BenchmarkResults::new(
            "my-dataset".to_string(),
            "openai".to_string(),
            vec![result],
        );

        assert_eq!(results.dataset_name, "my-dataset");
        assert_eq!(results.provider_name, "openai");
        assert_eq!(results.total_tests, 1);
        assert_eq!(results.results.len(), 1);
    }

    #[test]
    fn test_calculate_summary_empty() {
        let mut results = BenchmarkResults::new(
            "dataset".to_string(),
            "provider".to_string(),
            vec![],
        );

        results.calculate_summary();

        assert_eq!(results.summary.total, 0);
        assert_eq!(results.summary.succeeded, 0);
        assert_eq!(results.summary.success_rate, 0.0);
    }

    #[test]
    fn test_calculate_summary_all_success() {
        let results_vec = vec![
            TestResult::success(
                "test-1".to_string(),
                None,
                create_test_response((100, 50)),
                Duration::from_millis(1000),
            ),
            TestResult::success(
                "test-2".to_string(),
                None,
                create_test_response((200, 100)),
                Duration::from_millis(2000),
            ),
            TestResult::success(
                "test-3".to_string(),
                None,
                create_test_response((150, 75)),
                Duration::from_millis(1500),
            ),
        ];

        let mut benchmark =
            BenchmarkResults::new("dataset".to_string(), "provider".to_string(), results_vec);

        benchmark.calculate_summary();

        assert_eq!(benchmark.summary.total, 3);
        assert_eq!(benchmark.summary.succeeded, 3);
        assert_eq!(benchmark.summary.failed, 0);
        assert_eq!(benchmark.summary.success_rate, 1.0);
        assert_eq!(benchmark.summary.avg_duration_ms, 1500.0);
        assert_eq!(benchmark.summary.p50_duration_ms, 1500.0);
        assert_eq!(benchmark.summary.total_tokens, 150 + 300 + 225);
    }

    #[test]
    fn test_calculate_summary_mixed_results() {
        let results_vec = vec![
            TestResult::success(
                "test-1".to_string(),
                None,
                create_test_response((100, 50)),
                Duration::from_millis(1000),
            ),
            TestResult::failure(
                "test-2".to_string(),
                None,
                "Error".to_string(),
                Duration::from_millis(500),
            ),
            TestResult::timeout("test-3".to_string(), None, Duration::from_millis(30000)),
        ];

        let mut benchmark =
            BenchmarkResults::new("dataset".to_string(), "provider".to_string(), results_vec);

        benchmark.calculate_summary();

        assert_eq!(benchmark.summary.total, 3);
        assert_eq!(benchmark.summary.succeeded, 1);
        assert_eq!(benchmark.summary.failed, 1);
        assert_eq!(benchmark.summary.timeout, 1);
        assert!((benchmark.summary.success_rate - 0.333).abs() < 0.01);
    }

    #[test]
    fn test_benchmark_results_filter_by_status() {
        let results_vec = vec![
            TestResult::success(
                "test-1".to_string(),
                None,
                create_test_response((10, 5)),
                Duration::from_millis(100),
            ),
            TestResult::failure(
                "test-2".to_string(),
                None,
                "Error".to_string(),
                Duration::from_millis(50),
            ),
            TestResult::success(
                "test-3".to_string(),
                None,
                create_test_response((20, 10)),
                Duration::from_millis(200),
            ),
        ];

        let benchmark =
            BenchmarkResults::new("dataset".to_string(), "provider".to_string(), results_vec);

        let successes = benchmark.filter_by_status(TestStatus::Success);
        assert_eq!(successes.len(), 2);

        let failures = benchmark.filter_by_status(TestStatus::Failure);
        assert_eq!(failures.len(), 1);
    }

    #[test]
    fn test_result_summary_display() {
        let summary = ResultSummary {
            total: 100,
            succeeded: 95,
            failed: 3,
            timeout: 2,
            skipped: 0,
            success_rate: 0.95,
            avg_duration_ms: 1234.56,
            p50_duration_ms: 1000.0,
            p95_duration_ms: 2000.0,
            p99_duration_ms: 3000.0,
            total_tokens: 50000,
            total_cost: 2.5,
        };

        let display = summary.display();
        assert!(display.contains("Total: 100"));
        assert!(display.contains("Success: 95"));
        assert!(display.contains("95.00%"));
        assert!(display.contains("$2.5000"));
    }

    #[test]
    fn test_test_status_display() {
        assert_eq!(TestStatus::Success.to_string(), "Success");
        assert_eq!(TestStatus::Failure.to_string(), "Failure");
        assert_eq!(TestStatus::Timeout.to_string(), "Timeout");
        assert_eq!(TestStatus::Skipped.to_string(), "Skipped");
    }

    #[test]
    fn test_serialization_test_result() {
        let result = TestResult::success(
            "test-1".to_string(),
            Some("category".to_string()),
            create_test_response((10, 5)),
            Duration::from_millis(500),
        );

        let json = serde_json::to_string(&result).unwrap();
        let deserialized: TestResult = serde_json::from_str(&json).unwrap();

        assert_eq!(result.test_id, deserialized.test_id);
        assert_eq!(result.category, deserialized.category);
        assert_eq!(result.status, deserialized.status);
        assert_eq!(result.duration_ms, deserialized.duration_ms);
    }

    #[test]
    fn test_serialization_benchmark_results() {
        let results_vec = vec![TestResult::success(
            "test-1".to_string(),
            None,
            create_test_response((10, 5)),
            Duration::from_millis(100),
        )];

        let mut benchmark =
            BenchmarkResults::new("dataset".to_string(), "provider".to_string(), results_vec);
        benchmark.calculate_summary();

        let json = serde_json::to_string(&benchmark).unwrap();
        let deserialized: BenchmarkResults = serde_json::from_str(&json).unwrap();

        assert_eq!(benchmark.dataset_name, deserialized.dataset_name);
        assert_eq!(benchmark.provider_name, deserialized.provider_name);
        assert_eq!(benchmark.total_tests, deserialized.total_tests);
    }
}

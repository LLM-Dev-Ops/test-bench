// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Benchmark runner implementation with async execution and progress reporting

use super::config::BenchmarkConfig;
use super::{BenchmarkError, BenchmarkResult};
use crate::providers::{CompletionRequest, CompletionResponse, Provider};
use chrono::{DateTime, Utc};
use futures::stream::{self, StreamExt};
use indicatif::{ProgressBar, ProgressStyle};
use llm_test_bench_datasets::{Dataset, TestCase};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;

/// Result of running all benchmarks in a dataset.
///
/// Contains all individual test results, timing information, and summary statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResults {
    /// Name of the dataset that was executed
    pub dataset_name: String,

    /// Name of the provider used
    pub provider_name: String,

    /// Total number of tests in the dataset
    pub total_tests: usize,

    /// Individual test results
    pub results: Vec<TestResult>,

    /// When the benchmark started
    pub started_at: DateTime<Utc>,

    /// When the benchmark completed
    pub completed_at: DateTime<Utc>,

    /// Total duration of the benchmark
    pub total_duration_ms: u64,

    /// Summary statistics
    pub summary: ResultSummary,
}

impl BenchmarkResults {
    /// Creates a new BenchmarkResults instance.
    ///
    /// This constructor is primarily used for testing purposes.
    /// The summary statistics are initialized with placeholder values
    /// and should be calculated using `calculate_summary()`.
    pub fn new(dataset_name: String, provider_name: String, results: Vec<TestResult>) -> Self {
        let total_tests = results.len();
        let started_at = Utc::now();
        let completed_at = Utc::now();
        let total_duration_ms = 0;

        // Initialize with empty summary
        let summary = ResultSummary {
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
            min_duration_ms: 0,
            max_duration_ms: 0,
            total_tokens: 0,
            avg_tokens_per_request: 0.0,
            total_cost: 0.0,
        };

        Self {
            dataset_name,
            provider_name,
            total_tests,
            results,
            started_at,
            completed_at,
            total_duration_ms,
            summary,
        }
    }

    /// Calculates and updates summary statistics based on the current results.
    ///
    /// This method should be called after creating a BenchmarkResults with `new()`
    /// or after modifying the results vector.
    pub fn calculate_summary(&mut self) {
        self.summary = Self::compute_summary(&self.results);
    }

    /// Computes summary statistics from test results.
    pub(crate) fn compute_summary(results: &[TestResult]) -> ResultSummary {
        use super::results::calculate_percentile;

        let total = results.len();
        let succeeded = results
            .iter()
            .filter(|r| r.status == TestStatus::Success)
            .count();
        let failed = results
            .iter()
            .filter(|r| r.status == TestStatus::Failure)
            .count();
        let timeout = results
            .iter()
            .filter(|r| r.status == TestStatus::Timeout)
            .count();
        let skipped = results
            .iter()
            .filter(|r| r.status == TestStatus::Skipped)
            .count();

        let success_rate = if total > 0 {
            succeeded as f64 / total as f64
        } else {
            0.0
        };

        let mut durations: Vec<u64> = results.iter().map(|r| r.duration_ms).collect();
        let avg_duration_ms = if !durations.is_empty() {
            durations.iter().sum::<u64>() as f64 / durations.len() as f64
        } else {
            0.0
        };

        // Calculate percentiles
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

        let min_duration_ms = durations.iter().min().copied().unwrap_or(0);
        let max_duration_ms = durations.iter().max().copied().unwrap_or(0);

        let total_tokens: usize = results
            .iter()
            .filter_map(|r| r.response.as_ref())
            .map(|resp| resp.usage.total_tokens)
            .sum();

        let avg_tokens_per_request = if succeeded > 0 {
            total_tokens as f64 / succeeded as f64
        } else {
            0.0
        };

        // Estimate total cost (using average pricing)
        // GPT-4: ~$0.03/1K prompt, ~$0.06/1K completion
        let total_cost: f64 = results
            .iter()
            .filter_map(|r| r.response.as_ref())
            .map(|resp| {
                let prompt_cost = (resp.usage.prompt_tokens as f64 / 1000.0) * 0.03;
                let completion_cost = (resp.usage.completion_tokens as f64 / 1000.0) * 0.06;
                prompt_cost + completion_cost
            })
            .sum();

        ResultSummary {
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
            min_duration_ms,
            max_duration_ms,
            total_tokens,
            avg_tokens_per_request,
            total_cost,
        }
    }
}

/// Result of a single test case execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    /// Test case ID
    pub test_id: String,

    /// Test category/tag (if any)
    pub category: Option<String>,

    /// Test execution status
    pub status: TestStatus,

    /// The completion response (if successful)
    pub response: Option<CompletionResponse>,

    /// Error message (if failed)
    pub error: Option<String>,

    /// Duration of the test in milliseconds
    pub duration_ms: u64,

    /// When the test was executed
    pub timestamp: DateTime<Utc>,
}

/// Status of a test execution.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
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
            TestStatus::Success => write!(f, "success"),
            TestStatus::Failure => write!(f, "failure"),
            TestStatus::Timeout => write!(f, "timeout"),
            TestStatus::Skipped => write!(f, "skipped"),
        }
    }
}

/// Summary statistics for a benchmark run.
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

    /// Number of skipped tests
    pub skipped: usize,

    /// Success rate (0.0 to 1.0)
    pub success_rate: f64,

    /// Average duration per test in milliseconds
    pub avg_duration_ms: f64,

    /// Median latency - 50th percentile (milliseconds)
    pub p50_duration_ms: f64,

    /// 95th percentile latency (milliseconds)
    pub p95_duration_ms: f64,

    /// 99th percentile latency (milliseconds)
    pub p99_duration_ms: f64,

    /// Minimum duration in milliseconds
    pub min_duration_ms: u64,

    /// Maximum duration in milliseconds
    pub max_duration_ms: u64,

    /// Total tokens used across all tests
    pub total_tokens: usize,

    /// Average tokens per request
    pub avg_tokens_per_request: f64,

    /// Estimated total cost in USD
    pub total_cost: f64,
}

impl TestResult {
    /// Creates a successful test result.
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
    pub fn failure(test_id: String, category: Option<String>, error: String, duration: Duration) -> Self {
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
    pub fn skipped(test_id: String, category: Option<String>) -> Self {
        Self {
            test_id,
            category,
            status: TestStatus::Skipped,
            response: None,
            error: None,
            duration_ms: 0,
            timestamp: Utc::now(),
        }
    }
}

/// Benchmark runner that executes test cases concurrently with progress reporting.
///
/// # Examples
///
/// ```no_run
/// use llm_test_bench_core::benchmarks::{BenchmarkRunner, BenchmarkConfig};
/// use llm_test_bench_core::providers::OpenAIProvider;
/// use llm_test_bench_datasets::Dataset;
/// use std::sync::Arc;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let config = BenchmarkConfig::new().with_concurrency(10);
/// let runner = BenchmarkRunner::new(config);
///
/// let provider = Arc::new(OpenAIProvider::new("api-key".to_string()));
/// let dataset = Dataset::new("test".to_string(), "Test dataset".to_string());
///
/// let results = runner.run(&dataset, provider).await?;
/// println!("Completed {} tests", results.total_tests);
/// # Ok(())
/// # }
/// ```
pub struct BenchmarkRunner {
    config: BenchmarkConfig,
}

impl BenchmarkRunner {
    /// Creates a new benchmark runner with the given configuration.
    ///
    /// # Examples
    ///
    /// ```
    /// use llm_test_bench_core::benchmarks::{BenchmarkRunner, BenchmarkConfig};
    ///
    /// let config = BenchmarkConfig::new();
    /// let runner = BenchmarkRunner::new(config);
    /// ```
    pub fn new(config: BenchmarkConfig) -> Self {
        // Validate configuration
        if let Err(e) = config.validate() {
            panic!("Invalid benchmark configuration: {}", e);
        }
        Self { config }
    }

    /// Runs the benchmark on the given dataset using the specified provider.
    ///
    /// This method:
    /// - Creates the output directory if needed
    /// - Executes test cases concurrently (respecting concurrency limits)
    /// - Shows progress with a progress bar
    /// - Saves raw responses if configured
    /// - Handles errors according to continue_on_failure setting
    /// - Returns aggregated results with statistics
    ///
    /// # Arguments
    ///
    /// * `dataset` - The dataset containing test cases to run
    /// * `provider` - The LLM provider to use for completions
    ///
    /// # Returns
    ///
    /// Returns `BenchmarkResults` containing all test results and statistics,
    /// or a `BenchmarkError` if the benchmark cannot be executed.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The output directory cannot be created
    /// - All tests fail and continue_on_failure is false
    pub async fn run(
        &self,
        dataset: &Dataset,
        provider: Arc<dyn Provider>,
    ) -> Result<BenchmarkResults, BenchmarkError> {
        let start_time = Instant::now();
        let started_at = Utc::now();

        // Create output directory
        if self.config.save_responses {
            std::fs::create_dir_all(&self.config.output_dir).map_err(|e| {
                BenchmarkError::ExecutionFailed(format!("Failed to create output directory: {}", e))
            })?;
        }

        let total = dataset.test_cases.len();
        if total == 0 {
            return Err(BenchmarkError::InvalidConfiguration(
                "Dataset has no test cases".to_string(),
            ));
        }

        let pb = Self::create_progress_bar(total);

        // Create semaphore for concurrency control
        let semaphore = Arc::new(Semaphore::new(self.config.concurrency));

        // Clone test cases to avoid lifetime issues with async iteration
        let test_cases = dataset.test_cases.clone();

        // Process test cases concurrently
        let results: Vec<TestResult> = stream::iter(test_cases)
            .map(|test_case| {
                let provider = Arc::clone(&provider);
                let semaphore = Arc::clone(&semaphore);
                let pb = pb.clone();
                let config = self.config.clone();

                async move {
                    // Acquire semaphore permit
                    let _permit = semaphore.acquire().await.unwrap();

                    pb.set_message(format!("Testing: {}", test_case.id));

                    // Optional delay between requests
                    if let Some(delay) = config.request_delay_ms {
                        tokio::time::sleep(Duration::from_millis(delay)).await;
                    }

                    // Run test case
                    let result = Self::run_test_case(&test_case, &provider, &config).await;

                    pb.inc(1);
                    result
                }
            })
            .buffer_unordered(self.config.concurrency)
            .collect()
            .await;

        pb.finish_with_message("Benchmark complete");

        let total_duration = start_time.elapsed();
        let completed_at = Utc::now();

        // Calculate summary statistics
        let summary = BenchmarkResults::compute_summary(&results);

        Ok(BenchmarkResults {
            dataset_name: dataset.name.clone(),
            provider_name: provider.name().to_string(),
            total_tests: total,
            results,
            started_at,
            completed_at,
            total_duration_ms: total_duration.as_millis() as u64,
            summary,
        })
    }

    /// Executes a single test case.
    async fn run_test_case(
        test_case: &TestCase,
        provider: &Arc<dyn Provider>,
        config: &BenchmarkConfig,
    ) -> TestResult {
        let start = Instant::now();

        // Build completion request
        let request = CompletionRequest::new(
            // Use default model from provider (can be enhanced later with dataset config)
            provider.supported_models().first().map(|m| m.id.clone()).unwrap_or_else(|| "default".to_string()),
            test_case.prompt.clone(),
        );

        // Execute request
        let result = provider.complete(request).await;
        let duration = start.elapsed();

        match result {
            Ok(response) => {
                // Save raw response if configured
                if config.save_responses {
                    if let Err(e) = Self::save_response(&test_case.id, &response, config) {
                        tracing::warn!("Failed to save response for {}: {}", test_case.id, e);
                    }
                }

                TestResult::success(
                    test_case.id.clone(),
                    test_case.category.clone(),
                    response,
                    duration,
                )
            }
            Err(e) => {
                let error_msg = e.to_string();
                TestResult::failure(
                    test_case.id.clone(),
                    test_case.category.clone(),
                    error_msg,
                    duration,
                )
            }
        }
    }

    /// Creates a progress bar for tracking benchmark execution.
    fn create_progress_bar(total: usize) -> ProgressBar {
        let pb = ProgressBar::new(total as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} {msg}")
                .expect("Failed to create progress bar template")
                .progress_chars("=>-"),
        );
        pb
    }

    /// Saves a response to disk as JSON.
    fn save_response(
        test_id: &str,
        response: &CompletionResponse,
        config: &BenchmarkConfig,
    ) -> std::io::Result<()> {
        let filename = config.output_dir.join(format!("{}.json", test_id));
        let json = serde_json::to_string_pretty(response)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        std::fs::write(filename, json)
    }
}

impl Default for BenchmarkRunner {
    fn default() -> Self {
        Self::new(BenchmarkConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::{FinishReason, ModelInfo, ProviderError, TokenUsage};
    use async_trait::async_trait;
    use std::path::PathBuf;

    // Mock provider for testing
    struct MockProvider {
        name: String,
        should_fail: bool,
    }

    impl MockProvider {
        fn new(name: &str) -> Self {
            Self {
                name: name.to_string(),
                should_fail: false,
            }
        }

        fn with_failures(mut self) -> Self {
            self.should_fail = true;
            self
        }
    }

    #[async_trait]
    impl Provider for MockProvider {
        async fn complete(&self, _request: CompletionRequest) -> Result<CompletionResponse, ProviderError> {
            if self.should_fail {
                return Err(ProviderError::ApiError {
                    status: 500,
                    message: "Mock error".to_string(),
                });
            }

            tokio::time::sleep(Duration::from_millis(10)).await;

            Ok(CompletionResponse {
                id: "mock-123".to_string(),
                model: "mock-model".to_string(),
                content: "Mock response".to_string(),
                usage: TokenUsage::new(10, 20),
                finish_reason: FinishReason::Stop,
                created_at: Utc::now(),
            })
        }

        async fn stream(
            &self,
            _request: CompletionRequest,
        ) -> Result<crate::providers::ResponseStream, ProviderError> {
            unimplemented!("Stream not needed for tests")
        }

        fn supported_models(&self) -> Vec<ModelInfo> {
            vec![ModelInfo::new("mock-model", "Mock Model", 4096, false, false)]
        }

        fn max_context_length(&self, _model: &str) -> Option<usize> {
            Some(4096)
        }

        fn name(&self) -> &str {
            &self.name
        }

        async fn validate_config(&self) -> Result<(), ProviderError> {
            Ok(())
        }

        fn estimate_tokens(&self, text: &str, _model: &str) -> Result<usize, ProviderError> {
            Ok(text.split_whitespace().count())
        }
    }

    fn create_test_dataset(num_cases: usize) -> Dataset {
        let mut dataset = Dataset::new("test".to_string(), "Test dataset".to_string());
        for i in 0..num_cases {
            let tc = TestCase::new(format!("tc-{}", i), format!("Test prompt {}", i));
            dataset.add_test_case(tc);
        }
        dataset
    }

    #[tokio::test]
    async fn test_runner_creation() {
        let config = BenchmarkConfig::new();
        let _runner = BenchmarkRunner::new(config);
    }

    #[tokio::test]
    async fn test_run_benchmark_success() {
        let config = BenchmarkConfig::new()
            .with_concurrency(2)
            .with_save_responses(false);
        let runner = BenchmarkRunner::new(config);

        let dataset = create_test_dataset(5);
        let provider = Arc::new(MockProvider::new("mock"));

        let results = runner.run(&dataset, provider).await.unwrap();

        assert_eq!(results.total_tests, 5);
        assert_eq!(results.results.len(), 5);
        assert_eq!(results.summary.succeeded, 5);
        assert_eq!(results.summary.failed, 0);
        assert_eq!(results.summary.success_rate, 1.0);
    }

    #[tokio::test]
    async fn test_run_benchmark_with_failures() {
        let config = BenchmarkConfig::new()
            .with_concurrency(2)
            .with_save_responses(false)
            .with_continue_on_failure(true);
        let runner = BenchmarkRunner::new(config);

        let dataset = create_test_dataset(3);
        let provider = Arc::new(MockProvider::new("mock").with_failures());

        let results = runner.run(&dataset, provider).await.unwrap();

        assert_eq!(results.total_tests, 3);
        assert_eq!(results.summary.succeeded, 0);
        assert_eq!(results.summary.failed, 3);
        assert_eq!(results.summary.success_rate, 0.0);
    }

    #[tokio::test]
    async fn test_concurrent_execution() {
        let config = BenchmarkConfig::new()
            .with_concurrency(3)
            .with_save_responses(false);
        let runner = BenchmarkRunner::new(config);

        let dataset = create_test_dataset(10);
        let provider = Arc::new(MockProvider::new("mock"));

        let start = Instant::now();
        let results = runner.run(&dataset, provider).await.unwrap();
        let duration = start.elapsed();

        assert_eq!(results.results.len(), 10);

        // With concurrency of 3 and 10ms per test, should take ~40ms
        // (10 tests / 3 concurrent = 4 batches * 10ms = 40ms + overhead)
        // Allow for some overhead
        assert!(duration.as_millis() < 200);
    }

    #[tokio::test]
    async fn test_progress_bar_creation() {
        let pb = BenchmarkRunner::create_progress_bar(100);
        assert_eq!(pb.length().unwrap(), 100);
    }

    #[tokio::test]
    async fn test_summary_calculation() {
        let results = vec![
            TestResult::success(
                "tc1".to_string(),
                Some("test".to_string()),
                CompletionResponse {
                    id: "1".to_string(),
                    model: "mock".to_string(),
                    content: "response".to_string(),
                    usage: TokenUsage::new(10, 20),
                    finish_reason: FinishReason::Stop,
                    created_at: Utc::now(),
                },
                Duration::from_millis(100),
            ),
            TestResult::failure(
                "tc2".to_string(),
                None,
                "Error".to_string(),
                Duration::from_millis(50),
            ),
            TestResult::success(
                "tc3".to_string(),
                None,
                CompletionResponse {
                    id: "3".to_string(),
                    model: "mock".to_string(),
                    content: "response".to_string(),
                    usage: TokenUsage::new(15, 25),
                    finish_reason: FinishReason::Stop,
                    created_at: Utc::now(),
                },
                Duration::from_millis(150),
            ),
        ];

        let summary = BenchmarkResults::compute_summary(&results);

        assert_eq!(summary.total, 3);
        assert_eq!(summary.succeeded, 2);
        assert_eq!(summary.failed, 1);
        assert_eq!(summary.skipped, 0);
        assert!((summary.success_rate - 0.666).abs() < 0.01);
        assert_eq!(summary.avg_duration_ms, 100.0);
        assert_eq!(summary.min_duration_ms, 50);
        assert_eq!(summary.max_duration_ms, 150);
        assert_eq!(summary.total_tokens, 70); // 30 + 40
        assert_eq!(summary.avg_tokens_per_request, 35.0); // 70 / 2
    }

    #[tokio::test]
    async fn test_test_result_success() {
        let response = CompletionResponse {
            id: "test".to_string(),
            model: "mock".to_string(),
            content: "response".to_string(),
            usage: TokenUsage::new(10, 20),
            finish_reason: FinishReason::Stop,
            created_at: Utc::now(),
        };

        let result = TestResult::success(
            "tc1".to_string(),
            Some("category".to_string()),
            response,
            Duration::from_millis(100),
        );

        assert_eq!(result.status, TestStatus::Success);
        assert!(result.response.is_some());
        assert!(result.error.is_none());
        assert_eq!(result.duration_ms, 100);
    }

    #[tokio::test]
    async fn test_test_result_failure() {
        let result = TestResult::failure(
            "tc1".to_string(),
            None,
            "Error message".to_string(),
            Duration::from_millis(50),
        );

        assert_eq!(result.status, TestStatus::Failure);
        assert!(result.response.is_none());
        assert_eq!(result.error, Some("Error message".to_string()));
        assert_eq!(result.duration_ms, 50);
    }

    #[tokio::test]
    async fn test_test_result_skipped() {
        let result = TestResult::skipped("tc1".to_string(), None);

        assert_eq!(result.status, TestStatus::Skipped);
        assert!(result.response.is_none());
        assert!(result.error.is_none());
        assert_eq!(result.duration_ms, 0);
    }

    #[tokio::test]
    async fn test_save_response() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config = BenchmarkConfig::new()
            .with_output_dir(temp_dir.path().to_path_buf())
            .with_save_responses(true);

        let response = CompletionResponse {
            id: "test".to_string(),
            model: "mock".to_string(),
            content: "response".to_string(),
            usage: TokenUsage::new(10, 20),
            finish_reason: FinishReason::Stop,
            created_at: Utc::now(),
        };

        std::fs::create_dir_all(&config.output_dir).unwrap();
        BenchmarkRunner::save_response("test-case-1", &response, &config).unwrap();

        let saved_file = temp_dir.path().join("test-case-1.json");
        assert!(saved_file.exists());

        let content = std::fs::read_to_string(saved_file).unwrap();
        let loaded: CompletionResponse = serde_json::from_str(&content).unwrap();
        assert_eq!(loaded.id, response.id);
    }

    #[tokio::test]
    async fn test_empty_dataset() {
        let config = BenchmarkConfig::new().with_save_responses(false);
        let runner = BenchmarkRunner::new(config);

        let dataset = Dataset::new("empty".to_string(), "Empty dataset".to_string());
        let provider = Arc::new(MockProvider::new("mock"));

        let result = runner.run(&dataset, provider).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_with_request_delay() {
        let config = BenchmarkConfig::new()
            .with_concurrency(1)
            .with_request_delay_ms(50)
            .with_save_responses(false);
        let runner = BenchmarkRunner::new(config);

        let dataset = create_test_dataset(3);
        let provider = Arc::new(MockProvider::new("mock"));

        let start = Instant::now();
        let _results = runner.run(&dataset, provider).await.unwrap();
        let duration = start.elapsed();

        // 3 tests with 50ms delay each + 10ms execution = at least 180ms
        assert!(duration.as_millis() >= 180);
    }

    #[tokio::test]
    async fn test_benchmark_results_serialization() {
        let results = BenchmarkResults {
            dataset_name: "test".to_string(),
            provider_name: "mock".to_string(),
            total_tests: 1,
            results: vec![],
            started_at: Utc::now(),
            completed_at: Utc::now(),
            total_duration_ms: 100,
            summary: ResultSummary {
                total: 1,
                succeeded: 1,
                failed: 0,
                timeout: 0,
                skipped: 0,
                success_rate: 1.0,
                avg_duration_ms: 100.0,
                p50_duration_ms: 100.0,
                p95_duration_ms: 100.0,
                p99_duration_ms: 100.0,
                min_duration_ms: 100,
                max_duration_ms: 100,
                total_tokens: 30,
                avg_tokens_per_request: 30.0,
                total_cost: 0.0,
            },
        };

        let json = serde_json::to_string(&results).unwrap();
        let deserialized: BenchmarkResults = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.dataset_name, results.dataset_name);
    }
}

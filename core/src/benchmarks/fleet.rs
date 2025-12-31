// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Fleet-level benchmarking metrics and aggregation.
//!
//! This module extends the single-repository benchmarking system to support
//! fleet-level aggregation across multiple repositories. It maintains full
//! backward compatibility while adding cross-repository analysis capabilities.
//!
//! # Architecture
//!
//! - **Single-repo metrics**: Existing `BenchmarkResults` schema unchanged
//! - **Fleet metrics**: New `FleetBenchmarkResults` aggregates multiple repos
//! - **Aggregation**: Deterministic formulas for cross-repo statistics
//! - **Output**: CSV summaries, executive reports, deterministic JSON
//!
//! # Examples
//!
//! ```no_run
//! use llm_test_bench_core::benchmarks::fleet::{FleetBenchmarkResults, RepositoryResults};
//! use llm_test_bench_core::benchmarks::BenchmarkResults;
//!
//! # fn example(repo_results: Vec<BenchmarkResults>) -> Result<(), Box<dyn std::error::Error>> {
//! // Aggregate results from multiple repositories
//! let fleet_results = FleetBenchmarkResults::from_repositories(
//!     "my-fleet".to_string(),
//!     repo_results,
//! );
//!
//! // Access fleet-wide statistics
//! println!("Fleet success rate: {:.2}%", fleet_results.fleet_summary.success_rate * 100.0);
//! println!("Total repositories: {}", fleet_results.total_repositories);
//! println!("Average cost per repository: ${:.4}", fleet_results.fleet_summary.avg_cost_per_repository);
//! # Ok(())
//! # }
//! ```

use super::runner::{BenchmarkResults, ResultSummary, TestResult, TestStatus};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Fleet-level benchmark results aggregating multiple repositories.
///
/// This structure provides cross-repository analytics while preserving
/// individual repository results for drill-down analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FleetBenchmarkResults {
    /// Fleet identifier
    pub fleet_id: String,

    /// Timestamp when fleet benchmark was executed
    pub timestamp: DateTime<Utc>,

    /// Total number of repositories in the fleet
    pub total_repositories: usize,

    /// Results for each repository
    pub repository_results: Vec<RepositoryResults>,

    /// Fleet-wide aggregated statistics
    pub fleet_summary: FleetSummary,

    /// Per-provider statistics across the fleet
    pub provider_breakdown: HashMap<String, ProviderFleetStats>,

    /// Per-category statistics across the fleet
    pub category_breakdown: HashMap<String, CategoryFleetStats>,

    /// Fleet metadata
    pub metadata: FleetMetadata,
}

impl FleetBenchmarkResults {
    /// Creates fleet results from multiple repository benchmark results.
    ///
    /// # Arguments
    ///
    /// * `fleet_id` - Identifier for this fleet
    /// * `repo_results` - Benchmark results from each repository
    ///
    /// # Returns
    ///
    /// A new `FleetBenchmarkResults` with aggregated statistics.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use llm_test_bench_core::benchmarks::fleet::FleetBenchmarkResults;
    /// # use llm_test_bench_core::benchmarks::BenchmarkResults;
    /// # fn example(results: Vec<BenchmarkResults>) {
    /// let fleet = FleetBenchmarkResults::from_repositories(
    ///     "production-fleet".to_string(),
    ///     results,
    /// );
    /// # }
    /// ```
    pub fn from_repositories(
        fleet_id: String,
        benchmark_results: Vec<BenchmarkResults>,
    ) -> Self {
        let total_repositories = benchmark_results.len();
        let timestamp = Utc::now();

        // Convert benchmark results to repository results
        let repository_results: Vec<RepositoryResults> = benchmark_results
            .iter()
            .enumerate()
            .map(|(idx, br)| RepositoryResults {
                repository_id: format!("repo-{}", idx),
                repository_name: br.dataset_name.clone(),
                provider_name: br.provider_name.clone(),
                results: br.clone(),
                repository_metadata: HashMap::new(),
            })
            .collect();

        // Aggregate fleet-wide statistics
        let fleet_summary = Self::compute_fleet_summary(&repository_results);

        // Compute per-provider breakdown
        let provider_breakdown = Self::compute_provider_breakdown(&repository_results);

        // Compute per-category breakdown
        let category_breakdown = Self::compute_category_breakdown(&repository_results);

        // Fleet metadata
        let metadata = FleetMetadata {
            total_tests: fleet_summary.total_tests,
            total_duration_ms: fleet_summary.total_duration_ms,
            execution_timestamp: timestamp,
            aggregation_version: "1.0.0".to_string(),
            custom: HashMap::new(),
        };

        Self {
            fleet_id,
            timestamp,
            total_repositories,
            repository_results,
            fleet_summary,
            provider_breakdown,
            category_breakdown,
            metadata,
        }
    }

    /// Computes fleet-wide summary statistics.
    ///
    /// Uses deterministic aggregation formulas to ensure reproducibility.
    fn compute_fleet_summary(repo_results: &[RepositoryResults]) -> FleetSummary {
        use super::results::calculate_percentile;

        let mut all_durations = Vec::new();
        let mut total_tests = 0;
        let mut total_succeeded = 0;
        let mut total_failed = 0;
        let mut total_timeout = 0;
        let mut total_skipped = 0;
        let mut total_tokens = 0;
        let mut total_cost = 0.0;
        let mut total_duration_ms = 0u64;

        for repo in repo_results {
            total_tests += repo.results.summary.total;
            total_succeeded += repo.results.summary.succeeded;
            total_failed += repo.results.summary.failed;
            total_timeout += repo.results.summary.timeout;
            total_skipped += repo.results.summary.skipped;
            total_tokens += repo.results.summary.total_tokens;
            total_cost += repo.results.summary.total_cost;
            total_duration_ms += repo.results.total_duration_ms;

            // Collect all individual test durations for percentile calculation
            for test in &repo.results.results {
                all_durations.push(test.duration_ms);
            }
        }

        let success_rate = if total_tests > 0 {
            total_succeeded as f64 / total_tests as f64
        } else {
            0.0
        };

        let avg_duration_ms = if !all_durations.is_empty() {
            all_durations.iter().sum::<u64>() as f64 / all_durations.len() as f64
        } else {
            0.0
        };

        // Calculate fleet-wide percentiles
        let (p50, p95, p99) = if !all_durations.is_empty() {
            all_durations.sort_unstable();
            (
                calculate_percentile(&all_durations, 50.0),
                calculate_percentile(&all_durations, 95.0),
                calculate_percentile(&all_durations, 99.0),
            )
        } else {
            (0.0, 0.0, 0.0)
        };

        let min_duration_ms = all_durations.iter().min().copied().unwrap_or(0);
        let max_duration_ms = all_durations.iter().max().copied().unwrap_or(0);

        let avg_tokens_per_request = if total_succeeded > 0 {
            total_tokens as f64 / total_succeeded as f64
        } else {
            0.0
        };

        let avg_cost_per_repository = if !repo_results.is_empty() {
            total_cost / repo_results.len() as f64
        } else {
            0.0
        };

        let avg_tests_per_repository = if !repo_results.is_empty() {
            total_tests as f64 / repo_results.len() as f64
        } else {
            0.0
        };

        FleetSummary {
            total_repositories: repo_results.len(),
            total_tests,
            total_succeeded,
            total_failed,
            total_timeout,
            total_skipped,
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
            avg_cost_per_repository,
            avg_tests_per_repository,
            total_duration_ms,
        }
    }

    /// Computes per-provider statistics across the fleet.
    fn compute_provider_breakdown(
        repo_results: &[RepositoryResults],
    ) -> HashMap<String, ProviderFleetStats> {
        let mut provider_stats: HashMap<String, ProviderFleetStats> = HashMap::new();

        for repo in repo_results {
            let provider = &repo.provider_name;
            let stats = provider_stats
                .entry(provider.clone())
                .or_insert_with(|| ProviderFleetStats {
                    provider_name: provider.clone(),
                    repository_count: 0,
                    total_tests: 0,
                    total_succeeded: 0,
                    total_failed: 0,
                    success_rate: 0.0,
                    total_tokens: 0,
                    total_cost: 0.0,
                    avg_duration_ms: 0.0,
                });

            stats.repository_count += 1;
            stats.total_tests += repo.results.summary.total;
            stats.total_succeeded += repo.results.summary.succeeded;
            stats.total_failed += repo.results.summary.failed;
            stats.total_tokens += repo.results.summary.total_tokens;
            stats.total_cost += repo.results.summary.total_cost;
        }

        // Calculate derived metrics
        for stats in provider_stats.values_mut() {
            stats.success_rate = if stats.total_tests > 0 {
                stats.total_succeeded as f64 / stats.total_tests as f64
            } else {
                0.0
            };
        }

        provider_stats
    }

    /// Computes per-category statistics across the fleet.
    fn compute_category_breakdown(
        repo_results: &[RepositoryResults],
    ) -> HashMap<String, CategoryFleetStats> {
        let mut category_stats: HashMap<String, CategoryFleetStats> = HashMap::new();

        for repo in repo_results {
            for test in &repo.results.results {
                if let Some(category) = &test.category {
                    let stats = category_stats
                        .entry(category.clone())
                        .or_insert_with(|| CategoryFleetStats {
                            category_name: category.clone(),
                            total_tests: 0,
                            total_succeeded: 0,
                            total_failed: 0,
                            success_rate: 0.0,
                            avg_duration_ms: 0.0,
                        });

                    stats.total_tests += 1;
                    if test.status == TestStatus::Success {
                        stats.total_succeeded += 1;
                    } else if test.status == TestStatus::Failure {
                        stats.total_failed += 1;
                    }
                }
            }
        }

        // Calculate derived metrics
        for stats in category_stats.values_mut() {
            stats.success_rate = if stats.total_tests > 0 {
                stats.total_succeeded as f64 / stats.total_tests as f64
            } else {
                0.0
            };
        }

        category_stats
    }

    /// Returns the repository with the best success rate.
    pub fn best_repository(&self) -> Option<&RepositoryResults> {
        self.repository_results
            .iter()
            .max_by(|a, b| {
                a.results
                    .summary
                    .success_rate
                    .partial_cmp(&b.results.summary.success_rate)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    }

    /// Returns the repository with the worst success rate.
    pub fn worst_repository(&self) -> Option<&RepositoryResults> {
        self.repository_results
            .iter()
            .min_by(|a, b| {
                a.results
                    .summary
                    .success_rate
                    .partial_cmp(&b.results.summary.success_rate)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    }

    /// Returns repositories with success rate below the threshold.
    pub fn failing_repositories(&self, threshold: f64) -> Vec<&RepositoryResults> {
        self.repository_results
            .iter()
            .filter(|r| r.results.summary.success_rate < threshold)
            .collect()
    }
}

/// Results for a single repository within the fleet.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryResults {
    /// Repository identifier
    pub repository_id: String,

    /// Repository name
    pub repository_name: String,

    /// Provider used for this repository
    pub provider_name: String,

    /// Benchmark results for this repository
    pub results: BenchmarkResults,

    /// Repository-specific metadata
    pub repository_metadata: HashMap<String, String>,
}

/// Fleet-wide summary statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FleetSummary {
    /// Total number of repositories
    pub total_repositories: usize,

    /// Total tests across all repositories
    pub total_tests: usize,

    /// Total successful tests
    pub total_succeeded: usize,

    /// Total failed tests
    pub total_failed: usize,

    /// Total timed out tests
    pub total_timeout: usize,

    /// Total skipped tests
    pub total_skipped: usize,

    /// Overall success rate (0.0 - 1.0)
    pub success_rate: f64,

    /// Average duration across all tests (milliseconds)
    pub avg_duration_ms: f64,

    /// Median latency across all tests (milliseconds)
    pub p50_duration_ms: f64,

    /// 95th percentile latency across all tests (milliseconds)
    pub p95_duration_ms: f64,

    /// 99th percentile latency across all tests (milliseconds)
    pub p99_duration_ms: f64,

    /// Minimum duration across all tests (milliseconds)
    pub min_duration_ms: u64,

    /// Maximum duration across all tests (milliseconds)
    pub max_duration_ms: u64,

    /// Total tokens used across all repositories
    pub total_tokens: usize,

    /// Average tokens per successful request
    pub avg_tokens_per_request: f64,

    /// Total cost across all repositories (USD)
    pub total_cost: f64,

    /// Average cost per repository (USD)
    pub avg_cost_per_repository: f64,

    /// Average number of tests per repository
    pub avg_tests_per_repository: f64,

    /// Total execution duration (milliseconds)
    pub total_duration_ms: u64,
}

/// Per-provider statistics across the fleet.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderFleetStats {
    /// Provider name
    pub provider_name: String,

    /// Number of repositories using this provider
    pub repository_count: usize,

    /// Total tests for this provider
    pub total_tests: usize,

    /// Total successful tests
    pub total_succeeded: usize,

    /// Total failed tests
    pub total_failed: usize,

    /// Success rate for this provider
    pub success_rate: f64,

    /// Total tokens used by this provider
    pub total_tokens: usize,

    /// Total cost for this provider (USD)
    pub total_cost: f64,

    /// Average duration for this provider (milliseconds)
    pub avg_duration_ms: f64,
}

/// Per-category statistics across the fleet.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryFleetStats {
    /// Category name
    pub category_name: String,

    /// Total tests in this category
    pub total_tests: usize,

    /// Total successful tests
    pub total_succeeded: usize,

    /// Total failed tests
    pub total_failed: usize,

    /// Success rate for this category
    pub success_rate: f64,

    /// Average duration for this category (milliseconds)
    pub avg_duration_ms: f64,
}

/// Fleet metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FleetMetadata {
    /// Total tests across the fleet
    pub total_tests: usize,

    /// Total duration across the fleet
    pub total_duration_ms: u64,

    /// When the fleet benchmark was executed
    pub execution_timestamp: DateTime<Utc>,

    /// Aggregation algorithm version (for reproducibility)
    pub aggregation_version: String,

    /// Custom metadata fields
    pub custom: HashMap<String, String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::{CompletionResponse, FinishReason, TokenUsage};
    use std::time::Duration;

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

    fn create_benchmark_results(
        dataset_name: &str,
        provider: &str,
        num_success: usize,
        num_failure: usize,
    ) -> BenchmarkResults {
        let mut results = Vec::new();

        for i in 0..num_success {
            results.push(TestResult::success(
                format!("test-{}", i),
                Some("category".to_string()),
                create_test_response((100, 50)),
                Duration::from_millis(1000),
            ));
        }

        for i in 0..num_failure {
            results.push(TestResult::failure(
                format!("test-fail-{}", i),
                Some("category".to_string()),
                "Error".to_string(),
                Duration::from_millis(500),
            ));
        }

        let mut benchmark =
            BenchmarkResults::new(dataset_name.to_string(), provider.to_string(), results);
        benchmark.calculate_summary();
        benchmark
    }

    #[test]
    fn test_fleet_results_creation() {
        let repo1 = create_benchmark_results("repo1", "openai", 10, 0);
        let repo2 = create_benchmark_results("repo2", "anthropic", 8, 2);

        let fleet = FleetBenchmarkResults::from_repositories(
            "test-fleet".to_string(),
            vec![repo1, repo2],
        );

        assert_eq!(fleet.fleet_id, "test-fleet");
        assert_eq!(fleet.total_repositories, 2);
        assert_eq!(fleet.fleet_summary.total_tests, 20);
        assert_eq!(fleet.fleet_summary.total_succeeded, 18);
        assert_eq!(fleet.fleet_summary.total_failed, 2);
    }

    #[test]
    fn test_fleet_success_rate() {
        let repo1 = create_benchmark_results("repo1", "openai", 9, 1);
        let repo2 = create_benchmark_results("repo2", "openai", 7, 3);

        let fleet = FleetBenchmarkResults::from_repositories(
            "test-fleet".to_string(),
            vec![repo1, repo2],
        );

        assert_eq!(fleet.fleet_summary.success_rate, 0.8); // 16/20
    }

    #[test]
    fn test_provider_breakdown() {
        let repo1 = create_benchmark_results("repo1", "openai", 10, 0);
        let repo2 = create_benchmark_results("repo2", "anthropic", 8, 2);
        let repo3 = create_benchmark_results("repo3", "openai", 5, 5);

        let fleet = FleetBenchmarkResults::from_repositories(
            "test-fleet".to_string(),
            vec![repo1, repo2, repo3],
        );

        assert_eq!(fleet.provider_breakdown.len(), 2);

        let openai_stats = &fleet.provider_breakdown["openai"];
        assert_eq!(openai_stats.repository_count, 2);
        assert_eq!(openai_stats.total_tests, 20);
        assert_eq!(openai_stats.total_succeeded, 15);

        let anthropic_stats = &fleet.provider_breakdown["anthropic"];
        assert_eq!(anthropic_stats.repository_count, 1);
        assert_eq!(anthropic_stats.total_tests, 10);
        assert_eq!(anthropic_stats.total_succeeded, 8);
    }

    #[test]
    fn test_best_and_worst_repository() {
        let repo1 = create_benchmark_results("repo1", "openai", 10, 0); // 100%
        let repo2 = create_benchmark_results("repo2", "anthropic", 5, 5); // 50%
        let repo3 = create_benchmark_results("repo3", "openai", 7, 3); // 70%

        let fleet = FleetBenchmarkResults::from_repositories(
            "test-fleet".to_string(),
            vec![repo1, repo2, repo3],
        );

        let best = fleet.best_repository().unwrap();
        assert_eq!(best.repository_name, "repo1");
        assert_eq!(best.results.summary.success_rate, 1.0);

        let worst = fleet.worst_repository().unwrap();
        assert_eq!(worst.repository_name, "repo2");
        assert_eq!(worst.results.summary.success_rate, 0.5);
    }

    #[test]
    fn test_failing_repositories() {
        let repo1 = create_benchmark_results("repo1", "openai", 10, 0); // 100%
        let repo2 = create_benchmark_results("repo2", "anthropic", 5, 5); // 50%
        let repo3 = create_benchmark_results("repo3", "openai", 7, 3); // 70%

        let fleet = FleetBenchmarkResults::from_repositories(
            "test-fleet".to_string(),
            vec![repo1, repo2, repo3],
        );

        let failing = fleet.failing_repositories(0.8);
        assert_eq!(failing.len(), 2); // repo2 and repo3
    }

    #[test]
    fn test_fleet_metadata() {
        let repo1 = create_benchmark_results("repo1", "openai", 5, 0);

        let fleet =
            FleetBenchmarkResults::from_repositories("test-fleet".to_string(), vec![repo1]);

        assert_eq!(fleet.metadata.total_tests, 5);
        assert_eq!(fleet.metadata.aggregation_version, "1.0.0");
    }

    #[test]
    fn test_empty_fleet() {
        let fleet =
            FleetBenchmarkResults::from_repositories("empty-fleet".to_string(), vec![]);

        assert_eq!(fleet.total_repositories, 0);
        assert_eq!(fleet.fleet_summary.total_tests, 0);
        assert_eq!(fleet.fleet_summary.success_rate, 0.0);
    }

    #[test]
    fn test_cost_aggregation() {
        let repo1 = create_benchmark_results("repo1", "openai", 10, 0);
        let repo2 = create_benchmark_results("repo2", "openai", 10, 0);

        let fleet = FleetBenchmarkResults::from_repositories(
            "test-fleet".to_string(),
            vec![repo1, repo2],
        );

        assert!(fleet.fleet_summary.total_cost > 0.0);
        assert_eq!(
            fleet.fleet_summary.avg_cost_per_repository,
            fleet.fleet_summary.total_cost / 2.0
        );
    }
}

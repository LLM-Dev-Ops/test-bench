// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Integration tests for fleet-level metrics and output generation.

use llm_test_bench_core::benchmarks::fleet::{FleetBenchmarkResults, RepositoryResults};
use llm_test_bench_core::benchmarks::fleet_export::FleetCsvExporter;
use llm_test_bench_core::benchmarks::runner::{BenchmarkResults, TestResult, TestStatus};
use llm_test_bench_core::providers::{CompletionResponse, FinishReason, TokenUsage};
use chrono::Utc;
use std::fs;
use std::time::Duration;
use tempfile::TempDir;

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
            format!("test-success-{}", i),
            Some("category-a".to_string()),
            create_test_response((100, 50)),
            Duration::from_millis(1000 + (i * 100) as u64),
        ));
    }

    for i in 0..num_failure {
        results.push(TestResult::failure(
            format!("test-failure-{}", i),
            Some("category-b".to_string()),
            "Simulated error".to_string(),
            Duration::from_millis(500),
        ));
    }

    let mut benchmark =
        BenchmarkResults::new(dataset_name.to_string(), provider.to_string(), results);
    benchmark.calculate_summary();
    benchmark
}

#[test]
fn test_fleet_aggregation_accuracy() {
    // Create test data with known values
    let repo1 = create_benchmark_results("repo1", "openai", 10, 0); // 100% success
    let repo2 = create_benchmark_results("repo2", "anthropic", 7, 3); // 70% success
    let repo3 = create_benchmark_results("repo3", "openai", 5, 5); // 50% success

    let fleet = FleetBenchmarkResults::from_repositories(
        "test-fleet".to_string(),
        vec![repo1, repo2, repo3],
    );

    // Verify fleet-wide metrics
    assert_eq!(fleet.total_repositories, 3);
    assert_eq!(fleet.fleet_summary.total_tests, 30);
    assert_eq!(fleet.fleet_summary.total_succeeded, 22);
    assert_eq!(fleet.fleet_summary.total_failed, 8);

    // Success rate: 22/30 = 0.7333...
    assert!((fleet.fleet_summary.success_rate - 0.7333).abs() < 0.01);

    // Verify provider breakdown
    assert_eq!(fleet.provider_breakdown.len(), 2);

    let openai_stats = &fleet.provider_breakdown["openai"];
    assert_eq!(openai_stats.repository_count, 2);
    assert_eq!(openai_stats.total_tests, 20);
    assert_eq!(openai_stats.total_succeeded, 15);

    let anthropic_stats = &fleet.provider_breakdown["anthropic"];
    assert_eq!(anthropic_stats.repository_count, 1);
    assert_eq!(anthropic_stats.total_tests, 10);
    assert_eq!(anthropic_stats.total_succeeded, 7);
}

#[test]
fn test_percentile_calculation_across_fleet() {
    // Create repos with known duration distributions
    let repo1 = create_benchmark_results("repo1", "openai", 5, 0);
    let repo2 = create_benchmark_results("repo2", "openai", 5, 0);

    let fleet = FleetBenchmarkResults::from_repositories(
        "test-fleet".to_string(),
        vec![repo1, repo2],
    );

    // All tests have durations: 1000, 1100, 1200, 1300, 1400, 1000, 1100, 1200, 1300, 1400
    // Sorted: 1000, 1000, 1100, 1100, 1200, 1200, 1300, 1300, 1400, 1400

    assert!(fleet.fleet_summary.p50_duration_ms > 1100.0);
    assert!(fleet.fleet_summary.p50_duration_ms < 1300.0);
    assert!(fleet.fleet_summary.p95_duration_ms >= 1400.0);
}

#[test]
fn test_csv_export_all_formats() {
    let repo1 = create_benchmark_results("repo1", "openai", 10, 0);
    let repo2 = create_benchmark_results("repo2", "anthropic", 8, 2);

    let fleet = FleetBenchmarkResults::from_repositories(
        "test-fleet".to_string(),
        vec![repo1, repo2],
    );

    let temp_dir = TempDir::new().unwrap();

    // Export all formats
    let summary_path = temp_dir.path().join("summary.csv");
    let repos_path = temp_dir.path().join("repositories.csv");
    let providers_path = temp_dir.path().join("providers.csv");
    let categories_path = temp_dir.path().join("categories.csv");

    FleetCsvExporter::export_summary(&fleet, &summary_path).unwrap();
    FleetCsvExporter::export_repositories(&fleet, &repos_path).unwrap();
    FleetCsvExporter::export_providers(&fleet, &providers_path).unwrap();
    FleetCsvExporter::export_categories(&fleet, &categories_path).unwrap();

    // Verify files exist
    assert!(summary_path.exists());
    assert!(repos_path.exists());
    assert!(providers_path.exists());
    assert!(categories_path.exists());

    // Verify summary CSV content
    let summary_content = fs::read_to_string(&summary_path).unwrap();
    let summary_lines: Vec<&str> = summary_content.lines().collect();
    assert_eq!(summary_lines.len(), 2); // header + 1 data row

    // Verify repositories CSV content
    let repos_content = fs::read_to_string(&repos_path).unwrap();
    let repos_lines: Vec<&str> = repos_content.lines().collect();
    assert_eq!(repos_lines.len(), 3); // header + 2 repo rows

    // Verify providers CSV content
    let providers_content = fs::read_to_string(&providers_path).unwrap();
    let providers_lines: Vec<&str> = providers_content.lines().collect();
    assert_eq!(providers_lines.len(), 3); // header + 2 provider rows

    // Verify categories CSV content
    let categories_content = fs::read_to_string(&categories_path).unwrap();
    assert!(categories_content.contains("category-a"));
    assert!(categories_content.contains("category-b"));
}

#[test]
fn test_executive_report_generation() {
    let repo1 = create_benchmark_results("api-gateway", "openai", 10, 0);
    let repo2 = create_benchmark_results("user-service", "anthropic", 7, 3);
    let repo3 = create_benchmark_results("payment-service", "openai", 3, 7); // Failing repo

    let fleet = FleetBenchmarkResults::from_repositories(
        "production-fleet".to_string(),
        vec![repo1, repo2, repo3],
    );

    let temp_dir = TempDir::new().unwrap();
    let report_path = temp_dir.path().join("executive_report.html");

    FleetCsvExporter::export_executive_report(&fleet, &report_path).unwrap();

    assert!(report_path.exists());

    let html_content = fs::read_to_string(&report_path).unwrap();

    // Verify HTML structure
    assert!(html_content.contains("<!DOCTYPE html>"));
    assert!(html_content.contains("Fleet Benchmark Report"));
    assert!(html_content.contains("production-fleet"));

    // Verify metrics are present
    assert!(html_content.contains("Total Repositories"));
    assert!(html_content.contains("Success Rate"));
    assert!(html_content.contains("Total Cost"));

    // Verify provider breakdown is present
    assert!(html_content.contains("Provider Breakdown"));
    assert!(html_content.contains("openai"));
    assert!(html_content.contains("anthropic"));

    // Verify best/worst comparison
    assert!(html_content.contains("Repository Comparison"));
    assert!(html_content.contains("api-gateway")); // Best
    assert!(html_content.contains("payment-service")); // Worst

    // Verify failing repositories section
    assert!(html_content.contains("Repositories Below 90%"));
    assert!(html_content.contains("payment-service"));
}

#[test]
fn test_deterministic_json_output() {
    let repo1 = create_benchmark_results("repo1", "openai", 5, 0);
    let repo2 = create_benchmark_results("repo2", "anthropic", 5, 0);

    // Create fleet results twice with same input
    let fleet1 = FleetBenchmarkResults::from_repositories(
        "test-fleet".to_string(),
        vec![repo1.clone(), repo2.clone()],
    );

    let fleet2 = FleetBenchmarkResults::from_repositories(
        "test-fleet".to_string(),
        vec![repo1.clone(), repo2.clone()],
    );

    let temp_dir = TempDir::new().unwrap();
    let json_path1 = temp_dir.path().join("fleet1.json");
    let json_path2 = temp_dir.path().join("fleet2.json");

    FleetCsvExporter::export_deterministic_json(&fleet1, &json_path1).unwrap();
    FleetCsvExporter::export_deterministic_json(&fleet2, &json_path2).unwrap();

    let json1 = fs::read_to_string(&json_path1).unwrap();
    let json2 = fs::read_to_string(&json_path2).unwrap();

    // JSON should be identical (deterministic)
    // Note: Timestamps might differ slightly, so we check structure instead
    let parsed1: serde_json::Value = serde_json::from_str(&json1).unwrap();
    let parsed2: serde_json::Value = serde_json::from_str(&json2).unwrap();

    assert_eq!(parsed1["fleet_id"], parsed2["fleet_id"]);
    assert_eq!(parsed1["total_repositories"], parsed2["total_repositories"]);
    assert_eq!(parsed1["fleet_summary"]["total_tests"], parsed2["fleet_summary"]["total_tests"]);
}

#[test]
fn test_backward_compatibility() {
    // Verify existing BenchmarkResults still works as before
    let results = create_benchmark_results("test-repo", "openai", 10, 0);

    // All existing fields should be accessible
    assert_eq!(results.dataset_name, "test-repo");
    assert_eq!(results.provider_name, "openai");
    assert_eq!(results.total_tests, 10);
    assert_eq!(results.summary.total, 10);
    assert_eq!(results.summary.succeeded, 10);
    assert_eq!(results.summary.success_rate, 1.0);

    // Can still be used in fleet context
    let fleet = FleetBenchmarkResults::from_repositories(
        "fleet".to_string(),
        vec![results],
    );

    assert_eq!(fleet.total_repositories, 1);
}

#[test]
fn test_cost_aggregation() {
    let repo1 = create_benchmark_results("repo1", "openai", 10, 0);
    let repo2 = create_benchmark_results("repo2", "openai", 10, 0);

    let fleet = FleetBenchmarkResults::from_repositories(
        "test-fleet".to_string(),
        vec![repo1, repo2],
    );

    // Each successful test uses 100 prompt + 50 completion = 150 tokens
    // Cost per test: (100 / 1000 * 0.03) + (50 / 1000 * 0.06) = 0.003 + 0.003 = 0.006
    // Total: 20 tests * 0.006 = 0.12

    assert!((fleet.fleet_summary.total_cost - 0.12).abs() < 0.001);
    assert_eq!(
        fleet.fleet_summary.avg_cost_per_repository,
        fleet.fleet_summary.total_cost / 2.0
    );
}

#[test]
fn test_empty_fleet() {
    let fleet = FleetBenchmarkResults::from_repositories(
        "empty-fleet".to_string(),
        vec![],
    );

    assert_eq!(fleet.total_repositories, 0);
    assert_eq!(fleet.fleet_summary.total_tests, 0);
    assert_eq!(fleet.fleet_summary.success_rate, 0.0);
    assert_eq!(fleet.provider_breakdown.len(), 0);
    assert_eq!(fleet.category_breakdown.len(), 0);

    // Should still be exportable
    let temp_dir = TempDir::new().unwrap();
    let csv_path = temp_dir.path().join("empty.csv");

    FleetCsvExporter::export_summary(&fleet, &csv_path).unwrap();
    assert!(csv_path.exists());
}

#[test]
fn test_large_fleet_performance() {
    // Create a fleet with many repositories
    let mut repos = Vec::new();

    for i in 0..100 {
        repos.push(create_benchmark_results(
            &format!("repo-{}", i),
            if i % 2 == 0 { "openai" } else { "anthropic" },
            10,
            0,
        ));
    }

    let start = std::time::Instant::now();
    let fleet = FleetBenchmarkResults::from_repositories(
        "large-fleet".to_string(),
        repos,
    );
    let duration = start.elapsed();

    // Aggregation should be fast (< 1 second for 100 repos)
    assert!(duration.as_secs() < 1);

    assert_eq!(fleet.total_repositories, 100);
    assert_eq!(fleet.fleet_summary.total_tests, 1000);
}

#[test]
fn test_category_aggregation() {
    let mut results1 = Vec::new();
    results1.push(TestResult::success(
        "test-1".to_string(),
        Some("coding".to_string()),
        create_test_response((100, 50)),
        Duration::from_millis(1000),
    ));
    results1.push(TestResult::success(
        "test-2".to_string(),
        Some("reasoning".to_string()),
        create_test_response((100, 50)),
        Duration::from_millis(1000),
    ));

    let mut results2 = Vec::new();
    results2.push(TestResult::success(
        "test-3".to_string(),
        Some("coding".to_string()),
        create_test_response((100, 50)),
        Duration::from_millis(1000),
    ));
    results2.push(TestResult::failure(
        "test-4".to_string(),
        Some("reasoning".to_string()),
        "Error".to_string(),
        Duration::from_millis(500),
    ));

    let mut bench1 = BenchmarkResults::new("repo1".to_string(), "openai".to_string(), results1);
    bench1.calculate_summary();

    let mut bench2 = BenchmarkResults::new("repo2".to_string(), "openai".to_string(), results2);
    bench2.calculate_summary();

    let fleet = FleetBenchmarkResults::from_repositories(
        "test-fleet".to_string(),
        vec![bench1, bench2],
    );

    // Verify category breakdown
    assert_eq!(fleet.category_breakdown.len(), 2);

    let coding_stats = &fleet.category_breakdown["coding"];
    assert_eq!(coding_stats.total_tests, 2);
    assert_eq!(coding_stats.total_succeeded, 2);
    assert_eq!(coding_stats.success_rate, 1.0);

    let reasoning_stats = &fleet.category_breakdown["reasoning"];
    assert_eq!(reasoning_stats.total_tests, 2);
    assert_eq!(reasoning_stats.total_succeeded, 1);
    assert_eq!(reasoning_stats.success_rate, 0.5);
}

// ========== COMPREHENSIVE VALIDATION TESTS ==========

/// Scenario A: Single repository, multiple providers
#[test]
fn test_scenario_a_single_repo_multiple_providers() {
    // Run same dataset against multiple providers
    let dataset_name = "common-repo";
    let openai_results = create_benchmark_results(dataset_name, "openai", 10, 0);
    let anthropic_results = create_benchmark_results(dataset_name, "anthropic", 9, 1);
    let cohere_results = create_benchmark_results(dataset_name, "cohere", 8, 2);

    // Each provider gets its own fleet entry
    let fleet = FleetBenchmarkResults::from_repositories(
        "multi-provider-fleet".to_string(),
        vec![openai_results, anthropic_results, cohere_results],
    );

    // Verify we have 3 different providers
    assert_eq!(fleet.provider_breakdown.len(), 3);
    assert!(fleet.provider_breakdown.contains_key("openai"));
    assert!(fleet.provider_breakdown.contains_key("anthropic"));
    assert!(fleet.provider_breakdown.contains_key("cohere"));

    // Total tests: 30 (10 per provider)
    assert_eq!(fleet.fleet_summary.total_tests, 30);

    // Success: 10 + 9 + 8 = 27 out of 30
    assert_eq!(fleet.fleet_summary.total_succeeded, 27);
    assert!((fleet.fleet_summary.success_rate - 0.9).abs() < 0.001);

    // Each provider should have 1 repository
    for (_, stats) in &fleet.provider_breakdown {
        assert_eq!(stats.repository_count, 1);
    }
}

/// Scenario B: Multiple repositories, single provider
#[test]
fn test_scenario_b_multiple_repos_single_provider() {
    // Multiple repos all using the same provider
    let repo1 = create_benchmark_results("frontend-tests", "openai", 10, 0);
    let repo2 = create_benchmark_results("backend-tests", "openai", 15, 5);
    let repo3 = create_benchmark_results("integration-tests", "openai", 8, 2);
    let repo4 = create_benchmark_results("e2e-tests", "openai", 12, 3);

    let fleet = FleetBenchmarkResults::from_repositories(
        "openai-only-fleet".to_string(),
        vec![repo1, repo2, repo3, repo4],
    );

    // Only one provider
    assert_eq!(fleet.provider_breakdown.len(), 1);
    assert!(fleet.provider_breakdown.contains_key("openai"));

    // 4 repositories
    assert_eq!(fleet.total_repositories, 4);

    // OpenAI stats should include all repositories
    let openai_stats = &fleet.provider_breakdown["openai"];
    assert_eq!(openai_stats.repository_count, 4);
    assert_eq!(openai_stats.total_tests, 55); // 10 + 20 + 10 + 15
    assert_eq!(openai_stats.total_succeeded, 45); // 10 + 15 + 8 + 12

    // Verify average tests per repository
    let expected_avg = 55.0 / 4.0;
    assert!((fleet.fleet_summary.avg_tests_per_repository - expected_avg).abs() < 0.01);
}

/// Scenario C: Full fleet (multiple repos, multiple providers)
#[test]
fn test_scenario_c_full_fleet() {
    let mut repos = vec![];

    // Mix of providers across different repositories
    repos.push(create_benchmark_results("api-gateway", "openai", 10, 0));
    repos.push(create_benchmark_results("user-service", "anthropic", 15, 0));
    repos.push(create_benchmark_results("auth-service", "openai", 8, 2));
    repos.push(create_benchmark_results("payment-service", "cohere", 12, 3));
    repos.push(create_benchmark_results("notification-service", "anthropic", 10, 5));
    repos.push(create_benchmark_results("analytics-service", "openai", 20, 0));

    let fleet = FleetBenchmarkResults::from_repositories(
        "production-fleet".to_string(),
        repos,
    );

    // Verify fleet-wide metrics
    assert_eq!(fleet.total_repositories, 6);
    assert_eq!(fleet.provider_breakdown.len(), 3);

    // OpenAI: 3 repos (10+0, 8+2, 20+0) = 38 tests, 38 success
    let openai = &fleet.provider_breakdown["openai"];
    assert_eq!(openai.repository_count, 3);
    assert_eq!(openai.total_tests, 40);
    assert_eq!(openai.total_succeeded, 38);

    // Anthropic: 2 repos (15+0, 10+5) = 30 tests, 25 success
    let anthropic = &fleet.provider_breakdown["anthropic"];
    assert_eq!(anthropic.repository_count, 2);
    assert_eq!(anthropic.total_tests, 30);
    assert_eq!(anthropic.total_succeeded, 25);

    // Cohere: 1 repo (12+3) = 15 tests, 12 success
    let cohere = &fleet.provider_breakdown["cohere"];
    assert_eq!(cohere.repository_count, 1);
    assert_eq!(cohere.total_tests, 15);
    assert_eq!(cohere.total_succeeded, 12);

    // Total: 85 tests, 75 success
    assert_eq!(fleet.fleet_summary.total_tests, 85);
    assert_eq!(fleet.fleet_summary.total_succeeded, 75);
    assert!((fleet.fleet_summary.success_rate - 0.882).abs() < 0.01);
}

/// Scenario D: Error handling - empty manifests and edge cases
#[test]
fn test_scenario_d_error_handling() {
    // Test 1: Empty fleet
    let empty_fleet = FleetBenchmarkResults::from_repositories(
        "empty-fleet".to_string(),
        vec![],
    );

    assert_eq!(empty_fleet.total_repositories, 0);
    assert_eq!(empty_fleet.fleet_summary.total_tests, 0);
    assert_eq!(empty_fleet.fleet_summary.success_rate, 0.0);
    assert_eq!(empty_fleet.fleet_summary.avg_cost_per_repository, 0.0);

    // Test 2: Repository with all failures
    let all_failures = create_benchmark_results("failing-repo", "openai", 0, 10);
    let fleet_with_failures = FleetBenchmarkResults::from_repositories(
        "failure-fleet".to_string(),
        vec![all_failures],
    );

    assert_eq!(fleet_with_failures.fleet_summary.total_succeeded, 0);
    assert_eq!(fleet_with_failures.fleet_summary.total_failed, 10);
    assert_eq!(fleet_with_failures.fleet_summary.success_rate, 0.0);

    // Test 3: Mixed success/failure rates
    let perfect = create_benchmark_results("perfect-repo", "openai", 10, 0);
    let failing = create_benchmark_results("failing-repo", "openai", 0, 10);
    let mixed = create_benchmark_results("mixed-repo", "openai", 5, 5);

    let mixed_fleet = FleetBenchmarkResults::from_repositories(
        "mixed-fleet".to_string(),
        vec![perfect, failing, mixed],
    );

    // 15 success, 15 failure = 50% success rate
    assert_eq!(mixed_fleet.fleet_summary.total_tests, 30);
    assert_eq!(mixed_fleet.fleet_summary.total_succeeded, 15);
    assert_eq!(mixed_fleet.fleet_summary.total_failed, 15);
    assert!((mixed_fleet.fleet_summary.success_rate - 0.5).abs() < 0.001);
}

/// Test backward compatibility of BenchmarkResults schema
#[test]
fn test_benchmark_results_backward_compatibility() {
    // Create a traditional benchmark result
    let results = create_benchmark_results("test-dataset", "openai", 5, 0);

    // All existing fields should be accessible
    assert_eq!(results.dataset_name, "test-dataset");
    assert_eq!(results.provider_name, "openai");
    assert_eq!(results.total_tests, 5);
    assert_eq!(results.results.len(), 5);

    // Summary fields
    assert_eq!(results.summary.total, 5);
    assert_eq!(results.summary.succeeded, 5);
    assert_eq!(results.summary.failed, 0);
    assert_eq!(results.summary.success_rate, 1.0);

    // Ensure it can be serialized/deserialized
    let json = serde_json::to_string(&results).unwrap();
    let deserialized: BenchmarkResults = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.dataset_name, results.dataset_name);
    assert_eq!(deserialized.provider_name, results.provider_name);
    assert_eq!(deserialized.total_tests, results.total_tests);
}

/// Test that fleet results can be serialized and deserialized
#[test]
fn test_fleet_results_serialization() {
    let repo1 = create_benchmark_results("repo1", "openai", 10, 0);
    let repo2 = create_benchmark_results("repo2", "anthropic", 8, 2);

    let fleet = FleetBenchmarkResults::from_repositories(
        "test-fleet".to_string(),
        vec![repo1, repo2],
    );

    // Serialize to JSON
    let json = serde_json::to_string(&fleet).unwrap();

    // Deserialize back
    let deserialized: FleetBenchmarkResults = serde_json::from_str(&json).unwrap();

    // Verify integrity
    assert_eq!(deserialized.fleet_id, fleet.fleet_id);
    assert_eq!(deserialized.total_repositories, fleet.total_repositories);
    assert_eq!(deserialized.fleet_summary.total_tests, fleet.fleet_summary.total_tests);
    assert_eq!(deserialized.fleet_summary.success_rate, fleet.fleet_summary.success_rate);
}

/// Test deterministic run identifiers
#[test]
fn test_deterministic_run_identifiers() {
    let repo1 = create_benchmark_results("repo1", "openai", 5, 0);
    let repo2 = create_benchmark_results("repo2", "anthropic", 5, 0);

    // Create two fleets with the same data
    let fleet1 = FleetBenchmarkResults::from_repositories(
        "fleet-id-123".to_string(),
        vec![repo1.clone(), repo2.clone()],
    );

    let fleet2 = FleetBenchmarkResults::from_repositories(
        "fleet-id-123".to_string(),
        vec![repo1.clone(), repo2.clone()],
    );

    // Fleet IDs should be identical
    assert_eq!(fleet1.fleet_id, fleet2.fleet_id);

    // Summary statistics should be identical
    assert_eq!(fleet1.fleet_summary.total_tests, fleet2.fleet_summary.total_tests);
    assert_eq!(fleet1.fleet_summary.total_succeeded, fleet2.fleet_summary.total_succeeded);
    assert_eq!(fleet1.fleet_summary.success_rate, fleet2.fleet_summary.success_rate);

    // Metadata version should be consistent
    assert_eq!(fleet1.metadata.aggregation_version, "1.0.0");
    assert_eq!(fleet2.metadata.aggregation_version, "1.0.0");
}

/// Test artifact generation (JSON, CSV, HTML)
#[test]
fn test_artifact_generation() {
    let repo1 = create_benchmark_results("repo1", "openai", 10, 0);
    let repo2 = create_benchmark_results("repo2", "anthropic", 8, 2);
    let repo3 = create_benchmark_results("repo3", "cohere", 5, 5);

    let fleet = FleetBenchmarkResults::from_repositories(
        "artifact-test-fleet".to_string(),
        vec![repo1, repo2, repo3],
    );

    let temp_dir = TempDir::new().unwrap();

    // Generate all artifact types
    let json_path = temp_dir.path().join("fleet_results.json");
    let summary_csv = temp_dir.path().join("fleet_summary.csv");
    let repos_csv = temp_dir.path().join("repositories.csv");
    let providers_csv = temp_dir.path().join("providers.csv");
    let categories_csv = temp_dir.path().join("categories.csv");
    let html_report = temp_dir.path().join("executive_report.html");

    // Export all formats
    FleetCsvExporter::export_deterministic_json(&fleet, &json_path).unwrap();
    FleetCsvExporter::export_summary(&fleet, &summary_csv).unwrap();
    FleetCsvExporter::export_repositories(&fleet, &repos_csv).unwrap();
    FleetCsvExporter::export_providers(&fleet, &providers_csv).unwrap();
    FleetCsvExporter::export_categories(&fleet, &categories_csv).unwrap();
    FleetCsvExporter::export_executive_report(&fleet, &html_report).unwrap();

    // Verify all files exist
    assert!(json_path.exists());
    assert!(summary_csv.exists());
    assert!(repos_csv.exists());
    assert!(providers_csv.exists());
    assert!(categories_csv.exists());
    assert!(html_report.exists());

    // Verify JSON can be parsed back
    let json_content = fs::read_to_string(&json_path).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json_content).unwrap();
    assert_eq!(parsed["fleet_id"], "artifact-test-fleet");
    assert_eq!(parsed["total_repositories"], 3);

    // Verify CSV has correct number of rows
    let repos_content = fs::read_to_string(&repos_csv).unwrap();
    let repos_lines: Vec<&str> = repos_content.lines().collect();
    assert_eq!(repos_lines.len(), 4); // header + 3 repos

    // Verify HTML has required sections
    let html_content = fs::read_to_string(&html_report).unwrap();
    assert!(html_content.contains("<!DOCTYPE html>"));
    assert!(html_content.contains("artifact-test-fleet"));
    assert!(html_content.contains("Provider Breakdown"));
    assert!(html_content.contains("Repository Comparison"));
}

/// Test concurrent execution simulation
#[test]
fn test_concurrent_execution_metrics() {
    // Simulate results from concurrent benchmark runs
    let mut repos = vec![];

    for i in 0..10 {
        repos.push(create_benchmark_results(
            &format!("concurrent-repo-{}", i),
            if i % 2 == 0 { "openai" } else { "anthropic" },
            10,
            0,
        ));
    }

    let start = std::time::Instant::now();
    let fleet = FleetBenchmarkResults::from_repositories(
        "concurrent-fleet".to_string(),
        repos,
    );
    let aggregation_time = start.elapsed();

    // Aggregation should be fast even with multiple repos
    assert!(aggregation_time.as_millis() < 100);

    // Verify all repositories were processed
    assert_eq!(fleet.total_repositories, 10);
    assert_eq!(fleet.fleet_summary.total_tests, 100);

    // Verify provider distribution
    assert_eq!(fleet.provider_breakdown.len(), 2);
    assert_eq!(fleet.provider_breakdown["openai"].repository_count, 5);
    assert_eq!(fleet.provider_breakdown["anthropic"].repository_count, 5);
}

/// Test large fleet performance (10+ repositories)
#[test]
fn test_large_fleet_scaling() {
    let mut repos = vec![];

    // Create 50 repositories with varying success rates
    for i in 0..50 {
        let success_count = 10 - (i % 3); // Vary success: 10, 9, 8, 10, 9, 8, ...
        let failure_count = i % 3;

        repos.push(create_benchmark_results(
            &format!("large-repo-{:03}", i),
            match i % 4 {
                0 => "openai",
                1 => "anthropic",
                2 => "cohere",
                _ => "huggingface",
            },
            success_count,
            failure_count,
        ));
    }

    let start = std::time::Instant::now();
    let fleet = FleetBenchmarkResults::from_repositories(
        "large-fleet".to_string(),
        repos,
    );
    let aggregation_time = start.elapsed();

    // Aggregation should complete in reasonable time
    assert!(aggregation_time.as_secs() < 2);

    // Verify scale
    assert_eq!(fleet.total_repositories, 50);
    assert_eq!(fleet.fleet_summary.total_tests, 500);

    // Verify provider distribution
    assert_eq!(fleet.provider_breakdown.len(), 4);

    // Test export performance
    let temp_dir = TempDir::new().unwrap();
    let csv_path = temp_dir.path().join("large_fleet.csv");

    let export_start = std::time::Instant::now();
    FleetCsvExporter::export_repositories(&fleet, &csv_path).unwrap();
    let export_time = export_start.elapsed();

    assert!(export_time.as_millis() < 500);
    assert!(csv_path.exists());

    let content = fs::read_to_string(&csv_path).unwrap();
    let lines: Vec<&str> = content.lines().collect();
    assert_eq!(lines.len(), 51); // header + 50 repos
}

/// Test provider trait contract unchanged
#[test]
fn test_provider_trait_contract() {
    // This test verifies that the Provider trait contract is unchanged
    // by attempting to use providers in the same way as before fleet support

    use crate::providers::{CompletionResponse, TokenUsage};

    // Create a mock response (simulating provider output)
    let response = CompletionResponse {
        id: "test-id".to_string(),
        model: "gpt-4".to_string(),
        content: "test response".to_string(),
        usage: TokenUsage::new(100, 50),
        finish_reason: FinishReason::Stop,
        created_at: Utc::now(),
    };

    // Verify response structure unchanged
    assert_eq!(response.id, "test-id");
    assert_eq!(response.model, "gpt-4");
    assert_eq!(response.content, "test response");
    assert_eq!(response.usage.prompt_tokens, 100);
    assert_eq!(response.usage.completion_tokens, 50);
    assert_eq!(response.usage.total_tokens, 150);

    // Verify it can be used in test results
    let test_result = TestResult::success(
        "provider-test".to_string(),
        Some("contract-test".to_string()),
        response,
        Duration::from_millis(1000),
    );

    assert_eq!(test_result.status, TestStatus::Success);
    assert_eq!(test_result.test_name, "provider-test");
}

/// Test metrics calculation consistency
#[test]
fn test_metrics_calculation_consistency() {
    // Create test data with known values
    let repo = create_benchmark_results("metrics-test", "openai", 10, 0);

    // Verify single-repo metrics
    assert_eq!(repo.summary.total, 10);
    assert_eq!(repo.summary.succeeded, 10);
    assert_eq!(repo.summary.success_rate, 1.0);

    // Create fleet and verify metrics are consistent
    let fleet = FleetBenchmarkResults::from_repositories(
        "metrics-fleet".to_string(),
        vec![repo.clone()],
    );

    assert_eq!(fleet.fleet_summary.total_tests, repo.summary.total);
    assert_eq!(fleet.fleet_summary.total_succeeded, repo.summary.succeeded);
    assert_eq!(fleet.fleet_summary.success_rate, repo.summary.success_rate);

    // Token counts should match
    assert_eq!(fleet.fleet_summary.total_tokens, repo.summary.total_tokens);

    // Cost should match
    assert!((fleet.fleet_summary.total_cost - repo.summary.total_cost).abs() < 0.0001);
}

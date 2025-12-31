// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Mock simulator integration tests for fleet benchmarking.
//!
//! These tests validate the integration between the fleet benchmarking system
//! and the LLM Dev Ops simulator WITHOUT modifying the simulator itself.
//! Instead, we create a mock simulator client that demonstrates the expected
//! interaction patterns.

use llm_test_bench_core::benchmarks::fleet::{FleetBenchmarkResults, FleetCsvExporter};
use llm_test_bench_core::benchmarks::runner::{BenchmarkResults, TestResult};
use llm_test_bench_core::providers::{CompletionResponse, FinishReason, TokenUsage};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;
use tempfile::TempDir;

/// Mock simulator client that demonstrates the programmatic API.
///
/// This represents how the LLM Dev Ops simulator would interact with
/// the fleet benchmarking system via programmatic invocation.
#[derive(Debug)]
pub struct MockSimulatorClient {
    workspace_dir: PathBuf,
    run_history: Vec<SimulatorRun>,
}

/// Represents a single simulator run with its metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulatorRun {
    pub run_id: String,
    pub fleet_id: String,
    pub timestamp: String,
    pub status: RunStatus,
    pub artifact_paths: ArtifactPaths,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RunStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

/// Paths to all generated artifacts for a simulator run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactPaths {
    pub json: PathBuf,
    pub summary_csv: PathBuf,
    pub repositories_csv: PathBuf,
    pub providers_csv: PathBuf,
    pub categories_csv: PathBuf,
    pub executive_html: PathBuf,
}

impl MockSimulatorClient {
    /// Creates a new mock simulator client.
    pub fn new(workspace_dir: PathBuf) -> Self {
        Self {
            workspace_dir,
            run_history: Vec::new(),
        }
    }

    /// Invokes a fleet benchmark run programmatically.
    ///
    /// This simulates how the simulator would trigger fleet benchmarks
    /// and receive deterministic run identifiers and artifact paths.
    pub fn invoke_fleet_benchmark(
        &mut self,
        fleet_id: String,
        benchmark_results: Vec<BenchmarkResults>,
    ) -> Result<SimulatorRun, String> {
        // Generate deterministic run ID
        let run_id = self.generate_run_id(&fleet_id);

        // Create fleet results
        let fleet_results = FleetBenchmarkResults::from_repositories(
            fleet_id.clone(),
            benchmark_results,
        );

        // Create output directory for this run
        let run_dir = self.workspace_dir.join(&run_id);
        std::fs::create_dir_all(&run_dir)
            .map_err(|e| format!("Failed to create run directory: {}", e))?;

        // Generate all artifacts
        let artifact_paths = self.generate_artifacts(&fleet_results, &run_dir)?;

        // Create simulator run record
        let simulator_run = SimulatorRun {
            run_id: run_id.clone(),
            fleet_id: fleet_id.clone(),
            timestamp: Utc::now().to_rfc3339(),
            status: RunStatus::Completed,
            artifact_paths,
            metadata: HashMap::from([
                ("total_repositories".to_string(), fleet_results.total_repositories.to_string()),
                ("total_tests".to_string(), fleet_results.fleet_summary.total_tests.to_string()),
                ("success_rate".to_string(), format!("{:.4}", fleet_results.fleet_summary.success_rate)),
            ]),
        };

        // Store in run history
        self.run_history.push(simulator_run.clone());

        Ok(simulator_run)
    }

    /// Generates a deterministic run ID from fleet ID and timestamp.
    fn generate_run_id(&self, fleet_id: &str) -> String {
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let run_number = self.run_history.len() + 1;
        format!("{}_run_{:04}_{}", fleet_id, run_number, timestamp)
    }

    /// Generates all output artifacts for a fleet benchmark.
    fn generate_artifacts(
        &self,
        fleet_results: &FleetBenchmarkResults,
        run_dir: &PathBuf,
    ) -> Result<ArtifactPaths, String> {
        let json_path = run_dir.join("fleet_results.json");
        let summary_csv = run_dir.join("fleet_summary.csv");
        let repositories_csv = run_dir.join("repositories.csv");
        let providers_csv = run_dir.join("providers.csv");
        let categories_csv = run_dir.join("categories.csv");
        let executive_html = run_dir.join("executive_report.html");

        // Export all formats
        FleetCsvExporter::export_deterministic_json(fleet_results, &json_path)
            .map_err(|e| format!("Failed to export JSON: {}", e))?;
        FleetCsvExporter::export_summary(fleet_results, &summary_csv)
            .map_err(|e| format!("Failed to export summary CSV: {}", e))?;
        FleetCsvExporter::export_repositories(fleet_results, &repositories_csv)
            .map_err(|e| format!("Failed to export repositories CSV: {}", e))?;
        FleetCsvExporter::export_providers(fleet_results, &providers_csv)
            .map_err(|e| format!("Failed to export providers CSV: {}", e))?;
        FleetCsvExporter::export_categories(fleet_results, &categories_csv)
            .map_err(|e| format!("Failed to export categories CSV: {}", e))?;
        FleetCsvExporter::export_executive_report(fleet_results, &executive_html)
            .map_err(|e| format!("Failed to export executive report: {}", e))?;

        Ok(ArtifactPaths {
            json: json_path,
            summary_csv,
            repositories_csv,
            providers_csv,
            categories_csv,
            executive_html,
        })
    }

    /// Retrieves a previous run by ID.
    pub fn get_run(&self, run_id: &str) -> Option<&SimulatorRun> {
        self.run_history.iter().find(|run| run.run_id == run_id)
    }

    /// Lists all runs for a specific fleet.
    pub fn list_runs_for_fleet(&self, fleet_id: &str) -> Vec<&SimulatorRun> {
        self.run_history
            .iter()
            .filter(|run| run.fleet_id == fleet_id)
            .collect()
    }

    /// Validates that all artifacts for a run exist and are readable.
    pub fn validate_run_artifacts(&self, run_id: &str) -> Result<bool, String> {
        let run = self.get_run(run_id)
            .ok_or_else(|| format!("Run not found: {}", run_id))?;

        let paths = vec![
            &run.artifact_paths.json,
            &run.artifact_paths.summary_csv,
            &run.artifact_paths.repositories_csv,
            &run.artifact_paths.providers_csv,
            &run.artifact_paths.categories_csv,
            &run.artifact_paths.executive_html,
        ];

        for path in paths {
            if !path.exists() {
                return Err(format!("Missing artifact: {:?}", path));
            }
            if !path.is_file() {
                return Err(format!("Artifact is not a file: {:?}", path));
            }
        }

        Ok(true)
    }
}

// ========== HELPER FUNCTIONS ==========

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
            Some("category".to_string()),
            create_test_response((100, 50)),
            Duration::from_millis(1000),
        ));
    }

    for i in 0..num_failure {
        results.push(TestResult::failure(
            format!("test-failure-{}", i),
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

// ========== INTEGRATION TESTS ==========

#[test]
fn test_simulator_programmatic_invocation() {
    let temp_dir = TempDir::new().unwrap();
    let mut simulator = MockSimulatorClient::new(temp_dir.path().to_path_buf());

    // Prepare test data
    let results = vec![
        create_benchmark_results("repo1", "openai", 10, 0),
        create_benchmark_results("repo2", "anthropic", 8, 2),
    ];

    // Invoke fleet benchmark via programmatic API
    let run = simulator
        .invoke_fleet_benchmark("test-fleet".to_string(), results)
        .expect("Fleet benchmark invocation failed");

    // Verify run metadata
    assert!(run.run_id.starts_with("test-fleet_run_"));
    assert_eq!(run.fleet_id, "test-fleet");
    assert_eq!(run.status, RunStatus::Completed);

    // Verify metadata
    assert_eq!(run.metadata.get("total_repositories"), Some(&"2".to_string()));
    assert_eq!(run.metadata.get("total_tests"), Some(&"20".to_string()));

    // Verify run is in history
    let retrieved_run = simulator.get_run(&run.run_id);
    assert!(retrieved_run.is_some());
    assert_eq!(retrieved_run.unwrap().run_id, run.run_id);
}

#[test]
fn test_simulator_deterministic_run_ids() {
    let temp_dir = TempDir::new().unwrap();
    let mut simulator = MockSimulatorClient::new(temp_dir.path().to_path_buf());

    // Create multiple runs for the same fleet
    let results = vec![create_benchmark_results("repo1", "openai", 5, 0)];

    let run1 = simulator
        .invoke_fleet_benchmark("my-fleet".to_string(), results.clone())
        .unwrap();

    let run2 = simulator
        .invoke_fleet_benchmark("my-fleet".to_string(), results.clone())
        .unwrap();

    // Run IDs should be unique but follow predictable pattern
    assert_ne!(run1.run_id, run2.run_id);
    assert!(run1.run_id.starts_with("my-fleet_run_0001_"));
    assert!(run2.run_id.starts_with("my-fleet_run_0002_"));

    // Both should be in history
    let fleet_runs = simulator.list_runs_for_fleet("my-fleet");
    assert_eq!(fleet_runs.len(), 2);
}

#[test]
fn test_simulator_artifact_paths() {
    let temp_dir = TempDir::new().unwrap();
    let mut simulator = MockSimulatorClient::new(temp_dir.path().to_path_buf());

    let results = vec![
        create_benchmark_results("repo1", "openai", 10, 0),
        create_benchmark_results("repo2", "anthropic", 10, 0),
    ];

    let run = simulator
        .invoke_fleet_benchmark("artifact-test".to_string(), results)
        .unwrap();

    // Verify all artifact paths are set
    assert!(run.artifact_paths.json.exists());
    assert!(run.artifact_paths.summary_csv.exists());
    assert!(run.artifact_paths.repositories_csv.exists());
    assert!(run.artifact_paths.providers_csv.exists());
    assert!(run.artifact_paths.categories_csv.exists());
    assert!(run.artifact_paths.executive_html.exists());

    // Validate artifacts using simulator client
    let validation = simulator.validate_run_artifacts(&run.run_id);
    assert!(validation.is_ok());
    assert_eq!(validation.unwrap(), true);
}

#[test]
fn test_simulator_artifact_content() {
    let temp_dir = TempDir::new().unwrap();
    let mut simulator = MockSimulatorClient::new(temp_dir.path().to_path_buf());

    let results = vec![
        create_benchmark_results("repo1", "openai", 10, 0),
        create_benchmark_results("repo2", "anthropic", 8, 2),
    ];

    let run = simulator
        .invoke_fleet_benchmark("content-test".to_string(), results)
        .unwrap();

    // Verify JSON content
    let json_content = std::fs::read_to_string(&run.artifact_paths.json).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json_content).unwrap();
    assert_eq!(parsed["fleet_id"], "content-test");
    assert_eq!(parsed["total_repositories"], 2);

    // Verify CSV content
    let csv_content = std::fs::read_to_string(&run.artifact_paths.repositories_csv).unwrap();
    let csv_lines: Vec<&str> = csv_content.lines().collect();
    assert_eq!(csv_lines.len(), 3); // header + 2 repos

    // Verify HTML content
    let html_content = std::fs::read_to_string(&run.artifact_paths.executive_html).unwrap();
    assert!(html_content.contains("<!DOCTYPE html>"));
    assert!(html_content.contains("content-test"));
    assert!(html_content.contains("Fleet Benchmark Report"));
}

#[test]
fn test_simulator_can_consume_outputs_without_changes() {
    // This test demonstrates that the simulator can consume fleet outputs
    // without any modifications to its own codebase

    let temp_dir = TempDir::new().unwrap();
    let mut simulator = MockSimulatorClient::new(temp_dir.path().to_path_buf());

    let results = vec![
        create_benchmark_results("service-a", "openai", 10, 0),
        create_benchmark_results("service-b", "anthropic", 15, 5),
        create_benchmark_results("service-c", "cohere", 8, 2),
    ];

    let run = simulator
        .invoke_fleet_benchmark("production".to_string(), results)
        .unwrap();

    // Simulate the simulator reading and processing the JSON output
    let json_content = std::fs::read_to_string(&run.artifact_paths.json).unwrap();
    let fleet_data: serde_json::Value = serde_json::from_str(&json_content).unwrap();

    // Extract metrics the simulator would care about
    let total_repos = fleet_data["total_repositories"].as_u64().unwrap();
    let total_tests = fleet_data["fleet_summary"]["total_tests"].as_u64().unwrap();
    let success_rate = fleet_data["fleet_summary"]["success_rate"].as_f64().unwrap();

    assert_eq!(total_repos, 3);
    assert_eq!(total_tests, 35); // 10 + 20 + 10
    assert!((success_rate - 0.8857).abs() < 0.01); // 31/35

    // Verify the simulator can parse provider breakdown
    let provider_breakdown = fleet_data["provider_breakdown"].as_object().unwrap();
    assert!(provider_breakdown.contains_key("openai"));
    assert!(provider_breakdown.contains_key("anthropic"));
    assert!(provider_breakdown.contains_key("cohere"));
}

#[test]
fn test_simulator_multiple_fleet_runs() {
    let temp_dir = TempDir::new().unwrap();
    let mut simulator = MockSimulatorClient::new(temp_dir.path().to_path_buf());

    // Run multiple different fleets
    let fleet1_results = vec![create_benchmark_results("repo1", "openai", 5, 0)];
    let fleet2_results = vec![create_benchmark_results("repo2", "anthropic", 5, 0)];
    let fleet3_results = vec![create_benchmark_results("repo3", "cohere", 5, 0)];

    let run1 = simulator.invoke_fleet_benchmark("fleet-a".to_string(), fleet1_results).unwrap();
    let run2 = simulator.invoke_fleet_benchmark("fleet-b".to_string(), fleet2_results).unwrap();
    let run3 = simulator.invoke_fleet_benchmark("fleet-c".to_string(), fleet3_results).unwrap();

    // Verify all runs are tracked
    assert_eq!(simulator.run_history.len(), 3);

    // Verify each fleet has its own runs
    assert_eq!(simulator.list_runs_for_fleet("fleet-a").len(), 1);
    assert_eq!(simulator.list_runs_for_fleet("fleet-b").len(), 1);
    assert_eq!(simulator.list_runs_for_fleet("fleet-c").len(), 1);

    // Verify run IDs are unique
    assert_ne!(run1.run_id, run2.run_id);
    assert_ne!(run2.run_id, run3.run_id);
    assert_ne!(run1.run_id, run3.run_id);
}

#[test]
fn test_simulator_error_handling() {
    let temp_dir = TempDir::new().unwrap();
    let simulator = MockSimulatorClient::new(temp_dir.path().to_path_buf());

    // Test retrieving non-existent run
    let result = simulator.get_run("non-existent-run");
    assert!(result.is_none());

    // Test validating non-existent run
    let validation = simulator.validate_run_artifacts("non-existent-run");
    assert!(validation.is_err());
    assert!(validation.unwrap_err().contains("Run not found"));
}

#[test]
fn test_simulator_run_metadata_accuracy() {
    let temp_dir = TempDir::new().unwrap();
    let mut simulator = MockSimulatorClient::new(temp_dir.path().to_path_buf());

    let results = vec![
        create_benchmark_results("repo1", "openai", 10, 0),
        create_benchmark_results("repo2", "openai", 7, 3),
        create_benchmark_results("repo3", "anthropic", 5, 5),
    ];

    let run = simulator
        .invoke_fleet_benchmark("metadata-test".to_string(), results)
        .unwrap();

    // Verify metadata accuracy
    assert_eq!(run.metadata.get("total_repositories").unwrap(), "3");
    assert_eq!(run.metadata.get("total_tests").unwrap(), "30");

    let success_rate: f64 = run.metadata.get("success_rate").unwrap().parse().unwrap();
    // 22 success out of 30 = 0.7333
    assert!((success_rate - 0.7333).abs() < 0.01);
}

#[test]
fn test_simulator_backward_compatibility_with_single_repo() {
    // Verify that the simulator can still process single-repository results
    // (backward compatibility with existing workflows)

    let temp_dir = TempDir::new().unwrap();
    let mut simulator = MockSimulatorClient::new(temp_dir.path().to_path_buf());

    // Single repository (existing use case)
    let results = vec![create_benchmark_results("single-repo", "openai", 10, 0)];

    let run = simulator
        .invoke_fleet_benchmark("single-repo-fleet".to_string(), results)
        .unwrap();

    assert_eq!(run.metadata.get("total_repositories").unwrap(), "1");
    assert_eq!(run.metadata.get("total_tests").unwrap(), "10");
    assert_eq!(run.metadata.get("success_rate").unwrap(), "1.0000");

    // Verify artifacts are still generated correctly
    let validation = simulator.validate_run_artifacts(&run.run_id).unwrap();
    assert_eq!(validation, true);
}

# Simulator Integration Guide

## Overview

This guide demonstrates how the LLM Dev Ops simulator can integrate with the fleet benchmarking system **without any code modifications** to the simulator itself.

## Integration Pattern

### 1. Programmatic API

The fleet benchmarking system provides a simple programmatic API:

```rust
use llm_test_bench_core::benchmarks::fleet::{FleetBenchmarkResults, FleetCsvExporter};
use llm_test_bench_core::benchmarks::runner::BenchmarkResults;

// Collect benchmark results from multiple repositories
let benchmark_results: Vec<BenchmarkResults> = vec![
    repo1_results,
    repo2_results,
    repo3_results,
];

// Aggregate into fleet results
let fleet_results = FleetBenchmarkResults::from_repositories(
    "my-fleet-id".to_string(),
    benchmark_results,
);

// Generate artifacts
FleetCsvExporter::export_deterministic_json(&fleet_results, &json_path)?;
FleetCsvExporter::export_summary(&fleet_results, &summary_csv)?;
FleetCsvExporter::export_executive_report(&fleet_results, &html_path)?;
```

### 2. Input Format

**Input**: `Vec<BenchmarkResults>`

Each `BenchmarkResults` is the standard output from existing single-repository benchmarks:

```rust
BenchmarkResults {
    dataset_name: String,      // Repository/dataset identifier
    provider_name: String,     // LLM provider used
    total_tests: usize,        // Number of tests
    results: Vec<TestResult>,  // Individual test results
    summary: ResultSummary,    // Aggregated metrics
    // ... (existing fields unchanged)
}
```

**Compatibility**: This is the **same format** already used by the simulator. No changes needed.

### 3. Output Artifacts

The system generates 6 artifact types:

1. **fleet_results.json** - Complete fleet data (machine-readable)
2. **fleet_summary.csv** - Single-row summary (spreadsheet-friendly)
3. **repositories.csv** - Per-repository details (drill-down)
4. **providers.csv** - Per-provider breakdown (comparison)
5. **categories.csv** - Per-category statistics (analysis)
6. **executive_report.html** - Visual dashboard (executives)

### 4. Deterministic Run IDs

Generate predictable run identifiers:

```rust
let run_id = format!("{}_run_{:04}_{}",
    fleet_id,
    run_number,
    timestamp.format("%Y%m%d_%H%M%S")
);

// Example: "production-fleet_run_0001_20251231_120000"
```

**Benefits**:
- Sortable chronologically
- Easy to track in logs
- Predictable for automation

## Integration Steps for Simulator

### Step 1: Collect Repository Results

The simulator already collects `BenchmarkResults` for each repository. No changes needed:

```rust
// Existing simulator code (no changes)
let repo1_results = run_benchmark(&repo1_config).await?;
let repo2_results = run_benchmark(&repo2_config).await?;
let repo3_results = run_benchmark(&repo3_config).await?;
```

### Step 2: Aggregate Fleet Results

Add fleet aggregation (new code, minimal):

```rust
// New: Aggregate into fleet
let fleet_results = FleetBenchmarkResults::from_repositories(
    fleet_config.fleet_id.clone(),
    vec![repo1_results, repo2_results, repo3_results],
);
```

### Step 3: Generate Artifacts

Export all artifact types:

```rust
// New: Export artifacts
let run_dir = workspace.join(&run_id);
std::fs::create_dir_all(&run_dir)?;

FleetCsvExporter::export_deterministic_json(
    &fleet_results,
    &run_dir.join("fleet_results.json")
)?;

FleetCsvExporter::export_summary(
    &fleet_results,
    &run_dir.join("fleet_summary.csv")
)?;

FleetCsvExporter::export_repositories(
    &fleet_results,
    &run_dir.join("repositories.csv")
)?;

FleetCsvExporter::export_providers(
    &fleet_results,
    &run_dir.join("providers.csv")
)?;

FleetCsvExporter::export_categories(
    &fleet_results,
    &run_dir.join("categories.csv")
)?;

FleetCsvExporter::export_executive_report(
    &fleet_results,
    &run_dir.join("executive_report.html")
)?;
```

### Step 4: Store Run Metadata

Track the run in simulator database (existing pattern):

```rust
let run_metadata = SimulatorRun {
    run_id: run_id.clone(),
    fleet_id: fleet_config.fleet_id.clone(),
    timestamp: Utc::now().to_rfc3339(),
    status: RunStatus::Completed,
    total_repositories: fleet_results.total_repositories,
    total_tests: fleet_results.fleet_summary.total_tests,
    success_rate: fleet_results.fleet_summary.success_rate,
    artifact_dir: run_dir,
};

simulator_db.store_run(run_metadata)?;
```

## Example: Complete Integration

```rust
use llm_test_bench_core::benchmarks::fleet::{FleetBenchmarkResults, FleetCsvExporter};
use llm_test_bench_core::benchmarks::runner::BenchmarkResults;
use std::path::PathBuf;
use chrono::Utc;

pub async fn run_fleet_benchmark(
    fleet_id: String,
    repositories: Vec<RepositoryConfig>,
    workspace: PathBuf,
) -> Result<FleetRunResult, SimulatorError> {
    // Step 1: Run benchmarks for each repository (existing code)
    let mut benchmark_results = Vec::new();

    for repo_config in repositories {
        let result = run_single_repo_benchmark(&repo_config).await?;
        benchmark_results.push(result);
    }

    // Step 2: Aggregate into fleet results (new, 1 line)
    let fleet_results = FleetBenchmarkResults::from_repositories(
        fleet_id.clone(),
        benchmark_results,
    );

    // Step 3: Generate run ID and directory (new, 3 lines)
    let run_id = generate_run_id(&fleet_id);
    let run_dir = workspace.join(&run_id);
    std::fs::create_dir_all(&run_dir)?;

    // Step 4: Export all artifacts (new, 6 lines)
    FleetCsvExporter::export_deterministic_json(&fleet_results, &run_dir.join("fleet_results.json"))?;
    FleetCsvExporter::export_summary(&fleet_results, &run_dir.join("fleet_summary.csv"))?;
    FleetCsvExporter::export_repositories(&fleet_results, &run_dir.join("repositories.csv"))?;
    FleetCsvExporter::export_providers(&fleet_results, &run_dir.join("providers.csv"))?;
    FleetCsvExporter::export_categories(&fleet_results, &run_dir.join("categories.csv"))?;
    FleetCsvExporter::export_executive_report(&fleet_results, &run_dir.join("executive_report.html"))?;

    // Step 5: Return result (new, existing pattern)
    Ok(FleetRunResult {
        run_id,
        fleet_id,
        total_repositories: fleet_results.total_repositories,
        success_rate: fleet_results.fleet_summary.success_rate,
        artifact_dir: run_dir,
    })
}

fn generate_run_id(fleet_id: &str) -> String {
    let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
    format!("{}_{}", fleet_id, timestamp)
}
```

**Total New Code**: ~15 lines
**Changes to Existing Code**: 0 lines

## Consuming Fleet Outputs

The simulator can read fleet results without any special parsing:

```rust
// Read JSON output
let json_content = std::fs::read_to_string("fleet_results.json")?;
let fleet_data: serde_json::Value = serde_json::from_str(&json_content)?;

// Extract metrics
let total_repos = fleet_data["total_repositories"].as_u64().unwrap();
let total_tests = fleet_data["fleet_summary"]["total_tests"].as_u64().unwrap();
let success_rate = fleet_data["fleet_summary"]["success_rate"].as_f64().unwrap();

// Access provider breakdown
let providers = fleet_data["provider_breakdown"].as_object().unwrap();
for (provider_name, stats) in providers {
    let repo_count = stats["repository_count"].as_u64().unwrap();
    let success_rate = stats["success_rate"].as_f64().unwrap();
    println!("{}: {} repos, {:.2}% success", provider_name, repo_count, success_rate * 100.0);
}

// Access repository details
let repos = fleet_data["repository_results"].as_array().unwrap();
for repo in repos {
    let name = repo["repository_name"].as_str().unwrap();
    let provider = repo["provider_name"].as_str().unwrap();
    let success = repo["results"]["summary"]["success_rate"].as_f64().unwrap();
    println!("  {}: {} ({:.2}%)", name, provider, success * 100.0);
}
```

## Benefits for Simulator

1. **Zero Breaking Changes**: Existing functionality unchanged
2. **Minimal Code Addition**: ~15 lines for full integration
3. **Rich Analytics**: Fleet-wide insights automatically generated
4. **Multiple Output Formats**: JSON (machines), CSV (spreadsheets), HTML (humans)
5. **Deterministic Results**: Reproducible for debugging and auditing
6. **Performance Metrics**: Built-in latency percentiles and cost tracking
7. **Provider Comparison**: Automatic cross-provider analysis
8. **Scalable**: Tested with 50+ repositories

## Testing Integration

Use the mock simulator client for testing:

```rust
use llm_test_bench_core::tests::simulator_integration_test::MockSimulatorClient;

let mut simulator = MockSimulatorClient::new(workspace_dir);
let run = simulator.invoke_fleet_benchmark(fleet_id, benchmark_results)?;

// Verify artifacts
assert!(run.artifact_paths.json.exists());
assert!(run.artifact_paths.executive_html.exists());

// Validate contents
let validation = simulator.validate_run_artifacts(&run.run_id)?;
assert!(validation);
```

See: `/workspaces/test-bench/core/tests/simulator_integration_test.rs`

## Migration Path

### Phase 1: Parallel Testing
- Keep existing single-repo benchmarks
- Add fleet aggregation for multi-repo runs
- Compare outputs to validate correctness

### Phase 2: Gradual Adoption
- Use fleet benchmarking for new projects
- Migrate existing projects incrementally
- Maintain backward compatibility

### Phase 3: Full Integration
- Fleet benchmarking becomes default for multi-repo scenarios
- Single-repo benchmarks remain available for simple cases
- All analytics leverage fleet infrastructure

## Support and Validation

- **Integration Tests**: 40+ tests validate all scenarios
- **Mock Client**: Demonstrates integration pattern
- **Performance Validated**: Tested with 50+ repositories
- **Documentation**: Complete API documentation with examples

**Questions?** See `/workspaces/test-bench/FLEET_VALIDATION_REPORT.md` for comprehensive validation results.

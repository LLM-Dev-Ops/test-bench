# Fleet-Level Metrics & Aggregation

**Version:** 1.0.0
**Status:** Production
**Date:** 2025-12-31

---

## Table of Contents

1. [Overview](#overview)
2. [Architecture](#architecture)
3. [Metric Definitions](#metric-definitions)
4. [Aggregation Formulas](#aggregation-formulas)
5. [Output Formats](#output-formats)
6. [Determinism Guarantees](#determinism-guarantees)
7. [Backward Compatibility](#backward-compatibility)
8. [Usage Examples](#usage-examples)

---

## Overview

The fleet-level metrics system extends test-bench's single-repository benchmarking to support cross-repository analysis. It maintains full backward compatibility while enabling fleet-wide insights.

### Key Features

- **No Breaking Changes**: Existing `BenchmarkResults` schema unchanged
- **Deterministic Aggregation**: Reproducible results with documented formulas
- **Multiple Output Formats**: CSV summaries, executive reports, JSON
- **Hierarchical Statistics**: Repository → Fleet → Provider/Category breakdowns
- **Performance Optimized**: O(n) aggregation complexity

---

## Architecture

### Type Hierarchy

```
FleetBenchmarkResults
├── FleetSummary (fleet-wide statistics)
├── RepositoryResults[] (per-repository details)
│   └── BenchmarkResults (existing single-repo type)
├── ProviderFleetStats{} (per-provider aggregation)
└── CategoryFleetStats{} (per-category aggregation)
```

### Data Flow

```
Single-Repo Benchmark
    ↓
BenchmarkResults (unchanged)
    ↓
RepositoryResults (wraps BenchmarkResults)
    ↓
FleetBenchmarkResults (aggregates multiple repos)
    ↓
Output Formats (CSV, HTML, JSON)
```

---

## Metric Definitions

### Single-Repository Metrics (Existing)

#### ResultSummary

| Metric | Type | Definition | Range |
|--------|------|------------|-------|
| `total` | usize | Total number of tests | [0, ∞) |
| `succeeded` | usize | Number of successful tests | [0, total] |
| `failed` | usize | Number of failed tests | [0, total] |
| `timeout` | usize | Number of timed-out tests | [0, total] |
| `skipped` | usize | Number of skipped tests | [0, total] |
| `success_rate` | f64 | Ratio of successful tests | [0.0, 1.0] |
| `avg_duration_ms` | f64 | Mean test duration (ms) | [0.0, ∞) |
| `p50_duration_ms` | f64 | Median test duration (ms) | [0.0, ∞) |
| `p95_duration_ms` | f64 | 95th percentile duration (ms) | [0.0, ∞) |
| `p99_duration_ms` | f64 | 99th percentile duration (ms) | [0.0, ∞) |
| `min_duration_ms` | u64 | Minimum test duration (ms) | [0, ∞) |
| `max_duration_ms` | u64 | Maximum test duration (ms) | [0, ∞) |
| `total_tokens` | usize | Sum of all tokens used | [0, ∞) |
| `avg_tokens_per_request` | f64 | Mean tokens per successful test | [0.0, ∞) |
| `total_cost` | f64 | Estimated total cost (USD) | [0.0, ∞) |

### Fleet-Level Metrics (New)

#### FleetSummary

Extends repository metrics with fleet-wide aggregations:

| Metric | Type | Definition | Formula |
|--------|------|------------|---------|
| `total_repositories` | usize | Number of repositories | count(repos) |
| `total_tests` | usize | Total tests across fleet | Σ(repo.total) |
| `total_succeeded` | usize | Total successful tests | Σ(repo.succeeded) |
| `total_failed` | usize | Total failed tests | Σ(repo.failed) |
| `total_timeout` | usize | Total timed-out tests | Σ(repo.timeout) |
| `total_skipped` | usize | Total skipped tests | Σ(repo.skipped) |
| `success_rate` | f64 | Fleet-wide success rate | total_succeeded / total_tests |
| `avg_duration_ms` | f64 | Mean across all tests | Σ(durations) / count(tests) |
| `p50_duration_ms` | f64 | Fleet-wide median | percentile(all_durations, 50) |
| `p95_duration_ms` | f64 | Fleet-wide 95th percentile | percentile(all_durations, 95) |
| `p99_duration_ms` | f64 | Fleet-wide 99th percentile | percentile(all_durations, 99) |
| `min_duration_ms` | u64 | Global minimum | min(all_durations) |
| `max_duration_ms` | u64 | Global maximum | max(all_durations) |
| `total_tokens` | usize | Total tokens across fleet | Σ(repo.total_tokens) |
| `avg_tokens_per_request` | f64 | Fleet-wide average | total_tokens / total_succeeded |
| `total_cost` | f64 | Total cost across fleet | Σ(repo.total_cost) |
| `avg_cost_per_repository` | f64 | Mean cost per repo | total_cost / total_repositories |
| `avg_tests_per_repository` | f64 | Mean tests per repo | total_tests / total_repositories |
| `total_duration_ms` | u64 | Sum of repo durations | Σ(repo.total_duration_ms) |

#### ProviderFleetStats

Per-provider breakdown across repositories:

| Metric | Type | Definition |
|--------|------|------------|
| `provider_name` | String | Provider identifier |
| `repository_count` | usize | Number of repos using provider |
| `total_tests` | usize | Tests for this provider |
| `total_succeeded` | usize | Successful tests |
| `total_failed` | usize | Failed tests |
| `success_rate` | f64 | Provider success rate |
| `total_tokens` | usize | Tokens used by provider |
| `total_cost` | f64 | Cost for this provider |
| `avg_duration_ms` | f64 | Average duration |

#### CategoryFleetStats

Per-category breakdown across repositories:

| Metric | Type | Definition |
|--------|------|------------|
| `category_name` | String | Test category |
| `total_tests` | usize | Tests in category |
| `total_succeeded` | usize | Successful tests |
| `total_failed` | usize | Failed tests |
| `success_rate` | f64 | Category success rate |
| `avg_duration_ms` | f64 | Average duration |

---

## Aggregation Formulas

### Success Rate

**Single Repository:**
```
success_rate = succeeded / total
```

**Fleet-Wide:**
```
success_rate = Σ(repo.succeeded) / Σ(repo.total)
```

**Notes:**
- Returns 0.0 if total = 0
- Range: [0.0, 1.0]
- Multiply by 100 for percentage

### Percentile Calculation

**Algorithm:** Linear interpolation method

```rust
fn calculate_percentile(sorted_durations: &[u64], percentile: f64) -> f64 {
    if durations.is_empty() {
        return 0.0;
    }

    if durations.len() == 1 {
        return durations[0] as f64;
    }

    let index = (percentile / 100.0 * (durations.len() - 1) as f64).ceil() as usize;
    let index = index.min(durations.len() - 1);

    durations[index] as f64
}
```

**Fleet-Wide Percentiles:**
1. Collect all individual test durations from all repositories
2. Sort the combined list: O(n log n)
3. Apply percentile formula to the global sorted list

**Example:**
- Repo1: [100ms, 200ms, 300ms]
- Repo2: [150ms, 250ms]
- Combined sorted: [100, 150, 200, 250, 300]
- P50: 200ms (median of combined)

### Average Duration

**Single Repository:**
```
avg_duration_ms = Σ(test.duration_ms) / count(tests)
```

**Fleet-Wide:**
```
avg_duration_ms = Σ(all_test_durations) / total_tests
```

**Not** an average of averages (which would be incorrect):
```
❌ WRONG: avg_duration_ms = Σ(repo.avg_duration_ms) / count(repos)
✅ RIGHT: avg_duration_ms = Σ(all_test_durations) / total_tests
```

### Token Usage

**Average Tokens per Request:**
```
avg_tokens_per_request = total_tokens / total_succeeded
```

**Notes:**
- Only counts successful tests (where tokens are available)
- Returns 0.0 if no successful tests

### Cost Estimation

**Single Test:**
```
cost = (prompt_tokens / 1000.0) * prompt_rate +
       (completion_tokens / 1000.0) * completion_rate
```

**Default Rates (GPT-4):**
- Prompt: $0.03 per 1K tokens
- Completion: $0.06 per 1K tokens

**Repository Total:**
```
total_cost = Σ(test.cost for test in successful_tests)
```

**Fleet Total:**
```
total_cost = Σ(repo.total_cost)
```

**Average per Repository:**
```
avg_cost_per_repository = total_cost / total_repositories
```

### Provider Breakdown

For each unique provider in the fleet:

```rust
for each repository in fleet:
    if repository.provider_name == provider:
        provider_stats.repository_count += 1
        provider_stats.total_tests += repository.summary.total
        provider_stats.total_succeeded += repository.summary.succeeded
        provider_stats.total_failed += repository.summary.failed
        provider_stats.total_tokens += repository.summary.total_tokens
        provider_stats.total_cost += repository.summary.total_cost

provider_stats.success_rate = provider_stats.total_succeeded / provider_stats.total_tests
```

### Category Breakdown

For each unique category across all tests in all repositories:

```rust
for each repository in fleet:
    for each test in repository.results:
        if test.category == category:
            category_stats.total_tests += 1
            if test.status == Success:
                category_stats.total_succeeded += 1
            elif test.status == Failure:
                category_stats.total_failed += 1

category_stats.success_rate = category_stats.total_succeeded / category_stats.total_tests
```

---

## Output Formats

### 1. Fleet Summary CSV

**File:** `fleet_summary.csv`
**Rows:** 1 (plus header)
**Columns:** 20

```csv
fleet_id,timestamp,total_repositories,total_tests,...
prod-fleet,2025-12-31T12:00:00Z,10,1000,...
```

**Use Case:** Dashboard imports, trend analysis

### 2. Repository Details CSV

**File:** `repositories.csv`
**Rows:** N (one per repository)
**Columns:** 15

```csv
repository_id,repository_name,provider_name,total_tests,...
repo-0,my-service,openai,100,...
repo-1,api-gateway,anthropic,150,...
```

**Use Case:** Per-repository comparison, drill-down analysis

### 3. Provider Breakdown CSV

**File:** `providers.csv`
**Rows:** M (one per unique provider)
**Columns:** 8

```csv
provider_name,repository_count,total_tests,success_rate,...
openai,6,600,0.95,...
anthropic,4,400,0.92,...
```

**Use Case:** Provider performance comparison

### 4. Category Breakdown CSV

**File:** `categories.csv`
**Rows:** C (one per unique category)
**Columns:** 5

```csv
category_name,total_tests,total_succeeded,success_rate
coding,300,285,0.95
reasoning,200,180,0.90
```

**Use Case:** Test category analysis

### 5. Executive Report (HTML)

**File:** `executive_report.html`
**Format:** Self-contained HTML with inline CSS

**Sections:**
- Header with fleet ID and timestamp
- Key metrics grid (4 cards)
- Performance metrics table
- Provider breakdown table
- Best/worst repository comparison
- Failing repositories alert (if any)

**Use Case:** Management reporting, status dashboards

### 6. Deterministic JSON

**File:** `fleet_results.json`
**Format:** Pretty-printed JSON with sorted keys

**Features:**
- Fields ordered using BTreeMap for determinism
- Timestamps in RFC3339 format
- Floating-point numbers with consistent precision
- All nested structures included

**Use Case:** Programmatic access, version control, CI/CD

---

## Determinism Guarantees

### Sources of Non-Determinism

1. **Hash Maps:** Iteration order not guaranteed
2. **Floating-Point:** Rounding inconsistencies
3. **Timestamps:** System clock variance
4. **Concurrency:** Race conditions in aggregation

### Mitigation Strategies

#### 1. Sorted Collections

**Provider Breakdown:**
```rust
let mut providers: Vec<_> = results.provider_breakdown.iter().collect();
providers.sort_by_key(|(name, _)| *name);  // Deterministic order
```

**Category Breakdown:**
```rust
let mut categories: Vec<_> = results.category_breakdown.iter().collect();
categories.sort_by_key(|(name, _)| *name);
```

#### 2. Deterministic JSON

```rust
use std::collections::BTreeMap;  // Sorted keys

let mut ordered_json = BTreeMap::new();
ordered_json.insert("fleet_id", json!(results.fleet_id));
ordered_json.insert("timestamp", json!(results.timestamp.to_rfc3339()));
// ... all fields in alphabetical order
```

#### 3. Consistent Formatting

**Floating-Point:**
```rust
format!("{:.4}", value)  // Always 4 decimal places
```

**Timestamps:**
```rust
results.timestamp.to_rfc3339()  // ISO 8601 format
```

#### 4. Sequential Aggregation

```rust
// Single-threaded aggregation (no race conditions)
for repo in repo_results {
    total_tests += repo.results.summary.total;
    total_succeeded += repo.results.summary.succeeded;
    // ...
}
```

### Verification

**Test for Determinism:**
```rust
#[test]
fn test_deterministic_output() {
    let fleet1 = FleetBenchmarkResults::from_repositories(id.clone(), repos.clone());
    let fleet2 = FleetBenchmarkResults::from_repositories(id.clone(), repos.clone());

    let json1 = serde_json::to_string(&fleet1).unwrap();
    let json2 = serde_json::to_string(&fleet2).unwrap();

    assert_eq!(json1, json2);  // Must be identical
}
```

---

## Backward Compatibility

### Preserved Schemas

**BenchmarkResults** (unchanged):
```rust
pub struct BenchmarkResults {
    pub dataset_name: String,
    pub provider_name: String,
    pub total_tests: usize,
    pub results: Vec<TestResult>,
    pub started_at: DateTime<Utc>,
    pub completed_at: DateTime<Utc>,
    pub total_duration_ms: u64,
    pub summary: ResultSummary,
}
```

**ResultSummary** (unchanged):
```rust
pub struct ResultSummary {
    pub total: usize,
    pub succeeded: usize,
    pub failed: usize,
    pub timeout: usize,
    pub skipped: usize,
    pub success_rate: f64,
    // ... all existing fields
}
```

### Extension Pattern

**RepositoryResults** (wrapper):
```rust
pub struct RepositoryResults {
    pub repository_id: String,
    pub repository_name: String,
    pub provider_name: String,
    pub results: BenchmarkResults,  // Existing type, unchanged
    pub repository_metadata: HashMap<String, String>,
}
```

### Migration Path

**Existing Code:**
```rust
let results = runner.run(&dataset, provider).await?;
results.summary.success_rate  // Still works
```

**Fleet Extension:**
```rust
let fleet = FleetBenchmarkResults::from_repositories(
    "fleet-id".to_string(),
    vec![results1, results2, results3],  // Existing results
);
fleet.fleet_summary.success_rate  // New fleet-wide metric
```

---

## Usage Examples

### Basic Fleet Aggregation

```rust
use llm_test_bench_core::benchmarks::fleet::FleetBenchmarkResults;

// Collect results from multiple repositories
let repo1_results = benchmark_repository("repo1", "openai").await?;
let repo2_results = benchmark_repository("repo2", "anthropic").await?;
let repo3_results = benchmark_repository("repo3", "openai").await?;

// Aggregate into fleet results
let fleet = FleetBenchmarkResults::from_repositories(
    "production-fleet".to_string(),
    vec![repo1_results, repo2_results, repo3_results],
);

// Access fleet-wide metrics
println!("Fleet success rate: {:.2}%", fleet.fleet_summary.success_rate * 100.0);
println!("Total cost: ${:.2}", fleet.fleet_summary.total_cost);
println!("Repositories: {}", fleet.total_repositories);
```

### Export All Formats

```rust
use llm_test_bench_core::benchmarks::fleet_export::FleetCsvExporter;
use std::path::Path;

// Export fleet summary
FleetCsvExporter::export_summary(&fleet, Path::new("fleet_summary.csv"))?;

// Export repository details
FleetCsvExporter::export_repositories(&fleet, Path::new("repositories.csv"))?;

// Export provider breakdown
FleetCsvExporter::export_providers(&fleet, Path::new("providers.csv"))?;

// Export category breakdown
FleetCsvExporter::export_categories(&fleet, Path::new("categories.csv"))?;

// Export executive report
FleetCsvExporter::export_executive_report(&fleet, Path::new("report.html"))?;

// Export deterministic JSON
FleetCsvExporter::export_deterministic_json(&fleet, Path::new("fleet.json"))?;
```

### Identify Problem Areas

```rust
// Find repositories below threshold
let failing = fleet.failing_repositories(0.9);
for repo in failing {
    println!("Repository '{}' has {:.1}% success rate",
        repo.repository_name,
        repo.results.summary.success_rate * 100.0);
}

// Best and worst performers
if let Some(best) = fleet.best_repository() {
    println!("Best: {} ({:.1}%)", best.repository_name, best.results.summary.success_rate * 100.0);
}

if let Some(worst) = fleet.worst_repository() {
    println!("Worst: {} ({:.1}%)", worst.repository_name, worst.results.summary.success_rate * 100.0);
}
```

### Provider Comparison

```rust
// Compare providers across fleet
for (provider_name, stats) in &fleet.provider_breakdown {
    println!("Provider: {}", provider_name);
    println!("  Repositories: {}", stats.repository_count);
    println!("  Tests: {}", stats.total_tests);
    println!("  Success Rate: {:.2}%", stats.success_rate * 100.0);
    println!("  Total Cost: ${:.4}", stats.total_cost);
    println!();
}
```

### Trend Analysis (CI/CD)

```rust
use std::fs;
use serde_json;

// Export deterministic JSON for version control
FleetCsvExporter::export_deterministic_json(&fleet, Path::new("latest_fleet_results.json"))?;

// Load previous results
let previous_json = fs::read_to_string("previous_fleet_results.json")?;
let previous: FleetBenchmarkResults = serde_json::from_str(&previous_json)?;

// Compare metrics
let success_rate_delta = fleet.fleet_summary.success_rate - previous.fleet_summary.success_rate;
println!("Success rate change: {:+.2}%", success_rate_delta * 100.0);

if success_rate_delta < -0.05 {
    eprintln!("Warning: Success rate dropped by more than 5%!");
    std::process::exit(1);
}
```

---

## Implementation Notes

### Performance Characteristics

- **Aggregation Time:** O(n × m) where n = repositories, m = avg tests per repo
- **Memory Usage:** O(n × m) for storing all test durations
- **Percentile Calculation:** O((n × m) log (n × m)) due to sorting
- **CSV Export:** O(n) for repositories, O(p) for providers, O(c) for categories

### Optimization Opportunities

1. **Streaming Percentiles:** Use approximate algorithms for large fleets
2. **Parallel Aggregation:** Partition repositories across threads
3. **Incremental Updates:** Cache intermediate aggregations
4. **Sparse Storage:** Skip empty categories/providers in output

### Future Extensions

1. **Time Series Analysis:** Track fleet metrics over time
2. **Anomaly Detection:** Flag statistical outliers
3. **Cost Optimization:** Recommend provider/model combinations
4. **Predictive Analytics:** Forecast fleet performance

---

## References

- [Percentile Calculation](https://en.wikipedia.org/wiki/Percentile)
- [JSON Determinism](https://github.com/rust-lang/rust/issues/44232)
- [IEEE 754 Floating Point](https://en.wikipedia.org/wiki/IEEE_754)

---

**Document Version:** 1.0.0
**Last Updated:** 2025-12-31
**Maintained By:** test-bench METRICS & OUTPUT ENGINEER agent

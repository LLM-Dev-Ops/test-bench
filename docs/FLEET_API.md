# Fleet Benchmark API

The Fleet Benchmark API provides a programmatic interface for executing benchmarks across multiple repositories. It is designed for simulator integration and automation workflows that need to trigger fleet-wide testing and retrieve results.

## Table of Contents

- [Overview](#overview)
- [Architecture](#architecture)
- [Usage](#usage)
  - [Rust API](#rust-api)
  - [CLI](#cli)
  - [TypeScript SDK](#typescript-sdk)
- [Fleet Manifest](#fleet-manifest)
- [Execution Flow](#execution-flow)
- [Results and Artifacts](#results-and-artifacts)
- [Examples](#examples)

## Overview

The Fleet API enables you to:

- **Execute benchmarks across multiple repositories simultaneously**
- **Get immediate run identifiers and artifact locations**
- **Run benchmarks asynchronously without blocking**
- **Retrieve aggregated fleet-wide metrics**
- **Track execution progress and results**

## Architecture

The Fleet API follows a clean layered architecture:

```
┌─────────────────────────────────────────┐
│   Simulator / Automation Tool          │
└─────────────┬───────────────────────────┘
              │
              ↓
┌─────────────────────────────────────────┐
│   TypeScript SDK / Rust API             │
│   - FleetBenchmark class                │
│   - FleetBenchmarkAPI struct            │
└─────────────┬───────────────────────────┘
              │
              ↓
┌─────────────────────────────────────────┐
│   CLI Fleet Command                     │
│   - llm-test-bench fleet                │
└─────────────┬───────────────────────────┘
              │
              ↓
┌─────────────────────────────────────────┐
│   Core Fleet Execution Engine           │
│   - BenchmarkRunner (per repo)          │
│   - FleetBenchmarkResults (aggregation) │
└─────────────────────────────────────────┘
```

### Key Design Principles

1. **Immediate Return**: API returns handle immediately with `run_id` and artifact paths
2. **Async Execution**: Actual benchmark runs asynchronously in background
3. **Deterministic IDs**: Generates reproducible run identifiers based on fleet ID and timestamp
4. **Artifact Tracking**: Provides paths to results before execution completes
5. **No Simulator Changes**: Simulators can integrate without modification

## Usage

### Rust API

```rust
use llm_test_bench_core::benchmarks::fleet_api::{FleetBenchmarkAPI, FleetConfig};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure fleet API
    let config = FleetConfig::new(PathBuf::from("./fleet-results"))
        .with_concurrency(10)
        .with_save_responses(true);

    let api = FleetBenchmarkAPI::new(config);

    // Execute fleet benchmark (returns immediately)
    let handle = api
        .execute_fleet_benchmark(&PathBuf::from("./fleet-manifest.json"))
        .await?;

    println!("Run ID: {}", handle.run_id);
    println!("Artifacts: {}", handle.artifact_base_dir.display());

    // Option 1: Return immediately (async execution)
    println!("Fleet benchmark running in background");

    // Option 2: Wait for completion
    let results = handle.execution_future.await??;
    println!("Fleet completed: {} repositories", results.total_repositories);
    println!("Success rate: {:.2}%", results.fleet_summary.success_rate * 100.0);

    Ok(())
}
```

#### Retrieving Results Later

```rust
// Get results for a completed run
let results = api.get_fleet_results("my-fleet-20231215-123456").await?;
println!("Success rate: {:.2}%", results.fleet_summary.success_rate * 100.0);

// List all available runs
let runs = api.list_runs().await?;
for run_id in runs {
    println!("Available run: {}", run_id);
}
```

### CLI

```bash
# Execute fleet benchmark (async mode - returns immediately)
llm-test-bench fleet --manifest ./fleet-manifest.json --output ./fleet-results

# Output:
# Run ID: agentics-fleet-2025-20231231-abc123
# Artifacts: ./fleet-results/agentics-fleet-2025-20231231-abc123/
# Status: Executing (5 repositories, 2 providers)...

# Execute and wait for completion (sync mode)
llm-test-bench fleet --manifest ./fleet.json --wait --format summary

# Output:
# Fleet Benchmark Results
# ═════════════════════════════════════════════════════════
#
# Fleet Overview
# ─────────────────────────────────────────────────────────
#   Fleet ID:        agentics-fleet-2025
#   Repositories:    5
#   Total Tests:     250
#   Duration:        145.32s
# ...

# Execute with custom configuration
llm-test-bench fleet \
  --manifest ./fleet.json \
  --output ./custom-results \
  --concurrency 15 \
  --delay 100 \
  --config ./custom-config.toml

# Quiet mode (only output run_id)
llm-test-bench fleet --manifest ./fleet.json --format quiet
# Output: agentics-fleet-2025-20231231-abc123
```

### TypeScript SDK

```typescript
import { FleetBenchmark } from 'llm-test-bench';

// Initialize Fleet Benchmark API
const fleet = new FleetBenchmark({
  outputBaseDir: './fleet-results',
  defaultConcurrency: 10,
  saveResponses: true,
  requestDelayMs: 100,
});

// Execute fleet benchmark (async - returns immediately)
const handle = await fleet.executeFleet('./fleet-manifest.json');
console.log('Run ID:', handle.runId);
console.log('Artifacts:', handle.artifactBaseDir);
console.log('Fleet:', handle.metadata.fleetId);
console.log('Repositories:', handle.metadata.repositoryCount);

// Continue with other work while fleet executes...

// Later, wait for completion
const results = await fleet.waitForCompletion(handle.runId, {
  timeoutMs: 600000, // 10 minutes
  pollIntervalMs: 2000, // Check every 2 seconds
});

console.log('Fleet completed!');
console.log('Success rate:', results.fleet_summary.success_rate);
console.log('Total cost:', results.fleet_summary.total_cost);

// Or execute and wait immediately
const results = await fleet.executeFleet('./fleet-manifest.json', {
  wait: true,
  format: 'json',
});
```

#### Retrieving Existing Results

```typescript
// Get results for a specific run
const results = await fleet.getFleetResults('my-fleet-20231215-123456');

// List all available runs
const runs = await fleet.listRuns();
for (const runId of runs) {
  const results = await fleet.getFleetResults(runId);
  console.log(`${runId}: ${results.fleet_summary.success_rate * 100}% success`);
}
```

## Fleet Manifest

The fleet manifest is a JSON file that specifies repositories and providers to benchmark.

### Manifest Structure

```json
{
  "fleet_id": "agentics-fleet-2025",
  "description": "Production fleet for agentic workflows",
  "repositories": [
    {
      "id": "repo-1",
      "name": "Task Planning Repository",
      "dataset_path": "./datasets/task-planning.json",
      "metadata": {
        "team": "planning",
        "priority": "high"
      }
    },
    {
      "id": "repo-2",
      "name": "Code Generation Repository",
      "dataset_path": "./datasets/code-gen.json",
      "metadata": {
        "team": "codegen",
        "priority": "medium"
      }
    }
  ],
  "providers": [
    "openai",
    "anthropic"
  ],
  "config": {
    "concurrency": 10,
    "request_delay_ms": 100,
    "config_path": "./llm-test-bench.toml"
  }
}
```

### Manifest Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `fleet_id` | string | Yes | Unique identifier for this fleet |
| `description` | string | No | Human-readable description |
| `repositories` | array | Yes | List of repository specifications |
| `providers` | array | Yes | List of provider names to benchmark |
| `config` | object | No | Fleet-level configuration overrides |

### Repository Specification

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | string | Yes | Unique repository identifier |
| `name` | string | Yes | Human-readable repository name |
| `dataset_path` | string | Yes | Path to dataset file |
| `metadata` | object | No | Custom metadata key-value pairs |

## Execution Flow

1. **Submit Manifest**: Provide fleet manifest with repositories and providers
2. **Validation**: API validates manifest and checks for required files
3. **Run ID Generation**: Deterministic ID created: `{fleet_id}-{timestamp}`
4. **Directory Setup**: Creates artifact directory structure:
   ```
   fleet-results/
   └── agentics-fleet-2025-20231231-123456/
       ├── fleet-results.json
       ├── fleet-summary.csv
       ├── repo-1/
       │   ├── openai/
       │   │   ├── results.json
       │   │   └── summary.csv
       │   └── anthropic/
       │       ├── results.json
       │       └── summary.csv
       └── repo-2/
           └── ...
   ```
5. **Async Execution**: Spawns background task to execute benchmarks
6. **Immediate Return**: Returns handle with `run_id` and artifact paths
7. **Parallel Execution**: Runs benchmarks for each repository-provider combination
8. **Aggregation**: Computes fleet-wide statistics
9. **Result Storage**: Saves JSON and CSV results to artifact directory

## Results and Artifacts

### Artifacts Generated

Each fleet run generates:

- **`fleet-results.json`**: Complete fleet results with all data
- **`fleet-summary.csv`**: Executive summary in CSV format
- **Per-repository artifacts**:
  - `{repo_id}/{provider}/results.json`: Full benchmark results
  - `{repo_id}/{provider}/summary.csv`: Summary statistics
  - Individual test responses (if `save_responses: true`)

### Fleet Results Structure

```typescript
{
  "fleet_id": "agentics-fleet-2025",
  "timestamp": "2023-12-31T12:34:56Z",
  "total_repositories": 5,
  "repository_results": [...],
  "fleet_summary": {
    "total_repositories": 5,
    "total_tests": 250,
    "total_succeeded": 235,
    "total_failed": 15,
    "success_rate": 0.94,
    "avg_duration_ms": 1234.56,
    "p50_duration_ms": 1000.0,
    "p95_duration_ms": 2500.0,
    "p99_duration_ms": 3000.0,
    "total_tokens": 125000,
    "total_cost": 12.50,
    "avg_cost_per_repository": 2.50
  },
  "provider_breakdown": {
    "openai": {
      "repository_count": 5,
      "total_tests": 125,
      "success_rate": 0.96,
      "total_cost": 7.50
    },
    "anthropic": {
      "repository_count": 5,
      "total_tests": 125,
      "success_rate": 0.92,
      "total_cost": 5.00
    }
  }
}
```

## Examples

### Example 1: Simulator Integration

```typescript
// Simulator code that triggers fleet benchmarks
import { FleetBenchmark } from 'llm-test-bench';

class AgentSimulator {
  private fleet: FleetBenchmark;
  private activeRuns: Map<string, FleetExecutionHandle>;

  constructor() {
    this.fleet = new FleetBenchmark({
      outputBaseDir: './simulation-results',
      defaultConcurrency: 15,
    });
    this.activeRuns = new Map();
  }

  async startFleetBenchmark(scenario: string): Promise<string> {
    const manifestPath = `./scenarios/${scenario}/fleet-manifest.json`;

    // Execute fleet (non-blocking)
    const handle = await this.fleet.executeFleet(manifestPath);

    // Track active run
    this.activeRuns.set(handle.runId, handle);

    // Store run ID for later retrieval
    await this.storeRunId(scenario, handle.runId);

    return handle.runId;
  }

  async checkFleetStatus(runId: string): Promise<'running' | 'completed' | 'not_found'> {
    try {
      await this.fleet.getFleetResults(runId);
      return 'completed';
    } catch (error) {
      // Results not available yet
      return this.activeRuns.has(runId) ? 'running' : 'not_found';
    }
  }

  async getFleetMetrics(runId: string) {
    const results = await this.fleet.getFleetResults(runId);
    return {
      successRate: results.fleet_summary.success_rate,
      totalCost: results.fleet_summary.total_cost,
      avgLatency: results.fleet_summary.avg_duration_ms,
      repositories: results.total_repositories,
    };
  }
}
```

### Example 2: CI/CD Integration

```bash
#!/bin/bash
# ci-fleet-test.sh

set -e

echo "Starting fleet benchmark for PR #$PR_NUMBER"

# Execute fleet benchmark (quiet mode for scripting)
RUN_ID=$(llm-test-bench fleet \
  --manifest ./ci/fleet-manifest.json \
  --output ./ci-results \
  --format quiet)

echo "Fleet run started: $RUN_ID"
echo "run_id=$RUN_ID" >> $GITHUB_OUTPUT

# Wait for completion (separate job can check status)
llm-test-bench fleet-status $RUN_ID --wait

# Retrieve results
SUCCESS_RATE=$(jq -r '.fleet_summary.success_rate' \
  ./ci-results/$RUN_ID/fleet-results.json)

if (( $(echo "$SUCCESS_RATE < 0.95" | bc -l) )); then
  echo "Fleet benchmark failed: ${SUCCESS_RATE}% success rate (threshold: 95%)"
  exit 1
fi

echo "Fleet benchmark passed: ${SUCCESS_RATE}% success rate"
```

### Example 3: Monitoring Dashboard

```typescript
// Monitor multiple fleet runs
import { FleetBenchmark } from 'llm-test-bench';

async function generateDashboard() {
  const fleet = new FleetBenchmark({
    outputBaseDir: './fleet-results',
  });

  // Get all runs
  const runs = await fleet.listRuns();

  const dashboard = {
    totalRuns: runs.length,
    runs: [],
  };

  for (const runId of runs) {
    try {
      const results = await fleet.getFleetResults(runId);
      dashboard.runs.push({
        runId,
        fleetId: results.fleet_id,
        timestamp: results.timestamp,
        repositories: results.total_repositories,
        successRate: results.fleet_summary.success_rate,
        totalCost: results.fleet_summary.total_cost,
        status: 'completed',
      });
    } catch (error) {
      dashboard.runs.push({
        runId,
        status: 'running',
      });
    }
  }

  return dashboard;
}
```

## Best Practices

1. **Use Deterministic Fleet IDs**: Include version or environment in fleet_id (e.g., `prod-fleet-v2`)
2. **Set Appropriate Concurrency**: Balance speed vs. rate limits (typically 5-15)
3. **Enable Response Saving**: Useful for debugging and detailed analysis
4. **Add Repository Metadata**: Include team, priority, or category information
5. **Monitor Costs**: Check `total_cost` and `avg_cost_per_repository` regularly
6. **Archive Old Runs**: Periodically clean up old fleet results to save disk space

## Troubleshooting

### Common Issues

**Issue**: Manifest validation fails
- **Solution**: Ensure `fleet_id`, `repositories`, and `providers` are all non-empty

**Issue**: Dataset file not found
- **Solution**: Verify `dataset_path` is relative to working directory or use absolute paths

**Issue**: Provider not configured
- **Solution**: Ensure provider is defined in `llm-test-bench.toml` configuration file

**Issue**: Results not found
- **Solution**: Check if fleet execution completed; use `waitForCompletion()` or polling

## See Also

- [Fleet Metrics Documentation](./FLEET_METRICS.md)
- [Benchmark Configuration](./CONFIGURATION.md)
- [Dataset Format](./DATASETS.md)

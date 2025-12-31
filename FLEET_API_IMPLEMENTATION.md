# Fleet Benchmark API Implementation

## Overview

This document describes the implementation of the Fleet Benchmark API for the LLM Test Bench. This API provides a programmatic interface for executing benchmarks across multiple repositories, designed specifically for simulator integration and automation workflows.

## Implementation Summary

### Components Delivered

1. **Rust API Module** (`core/src/benchmarks/fleet_api.rs`)
   - `FleetBenchmarkAPI` struct for programmatic execution
   - `FleetConfig` for configuration management
   - `FleetExecutionHandle` returned immediately with run ID and artifact paths
   - `FleetManifest` structure for defining fleet benchmarks
   - Async execution with immediate return

2. **CLI Command** (`cli/src/commands/fleet.rs`)
   - `llm-test-bench fleet` command
   - Support for async and sync execution modes
   - Multiple output formats (json, summary, quiet)
   - Integration with existing CLI infrastructure

3. **TypeScript SDK** (`src/core/fleet-benchmark.ts`)
   - `FleetBenchmark` class for Node.js/TypeScript integration
   - Promise-based API matching Rust capabilities
   - Result polling and retrieval methods
   - Full type definitions for all data structures

4. **Documentation**
   - Comprehensive API documentation (`docs/FLEET_API.md`)
   - Usage examples (`examples/fleet-api-example.ts`)
   - Example fleet manifest (`examples/fleet-manifest-example.json`)
   - Integration patterns for simulators

## Architecture

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

## Key Features

### 1. Immediate Return with Run Identifier

```rust
// Rust
let handle = api.execute_fleet_benchmark(&manifest_path).await?;
println!("Run ID: {}", handle.run_id);
println!("Artifacts: {}", handle.artifact_base_dir.display());
// Execution continues in background
```

```typescript
// TypeScript
const handle = await fleet.executeFleet('./manifest.json');
console.log('Run ID:', handle.runId);
console.log('Artifacts:', handle.artifactBaseDir);
// Execution continues in background
```

### 2. Deterministic Run IDs

Format: `{fleet_id}-{timestamp}`

Example: `agentics-fleet-2025-20231231-123456`

### 3. Pre-calculated Artifact Paths

```typescript
{
  runId: "agentics-fleet-2025-20231231-123456",
  artifactBaseDir: "./fleet-results/agentics-fleet-2025-20231231-123456",
  metadata: {
    expectedArtifacts: [
      "./fleet-results/agentics-fleet-2025-20231231-abc123/fleet-results.json",
      "./fleet-results/agentics-fleet-2025-20231231-abc123/fleet-summary.csv",
      // ... per-repository artifacts
    ]
  }
}
```

### 4. Flexible Execution Modes

**Async Mode (Non-blocking)**:
```bash
llm-test-bench fleet --manifest ./fleet.json
# Returns immediately with run_id
```

**Sync Mode (Wait for completion)**:
```bash
llm-test-bench fleet --manifest ./fleet.json --wait
# Waits and displays results when complete
```

**Quiet Mode (Scripting)**:
```bash
RUN_ID=$(llm-test-bench fleet --manifest ./fleet.json --format quiet)
echo "Started: $RUN_ID"
```

## Usage Examples

### Simulator Integration Pattern

```typescript
import { FleetBenchmark } from 'llm-test-bench';

class AgentSimulator {
  private fleet: FleetBenchmark;

  constructor() {
    this.fleet = new FleetBenchmark({
      outputBaseDir: './simulation-results',
      defaultConcurrency: 15,
    });
  }

  async startBenchmark(scenario: string): Promise<string> {
    const handle = await this.fleet.executeFleet(
      `./scenarios/${scenario}/fleet-manifest.json`
    );

    // Store run_id for later retrieval
    await this.storeRunId(scenario, handle.runId);

    return handle.runId;
  }

  async getResults(runId: string) {
    return await this.fleet.getFleetResults(runId);
  }
}
```

### CI/CD Integration

```bash
#!/bin/bash
# Execute fleet benchmark in CI pipeline

RUN_ID=$(llm-test-bench fleet \
  --manifest ./ci/fleet-manifest.json \
  --format quiet)

echo "Fleet benchmark started: $RUN_ID"

# Wait for completion (can be separate job)
llm-test-bench fleet-status $RUN_ID --wait

# Validate results
SUCCESS_RATE=$(jq -r '.fleet_summary.success_rate' \
  ./fleet-results/$RUN_ID/fleet-results.json)

if (( $(echo "$SUCCESS_RATE < 0.95" | bc -l) )); then
  echo "Failed: ${SUCCESS_RATE}% success (threshold: 95%)"
  exit 1
fi

echo "Passed: ${SUCCESS_RATE}% success"
```

## Fleet Manifest Format

```json
{
  "fleet_id": "agentics-fleet-2025",
  "description": "Production fleet benchmarks",
  "repositories": [
    {
      "id": "repo-1",
      "name": "Task Planning",
      "dataset_path": "./datasets/task-planning.json",
      "metadata": {
        "team": "planning",
        "priority": "high"
      }
    }
  ],
  "providers": ["openai", "anthropic"],
  "config": {
    "concurrency": 10,
    "request_delay_ms": 100
  }
}
```

## Results Structure

### Fleet Results JSON

```json
{
  "fleet_id": "agentics-fleet-2025",
  "timestamp": "2023-12-31T12:34:56Z",
  "total_repositories": 5,
  "repository_results": [...],
  "fleet_summary": {
    "total_repositories": 5,
    "total_tests": 250,
    "total_succeeded": 235,
    "success_rate": 0.94,
    "avg_duration_ms": 1234.56,
    "p95_duration_ms": 2500.0,
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
    }
  }
}
```

### Artifact Directory Structure

```
fleet-results/
└── agentics-fleet-2025-20231231-123456/
    ├── fleet-results.json          # Complete fleet results
    ├── fleet-summary.csv            # Executive summary
    ├── repo-1/
    │   ├── openai/
    │   │   ├── results.json
    │   │   ├── summary.csv
    │   │   └── responses/           # Individual test responses
    │   └── anthropic/
    │       └── ...
    └── repo-2/
        └── ...
```

## API Reference

### Rust API

```rust
// Create API instance
let config = FleetConfig::new(PathBuf::from("./fleet-results"))
    .with_concurrency(10)
    .with_save_responses(true);

let api = FleetBenchmarkAPI::new(config);

// Execute fleet benchmark
let handle = api.execute_fleet_benchmark(&manifest_path).await?;

// Wait for completion (optional)
let results = handle.execution_future.await??;

// Retrieve results later
let results = api.get_fleet_results("run-id").await?;

// List all runs
let runs = api.list_runs().await?;
```

### TypeScript SDK

```typescript
// Create SDK instance
const fleet = new FleetBenchmark({
  outputBaseDir: './fleet-results',
  defaultConcurrency: 10,
});

// Execute fleet benchmark
const handle = await fleet.executeFleet('./manifest.json');

// Wait for completion (polling)
const results = await fleet.waitForCompletion(handle.runId);

// Retrieve results later
const results = await fleet.getFleetResults('run-id');

// List all runs
const runs = await fleet.listRuns();
```

### CLI

```bash
# Execute fleet benchmark
llm-test-bench fleet --manifest <path> [OPTIONS]

# Options:
#   --output <dir>          Output directory (default: ./fleet-results)
#   --concurrency <n>       Concurrent requests (default: 5)
#   --delay <ms>            Request delay in milliseconds
#   --config <path>         Custom configuration file
#   --wait                  Wait for completion
#   --format <type>         Output format (json|summary|quiet)
#   --save-responses        Save individual responses
```

## Integration with Existing Codebase

The Fleet API integrates seamlessly with existing test-bench infrastructure:

- **Uses existing BenchmarkRunner** for individual repository execution
- **Leverages FleetBenchmarkResults** for aggregation (already implemented)
- **Reuses provider configuration** from llm-test-bench.toml
- **Compatible with existing datasets** and dataset loaders
- **Exports using existing exporters** (JSON, CSV, HTML)

## No Modifications Required to Simulators

The API is designed so simulators can integrate without any changes to their existing logic:

1. Simulator calls Fleet API with manifest path
2. Receives run_id and artifact paths immediately
3. Continues with other work
4. Polls or waits for results when needed
5. Retrieves results from known artifact locations

## Testing

### Unit Tests

Included in:
- `core/src/benchmarks/fleet_api.rs` (Rust tests)
- Tests for manifest validation, run ID generation, configuration

### Integration Tests

To be added in Phase 4:
- `core/tests/fleet_api_integration_test.rs`
- End-to-end fleet execution tests
- TypeScript SDK integration tests

## Future Enhancements

Potential improvements for future iterations:

1. **Status Monitoring**: Real-time progress updates via WebSocket or HTTP endpoint
2. **Partial Results**: Stream results as repositories complete
3. **Resume Capability**: Resume failed fleet executions
4. **Distributed Execution**: Run fleet across multiple machines
5. **Result Comparison**: Built-in comparison of fleet runs
6. **Cost Estimation**: Pre-execution cost estimation

## Files Modified/Created

### Created Files

1. `core/src/benchmarks/fleet_api.rs` - Core Rust API implementation
2. `cli/src/commands/fleet.rs` - CLI command implementation
3. `src/core/fleet-benchmark.ts` - TypeScript SDK wrapper
4. `docs/FLEET_API.md` - Comprehensive API documentation
5. `examples/fleet-api-example.ts` - Usage examples
6. `examples/fleet-manifest-example.json` - Example manifest
7. `FLEET_API_IMPLEMENTATION.md` - This file

### Modified Files

1. `core/src/benchmarks/mod.rs` - Added fleet_api module exports
2. `cli/src/commands/mod.rs` - Added fleet module
3. `cli/src/main.rs` - Added Fleet command to CLI
4. `src/index.ts` - Exported FleetBenchmark class and types

## Deliverables Checklist

- [x] Rust API module (`fleet_api.rs`)
- [x] CLI command integration (`fleet.rs`)
- [x] TypeScript SDK wrapper (`fleet-benchmark.ts`)
- [x] Comprehensive documentation (`FLEET_API.md`)
- [x] Usage examples (`fleet-api-example.ts`)
- [x] Example fleet manifest (`fleet-manifest-example.json`)
- [x] Integration with existing CLI structure
- [x] Type definitions for TypeScript
- [ ] Integration tests (pending - Phase 4)
- [ ] End-to-end test with actual datasets (pending - Phase 4)

## Conclusion

The Fleet Benchmark API provides a clean, programmatic interface for executing fleet-wide benchmarks with immediate run identification and artifact location tracking. It integrates seamlessly with the existing test-bench infrastructure while enabling simulators to trigger benchmarks without requiring any modifications to their code.

The API supports both synchronous and asynchronous execution modes, making it suitable for a wide range of use cases from CI/CD pipelines to long-running simulation workflows.

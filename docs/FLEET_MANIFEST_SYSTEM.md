# Fleet Manifest System

The Fleet Manifest System enables orchestrated benchmarking across multiple repositories with different adapters, scenarios, and providers. This system maintains full backward compatibility with test-bench's existing infrastructure while adding powerful cross-repository orchestration capabilities.

## Table of Contents

- [Overview](#overview)
- [Architecture](#architecture)
- [Fleet Manifest Schema](#fleet-manifest-schema)
- [Repository Adapters](#repository-adapters)
- [Usage Examples](#usage-examples)
- [Integration Points](#integration-points)
- [Output Structure](#output-structure)

## Overview

The Fleet Manifest System consists of three core components:

1. **Fleet Manifest** (`fleet_manifest.rs`) - JSON/YAML schema defining what to benchmark
2. **Repository Adapters** (`fleet_adapters.rs`) - Thin translation layers for different repo types
3. **Fleet Runner** (`fleet_runner.rs`) - Orchestration engine that executes the plan

### Key Principles

- **Thin Adapters**: No business logic, only dataset discovery and loading
- **Reuse Existing Infrastructure**: Builds on `BenchmarkRunner`, `BenchmarkResults`, and providers
- **Deterministic Outputs**: Reproducible run identifiers and structured artifacts
- **Backward Compatible**: Existing test-bench functionality unchanged

## Architecture

```
┌─────────────────────┐
│  Fleet Manifest     │  ← JSON/YAML configuration
│  (fleet.json)       │
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│  FleetRunner        │  ← Orchestration engine
└──────────┬──────────┘
           │
           ├──────────────┬──────────────┬──────────────┐
           ▼              ▼              ▼              ▼
    ┌──────────┐   ┌──────────┐   ┌──────────┐   ┌──────────┐
    │ Adapter  │   │ Adapter  │   │ Adapter  │   │ Adapter  │
    │ (native) │   │ (generic)│   │ (native) │   │ (custom) │
    └─────┬────┘   └─────┬────┘   └─────┬────┘   └─────┬────┘
          │              │              │              │
          ▼              ▼              ▼              ▼
    ┌──────────┐   ┌──────────┐   ┌──────────┐   ┌──────────┐
    │ Dataset  │   │ Dataset  │   │ Dataset  │   │ Dataset  │
    └─────┬────┘   └─────┬────┘   └─────┬────┘   └─────┬────┘
          │              │              │              │
          └──────────────┴──────────────┴──────────────┘
                         │
                         ▼
                ┌─────────────────┐
                │ BenchmarkRunner │  ← Existing test-bench runner
                └────────┬────────┘
                         │
                         ▼
                ┌─────────────────┐
                │ FleetResults    │  ← Aggregated results
                └─────────────────┘
```

## Fleet Manifest Schema

### Top-Level Structure

```json
{
  "fleet_id": "agentics-fleet-2025",
  "version": "1.0",
  "description": "Full Agentics system benchmark",
  "repositories": [...],
  "providers": [...],
  "scenario_profiles": {...},
  "output": {...},
  "global_settings": {...}
}
```

### Fields

#### `fleet_id` (required)
Unique identifier for this fleet. Used to generate deterministic run IDs.

```json
"fleet_id": "agentics-fleet-2025"
```

#### `version` (required)
Manifest schema version. Currently only `"1.0"` is supported.

```json
"version": "1.0"
```

#### `repositories` (required)
Array of repository configurations to benchmark.

```json
"repositories": [
  {
    "repo_id": "test-bench",
    "path": ".",
    "adapter": "native",
    "scenarios": ["coding", "reasoning"],
    "metadata": {
      "team": "core",
      "priority": "high"
    }
  }
]
```

**Repository Fields:**
- `repo_id` (required): Unique identifier for this repository
- `path` (required): Path to the repository (relative or absolute)
- `git_url` (optional): Git URL for external repositories
- `adapter` (required): Adapter type (`"native"`, `"generic"`, or custom)
- `scenarios` (required): List of scenario names to run
- `metadata` (optional): Custom metadata key-value pairs

#### `providers` (required)
Array of provider specifications in `"provider:model"` format.

```json
"providers": [
  "openai:gpt-4",
  "anthropic:claude-3-opus-20240229"
]
```

#### `scenario_profiles` (required)
Scenario configurations defining execution parameters.

```json
"scenario_profiles": {
  "coding": {
    "dataset": "coding-tasks",
    "concurrency": 5,
    "num_examples": 100,
    "request_delay_ms": 100,
    "settings": {
      "temperature": 0.7
    }
  }
}
```

**Scenario Fields:**
- `dataset` (required): Dataset name to load (adapter-dependent)
- `concurrency` (optional): Parallel execution limit (default: 5)
- `num_examples` (optional): Number of examples to run (default: all)
- `request_delay_ms` (optional): Delay between requests in milliseconds
- `settings` (optional): Scenario-specific custom settings

#### `output` (required)
Output configuration for results.

```json
"output": {
  "base_dir": "./fleet-results",
  "formats": ["json", "csv", "html"],
  "save_responses": true,
  "generate_reports": true
}
```

**Output Fields:**
- `base_dir` (required): Base directory for all outputs
- `formats` (optional): Output formats (default: `["json", "csv"]`)
- `save_responses` (optional): Save individual responses (default: `true`)
- `generate_reports` (optional): Generate summary reports (default: `true`)

#### `global_settings` (optional)
Global execution settings.

```json
"global_settings": {
  "continue_on_failure": true,
  "random_seed": 42,
  "test_timeout_seconds": 120,
  "max_retries": 3,
  "custom": {}
}
```

## Repository Adapters

Adapters provide a thin translation layer between repository structures and test-bench's Dataset format.

### Native Adapter

For repositories following test-bench conventions:
- Looks for datasets in `./datasets/` directory
- Supports `.json`, `.yaml`, `.yml` formats
- Direct compatibility with existing test-bench datasets

```rust
use llm_test_bench_core::benchmarks::fleet_adapters::NativeAdapter;

let adapter = NativeAdapter::new(Path::new("."));
let datasets = adapter.discover_datasets()?;
let dataset = adapter.load_dataset("coding-tasks").await?;
```

### Generic Adapter

For external repositories with standard dataset formats:
- Searches in: root, `./data/`, `./datasets/`, `./benchmarks/`
- Auto-validates datasets before reporting them
- Flexible path resolution

```rust
use llm_test_bench_core::benchmarks::fleet_adapters::GenericAdapter;

let adapter = GenericAdapter::new(Path::new("./external-repo"));
let datasets = adapter.discover_datasets()?;
```

### Custom Adapters

Implement the `RepositoryAdapter` trait:

```rust
use async_trait::async_trait;
use llm_test_bench_core::benchmarks::fleet_adapters::{RepositoryAdapter, AdapterError};

struct CustomAdapter {
    base_path: PathBuf,
}

#[async_trait]
impl RepositoryAdapter for CustomAdapter {
    fn adapter_type(&self) -> &str {
        "custom"
    }

    fn discover_datasets(&self) -> Result<Vec<String>, AdapterError> {
        // Custom discovery logic
        Ok(vec!["dataset1".to_string()])
    }

    async fn load_dataset(&self, dataset_id: &str) -> Result<Dataset, AdapterError> {
        // Custom loading logic
        todo!()
    }

    fn base_path(&self) -> &Path {
        &self.base_path
    }
}
```

## Usage Examples

### Programmatic Usage

```rust
use llm_test_bench_core::benchmarks::{FleetRunner, FleetManifest};
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load manifest
    let manifest = FleetManifest::load_from_file(
        Path::new("./fleet-manifest.json")
    )?;

    // Create runner and execute
    let runner = FleetRunner::new();
    let results = runner.run(&manifest).await?;

    // Access results
    println!("Fleet: {}", results.fleet_id);
    println!("Success Rate: {:.2}%", results.fleet_summary.success_rate * 100.0);
    println!("Total Cost: ${:.4}", results.fleet_summary.total_cost);

    // Per-repository analysis
    for repo in &results.repository_results {
        println!("Repo {}: {:.2}% success",
                 repo.repository_id,
                 repo.results.summary.success_rate * 100.0);
    }

    Ok(())
}
```

### CLI Usage (Future)

```bash
# Run fleet benchmark from manifest
test-bench fleet run --manifest ./fleet-manifest.json

# Validate manifest without running
test-bench fleet validate --manifest ./fleet-manifest.json

# Generate sample manifest
test-bench fleet init --output ./my-fleet.json
```

## Integration Points

The Fleet Manifest System integrates with existing test-bench components:

### BenchmarkRunner
- **Location**: `/workspaces/test-bench/core/src/benchmarks/runner.rs`
- **Integration**: FleetRunner uses BenchmarkRunner for actual execution
- **No Changes**: Existing API unchanged

### BenchmarkResults
- **Location**: `/workspaces/test-bench/core/src/benchmarks/results.rs`
- **Integration**: Individual results aggregated into FleetBenchmarkResults
- **Extension**: FleetBenchmarkResults wraps multiple BenchmarkResults

### DatasetLoader
- **Location**: `/workspaces/test-bench/datasets/src/loader.rs`
- **Integration**: Adapters use DatasetLoader for dataset loading
- **No Changes**: Existing API unchanged

### ProviderFactory
- **Location**: `/workspaces/test-bench/core/src/providers/factory.rs`
- **Integration**: FleetRunner uses ProviderFactory to create providers
- **No Changes**: Existing API unchanged

## Output Structure

Fleet benchmarks generate a deterministic directory structure:

```
{base_dir}/
└── {fleet_id}-{timestamp}-{hash}/
    ├── fleet-results.json          # Aggregated fleet results
    ├── fleet-results.yaml          # YAML format (optional)
    ├── csv/                        # CSV exports (optional)
    │   ├── fleet-summary.csv
    │   ├── provider-breakdown.csv
    │   └── category-breakdown.csv
    └── {repo_id}/                  # Per-repository results
        └── {provider}_{model}/     # Per-provider results
            └── {scenario}/         # Per-scenario results
                ├── test-1.json
                ├── test-2.json
                └── ...
```

### Run ID Format

Run IDs are deterministic and follow the format:
```
{fleet_id}-{timestamp}-{hash}
```

Example: `agentics-fleet-2025-20250131-143022-a1b2c3d4`

### Fleet Results Schema

```json
{
  "fleet_id": "agentics-fleet-2025",
  "timestamp": "2025-01-31T14:30:22Z",
  "total_repositories": 3,
  "repository_results": [
    {
      "repository_id": "test-bench",
      "repository_name": "coding-tasks",
      "provider_name": "openai",
      "results": { /* BenchmarkResults */ },
      "repository_metadata": {
        "run_id": "...",
        "adapter": "native"
      }
    }
  ],
  "fleet_summary": {
    "total_repositories": 3,
    "total_tests": 500,
    "total_succeeded": 475,
    "success_rate": 0.95,
    "total_cost": 12.50,
    /* ... more metrics ... */
  },
  "provider_breakdown": {
    "openai": {
      "repository_count": 2,
      "total_tests": 300,
      "success_rate": 0.94,
      /* ... */
    }
  },
  "category_breakdown": {
    "coding": {
      "total_tests": 200,
      "success_rate": 0.96,
      /* ... */
    }
  },
  "metadata": {
    "total_tests": 500,
    "aggregation_version": "1.0.0",
    /* ... */
  }
}
```

## Error Handling

The Fleet Manifest System provides detailed error messages:

```rust
match runner.run(&manifest).await {
    Ok(results) => { /* Success */ }
    Err(FleetRunnerError::ManifestError(e)) => {
        eprintln!("Manifest error: {}", e);
    }
    Err(FleetRunnerError::AdapterError(e)) => {
        eprintln!("Adapter error: {}", e);
    }
    Err(FleetRunnerError::RepositoryFailed(repo_id, msg)) => {
        eprintln!("Repository '{}' failed: {}", repo_id, msg);
    }
    Err(e) => {
        eprintln!("Fleet error: {}", e);
    }
}
```

## Best Practices

### 1. Use Descriptive Fleet IDs
```json
"fleet_id": "prod-weekly-benchmark-2025"  // Good
"fleet_id": "test"                         // Bad
```

### 2. Set Appropriate Concurrency
```json
"scenario_profiles": {
  "heavy": {
    "concurrency": 2,  // Lower for resource-intensive tasks
    "dataset": "large-dataset"
  },
  "light": {
    "concurrency": 10,  // Higher for quick tasks
    "dataset": "small-dataset"
  }
}
```

### 3. Use Request Delays to Avoid Rate Limits
```json
"scenario_profiles": {
  "production": {
    "request_delay_ms": 100,  // 10 requests/second max
    "dataset": "prod-dataset"
  }
}
```

### 4. Organize Repositories by Team/Function
```json
"repositories": [
  {
    "repo_id": "core-benchmarks",
    "metadata": {
      "team": "platform",
      "category": "core"
    }
  },
  {
    "repo_id": "integration-tests",
    "metadata": {
      "team": "integrations",
      "category": "external"
    }
  }
]
```

### 5. Version Your Manifests
```bash
fleet-manifest-v1.0.0.json
fleet-manifest-v1.1.0.json
```

## Future Enhancements

Potential future additions:

1. **Remote Repository Cloning**: Auto-clone git repositories
2. **Parallel Repository Execution**: Execute repositories in parallel
3. **Conditional Execution**: Skip scenarios based on conditions
4. **Dependency Management**: Declare inter-repository dependencies
5. **Custom Reporters**: Pluggable reporting engines
6. **Live Progress Tracking**: Real-time execution monitoring
7. **Result Comparison**: Compare fleet runs over time
8. **Cost Budgets**: Fail if cost exceeds threshold

## See Also

- [Fleet Aggregation System](./FLEET_AGGREGATION.md) - Result aggregation details
- [Benchmark Configuration](../README.md#configuration) - Individual benchmark config
- [Provider Factory](../core/src/providers/factory.rs) - Provider creation
- [Dataset Schema](../datasets/README.md) - Dataset format specification

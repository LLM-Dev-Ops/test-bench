# Fleet Manifest System - Quick Start Guide

Get started with fleet benchmarking in 5 minutes.

## 1. Create a Fleet Manifest

Create `fleet.json`:

```json
{
  "fleet_id": "my-first-fleet",
  "version": "1.0",
  "description": "My first fleet benchmark",
  "repositories": [
    {
      "repo_id": "main-repo",
      "path": ".",
      "adapter": "native",
      "scenarios": ["quick-test"]
    }
  ],
  "providers": ["openai:gpt-4"],
  "scenario_profiles": {
    "quick-test": {
      "dataset": "my-dataset",
      "concurrency": 5
    }
  },
  "output": {
    "base_dir": "./fleet-results",
    "formats": ["json", "csv"]
  }
}
```

## 2. Write the Code

Create `my_fleet_runner.rs`:

```rust
use llm_test_bench_core::benchmarks::{FleetRunner, FleetManifest};
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load and run
    let manifest = FleetManifest::load_from_file(Path::new("./fleet.json"))?;
    let runner = FleetRunner::new();
    let results = runner.run(&manifest).await?;

    // Show results
    println!("Fleet: {}", results.fleet_id);
    println!("Success Rate: {:.2}%", results.fleet_summary.success_rate * 100.0);
    println!("Total Cost: ${:.4}", results.fleet_summary.total_cost);

    Ok(())
}
```

## 3. Run It

```bash
cargo run --bin my_fleet_runner
```

## 4. View Results

Results are saved to `./fleet-results/{run_id}/`:

```bash
# View fleet summary
cat fleet-results/my-first-fleet-*/fleet-results.json | jq '.fleet_summary'

# View CSV reports
ls fleet-results/my-first-fleet-*/csv/
```

## Common Patterns

### Multi-Repository Fleet

```json
{
  "repositories": [
    {
      "repo_id": "repo1",
      "path": "./repo1",
      "adapter": "native",
      "scenarios": ["scenario1"]
    },
    {
      "repo_id": "repo2",
      "path": "./repo2",
      "adapter": "generic",
      "scenarios": ["scenario2"]
    }
  ]
}
```

### Multiple Providers

```json
{
  "providers": [
    "openai:gpt-4",
    "openai:gpt-3.5-turbo",
    "anthropic:claude-3-opus-20240229"
  ]
}
```

### Multiple Scenarios

```json
{
  "scenario_profiles": {
    "quick": {
      "dataset": "small-dataset",
      "concurrency": 10,
      "num_examples": 10
    },
    "comprehensive": {
      "dataset": "full-dataset",
      "concurrency": 5,
      "num_examples": 1000,
      "request_delay_ms": 100
    }
  }
}
```

## Environment Variables

Set API keys before running:

```bash
export OPENAI_API_KEY="sk-..."
export ANTHROPIC_API_KEY="sk-ant-..."
```

## Validation

Validate your manifest programmatically:

```rust
let manifest = FleetManifest::load_from_file(Path::new("./fleet.json"))?;
manifest.validate()?;
println!("Manifest is valid!");
```

## Error Handling

```rust
match runner.run(&manifest).await {
    Ok(results) => println!("Success!"),
    Err(e) => eprintln!("Error: {}", e),
}
```

## Next Steps

- Read [FLEET_MANIFEST_SYSTEM.md](./FLEET_MANIFEST_SYSTEM.md) for complete documentation
- Check [examples/fleet_runner_example.rs](../examples/fleet_runner_example.rs) for advanced usage
- See [FLEET_IMPLEMENTATION_SUMMARY.md](./FLEET_IMPLEMENTATION_SUMMARY.md) for architecture details

## Troubleshooting

### "Dataset not found"
- Verify `dataset` name matches file in `./datasets/` directory
- Check adapter type matches repository structure

### "Provider error"
- Ensure API keys are set in environment
- Verify provider:model format is correct

### "Scenario not found"
- Check scenario name in `repositories[].scenarios` exists in `scenario_profiles`

### "Validation failed"
- Run manifest validation separately to see detailed errors
- Check all required fields are present
- Verify version is "1.0"

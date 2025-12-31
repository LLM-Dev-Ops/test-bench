# Simulator Integration Guide

## Quick Start

This guide shows how to integrate the Fleet Benchmark API into your simulator with minimal code changes.

## Installation

### TypeScript/JavaScript

```bash
npm install @llm-dev-ops/test-bench-sdk
```

### Rust

```toml
[dependencies]
llm-test-bench-core = "0.2.0"
```

## Basic Integration Pattern

### Step 1: Create Fleet Manifest

Create a `fleet-manifest.json` file describing your repositories:

```json
{
  "fleet_id": "my-simulator-fleet",
  "description": "My simulator fleet benchmarks",
  "repositories": [
    {
      "id": "repo-1",
      "name": "Repository 1",
      "dataset_path": "./datasets/repo1.json"
    },
    {
      "id": "repo-2",
      "name": "Repository 2",
      "dataset_path": "./datasets/repo2.json"
    }
  ],
  "providers": ["openai", "anthropic"],
  "config": {
    "concurrency": 10
  }
}
```

### Step 2: Execute Fleet Benchmark

**TypeScript:**

```typescript
import { FleetBenchmark } from 'llm-test-bench';

const fleet = new FleetBenchmark({
  outputBaseDir: './fleet-results',
  defaultConcurrency: 10,
});

// Execute and get run ID immediately
const handle = await fleet.executeFleet('./fleet-manifest.json');

console.log('Run ID:', handle.runId);
console.log('Artifacts:', handle.artifactBaseDir);

// Continue with other simulator work...
```

**Rust:**

```rust
use llm_test_bench_core::benchmarks::fleet_api::{FleetBenchmarkAPI, FleetConfig};
use std::path::PathBuf;

let config = FleetConfig::new(PathBuf::from("./fleet-results"));
let api = FleetBenchmarkAPI::new(config);

let handle = api.execute_fleet_benchmark(&PathBuf::from("./fleet-manifest.json")).await?;

println!("Run ID: {}", handle.run_id);
println!("Artifacts: {}", handle.artifact_base_dir.display());

// Continue with other simulator work...
```

### Step 3: Retrieve Results

**TypeScript:**

```typescript
// Wait for completion
const results = await fleet.waitForCompletion(handle.runId);

console.log('Success rate:', results.fleet_summary.success_rate);
console.log('Total cost:', results.fleet_summary.total_cost);

// Or retrieve later
const results = await fleet.getFleetResults('my-simulator-fleet-20231231-123456');
```

**Rust:**

```rust
// Wait for completion
let results = handle.execution_future.await??;

println!("Success rate: {:.2}%", results.fleet_summary.success_rate * 100.0);

// Or retrieve later
let results = api.get_fleet_results("my-simulator-fleet-20231231-123456").await?;
```

## Complete Simulator Example

```typescript
import { FleetBenchmark } from 'llm-test-bench';

export class MySimulator {
  private fleet: FleetBenchmark;
  private runs: Map<string, string>; // scenario -> run_id

  constructor() {
    this.fleet = new FleetBenchmark({
      outputBaseDir: './simulation-results',
      defaultConcurrency: 15,
    });
    this.runs = new Map();
  }

  /**
   * Start a fleet benchmark for a scenario
   */
  async startScenario(scenarioName: string): Promise<string> {
    const manifestPath = `./scenarios/${scenarioName}/fleet-manifest.json`;

    // Execute fleet (returns immediately)
    const handle = await this.fleet.executeFleet(manifestPath);

    // Track run
    this.runs.set(scenarioName, handle.runId);

    return handle.runId;
  }

  /**
   * Check if scenario is complete
   */
  async isScenarioComplete(scenarioName: string): Promise<boolean> {
    const runId = this.runs.get(scenarioName);
    if (!runId) return false;

    try {
      await this.fleet.getFleetResults(runId);
      return true;
    } catch {
      return false;
    }
  }

  /**
   * Get scenario results
   */
  async getScenarioResults(scenarioName: string) {
    const runId = this.runs.get(scenarioName);
    if (!runId) throw new Error(`No run for scenario: ${scenarioName}`);

    const results = await this.fleet.getFleetResults(runId);

    return {
      successRate: results.fleet_summary.success_rate,
      totalCost: results.fleet_summary.total_cost,
      avgLatency: results.fleet_summary.avg_duration_ms,
      repositories: results.total_repositories,
      providers: Object.keys(results.provider_breakdown),
    };
  }

  /**
   * Run complete simulation
   */
  async runSimulation() {
    console.log('Starting simulation...');

    // Start multiple scenarios
    await this.startScenario('baseline');
    await this.startScenario('optimized');
    await this.startScenario('experimental');

    console.log('All scenarios started, continuing simulation...');

    // Simulate other work
    await this.simulateWork();

    // Wait for all scenarios to complete
    for (const scenario of this.runs.keys()) {
      console.log(`Waiting for ${scenario}...`);
      const results = await this.getScenarioResults(scenario);
      console.log(`  Success rate: ${(results.successRate * 100).toFixed(2)}%`);
      console.log(`  Total cost: $${results.totalCost.toFixed(4)}`);
    }

    console.log('Simulation complete!');
  }

  private async simulateWork() {
    // Your simulation logic here
    await new Promise((resolve) => setTimeout(resolve, 5000));
  }
}

// Usage
const simulator = new MySimulator();
await simulator.runSimulation();
```

## CLI Integration

If you prefer using the CLI from your simulator:

```typescript
import { exec } from 'child_process';
import { promisify } from 'util';

const execAsync = promisify(exec);

async function runFleetBenchmark(manifestPath: string): Promise<string> {
  // Execute fleet benchmark (quiet mode returns just run_id)
  const { stdout } = await execAsync(
    `llm-test-bench fleet --manifest ${manifestPath} --format quiet`
  );

  const runId = stdout.trim();
  console.log('Fleet benchmark started:', runId);

  return runId;
}

async function getFleetResults(runId: string) {
  const resultsPath = `./fleet-results/${runId}/fleet-results.json`;

  // Wait for results file to exist
  while (!fs.existsSync(resultsPath)) {
    await new Promise((resolve) => setTimeout(resolve, 2000));
  }

  const results = JSON.parse(fs.readFileSync(resultsPath, 'utf-8'));
  return results;
}
```

## Key Features for Simulators

### 1. Non-Blocking Execution

The API returns immediately with a run ID, allowing your simulator to continue with other work:

```typescript
const handle = await fleet.executeFleet('./manifest.json');
// Returns immediately - execution happens in background
```

### 2. Deterministic Run IDs

Run IDs follow the pattern `{fleet_id}-{timestamp}`, making them predictable and easy to track:

```
my-simulator-fleet-20231231-123456
```

### 3. Known Artifact Locations

Artifact paths are provided immediately, even before execution completes:

```typescript
{
  runId: "my-simulator-fleet-20231231-123456",
  artifactBaseDir: "./fleet-results/my-simulator-fleet-20231231-123456",
  metadata: {
    expectedArtifacts: [
      "./fleet-results/.../fleet-results.json",
      "./fleet-results/.../fleet-summary.csv",
      // ... per-repository artifacts
    ]
  }
}
```

### 4. Multiple Execution Modes

**Async Mode** (default): Returns immediately, execution in background
```typescript
const handle = await fleet.executeFleet('./manifest.json');
```

**Sync Mode**: Waits for completion before returning
```typescript
const handle = await fleet.executeFleet('./manifest.json', { wait: true });
```

## Best Practices

1. **Use Unique Fleet IDs**: Include version or environment in fleet_id
   ```json
   "fleet_id": "my-sim-v2-prod"
   ```

2. **Set Appropriate Concurrency**: Balance speed vs. rate limits
   ```typescript
   new FleetBenchmark({ defaultConcurrency: 10 })
   ```

3. **Monitor Costs**: Check total_cost in results
   ```typescript
   const results = await fleet.getFleetResults(runId);
   console.log('Cost:', results.fleet_summary.total_cost);
   ```

4. **Archive Old Runs**: Clean up old results periodically
   ```typescript
   const runs = await fleet.listRuns();
   // Archive or delete old runs
   ```

5. **Use Metadata**: Add custom metadata to repositories
   ```json
   {
     "id": "repo-1",
     "metadata": {
       "team": "planning",
       "priority": "high"
     }
   }
   ```

## Error Handling

```typescript
try {
  const handle = await fleet.executeFleet('./manifest.json');

  // Wait for completion with timeout
  const results = await fleet.waitForCompletion(handle.runId, {
    timeoutMs: 600000, // 10 minutes
    pollIntervalMs: 2000, // Check every 2 seconds
  });

  if (results.fleet_summary.success_rate < 0.95) {
    console.error('Fleet benchmark failed quality threshold');
  }
} catch (error) {
  console.error('Fleet benchmark error:', error);
  // Handle error appropriately
}
```

## FAQ

**Q: Do I need to modify my simulator code?**
A: No! The API is designed to integrate without requiring changes to your existing simulator logic.

**Q: How long does a fleet benchmark take?**
A: Depends on the number of repositories, providers, and tests. Typically 5-30 minutes for a medium-sized fleet.

**Q: Can I run multiple fleet benchmarks in parallel?**
A: Yes! Each execution gets a unique run_id and separate artifact directory.

**Q: How do I know when execution is complete?**
A: Use `waitForCompletion()` to poll, or check if `fleet-results.json` exists in the artifact directory.

**Q: What if execution fails?**
A: Partial results are still saved. Check `fleet-results.json` for whatever data was collected.

## Support

For more information, see:
- [Fleet API Documentation](./docs/FLEET_API.md)
- [Fleet Metrics Documentation](./docs/FLEET_METRICS.md)
- [Examples](./examples/fleet-api-example.ts)

## License

MIT

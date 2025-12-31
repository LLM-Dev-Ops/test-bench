/**
 * Fleet Benchmark API Usage Examples
 *
 * This file demonstrates how to use the Fleet Benchmark API for
 * simulator integration and automation workflows.
 */

import { FleetBenchmark } from '../src/index.js';
import type { FleetExecutionHandle, FleetBenchmarkResults } from '../src/index.js';

/**
 * Example 1: Basic Fleet Execution
 *
 * Execute a fleet benchmark and get immediate run identifier.
 */
async function example1_BasicExecution() {
  console.log('\n=== Example 1: Basic Fleet Execution ===\n');

  // Initialize Fleet Benchmark API
  const fleet = new FleetBenchmark({
    outputBaseDir: './fleet-results',
    defaultConcurrency: 10,
    saveResponses: true,
  });

  // Execute fleet benchmark (returns immediately)
  const handle = await fleet.executeFleet('./examples/fleet-manifest-example.json');

  console.log('Fleet execution started:');
  console.log('  Run ID:', handle.runId);
  console.log('  Artifacts:', handle.artifactBaseDir);
  console.log('  Fleet:', handle.metadata.fleetId);
  console.log('  Repositories:', handle.metadata.repositoryCount);
  console.log('  Providers:', handle.metadata.providers.join(', '));
  console.log('  Started:', handle.metadata.startedAt);

  console.log('\nFleet benchmark executing in background...');
  console.log('Use fleet.waitForCompletion() or fleet.getFleetResults() to retrieve results.');
}

/**
 * Example 2: Execute and Wait for Completion
 *
 * Execute a fleet benchmark and wait for it to complete.
 */
async function example2_ExecuteAndWait() {
  console.log('\n=== Example 2: Execute and Wait for Completion ===\n');

  const fleet = new FleetBenchmark({
    outputBaseDir: './fleet-results',
    defaultConcurrency: 5,
  });

  console.log('Executing fleet benchmark (waiting for completion)...');

  // Execute and wait for completion
  const handle = await fleet.executeFleet('./examples/fleet-manifest-example.json', {
    wait: true,
    format: 'json',
  });

  console.log('\nFleet benchmark completed!');
  console.log('  Run ID:', handle.runId);

  // Retrieve results
  const results = await fleet.getFleetResults(handle.runId);

  console.log('\nFleet Summary:');
  console.log('  Total Repositories:', results.total_repositories);
  console.log('  Total Tests:', results.fleet_summary.total_tests);
  console.log('  Success Rate:', `${(results.fleet_summary.success_rate * 100).toFixed(2)}%`);
  console.log('  Total Cost:', `$${results.fleet_summary.total_cost.toFixed(4)}`);
  console.log('  Avg Latency:', `${results.fleet_summary.avg_duration_ms.toFixed(0)}ms`);

  console.log('\nProvider Breakdown:');
  for (const [provider, stats] of Object.entries(results.provider_breakdown)) {
    console.log(`  ${provider}:`);
    console.log(`    Repositories: ${stats.repository_count}`);
    console.log(`    Success Rate: ${(stats.success_rate * 100).toFixed(2)}%`);
    console.log(`    Total Cost: $${stats.total_cost.toFixed(4)}`);
  }
}

/**
 * Example 3: Polling for Completion
 *
 * Execute a fleet benchmark and poll for completion.
 */
async function example3_PollingForCompletion() {
  console.log('\n=== Example 3: Polling for Completion ===\n');

  const fleet = new FleetBenchmark({
    outputBaseDir: './fleet-results',
    defaultConcurrency: 10,
  });

  // Start execution (non-blocking)
  const handle = await fleet.executeFleet('./examples/fleet-manifest-example.json');
  console.log('Fleet execution started:', handle.runId);

  console.log('Polling for completion...');

  // Wait for completion with custom timeout
  const results = await fleet.waitForCompletion(handle.runId, {
    timeoutMs: 600000, // 10 minutes
    pollIntervalMs: 2000, // Check every 2 seconds
  });

  console.log('Fleet completed!');
  console.log('Success rate:', `${(results.fleet_summary.success_rate * 100).toFixed(2)}%`);
}

/**
 * Example 4: List and Retrieve Historical Runs
 *
 * List all available fleet runs and retrieve results.
 */
async function example4_ListHistoricalRuns() {
  console.log('\n=== Example 4: List Historical Runs ===\n');

  const fleet = new FleetBenchmark({
    outputBaseDir: './fleet-results',
  });

  // List all available runs
  const runs = await fleet.listRuns();
  console.log(`Found ${runs.length} fleet runs:\n`);

  for (const runId of runs) {
    try {
      const results = await fleet.getFleetResults(runId);

      console.log(`${runId}:`);
      console.log(`  Fleet: ${results.fleet_id}`);
      console.log(`  Timestamp: ${results.timestamp}`);
      console.log(`  Repositories: ${results.total_repositories}`);
      console.log(`  Success Rate: ${(results.fleet_summary.success_rate * 100).toFixed(2)}%`);
      console.log(`  Total Cost: $${results.fleet_summary.total_cost.toFixed(4)}`);
      console.log();
    } catch (error) {
      console.log(`${runId}: Still running or incomplete`);
      console.log();
    }
  }
}

/**
 * Example 5: Simulator Integration
 *
 * Demonstrates how a simulator would integrate with the Fleet API.
 */
class AgentSimulator {
  private fleet: FleetBenchmark;
  private activeRuns: Map<string, FleetExecutionHandle>;

  constructor() {
    this.fleet = new FleetBenchmark({
      outputBaseDir: './simulation-results',
      defaultConcurrency: 15,
      saveResponses: true,
      requestDelayMs: 100,
    });
    this.activeRuns = new Map();
  }

  /**
   * Start a fleet benchmark for a scenario
   */
  async startScenario(scenarioName: string): Promise<string> {
    console.log(`Starting scenario: ${scenarioName}`);

    const manifestPath = `./scenarios/${scenarioName}/fleet-manifest.json`;

    // Execute fleet (non-blocking)
    const handle = await this.fleet.executeFleet(manifestPath);

    // Track active run
    this.activeRuns.set(handle.runId, handle);

    console.log(`  Run ID: ${handle.runId}`);
    console.log(`  Artifacts: ${handle.artifactBaseDir}`);

    return handle.runId;
  }

  /**
   * Check status of a fleet run
   */
  async getStatus(runId: string): Promise<'running' | 'completed' | 'not_found'> {
    try {
      await this.fleet.getFleetResults(runId);
      return 'completed';
    } catch (error) {
      return this.activeRuns.has(runId) ? 'running' : 'not_found';
    }
  }

  /**
   * Get metrics for a completed fleet run
   */
  async getMetrics(runId: string) {
    const results = await this.fleet.getFleetResults(runId);

    return {
      runId: results.fleet_id,
      timestamp: results.timestamp,
      repositories: results.total_repositories,
      successRate: results.fleet_summary.success_rate,
      totalTests: results.fleet_summary.total_tests,
      totalCost: results.fleet_summary.total_cost,
      avgLatency: results.fleet_summary.avg_duration_ms,
      p95Latency: results.fleet_summary.p95_duration_ms,
      providerBreakdown: results.provider_breakdown,
      categoryBreakdown: results.category_breakdown,
    };
  }

  /**
   * Compare multiple scenario runs
   */
  async compareScenarios(runIds: string[]) {
    console.log('\n=== Scenario Comparison ===\n');

    const comparisons = [];

    for (const runId of runIds) {
      const metrics = await this.getMetrics(runId);
      comparisons.push(metrics);

      console.log(`${metrics.runId}:`);
      console.log(`  Success Rate: ${(metrics.successRate * 100).toFixed(2)}%`);
      console.log(`  Total Cost: $${metrics.totalCost.toFixed(4)}`);
      console.log(`  Avg Latency: ${metrics.avgLatency.toFixed(0)}ms`);
      console.log(`  P95 Latency: ${metrics.p95Latency.toFixed(0)}ms`);
      console.log();
    }

    return comparisons;
  }
}

async function example5_SimulatorIntegration() {
  console.log('\n=== Example 5: Simulator Integration ===\n');

  const simulator = new AgentSimulator();

  // Start multiple scenarios
  const scenario1 = await simulator.startScenario('baseline');
  const scenario2 = await simulator.startScenario('optimized');

  console.log('\nScenarios started, continuing with other work...\n');

  // Simulate other work
  await new Promise((resolve) => setTimeout(resolve, 5000));

  // Check status
  console.log('Checking status:');
  console.log(`  Scenario 1: ${await simulator.getStatus(scenario1)}`);
  console.log(`  Scenario 2: ${await simulator.getStatus(scenario2)}`);

  // Note: In real usage, you would wait for completion or poll periodically
  console.log('\nIn production, use waitForCompletion() or periodic polling');
}

/**
 * Example 6: Custom Configuration
 *
 * Execute fleet with custom configuration overrides.
 */
async function example6_CustomConfiguration() {
  console.log('\n=== Example 6: Custom Configuration ===\n');

  const fleet = new FleetBenchmark(
    {
      outputBaseDir: './custom-fleet-results',
      defaultConcurrency: 20, // High concurrency
      saveResponses: false, // Don't save individual responses
      requestDelayMs: 50, // Small delay between requests
      configPath: './custom-config.toml', // Custom provider config
    },
    {
      cliPath: '/usr/local/bin/llm-test-bench', // Custom CLI path
      timeout: 900000, // 15 minute timeout
    }
  );

  const handle = await fleet.executeFleet('./examples/fleet-manifest-example.json');

  console.log('Fleet execution started with custom configuration:');
  console.log('  Run ID:', handle.runId);
  console.log('  Output:', fleet['config'].outputBaseDir);
  console.log('  Concurrency:', fleet['config'].defaultConcurrency);
}

/**
 * Run all examples
 */
async function runAllExamples() {
  try {
    // Note: Uncomment examples as needed
    // Most examples require actual manifests and datasets to run

    await example1_BasicExecution();
    // await example2_ExecuteAndWait();
    // await example3_PollingForCompletion();
    // await example4_ListHistoricalRuns();
    // await example5_SimulatorIntegration();
    // await example6_CustomConfiguration();
  } catch (error) {
    console.error('Error running examples:', error);
  }
}

// Run examples if executed directly
if (import.meta.url === `file://${process.argv[1]}`) {
  runAllExamples().catch(console.error);
}

export {
  example1_BasicExecution,
  example2_ExecuteAndWait,
  example3_PollingForCompletion,
  example4_ListHistoricalRuns,
  example5_SimulatorIntegration,
  example6_CustomConfiguration,
  AgentSimulator,
};

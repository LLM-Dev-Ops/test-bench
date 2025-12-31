/**
 * Fleet Benchmark API for TypeScript/JavaScript
 *
 * Provides programmatic access to fleet benchmark execution for simulators
 * and automation tools.
 */

import { executeCLI, findCLIPath } from '../utils/cli-executor.js';
import type { SDKConfig } from '../types/index.js';

/**
 * Fleet configuration options
 */
export interface FleetConfig {
  /** Base directory for fleet results */
  outputBaseDir: string;
  /** Default concurrency for benchmark execution */
  defaultConcurrency?: number;
  /** Whether to save individual responses */
  saveResponses?: boolean;
  /** Whether to continue on individual test failures */
  continueOnFailure?: boolean;
  /** Optional request delay in milliseconds */
  requestDelayMs?: number;
  /** Path to configuration file */
  configPath?: string;
}

/**
 * Fleet manifest structure
 */
export interface FleetManifest {
  /** Fleet identifier */
  fleet_id: string;
  /** Fleet description */
  description?: string;
  /** Repositories to benchmark */
  repositories: RepositorySpec[];
  /** Providers to benchmark against */
  providers: string[];
  /** Fleet-level configuration overrides */
  config?: FleetManifestConfig;
}

/**
 * Repository specification in fleet manifest
 */
export interface RepositorySpec {
  /** Repository identifier */
  id: string;
  /** Repository name */
  name: string;
  /** Path to dataset file */
  dataset_path: string;
  /** Repository-specific metadata */
  metadata?: Record<string, string>;
}

/**
 * Fleet manifest configuration
 */
export interface FleetManifestConfig {
  /** Concurrency override */
  concurrency?: number;
  /** Request delay override */
  request_delay_ms?: number;
  /** Configuration file override */
  config_path?: string;
}

/**
 * Fleet execution handle returned immediately when executing a fleet benchmark
 */
export interface FleetExecutionHandle {
  /** Unique identifier for this fleet benchmark run */
  runId: string;
  /** Base directory where all artifacts will be stored */
  artifactBaseDir: string;
  /** Optional URL for monitoring execution status */
  statusUrl?: string;
  /** Metadata about the execution */
  metadata: FleetExecutionMetadata;
}

/**
 * Metadata about a fleet execution
 */
export interface FleetExecutionMetadata {
  /** Fleet identifier from manifest */
  fleetId: string;
  /** Number of repositories in the fleet */
  repositoryCount: number;
  /** Providers being benchmarked */
  providers: string[];
  /** When execution was initiated */
  startedAt: string;
  /** Expected artifact paths */
  expectedArtifacts: string[];
}

/**
 * Fleet benchmark results
 */
export interface FleetBenchmarkResults {
  /** Fleet identifier */
  fleet_id: string;
  /** Timestamp when fleet benchmark was executed */
  timestamp: string;
  /** Total number of repositories in the fleet */
  total_repositories: number;
  /** Results for each repository */
  repository_results: RepositoryResults[];
  /** Fleet-wide aggregated statistics */
  fleet_summary: FleetSummary;
  /** Per-provider statistics across the fleet */
  provider_breakdown: Record<string, ProviderFleetStats>;
  /** Per-category statistics across the fleet */
  category_breakdown: Record<string, CategoryFleetStats>;
  /** Fleet metadata */
  metadata: FleetMetadata;
}

/**
 * Results for a single repository within the fleet
 */
export interface RepositoryResults {
  /** Repository identifier */
  repository_id: string;
  /** Repository name */
  repository_name: string;
  /** Provider used for this repository */
  provider_name: string;
  /** Benchmark results for this repository */
  results: any; // BenchmarkResults
  /** Repository-specific metadata */
  repository_metadata: Record<string, string>;
}

/**
 * Fleet-wide summary statistics
 */
export interface FleetSummary {
  /** Total number of repositories */
  total_repositories: number;
  /** Total tests across all repositories */
  total_tests: number;
  /** Total successful tests */
  total_succeeded: number;
  /** Total failed tests */
  total_failed: number;
  /** Total timed out tests */
  total_timeout: number;
  /** Total skipped tests */
  total_skipped: number;
  /** Overall success rate (0.0 - 1.0) */
  success_rate: number;
  /** Average duration across all tests (milliseconds) */
  avg_duration_ms: number;
  /** Median latency across all tests (milliseconds) */
  p50_duration_ms: number;
  /** 95th percentile latency across all tests (milliseconds) */
  p95_duration_ms: number;
  /** 99th percentile latency across all tests (milliseconds) */
  p99_duration_ms: number;
  /** Minimum duration across all tests (milliseconds) */
  min_duration_ms: number;
  /** Maximum duration across all tests (milliseconds) */
  max_duration_ms: number;
  /** Total tokens used across all repositories */
  total_tokens: number;
  /** Average tokens per successful request */
  avg_tokens_per_request: number;
  /** Total cost across all repositories (USD) */
  total_cost: number;
  /** Average cost per repository (USD) */
  avg_cost_per_repository: number;
  /** Average number of tests per repository */
  avg_tests_per_repository: number;
  /** Total execution duration (milliseconds) */
  total_duration_ms: number;
}

/**
 * Per-provider statistics across the fleet
 */
export interface ProviderFleetStats {
  /** Provider name */
  provider_name: string;
  /** Number of repositories using this provider */
  repository_count: number;
  /** Total tests for this provider */
  total_tests: number;
  /** Total successful tests */
  total_succeeded: number;
  /** Total failed tests */
  total_failed: number;
  /** Success rate for this provider */
  success_rate: number;
  /** Total tokens used by this provider */
  total_tokens: number;
  /** Total cost for this provider (USD) */
  total_cost: number;
  /** Average duration for this provider (milliseconds) */
  avg_duration_ms: number;
}

/**
 * Per-category statistics across the fleet
 */
export interface CategoryFleetStats {
  /** Category name */
  category_name: string;
  /** Total tests in this category */
  total_tests: number;
  /** Total successful tests */
  total_succeeded: number;
  /** Total failed tests */
  total_failed: number;
  /** Success rate for this category */
  success_rate: number;
  /** Average duration for this category (milliseconds) */
  avg_duration_ms: number;
}

/**
 * Fleet metadata
 */
export interface FleetMetadata {
  /** Total tests across the fleet */
  total_tests: number;
  /** Total duration across the fleet */
  total_duration_ms: number;
  /** When the fleet benchmark was executed */
  execution_timestamp: string;
  /** Aggregation algorithm version (for reproducibility) */
  aggregation_version: string;
  /** Custom metadata fields */
  custom: Record<string, string>;
}

/**
 * Fleet Benchmark SDK
 *
 * Provides programmatic API for executing fleet benchmarks and retrieving results.
 * Designed to be consumed by simulators that need to trigger benchmarks programmatically.
 *
 * @example
 * ```typescript
 * import { FleetBenchmark } from 'llm-test-bench';
 *
 * const fleet = new FleetBenchmark({
 *   outputBaseDir: './fleet-results',
 *   defaultConcurrency: 10,
 * });
 *
 * // Execute fleet benchmark (returns immediately)
 * const handle = await fleet.executeFleet('./fleet-manifest.json');
 * console.log('Run ID:', handle.runId);
 * console.log('Artifacts:', handle.artifactBaseDir);
 *
 * // Wait for completion (optional)
 * const results = await fleet.waitForCompletion(handle.runId);
 * console.log('Fleet completed:', results.fleet_summary.success_rate);
 * ```
 */
export class FleetBenchmark {
  private cliPath: string;
  private config: Required<FleetConfig>;

  /**
   * Create a new Fleet Benchmark SDK instance
   *
   * @param config - Fleet configuration
   * @param sdkConfig - Optional SDK configuration (for CLI path, etc.)
   * @throws {Error} If CLI binary cannot be found
   */
  constructor(config: FleetConfig, sdkConfig?: SDKConfig) {
    const resolvedCliPath = sdkConfig?.cliPath ?? findCLIPath();
    if (!resolvedCliPath) {
      throw new Error(
        'LLM Test Bench CLI not found. Please install it via cargo or npm, or provide cliPath in config.'
      );
    }

    this.cliPath = resolvedCliPath;
    this.config = {
      outputBaseDir: config.outputBaseDir,
      defaultConcurrency: config.defaultConcurrency ?? 5,
      saveResponses: config.saveResponses ?? true,
      continueOnFailure: config.continueOnFailure ?? true,
      requestDelayMs: config.requestDelayMs,
      configPath: config.configPath,
    };
  }

  /**
   * Execute a fleet benchmark and return handle immediately
   *
   * This method:
   * 1. Validates the fleet manifest
   * 2. Generates a deterministic run_id
   * 3. Spawns async execution
   * 4. Returns handle immediately with run_id and artifact locations
   *
   * @param manifestPath - Path to fleet manifest JSON file
   * @param options - Optional execution options
   * @returns Promise resolving to FleetExecutionHandle
   * @throws {Error} If manifest is invalid or execution fails to start
   *
   * @example
   * ```typescript
   * const handle = await fleet.executeFleet('./fleet-manifest.json');
   * console.log('Run ID:', handle.runId);
   * console.log('Artifacts will be at:', handle.artifactBaseDir);
   * ```
   */
  async executeFleet(
    manifestPath: string,
    options?: {
      wait?: boolean;
      format?: 'json' | 'summary' | 'quiet';
    }
  ): Promise<FleetExecutionHandle> {
    const args = ['fleet', '--manifest', manifestPath, '--output', this.config.outputBaseDir];

    args.push('--concurrency', this.config.defaultConcurrency.toString());

    if (this.config.saveResponses) {
      args.push('--save-responses');
    }

    if (this.config.requestDelayMs !== undefined) {
      args.push('--delay', this.config.requestDelayMs.toString());
    }

    if (this.config.configPath) {
      args.push('--config', this.config.configPath);
    }

    const format = options?.format ?? 'quiet';
    args.push('--format', format);

    if (!options?.wait) {
      // Execute without waiting (async mode)
      const result = await executeCLI<string>(this.cliPath, {
        args,
        cwd: process.cwd(),
        timeout: 30000, // 30s timeout for startup
        parseJson: false,
      });

      if (!result.success) {
        throw new Error(`Failed to execute fleet benchmark: ${result.error}`);
      }

      // In quiet mode, stdout is just the run_id
      const runId = result.stdout.trim();

      // Parse metadata from stderr or construct expected paths
      const handle: FleetExecutionHandle = {
        runId,
        artifactBaseDir: `${this.config.outputBaseDir}/${runId}`,
        metadata: {
          fleetId: runId.split('-')[0] || 'unknown',
          repositoryCount: 0, // Unknown until execution completes
          providers: [],
          startedAt: new Date().toISOString(),
          expectedArtifacts: [
            `${this.config.outputBaseDir}/${runId}/fleet-results.json`,
            `${this.config.outputBaseDir}/${runId}/fleet-summary.csv`,
          ],
        },
      };

      return handle;
    } else {
      // Execute and wait for completion
      args.push('--wait');

      const result = await executeCLI<FleetBenchmarkResults>(this.cliPath, {
        args,
        cwd: process.cwd(),
        timeout: 600000, // 10 minute timeout for full execution
        parseJson: format === 'json',
      });

      if (!result.success) {
        throw new Error(`Fleet benchmark failed: ${result.error}`);
      }

      if (!result.data) {
        throw new Error('No results returned from fleet benchmark');
      }

      const runId = result.data.fleet_id;

      const handle: FleetExecutionHandle = {
        runId,
        artifactBaseDir: `${this.config.outputBaseDir}/${runId}`,
        metadata: {
          fleetId: result.data.fleet_id,
          repositoryCount: result.data.total_repositories,
          providers: Object.keys(result.data.provider_breakdown),
          startedAt: result.data.timestamp,
          expectedArtifacts: [
            `${this.config.outputBaseDir}/${runId}/fleet-results.json`,
            `${this.config.outputBaseDir}/${runId}/fleet-summary.csv`,
          ],
        },
      };

      return handle;
    }
  }

  /**
   * Get fleet results for a given run ID
   *
   * Loads the fleet results from disk for a previously executed fleet benchmark.
   *
   * @param runId - The run identifier to load results for
   * @returns Promise resolving to FleetBenchmarkResults
   * @throws {Error} If results cannot be found or loaded
   *
   * @example
   * ```typescript
   * const results = await fleet.getFleetResults('my-fleet-20231215-123456');
   * console.log('Success rate:', results.fleet_summary.success_rate);
   * ```
   */
  async getFleetResults(runId: string): Promise<FleetBenchmarkResults> {
    const resultsPath = `${this.config.outputBaseDir}/${runId}/fleet-results.json`;

    try {
      const fs = await import('fs/promises');
      const content = await fs.readFile(resultsPath, 'utf-8');
      return JSON.parse(content) as FleetBenchmarkResults;
    } catch (error) {
      throw new Error(`Failed to load fleet results for ${runId}: ${error}`);
    }
  }

  /**
   * Wait for a fleet benchmark to complete
   *
   * Polls the results directory until the fleet-results.json file is available.
   *
   * @param runId - The run identifier to wait for
   * @param options - Polling options
   * @returns Promise resolving to FleetBenchmarkResults when complete
   *
   * @example
   * ```typescript
   * const handle = await fleet.executeFleet('./manifest.json');
   * const results = await fleet.waitForCompletion(handle.runId);
   * ```
   */
  async waitForCompletion(
    runId: string,
    options?: {
      timeoutMs?: number;
      pollIntervalMs?: number;
    }
  ): Promise<FleetBenchmarkResults> {
    const timeoutMs = options?.timeoutMs ?? 600000; // 10 minutes default
    const pollIntervalMs = options?.pollIntervalMs ?? 2000; // 2 seconds default

    const startTime = Date.now();

    while (Date.now() - startTime < timeoutMs) {
      try {
        return await this.getFleetResults(runId);
      } catch (error) {
        // Results not yet available, wait and retry
        await new Promise((resolve) => setTimeout(resolve, pollIntervalMs));
      }
    }

    throw new Error(`Timeout waiting for fleet benchmark ${runId} to complete`);
  }

  /**
   * List all available fleet runs
   *
   * @returns Promise resolving to array of run IDs
   *
   * @example
   * ```typescript
   * const runs = await fleet.listRuns();
   * console.log('Available runs:', runs);
   * ```
   */
  async listRuns(): Promise<string[]> {
    try {
      const fs = await import('fs/promises');
      const entries = await fs.readdir(this.config.outputBaseDir, { withFileTypes: true });
      return entries.filter((entry) => entry.isDirectory()).map((entry) => entry.name);
    } catch (error) {
      // Directory doesn't exist yet
      return [];
    }
  }
}

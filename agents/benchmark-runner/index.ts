/**
 * Benchmark Runner Agent
 *
 * Execute deterministic benchmark suites against one or more LLMs,
 * producing reproducible performance, quality, latency, and cost metrics.
 */

export { handler, BENCHMARK_RUNNER_AGENT } from './handler';

// Re-export contracts for consumers
export {
  BenchmarkRunnerInputSchema,
  BenchmarkRunnerOutputSchema,
  BenchmarkRunnerDecisionEventSchema,
  VALID_CONSTRAINTS,
  NON_RESPONSIBILITIES,
  ALLOWED_CONSUMERS,
  type BenchmarkRunnerInput,
  type BenchmarkRunnerOutput,
  type BenchmarkProviderConfig,
  type BenchmarkTestCase,
  type BenchmarkSuite,
  type TestExecutionResult,
  type AggregatedStats,
} from '../contracts';

/**
 * LLM Test Bench - TypeScript SDK
 *
 * A comprehensive, production-ready framework for benchmarking, testing,
 * and evaluating Large Language Models.
 *
 * @packageDocumentation
 */

// Export main SDK class
export { LLMTestBench } from './core/llm-test-bench.js';

// Export Fleet Benchmark API
export { FleetBenchmark } from './core/fleet-benchmark.js';
export type {
  FleetConfig,
  FleetManifest,
  RepositorySpec,
  FleetManifestConfig,
  FleetExecutionHandle,
  FleetExecutionMetadata,
  FleetBenchmarkResults,
  FleetSummary,
  ProviderFleetStats,
  CategoryFleetStats,
} from './core/fleet-benchmark.js';

// Export provider clients
export {
  ProviderClient,
  OpenAIClient,
  AnthropicClient,
  GoogleClient,
  ProviderClientFactory,
} from './core/provider-client.js';

// Export evaluator utilities
export { Evaluator, createEvaluator } from './evaluators/index.js';

// Export all types
export * from './types/index.js';

// Export execution span module (Agentics Foundational Execution Unit)
export {
  InstrumentedTestBench,
  SpanManager,
  executeWithSpans,
  validateExecutionContext,
  ExecutionContextError,
  ExecutionInvariantError,
  generateSpanId,
  nowISO,
  REPO_NAME,
} from './execution/index.js';
export type {
  SpanType,
  SpanStatus,
  ExecutionContext,
  ExecutionSpan,
  ExecutionArtifact,
  ExecutionResult,
} from './execution/index.js';

// Export policy gate (execution dependency enforcement)
export {
  PolicyGate,
  PolicyGateError,
  DEFAULT_EXECUTION_POLICY,
} from './execution/index.js';
export type {
  SimulationIntent,
  GateDisposition,
  ExecutionPolicy,
  ExemptionRecord,
  ExemptionRequest,
  PolicyCheckResult,
} from './execution/index.js';

// Export utilities
export { executeCLI, findCLIPath } from './utils/cli-executor.js';
export {
  validateProviderConfig,
  validateCompletionRequest,
  validateBenchmarkConfig,
  validateEvaluationConfig,
  isValidModel,
  isValidProvider,
} from './utils/validators.js';

/**
 * SDK version
 */
export const VERSION = '0.1.2';

/**
 * Default export for convenience
 */
import { LLMTestBench } from './core/llm-test-bench.js';

export default LLMTestBench;

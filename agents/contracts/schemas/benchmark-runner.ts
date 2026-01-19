/**
 * Benchmark Runner Agent Contract Schemas
 *
 * AGENT PURPOSE:
 * Execute deterministic benchmark suites against one or more LLMs, producing
 * reproducible performance, quality, latency, and cost metrics.
 *
 * This agent:
 * - Executes benchmarks
 * - Does NOT compare models
 * - Does NOT score regressions
 * - Does NOT rank outputs
 *
 * decision_type: "benchmark_execution"
 */

import { z } from 'zod';
import { AgentIdentifierSchema, ExecutionRefSchema, DecisionEventSchema } from './base';

// =============================================================================
// AGENT METADATA
// =============================================================================

export const BENCHMARK_RUNNER_AGENT = {
  agent_id: 'benchmark-runner',
  agent_version: '1.0.0',
  decision_type: 'benchmark_execution',
} as const;

// =============================================================================
// INPUT SCHEMAS
// =============================================================================

/**
 * Provider configuration for benchmark execution
 */
export const BenchmarkProviderConfigSchema = z.object({
  provider_name: z.enum([
    'openai',
    'anthropic',
    'google',
    'mistral',
    'azure-openai',
    'bedrock',
    'cohere',
    'groq',
    'huggingface',
    'ollama',
    'perplexity',
    'replicate',
    'together',
  ]),
  model_id: z.string().min(1),
  api_key_ref: z.string().optional(), // Reference to secret, NOT the actual key
  base_url: z.string().url().optional(),
  timeout_ms: z.number().positive().default(30000),
  max_retries: z.number().nonnegative().default(3),
});

export type BenchmarkProviderConfig = z.infer<typeof BenchmarkProviderConfigSchema>;

/**
 * Single benchmark prompt/test case
 */
export const BenchmarkTestCaseSchema = z.object({
  test_id: z.string().min(1),
  prompt: z.string().min(1),
  expected_output: z.string().optional(),
  max_tokens: z.number().positive().default(1024),
  temperature: z.number().min(0).max(2).default(0.7),
  top_p: z.number().min(0).max(1).optional(),
  stop_sequences: z.array(z.string()).optional(),
  metadata: z.record(z.unknown()).optional(),
});

export type BenchmarkTestCase = z.infer<typeof BenchmarkTestCaseSchema>;

/**
 * Benchmark suite definition
 */
export const BenchmarkSuiteSchema = z.object({
  suite_id: z.string().min(1),
  suite_name: z.string().min(1),
  description: z.string().optional(),
  test_cases: z.array(BenchmarkTestCaseSchema).min(1),
  tags: z.array(z.string()).optional(),
});

export type BenchmarkSuite = z.infer<typeof BenchmarkSuiteSchema>;

/**
 * Execution configuration
 */
export const BenchmarkExecutionConfigSchema = z.object({
  concurrency: z.number().positive().max(100).default(1),
  warm_up_runs: z.number().nonnegative().default(0),
  iterations_per_test: z.number().positive().default(1),
  max_duration_ms: z.number().positive().optional(),
  save_responses: z.boolean().default(true),
  fail_fast: z.boolean().default(false),
});

export type BenchmarkExecutionConfig = z.infer<typeof BenchmarkExecutionConfigSchema>;

/**
 * Main input schema for Benchmark Runner Agent
 */
export const BenchmarkRunnerInputSchema = z.object({
  // Required: what to benchmark
  providers: z.array(BenchmarkProviderConfigSchema).min(1),
  suite: BenchmarkSuiteSchema,

  // Optional: how to run
  execution_config: BenchmarkExecutionConfigSchema.optional(),

  // Optional: caller context
  caller_id: z.string().optional(),
  correlation_id: z.string().uuid().optional(),
});

export type BenchmarkRunnerInput = z.infer<typeof BenchmarkRunnerInputSchema>;

// =============================================================================
// OUTPUT SCHEMAS
// =============================================================================

/**
 * Latency metrics for a single execution
 */
export const LatencyMetricsSchema = z.object({
  total_ms: z.number().nonnegative(),
  time_to_first_token_ms: z.number().nonnegative().optional(),
  tokens_per_second: z.number().nonnegative().optional(),
});

export type LatencyMetrics = z.infer<typeof LatencyMetricsSchema>;

/**
 * Token usage metrics
 */
export const TokenUsageSchema = z.object({
  prompt_tokens: z.number().nonnegative(),
  completion_tokens: z.number().nonnegative(),
  total_tokens: z.number().nonnegative(),
});

export type TokenUsage = z.infer<typeof TokenUsageSchema>;

/**
 * Cost metrics (in USD)
 */
export const CostMetricsSchema = z.object({
  input_cost_usd: z.number().nonnegative(),
  output_cost_usd: z.number().nonnegative(),
  total_cost_usd: z.number().nonnegative(),
});

export type CostMetrics = z.infer<typeof CostMetricsSchema>;

/**
 * Single test execution result
 */
export const TestExecutionResultSchema = z.object({
  test_id: z.string(),
  iteration: z.number().nonnegative(),
  provider_name: z.string(),
  model_id: z.string(),

  // Success/failure
  success: z.boolean(),
  error_message: z.string().optional(),

  // Response (if saved)
  response_content: z.string().optional(),
  finish_reason: z.string().optional(),

  // Metrics
  latency: LatencyMetricsSchema,
  token_usage: TokenUsageSchema.optional(),
  cost: CostMetricsSchema.optional(),

  // Timing
  started_at: z.string().datetime(),
  completed_at: z.string().datetime(),
});

export type TestExecutionResult = z.infer<typeof TestExecutionResultSchema>;

/**
 * Aggregated statistics for a provider/model
 */
export const AggregatedStatsSchema = z.object({
  provider_name: z.string(),
  model_id: z.string(),

  // Counts
  total_executions: z.number().nonnegative(),
  successful_executions: z.number().nonnegative(),
  failed_executions: z.number().nonnegative(),
  success_rate: z.number().min(0).max(1),

  // Latency percentiles
  latency_p50_ms: z.number().nonnegative(),
  latency_p95_ms: z.number().nonnegative(),
  latency_p99_ms: z.number().nonnegative(),
  latency_mean_ms: z.number().nonnegative(),
  latency_min_ms: z.number().nonnegative(),
  latency_max_ms: z.number().nonnegative(),
  latency_stddev_ms: z.number().nonnegative(),

  // Token stats
  total_tokens: z.number().nonnegative(),
  avg_tokens_per_request: z.number().nonnegative(),

  // Cost stats
  total_cost_usd: z.number().nonnegative(),
  avg_cost_per_request_usd: z.number().nonnegative(),

  // Throughput
  avg_tokens_per_second: z.number().nonnegative().optional(),
});

export type AggregatedStats = z.infer<typeof AggregatedStatsSchema>;

/**
 * Main output schema for Benchmark Runner Agent
 */
export const BenchmarkRunnerOutputSchema = z.object({
  // Execution identity
  execution_id: z.string().uuid(),
  suite_id: z.string(),
  suite_name: z.string(),

  // Timing
  started_at: z.string().datetime(),
  completed_at: z.string().datetime(),
  total_duration_ms: z.number().nonnegative(),

  // Summary
  total_tests: z.number().nonnegative(),
  total_executions: z.number().nonnegative(),
  successful_executions: z.number().nonnegative(),
  failed_executions: z.number().nonnegative(),

  // Detailed results (per test execution)
  results: z.array(TestExecutionResultSchema),

  // Aggregated stats (per provider/model)
  aggregated_stats: z.array(AggregatedStatsSchema),

  // Metadata
  execution_config: BenchmarkExecutionConfigSchema,
  providers_tested: z.array(z.object({
    provider_name: z.string(),
    model_id: z.string(),
  })),
});

export type BenchmarkRunnerOutput = z.infer<typeof BenchmarkRunnerOutputSchema>;

// =============================================================================
// DECISION EVENT SCHEMA (SPECIALIZED)
// =============================================================================

/**
 * Benchmark Runner Decision Event
 * Extends base DecisionEvent with benchmark-specific outputs
 */
export const BenchmarkRunnerDecisionEventSchema = DecisionEventSchema.extend({
  decision_type: z.literal('benchmark_execution'),
  outputs: BenchmarkRunnerOutputSchema,
});

export type BenchmarkRunnerDecisionEvent = z.infer<typeof BenchmarkRunnerDecisionEventSchema>;

// =============================================================================
// CLI CONTRACT
// =============================================================================

/**
 * CLI invocation shape for Benchmark Runner Agent
 */
export const BenchmarkRunnerCLIArgsSchema = z.object({
  // Input source (one required)
  input_file: z.string().optional(),
  input_json: z.string().optional(),
  input_stdin: z.boolean().optional(),

  // Output format
  output_format: z.enum(['json', 'csv', 'table']).default('json'),
  output_file: z.string().optional(),

  // Verbosity
  verbose: z.boolean().default(false),
  quiet: z.boolean().default(false),

  // Execution modifiers
  dry_run: z.boolean().default(false),
});

export type BenchmarkRunnerCLIArgs = z.infer<typeof BenchmarkRunnerCLIArgsSchema>;

// =============================================================================
// CONSTRAINTS & NON-RESPONSIBILITIES
// =============================================================================

/**
 * Constraints that MAY be applied during execution
 */
export const VALID_CONSTRAINTS = [
  'max_duration_exceeded',
  'max_cost_exceeded',
  'rate_limit_applied',
  'fail_fast_triggered',
  'warm_up_skipped',
  'concurrency_limited',
  'provider_unavailable',
] as const;

/**
 * Explicit non-responsibilities - this agent MUST NOT:
 */
export const NON_RESPONSIBILITIES = [
  'compare_models',           // No model comparison logic
  'score_regressions',        // No regression scoring
  'rank_outputs',             // No ranking/ordering
  'enforce_policy',           // No policy enforcement
  'orchestrate_workflows',    // No workflow orchestration
  'call_other_agents',        // No direct agent-to-agent calls
  'store_api_keys',           // Never persist API keys
  'execute_arbitrary_code',   // No code execution beyond LLM calls
  'bypass_schemas',           // Must validate all I/O
] as const;

// =============================================================================
// CONFIDENCE SCORING
// =============================================================================

/**
 * Factors that contribute to confidence scoring
 */
export const CONFIDENCE_FACTORS = {
  execution_success_rate: {
    description: 'Percentage of successful executions',
    weight: 0.4,
  },
  latency_consistency: {
    description: 'Standard deviation of latency (lower = more consistent)',
    weight: 0.2,
  },
  provider_reliability: {
    description: 'Historical provider reliability score',
    weight: 0.2,
  },
  sample_size: {
    description: 'Number of iterations (more = higher confidence)',
    weight: 0.2,
  },
} as const;

/**
 * Calculate confidence score based on execution results
 */
export function calculateConfidence(stats: AggregatedStats): number {
  const factors = [
    // Success rate (0-1)
    stats.success_rate * CONFIDENCE_FACTORS.execution_success_rate.weight,

    // Latency consistency (inverse of normalized stddev, capped)
    Math.max(0, 1 - (stats.latency_stddev_ms / stats.latency_mean_ms)) *
      CONFIDENCE_FACTORS.latency_consistency.weight,

    // Provider reliability (placeholder - would come from historical data)
    0.8 * CONFIDENCE_FACTORS.provider_reliability.weight,

    // Sample size (logarithmic scale, capped at 100 executions)
    Math.min(1, Math.log10(stats.total_executions + 1) / 2) *
      CONFIDENCE_FACTORS.sample_size.weight,
  ];

  return Math.min(1, Math.max(0, factors.reduce((a, b) => a + b, 0)));
}

// =============================================================================
// CORE BUNDLE CONSUMERS
// =============================================================================

/**
 * Core bundles that may consume this agent's output
 */
export const ALLOWED_CONSUMERS = [
  'llm-orchestrator',         // For workflow coordination
  'llm-observatory',          // For telemetry/monitoring
  'llm-analytics',            // For aggregation/analysis
  'llm-test-bench-ui',        // For dashboard display
] as const;

// =============================================================================
// VERSIONING RULES
// =============================================================================

export const VERSIONING_RULES = {
  major: 'Breaking changes to input/output schemas',
  minor: 'New optional fields, new providers, new metrics',
  patch: 'Bug fixes, performance improvements, documentation',
} as const;

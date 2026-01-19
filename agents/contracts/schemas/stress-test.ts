/**
 * Stress Test Agent Contract Schemas
 *
 * AGENT PURPOSE:
 * Evaluate model robustness under extreme input, load, or adversarial conditions.
 * Produces metrics on failure modes, degradation patterns, and recovery behavior
 * when models are pushed beyond normal operating parameters.
 *
 * This agent:
 * - Executes stress tests (extreme inputs, high concurrency, adversarial prompts)
 * - Measures degradation under load
 * - Detects failure thresholds and breaking points
 * - Does NOT benchmark normal performance (use benchmark-runner)
 * - Does NOT compare models (no ranking/comparison logic)
 * - Does NOT enforce policies or orchestrate workflows
 *
 * decision_type: "stress_test_execution"
 */

import { z } from 'zod';
import { DecisionEventSchema, ExecutionRefSchema } from './base';

// =============================================================================
// AGENT METADATA
// =============================================================================

export const STRESS_TEST_AGENT = {
  agent_id: 'stress-test',
  agent_version: '1.0.0',
  decision_type: 'stress_test_execution',
} as const;

// =============================================================================
// INPUT SCHEMAS
// =============================================================================

/**
 * Provider configuration for stress test execution
 */
export const StressTestProviderConfigSchema = z.object({
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
  timeout_ms: z.number().positive().default(60000), // Higher default for stress tests
  max_retries: z.number().nonnegative().default(1), // Lower retries to detect failures
});

export type StressTestProviderConfig = z.infer<typeof StressTestProviderConfigSchema>;

/**
 * Types of stress tests supported
 */
export const StressTestTypeSchema = z.enum([
  'load_ramp',           // Gradually increase concurrency until failure
  'spike',               // Sudden burst of concurrent requests
  'soak',                // Sustained load over time
  'extreme_input',       // Extremely long inputs, edge cases
  'adversarial',         // Malformed, edge-case, or potentially harmful inputs
  'rate_limit_probe',    // Probe rate limits and throttling behavior
  'timeout_boundary',    // Test timeout thresholds
  'token_limit',         // Push token limits
  'context_overflow',    // Test context window boundaries
  'malformed_request',   // Test error handling with invalid requests
]);

export type StressTestType = z.infer<typeof StressTestTypeSchema>;

/**
 * Load ramp configuration
 */
export const LoadRampConfigSchema = z.object({
  initial_concurrency: z.number().positive().default(1),
  max_concurrency: z.number().positive().default(100),
  step_size: z.number().positive().default(5),
  step_duration_ms: z.number().positive().default(10000),
  requests_per_step: z.number().positive().default(10),
});

export type LoadRampConfig = z.infer<typeof LoadRampConfigSchema>;

/**
 * Spike test configuration
 */
export const SpikeConfigSchema = z.object({
  baseline_concurrency: z.number().nonnegative().default(0),
  spike_concurrency: z.number().positive().default(50),
  spike_duration_ms: z.number().positive().default(5000),
  recovery_observation_ms: z.number().positive().default(10000),
});

export type SpikeConfig = z.infer<typeof SpikeConfigSchema>;

/**
 * Soak test configuration
 */
export const SoakConfigSchema = z.object({
  concurrency: z.number().positive().default(10),
  duration_ms: z.number().positive().default(300000), // 5 minutes default
  request_interval_ms: z.number().positive().default(1000),
  metrics_sample_interval_ms: z.number().positive().default(5000),
});

export type SoakConfig = z.infer<typeof SoakConfigSchema>;

/**
 * Extreme input configuration
 */
export const ExtremeInputConfigSchema = z.object({
  input_sizes: z.array(z.number().positive()).default([1000, 5000, 10000, 50000, 100000]),
  character_types: z.array(z.enum(['ascii', 'unicode', 'emoji', 'mixed', 'special'])).default(['ascii']),
  include_edge_cases: z.boolean().default(true),
  max_token_request: z.number().positive().optional(),
});

export type ExtremeInputConfig = z.infer<typeof ExtremeInputConfigSchema>;

/**
 * Adversarial test configuration
 */
export const AdversarialConfigSchema = z.object({
  test_categories: z.array(z.enum([
    'prompt_injection',      // Attempt to override system prompts
    'encoding_tricks',       // Unicode normalization, RTL, etc.
    'repetition',            // Repeated patterns
    'nested_structures',     // Deeply nested JSON, markdown, etc.
    'boundary_chars',        // NULL bytes, control chars
    'format_confusion',      // Mixing formats (JSON in markdown, etc.)
  ])).default(['encoding_tricks', 'repetition']),
  severity_level: z.enum(['low', 'medium', 'high']).default('medium'),
  samples_per_category: z.number().positive().default(5),
});

export type AdversarialConfig = z.infer<typeof AdversarialConfigSchema>;

/**
 * Rate limit probe configuration
 */
export const RateLimitProbeConfigSchema = z.object({
  initial_rps: z.number().positive().default(1),
  max_rps: z.number().positive().default(100),
  increment: z.number().positive().default(5),
  duration_per_level_ms: z.number().positive().default(5000),
  detect_throttling: z.boolean().default(true),
});

export type RateLimitProbeConfig = z.infer<typeof RateLimitProbeConfigSchema>;

/**
 * Stress test scenario definition
 */
export const StressTestScenarioSchema = z.object({
  scenario_id: z.string().min(1),
  scenario_name: z.string().min(1),
  description: z.string().optional(),
  test_type: StressTestTypeSchema,

  // Type-specific configurations (one required based on test_type)
  load_ramp_config: LoadRampConfigSchema.optional(),
  spike_config: SpikeConfigSchema.optional(),
  soak_config: SoakConfigSchema.optional(),
  extreme_input_config: ExtremeInputConfigSchema.optional(),
  adversarial_config: AdversarialConfigSchema.optional(),
  rate_limit_probe_config: RateLimitProbeConfigSchema.optional(),

  // Base prompt template for tests
  base_prompt: z.string().min(1).optional(),
  system_prompt: z.string().optional(),

  // Expected behavior
  expected_max_latency_ms: z.number().positive().optional(),
  expected_min_success_rate: z.number().min(0).max(1).optional(),

  // Tags for categorization
  tags: z.array(z.string()).optional(),
});

export type StressTestScenario = z.infer<typeof StressTestScenarioSchema>;

/**
 * Global execution configuration
 */
export const StressTestExecutionConfigSchema = z.object({
  max_total_duration_ms: z.number().positive().default(600000), // 10 min max
  max_total_requests: z.number().positive().default(10000),
  max_total_cost_usd: z.number().positive().optional(),
  stop_on_critical_failure: z.boolean().default(true),
  collect_response_samples: z.boolean().default(false), // For privacy
  sample_rate: z.number().min(0).max(1).default(0.1),
});

export type StressTestExecutionConfig = z.infer<typeof StressTestExecutionConfigSchema>;

/**
 * Main input schema for Stress Test Agent
 */
export const StressTestInputSchema = z.object({
  // Required: what to test
  providers: z.array(StressTestProviderConfigSchema).min(1),
  scenarios: z.array(StressTestScenarioSchema).min(1),

  // Optional: how to run
  execution_config: StressTestExecutionConfigSchema.optional(),

  // Optional: caller context
  caller_id: z.string().optional(),
  correlation_id: z.string().uuid().optional(),
});

export type StressTestInput = z.infer<typeof StressTestInputSchema>;

// =============================================================================
// OUTPUT SCHEMAS
// =============================================================================

/**
 * Failure mode classification
 */
export const FailureModeSchema = z.enum([
  'timeout',              // Request timed out
  'rate_limited',         // 429 or similar
  'context_exceeded',     // Token limit exceeded
  'invalid_response',     // Malformed or empty response
  'server_error',         // 5xx errors
  'connection_error',     // Network issues
  'authentication_error', // Auth failures
  'content_filtered',     // Safety filters triggered
  'unknown',              // Unclassified error
]);

export type FailureMode = z.infer<typeof FailureModeSchema>;

/**
 * Single stress test request result
 */
export const StressRequestResultSchema = z.object({
  request_id: z.string().uuid(),
  scenario_id: z.string(),
  provider_name: z.string(),
  model_id: z.string(),

  // Request details
  concurrency_level: z.number().nonnegative(),
  input_size_chars: z.number().nonnegative(),
  input_tokens_approx: z.number().nonnegative().optional(),

  // Result
  success: z.boolean(),
  failure_mode: FailureModeSchema.optional(),
  error_message: z.string().optional(),
  http_status: z.number().optional(),

  // Latency
  latency_ms: z.number().nonnegative(),
  time_to_first_token_ms: z.number().nonnegative().optional(),

  // Tokens (if available)
  prompt_tokens: z.number().nonnegative().optional(),
  completion_tokens: z.number().nonnegative().optional(),

  // Timing
  timestamp: z.string().datetime(),
});

export type StressRequestResult = z.infer<typeof StressRequestResultSchema>;

/**
 * Breaking point detection result
 */
export const BreakingPointSchema = z.object({
  detected: z.boolean(),
  metric: z.enum(['concurrency', 'rps', 'input_size', 'token_count']),
  threshold_value: z.number(),
  failure_rate_at_threshold: z.number().min(0).max(1),
  first_failure_at: z.number(),
  degradation_pattern: z.enum(['gradual', 'cliff', 'oscillating', 'immediate']),
});

export type BreakingPoint = z.infer<typeof BreakingPointSchema>;

/**
 * Scenario execution summary
 */
export const ScenarioResultSchema = z.object({
  scenario_id: z.string(),
  scenario_name: z.string(),
  test_type: StressTestTypeSchema,
  provider_name: z.string(),
  model_id: z.string(),

  // Execution stats
  total_requests: z.number().nonnegative(),
  successful_requests: z.number().nonnegative(),
  failed_requests: z.number().nonnegative(),
  success_rate: z.number().min(0).max(1),

  // Failure analysis
  failure_modes: z.array(z.object({
    mode: FailureModeSchema,
    count: z.number().nonnegative(),
    percentage: z.number().min(0).max(1),
    first_occurrence_ms: z.number().nonnegative(),
  })),

  // Latency under stress
  latency_mean_ms: z.number().nonnegative(),
  latency_p50_ms: z.number().nonnegative(),
  latency_p95_ms: z.number().nonnegative(),
  latency_p99_ms: z.number().nonnegative(),
  latency_max_ms: z.number().nonnegative(),
  latency_degradation_percent: z.number(), // Compared to baseline

  // Breaking points (if detected)
  breaking_points: z.array(BreakingPointSchema),

  // Recovery metrics (for spike/soak tests)
  recovery_time_ms: z.number().nonnegative().optional(),
  stability_after_recovery: z.number().min(0).max(1).optional(),

  // Timing
  started_at: z.string().datetime(),
  completed_at: z.string().datetime(),
  duration_ms: z.number().nonnegative(),

  // Thresholds
  passed_latency_threshold: z.boolean().optional(),
  passed_success_rate_threshold: z.boolean().optional(),
});

export type ScenarioResult = z.infer<typeof ScenarioResultSchema>;

/**
 * Provider robustness summary
 */
export const ProviderRobustnessSummarySchema = z.object({
  provider_name: z.string(),
  model_id: z.string(),

  // Overall robustness score (0-1)
  robustness_score: z.number().min(0).max(1),

  // Key findings
  max_sustainable_concurrency: z.number().nonnegative().optional(),
  max_sustainable_rps: z.number().nonnegative().optional(),
  max_input_size_handled: z.number().nonnegative().optional(),

  // Degradation characteristics
  degradation_onset_concurrency: z.number().nonnegative().optional(),
  degradation_severity: z.enum(['none', 'mild', 'moderate', 'severe']),

  // Recovery characteristics
  avg_recovery_time_ms: z.number().nonnegative().optional(),
  recovery_reliability: z.number().min(0).max(1).optional(),

  // Failure resistance
  failure_resistance_score: z.number().min(0).max(1),
  most_common_failure_mode: FailureModeSchema.optional(),

  // Recommendations
  recommended_max_concurrency: z.number().nonnegative().optional(),
  recommended_max_rps: z.number().nonnegative().optional(),
  warnings: z.array(z.string()),
});

export type ProviderRobustnessSummary = z.infer<typeof ProviderRobustnessSummarySchema>;

/**
 * Main output schema for Stress Test Agent
 */
export const StressTestOutputSchema = z.object({
  // Execution identity
  execution_id: z.string().uuid(),

  // Timing
  started_at: z.string().datetime(),
  completed_at: z.string().datetime(),
  total_duration_ms: z.number().nonnegative(),

  // Summary counts
  total_scenarios: z.number().nonnegative(),
  total_requests: z.number().nonnegative(),
  total_successful: z.number().nonnegative(),
  total_failed: z.number().nonnegative(),
  overall_success_rate: z.number().min(0).max(1),

  // Scenario results
  scenario_results: z.array(ScenarioResultSchema),

  // Provider summaries
  provider_summaries: z.array(ProviderRobustnessSummarySchema),

  // Sampled raw results (if enabled)
  sampled_results: z.array(StressRequestResultSchema).optional(),

  // Execution config used
  execution_config: StressTestExecutionConfigSchema,

  // Constraints that were applied
  constraints_applied: z.array(z.string()),

  // Cost tracking
  estimated_total_cost_usd: z.number().nonnegative().optional(),
});

export type StressTestOutput = z.infer<typeof StressTestOutputSchema>;

// =============================================================================
// DECISION EVENT SCHEMA (SPECIALIZED)
// =============================================================================

/**
 * Stress Test Decision Event
 * Extends base DecisionEvent with stress-test-specific outputs
 */
export const StressTestDecisionEventSchema = DecisionEventSchema.extend({
  decision_type: z.literal('stress_test_execution'),
  outputs: StressTestOutputSchema,
});

export type StressTestDecisionEvent = z.infer<typeof StressTestDecisionEventSchema>;

// =============================================================================
// CLI CONTRACT
// =============================================================================

/**
 * CLI invocation shape for Stress Test Agent
 */
export const StressTestCLIArgsSchema = z.object({
  // Input source (one required)
  input_file: z.string().optional(),
  input_json: z.string().optional(),
  input_stdin: z.boolean().optional(),

  // Quick test presets
  preset: z.enum([
    'quick-load',       // Quick load ramp test
    'spike',            // Spike test
    'soak-5min',        // 5-minute soak test
    'adversarial',      // Adversarial input tests
    'full-suite',       // All test types
  ]).optional(),

  // Output format
  output_format: z.enum(['json', 'csv', 'table', 'summary']).default('json'),
  output_file: z.string().optional(),

  // Verbosity
  verbose: z.boolean().default(false),
  quiet: z.boolean().default(false),

  // Execution modifiers
  dry_run: z.boolean().default(false),

  // Safety limits
  max_requests: z.number().positive().optional(),
  max_cost_usd: z.number().positive().optional(),
});

export type StressTestCLIArgs = z.infer<typeof StressTestCLIArgsSchema>;

// =============================================================================
// CONSTRAINTS & NON-RESPONSIBILITIES
// =============================================================================

/**
 * Constraints that MAY be applied during execution
 */
export const STRESS_TEST_VALID_CONSTRAINTS = [
  'max_duration_exceeded',
  'max_requests_exceeded',
  'max_cost_exceeded',
  'critical_failure_stop',
  'rate_limit_backoff',
  'provider_unavailable',
  'safety_limit_triggered',
  'memory_limit_approached',
] as const;

/**
 * Explicit non-responsibilities - this agent MUST NOT:
 */
export const STRESS_TEST_NON_RESPONSIBILITIES = [
  'compare_models',           // No model comparison logic
  'rank_outputs',             // No ranking/ordering
  'score_quality',            // No quality scoring (use quality-scoring agent)
  'benchmark_normal_ops',     // Use benchmark-runner for normal benchmarks
  'enforce_policy',           // No policy enforcement
  'orchestrate_workflows',    // No workflow orchestration
  'call_other_agents',        // No direct agent-to-agent calls
  'store_api_keys',           // Never persist API keys
  'execute_arbitrary_code',   // No code execution beyond LLM calls
  'bypass_schemas',           // Must validate all I/O
  'persist_pii',              // Never store PII from responses
  'attempt_jailbreaks',       // Security testing != attempting actual attacks
] as const;

// =============================================================================
// CONFIDENCE SCORING
// =============================================================================

/**
 * Factors that contribute to confidence scoring for stress tests
 */
export const STRESS_TEST_CONFIDENCE_FACTORS = {
  sample_size: {
    description: 'Number of requests executed (more = higher confidence)',
    weight: 0.25,
  },
  scenario_coverage: {
    description: 'Variety of stress conditions tested',
    weight: 0.20,
  },
  result_consistency: {
    description: 'Consistency of results across repeated tests',
    weight: 0.25,
  },
  breaking_point_clarity: {
    description: 'Clear identification of failure thresholds',
    weight: 0.15,
  },
  recovery_observation: {
    description: 'Whether recovery behavior was observed',
    weight: 0.15,
  },
} as const;

/**
 * Calculate confidence score based on stress test results
 */
export function calculateStressTestConfidence(output: StressTestOutput): number {
  // Return 0 confidence for empty results
  if (output.total_requests === 0 || output.scenario_results.length === 0) {
    return 0;
  }

  const factors: number[] = [];

  // Sample size factor (log scale, cap at 1000 requests)
  const sampleFactor = Math.min(1, Math.log10(output.total_requests + 1) / 3);
  factors.push(sampleFactor * STRESS_TEST_CONFIDENCE_FACTORS.sample_size.weight);

  // Scenario coverage factor
  const uniqueTypes = new Set(output.scenario_results.map(s => s.test_type)).size;
  const coverageFactor = Math.min(1, uniqueTypes / 5); // 5 types = full coverage
  factors.push(coverageFactor * STRESS_TEST_CONFIDENCE_FACTORS.scenario_coverage.weight);

  // Result consistency (inverse of failure rate variance across scenarios)
  const successRates = output.scenario_results.map(s => s.success_rate);
  const avgRate = successRates.reduce((a, b) => a + b, 0) / successRates.length;
  const variance = successRates.reduce((sum, r) => sum + Math.pow(r - avgRate, 2), 0) / successRates.length;
  const consistencyFactor = Math.max(0, 1 - Math.sqrt(variance));
  factors.push(consistencyFactor * STRESS_TEST_CONFIDENCE_FACTORS.result_consistency.weight);

  // Breaking point clarity
  const hasBreakingPoints = output.scenario_results.some(s => s.breaking_points.length > 0);
  factors.push((hasBreakingPoints ? 1 : 0.5) * STRESS_TEST_CONFIDENCE_FACTORS.breaking_point_clarity.weight);

  // Recovery observation
  const hasRecoveryData = output.scenario_results.some(s => s.recovery_time_ms !== undefined);
  factors.push((hasRecoveryData ? 1 : 0.5) * STRESS_TEST_CONFIDENCE_FACTORS.recovery_observation.weight);

  return Math.min(1, Math.max(0, factors.reduce((a, b) => a + b, 0)));
}

// =============================================================================
// CORE BUNDLE CONSUMERS
// =============================================================================

/**
 * Core bundles that may consume this agent's output
 */
export const STRESS_TEST_ALLOWED_CONSUMERS = [
  'llm-orchestrator',         // For workflow coordination
  'llm-observatory',          // For telemetry/monitoring
  'llm-analytics',            // For aggregation/analysis
  'llm-test-bench-ui',        // For dashboard display
  'llm-capacity-planner',     // For capacity planning decisions
] as const;

// =============================================================================
// VERSIONING RULES
// =============================================================================

export const STRESS_TEST_VERSIONING_RULES = {
  major: 'Breaking changes to input/output schemas, new test types',
  minor: 'New optional fields, new failure modes, new metrics',
  patch: 'Bug fixes, accuracy improvements, documentation',
} as const;

// =============================================================================
// FAILURE MODE METADATA
// =============================================================================

export const FAILURE_MODE_METADATA = {
  timeout: {
    description: 'Request exceeded timeout threshold',
    recoverable: true,
    typical_cause: 'Server overload or network issues',
  },
  rate_limited: {
    description: 'Provider returned rate limit error (429)',
    recoverable: true,
    typical_cause: 'Exceeded provider rate limits',
  },
  context_exceeded: {
    description: 'Input or context exceeded token limits',
    recoverable: false,
    typical_cause: 'Input too large for model context window',
  },
  invalid_response: {
    description: 'Response was malformed or empty',
    recoverable: true,
    typical_cause: 'Model instability under load',
  },
  server_error: {
    description: 'Provider returned 5xx error',
    recoverable: true,
    typical_cause: 'Provider infrastructure issues',
  },
  connection_error: {
    description: 'Network connection failed',
    recoverable: true,
    typical_cause: 'Network issues or DNS failures',
  },
  authentication_error: {
    description: 'Authentication failed',
    recoverable: false,
    typical_cause: 'Invalid or expired credentials',
  },
  content_filtered: {
    description: 'Response blocked by content filters',
    recoverable: false,
    typical_cause: 'Input triggered safety filters',
  },
  unknown: {
    description: 'Unclassified error',
    recoverable: true,
    typical_cause: 'Unexpected error condition',
  },
} as const;

// Note: Use STRESS_TEST_* prefixed exports to avoid conflicts with other agents

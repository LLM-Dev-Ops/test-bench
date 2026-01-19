/**
 * Model Comparator Agent Contract Schemas
 *
 * AGENT PURPOSE:
 * Compare benchmark outputs across models and rank them using configurable
 * scoring criteria. Produces deterministic rankings with confidence scores.
 *
 * This agent:
 * - Compares benchmark results (YES)
 * - Ranks models by composite scores (YES)
 * - Calculates normalized metrics (YES)
 * - Does NOT execute benchmarks (NO - that's benchmark-runner)
 * - Does NOT enforce policies (NO - that's policy agents)
 * - Does NOT orchestrate workflows (NO - that's orchestrator)
 *
 * decision_type: "model_comparison"
 */

import { z } from 'zod';
import { DecisionEventSchema } from './base';
import { BenchmarkRunnerOutputSchema, AggregatedStatsSchema } from './benchmark-runner';

// =============================================================================
// AGENT METADATA
// =============================================================================

export const MODEL_COMPARATOR_AGENT = {
  agent_id: 'model-comparator',
  agent_version: '1.0.0',
  decision_type: 'model_comparison',
} as const;

// =============================================================================
// INPUT SCHEMAS
// =============================================================================

/**
 * Scoring criteria weights for comparison
 * All weights must sum to 1.0
 */
export const ScoringCriteriaSchema = z.object({
  /** Weight for latency score (lower latency = better) */
  latency_weight: z.number().min(0).max(1).default(0.25),

  /** Weight for cost score (lower cost = better) */
  cost_weight: z.number().min(0).max(1).default(0.25),

  /** Weight for throughput score (higher throughput = better) */
  throughput_weight: z.number().min(0).max(1).default(0.25),

  /** Weight for success rate score (higher success = better) */
  success_rate_weight: z.number().min(0).max(1).default(0.25),
}).refine(
  (data) => {
    const sum = data.latency_weight + data.cost_weight +
                data.throughput_weight + data.success_rate_weight;
    return Math.abs(sum - 1.0) < 0.001; // Allow for floating point tolerance
  },
  { message: 'Scoring weights must sum to 1.0' }
);

export type ScoringCriteria = z.infer<typeof ScoringCriteriaSchema>;

/**
 * Comparison configuration options
 */
export const ComparisonConfigSchema = z.object({
  /** Minimum number of models required for comparison */
  min_models: z.number().int().positive().default(2),

  /** Minimum success rate required to include a model in ranking */
  min_success_rate: z.number().min(0).max(1).default(0),

  /** Whether to include models with zero successful executions */
  include_failed_models: z.boolean().default(true),

  /** Normalization method for scores */
  normalization_method: z.enum(['min-max', 'z-score']).default('min-max'),
});

export type ComparisonConfig = z.infer<typeof ComparisonConfigSchema>;

/**
 * Main input schema for Model Comparator Agent
 */
export const ModelComparatorInputSchema = z.object({
  /** Benchmark run results to compare */
  benchmark_runs: z.array(BenchmarkRunnerOutputSchema).min(1),

  /** Scoring criteria with weights */
  scoring_criteria: ScoringCriteriaSchema.optional(),

  /** Comparison configuration */
  comparison_config: ComparisonConfigSchema.optional(),

  /** Optional: caller context */
  caller_id: z.string().optional(),

  /** Optional: correlation ID for tracing */
  correlation_id: z.string().uuid().optional(),
});

export type ModelComparatorInput = z.infer<typeof ModelComparatorInputSchema>;

// =============================================================================
// OUTPUT SCHEMAS
// =============================================================================

/**
 * Individual metric scores (normalized 0-1)
 */
export const MetricScoresSchema = z.object({
  /** Normalized latency score (0-1, higher = better/faster) */
  latency_score: z.number().min(0).max(1),

  /** Normalized cost score (0-1, higher = better/cheaper) */
  cost_score: z.number().min(0).max(1),

  /** Normalized throughput score (0-1, higher = better) */
  throughput_score: z.number().min(0).max(1),

  /** Success rate (0-1, as-is) */
  success_rate_score: z.number().min(0).max(1),
});

export type MetricScores = z.infer<typeof MetricScoresSchema>;

/**
 * Raw metric values before normalization
 */
export const RawMetricsSchema = z.object({
  latency_p50_ms: z.number().nonnegative(),
  latency_p95_ms: z.number().nonnegative(),
  avg_cost_per_request_usd: z.number().nonnegative(),
  total_cost_usd: z.number().nonnegative(),
  avg_tokens_per_second: z.number().nonnegative(),
  success_rate: z.number().min(0).max(1),
  total_executions: z.number().nonnegative(),
});

export type RawMetrics = z.infer<typeof RawMetricsSchema>;

/**
 * Single model ranking entry
 */
export const ModelRankingSchema = z.object({
  /** Rank position (1 = best) */
  rank: z.number().int().positive(),

  /** Provider name */
  provider_name: z.string(),

  /** Model identifier */
  model_id: z.string(),

  /** Weighted composite score (0-1) */
  composite_score: z.number().min(0).max(1),

  /** Individual metric scores */
  metric_scores: MetricScoresSchema,

  /** Raw metric values */
  raw_metrics: RawMetricsSchema,

  /** Delta from previous rank (null for rank 1) */
  delta_from_previous: z.number().nullable(),

  /** Source execution ID from benchmark run */
  source_execution_id: z.string().uuid(),
});

export type ModelRanking = z.infer<typeof ModelRankingSchema>;

/**
 * Comparison summary statistics
 */
export const ComparisonSummarySchema = z.object({
  /** Total models compared */
  total_models_compared: z.number().int().nonnegative(),

  /** Models excluded due to filters */
  models_excluded: z.number().int().nonnegative(),

  /** Best performing model (rank 1) */
  best_model: z.object({
    provider_name: z.string(),
    model_id: z.string(),
    composite_score: z.number(),
  }),

  /** Score spread (max - min composite score) */
  score_spread: z.number().min(0).max(1),

  /** Whether there are clear winners (spread > 0.1) */
  has_clear_winner: z.boolean(),
});

export type ComparisonSummary = z.infer<typeof ComparisonSummarySchema>;

/**
 * Main output schema for Model Comparator Agent
 */
export const ModelComparatorOutputSchema = z.object({
  /** Unique comparison ID */
  comparison_id: z.string().uuid(),

  /** Rankings ordered by composite score (descending) */
  rankings: z.array(ModelRankingSchema),

  /** Summary of comparison results */
  summary: ComparisonSummarySchema,

  /** Scoring criteria used */
  scoring_criteria_used: ScoringCriteriaSchema,

  /** Comparison configuration used */
  comparison_config_used: ComparisonConfigSchema,

  /** Timestamp of comparison */
  compared_at: z.string().datetime(),

  /** Total benchmark runs analyzed */
  benchmark_runs_analyzed: z.number().int().positive(),
});

export type ModelComparatorOutput = z.infer<typeof ModelComparatorOutputSchema>;

// =============================================================================
// DECISION EVENT SCHEMA (SPECIALIZED)
// =============================================================================

/**
 * Model Comparator Decision Event
 * Extends base DecisionEvent with comparison-specific outputs
 */
export const ModelComparatorDecisionEventSchema = DecisionEventSchema.extend({
  decision_type: z.literal('model_comparison'),
  outputs: ModelComparatorOutputSchema,
});

export type ModelComparatorDecisionEvent = z.infer<typeof ModelComparatorDecisionEventSchema>;

// =============================================================================
// CONSTRAINTS & NON-RESPONSIBILITIES
// =============================================================================

/**
 * Constraints that MAY be applied during comparison
 */
export const VALID_COMPARISON_CONSTRAINTS = [
  'insufficient_models',       // Less than min_models after filtering
  'no_successful_executions', // All models failed
  'identical_scores',         // Unable to differentiate models
  'single_model_comparison',  // Only one model available
  'missing_metrics',          // Some metrics unavailable for scoring
] as const;

/**
 * Explicit non-responsibilities - this agent MUST NOT:
 */
export const MODEL_COMPARATOR_NON_RESPONSIBILITIES = [
  'execute_benchmarks',       // No benchmark execution
  'enforce_policy',           // No policy decisions
  'orchestrate_workflows',    // No workflow orchestration
  'call_other_agents',        // No direct agent-to-agent calls
  'store_api_keys',           // Never persist API keys
  'modify_benchmark_results', // No mutation of input data
  'make_recommendations',     // Only rank, don't recommend
] as const;

// =============================================================================
// CONFIDENCE SCORING
// =============================================================================

/**
 * Factors that contribute to comparison confidence scoring
 */
export const COMPARISON_CONFIDENCE_FACTORS = {
  sample_size: {
    description: 'Total number of executions across all models',
    weight: 0.25,
  },
  model_count: {
    description: 'Number of models being compared (more = more meaningful)',
    weight: 0.2,
  },
  score_spread: {
    description: 'Spread between rankings (higher = clearer differentiation)',
    weight: 0.25,
  },
  execution_success_rate: {
    description: 'Average success rate across all models',
    weight: 0.2,
  },
  data_completeness: {
    description: 'Percentage of metrics available for scoring',
    weight: 0.1,
  },
} as const;

/**
 * Calculate confidence score for a comparison result
 */
export function calculateComparisonConfidence(
  rankings: ModelRanking[],
  totalExecutions: number
): { confidence: number; factors: Array<{ factor: string; weight: number; value: number }> } {
  if (rankings.length === 0) {
    return { confidence: 0, factors: [] };
  }

  const factors: Array<{ factor: string; weight: number; value: number }> = [];

  // Sample size factor (logarithmic, capped at 1000 executions)
  const sampleSizeValue = Math.min(1, Math.log10(totalExecutions + 1) / 3);
  factors.push({
    factor: 'sample_size',
    weight: COMPARISON_CONFIDENCE_FACTORS.sample_size.weight,
    value: sampleSizeValue,
  });

  // Model count factor (more models = more meaningful comparison)
  const modelCountValue = Math.min(1, rankings.length / 10);
  factors.push({
    factor: 'model_count',
    weight: COMPARISON_CONFIDENCE_FACTORS.model_count.weight,
    value: modelCountValue,
  });

  // Score spread factor
  const scores = rankings.map(r => r.composite_score);
  const maxScore = Math.max(...scores);
  const minScore = Math.min(...scores);
  const spreadValue = maxScore - minScore;
  factors.push({
    factor: 'score_spread',
    weight: COMPARISON_CONFIDENCE_FACTORS.score_spread.weight,
    value: spreadValue,
  });

  // Average success rate
  const avgSuccessRate = rankings.reduce(
    (sum, r) => sum + r.metric_scores.success_rate_score,
    0
  ) / rankings.length;
  factors.push({
    factor: 'execution_success_rate',
    weight: COMPARISON_CONFIDENCE_FACTORS.execution_success_rate.weight,
    value: avgSuccessRate,
  });

  // Data completeness (assume complete for now)
  factors.push({
    factor: 'data_completeness',
    weight: COMPARISON_CONFIDENCE_FACTORS.data_completeness.weight,
    value: 1.0,
  });

  // Calculate weighted confidence
  const confidence = factors.reduce(
    (sum, f) => sum + f.weight * f.value,
    0
  );

  return {
    confidence: Math.min(1, Math.max(0, confidence)),
    factors,
  };
}

// =============================================================================
// CORE BUNDLE CONSUMERS
// =============================================================================

/**
 * Core bundles that may consume this agent's output
 */
export const COMPARISON_ALLOWED_CONSUMERS = [
  'llm-orchestrator',         // For workflow decisions
  'llm-observatory',          // For monitoring/dashboards
  'llm-analytics',            // For trend analysis
  'llm-test-bench-ui',        // For visualization
  'model-selector',           // For automated model selection
] as const;

// =============================================================================
// VERSIONING RULES
// =============================================================================

export const COMPARATOR_VERSIONING_RULES = {
  major: 'Breaking changes to input/output schemas or ranking algorithm',
  minor: 'New metrics, new normalization methods, new config options',
  patch: 'Bug fixes, performance improvements, documentation',
} as const;

// =============================================================================
// CLI CONTRACT
// =============================================================================

/**
 * CLI invocation shape for Model Comparator Agent
 */
export const ModelComparatorCLIArgsSchema = z.object({
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

export type ModelComparatorCLIArgs = z.infer<typeof ModelComparatorCLIArgsSchema>;

// =============================================================================
// VERIFICATION CHECKLIST
// =============================================================================

export const VERIFICATION_CHECKLIST = [
  'Agent registered in contracts index',
  'CLI command callable with --help',
  'Input validation rejects invalid data',
  'Output matches schema exactly',
  'DecisionEvent persisted to ruvector-service',
  'Telemetry emitted for LLM-Observatory',
  'Dry-run mode produces no side effects',
  'Error responses include proper error codes',
];

// =============================================================================
// SMOKE TEST COMMANDS
// =============================================================================

export const SMOKE_TESTS = [
  'npx ts-node agents/model-comparator/cli.ts --help',
  'npx ts-node agents/model-comparator/cli.ts --dry-run --input-json \'{"benchmark_runs":[],"criteria":{}}\'',
  'npx ts-node agents/model-comparator/cli.ts --input-file test-fixtures/benchmark-output.json',
];

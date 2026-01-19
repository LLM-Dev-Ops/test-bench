/**
 * Quality Scoring Agent Contract Schemas
 *
 * AGENT PURPOSE:
 * Compute normalized quality scores for model outputs using deterministic
 * scoring profiles. Produces consistent, reproducible quality assessments
 * based on configurable evaluation criteria.
 *
 * This agent:
 * - Evaluates output quality (YES)
 * - Applies deterministic scoring profiles (YES)
 * - Normalizes scores to 0-1 range (YES)
 * - Aggregates multiple quality dimensions (YES)
 * - Does NOT execute benchmarks (NO - that's benchmark-runner)
 * - Does NOT compare models (NO - that's model-comparator)
 * - Does NOT enforce policies (NO - that's policy agents)
 * - Does NOT orchestrate workflows (NO - that's orchestrator)
 *
 * decision_type: "quality_scoring"
 */

import { z } from 'zod';
import { DecisionEventSchema } from './base';

// =============================================================================
// AGENT METADATA
// =============================================================================

export const QUALITY_SCORING_AGENT = {
  agent_id: 'quality-scoring',
  agent_version: '1.0.0',
  decision_type: 'quality_scoring',
} as const;

// =============================================================================
// SCORING PROFILE SCHEMAS
// =============================================================================

/**
 * Individual quality dimension definition
 */
export const QualityDimensionSchema = z.object({
  /** Unique dimension identifier */
  dimension_id: z.string().min(1).regex(/^[a-z][a-z0-9_]*$/),

  /** Human-readable name */
  name: z.string().min(1),

  /** Description of what this dimension measures */
  description: z.string().optional(),

  /** Weight in composite score (0-1) */
  weight: z.number().min(0).max(1),

  /** Scoring method for this dimension */
  scoring_method: z.enum([
    'exact_match',           // 1.0 if matches expected, 0.0 otherwise
    'contains',              // 1.0 if contains expected substring
    'regex_match',           // 1.0 if matches regex pattern
    'semantic_similarity',   // Cosine similarity to expected (requires embeddings)
    'length_ratio',          // Ratio of actual/expected length (capped at 1.0)
    'keyword_presence',      // Proportion of expected keywords present
    'format_compliance',     // Adherence to expected format (JSON, XML, etc.)
    'custom_evaluator',      // External evaluator function reference
  ]),

  /** Expected value or pattern for comparison */
  expected: z.string().optional(),

  /** Keywords to check for (for keyword_presence method) */
  keywords: z.array(z.string()).optional(),

  /** Format type (for format_compliance method) */
  format_type: z.enum(['json', 'xml', 'yaml', 'markdown', 'code']).optional(),

  /** Custom evaluator reference (for custom_evaluator method) */
  evaluator_ref: z.string().optional(),

  /** Invert score (1 - score) for negative dimensions */
  invert: z.boolean().default(false),

  /** Minimum threshold to count as passing */
  pass_threshold: z.number().min(0).max(1).default(0.5),
});

export type QualityDimension = z.infer<typeof QualityDimensionSchema>;

/**
 * Scoring profile containing multiple dimensions
 */
export const ScoringProfileSchema = z.object({
  /** Unique profile identifier */
  profile_id: z.string().min(1).regex(/^[a-z][a-z0-9-]*$/),

  /** Human-readable profile name */
  name: z.string().min(1),

  /** Profile description */
  description: z.string().optional(),

  /** Quality dimensions to evaluate */
  dimensions: z.array(QualityDimensionSchema).min(1),

  /** Normalization method for composite score */
  normalization: z.enum(['weighted_sum', 'harmonic_mean', 'geometric_mean']).default('weighted_sum'),

  /** Profile version for tracking changes */
  version: z.string().regex(/^\d+\.\d+\.\d+$/).default('1.0.0'),
}).refine(
  (data) => {
    const sum = data.dimensions.reduce((acc, d) => acc + d.weight, 0);
    return Math.abs(sum - 1.0) < 0.001;
  },
  { message: 'Dimension weights must sum to 1.0' }
);

export type ScoringProfile = z.infer<typeof ScoringProfileSchema>;

// =============================================================================
// INPUT SCHEMAS
// =============================================================================

/**
 * Single model output to score
 */
export const ModelOutputSchema = z.object({
  /** Unique output identifier */
  output_id: z.string().uuid(),

  /** Provider that generated the output */
  provider_name: z.string().min(1),

  /** Model that generated the output */
  model_id: z.string().min(1),

  /** The actual output content to score */
  content: z.string(),

  /** Original prompt that generated this output */
  prompt: z.string().optional(),

  /** Expected/reference output for comparison */
  expected_output: z.string().optional(),

  /** Source benchmark execution ID (if from benchmark-runner) */
  source_execution_id: z.string().uuid().optional(),

  /** Source test case ID (if from benchmark suite) */
  test_id: z.string().optional(),

  /** Additional metadata */
  metadata: z.record(z.unknown()).optional(),
});

export type ModelOutput = z.infer<typeof ModelOutputSchema>;

/**
 * Evaluation configuration options
 */
export const EvaluationConfigSchema = z.object({
  /** Whether to compute dimension-level scores */
  include_dimension_scores: z.boolean().default(true),

  /** Whether to include detailed scoring breakdown */
  include_breakdown: z.boolean().default(true),

  /** Case sensitivity for text matching */
  case_sensitive: z.boolean().default(false),

  /** Strip whitespace before comparison */
  normalize_whitespace: z.boolean().default(true),

  /** Fail entire scoring if any dimension fails threshold */
  fail_fast_on_threshold: z.boolean().default(false),

  /** Parallel evaluation (for large batches) */
  parallel_evaluation: z.boolean().default(true),
});

export type EvaluationConfig = z.infer<typeof EvaluationConfigSchema>;

/**
 * Main input schema for Quality Scoring Agent
 */
export const QualityScoringInputSchema = z.object({
  /** Model outputs to score */
  outputs: z.array(ModelOutputSchema).min(1).max(1000),

  /** Scoring profile to use */
  scoring_profile: ScoringProfileSchema,

  /** Evaluation configuration */
  evaluation_config: EvaluationConfigSchema.optional(),

  /** Optional: caller context */
  caller_id: z.string().optional(),

  /** Optional: correlation ID for tracing */
  correlation_id: z.string().uuid().optional(),
});

export type QualityScoringInput = z.infer<typeof QualityScoringInputSchema>;

// =============================================================================
// OUTPUT SCHEMAS
// =============================================================================

/**
 * Score for a single dimension
 */
export const DimensionScoreSchema = z.object({
  /** Dimension identifier */
  dimension_id: z.string(),

  /** Dimension name */
  name: z.string(),

  /** Raw score before weighting (0-1) */
  raw_score: z.number().min(0).max(1),

  /** Weighted contribution to composite (raw_score * weight) */
  weighted_score: z.number().min(0).max(1),

  /** Weight used */
  weight: z.number().min(0).max(1),

  /** Whether score meets pass threshold */
  passed: z.boolean(),

  /** Scoring method used */
  scoring_method: z.string(),

  /** Additional scoring details */
  details: z.record(z.unknown()).optional(),
});

export type DimensionScore = z.infer<typeof DimensionScoreSchema>;

/**
 * Score breakdown for a single output
 */
export const OutputScoreSchema = z.object({
  /** Output identifier */
  output_id: z.string().uuid(),

  /** Provider name */
  provider_name: z.string(),

  /** Model identifier */
  model_id: z.string(),

  /** Composite quality score (0-1, higher = better) */
  composite_score: z.number().min(0).max(1),

  /** Individual dimension scores */
  dimension_scores: z.array(DimensionScoreSchema),

  /** Number of dimensions that passed threshold */
  dimensions_passed: z.number().int().nonnegative(),

  /** Total number of dimensions evaluated */
  dimensions_total: z.number().int().positive(),

  /** Pass rate (dimensions_passed / dimensions_total) */
  pass_rate: z.number().min(0).max(1),

  /** Overall pass/fail status */
  overall_passed: z.boolean(),

  /** Timestamp of scoring */
  scored_at: z.string().datetime(),
});

export type OutputScore = z.infer<typeof OutputScoreSchema>;

/**
 * Aggregated statistics for a model
 */
export const ModelQualityStatsSchema = z.object({
  /** Provider name */
  provider_name: z.string(),

  /** Model identifier */
  model_id: z.string(),

  /** Number of outputs scored */
  outputs_scored: z.number().int().positive(),

  /** Average composite score */
  avg_composite_score: z.number().min(0).max(1),

  /** Minimum composite score */
  min_composite_score: z.number().min(0).max(1),

  /** Maximum composite score */
  max_composite_score: z.number().min(0).max(1),

  /** Standard deviation of composite scores */
  stddev_composite_score: z.number().nonnegative(),

  /** Score percentiles */
  p50_composite_score: z.number().min(0).max(1),
  p95_composite_score: z.number().min(0).max(1),
  p99_composite_score: z.number().min(0).max(1),

  /** Number of outputs that passed overall */
  outputs_passed: z.number().int().nonnegative(),

  /** Pass rate */
  overall_pass_rate: z.number().min(0).max(1),

  /** Average per-dimension scores */
  avg_dimension_scores: z.record(z.number().min(0).max(1)),
});

export type ModelQualityStats = z.infer<typeof ModelQualityStatsSchema>;

/**
 * Scoring summary
 */
export const ScoringSummarySchema = z.object({
  /** Total outputs scored */
  total_outputs_scored: z.number().int().positive(),

  /** Total models evaluated */
  total_models_evaluated: z.number().int().positive(),

  /** Overall average composite score */
  overall_avg_score: z.number().min(0).max(1),

  /** Overall pass rate */
  overall_pass_rate: z.number().min(0).max(1),

  /** Best performing model */
  best_model: z.object({
    provider_name: z.string(),
    model_id: z.string(),
    avg_score: z.number().min(0).max(1),
  }).nullable(),

  /** Score distribution buckets */
  score_distribution: z.object({
    excellent: z.number().int().nonnegative(),  // 0.9-1.0
    good: z.number().int().nonnegative(),       // 0.7-0.9
    fair: z.number().int().nonnegative(),       // 0.5-0.7
    poor: z.number().int().nonnegative(),       // 0.3-0.5
    failed: z.number().int().nonnegative(),     // 0.0-0.3
  }),
});

export type ScoringSummary = z.infer<typeof ScoringSummarySchema>;

/**
 * Main output schema for Quality Scoring Agent
 */
export const QualityScoringOutputSchema = z.object({
  /** Unique scoring run identifier */
  scoring_id: z.string().uuid(),

  /** Profile used for scoring */
  profile_id: z.string(),

  /** Profile name */
  profile_name: z.string(),

  /** Individual output scores */
  scores: z.array(OutputScoreSchema),

  /** Per-model aggregated statistics */
  model_stats: z.array(ModelQualityStatsSchema),

  /** Overall summary */
  summary: ScoringSummarySchema,

  /** Evaluation configuration used */
  evaluation_config_used: EvaluationConfigSchema,

  /** Timing */
  started_at: z.string().datetime(),
  completed_at: z.string().datetime(),
  duration_ms: z.number().nonnegative(),
});

export type QualityScoringOutput = z.infer<typeof QualityScoringOutputSchema>;

// =============================================================================
// DECISION EVENT SCHEMA (SPECIALIZED)
// =============================================================================

/**
 * Quality Scoring Decision Event
 * Extends base DecisionEvent with quality scoring-specific outputs
 */
export const QualityScoringDecisionEventSchema = DecisionEventSchema.extend({
  decision_type: z.literal('quality_scoring'),
  outputs: QualityScoringOutputSchema,
});

export type QualityScoringDecisionEvent = z.infer<typeof QualityScoringDecisionEventSchema>;

// =============================================================================
// CONSTRAINTS & NON-RESPONSIBILITIES
// =============================================================================

/**
 * Constraints that MAY be applied during scoring
 */
export const VALID_SCORING_CONSTRAINTS = [
  'max_outputs_exceeded',          // Batch size limit reached
  'semantic_similarity_unavailable', // Embeddings not available
  'custom_evaluator_failed',       // External evaluator error
  'dimension_weight_adjusted',     // Weights renormalized
  'threshold_breach_detected',     // Output failed threshold check
  'parallel_evaluation_disabled',  // Fell back to sequential
  'normalization_edge_case',       // All scores identical or extreme
] as const;

/**
 * Explicit non-responsibilities - this agent MUST NOT:
 */
export const QUALITY_SCORING_NON_RESPONSIBILITIES = [
  'execute_benchmarks',          // No benchmark execution
  'compare_models',              // No model comparison/ranking
  'enforce_policy',              // No policy decisions
  'orchestrate_workflows',       // No workflow orchestration
  'call_other_agents',           // No direct agent-to-agent calls
  'store_api_keys',              // Never persist API keys
  'modify_outputs',              // No mutation of input data
  'generate_content',            // No content generation
  'make_recommendations',        // Only score, don't recommend
  'cache_scoring_results',       // Stateless, no caching
] as const;

// =============================================================================
// CONFIDENCE SCORING
// =============================================================================

/**
 * Factors that contribute to quality scoring confidence
 */
export const QUALITY_CONFIDENCE_FACTORS = {
  sample_size: {
    description: 'Number of outputs scored (more = higher confidence)',
    weight: 0.25,
  },
  dimension_coverage: {
    description: 'Proportion of dimensions that could be evaluated',
    weight: 0.2,
  },
  score_consistency: {
    description: 'Consistency of scores (lower variance = higher confidence)',
    weight: 0.25,
  },
  profile_maturity: {
    description: 'Profile version maturity (higher version = more refined)',
    weight: 0.15,
  },
  method_reliability: {
    description: 'Reliability of scoring methods used (exact > regex > semantic)',
    weight: 0.15,
  },
} as const;

/**
 * Calculate confidence score for quality scoring results
 */
export function calculateQualityConfidence(
  output: QualityScoringOutput,
  profile: ScoringProfile
): { confidence: number; factors: Array<{ factor: string; weight: number; value: number }> } {
  const factors: Array<{ factor: string; weight: number; value: number }> = [];

  // Sample size factor (logarithmic, capped at 100 outputs)
  const sampleSizeValue = Math.min(1, Math.log10(output.scores.length + 1) / 2);
  factors.push({
    factor: 'sample_size',
    weight: QUALITY_CONFIDENCE_FACTORS.sample_size.weight,
    value: sampleSizeValue,
  });

  // Dimension coverage (all dimensions evaluated)
  const avgDimensionsCovered = output.scores.reduce(
    (sum, s) => sum + (s.dimensions_total / profile.dimensions.length),
    0
  ) / output.scores.length;
  factors.push({
    factor: 'dimension_coverage',
    weight: QUALITY_CONFIDENCE_FACTORS.dimension_coverage.weight,
    value: avgDimensionsCovered,
  });

  // Score consistency (inverse of normalized variance)
  const avgStats = output.model_stats[0];
  const consistencyValue = avgStats
    ? Math.max(0, 1 - (avgStats.stddev_composite_score / (avgStats.avg_composite_score || 1)))
    : 0.5;
  factors.push({
    factor: 'score_consistency',
    weight: QUALITY_CONFIDENCE_FACTORS.score_consistency.weight,
    value: Math.min(1, Math.max(0, consistencyValue)),
  });

  // Profile maturity (based on version)
  const [major, minor] = profile.version.split('.').map(Number);
  const maturityValue = Math.min(1, (major * 0.3 + minor * 0.1));
  factors.push({
    factor: 'profile_maturity',
    weight: QUALITY_CONFIDENCE_FACTORS.profile_maturity.weight,
    value: maturityValue,
  });

  // Method reliability (weighted by usage)
  const methodReliability: Record<string, number> = {
    exact_match: 1.0,
    contains: 0.9,
    regex_match: 0.85,
    format_compliance: 0.8,
    keyword_presence: 0.75,
    length_ratio: 0.7,
    semantic_similarity: 0.65,
    custom_evaluator: 0.5,
  };
  const avgReliability = profile.dimensions.reduce(
    (sum, d) => sum + (methodReliability[d.scoring_method] || 0.5) * d.weight,
    0
  );
  factors.push({
    factor: 'method_reliability',
    weight: QUALITY_CONFIDENCE_FACTORS.method_reliability.weight,
    value: avgReliability,
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
export const QUALITY_SCORING_ALLOWED_CONSUMERS = [
  'llm-orchestrator',           // For workflow quality gates
  'llm-observatory',            // For quality monitoring
  'llm-analytics',              // For quality trend analysis
  'llm-test-bench-ui',          // For quality dashboards
  'model-comparator',           // For quality-weighted comparisons
  'regression-detector',        // For quality regression detection
] as const;

// =============================================================================
// VERSIONING RULES
// =============================================================================

export const QUALITY_SCORING_VERSIONING_RULES = {
  major: 'Breaking changes to input/output schemas or scoring algorithms',
  minor: 'New scoring methods, new dimensions, new config options',
  patch: 'Bug fixes, performance improvements, documentation',
} as const;

// =============================================================================
// CLI CONTRACT
// =============================================================================

/**
 * CLI invocation shape for Quality Scoring Agent
 */
export const QualityScoringCLIArgsSchema = z.object({
  // Input source (one required)
  input_file: z.string().optional(),
  input_json: z.string().optional(),
  input_stdin: z.boolean().optional(),

  // Profile source (alternative to embedded in input)
  profile_file: z.string().optional(),

  // Output format
  output_format: z.enum(['json', 'csv', 'table']).default('json'),
  output_file: z.string().optional(),

  // Verbosity
  verbose: z.boolean().default(false),
  quiet: z.boolean().default(false),

  // Execution modifiers
  dry_run: z.boolean().default(false),
});

export type QualityScoringCLIArgs = z.infer<typeof QualityScoringCLIArgsSchema>;

// =============================================================================
// PRESET SCORING PROFILES
// =============================================================================

/**
 * Preset scoring profiles for common use cases
 */
export const PRESET_PROFILES = {
  /** Basic accuracy profile - exact match focus */
  accuracy_basic: {
    profile_id: 'accuracy-basic',
    name: 'Basic Accuracy',
    description: 'Simple accuracy scoring based on exact match and keyword presence',
    dimensions: [
      {
        dimension_id: 'exact_match',
        name: 'Exact Match',
        weight: 0.6,
        scoring_method: 'exact_match' as const,
        pass_threshold: 1.0,
        invert: false,
      },
      {
        dimension_id: 'keyword_coverage',
        name: 'Keyword Coverage',
        weight: 0.4,
        scoring_method: 'keyword_presence' as const,
        pass_threshold: 0.5,
        invert: false,
      },
    ],
    normalization: 'weighted_sum' as const,
    version: '1.0.0',
  },

  /** Comprehensive quality profile */
  comprehensive: {
    profile_id: 'comprehensive',
    name: 'Comprehensive Quality',
    description: 'Multi-dimensional quality assessment',
    dimensions: [
      {
        dimension_id: 'accuracy',
        name: 'Accuracy',
        weight: 0.35,
        scoring_method: 'contains' as const,
        pass_threshold: 0.7,
        invert: false,
      },
      {
        dimension_id: 'completeness',
        name: 'Completeness',
        weight: 0.25,
        scoring_method: 'length_ratio' as const,
        pass_threshold: 0.5,
        invert: false,
      },
      {
        dimension_id: 'format',
        name: 'Format Compliance',
        weight: 0.2,
        scoring_method: 'format_compliance' as const,
        format_type: 'json' as const,
        pass_threshold: 0.8,
        invert: false,
      },
      {
        dimension_id: 'keywords',
        name: 'Key Terms',
        weight: 0.2,
        scoring_method: 'keyword_presence' as const,
        pass_threshold: 0.6,
        invert: false,
      },
    ],
    normalization: 'weighted_sum' as const,
    version: '1.0.0',
  },
} as const;

// =============================================================================
// VERIFICATION CHECKLIST
// =============================================================================

export const QUALITY_SCORING_VERIFICATION_CHECKLIST = [
  'Agent registered in contracts index',
  'CLI command callable with --help',
  'Input validation rejects invalid data',
  'Output matches schema exactly',
  'DecisionEvent persisted to ruvector-service',
  'Telemetry emitted for LLM-Observatory',
  'Dry-run mode produces no side effects',
  'Error responses include proper error codes',
  'Dimension weights sum to 1.0',
  'All scoring methods produce 0-1 range',
  'Preset profiles validate successfully',
];

// =============================================================================
// SMOKE TEST COMMANDS
// =============================================================================

export const QUALITY_SCORING_SMOKE_TESTS = [
  'npx ts-node agents/quality-scoring/cli.ts --help',
  'npx ts-node agents/quality-scoring/cli.ts --dry-run --input-file agents/quality-scoring/examples/sample-input.json',
  'echo \'{"outputs":[{"output_id":"...","provider_name":"test","model_id":"test","content":"test"}],"scoring_profile":{}}\' | npx ts-node agents/quality-scoring/cli.ts --input-stdin',
];

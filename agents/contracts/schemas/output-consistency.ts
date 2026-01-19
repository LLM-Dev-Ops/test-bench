/**
 * Output Consistency Agent Contract Schemas
 *
 * AGENT PURPOSE:
 * Measure consistency across repeated executions of identical prompts.
 * Produces deterministic consistency metrics by analyzing output variations
 * when the same prompt is executed multiple times against the same model.
 *
 * This agent:
 * - Measures output consistency (YES)
 * - Calculates similarity/variance metrics (YES)
 * - Detects semantic drift across executions (YES)
 * - Aggregates consistency scores by model/prompt (YES)
 * - Does NOT execute prompts (NO - that's benchmark-runner)
 * - Does NOT compare different models (NO - that's model-comparator)
 * - Does NOT enforce policies (NO - that's policy agents)
 * - Does NOT orchestrate workflows (NO - that's orchestrator)
 *
 * decision_type: "output_consistency_analysis"
 */

import { z } from 'zod';
import { DecisionEventSchema } from './base';

// =============================================================================
// AGENT METADATA
// =============================================================================

export const OUTPUT_CONSISTENCY_AGENT = {
  agent_id: 'output-consistency',
  agent_version: '1.0.0',
  decision_type: 'output_consistency_analysis',
} as const;

// =============================================================================
// CONSISTENCY ANALYSIS CONFIGURATION
// =============================================================================

/**
 * Similarity method for comparing outputs
 */
export const SimilarityMethodSchema = z.enum([
  'exact_match',           // Binary: 1.0 if identical, 0.0 otherwise
  'normalized_levenshtein', // Edit distance normalized to 0-1
  'jaccard_tokens',        // Jaccard similarity on tokenized content
  'cosine_tfidf',          // Cosine similarity on TF-IDF vectors
  'semantic_embedding',    // Cosine similarity on embeddings (requires service)
  'character_ngram',       // Character n-gram overlap
  'word_ngram',            // Word n-gram overlap
]);

export type SimilarityMethod = z.infer<typeof SimilarityMethodSchema>;

/**
 * Configuration for consistency analysis
 */
export const ConsistencyConfigSchema = z.object({
  /** Primary similarity method to use */
  similarity_method: SimilarityMethodSchema.default('jaccard_tokens'),

  /** Secondary methods to compute (for comparison) */
  additional_methods: z.array(SimilarityMethodSchema).optional(),

  /** N-gram size for n-gram based methods */
  ngram_size: z.number().int().min(1).max(10).default(3),

  /** Minimum consistency score to consider "consistent" */
  consistency_threshold: z.number().min(0).max(1).default(0.85),

  /** Whether to normalize whitespace before comparison */
  normalize_whitespace: z.boolean().default(true),

  /** Case sensitivity for comparison */
  case_sensitive: z.boolean().default(false),

  /** Strip leading/trailing whitespace */
  trim_content: z.boolean().default(true),

  /** Include token-level analysis in output */
  include_token_analysis: z.boolean().default(true),

  /** Include character-level variance metrics */
  include_char_variance: z.boolean().default(false),

  /** Compute pairwise similarity matrix (can be large for many outputs) */
  compute_pairwise_matrix: z.boolean().default(false),
});

export type ConsistencyConfig = z.infer<typeof ConsistencyConfigSchema>;

// =============================================================================
// INPUT SCHEMAS
// =============================================================================

/**
 * A single execution output from a repeated prompt
 */
export const ExecutionOutputSchema = z.object({
  /** Unique output identifier */
  output_id: z.string().uuid(),

  /** The actual output content */
  content: z.string(),

  /** Execution sequence number (1, 2, 3, ...) */
  execution_number: z.number().int().positive(),

  /** Timestamp of execution */
  executed_at: z.string().datetime(),

  /** Execution latency in milliseconds (optional) */
  latency_ms: z.number().nonnegative().optional(),

  /** Temperature used for this execution (optional) */
  temperature: z.number().min(0).max(2).optional(),

  /** Token count (optional) */
  token_count: z.number().int().nonnegative().optional(),

  /** Additional metadata */
  metadata: z.record(z.unknown()).optional(),
});

export type ExecutionOutput = z.infer<typeof ExecutionOutputSchema>;

/**
 * A group of outputs from repeated executions of the same prompt
 */
export const PromptExecutionGroupSchema = z.object({
  /** Unique group identifier */
  group_id: z.string().min(1),

  /** The prompt that was executed */
  prompt: z.string().min(1),

  /** Hash of the prompt (for deduplication) */
  prompt_hash: z.string().optional(),

  /** Provider name */
  provider_name: z.string().min(1),

  /** Model identifier */
  model_id: z.string().min(1),

  /** All execution outputs for this prompt */
  outputs: z.array(ExecutionOutputSchema).min(2, 'At least 2 outputs required for consistency analysis'),

  /** Expected/reference output (optional, for correctness comparison) */
  expected_output: z.string().optional(),

  /** Test case ID (if from benchmark suite) */
  test_id: z.string().optional(),

  /** Source benchmark execution ID */
  source_execution_id: z.string().uuid().optional(),
});

export type PromptExecutionGroup = z.infer<typeof PromptExecutionGroupSchema>;

/**
 * Main input schema for Output Consistency Agent
 */
export const OutputConsistencyInputSchema = z.object({
  /** Groups of repeated prompt executions to analyze */
  execution_groups: z.array(PromptExecutionGroupSchema).min(1).max(500),

  /** Analysis configuration */
  config: ConsistencyConfigSchema.optional(),

  /** Optional: caller context */
  caller_id: z.string().optional(),

  /** Optional: correlation ID for tracing */
  correlation_id: z.string().uuid().optional(),
});

export type OutputConsistencyInput = z.infer<typeof OutputConsistencyInputSchema>;

// =============================================================================
// OUTPUT SCHEMAS
// =============================================================================

/**
 * Token-level analysis for a group
 */
export const TokenAnalysisSchema = z.object({
  /** Average token count across outputs */
  avg_token_count: z.number().nonnegative(),

  /** Minimum token count */
  min_token_count: z.number().int().nonnegative(),

  /** Maximum token count */
  max_token_count: z.number().int().nonnegative(),

  /** Token count standard deviation */
  stddev_token_count: z.number().nonnegative(),

  /** Token count coefficient of variation */
  cv_token_count: z.number().nonnegative(),

  /** Common tokens across all outputs (intersection) */
  common_token_count: z.number().int().nonnegative(),

  /** Total unique tokens across all outputs (union) */
  total_unique_tokens: z.number().int().nonnegative(),
});

export type TokenAnalysis = z.infer<typeof TokenAnalysisSchema>;

/**
 * Character-level variance metrics
 */
export const CharVarianceSchema = z.object({
  /** Average character count */
  avg_char_count: z.number().nonnegative(),

  /** Character count standard deviation */
  stddev_char_count: z.number().nonnegative(),

  /** Coefficient of variation for character count */
  cv_char_count: z.number().nonnegative(),

  /** Average edit distance between outputs */
  avg_edit_distance: z.number().nonnegative(),

  /** Maximum edit distance between any two outputs */
  max_edit_distance: z.number().int().nonnegative(),
});

export type CharVariance = z.infer<typeof CharVarianceSchema>;

/**
 * Similarity scores computed using different methods
 */
export const SimilarityScoresSchema = z.object({
  /** Primary method score (0-1, higher = more consistent) */
  primary_score: z.number().min(0).max(1),

  /** Primary method name */
  primary_method: SimilarityMethodSchema,

  /** Scores from additional methods (if computed) */
  additional_scores: z.record(z.number().min(0).max(1)).optional(),
});

export type SimilarityScores = z.infer<typeof SimilarityScoresSchema>;

/**
 * Consistency analysis for a single prompt execution group
 */
export const GroupConsistencyResultSchema = z.object({
  /** Group identifier */
  group_id: z.string(),

  /** Provider name */
  provider_name: z.string(),

  /** Model identifier */
  model_id: z.string(),

  /** Number of outputs analyzed */
  output_count: z.number().int().positive(),

  /** Overall consistency score (0-1, higher = more consistent) */
  consistency_score: z.number().min(0).max(1),

  /** Whether outputs meet consistency threshold */
  is_consistent: z.boolean(),

  /** Similarity scores by method */
  similarity_scores: SimilarityScoresSchema,

  /** Token-level analysis (if enabled) */
  token_analysis: TokenAnalysisSchema.optional(),

  /** Character variance (if enabled) */
  char_variance: CharVarianceSchema.optional(),

  /** Pairwise similarity matrix (if enabled) - flattened upper triangle */
  pairwise_similarities: z.array(z.number().min(0).max(1)).optional(),

  /** Index of most representative output (closest to centroid) */
  representative_output_index: z.number().int().nonnegative(),

  /** Index of most divergent output (furthest from others) */
  most_divergent_output_index: z.number().int().nonnegative(),

  /** Divergence score of most divergent output */
  max_divergence_score: z.number().min(0).max(1),

  /** Prompt hash */
  prompt_hash: z.string().optional(),

  /** Analysis timestamp */
  analyzed_at: z.string().datetime(),
});

export type GroupConsistencyResult = z.infer<typeof GroupConsistencyResultSchema>;

/**
 * Aggregated statistics for a model
 */
export const ModelConsistencyStatsSchema = z.object({
  /** Provider name */
  provider_name: z.string(),

  /** Model identifier */
  model_id: z.string(),

  /** Number of prompt groups analyzed */
  groups_analyzed: z.number().int().positive(),

  /** Total outputs analyzed */
  outputs_analyzed: z.number().int().positive(),

  /** Average consistency score across groups */
  avg_consistency_score: z.number().min(0).max(1),

  /** Minimum consistency score */
  min_consistency_score: z.number().min(0).max(1),

  /** Maximum consistency score */
  max_consistency_score: z.number().min(0).max(1),

  /** Standard deviation of consistency scores */
  stddev_consistency_score: z.number().nonnegative(),

  /** Consistency score percentiles */
  p50_consistency_score: z.number().min(0).max(1),
  p95_consistency_score: z.number().min(0).max(1),
  p99_consistency_score: z.number().min(0).max(1),

  /** Number of groups meeting consistency threshold */
  consistent_groups: z.number().int().nonnegative(),

  /** Consistency rate (consistent_groups / groups_analyzed) */
  consistency_rate: z.number().min(0).max(1),

  /** Average token variance across groups */
  avg_token_variance: z.number().nonnegative().optional(),
});

export type ModelConsistencyStats = z.infer<typeof ModelConsistencyStatsSchema>;

/**
 * Overall analysis summary
 */
export const ConsistencySummarySchema = z.object({
  /** Total groups analyzed */
  total_groups_analyzed: z.number().int().positive(),

  /** Total outputs analyzed */
  total_outputs_analyzed: z.number().int().positive(),

  /** Total models evaluated */
  total_models_evaluated: z.number().int().positive(),

  /** Overall average consistency score */
  overall_avg_consistency: z.number().min(0).max(1),

  /** Overall consistency rate */
  overall_consistency_rate: z.number().min(0).max(1),

  /** Most consistent model */
  most_consistent_model: z.object({
    provider_name: z.string(),
    model_id: z.string(),
    avg_score: z.number().min(0).max(1),
  }).nullable(),

  /** Least consistent model */
  least_consistent_model: z.object({
    provider_name: z.string(),
    model_id: z.string(),
    avg_score: z.number().min(0).max(1),
  }).nullable(),

  /** Consistency distribution */
  consistency_distribution: z.object({
    highly_consistent: z.number().int().nonnegative(),   // 0.95-1.0
    consistent: z.number().int().nonnegative(),          // 0.85-0.95
    moderate: z.number().int().nonnegative(),            // 0.70-0.85
    inconsistent: z.number().int().nonnegative(),        // 0.50-0.70
    highly_inconsistent: z.number().int().nonnegative(), // 0.0-0.50
  }),
});

export type ConsistencySummary = z.infer<typeof ConsistencySummarySchema>;

/**
 * Main output schema for Output Consistency Agent
 */
export const OutputConsistencyOutputSchema = z.object({
  /** Unique analysis run identifier */
  analysis_id: z.string().uuid(),

  /** Consistency results for each group */
  results: z.array(GroupConsistencyResultSchema),

  /** Per-model aggregated statistics */
  model_stats: z.array(ModelConsistencyStatsSchema),

  /** Overall summary */
  summary: ConsistencySummarySchema,

  /** Configuration used */
  config_used: ConsistencyConfigSchema,

  /** Timing */
  started_at: z.string().datetime(),
  completed_at: z.string().datetime(),
  duration_ms: z.number().nonnegative(),
});

export type OutputConsistencyOutput = z.infer<typeof OutputConsistencyOutputSchema>;

// =============================================================================
// DECISION EVENT SCHEMA (SPECIALIZED)
// =============================================================================

/**
 * Output Consistency Decision Event
 * Extends base DecisionEvent with consistency analysis-specific outputs
 */
export const OutputConsistencyDecisionEventSchema = DecisionEventSchema.extend({
  decision_type: z.literal('output_consistency_analysis'),
  outputs: OutputConsistencyOutputSchema,
});

export type OutputConsistencyDecisionEvent = z.infer<typeof OutputConsistencyDecisionEventSchema>;

// =============================================================================
// CONSTRAINTS & NON-RESPONSIBILITIES
// =============================================================================

/**
 * Constraints that MAY be applied during analysis
 */
export const VALID_CONSISTENCY_CONSTRAINTS = [
  'max_groups_exceeded',              // Batch size limit reached
  'semantic_embedding_unavailable',   // Embedding service not available
  'pairwise_matrix_too_large',        // Too many outputs for pairwise computation
  'outputs_too_short',                // Outputs too short for meaningful analysis
  'identical_outputs_detected',       // All outputs are identical
  'encoding_normalization_applied',   // Unicode normalization applied
  'truncation_applied',               // Very long outputs were truncated
] as const;

/**
 * Explicit non-responsibilities - this agent MUST NOT:
 */
export const OUTPUT_CONSISTENCY_NON_RESPONSIBILITIES = [
  'execute_prompts',            // No prompt execution
  'compare_different_models',   // No cross-model comparison
  'enforce_policy',             // No policy decisions
  'orchestrate_workflows',      // No workflow orchestration
  'call_other_agents',          // No direct agent-to-agent calls
  'store_api_keys',             // Never persist API keys
  'modify_outputs',             // No mutation of input data
  'generate_content',           // No content generation
  'make_recommendations',       // Only analyze, don't recommend
  'cache_analysis_results',     // Stateless, no caching
] as const;

// =============================================================================
// CONFIDENCE SCORING
// =============================================================================

/**
 * Factors that contribute to consistency analysis confidence
 */
export const CONSISTENCY_CONFIDENCE_FACTORS = {
  sample_size: {
    description: 'Number of output groups analyzed (more = higher confidence)',
    weight: 0.2,
  },
  outputs_per_group: {
    description: 'Average number of outputs per group (more = higher confidence)',
    weight: 0.25,
  },
  method_reliability: {
    description: 'Reliability of similarity method used',
    weight: 0.2,
  },
  content_length: {
    description: 'Average content length (longer = more signal)',
    weight: 0.15,
  },
  score_variance: {
    description: 'Consistency of consistency scores (lower variance = higher confidence)',
    weight: 0.2,
  },
} as const;

/**
 * Calculate confidence score for consistency analysis results
 */
export function calculateConsistencyConfidence(
  output: OutputConsistencyOutput,
  _config: ConsistencyConfig
): { confidence: number; factors: Array<{ factor: string; weight: number; value: number }> } {
  const factors: Array<{ factor: string; weight: number; value: number }> = [];

  // Sample size factor (logarithmic, capped at 100 groups)
  const sampleSizeValue = Math.min(1, Math.log10(output.results.length + 1) / 2);
  factors.push({
    factor: 'sample_size',
    weight: CONSISTENCY_CONFIDENCE_FACTORS.sample_size.weight,
    value: sampleSizeValue,
  });

  // Outputs per group (average, capped at 10)
  const avgOutputs = output.summary.total_outputs_analyzed / output.summary.total_groups_analyzed;
  const outputsValue = Math.min(1, (avgOutputs - 2) / 8); // Scale: 2 outputs = 0, 10+ = 1
  factors.push({
    factor: 'outputs_per_group',
    weight: CONSISTENCY_CONFIDENCE_FACTORS.outputs_per_group.weight,
    value: Math.max(0, outputsValue),
  });

  // Method reliability
  const methodReliability: Record<string, number> = {
    exact_match: 1.0,
    normalized_levenshtein: 0.9,
    jaccard_tokens: 0.85,
    character_ngram: 0.8,
    word_ngram: 0.8,
    cosine_tfidf: 0.75,
    semantic_embedding: 0.7, // Depends on embedding quality
  };
  const methodValue = methodReliability[output.config_used.similarity_method] ?? 0.7;
  factors.push({
    factor: 'method_reliability',
    weight: CONSISTENCY_CONFIDENCE_FACTORS.method_reliability.weight,
    value: methodValue,
  });

  // Content length factor (use token analysis if available)
  let contentLengthValue = 0.5; // Default
  if (output.results.length > 0 && output.results[0].token_analysis) {
    const avgTokens = output.results.reduce(
      (sum, r) => sum + (r.token_analysis?.avg_token_count ?? 0),
      0
    ) / output.results.length;
    contentLengthValue = Math.min(1, avgTokens / 200); // Scale: 200+ tokens = 1
  }
  factors.push({
    factor: 'content_length',
    weight: CONSISTENCY_CONFIDENCE_FACTORS.content_length.weight,
    value: contentLengthValue,
  });

  // Score variance (inverse - lower variance = higher confidence)
  const scores = output.results.map(r => r.consistency_score);
  const avgScore = scores.reduce((a, b) => a + b, 0) / scores.length;
  const variance = scores.reduce((sum, s) => sum + Math.pow(s - avgScore, 2), 0) / scores.length;
  const varianceValue = Math.max(0, 1 - Math.sqrt(variance) * 2);
  factors.push({
    factor: 'score_variance',
    weight: CONSISTENCY_CONFIDENCE_FACTORS.score_variance.weight,
    value: varianceValue,
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
export const OUTPUT_CONSISTENCY_ALLOWED_CONSUMERS = [
  'llm-orchestrator',           // For consistency-aware routing
  'llm-observatory',            // For consistency monitoring
  'llm-analytics',              // For consistency trend analysis
  'llm-test-bench-ui',          // For consistency dashboards
  'regression-detector',        // For consistency regression detection
  'model-comparator',           // For consistency-weighted comparisons
  'reliability-scorer',         // For reliability metrics
] as const;

// =============================================================================
// VERSIONING RULES
// =============================================================================

export const OUTPUT_CONSISTENCY_VERSIONING_RULES = {
  major: 'Breaking changes to input/output schemas or similarity algorithms',
  minor: 'New similarity methods, new config options, new metrics',
  patch: 'Bug fixes, performance improvements, documentation',
} as const;

// =============================================================================
// CLI CONTRACT
// =============================================================================

/**
 * CLI invocation shape for Output Consistency Agent
 */
export const OutputConsistencyCLIArgsSchema = z.object({
  // Input source (one required)
  input_file: z.string().optional(),
  input_json: z.string().optional(),
  input_stdin: z.boolean().optional(),

  // Config overrides
  similarity_method: SimilarityMethodSchema.optional(),
  consistency_threshold: z.number().min(0).max(1).optional(),

  // Output format
  output_format: z.enum(['json', 'csv', 'table']).default('json'),
  output_file: z.string().optional(),

  // Verbosity
  verbose: z.boolean().default(false),
  quiet: z.boolean().default(false),

  // Execution modifiers
  dry_run: z.boolean().default(false),
});

export type OutputConsistencyCLIArgs = z.infer<typeof OutputConsistencyCLIArgsSchema>;

// =============================================================================
// VERIFICATION CHECKLIST
// =============================================================================

export const OUTPUT_CONSISTENCY_VERIFICATION_CHECKLIST = [
  'Agent registered in contracts index',
  'CLI command callable with --help',
  'Input validation rejects invalid data',
  'Output matches schema exactly',
  'DecisionEvent persisted to ruvector-service',
  'Telemetry emitted for LLM-Observatory',
  'Dry-run mode produces no side effects',
  'Error responses include proper error codes',
  'At least 2 outputs required per group',
  'All similarity methods produce 0-1 range',
  'Pairwise matrix correctly computes upper triangle',
];

// =============================================================================
// SMOKE TEST COMMANDS
// =============================================================================

export const OUTPUT_CONSISTENCY_SMOKE_TESTS = [
  'npx ts-node agents/output-consistency/cli.ts --help',
  'npx ts-node agents/output-consistency/cli.ts --dry-run --input-file agents/output-consistency/examples/sample-input.json',
  'echo \'{"execution_groups":[{"group_id":"g1","prompt":"test","provider_name":"test","model_id":"test","outputs":[{"output_id":"...","content":"a","execution_number":1,"executed_at":"..."},{"output_id":"...","content":"a","execution_number":2,"executed_at":"..."}]}]}\' | npx ts-node agents/output-consistency/cli.ts --input-stdin',
];

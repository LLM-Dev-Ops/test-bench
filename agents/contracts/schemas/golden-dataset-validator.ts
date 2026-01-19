/**
 * Golden Dataset Validator Agent Contract Schemas
 *
 * AGENT PURPOSE:
 * Validate model outputs against canonical, human-verified datasets.
 * Compares LLM responses to golden reference answers to measure accuracy,
 * semantic similarity, and output quality relative to known-good baselines.
 *
 * This agent:
 * - Validates outputs against golden datasets (YES)
 * - Calculates exact match and semantic similarity scores (YES)
 * - Produces per-sample and aggregate validation metrics (YES)
 * - Does NOT generate or modify content (NO)
 * - Does NOT train models (NO)
 * - Does NOT orchestrate workflows (NO)
 * - Does NOT call other agents (NO)
 *
 * decision_type: "golden_dataset_validation"
 */

import { z } from 'zod';
import { DecisionEventSchema, ExecutionRefSchema } from './base';

// =============================================================================
// AGENT METADATA
// =============================================================================

export const GOLDEN_DATASET_VALIDATOR_AGENT = {
  agent_id: 'golden-dataset-validator',
  agent_version: '1.0.0',
  decision_type: 'golden_dataset_validation',
} as const;

// =============================================================================
// INPUT SCHEMAS
// =============================================================================

/**
 * A single sample from the golden dataset
 */
export const GoldenSampleSchema = z.object({
  /** Unique identifier for the sample */
  sample_id: z.string().min(1),

  /** The input/prompt that was given to the model */
  input: z.string().min(1),

  /** The expected golden (canonical) output */
  golden_output: z.string().min(1),

  /** Optional: category or type of the sample */
  category: z.string().optional(),

  /** Optional: difficulty level */
  difficulty: z.enum(['easy', 'medium', 'hard', 'expert']).optional(),

  /** Optional: tags for filtering/grouping */
  tags: z.array(z.string()).optional(),

  /** Optional: additional metadata */
  metadata: z.record(z.unknown()).optional(),
});

export type GoldenSample = z.infer<typeof GoldenSampleSchema>;

/**
 * Model output to validate against golden sample
 */
export const GoldenValidatorModelOutputSchema = z.object({
  /** Sample ID this output corresponds to */
  sample_id: z.string().min(1),

  /** The actual model output to validate */
  model_output: z.string(),

  /** Optional: model identifier that produced this output */
  model_id: z.string().optional(),

  /** Optional: provider name */
  provider: z.string().optional(),

  /** Optional: generation timestamp */
  generated_at: z.string().datetime().optional(),

  /** Optional: additional context */
  metadata: z.record(z.unknown()).optional(),
});

export type GoldenValidatorModelOutput = z.infer<typeof GoldenValidatorModelOutputSchema>;

/**
 * Validation configuration options
 */
export const ValidationConfigSchema = z.object({
  /** Enable exact string matching */
  enable_exact_match: z.boolean().default(true),

  /** Enable case-insensitive matching */
  case_insensitive: z.boolean().default(false),

  /** Enable semantic similarity analysis */
  enable_semantic_similarity: z.boolean().default(true),

  /** Threshold for semantic similarity to be considered a match (0-1) */
  semantic_similarity_threshold: z.number().min(0).max(1).default(0.85),

  /** Enable keyword overlap analysis */
  enable_keyword_analysis: z.boolean().default(true),

  /** Minimum keyword overlap ratio for partial match (0-1) */
  keyword_overlap_threshold: z.number().min(0).max(1).default(0.7),

  /** Enable structural similarity (for structured outputs) */
  enable_structural_similarity: z.boolean().default(false),

  /** Enable numeric tolerance for number comparisons */
  numeric_tolerance: z.number().nonnegative().default(0.001),

  /** Trim whitespace before comparison */
  trim_whitespace: z.boolean().default(true),

  /** Normalize unicode before comparison */
  normalize_unicode: z.boolean().default(true),

  /** Maximum samples to process */
  max_samples: z.number().int().positive().default(1000),

  /** Timeout for validation in milliseconds */
  timeout_ms: z.number().int().positive().default(60000),

  /** Include detailed per-sample analysis */
  include_detailed_analysis: z.boolean().default(true),
});

export type ValidationConfig = z.infer<typeof ValidationConfigSchema>;

/**
 * Main input schema for Golden Dataset Validator Agent
 */
export const GoldenDatasetValidatorInputSchema = z.object({
  /** Golden samples to validate against */
  golden_samples: z.array(GoldenSampleSchema).min(1).max(10000),

  /** Model outputs to validate */
  model_outputs: z.array(GoldenValidatorModelOutputSchema).min(1).max(10000),

  /** Validation configuration */
  validation_config: ValidationConfigSchema.optional(),

  /** Dataset metadata */
  dataset: z.object({
    /** Dataset name/identifier */
    name: z.string().min(1),
    /** Dataset version */
    version: z.string().optional(),
    /** Dataset description */
    description: z.string().optional(),
    /** Dataset source */
    source: z.string().optional(),
  }).optional(),

  /** Optional: caller context */
  caller_id: z.string().optional(),

  /** Optional: correlation ID for tracing */
  correlation_id: z.string().uuid().optional(),
}).refine(
  (data) => {
    // Ensure all model outputs have corresponding golden samples
    const goldenIds = new Set(data.golden_samples.map(s => s.sample_id));
    return data.model_outputs.every(o => goldenIds.has(o.sample_id));
  },
  { message: 'All model_outputs must have corresponding sample_ids in golden_samples' }
);

export type GoldenDatasetValidatorInput = z.infer<typeof GoldenDatasetValidatorInputSchema>;

// =============================================================================
// OUTPUT SCHEMAS
// =============================================================================

/**
 * Match type classification
 */
export const MatchTypeSchema = z.enum([
  'exact_match',           // Exact string match
  'semantic_match',        // High semantic similarity
  'partial_match',         // Some overlap but not complete
  'structural_match',      // Structure matches but content differs
  'no_match',              // No significant match found
  'error',                 // Validation error occurred
]);

export type MatchType = z.infer<typeof MatchTypeSchema>;

/**
 * Validation severity for mismatches
 */
export const ValidationSeveritySchema = z.enum([
  'pass',      // Meets validation criteria
  'warning',   // Minor deviation
  'fail',      // Significant deviation
  'critical',  // Complete failure
]);

export type ValidationSeverity = z.infer<typeof ValidationSeveritySchema>;

/**
 * Per-sample validation result
 */
export const SampleValidationResultSchema = z.object({
  /** Sample identifier */
  sample_id: z.string(),

  /** Whether the validation passed */
  passed: z.boolean(),

  /** Match type classification */
  match_type: MatchTypeSchema,

  /** Validation severity */
  severity: ValidationSeveritySchema,

  /** Confidence in the validation result (0-1) */
  confidence: z.number().min(0).max(1),

  /** Exact match result */
  exact_match: z.boolean(),

  /** Semantic similarity score (0-1) */
  semantic_similarity: z.number().min(0).max(1).nullable(),

  /** Keyword overlap ratio (0-1) */
  keyword_overlap: z.number().min(0).max(1).nullable(),

  /** Structural similarity score (0-1, for structured outputs) */
  structural_similarity: z.number().min(0).max(1).nullable(),

  /** The golden output (for reference) */
  golden_output: z.string(),

  /** The model output that was validated */
  model_output: z.string(),

  /** Difference summary */
  diff_summary: z.object({
    /** Characters added */
    chars_added: z.number().nonnegative(),
    /** Characters removed */
    chars_removed: z.number().nonnegative(),
    /** Words added */
    words_added: z.number().nonnegative(),
    /** Words removed */
    words_removed: z.number().nonnegative(),
    /** Key differences identified */
    key_differences: z.array(z.string()).optional(),
  }).optional(),

  /** Explanation of the validation result */
  explanation: z.string(),

  /** Validation timestamp */
  validated_at: z.string().datetime(),

  /** Sample category (if provided) */
  category: z.string().optional(),

  /** Additional metadata */
  metadata: z.record(z.unknown()).optional(),
});

export type SampleValidationResult = z.infer<typeof SampleValidationResultSchema>;

/**
 * Aggregated validation statistics
 */
export const ValidationStatsSchema = z.object({
  /** Total samples validated */
  total_samples: z.number().int().nonnegative(),

  /** Samples that passed validation */
  passed: z.number().int().nonnegative(),

  /** Samples that failed validation */
  failed: z.number().int().nonnegative(),

  /** Pass rate (0-1) */
  pass_rate: z.number().min(0).max(1),

  /** Exact match count */
  exact_matches: z.number().int().nonnegative(),

  /** Exact match rate (0-1) */
  exact_match_rate: z.number().min(0).max(1),

  /** Semantic match count (excluding exact) */
  semantic_matches: z.number().int().nonnegative(),

  /** Semantic match rate (0-1) */
  semantic_match_rate: z.number().min(0).max(1),

  /** Partial match count */
  partial_matches: z.number().int().nonnegative(),

  /** No match count */
  no_matches: z.number().int().nonnegative(),

  /** Error count */
  errors: z.number().int().nonnegative(),

  /** Average semantic similarity across all samples */
  avg_semantic_similarity: z.number().min(0).max(1),

  /** Average keyword overlap across all samples */
  avg_keyword_overlap: z.number().min(0).max(1),

  /** Average confidence across all validations */
  avg_confidence: z.number().min(0).max(1),

  /** Breakdown by match type */
  by_match_type: z.object({
    exact_match: z.number().int().nonnegative(),
    semantic_match: z.number().int().nonnegative(),
    partial_match: z.number().int().nonnegative(),
    structural_match: z.number().int().nonnegative(),
    no_match: z.number().int().nonnegative(),
    error: z.number().int().nonnegative(),
  }),

  /** Breakdown by severity */
  by_severity: z.object({
    pass: z.number().int().nonnegative(),
    warning: z.number().int().nonnegative(),
    fail: z.number().int().nonnegative(),
    critical: z.number().int().nonnegative(),
  }),

  /** Breakdown by category (if categories provided) */
  by_category: z.record(z.object({
    total: z.number().int().nonnegative(),
    passed: z.number().int().nonnegative(),
    pass_rate: z.number().min(0).max(1),
  })).optional(),
});

export type ValidationStats = z.infer<typeof ValidationStatsSchema>;

/**
 * Main output schema for Golden Dataset Validator Agent
 */
export const GoldenDatasetValidatorOutputSchema = z.object({
  /** Unique validation run identifier */
  validation_id: z.string().uuid(),

  /** Individual sample results */
  results: z.array(SampleValidationResultSchema),

  /** Aggregated statistics */
  stats: ValidationStatsSchema,

  /** Dataset metadata used */
  dataset: z.object({
    name: z.string(),
    version: z.string().optional(),
    sample_count: z.number().int().nonnegative(),
  }),

  /** Validation configuration used */
  validation_config_used: ValidationConfigSchema,

  /** Model information (if provided) */
  model_info: z.object({
    model_id: z.string().optional(),
    provider: z.string().optional(),
    output_count: z.number().int().nonnegative(),
  }).optional(),

  /** Quality assessment */
  quality_assessment: z.object({
    /** Overall quality grade */
    grade: z.enum(['A', 'B', 'C', 'D', 'F']),
    /** Quality score (0-100) */
    score: z.number().min(0).max(100),
    /** Assessment summary */
    summary: z.string(),
    /** Recommendations for improvement */
    recommendations: z.array(z.string()).optional(),
  }),

  /** Timing */
  started_at: z.string().datetime(),
  completed_at: z.string().datetime(),
  duration_ms: z.number().nonnegative(),
});

export type GoldenDatasetValidatorOutput = z.infer<typeof GoldenDatasetValidatorOutputSchema>;

/**
 * Golden Dataset Validator Decision Event
 */
export const GoldenDatasetValidatorDecisionEventSchema = DecisionEventSchema.extend({
  decision_type: z.literal('golden_dataset_validation'),
  outputs: GoldenDatasetValidatorOutputSchema,
});

export type GoldenDatasetValidatorDecisionEvent = z.infer<typeof GoldenDatasetValidatorDecisionEventSchema>;

// =============================================================================
// CLI CONTRACT
// =============================================================================

/**
 * CLI invocation shape for Golden Dataset Validator Agent
 */
export const GoldenDatasetValidatorCLIArgsSchema = z.object({
  // Input sources
  input_file: z.string().optional(),
  input_json: z.string().optional(),
  input_stdin: z.boolean().optional(),

  // Golden dataset source
  golden_file: z.string().optional(),
  golden_url: z.string().url().optional(),

  // Model outputs source
  outputs_file: z.string().optional(),
  outputs_url: z.string().url().optional(),

  // Validation options
  exact_match_only: z.boolean().default(false),
  case_insensitive: z.boolean().default(false),
  similarity_threshold: z.number().min(0).max(1).optional(),

  // Output format
  output_format: z.enum(['json', 'csv', 'table', 'summary', 'report']).default('json'),
  output_file: z.string().optional(),

  // Verbosity
  verbose: z.boolean().default(false),
  quiet: z.boolean().default(false),

  // Execution modifiers
  dry_run: z.boolean().default(false),
  fail_fast: z.boolean().default(false),
});

export type GoldenDatasetValidatorCLIArgs = z.infer<typeof GoldenDatasetValidatorCLIArgsSchema>;

// =============================================================================
// CONSTRAINTS & NON-RESPONSIBILITIES
// =============================================================================

/**
 * Constraints that MAY be applied during execution
 */
export const GOLDEN_DATASET_VALID_CONSTRAINTS = [
  'max_samples_exceeded',        // Sample count exceeded limit
  'timeout_exceeded',            // Processing timeout reached
  'semantic_analysis_unavailable', // Embeddings not available
  'sample_mismatch',             // Golden sample not found for output
  'invalid_sample_format',       // Sample format not recognized
  'memory_limit_exceeded',       // Too much data to process
  'low_confidence_result',       // Result confidence below threshold
  'partial_validation_only',     // Some samples could not be validated
] as const;

export type GoldenDatasetConstraint = typeof GOLDEN_DATASET_VALID_CONSTRAINTS[number];

/**
 * Explicit non-responsibilities - this agent MUST NOT:
 */
export const GOLDEN_DATASET_NON_RESPONSIBILITIES = [
  'generate_content',            // No content generation
  'modify_outputs',              // No output modification
  'train_models',                // No model training
  'create_golden_datasets',      // No dataset creation
  'orchestrate_workflows',       // No workflow orchestration
  'call_other_agents',           // No direct agent-to-agent calls
  'rank_models',                 // No model ranking (just validation)
  'make_recommendations',        // No model recommendations
  'execute_arbitrary_code',      // No code execution
  'bypass_schemas',              // Must validate all I/O
  'persist_raw_outputs',         // No storing raw model outputs
  'modify_golden_samples',       // Golden samples are read-only
] as const;

// =============================================================================
// CONFIDENCE SCORING
// =============================================================================

/**
 * Factors that contribute to confidence scoring
 */
export const GOLDEN_DATASET_CONFIDENCE_FACTORS = {
  sample_size: {
    description: 'Number of samples validated',
    weight: 0.20,
  },
  match_consistency: {
    description: 'Consistency of match types across samples',
    weight: 0.25,
  },
  semantic_coverage: {
    description: 'Availability of semantic similarity analysis',
    weight: 0.20,
  },
  result_clarity: {
    description: 'Clarity of pass/fail determinations',
    weight: 0.20,
  },
  validation_coverage: {
    description: 'Proportion of samples successfully validated',
    weight: 0.15,
  },
} as const;

/**
 * Calculate confidence score for a sample validation
 */
export function calculateSampleConfidence(
  result: Partial<SampleValidationResult>,
  config: ValidationConfig
): number {
  let confidence = 0.5; // Base confidence

  // Exact match has highest confidence
  if (result.exact_match) {
    confidence = 0.98;
  } else if (result.semantic_similarity !== null && result.semantic_similarity !== undefined) {
    // Semantic similarity available - use it
    const semSim = result.semantic_similarity;
    if (semSim >= 0.95) {
      confidence = 0.92;
    } else if (semSim >= config.semantic_similarity_threshold) {
      confidence = 0.75 + (semSim - config.semantic_similarity_threshold) * 0.5;
    } else if (semSim >= 0.5) {
      confidence = 0.5 + semSim * 0.3;
    } else {
      confidence = 0.4 + semSim * 0.2;
    }
  } else if (result.keyword_overlap !== null && result.keyword_overlap !== undefined) {
    // Fall back to keyword overlap
    confidence = 0.4 + result.keyword_overlap * 0.4;
  }

  // Boost confidence for clear results
  if (result.match_type === 'no_match' || result.match_type === 'exact_match') {
    confidence = Math.min(1, confidence + 0.1);
  }

  return Math.min(1, Math.max(0, confidence));
}

// =============================================================================
// CORE BUNDLE CONSUMERS
// =============================================================================

/**
 * Core bundles that may consume this agent's output
 */
export const GOLDEN_DATASET_ALLOWED_CONSUMERS = [
  'llm-orchestrator',           // For workflow coordination
  'llm-observatory',            // For telemetry/monitoring
  'llm-analytics',              // For aggregation/analysis
  'llm-test-bench-ui',          // For dashboard display
  'benchmark-runner-agent',     // For benchmark integration
  'regression-detection-agent', // For regression analysis
  'quality-scorer-agent',       // For quality assessment
  'model-comparator-agent',     // For model comparison
] as const;

// =============================================================================
// VERSIONING RULES
// =============================================================================

export const GOLDEN_DATASET_VERSIONING_RULES = {
  major: 'Breaking changes to input/output schemas or validation methodology',
  minor: 'New match types, new validation methods, optional fields',
  patch: 'Bug fixes, accuracy improvements, documentation updates',
} as const;

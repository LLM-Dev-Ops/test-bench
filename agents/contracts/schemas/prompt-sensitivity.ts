/**
 * Prompt Sensitivity Agent Contract Schemas
 *
 * AGENT PURPOSE:
 * Measure output variance under controlled prompt perturbations. This agent applies
 * systematic perturbations to a base prompt (paraphrasing, instruction rephrasing,
 * format changes, etc.) and measures how model outputs vary, computing variance metrics.
 *
 * This agent:
 * - Analyzes prompt sensitivity
 * - Does NOT optimize prompts
 * - Does NOT generate improved prompts
 * - Does NOT compare models
 *
 * decision_type: "prompt_sensitivity_analysis"
 */

import { z } from 'zod';
import { AgentIdentifierSchema, ExecutionRefSchema, DecisionEventSchema } from './base';

// =============================================================================
// AGENT METADATA
// =============================================================================

export const PROMPT_SENSITIVITY_AGENT = {
  agent_id: 'prompt-sensitivity',
  agent_version: '1.0.0',
  decision_type: 'prompt_sensitivity_analysis',
} as const;

// =============================================================================
// ENUMS & TYPES
// =============================================================================

/**
 * Types of prompt perturbations that can be applied
 */
export const PerturbationType = z.enum([
  'paraphrase',              // Semantic-preserving rewording
  'instruction_rephrase',    // Alternative instruction phrasing
  'format_change',           // Structural formatting changes
  'tone_shift',              // Formal/informal tone variations
  'detail_expansion',        // Adding more context/detail
  'detail_reduction',        // Removing context/detail
  'order_change',            // Reordering prompt components
  'emphasis_change',         // Changing emphasis/focus
]);

export type PerturbationType = z.infer<typeof PerturbationType>;

/**
 * Similarity metrics for comparing outputs
 */
export const SimilarityMetric = z.enum([
  'cosine',        // Cosine similarity of embeddings
  'jaccard',       // Jaccard similarity of token sets
  'levenshtein',   // Levenshtein distance
  'semantic',      // Semantic similarity (model-based)
]);

export type SimilarityMetric = z.infer<typeof SimilarityMetric>;

/**
 * Statistical tests for variance analysis
 */
export const StatisticalTest = z.enum([
  'variance',         // Basic variance calculation
  'anova',            // ANOVA for group differences
  'kruskal_wallis',   // Non-parametric alternative to ANOVA
]);

export type StatisticalTest = z.infer<typeof StatisticalTest>;

// =============================================================================
// INPUT SCHEMAS
// =============================================================================

/**
 * Provider configuration for sensitivity analysis
 */
export const SensitivityProviderConfigSchema = z.object({
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
});

export type SensitivityProviderConfig = z.infer<typeof SensitivityProviderConfigSchema>;

/**
 * Custom user-provided perturbation
 */
export const CustomPerturbationSchema = z.object({
  type: PerturbationType,
  prompt: z.string().min(1),
  description: z.string().optional(),
});

export type CustomPerturbation = z.infer<typeof CustomPerturbationSchema>;

/**
 * Perturbation configuration
 */
export const PerturbationConfigSchema = z.object({
  types: z.array(PerturbationType).min(1),
  perturbations_per_type: z.number().min(1).max(10).default(3),
  auto_generate: z.boolean().default(true),
  custom_perturbations: z.array(CustomPerturbationSchema).optional(),
});

export type PerturbationConfig = z.infer<typeof PerturbationConfigSchema>;

/**
 * Sampling configuration
 */
export const SamplingConfigSchema = z.object({
  runs_per_perturbation: z.number().min(1).max(20).default(5),
  temperature: z.number().min(0).max(2).optional(),
  max_tokens: z.number().positive().default(1024),
  top_p: z.number().min(0).max(1).optional(),
});

export type SamplingConfig = z.infer<typeof SamplingConfigSchema>;

/**
 * Analysis configuration
 */
export const AnalysisConfigSchema = z.object({
  similarity_metric: SimilarityMetric.default('cosine'),
  compute_embeddings: z.boolean().default(true),
  statistical_tests: z.array(StatisticalTest).default(['variance', 'anova']),
});

export type AnalysisConfig = z.infer<typeof AnalysisConfigSchema>;

/**
 * Execution configuration
 */
export const SensitivityExecutionConfigSchema = z.object({
  concurrency: z.number().positive().max(20).default(5),
  timeout_ms: z.number().positive().default(300000), // 5 minutes
  save_responses: z.boolean().default(true),
});

export type SensitivityExecutionConfig = z.infer<typeof SensitivityExecutionConfigSchema>;

/**
 * Main input schema for Prompt Sensitivity Agent
 */
export const PromptSensitivityInputSchema = z.object({
  // Required: base prompt to analyze
  base_prompt: z.string().min(1),
  provider: SensitivityProviderConfigSchema,

  // Required: perturbation configuration
  perturbation_config: PerturbationConfigSchema,

  // Required: sampling configuration
  sampling_config: SamplingConfigSchema,

  // Optional: analysis configuration
  analysis_config: AnalysisConfigSchema.optional(),

  // Optional: execution configuration
  execution_config: SensitivityExecutionConfigSchema.optional(),

  // Optional: caller context
  caller_id: z.string().optional(),
  correlation_id: z.string().uuid().optional(),
});

export type PromptSensitivityInput = z.infer<typeof PromptSensitivityInputSchema>;

// =============================================================================
// OUTPUT SCHEMAS
// =============================================================================

/**
 * Token usage for a single run
 */
export const SensitivityTokenUsageSchema = z.object({
  prompt_tokens: z.number().nonnegative(),
  completion_tokens: z.number().nonnegative(),
  total_tokens: z.number().nonnegative(),
});

export type SensitivityTokenUsage = z.infer<typeof SensitivityTokenUsageSchema>;

/**
 * Single run result within a perturbation
 */
export const PerturbationRunSchema = z.object({
  run_id: z.string(),
  response: z.string(),
  latency_ms: z.number().nonnegative(),
  tokens_used: SensitivityTokenUsageSchema,
  success: z.boolean(),
  error_message: z.string().optional(),
});

export type PerturbationRun = z.infer<typeof PerturbationRunSchema>;

/**
 * Aggregate metrics for a perturbation
 */
export const PerturbationAggregateMetricsSchema = z.object({
  response_variance: z.number().min(0).max(1), // How much responses varied (0=identical, 1=completely different)
  semantic_similarity_to_base: z.number().min(0).max(1), // Similarity to base prompt responses
  average_latency_ms: z.number().nonnegative(),
  latency_variance: z.number().nonnegative(),
  average_tokens: z.number().nonnegative(),
  success_rate: z.number().min(0).max(1),
});

export type PerturbationAggregateMetrics = z.infer<typeof PerturbationAggregateMetricsSchema>;

/**
 * Results for a single perturbation
 */
export const PerturbationResultSchema = z.object({
  type: PerturbationType,
  perturbation_id: z.string(),
  perturbation_prompt: z.string(),
  runs: z.array(PerturbationRunSchema),
  aggregate_metrics: PerturbationAggregateMetricsSchema,
});

export type PerturbationResult = z.infer<typeof PerturbationResultSchema>;

/**
 * Confidence interval for overall sensitivity
 */
export const ConfidenceIntervalSchema = z.object({
  lower: z.number().min(0).max(1),
  upper: z.number().min(0).max(1),
});

export type ConfidenceInterval = z.infer<typeof ConfidenceIntervalSchema>;

/**
 * Overall sensitivity analysis results
 */
export const OverallSensitivitySchema = z.object({
  variance_score: z.number().min(0).max(1), // Overall sensitivity (0=robust, 1=highly sensitive)
  most_sensitive_perturbation: PerturbationType,
  least_sensitive_perturbation: PerturbationType,
  confidence_interval: ConfidenceIntervalSchema,
  statistical_significance: z.boolean(),
  p_value: z.number().min(0).max(1).optional(),
  degrees_of_freedom: z.number().nonnegative().optional(),
});

export type OverallSensitivity = z.infer<typeof OverallSensitivitySchema>;

/**
 * Main output schema for Prompt Sensitivity Agent
 */
export const PromptSensitivityOutputSchema = z.object({
  // Execution identity
  execution_id: z.string().uuid(),
  base_prompt: z.string(),
  provider_name: z.string(),
  model_id: z.string(),

  // Timing
  started_at: z.string().datetime(),
  completed_at: z.string().datetime(),
  total_duration_ms: z.number().nonnegative(),

  // Perturbation results
  perturbation_results: z.array(PerturbationResultSchema),

  // Overall sensitivity analysis
  overall_sensitivity: OverallSensitivitySchema,

  // Base response samples (for comparison)
  base_response_samples: z.array(z.string()),

  // Summary statistics
  total_perturbations: z.number().nonnegative(),
  total_runs: z.number().nonnegative(),
  successful_runs: z.number().nonnegative(),
  failed_runs: z.number().nonnegative(),

  // Warnings and issues
  warnings: z.array(z.string()),
});

export type PromptSensitivityOutput = z.infer<typeof PromptSensitivityOutputSchema>;

// =============================================================================
// DECISION EVENT SCHEMA (SPECIALIZED)
// =============================================================================

/**
 * Prompt Sensitivity Decision Event
 * Extends base DecisionEvent with sensitivity-specific outputs
 */
export const PromptSensitivityDecisionEventSchema = DecisionEventSchema.extend({
  decision_type: z.literal('prompt_sensitivity_analysis'),
  outputs: PromptSensitivityOutputSchema,
});

export type PromptSensitivityDecisionEvent = z.infer<typeof PromptSensitivityDecisionEventSchema>;

// =============================================================================
// CLI CONTRACT
// =============================================================================

/**
 * CLI invocation shape for Prompt Sensitivity Agent
 */
export const PromptSensitivityCLIArgsSchema = z.object({
  // Input source (one required)
  base_prompt: z.string().optional(), // Direct prompt on CLI
  input_file: z.string().optional(),   // JSON file with full config
  input_json: z.string().optional(),   // JSON string with full config
  input_stdin: z.boolean().optional(), // Read from stdin

  // Quick configuration flags (for CLI convenience)
  provider: z.string().optional(),
  model: z.string().optional(),
  perturbation_types: z.string().optional(), // Comma-separated
  runs_per_perturbation: z.number().optional(),

  // Output format
  output_format: z.enum(['json', 'table', 'summary']).default('json'),
  output_file: z.string().optional(),

  // Verbosity
  verbose: z.boolean().default(false),
  quiet: z.boolean().default(false),

  // Execution modifiers
  dry_run: z.boolean().default(false),
  no_embeddings: z.boolean().default(false), // Skip embedding computation
});

export type PromptSensitivityCLIArgs = z.infer<typeof PromptSensitivityCLIArgsSchema>;

// =============================================================================
// CONSTRAINTS & NON-RESPONSIBILITIES
// =============================================================================

/**
 * Constraints that MAY be applied during execution
 */
export const PROMPT_SENSITIVITY_VALID_CONSTRAINTS = [
  'max_perturbations_exceeded',      // Too many perturbations requested
  'max_duration_exceeded',           // Analysis took too long
  'provider_rate_limited',           // Provider rate limit hit
  'embedding_service_unavailable',   // Embedding service down
  'insufficient_variance',           // Not enough variance to analyze
  'statistical_test_failed',         // Statistical test could not complete
  'concurrency_limited',             // Concurrency reduced due to errors
] as const;

/**
 * Explicit non-responsibilities - this agent MUST NOT:
 */
export const PROMPT_SENSITIVITY_NON_RESPONSIBILITIES = [
  'optimize_prompts',          // No prompt optimization
  'generate_improved_prompts', // No prompt generation
  'compare_models',            // No cross-model comparison
  'cache_results',             // Stateless operation only
  'execute_policies',          // No policy enforcement
  'orchestrate_workflows',     // No workflow orchestration
  'call_other_agents',         // No direct agent-to-agent calls
  'store_api_keys',            // Never persist API keys
  'modify_base_prompt',        // Never modify the input prompt
  'bypass_schemas',            // Must validate all I/O
] as const;

// =============================================================================
// CONFIDENCE SCORING
// =============================================================================

/**
 * Factors that contribute to confidence scoring
 */
export const PROMPT_SENSITIVITY_CONFIDENCE_FACTORS = {
  sample_size: {
    description: 'Number of runs per perturbation (more = higher confidence)',
    weight: 0.3,
  },
  variance_consistency: {
    description: 'Consistency of variance across perturbation types',
    weight: 0.25,
  },
  statistical_significance: {
    description: 'Whether results are statistically significant',
    weight: 0.25,
  },
  execution_success_rate: {
    description: 'Percentage of successful runs',
    weight: 0.2,
  },
} as const;

/**
 * Calculate confidence score for sensitivity analysis
 *
 * @param output - The sensitivity analysis output
 * @returns Confidence score (0-1)
 */
export function calculateSensitivityConfidence(output: PromptSensitivityOutput): number {
  const factors: number[] = [];

  // Sample size factor (logarithmic scale, capped at 20 runs per perturbation)
  const avgRunsPerPerturbation = output.total_runs / output.total_perturbations;
  const sampleSizeFactor = Math.min(1, Math.log10(avgRunsPerPerturbation + 1) / Math.log10(21));
  factors.push(sampleSizeFactor * PROMPT_SENSITIVITY_CONFIDENCE_FACTORS.sample_size.weight);

  // Variance consistency factor (lower coefficient of variation = more consistent)
  const variances = output.perturbation_results.map(r => r.aggregate_metrics.response_variance);
  const meanVariance = variances.reduce((a, b) => a + b, 0) / variances.length;
  const varianceStdDev = Math.sqrt(
    variances.reduce((sum, v) => sum + Math.pow(v - meanVariance, 2), 0) / variances.length
  );
  const coefficientOfVariation = meanVariance > 0 ? varianceStdDev / meanVariance : 0;
  const consistencyFactor = Math.max(0, 1 - coefficientOfVariation);
  factors.push(consistencyFactor * PROMPT_SENSITIVITY_CONFIDENCE_FACTORS.variance_consistency.weight);

  // Statistical significance factor
  const significanceFactor = output.overall_sensitivity.statistical_significance ? 1 : 0.5;
  factors.push(significanceFactor * PROMPT_SENSITIVITY_CONFIDENCE_FACTORS.statistical_significance.weight);

  // Execution success rate
  const successRate = output.successful_runs / output.total_runs;
  factors.push(successRate * PROMPT_SENSITIVITY_CONFIDENCE_FACTORS.execution_success_rate.weight);

  return Math.min(1, Math.max(0, factors.reduce((a, b) => a + b, 0)));
}

// =============================================================================
// CORE BUNDLE CONSUMERS
// =============================================================================

/**
 * Core bundles that may consume this agent's output
 */
export const PROMPT_SENSITIVITY_ALLOWED_CONSUMERS = [
  'llm-orchestrator',           // For workflow coordination
  'llm-test-runner',            // For test execution
  'prompt-engineering-suite',   // For prompt optimization workflows
  'llm-analytics',              // For aggregation/analysis
] as const;

// =============================================================================
// VERSIONING RULES
// =============================================================================

export const PROMPT_SENSITIVITY_VERSIONING_RULES = {
  major: 'Breaking changes to input/output schemas, perturbation types',
  minor: 'New perturbation types, new metrics, new statistical tests',
  patch: 'Bug fixes, performance improvements, documentation',
} as const;

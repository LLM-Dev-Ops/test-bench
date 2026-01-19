/**
 * Synthetic Data Generator Agent Contract Schemas
 *
 * AGENT PURPOSE:
 * Generate synthetic datasets for testing, benchmarking, and stress evaluation
 * of LLM systems. This agent produces high-quality synthetic data across multiple
 * formats using pure algorithmic generation (no LLM calls).
 *
 * This agent:
 * - Generates synthetic datasets (YES)
 * - Supports multiple data types (YES)
 * - Applies configurable generation strategies (YES)
 * - Does NOT call LLMs for generation (NO - pure algorithmic)
 * - Does NOT execute benchmarks (NO - use benchmark-runner)
 * - Does NOT compare models (NO)
 * - Does NOT enforce policy (NO)
 *
 * decision_type: "synthetic_data_generation"
 */

import { z } from 'zod';
import { DecisionEventSchema } from './base';

// =============================================================================
// AGENT METADATA
// =============================================================================

export const SYNTHETIC_DATA_GENERATOR_AGENT = {
  agent_id: 'synthetic-data-generator',
  agent_version: '1.0.0',
  decision_type: 'synthetic_data_generation',
} as const;

// =============================================================================
// ENUMS
// =============================================================================

/**
 * Supported synthetic data types
 */
export const SyntheticDataTypeSchema = z.enum([
  'text_prompt',             // Single prompts/instructions
  'qa_pair',                 // Question-answer pairs
  'multi_turn_conversation', // Multi-turn dialogues
  'coding_task',             // Code problems with test cases
  'summarization',           // Document + summary pairs
  'creative_writing',        // Creative writing prompts
  'classification',          // Text + label pairs
  'entity_extraction',       // Text + entities pairs
  'translation',             // Source + target language pairs
  'reasoning_chain',         // Multi-step reasoning problems
]);

export type SyntheticDataType = z.infer<typeof SyntheticDataTypeSchema>;

/**
 * Generation strategies
 */
export const GenerationStrategySchema = z.enum([
  'template_based',          // Generate from templates with placeholders
  'variation',               // Generate variations from seed examples
  'distribution_aware',      // Match specified distributions
  'edge_case',               // Focus on boundary conditions
  'adversarial',             // Generate challenging/tricky examples
  'combinatorial',           // Combine elements systematically
  'progressive_difficulty',  // Increase complexity gradually
  'cross_domain',            // Mix elements across domains
]);

export type GenerationStrategy = z.infer<typeof GenerationStrategySchema>;

// =============================================================================
// INPUT SCHEMAS
// =============================================================================

/**
 * Placeholder definition for template-based generation
 */
export const PlaceholderSchema = z.object({
  name: z.string().min(1),
  type: z.enum(['word_list', 'number_range', 'pattern', 'reference']),
  values: z.array(z.string()).optional(),
  min: z.number().optional(),
  max: z.number().optional(),
  pattern: z.string().optional(),
});

export type Placeholder = z.infer<typeof PlaceholderSchema>;

/**
 * Template definition for template-based generation
 */
export const TemplateSchema = z.object({
  template_id: z.string().min(1),
  template: z.string().min(1),
  placeholders: z.array(PlaceholderSchema),
  weight: z.number().min(0).max(1).default(1),
});

export type Template = z.infer<typeof TemplateSchema>;

/**
 * Seed example for variation-based generation
 */
export const SeedExampleSchema = z.object({
  id: z.string().min(1),
  content: z.record(z.unknown()),
  metadata: z.record(z.unknown()).optional(),
});

export type SeedExample = z.infer<typeof SeedExampleSchema>;

/**
 * Generation constraints
 */
export const GenerationConstraintsSchema = z.object({
  min_length_chars: z.number().nonnegative().optional(),
  max_length_chars: z.number().positive().optional(),
  min_tokens_approx: z.number().nonnegative().optional(),
  max_tokens_approx: z.number().positive().optional(),
  required_keywords: z.array(z.string()).optional(),
  forbidden_keywords: z.array(z.string()).optional(),
  language: z.string().default('en'),
  tone: z.enum(['formal', 'casual', 'technical', 'creative']).optional(),
  domain: z.string().optional(),
  complexity_level: z.enum(['simple', 'moderate', 'complex', 'expert']).optional(),
});

export type GenerationConstraints = z.infer<typeof GenerationConstraintsSchema>;

/**
 * Difficulty distribution for progressive strategy
 */
export const DifficultyDistributionSchema = z.object({
  easy: z.number().min(0).max(1).default(0.3),
  medium: z.number().min(0).max(1).default(0.5),
  hard: z.number().min(0).max(1).default(0.2),
});

export type DifficultyDistribution = z.infer<typeof DifficultyDistributionSchema>;

/**
 * Coding-specific configuration
 */
export const CodingConfigSchema = z.object({
  languages: z.array(z.string()).default(['python']),
  include_test_cases: z.boolean().default(true),
  test_case_count: z.number().positive().default(3),
  include_edge_cases: z.boolean().default(true),
  problem_types: z.array(z.enum([
    'algorithm',
    'data_structure',
    'string_manipulation',
    'math',
    'recursion',
    'sorting',
    'searching',
    'dynamic_programming',
  ])).optional(),
});

export type CodingConfig = z.infer<typeof CodingConfigSchema>;

/**
 * Persona definition for conversations
 */
export const PersonaSchema = z.object({
  name: z.string().min(1),
  description: z.string(),
  traits: z.array(z.string()),
});

export type Persona = z.infer<typeof PersonaSchema>;

/**
 * Conversation-specific configuration
 */
export const ConversationConfigSchema = z.object({
  min_turns: z.number().positive().default(2),
  max_turns: z.number().positive().default(10),
  personas: z.array(PersonaSchema).optional(),
  topics: z.array(z.string()).optional(),
  include_system_messages: z.boolean().default(false),
});

export type ConversationConfig = z.infer<typeof ConversationConfigSchema>;

/**
 * Main input schema for Synthetic Data Generator Agent
 */
export const SyntheticDataGeneratorInputSchema = z.object({
  // Required fields
  data_type: SyntheticDataTypeSchema,
  generation_strategy: GenerationStrategySchema,
  count: z.number().positive().max(10000),

  // Optional: Seed examples for variation strategy
  seed_examples: z.array(SeedExampleSchema).optional(),

  // Optional: Templates for template-based generation
  templates: z.array(TemplateSchema).optional(),

  // Optional: Generation constraints
  constraints: GenerationConstraintsSchema.optional(),

  // Optional: Difficulty distribution for progressive strategy
  difficulty_distribution: DifficultyDistributionSchema.optional(),

  // Optional: Domain-specific configurations
  coding_config: CodingConfigSchema.optional(),
  conversation_config: ConversationConfigSchema.optional(),

  // Optional: Output format
  output_format: z.enum(['json', 'jsonl', 'csv']).default('json'),
  include_metadata: z.boolean().default(true),

  // Optional: Determinism
  random_seed: z.number().optional(),

  // Caller context
  caller_id: z.string().optional(),
  correlation_id: z.string().uuid().optional(),
});

export type SyntheticDataGeneratorInput = z.infer<typeof SyntheticDataGeneratorInputSchema>;

// =============================================================================
// OUTPUT SCHEMAS
// =============================================================================

/**
 * Quality indicators for a generated item
 */
export const QualityIndicatorsSchema = z.object({
  length_chars: z.number().nonnegative(),
  token_count_approx: z.number().nonnegative(),
  complexity_score: z.number().min(0).max(1),
  uniqueness_hash: z.string(),
  constraint_satisfaction: z.number().min(0).max(1),
});

export type QualityIndicators = z.infer<typeof QualityIndicatorsSchema>;

/**
 * Generation metadata for a single item
 */
export const GenerationMetadataSchema = z.object({
  strategy_used: GenerationStrategySchema,
  template_id: z.string().optional(),
  seed_example_id: z.string().optional(),
  difficulty_level: z.enum(['easy', 'medium', 'hard']).optional(),
  variation_index: z.number().optional(),
});

export type GenerationMetadata = z.infer<typeof GenerationMetadataSchema>;

/**
 * Single generated item
 */
export const GeneratedItemSchema = z.object({
  item_id: z.string().uuid(),
  data_type: SyntheticDataTypeSchema,

  // The generated content (structure varies by data_type)
  content: z.record(z.unknown()),

  // Generation metadata
  generation_metadata: GenerationMetadataSchema,

  // Quality indicators (heuristic)
  quality_indicators: QualityIndicatorsSchema,

  // Tags for categorization
  tags: z.array(z.string()).optional(),
});

export type GeneratedItem = z.infer<typeof GeneratedItemSchema>;

/**
 * Generation statistics
 */
export const GenerationStatsSchema = z.object({
  requested_count: z.number().nonnegative(),
  generated_count: z.number().nonnegative(),
  failed_count: z.number().nonnegative(),
  duplicate_count: z.number().nonnegative(),
  strategy_distribution: z.record(z.number()),
  template_distribution: z.record(z.number()).optional(),
  difficulty_distribution: z.record(z.number()).optional(),
});

export type GenerationStats = z.infer<typeof GenerationStatsSchema>;

/**
 * Aggregate quality metrics
 */
export const QualityMetricsSchema = z.object({
  avg_length_chars: z.number().nonnegative(),
  avg_token_count: z.number().nonnegative(),
  avg_complexity_score: z.number().min(0).max(1),
  constraint_satisfaction_rate: z.number().min(0).max(1),
  unique_items_rate: z.number().min(0).max(1),
});

export type QualityMetrics = z.infer<typeof QualityMetricsSchema>;

/**
 * Length distribution analysis
 */
export const LengthDistributionSchema = z.object({
  min: z.number().nonnegative(),
  max: z.number().nonnegative(),
  mean: z.number().nonnegative(),
  median: z.number().nonnegative(),
  stddev: z.number().nonnegative(),
});

export type LengthDistribution = z.infer<typeof LengthDistributionSchema>;

/**
 * Distribution analysis
 */
export const DistributionAnalysisSchema = z.object({
  length_distribution: LengthDistributionSchema,
  difficulty_actual: z.record(z.number()).optional(),
  keyword_coverage: z.record(z.number()).optional(),
});

export type DistributionAnalysis = z.infer<typeof DistributionAnalysisSchema>;

/**
 * Input config summary for output
 */
export const InputConfigSummarySchema = z.object({
  data_type: SyntheticDataTypeSchema,
  generation_strategy: GenerationStrategySchema,
  requested_count: z.number(),
  constraints_applied: z.array(z.string()),
});

export type InputConfigSummary = z.infer<typeof InputConfigSummarySchema>;

/**
 * Main output schema for Synthetic Data Generator Agent
 */
export const SyntheticDataGeneratorOutputSchema = z.object({
  // Execution identity
  execution_id: z.string().uuid(),

  // The generated items
  generated_items: z.array(GeneratedItemSchema),

  // Generation statistics
  generation_stats: GenerationStatsSchema,

  // Quality metrics (aggregate)
  quality_metrics: QualityMetricsSchema,

  // Distribution analysis
  distribution_analysis: DistributionAnalysisSchema,

  // Timing
  started_at: z.string().datetime(),
  completed_at: z.string().datetime(),
  duration_ms: z.number().nonnegative(),

  // Configuration used
  input_config_summary: InputConfigSummarySchema,
});

export type SyntheticDataGeneratorOutput = z.infer<typeof SyntheticDataGeneratorOutputSchema>;

// =============================================================================
// DECISION EVENT SCHEMA (SPECIALIZED)
// =============================================================================

/**
 * Synthetic Data Generator Decision Event
 * Extends base DecisionEvent with generation-specific outputs
 */
export const SyntheticDataGeneratorDecisionEventSchema = DecisionEventSchema.extend({
  decision_type: z.literal('synthetic_data_generation'),
  outputs: SyntheticDataGeneratorOutputSchema,
});

export type SyntheticDataGeneratorDecisionEvent = z.infer<typeof SyntheticDataGeneratorDecisionEventSchema>;

// =============================================================================
// CLI CONTRACT
// =============================================================================

/**
 * CLI invocation shape for Synthetic Data Generator Agent
 */
export const SyntheticDataGeneratorCLIArgsSchema = z.object({
  // Input source (one required)
  input_file: z.string().optional(),
  input_json: z.string().optional(),

  // Quick generation mode
  type: z.string().optional(),
  strategy: z.string().optional(),
  count: z.number().optional(),

  // Preset configurations
  preset: z.string().optional(),

  // Output format
  output_format: z.enum(['json', 'jsonl', 'csv']).default('json'),
  output_file: z.string().optional(),

  // Modifiers
  seed: z.number().optional(),
  verbose: z.boolean().default(false),
  quiet: z.boolean().default(false),
  dry_run: z.boolean().default(false),
});

export type SyntheticDataGeneratorCLIArgs = z.infer<typeof SyntheticDataGeneratorCLIArgsSchema>;

// =============================================================================
// CONSTRAINTS & NON-RESPONSIBILITIES
// =============================================================================

/**
 * Constraints that MAY be applied during execution
 */
export const SYNTHETIC_DATA_VALID_CONSTRAINTS = [
  'max_generation_time_exceeded',
  'max_item_count_reached',
  'memory_limit_approached',
  'uniqueness_threshold_unmet',
  'constraint_satisfaction_below_threshold',
  'template_exhausted',
  'seed_examples_depleted',
  'complexity_target_unreachable',
] as const;

/**
 * Explicit non-responsibilities - this agent MUST NOT:
 */
export const SYNTHETIC_DATA_NON_RESPONSIBILITIES = [
  'call_llms',                  // No LLM API calls - pure algorithmic generation
  'compare_models',             // No model comparison logic
  'execute_benchmarks',         // Use benchmark-runner for that
  'score_quality_semantically', // No semantic quality scoring (no LLM)
  'enforce_policy',             // No policy enforcement
  'orchestrate_workflows',      // No workflow orchestration
  'call_other_agents',          // No direct agent-to-agent calls
  'store_api_keys',             // Never persist API keys
  'persist_pii',                // Never store or generate PII
  'generate_harmful_content',   // No harmful/offensive content generation
  'bypass_schemas',             // Must validate all I/O
  'execute_generated_code',     // Generate code, never execute it
] as const;

// =============================================================================
// CONFIDENCE SCORING
// =============================================================================

/**
 * Factors that contribute to confidence scoring
 */
export const SYNTHETIC_DATA_CONFIDENCE_FACTORS = {
  coverage_score: {
    description: 'Variety and completeness of generated data (generated/requested ratio)',
    weight: 0.25,
  },
  constraint_satisfaction: {
    description: 'How well constraints were met',
    weight: 0.30,
  },
  distribution_match: {
    description: 'Match between requested and actual distributions',
    weight: 0.20,
  },
  uniqueness_score: {
    description: 'Percentage of unique items generated',
    weight: 0.25,
  },
} as const;

/**
 * Calculate confidence score based on generation results
 */
export function calculateSyntheticDataConfidence(
  output: SyntheticDataGeneratorOutput,
  requestedDifficultyDistribution?: DifficultyDistribution
): number {
  const factors: number[] = [];

  // Coverage score (generated/requested ratio)
  const coverageScore = output.generation_stats.requested_count > 0
    ? output.generation_stats.generated_count / output.generation_stats.requested_count
    : 0;
  factors.push(Math.min(1, coverageScore) * SYNTHETIC_DATA_CONFIDENCE_FACTORS.coverage_score.weight);

  // Constraint satisfaction
  factors.push(
    output.quality_metrics.constraint_satisfaction_rate *
    SYNTHETIC_DATA_CONFIDENCE_FACTORS.constraint_satisfaction.weight
  );

  // Distribution match (compare requested vs actual difficulty if available)
  let distributionScore = 0.8; // Default score if no distribution specified
  if (requestedDifficultyDistribution && output.distribution_analysis.difficulty_actual) {
    const actual = output.distribution_analysis.difficulty_actual;
    const requested = requestedDifficultyDistribution;

    // Calculate simple distribution match using absolute differences
    const easyDiff = Math.abs((actual['easy'] ?? 0) - requested.easy);
    const mediumDiff = Math.abs((actual['medium'] ?? 0) - requested.medium);
    const hardDiff = Math.abs((actual['hard'] ?? 0) - requested.hard);
    const avgDiff = (easyDiff + mediumDiff + hardDiff) / 3;

    distributionScore = Math.max(0, 1 - avgDiff);
  }
  factors.push(distributionScore * SYNTHETIC_DATA_CONFIDENCE_FACTORS.distribution_match.weight);

  // Uniqueness score
  factors.push(
    output.quality_metrics.unique_items_rate *
    SYNTHETIC_DATA_CONFIDENCE_FACTORS.uniqueness_score.weight
  );

  return Math.min(1, Math.max(0, factors.reduce((a, b) => a + b, 0)));
}

// =============================================================================
// CORE BUNDLE CONSUMERS
// =============================================================================

/**
 * Core bundles that may consume this agent's output
 */
export const SYNTHETIC_DATA_ALLOWED_CONSUMERS = [
  'llm-orchestrator',       // For workflow coordination
  'llm-observatory',        // For telemetry/monitoring
  'llm-analytics',          // For aggregation/analysis
  'llm-test-bench-ui',      // For dashboard display
  'benchmark-runner',       // Can consume generated data for benchmarking
  'stress-test',            // Can consume generated data for stress testing
  'quality-scoring',        // Can validate generated data quality
] as const;

// =============================================================================
// VERSIONING RULES
// =============================================================================

export const SYNTHETIC_DATA_VERSIONING_RULES = {
  major: 'Breaking changes to input/output schemas or generation algorithms',
  minor: 'New data types, generation strategies, or optional fields',
  patch: 'Bug fixes, performance improvements, documentation',
} as const;

// =============================================================================
// PRESETS
// =============================================================================

/**
 * Built-in presets for common use cases
 */
export const GENERATION_PRESETS = {
  'qa-benchmark': {
    data_type: 'qa_pair',
    generation_strategy: 'distribution_aware',
    difficulty_distribution: { easy: 0.3, medium: 0.5, hard: 0.2 },
    constraints: { complexity_level: 'moderate' },
  },
  'coding-challenge': {
    data_type: 'coding_task',
    generation_strategy: 'progressive_difficulty',
    coding_config: {
      languages: ['python', 'javascript'],
      include_test_cases: true,
      test_case_count: 5,
      include_edge_cases: true,
    },
  },
  'stress-test-prompts': {
    data_type: 'text_prompt',
    generation_strategy: 'edge_case',
    constraints: {
      max_length_chars: 10000,
      complexity_level: 'expert',
    },
  },
  'conversation-dataset': {
    data_type: 'multi_turn_conversation',
    generation_strategy: 'template_based',
    conversation_config: {
      min_turns: 3,
      max_turns: 8,
    },
  },
  'adversarial-inputs': {
    data_type: 'text_prompt',
    generation_strategy: 'adversarial',
    constraints: { complexity_level: 'complex' },
  },
} as const;

export type GenerationPreset = keyof typeof GENERATION_PRESETS;

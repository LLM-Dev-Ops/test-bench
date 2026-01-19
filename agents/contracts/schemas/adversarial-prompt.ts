/**
 * Adversarial Prompt Agent Contract Schemas
 *
 * AGENT PURPOSE:
 * Generate adversarial prompts used by Red Team and Stress Test agents.
 * Produces categorized, severity-ranked adversarial inputs designed to probe
 * LLM robustness, safety boundaries, and failure modes without executing
 * actual attacks or bypassing safety measures.
 *
 * This agent:
 * - Generates adversarial prompts for testing purposes (YES)
 * - Categorizes prompts by attack vector and severity (YES)
 * - Provides metadata for test execution (YES)
 * - Does NOT execute prompts against models (use stress-test agent)
 * - Does NOT evaluate model responses (use evaluation agents)
 * - Does NOT orchestrate test workflows (use LLM-Orchestrator)
 * - Does NOT attempt actual attacks or jailbreaks
 * - Does NOT store or generate harmful content for malicious purposes
 *
 * decision_type: "adversarial_prompt_generation"
 */

import { z } from 'zod';
import { DecisionEventSchema, ExecutionRefSchema } from './base';

// =============================================================================
// AGENT METADATA
// =============================================================================

export const ADVERSARIAL_PROMPT_AGENT = {
  agent_id: 'adversarial-prompt',
  agent_version: '1.0.0',
  decision_type: 'adversarial_prompt_generation',
} as const;

// =============================================================================
// ADVERSARIAL PROMPT CATEGORIES
// =============================================================================

/**
 * Categories of adversarial prompts that can be generated
 */
export const AdversarialCategorySchema = z.enum([
  // Prompt manipulation attacks
  'prompt_injection',           // Attempts to override/bypass system prompts
  'jailbreak_attempt',          // Attempts to bypass safety guidelines
  'instruction_override',       // Attempts to change model behavior
  'role_play_escape',           // Escape role-play constraints

  // Encoding and format attacks
  'encoding_attacks',           // Unicode tricks, RTL override, normalization
  'format_confusion',           // Mixed formats (JSON in markdown, nested structures)
  'delimiter_attacks',          // Exploiting delimiters and special markers
  'whitespace_exploitation',    // Hidden whitespace, zero-width chars

  // Content boundary probing
  'content_policy_probe',       // Test content policy boundaries
  'sensitive_topic_probe',      // Probe handling of sensitive topics
  'harmful_request_variants',   // Obfuscated harmful requests
  'dual_use_content',           // Legitimate content with dual-use potential

  // Structural attacks
  'context_overflow',           // Exceed context window boundaries
  'token_manipulation',         // Token boundary exploits
  'repetition_attacks',         // Repeated patterns causing issues
  'nested_structures',          // Deeply nested JSON/XML/markdown

  // Logic and reasoning attacks
  'logical_contradictions',     // Self-contradictory instructions
  'multi_turn_manipulation',    // Building up to harmful request
  'authority_impersonation',    // Claiming false authority
  'urgency_manipulation',       // Creating false urgency

  // Output manipulation
  'output_format_attacks',      // Force specific output formats
  'hallucination_triggers',     // Prompts designed to cause hallucination
  'confidence_manipulation',    // Manipulate model confidence signals

  // Technical exploits
  'api_confusion',              // Confuse API boundaries
  'system_prompt_extraction',   // Attempt to extract system prompts
  'training_data_extraction',   // Attempt to extract training data
]);

export type AdversarialCategory = z.infer<typeof AdversarialCategorySchema>;

/**
 * Severity levels for adversarial prompts
 */
export const AdversarialSeveritySchema = z.enum([
  'low',      // Benign edge cases, formatting issues
  'medium',   // Moderate manipulation attempts
  'high',     // Significant attack vectors
  'critical', // Severe attack vectors (for authorized testing only)
]);

export type AdversarialSeverity = z.infer<typeof AdversarialSeveritySchema>;

/**
 * Target model types for prompt optimization
 */
export const TargetModelTypeSchema = z.enum([
  'general',       // Generic prompts for any model
  'chat',          // Chat/conversational models
  'instruct',      // Instruction-following models
  'code',          // Code generation models
  'multimodal',    // Vision-language models
  'agent',         // Agentic/tool-using models
]);

export type TargetModelType = z.infer<typeof TargetModelTypeSchema>;

// =============================================================================
// INPUT SCHEMAS
// =============================================================================

/**
 * Generation strategy for adversarial prompts
 */
export const AdversarialGenerationStrategySchema = z.enum([
  'template_based',    // Use predefined templates with variations
  'mutation_based',    // Mutate seed prompts
  'combinatorial',     // Combine multiple attack vectors
  'gradient_inspired', // Techniques inspired by gradient-based attacks
  'random_fuzzing',    // Random fuzzing with constraints
]);

export type AdversarialGenerationStrategy = z.infer<typeof AdversarialGenerationStrategySchema>;

/**
 * Template configuration for prompt generation
 */
export const PromptTemplateConfigSchema = z.object({
  template_id: z.string().optional(),
  base_template: z.string().optional(),
  placeholders: z.record(z.string()).optional(),
  variation_count: z.number().positive().default(1),
});

export type PromptTemplateConfig = z.infer<typeof PromptTemplateConfigSchema>;

/**
 * Mutation configuration for prompt generation
 */
export const MutationConfigSchema = z.object({
  seed_prompts: z.array(z.string()).optional(),
  mutation_types: z.array(z.enum([
    'character_substitution',
    'encoding_change',
    'structure_modification',
    'semantic_preservation',
    'delimiter_injection',
    'whitespace_injection',
  ])).default(['encoding_change', 'structure_modification']),
  mutation_rate: z.number().min(0).max(1).default(0.3),
  max_mutations_per_prompt: z.number().positive().default(3),
});

export type MutationConfig = z.infer<typeof MutationConfigSchema>;

/**
 * Filter configuration for generated prompts
 */
export const PromptFilterConfigSchema = z.object({
  min_length_chars: z.number().nonnegative().default(10),
  max_length_chars: z.number().positive().default(10000),
  min_token_estimate: z.number().nonnegative().default(5),
  max_token_estimate: z.number().positive().default(4000),
  exclude_patterns: z.array(z.string()).optional(),
  require_patterns: z.array(z.string()).optional(),
  deduplicate: z.boolean().default(true),
  similarity_threshold: z.number().min(0).max(1).default(0.85),
});

export type PromptFilterConfig = z.infer<typeof PromptFilterConfigSchema>;

/**
 * Main input schema for Adversarial Prompt Agent
 */
export const AdversarialPromptInputSchema = z.object({
  // What to generate (required - callers must be explicit)
  categories: z.array(AdversarialCategorySchema).min(1),
  severities: z.array(AdversarialSeveritySchema).min(1),
  target_model_types: z.array(TargetModelTypeSchema).default(['general']),

  // How many to generate
  count_per_category: z.number().positive().default(5),
  total_max_count: z.number().positive().default(100),

  // Generation strategy
  strategy: AdversarialGenerationStrategySchema.default('template_based'),
  template_config: PromptTemplateConfigSchema.optional(),
  mutation_config: MutationConfigSchema.optional(),

  // Filtering and quality
  filter_config: PromptFilterConfigSchema.optional(),

  // Context for generation
  system_prompt_context: z.string().optional(), // Context about target system
  domain_context: z.string().optional(), // Domain-specific context
  language: z.string().default('en'), // Target language

  // Safety constraints
  include_benign_variants: z.boolean().default(true), // Include safe baseline
  safety_ceiling: AdversarialSeveritySchema.default('high'), // Max severity allowed

  // Metadata
  purpose: z.enum([
    'red_team_testing',
    'stress_testing',
    'security_audit',
    'research',
    'safety_evaluation',
  ]).default('stress_testing'),
  authorization_ref: z.string().optional(), // Reference to authorization

  // Caller context
  caller_id: z.string().optional(),
  correlation_id: z.string().uuid().optional(),
});

export type AdversarialPromptInput = z.infer<typeof AdversarialPromptInputSchema>;

// =============================================================================
// OUTPUT SCHEMAS
// =============================================================================

/**
 * Single generated adversarial prompt
 */
export const GeneratedPromptSchema = z.object({
  // Identity
  prompt_id: z.string().uuid(),

  // Content
  prompt_text: z.string(),
  prompt_hash: z.string().length(64), // SHA-256 for deduplication

  // Classification
  category: AdversarialCategorySchema,
  severity: AdversarialSeveritySchema,
  target_model_types: z.array(TargetModelTypeSchema),

  // Metadata
  attack_vector: z.string(), // Specific attack vector description
  expected_behavior: z.string(), // What a robust model should do
  failure_indicators: z.array(z.string()), // Signs of successful attack

  // Generation metadata
  generation_strategy: AdversarialGenerationStrategySchema,
  template_id: z.string().optional(),
  mutation_chain: z.array(z.string()).optional(),

  // Metrics
  estimated_tokens: z.number().nonnegative(),
  complexity_score: z.number().min(0).max(1),
  novelty_score: z.number().min(0).max(1).optional(),

  // Benign variant (if requested)
  benign_variant: z.string().optional(),
  benign_variant_hash: z.string().optional(),

  // Tags for filtering
  tags: z.array(z.string()),
});

export type GeneratedPrompt = z.infer<typeof GeneratedPromptSchema>;

/**
 * Category summary in output
 */
export const CategorySummarySchema = z.object({
  category: AdversarialCategorySchema,
  total_generated: z.number().nonnegative(),
  by_severity: z.record(AdversarialSeveritySchema, z.number().nonnegative()),
  avg_complexity: z.number().min(0).max(1),
  avg_estimated_tokens: z.number().nonnegative(),
});

export type CategorySummary = z.infer<typeof CategorySummarySchema>;

/**
 * Generation quality metrics
 */
export const GenerationQualityMetricsSchema = z.object({
  total_generated: z.number().nonnegative(),
  total_filtered_out: z.number().nonnegative(),
  filter_reasons: z.record(z.string(), z.number()),
  duplicates_removed: z.number().nonnegative(),
  diversity_score: z.number().min(0).max(1),
  category_coverage: z.number().min(0).max(1),
  severity_distribution: z.record(AdversarialSeveritySchema, z.number()),
});

export type GenerationQualityMetrics = z.infer<typeof GenerationQualityMetricsSchema>;

/**
 * Main output schema for Adversarial Prompt Agent
 */
export const AdversarialPromptOutputSchema = z.object({
  // Execution identity
  execution_id: z.string().uuid(),
  generation_run_id: z.string().uuid(),

  // Generated prompts
  prompts: z.array(GeneratedPromptSchema),

  // Summaries
  category_summaries: z.array(CategorySummarySchema),
  quality_metrics: GenerationQualityMetricsSchema,

  // Request echo (for traceability)
  request_summary: z.object({
    categories_requested: z.array(AdversarialCategorySchema),
    severities_requested: z.array(AdversarialSeveritySchema),
    strategy_used: AdversarialGenerationStrategySchema,
    purpose: z.string(),
  }),

  // Timing
  started_at: z.string().datetime(),
  completed_at: z.string().datetime(),
  duration_ms: z.number().nonnegative(),

  // Constraints applied
  constraints_applied: z.array(z.string()),

  // Warnings (e.g., if ceiling was hit)
  warnings: z.array(z.string()),
});

export type AdversarialPromptOutput = z.infer<typeof AdversarialPromptOutputSchema>;

// =============================================================================
// DECISION EVENT SCHEMA (SPECIALIZED)
// =============================================================================

/**
 * Adversarial Prompt Decision Event
 * Extends base DecisionEvent with agent-specific outputs
 */
export const AdversarialPromptDecisionEventSchema = DecisionEventSchema.extend({
  decision_type: z.literal('adversarial_prompt_generation'),
  outputs: AdversarialPromptOutputSchema,
});

export type AdversarialPromptDecisionEvent = z.infer<typeof AdversarialPromptDecisionEventSchema>;

// =============================================================================
// CLI CONTRACT
// =============================================================================

/**
 * CLI invocation shape for Adversarial Prompt Agent
 */
export const AdversarialPromptCLIArgsSchema = z.object({
  // Input source (one required)
  input_file: z.string().optional(),
  input_json: z.string().optional(),
  input_stdin: z.boolean().optional(),

  // Quick presets
  preset: z.enum([
    'basic',              // Low severity, common categories
    'comprehensive',      // All categories, medium severity
    'red-team',           // High severity for authorized testing
    'encoding-focus',     // Focus on encoding attacks
    'injection-focus',    // Focus on injection attacks
  ]).optional(),

  // Category filters (comma-separated)
  categories: z.string().optional(),

  // Severity filter
  max_severity: AdversarialSeveritySchema.optional(),

  // Count
  count: z.number().positive().optional(),

  // Output format
  output_format: z.enum(['json', 'jsonl', 'csv', 'prompts-only']).default('json'),
  output_file: z.string().optional(),

  // Verbosity
  verbose: z.boolean().default(false),
  quiet: z.boolean().default(false),

  // Execution modifiers
  dry_run: z.boolean().default(false),
  include_benign: z.boolean().default(true),
});

export type AdversarialPromptCLIArgs = z.infer<typeof AdversarialPromptCLIArgsSchema>;

// =============================================================================
// CONSTRAINTS & NON-RESPONSIBILITIES
// =============================================================================

/**
 * Constraints that MAY be applied during execution
 */
export const ADVERSARIAL_PROMPT_VALID_CONSTRAINTS = [
  'severity_ceiling_applied',
  'count_limit_reached',
  'category_filtered',
  'content_policy_filter',
  'duplicate_removed',
  'length_limit_applied',
  'token_limit_applied',
  'authorization_required',
] as const;

/**
 * Explicit non-responsibilities - this agent MUST NOT:
 */
export const ADVERSARIAL_PROMPT_NON_RESPONSIBILITIES = [
  'execute_prompts',            // Does NOT execute against models
  'evaluate_responses',         // Does NOT evaluate model responses
  'compare_models',             // No model comparison
  'orchestrate_workflows',      // No workflow orchestration
  'call_other_agents',          // No direct agent-to-agent calls
  'store_api_keys',             // Never persist API keys
  'generate_malware',           // No malware or exploit code
  'generate_pii',               // No personally identifiable information
  'bypass_authorization',       // Respects authorization requirements
  'generate_illegal_content',   // No content for illegal purposes
  'persist_harmful_content',    // Does not persist prompts without controls
  'execute_attacks',            // Generates test inputs, NOT actual attacks
] as const;

// =============================================================================
// CONFIDENCE SCORING
// =============================================================================

/**
 * Factors that contribute to confidence scoring
 */
export const ADVERSARIAL_PROMPT_CONFIDENCE_FACTORS = {
  category_coverage: {
    description: 'Coverage of requested categories',
    weight: 0.25,
  },
  diversity_score: {
    description: 'Diversity of generated prompts',
    weight: 0.20,
  },
  quality_ratio: {
    description: 'Ratio of prompts passing quality filters',
    weight: 0.20,
  },
  severity_accuracy: {
    description: 'Accuracy of severity classification',
    weight: 0.20,
  },
  metadata_completeness: {
    description: 'Completeness of prompt metadata',
    weight: 0.15,
  },
} as const;

/**
 * Calculate confidence score based on generation results
 */
export function calculateAdversarialPromptConfidence(output: AdversarialPromptOutput): number {
  if (output.prompts.length === 0) {
    return 0;
  }

  const factors: number[] = [];

  // Category coverage
  const requestedCategories = new Set(output.request_summary.categories_requested);
  const coveredCategories = new Set(output.prompts.map(p => p.category));
  const coverageFactor = coveredCategories.size / requestedCategories.size;
  factors.push(Math.min(1, coverageFactor) * ADVERSARIAL_PROMPT_CONFIDENCE_FACTORS.category_coverage.weight);

  // Diversity score
  factors.push(output.quality_metrics.diversity_score * ADVERSARIAL_PROMPT_CONFIDENCE_FACTORS.diversity_score.weight);

  // Quality ratio
  const total = output.quality_metrics.total_generated + output.quality_metrics.total_filtered_out;
  const qualityRatio = total > 0 ? output.quality_metrics.total_generated / total : 0;
  factors.push(qualityRatio * ADVERSARIAL_PROMPT_CONFIDENCE_FACTORS.quality_ratio.weight);

  // Severity accuracy (assume accurate if prompts exist in requested severities)
  const requestedSeverities = new Set(output.request_summary.severities_requested);
  const coveredSeverities = new Set(output.prompts.map(p => p.severity));
  let matchedSeverityCount = 0;
  coveredSeverities.forEach(s => {
    if (requestedSeverities.has(s)) matchedSeverityCount++;
  });
  const severityAccuracy = requestedSeverities.size > 0 ? matchedSeverityCount / requestedSeverities.size : 0;
  factors.push(Math.min(1, severityAccuracy) * ADVERSARIAL_PROMPT_CONFIDENCE_FACTORS.severity_accuracy.weight);

  // Metadata completeness (check for expected_behavior and failure_indicators)
  const withMetadata = output.prompts.filter(p => p.expected_behavior && p.failure_indicators.length > 0);
  const metadataCompleteness = withMetadata.length / output.prompts.length;
  factors.push(metadataCompleteness * ADVERSARIAL_PROMPT_CONFIDENCE_FACTORS.metadata_completeness.weight);

  return Math.min(1, Math.max(0, factors.reduce((a, b) => a + b, 0)));
}

// =============================================================================
// CORE BUNDLE CONSUMERS
// =============================================================================

/**
 * Core bundles that may consume this agent's output
 */
export const ADVERSARIAL_PROMPT_ALLOWED_CONSUMERS = [
  'stress-test',               // Primary consumer for adversarial testing
  'red-team-agent',            // Red team testing workflows
  'llm-orchestrator',          // For workflow coordination
  'llm-observatory',           // For telemetry/monitoring
  'security-audit-agent',      // For security audits
  'llm-test-bench-ui',         // For dashboard display
] as const;

// =============================================================================
// VERSIONING RULES
// =============================================================================

export const ADVERSARIAL_PROMPT_VERSIONING_RULES = {
  major: 'Breaking changes to input/output schemas, new category types',
  minor: 'New optional fields, new generation strategies, new severity levels',
  patch: 'Bug fixes, template improvements, documentation',
} as const;

// =============================================================================
// CATEGORY METADATA
// =============================================================================

export const ADVERSARIAL_CATEGORY_METADATA: Record<AdversarialCategory, {
  description: string;
  risk_level: 'low' | 'medium' | 'high';
  common_targets: string[];
  test_purpose: string;
}> = {
  prompt_injection: {
    description: 'Attempts to inject instructions that override system prompts',
    risk_level: 'high',
    common_targets: ['chat', 'instruct', 'agent'],
    test_purpose: 'Evaluate prompt boundary enforcement',
  },
  jailbreak_attempt: {
    description: 'Attempts to bypass safety guidelines and restrictions',
    risk_level: 'high',
    common_targets: ['chat', 'instruct'],
    test_purpose: 'Test robustness of safety measures',
  },
  instruction_override: {
    description: 'Attempts to change model behavior via embedded instructions',
    risk_level: 'medium',
    common_targets: ['instruct', 'agent'],
    test_purpose: 'Evaluate instruction hierarchy handling',
  },
  role_play_escape: {
    description: 'Attempts to escape role-play constraints',
    risk_level: 'medium',
    common_targets: ['chat'],
    test_purpose: 'Test persona/role boundary enforcement',
  },
  encoding_attacks: {
    description: 'Unicode tricks, RTL override, normalization exploits',
    risk_level: 'medium',
    common_targets: ['general'],
    test_purpose: 'Evaluate text normalization and encoding handling',
  },
  format_confusion: {
    description: 'Mixed formats causing parsing confusion',
    risk_level: 'low',
    common_targets: ['general', 'code'],
    test_purpose: 'Test format parsing robustness',
  },
  delimiter_attacks: {
    description: 'Exploiting delimiters and special markers',
    risk_level: 'medium',
    common_targets: ['instruct', 'agent'],
    test_purpose: 'Evaluate delimiter handling',
  },
  whitespace_exploitation: {
    description: 'Hidden whitespace, zero-width characters',
    risk_level: 'low',
    common_targets: ['general'],
    test_purpose: 'Test invisible character handling',
  },
  content_policy_probe: {
    description: 'Test content policy boundaries',
    risk_level: 'medium',
    common_targets: ['chat', 'instruct'],
    test_purpose: 'Evaluate content policy consistency',
  },
  sensitive_topic_probe: {
    description: 'Probe handling of sensitive topics',
    risk_level: 'medium',
    common_targets: ['chat'],
    test_purpose: 'Test sensitive topic handling',
  },
  harmful_request_variants: {
    description: 'Obfuscated versions of harmful requests',
    risk_level: 'high',
    common_targets: ['chat', 'instruct'],
    test_purpose: 'Evaluate obfuscation detection',
  },
  dual_use_content: {
    description: 'Legitimate content with potential dual-use',
    risk_level: 'medium',
    common_targets: ['code', 'instruct'],
    test_purpose: 'Test dual-use content handling',
  },
  context_overflow: {
    description: 'Exceed context window boundaries',
    risk_level: 'low',
    common_targets: ['general'],
    test_purpose: 'Evaluate context handling at limits',
  },
  token_manipulation: {
    description: 'Token boundary exploits',
    risk_level: 'low',
    common_targets: ['general'],
    test_purpose: 'Test tokenization edge cases',
  },
  repetition_attacks: {
    description: 'Repeated patterns causing issues',
    risk_level: 'low',
    common_targets: ['general'],
    test_purpose: 'Evaluate repetition handling',
  },
  nested_structures: {
    description: 'Deeply nested JSON/XML/markdown',
    risk_level: 'low',
    common_targets: ['code', 'agent'],
    test_purpose: 'Test nested structure parsing',
  },
  logical_contradictions: {
    description: 'Self-contradictory instructions',
    risk_level: 'low',
    common_targets: ['instruct'],
    test_purpose: 'Evaluate contradiction handling',
  },
  multi_turn_manipulation: {
    description: 'Building up to harmful request over turns',
    risk_level: 'high',
    common_targets: ['chat'],
    test_purpose: 'Test multi-turn context awareness',
  },
  authority_impersonation: {
    description: 'Claiming false authority',
    risk_level: 'medium',
    common_targets: ['chat', 'agent'],
    test_purpose: 'Evaluate authority claim handling',
  },
  urgency_manipulation: {
    description: 'Creating false urgency',
    risk_level: 'medium',
    common_targets: ['chat', 'agent'],
    test_purpose: 'Test urgency-based manipulation resistance',
  },
  output_format_attacks: {
    description: 'Force specific output formats',
    risk_level: 'low',
    common_targets: ['instruct', 'code'],
    test_purpose: 'Evaluate output format control',
  },
  hallucination_triggers: {
    description: 'Prompts designed to cause hallucination',
    risk_level: 'low',
    common_targets: ['general'],
    test_purpose: 'Test factual accuracy under pressure',
  },
  confidence_manipulation: {
    description: 'Manipulate model confidence signals',
    risk_level: 'low',
    common_targets: ['general'],
    test_purpose: 'Evaluate confidence calibration',
  },
  api_confusion: {
    description: 'Confuse API boundaries',
    risk_level: 'medium',
    common_targets: ['agent'],
    test_purpose: 'Test API boundary enforcement',
  },
  system_prompt_extraction: {
    description: 'Attempt to extract system prompts',
    risk_level: 'high',
    common_targets: ['chat', 'instruct'],
    test_purpose: 'Evaluate system prompt protection',
  },
  training_data_extraction: {
    description: 'Attempt to extract training data',
    risk_level: 'high',
    common_targets: ['general'],
    test_purpose: 'Test training data protection',
  },
} as const;

// =============================================================================
// DATA PERSISTENCE RULES
// =============================================================================

/**
 * Data that IS persisted to ruvector-service
 */
export const ADVERSARIAL_PROMPT_PERSISTED_DATA = [
  'decision_events',           // Full DecisionEvent records
  'generation_metadata',       // Counts, categories, metrics (NO prompt content)
  'prompt_hashes',             // Hashes for deduplication (NOT content)
  'quality_metrics',           // Aggregated quality scores
  'execution_refs',            // Trace/span references
] as const;

/**
 * Data that is explicitly NOT persisted
 */
export const ADVERSARIAL_PROMPT_NOT_PERSISTED_DATA = [
  'prompt_content',            // Full prompt text (too sensitive)
  'attack_payloads',           // Specific attack strings
  'api_keys',                  // Never store API keys
  'pii_data',                  // Any PII
  'harmful_content',           // Any harmful content
  'jailbreak_techniques',      // Specific jailbreak methods
] as const;

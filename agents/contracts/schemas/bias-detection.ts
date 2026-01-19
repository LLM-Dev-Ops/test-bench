/**
 * Bias Detection Agent Contract Schemas
 *
 * AGENT PURPOSE:
 * Detect demographic, cultural, or systemic bias in model outputs.
 * Identifies gender bias, racial bias, cultural bias, socioeconomic bias,
 * age bias, disability bias, religious bias, and other forms of systematic unfairness.
 *
 * This agent:
 * - Detects bias in text outputs (YES)
 * - Classifies bias types and severity (YES)
 * - Provides confidence-scored assessments (YES)
 * - Does NOT modify or debias content (NO)
 * - Does NOT orchestrate workflows (NO)
 * - Does NOT call other agents (NO)
 * - Does NOT enforce policies (NO - that's LLM-Policy-Engine)
 *
 * decision_type: "bias_detection"
 */

import { z } from 'zod';
import { DecisionEventSchema, ExecutionRefSchema } from './base';

// =============================================================================
// AGENT METADATA
// =============================================================================

export const BIAS_DETECTION_AGENT = {
  agent_id: 'bias-detection',
  agent_version: '1.0.0',
  decision_type: 'bias_detection',
} as const;

// =============================================================================
// BIAS TYPE DEFINITIONS
// =============================================================================

/**
 * Categories of bias that can be detected
 */
export const BiasTypeSchema = z.enum([
  'gender',           // Gender-based bias (male/female/non-binary)
  'racial',           // Race or ethnicity-based bias
  'cultural',         // Cultural or national origin bias
  'socioeconomic',    // Class or economic status bias
  'age',              // Age-based bias (ageism)
  'disability',       // Disability-related bias (ableism)
  'religious',        // Religious or belief-based bias
  'political',        // Political affiliation bias
  'sexual_orientation', // LGBTQ+ related bias
  'geographic',       // Regional or geographic bias
  'linguistic',       // Language or accent-based bias
  'educational',      // Educational background bias
  'appearance',       // Physical appearance bias
  'intersectional',   // Multiple overlapping bias categories
  'other',            // Other unclassified bias
]);

export type BiasType = z.infer<typeof BiasTypeSchema>;

/**
 * Severity levels for detected bias
 */
export const BiasSeveritySchema = z.enum([
  'negligible',   // Minor or borderline, may be acceptable in context
  'low',          // Subtle bias, potentially problematic
  'medium',       // Clear bias that should be addressed
  'high',         // Significant bias requiring immediate attention
  'critical',     // Severe bias that is harmful or offensive
]);

export type BiasSeverity = z.infer<typeof BiasSeveritySchema>;

/**
 * Direction of the bias
 */
export const BiasDirectionSchema = z.enum([
  'positive',     // Favorable bias toward a group
  'negative',     // Unfavorable bias against a group
  'comparative',  // Relative comparison between groups
  'neutral',      // Bias present but no clear direction
]);

export type BiasDirection = z.infer<typeof BiasDirectionSchema>;

// =============================================================================
// INPUT SCHEMAS
// =============================================================================

/**
 * A single text sample to analyze for bias
 */
export const TextSampleSchema = z.object({
  /** Unique identifier for the sample */
  sample_id: z.string().min(1),

  /** The text content to analyze */
  content: z.string().min(1).max(50000),

  /** Optional: source of the text (e.g., model name, document ID) */
  source: z.string().optional(),

  /** Optional: context about the text generation */
  context: z.string().optional(),

  /** Optional: metadata */
  metadata: z.record(z.unknown()).optional(),
});

export type TextSample = z.infer<typeof TextSampleSchema>;

/**
 * Demographic context for bias analysis
 */
export const DemographicContextSchema = z.object({
  /** Groups to specifically check for bias regarding */
  focus_groups: z.array(z.string()).optional(),

  /** Cultural context for the analysis */
  cultural_context: z.enum([
    'us_english',
    'uk_english',
    'global',
    'specific',
  ]).default('global'),

  /** Specific region if cultural_context is 'specific' */
  region: z.string().optional(),

  /** Domain context affects what's considered biased */
  domain: z.enum([
    'general',
    'healthcare',
    'legal',
    'education',
    'employment',
    'finance',
    'media',
    'technology',
    'government',
    'other',
  ]).default('general'),
});

export type DemographicContext = z.infer<typeof DemographicContextSchema>;

/**
 * Detection configuration options
 */
export const BiasDetectionConfigSchema = z.object({
  /** Minimum confidence threshold for reporting (0-1) */
  confidence_threshold: z.number().min(0).max(1).default(0.5),

  /** Minimum severity to report */
  min_severity: BiasSeveritySchema.default('low'),

  /** Bias types to check for (empty = all) */
  bias_types: z.array(BiasTypeSchema).optional(),

  /** Enable sentiment analysis for bias direction */
  enable_sentiment_analysis: z.boolean().default(true),

  /** Enable entity extraction for demographic identification */
  enable_entity_extraction: z.boolean().default(true),

  /** Enable stereotype pattern matching */
  enable_stereotype_detection: z.boolean().default(true),

  /** Enable representation disparity analysis */
  enable_representation_analysis: z.boolean().default(true),

  /** Enable language pattern analysis (pronouns, terms) */
  enable_language_pattern_analysis: z.boolean().default(true),

  /** Case sensitivity for text matching */
  case_sensitive: z.boolean().default(false),

  /** Maximum samples to process */
  max_samples: z.number().int().positive().default(100),

  /** Timeout for analysis in milliseconds */
  timeout_ms: z.number().int().positive().default(60000),

  /** Include detailed explanations in output */
  include_explanations: z.boolean().default(true),

  /** Include recommendations for remediation */
  include_recommendations: z.boolean().default(true),
});

export type BiasDetectionConfig = z.infer<typeof BiasDetectionConfigSchema>;

/**
 * Main input schema for Bias Detection Agent
 */
export const BiasDetectionInputSchema = z.object({
  /** Text samples to analyze for bias */
  samples: z.array(TextSampleSchema).min(1).max(1000),

  /** Optional: demographic context for analysis */
  demographic_context: DemographicContextSchema.optional(),

  /** Detection configuration */
  detection_config: BiasDetectionConfigSchema.optional(),

  /** Optional: caller context */
  caller_id: z.string().optional(),

  /** Optional: correlation ID for tracing */
  correlation_id: z.string().uuid().optional(),
});

export type BiasDetectionInput = z.infer<typeof BiasDetectionInputSchema>;

// =============================================================================
// OUTPUT SCHEMAS
// =============================================================================

/**
 * Evidence supporting bias detection
 */
export const BiasEvidenceSchema = z.object({
  /** The specific text span containing bias */
  text_span: z.string(),

  /** Start position in the original text */
  start_offset: z.number().nonnegative().optional(),

  /** End position in the original text */
  end_offset: z.number().nonnegative().optional(),

  /** Detection method that identified this evidence */
  detection_method: z.enum([
    'keyword_match',
    'stereotype_pattern',
    'sentiment_disparity',
    'representation_gap',
    'language_pattern',
    'entity_association',
    'comparative_framing',
    'implicit_association',
  ]),

  /** Relevance score for this evidence (0-1) */
  relevance_score: z.number().min(0).max(1),

  /** Explanation of why this is evidence of bias */
  explanation: z.string().optional(),
});

export type BiasEvidence = z.infer<typeof BiasEvidenceSchema>;

/**
 * A single detected bias instance
 */
export const DetectedBiasSchema = z.object({
  /** Unique identifier for this bias instance */
  bias_id: z.string().uuid(),

  /** Type of bias detected */
  bias_type: BiasTypeSchema,

  /** Severity of the bias */
  severity: BiasSeveritySchema,

  /** Direction of the bias */
  direction: BiasDirectionSchema,

  /** Confidence in the detection (0-1) */
  confidence: z.number().min(0).max(1),

  /** Affected group(s) */
  affected_groups: z.array(z.string()).min(1),

  /** Advantaged group(s), if comparative */
  advantaged_groups: z.array(z.string()).optional(),

  /** Evidence supporting this detection */
  evidence: z.array(BiasEvidenceSchema).min(1),

  /** Human-readable explanation */
  explanation: z.string(),

  /** Potential impact of this bias */
  potential_impact: z.string().optional(),

  /** Recommended remediation */
  recommendation: z.string().optional(),
});

export type DetectedBias = z.infer<typeof DetectedBiasSchema>;

/**
 * Result for a single sample analysis
 */
export const BiasSampleResultSchema = z.object({
  /** Sample identifier */
  sample_id: z.string(),

  /** Whether any bias was detected */
  has_bias: z.boolean(),

  /** Overall bias score for this sample (0-1, 0 = no bias) */
  bias_score: z.number().min(0).max(1),

  /** Maximum severity of detected biases */
  max_severity: BiasSeveritySchema.nullable(),

  /** List of detected biases */
  detected_biases: z.array(DetectedBiasSchema),

  /** Summary of bias types found */
  bias_types_found: z.array(BiasTypeSchema),

  /** Overall assessment */
  assessment: z.enum([
    'no_bias_detected',
    'minimal_bias',
    'moderate_bias',
    'significant_bias',
    'severe_bias',
  ]),

  /** Analysis timestamp */
  analyzed_at: z.string().datetime(),

  /** Processing duration for this sample */
  processing_ms: z.number().nonnegative(),
});

export type BiasSampleResult = z.infer<typeof BiasSampleResultSchema>;

/**
 * Aggregated statistics for the detection run
 */
export const BiasDetectionStatsSchema = z.object({
  /** Total samples analyzed */
  total_samples: z.number().int().nonnegative(),

  /** Samples with bias detected */
  samples_with_bias: z.number().int().nonnegative(),

  /** Samples without detected bias */
  samples_without_bias: z.number().int().nonnegative(),

  /** Total number of bias instances detected */
  total_biases_detected: z.number().int().nonnegative(),

  /** Bias rate (samples with bias / total) */
  bias_rate: z.number().min(0).max(1),

  /** Average bias score across samples */
  avg_bias_score: z.number().min(0).max(1),

  /** Average confidence across detections */
  avg_confidence: z.number().min(0).max(1),

  /** Breakdown by bias type */
  by_type: z.record(BiasTypeSchema, z.number().int().nonnegative()),

  /** Breakdown by severity */
  by_severity: z.object({
    negligible: z.number().int().nonnegative(),
    low: z.number().int().nonnegative(),
    medium: z.number().int().nonnegative(),
    high: z.number().int().nonnegative(),
    critical: z.number().int().nonnegative(),
  }),

  /** Most frequently affected groups */
  top_affected_groups: z.array(z.object({
    group: z.string(),
    count: z.number().int().nonnegative(),
  })).optional(),
});

export type BiasDetectionStats = z.infer<typeof BiasDetectionStatsSchema>;

/**
 * Main output schema for Bias Detection Agent
 */
export const BiasDetectionOutputSchema = z.object({
  /** Unique detection run identifier */
  detection_id: z.string().uuid(),

  /** Individual sample results */
  results: z.array(BiasSampleResultSchema),

  /** Aggregated statistics */
  stats: BiasDetectionStatsSchema,

  /** Detection configuration used */
  config_used: BiasDetectionConfigSchema,

  /** Demographic context applied */
  demographic_context_applied: DemographicContextSchema.optional(),

  /** Overall assessment across all samples */
  overall_assessment: z.enum([
    'no_significant_bias',
    'minimal_bias_detected',
    'moderate_bias_detected',
    'significant_bias_detected',
    'severe_bias_detected',
  ]),

  /** Key findings summary */
  key_findings: z.array(z.string()).optional(),

  /** Timing */
  started_at: z.string().datetime(),
  completed_at: z.string().datetime(),
  duration_ms: z.number().nonnegative(),
});

export type BiasDetectionOutput = z.infer<typeof BiasDetectionOutputSchema>;

// =============================================================================
// DECISION EVENT SCHEMA (SPECIALIZED)
// =============================================================================

/**
 * Bias Detection Decision Event
 * Extends base DecisionEvent with bias-detection-specific outputs
 */
export const BiasDetectionDecisionEventSchema = DecisionEventSchema.extend({
  decision_type: z.literal('bias_detection'),
  outputs: BiasDetectionOutputSchema,
});

export type BiasDetectionDecisionEvent = z.infer<typeof BiasDetectionDecisionEventSchema>;

// =============================================================================
// CLI CONTRACT
// =============================================================================

/**
 * CLI invocation shape for Bias Detection Agent
 */
export const BiasDetectionCLIArgsSchema = z.object({
  // Input source (one required)
  input_file: z.string().optional(),
  input_json: z.string().optional(),
  input_stdin: z.boolean().optional(),
  input_text: z.string().optional(),

  // Configuration
  bias_types: z.string().optional(), // comma-separated
  min_severity: BiasSeveritySchema.optional(),
  confidence_threshold: z.number().min(0).max(1).optional(),
  domain: z.string().optional(),
  cultural_context: z.string().optional(),

  // Output format
  output_format: z.enum(['json', 'csv', 'table', 'summary', 'report']).default('json'),
  output_file: z.string().optional(),

  // Verbosity
  verbose: z.boolean().default(false),
  quiet: z.boolean().default(false),

  // Execution modifiers
  dry_run: z.boolean().default(false),
});

export type BiasDetectionCLIArgs = z.infer<typeof BiasDetectionCLIArgsSchema>;

// =============================================================================
// CONSTRAINTS & NON-RESPONSIBILITIES
// =============================================================================

/**
 * Constraints that MAY be applied during execution
 */
export const BIAS_DETECTION_VALID_CONSTRAINTS = [
  'max_samples_exceeded',
  'timeout_exceeded',
  'content_too_long',
  'confidence_below_threshold',
  'rate_limit_applied',
  'entity_extraction_unavailable',
  'sentiment_analysis_unavailable',
  'stereotype_database_unavailable',
  'partial_analysis_only',
  'language_not_supported',
] as const;

export type BiasDetectionConstraint = typeof BIAS_DETECTION_VALID_CONSTRAINTS[number];

/**
 * Explicit non-responsibilities - this agent MUST NOT:
 */
export const BIAS_DETECTION_NON_RESPONSIBILITIES = [
  'debias_content',              // No content modification
  'generate_content',            // No content generation
  'rewrite_text',                // No text rewriting
  'orchestrate_workflows',       // No workflow orchestration
  'call_other_agents',           // No direct agent-to-agent calls
  'enforce_policies',            // No policy enforcement (LLM-Policy-Engine does this)
  'make_moral_judgments',        // Detection only, not judgment
  'recommend_hiring_decisions',  // No employment decisions
  'execute_arbitrary_code',      // No code execution
  'bypass_schemas',              // Must validate all I/O
  'access_external_apis',        // No external API calls beyond configured
  'persist_analyzed_content',    // No storing of input content
  'modify_input_text',           // Input text is read-only
  'provide_legal_advice',        // Not qualified for legal determinations
] as const;

// =============================================================================
// CONFIDENCE SCORING
// =============================================================================

/**
 * Factors that contribute to confidence scoring
 */
export const BIAS_DETECTION_CONFIDENCE_FACTORS = {
  pattern_strength: {
    description: 'Strength of matched bias patterns',
    weight: 0.25,
  },
  evidence_quantity: {
    description: 'Number of independent evidence pieces',
    weight: 0.20,
  },
  context_relevance: {
    description: 'Relevance to demographic context',
    weight: 0.15,
  },
  method_agreement: {
    description: 'Agreement between multiple detection methods',
    weight: 0.20,
  },
  stereotype_database_match: {
    description: 'Match against known stereotype patterns',
    weight: 0.10,
  },
  linguistic_signal_strength: {
    description: 'Clarity of linguistic bias indicators',
    weight: 0.10,
  },
} as const;

/**
 * Calculate confidence score based on detection factors
 */
export function calculateBiasConfidence(
  evidence: BiasEvidence[],
  methodsAgreed: number,
  totalMethods: number,
  stereotypeMatch: boolean
): number {
  if (evidence.length === 0) return 0;

  // Pattern strength from evidence
  const avgRelevance = evidence.reduce((sum, e) => sum + e.relevance_score, 0) / evidence.length;
  const patternStrength = avgRelevance * BIAS_DETECTION_CONFIDENCE_FACTORS.pattern_strength.weight;

  // Evidence quantity (logarithmic scale, capped)
  const evidenceScore = Math.min(1, Math.log10(evidence.length + 1) / 1.5);
  const evidenceQuantity = evidenceScore * BIAS_DETECTION_CONFIDENCE_FACTORS.evidence_quantity.weight;

  // Context relevance (placeholder - would be calculated based on demographic match)
  const contextRelevance = 0.7 * BIAS_DETECTION_CONFIDENCE_FACTORS.context_relevance.weight;

  // Method agreement
  const agreementRatio = totalMethods > 0 ? methodsAgreed / totalMethods : 0.5;
  const methodAgreement = agreementRatio * BIAS_DETECTION_CONFIDENCE_FACTORS.method_agreement.weight;

  // Stereotype match bonus
  const stereotypeBonus = stereotypeMatch
    ? 0.9 * BIAS_DETECTION_CONFIDENCE_FACTORS.stereotype_database_match.weight
    : 0.3 * BIAS_DETECTION_CONFIDENCE_FACTORS.stereotype_database_match.weight;

  // Linguistic signal (based on detection methods used)
  const linguisticMethods = evidence.filter(e =>
    ['keyword_match', 'language_pattern', 'implicit_association'].includes(e.detection_method)
  );
  const linguisticScore = linguisticMethods.length > 0
    ? 0.8 * BIAS_DETECTION_CONFIDENCE_FACTORS.linguistic_signal_strength.weight
    : 0.4 * BIAS_DETECTION_CONFIDENCE_FACTORS.linguistic_signal_strength.weight;

  const total = patternStrength + evidenceQuantity + contextRelevance +
    methodAgreement + stereotypeBonus + linguisticScore;

  return Math.min(1, Math.max(0, total));
}

// =============================================================================
// CORE BUNDLE CONSUMERS
// =============================================================================

/**
 * Core bundles that may consume this agent's output
 */
export const BIAS_DETECTION_ALLOWED_CONSUMERS = [
  'llm-orchestrator',           // For workflow coordination
  'llm-observatory',            // For telemetry/monitoring
  'llm-analytics',              // For aggregation/analysis
  'llm-test-bench-ui',          // For dashboard display
  'llm-policy-engine',          // For policy evaluation
  'fairness-auditor-agent',     // For comprehensive fairness audits
  'quality-scorer-agent',       // For quality assessment integration
  'content-moderator-agent',    // For moderation pipelines
  'model-comparator-agent',     // For model comparison on fairness
] as const;

// =============================================================================
// VERSIONING RULES
// =============================================================================

export const BIAS_DETECTION_VERSIONING_RULES = {
  major: 'Breaking changes to input/output schemas, bias type taxonomy, or detection methodology',
  minor: 'New bias types, new detection methods, optional fields, expanded demographic contexts',
  patch: 'Bug fixes, accuracy improvements, documentation updates, stereotype database updates',
} as const;

// =============================================================================
// FAILURE MODES
// =============================================================================

/**
 * Defined failure modes for the Bias Detection Agent
 */
export const BIAS_DETECTION_FAILURE_MODES = {
  VALIDATION_FAILURE: {
    code: 'VALIDATION_ERROR',
    description: 'Input failed schema validation',
    recoverable: true,
    resolution: 'Check input against BiasDetectionInputSchema',
  },
  TIMEOUT_EXCEEDED: {
    code: 'TIMEOUT_ERROR',
    description: 'Analysis exceeded configured timeout',
    recoverable: true,
    resolution: 'Reduce sample count or increase timeout_ms',
  },
  CONTENT_TOO_LONG: {
    code: 'CONTENT_TOO_LONG',
    description: 'Individual sample content exceeds maximum length',
    recoverable: true,
    resolution: 'Split content into smaller samples',
  },
  LANGUAGE_NOT_SUPPORTED: {
    code: 'LANGUAGE_NOT_SUPPORTED',
    description: 'Content language not supported for bias detection',
    recoverable: false,
    resolution: 'Translate content to supported language',
  },
  ANALYSIS_PARTIAL: {
    code: 'PARTIAL_ANALYSIS',
    description: 'Some detection methods unavailable, partial results returned',
    recoverable: true,
    resolution: 'Review constraints_applied for missing methods',
  },
  PERSISTENCE_FAILURE: {
    code: 'PERSISTENCE_ERROR',
    description: 'Failed to persist DecisionEvent to ruvector-service',
    recoverable: true,
    resolution: 'DecisionEvent will be retried asynchronously',
  },
} as const;

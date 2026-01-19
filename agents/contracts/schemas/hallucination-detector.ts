/**
 * Hallucination Detector Agent Contract Schemas
 *
 * AGENT PURPOSE:
 * Detect unsupported or fabricated claims relative to provided reference context.
 * Identifies fabrication, exaggeration, misattribution, contradiction, and unsupported claims.
 *
 * This agent:
 * - Detects hallucinations in claims against reference context
 * - Does NOT generate or fix content
 * - Does NOT orchestrate workflows
 * - Does NOT call other agents
 *
 * decision_type: "hallucination_detection"
 */

import { z } from 'zod';
import { AgentIdentifierSchema, ExecutionRefSchema, DecisionEventSchema } from './base';

// =============================================================================
// AGENT METADATA
// =============================================================================

export const HALLUCINATION_DETECTOR_AGENT = {
  agent_id: 'hallucination-detector',
  agent_version: '1.0.0',
  decision_type: 'hallucination_detection',
} as const;

// =============================================================================
// INPUT SCHEMAS
// =============================================================================

/**
 * Reference source for context grounding
 */
export const ReferenceSourceSchema = z.object({
  source_id: z.string().min(1),
  content: z.string().min(1),
  source_type: z.enum([
    'document',
    'webpage',
    'database',
    'api_response',
    'knowledge_base',
    'transcript',
    'other',
  ]).optional(),
  metadata: z.record(z.unknown()).optional(),
});

export type ReferenceSource = z.infer<typeof ReferenceSourceSchema>;

/**
 * Detection configuration options
 */
export const DetectionConfigSchema = z.object({
  // Sensitivity controls
  sensitivity: z.enum(['low', 'medium', 'high']).default('medium'),
  confidence_threshold: z.number().min(0).max(1).default(0.7),

  // Detection methods to use
  methods: z.array(z.enum([
    'semantic_similarity',
    'entailment_analysis',
    'fact_extraction',
    'entity_verification',
    'temporal_consistency',
    'logical_consistency',
  ])).default(['semantic_similarity', 'entailment_analysis']),

  // Hallucination types to detect
  detect_types: z.array(z.enum([
    'fabrication',
    'exaggeration',
    'misattribution',
    'contradiction',
    'unsupported',
  ])).default(['fabrication', 'exaggeration', 'misattribution', 'contradiction', 'unsupported']),

  // Processing options
  max_claim_length: z.number().positive().default(2000),
  max_reference_length: z.number().positive().default(50000),
  chunk_overlap: z.number().nonnegative().default(200),
  embedding_model: z.string().optional(),
});

export type DetectionConfig = z.infer<typeof DetectionConfigSchema>;

/**
 * Main input schema for Hallucination Detector Agent
 */
export const HallucinationDetectorInputSchema = z.object({
  // Claims to check (string or array)
  claim: z.string().min(1).optional(),
  claims: z.array(z.object({
    claim_id: z.string().min(1),
    text: z.string().min(1),
    metadata: z.record(z.unknown()).optional(),
  })).optional(),

  // Reference context (string or array of sources)
  reference_context: z.union([
    z.string().min(1),
    z.array(ReferenceSourceSchema).min(1),
  ]),

  // Optional: detection configuration
  detection_config: DetectionConfigSchema.optional(),

  // Optional: caller context
  caller_id: z.string().optional(),
  correlation_id: z.string().uuid().optional(),
}).refine(
  (data) => data.claim !== undefined || (data.claims !== undefined && data.claims.length > 0),
  { message: 'Either "claim" or "claims" must be provided' }
);

export type HallucinationDetectorInput = z.infer<typeof HallucinationDetectorInputSchema>;

// =============================================================================
// OUTPUT SCHEMAS
// =============================================================================

/**
 * Hallucination type classification
 */
export const HallucinationTypeSchema = z.enum([
  'fabrication',      // Completely made up information
  'exaggeration',     // Overstated or inflated claims
  'misattribution',   // Incorrectly attributed to wrong source
  'contradiction',    // Directly contradicts reference material
  'unsupported',      // No evidence in reference to support claim
  'none',             // Claim is verified/supported
]);

export type HallucinationType = z.infer<typeof HallucinationTypeSchema>;

/**
 * Evidence reference linking claim to source material
 */
export const EvidenceReferenceSchema = z.object({
  source_id: z.string(),
  excerpt: z.string(),
  relevance_score: z.number().min(0).max(1),
  position: z.object({
    start: z.number().nonnegative().optional(),
    end: z.number().nonnegative().optional(),
  }).optional(),
});

export type EvidenceReference = z.infer<typeof EvidenceReferenceSchema>;

/**
 * Per-claim hallucination detection result
 */
export const HallucinationClaimResultSchema = z.object({
  // Claim identification
  claim_id: z.string(),
  claim_text: z.string(),

  // Detection result
  is_hallucination: z.boolean(),
  hallucination_type: HallucinationTypeSchema,

  // Confidence in the detection
  confidence: z.number().min(0).max(1),

  // Evidence analysis
  supporting_evidence: z.array(EvidenceReferenceSchema),
  contradicting_evidence: z.array(EvidenceReferenceSchema),

  // Explanation
  explanation: z.string(),

  // Detection method results
  method_scores: z.record(z.number().min(0).max(1)).optional(),

  // Additional metadata
  metadata: z.record(z.unknown()).optional(),
});

export type HallucinationClaimResult = z.infer<typeof HallucinationClaimResultSchema>;

/**
 * Summary statistics for the detection run
 */
export const DetectionSummarySchema = z.object({
  by_type: z.object({
    fabrication: z.number().nonnegative(),
    exaggeration: z.number().nonnegative(),
    misattribution: z.number().nonnegative(),
    contradiction: z.number().nonnegative(),
    unsupported: z.number().nonnegative(),
  }),
  by_confidence: z.object({
    high: z.number().nonnegative(),    // >= 0.8
    medium: z.number().nonnegative(),  // >= 0.5 && < 0.8
    low: z.number().nonnegative(),     // < 0.5
  }),
});

export type DetectionSummary = z.infer<typeof DetectionSummarySchema>;

/**
 * Main output schema for Hallucination Detector Agent
 */
export const HallucinationDetectorOutputSchema = z.object({
  // Execution identity
  execution_id: z.string().uuid(),

  // Summary counts
  total_claims: z.number().nonnegative(),
  hallucinated_claims: z.number().nonnegative(),
  verified_claims: z.number().nonnegative(),

  // Overall hallucination rate
  overall_hallucination_rate: z.number().min(0).max(1),

  // Detailed results per claim
  results: z.array(HallucinationClaimResultSchema),

  // Breakdown summary
  summary: DetectionSummarySchema.optional(),

  // Detection configuration used
  detection_config: DetectionConfigSchema,

  // Reference context stats
  reference_stats: z.object({
    total_sources: z.number().nonnegative(),
    total_characters: z.number().nonnegative(),
    sources_used: z.array(z.string()),
  }).optional(),

  // Timing
  started_at: z.string().datetime(),
  completed_at: z.string().datetime(),
  total_duration_ms: z.number().nonnegative(),
});

export type HallucinationDetectorOutput = z.infer<typeof HallucinationDetectorOutputSchema>;

// =============================================================================
// DECISION EVENT SCHEMA (SPECIALIZED)
// =============================================================================

/**
 * Hallucination Detector Decision Event
 * Extends base DecisionEvent with hallucination-detection-specific outputs
 */
export const HallucinationDetectorDecisionEventSchema = DecisionEventSchema.extend({
  decision_type: z.literal('hallucination_detection'),
  outputs: HallucinationDetectorOutputSchema,
});

export type HallucinationDetectorDecisionEvent = z.infer<typeof HallucinationDetectorDecisionEventSchema>;

// =============================================================================
// CLI CONTRACT
// =============================================================================

/**
 * CLI invocation shape for Hallucination Detector Agent
 */
export const HallucinationDetectorCLIArgsSchema = z.object({
  // Input source (one required)
  input_file: z.string().optional(),
  input_json: z.string().optional(),
  input_stdin: z.boolean().optional(),

  // Reference source (alternative to embedding in input)
  reference_file: z.string().optional(),
  reference_url: z.string().url().optional(),

  // Detection options
  sensitivity: z.enum(['low', 'medium', 'high']).optional(),
  confidence_threshold: z.number().min(0).max(1).optional(),

  // Output format
  output_format: z.enum(['json', 'csv', 'table', 'summary']).default('json'),
  output_file: z.string().optional(),

  // Verbosity
  verbose: z.boolean().default(false),
  quiet: z.boolean().default(false),

  // Execution modifiers
  dry_run: z.boolean().default(false),
});

export type HallucinationDetectorCLIArgs = z.infer<typeof HallucinationDetectorCLIArgsSchema>;

// =============================================================================
// CONSTRAINTS & NON-RESPONSIBILITIES
// =============================================================================

/**
 * Constraints that MAY be applied during execution
 */
export const HALLUCINATION_VALID_CONSTRAINTS = [
  'max_claims_exceeded',
  'reference_too_large',
  'timeout_exceeded',
  'confidence_below_threshold',
  'rate_limit_applied',
  'embedding_unavailable',
  'context_window_exceeded',
  'claim_too_long',
  'reference_truncated',
] as const;

export type HallucinationConstraint = typeof HALLUCINATION_VALID_CONSTRAINTS[number];

/**
 * Explicit non-responsibilities - this agent MUST NOT:
 */
export const HALLUCINATION_NON_RESPONSIBILITIES = [
  'generate_content',           // No content generation
  'fix_hallucinations',         // No hallucination correction
  'rewrite_claims',             // No claim rewriting
  'orchestrate_workflows',      // No workflow orchestration
  'call_other_agents',          // No direct agent-to-agent calls
  'rank_models',                // No model ranking
  'compare_outputs',            // No output comparison
  'execute_arbitrary_code',     // No code execution
  'bypass_schemas',             // Must validate all I/O
  'access_external_apis',       // No external API calls beyond configured
  'persist_reference_material', // No storing of reference context
  'modify_reference_context',   // Reference context is read-only
] as const;

// =============================================================================
// CONFIDENCE SCORING
// =============================================================================

/**
 * Factors that contribute to confidence scoring
 */
export const HALLUCINATION_CONFIDENCE_FACTORS = {
  semantic_similarity: {
    description: 'Cosine similarity between claim and reference embeddings',
    weight: 0.25,
  },
  entailment_score: {
    description: 'NLI model entailment probability',
    weight: 0.30,
  },
  evidence_coverage: {
    description: 'Proportion of claim entities found in reference',
    weight: 0.20,
  },
  reference_quality: {
    description: 'Quality and relevance of reference material',
    weight: 0.15,
  },
  method_agreement: {
    description: 'Agreement between multiple detection methods',
    weight: 0.10,
  },
} as const;

/**
 * Calculate confidence score based on detection results
 */
export function calculateHallucinationConfidence(result: HallucinationClaimResult): number {
  const methodScores = result.method_scores || {};

  // Get scores from available methods
  const semanticScore = methodScores['semantic_similarity'] ?? 0.5;
  const entailmentScore = methodScores['entailment_analysis'] ?? 0.5;
  const entityScore = methodScores['entity_verification'] ?? 0.5;

  // Calculate evidence coverage
  const totalEvidence = result.supporting_evidence.length + result.contradicting_evidence.length;
  const evidenceCoverage = totalEvidence > 0
    ? result.supporting_evidence.reduce((sum, e) => sum + e.relevance_score, 0) / totalEvidence
    : 0.5;

  // Calculate method agreement
  const scores = Object.values(methodScores);
  const meanScore = scores.length > 0
    ? scores.reduce((a, b) => a + b, 0) / scores.length
    : 0.5;
  const variance = scores.length > 0
    ? scores.reduce((sum, s) => sum + Math.pow(s - meanScore, 2), 0) / scores.length
    : 0.25;
  const methodAgreement = Math.max(0, 1 - Math.sqrt(variance));

  // Weighted combination
  const factors = [
    semanticScore * HALLUCINATION_CONFIDENCE_FACTORS.semantic_similarity.weight,
    entailmentScore * HALLUCINATION_CONFIDENCE_FACTORS.entailment_score.weight,
    evidenceCoverage * HALLUCINATION_CONFIDENCE_FACTORS.evidence_coverage.weight,
    0.8 * HALLUCINATION_CONFIDENCE_FACTORS.reference_quality.weight, // Placeholder for reference quality
    methodAgreement * HALLUCINATION_CONFIDENCE_FACTORS.method_agreement.weight,
  ];

  return Math.min(1, Math.max(0, factors.reduce((a, b) => a + b, 0)));
}

// =============================================================================
// CORE BUNDLE CONSUMERS
// =============================================================================

/**
 * Core bundles that may consume this agent's output
 */
export const HALLUCINATION_ALLOWED_CONSUMERS = [
  'llm-orchestrator',         // For workflow coordination
  'llm-observatory',          // For telemetry/monitoring
  'llm-analytics',            // For aggregation/analysis
  'llm-test-bench-ui',        // For dashboard display
  'fact-checker-agent',       // For downstream fact verification
  'content-validator-agent',  // For content validation pipelines
  'quality-scorer-agent',     // For quality assessment
] as const;

// =============================================================================
// VERSIONING RULES
// =============================================================================

export const HALLUCINATION_VERSIONING_RULES = {
  major: 'Breaking changes to input/output schemas or detection methodology',
  minor: 'New detection methods, new hallucination types, optional fields',
  patch: 'Bug fixes, accuracy improvements, documentation updates',
} as const;

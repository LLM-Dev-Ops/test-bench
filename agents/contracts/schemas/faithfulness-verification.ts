/**
 * Faithfulness Verification Agent Contract Schemas
 *
 * AGENT PURPOSE:
 * Measure alignment between model output and supplied source documents.
 * The agent evaluates whether the generated content faithfully represents
 * the information in the source documents without hallucination, distortion,
 * or unsupported claims.
 *
 * This agent:
 * - Verifies faithfulness of model outputs against sources
 * - Identifies hallucinations and unsupported claims
 * - Calculates faithfulness scores with evidence
 * - Does NOT generate new content
 * - Does NOT compare models
 * - Does NOT orchestrate workflows
 *
 * decision_type: "faithfulness_verification"
 */

import { z } from 'zod';
import { DecisionEventSchema, ExecutionRefSchema } from './base';

// =============================================================================
// AGENT METADATA
// =============================================================================

export const FAITHFULNESS_VERIFICATION_AGENT = {
  agent_id: 'faithfulness-verification',
  agent_version: '1.0.0',
  decision_type: 'faithfulness_verification',
} as const;

// =============================================================================
// INPUT SCHEMAS
// =============================================================================

/**
 * Source document for faithfulness verification
 */
export const SourceDocumentSchema = z.object({
  document_id: z.string().min(1),
  content: z.string().min(1),
  source_type: z.enum([
    'context',        // Retrieval-augmented context
    'knowledge_base', // Knowledge base document
    'reference',      // Reference material
    'ground_truth',   // Ground truth answer
    'transcript',     // Transcript or record
    'document',       // General document
  ]).default('document'),
  metadata: z.record(z.unknown()).optional(),
});

export type SourceDocument = z.infer<typeof SourceDocumentSchema>;

/**
 * Model output to verify for faithfulness
 */
export const VerificationModelOutputSchema = z.object({
  output_id: z.string().min(1),
  content: z.string().min(1),
  model_id: z.string().optional(),
  provider_name: z.string().optional(),
  prompt: z.string().optional(),
  metadata: z.record(z.unknown()).optional(),
});

export type VerificationModelOutput = z.infer<typeof VerificationModelOutputSchema>;

/**
 * Verification configuration
 */
export const VerificationConfigSchema = z.object({
  /**
   * Minimum faithfulness score threshold (0-1)
   * Outputs below this are flagged as unfaithful
   */
  faithfulness_threshold: z.number().min(0).max(1).default(0.7),

  /**
   * Enable claim extraction and verification
   */
  extract_claims: z.boolean().default(true),

  /**
   * Enable hallucination detection
   */
  detect_hallucinations: z.boolean().default(true),

  /**
   * Enable contradiction detection
   */
  detect_contradictions: z.boolean().default(true),

  /**
   * Verification granularity
   */
  granularity: z.enum([
    'document',   // Overall document-level score
    'paragraph',  // Paragraph-level analysis
    'sentence',   // Sentence-level analysis
    'claim',      // Claim-level analysis
  ]).default('claim'),

  /**
   * Maximum number of claims to extract per output
   */
  max_claims: z.number().positive().default(50),

  /**
   * Include evidence citations in output
   */
  include_evidence: z.boolean().default(true),

  /**
   * Verification method
   */
  method: z.enum([
    'nli',              // Natural Language Inference
    'semantic',         // Semantic similarity
    'entailment',       // Entailment-based
    'hybrid',           // Combination of methods
  ]).default('hybrid'),
});

export type VerificationConfig = z.infer<typeof VerificationConfigSchema>;

/**
 * LLM provider configuration for verification
 */
export const VerificationProviderConfigSchema = z.object({
  provider_name: z.enum([
    'openai',
    'anthropic',
    'google',
    'mistral',
    'azure-openai',
    'bedrock',
    'groq',
    'together',
  ]),
  model_id: z.string().min(1),
  api_key_ref: z.string().optional(),
  base_url: z.string().url().optional(),
  timeout_ms: z.number().positive().default(60000),
  max_retries: z.number().nonnegative().default(3),
});

export type VerificationProviderConfig = z.infer<typeof VerificationProviderConfigSchema>;

/**
 * Main input schema for Faithfulness Verification Agent
 */
export const FaithfulnessVerificationInputSchema = z.object({
  // Required: source documents to verify against
  sources: z.array(SourceDocumentSchema).min(1),

  // Required: model output to verify
  output: VerificationModelOutputSchema,

  // Optional: verification configuration
  config: VerificationConfigSchema.optional(),

  // Optional: LLM provider for verification (if using LLM-based methods)
  provider: VerificationProviderConfigSchema.optional(),

  // Optional: caller context
  caller_id: z.string().optional(),
  correlation_id: z.string().uuid().optional(),
});

export type FaithfulnessVerificationInput = z.infer<typeof FaithfulnessVerificationInputSchema>;

// =============================================================================
// OUTPUT SCHEMAS
// =============================================================================

/**
 * Evidence for a claim
 */
export const EvidenceSchema = z.object({
  document_id: z.string(),
  text: z.string(),
  start_offset: z.number().nonnegative().optional(),
  end_offset: z.number().nonnegative().optional(),
  relevance_score: z.number().min(0).max(1),
});

export type Evidence = z.infer<typeof EvidenceSchema>;

/**
 * Individual claim verification result
 */
export const ClaimVerificationSchema = z.object({
  claim_id: z.string(),
  claim_text: z.string(),
  claim_type: z.enum([
    'factual',       // Statement of fact
    'inference',     // Logical inference
    'opinion',       // Opinion or judgment
    'numerical',     // Numerical claim
    'temporal',      // Time-related claim
    'causal',        // Cause-effect claim
    'comparison',    // Comparative claim
  ]),

  // Verification result
  verdict: z.enum([
    'supported',       // Claim is supported by sources
    'partially_supported', // Some support but incomplete
    'not_supported',   // No support found
    'contradicted',    // Contradicts sources
    'unverifiable',    // Cannot be verified against sources
  ]),

  // Confidence in verdict
  confidence: z.number().min(0).max(1),

  // Supporting evidence
  evidence: z.array(EvidenceSchema).optional(),

  // Explanation of verdict
  explanation: z.string().optional(),

  // Position in output
  start_offset: z.number().nonnegative().optional(),
  end_offset: z.number().nonnegative().optional(),
});

export type ClaimVerification = z.infer<typeof ClaimVerificationSchema>;

/**
 * Hallucination detection result
 */
export const HallucinationSchema = z.object({
  hallucination_id: z.string(),
  text: z.string(),
  hallucination_type: z.enum([
    'fabrication',       // Completely fabricated information
    'exaggeration',      // Exaggerated claims
    'misattribution',    // Incorrect attribution
    'conflation',        // Conflated information
    'outdated',          // Outdated information
    'unsupported_inference', // Inference not supported by sources
  ]),
  severity: z.enum(['critical', 'major', 'minor']),
  confidence: z.number().min(0).max(1),
  explanation: z.string().optional(),
  start_offset: z.number().nonnegative().optional(),
  end_offset: z.number().nonnegative().optional(),
});

export type Hallucination = z.infer<typeof HallucinationSchema>;

/**
 * Contradiction detection result
 */
export const ContradictionSchema = z.object({
  contradiction_id: z.string(),
  output_text: z.string(),
  source_text: z.string(),
  source_document_id: z.string(),
  contradiction_type: z.enum([
    'direct',        // Direct contradiction
    'implicit',      // Implicit contradiction
    'temporal',      // Temporal contradiction
    'numerical',     // Numerical discrepancy
    'logical',       // Logical inconsistency
  ]),
  severity: z.enum(['critical', 'major', 'minor']),
  confidence: z.number().min(0).max(1),
  explanation: z.string().optional(),
});

export type Contradiction = z.infer<typeof ContradictionSchema>;

/**
 * Faithfulness score breakdown
 */
export const FaithfulnessScoresSchema = z.object({
  // Overall faithfulness score (0-1)
  overall: z.number().min(0).max(1),

  // Component scores
  claim_support_rate: z.number().min(0).max(1),
  hallucination_rate: z.number().min(0).max(1),
  contradiction_rate: z.number().min(0).max(1),
  coverage_score: z.number().min(0).max(1),

  // Weighted factors
  factors: z.array(z.object({
    factor: z.string(),
    weight: z.number().min(0).max(1),
    value: z.number().min(0).max(1),
    description: z.string().optional(),
  })),
});

export type FaithfulnessScores = z.infer<typeof FaithfulnessScoresSchema>;

/**
 * Main output schema for Faithfulness Verification Agent
 */
export const FaithfulnessVerificationOutputSchema = z.object({
  // Execution identity
  execution_id: z.string().uuid(),
  output_id: z.string(),

  // Timing
  started_at: z.string().datetime(),
  completed_at: z.string().datetime(),
  duration_ms: z.number().nonnegative(),

  // Overall result
  is_faithful: z.boolean(),
  faithfulness_scores: FaithfulnessScoresSchema,

  // Detailed results
  claims: z.array(ClaimVerificationSchema).optional(),
  hallucinations: z.array(HallucinationSchema).optional(),
  contradictions: z.array(ContradictionSchema).optional(),

  // Summary statistics
  summary: z.object({
    total_claims: z.number().nonnegative(),
    supported_claims: z.number().nonnegative(),
    partially_supported_claims: z.number().nonnegative(),
    unsupported_claims: z.number().nonnegative(),
    contradicted_claims: z.number().nonnegative(),
    unverifiable_claims: z.number().nonnegative(),
    total_hallucinations: z.number().nonnegative(),
    total_contradictions: z.number().nonnegative(),
    sources_used: z.number().nonnegative(),
  }),

  // Configuration used
  config: VerificationConfigSchema,

  // Constraints applied during execution
  constraints_applied: z.array(z.string()),
});

export type FaithfulnessVerificationOutput = z.infer<typeof FaithfulnessVerificationOutputSchema>;

// =============================================================================
// DECISION EVENT SCHEMA (SPECIALIZED)
// =============================================================================

/**
 * Faithfulness Verification Decision Event
 * Extends base DecisionEvent with faithfulness-specific outputs
 */
export const FaithfulnessVerificationDecisionEventSchema = DecisionEventSchema.extend({
  decision_type: z.literal('faithfulness_verification'),
  outputs: FaithfulnessVerificationOutputSchema,
});

export type FaithfulnessVerificationDecisionEvent = z.infer<typeof FaithfulnessVerificationDecisionEventSchema>;

// =============================================================================
// CLI CONTRACT
// =============================================================================

/**
 * CLI invocation shape for Faithfulness Verification Agent
 */
export const FaithfulnessVerificationCLIArgsSchema = z.object({
  // Input source (one required)
  input_file: z.string().optional(),
  input_json: z.string().optional(),
  input_stdin: z.boolean().optional(),

  // Source documents (alternative to input)
  sources_file: z.string().optional(),
  output_text: z.string().optional(),

  // Output format
  output_format: z.enum(['json', 'table', 'summary']).default('json'),
  output_file: z.string().optional(),

  // Configuration overrides
  threshold: z.number().min(0).max(1).optional(),
  granularity: z.enum(['document', 'paragraph', 'sentence', 'claim']).optional(),
  method: z.enum(['nli', 'semantic', 'entailment', 'hybrid']).optional(),

  // Verbosity
  verbose: z.boolean().default(false),
  quiet: z.boolean().default(false),

  // Execution modifiers
  dry_run: z.boolean().default(false),
});

export type FaithfulnessVerificationCLIArgs = z.infer<typeof FaithfulnessVerificationCLIArgsSchema>;

// =============================================================================
// CONSTRAINTS & NON-RESPONSIBILITIES
// =============================================================================

/**
 * Constraints that MAY be applied during execution
 */
export const FAITHFULNESS_VALID_CONSTRAINTS = [
  'max_claims_exceeded',
  'max_duration_exceeded',
  'provider_unavailable',
  'source_too_large',
  'output_too_large',
  'rate_limit_applied',
  'low_confidence_threshold',
  'method_fallback',
] as const;

/**
 * Explicit non-responsibilities - this agent MUST NOT:
 */
export const FAITHFULNESS_NON_RESPONSIBILITIES = [
  'generate_content',         // No content generation
  'modify_output',            // No modification of model output
  'compare_models',           // No model comparison
  'score_quality',            // No general quality scoring (only faithfulness)
  'enforce_policy',           // No policy enforcement
  'orchestrate_workflows',    // No workflow orchestration
  'call_other_agents',        // No direct agent-to-agent calls
  'store_api_keys',           // Never persist API keys
  'execute_arbitrary_code',   // No code execution
  'bypass_schemas',           // Must validate all I/O
  'make_edits',               // No editing or rewriting
  'cache_sources',            // No caching of source documents
] as const;

// =============================================================================
// CONFIDENCE SCORING
// =============================================================================

/**
 * Factors that contribute to confidence scoring
 */
export const FAITHFULNESS_CONFIDENCE_FACTORS = {
  claim_verification_rate: {
    description: 'Percentage of claims with high-confidence verdicts',
    weight: 0.35,
  },
  evidence_quality: {
    description: 'Quality and relevance of evidence found',
    weight: 0.25,
  },
  method_reliability: {
    description: 'Reliability of verification method used',
    weight: 0.20,
  },
  source_coverage: {
    description: 'Coverage of source documents analyzed',
    weight: 0.10,
  },
  claim_count: {
    description: 'Number of claims analyzed (more = higher confidence)',
    weight: 0.10,
  },
} as const;

/**
 * Calculate confidence score based on verification results
 */
export function calculateFaithfulnessConfidence(
  output: FaithfulnessVerificationOutput
): { confidence: number; factors: Array<{ factor: string; weight: number; value: number }> } {
  const factors: Array<{ factor: string; weight: number; value: number }> = [];

  // Claim verification rate
  const totalClaims = output.summary.total_claims;
  const verifiedClaims = output.summary.supported_claims +
    output.summary.partially_supported_claims +
    output.summary.contradicted_claims;
  const claimVerificationRate = totalClaims > 0 ? verifiedClaims / totalClaims : 0;
  factors.push({
    factor: 'claim_verification_rate',
    weight: FAITHFULNESS_CONFIDENCE_FACTORS.claim_verification_rate.weight,
    value: claimVerificationRate,
  });

  // Evidence quality (based on average relevance score)
  const allEvidence = output.claims?.flatMap(c => c.evidence || []) || [];
  const avgEvidenceQuality = allEvidence.length > 0
    ? allEvidence.reduce((sum, e) => sum + e.relevance_score, 0) / allEvidence.length
    : 0.5;
  factors.push({
    factor: 'evidence_quality',
    weight: FAITHFULNESS_CONFIDENCE_FACTORS.evidence_quality.weight,
    value: avgEvidenceQuality,
  });

  // Method reliability (hybrid = highest, semantic = lowest)
  const methodReliability: Record<string, number> = {
    hybrid: 0.9,
    nli: 0.85,
    entailment: 0.8,
    semantic: 0.7,
  };
  const method = output.config.method ?? 'hybrid';
  factors.push({
    factor: 'method_reliability',
    weight: FAITHFULNESS_CONFIDENCE_FACTORS.method_reliability.weight,
    value: methodReliability[method] ?? 0.7,
  });

  // Source coverage
  const sourceCoverage = Math.min(1, output.summary.sources_used / 5);
  factors.push({
    factor: 'source_coverage',
    weight: FAITHFULNESS_CONFIDENCE_FACTORS.source_coverage.weight,
    value: sourceCoverage,
  });

  // Claim count (logarithmic scale, capped at 50 claims)
  const claimCountScore = Math.min(1, Math.log10(totalClaims + 1) / Math.log10(51));
  factors.push({
    factor: 'claim_count',
    weight: FAITHFULNESS_CONFIDENCE_FACTORS.claim_count.weight,
    value: claimCountScore,
  });

  // Calculate weighted confidence
  const confidence = Math.min(1, Math.max(0,
    factors.reduce((sum, f) => sum + f.weight * f.value, 0)
  ));

  return { confidence, factors };
}

// =============================================================================
// CORE BUNDLE CONSUMERS
// =============================================================================

/**
 * Core bundles that may consume this agent's output
 */
export const FAITHFULNESS_ALLOWED_CONSUMERS = [
  'llm-orchestrator',         // For workflow coordination
  'llm-observatory',          // For telemetry/monitoring
  'llm-analytics',            // For aggregation/analysis
  'llm-test-bench-ui',        // For dashboard display
  'llm-quality-gate',         // For quality gating decisions
  'rag-evaluator',            // For RAG evaluation pipelines
] as const;

// =============================================================================
// VERSIONING RULES
// =============================================================================

export const FAITHFULNESS_VERSIONING_RULES = {
  major: 'Breaking changes to input/output schemas, removal of verification methods',
  minor: 'New optional fields, new claim types, new hallucination types, new methods',
  patch: 'Bug fixes, performance improvements, accuracy improvements, documentation',
} as const;

// =============================================================================
// FAILURE MODES
// =============================================================================

export const FAITHFULNESS_FAILURE_MODES = [
  {
    code: 'VALIDATION_ERROR',
    description: 'Input validation failed',
    recoverable: true,
    action: 'Fix input data and retry',
  },
  {
    code: 'PROVIDER_ERROR',
    description: 'LLM provider returned an error',
    recoverable: true,
    action: 'Retry with backoff or use fallback provider',
  },
  {
    code: 'TIMEOUT_ERROR',
    description: 'Verification exceeded time limit',
    recoverable: true,
    action: 'Reduce input size or increase timeout',
  },
  {
    code: 'SOURCE_TOO_LARGE',
    description: 'Source documents exceed size limits',
    recoverable: false,
    action: 'Reduce source document size',
  },
  {
    code: 'OUTPUT_TOO_LARGE',
    description: 'Model output exceeds size limits',
    recoverable: false,
    action: 'Reduce output size',
  },
  {
    code: 'NO_CLAIMS_EXTRACTED',
    description: 'No verifiable claims could be extracted',
    recoverable: false,
    action: 'Check output content or adjust granularity',
  },
  {
    code: 'METHOD_UNAVAILABLE',
    description: 'Requested verification method unavailable',
    recoverable: true,
    action: 'Use fallback method (hybrid)',
  },
  {
    code: 'PERSISTENCE_ERROR',
    description: 'Failed to persist decision event',
    recoverable: true,
    action: 'Retry persistence or log for manual recovery',
  },
] as const;

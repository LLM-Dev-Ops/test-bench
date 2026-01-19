/**
 * Hallucination Detection Agent - Edge Function Handler
 *
 * AGENT PURPOSE:
 * Detect unsupported or fabricated claims relative to provided reference context.
 * Analyzes claims against reference material to identify fabrications, exaggerations,
 * misattributions, and contradictions.
 *
 * This agent:
 * - Detects hallucinations in claims (YES)
 * - Analyzes claims against reference context (YES)
 * - Classifies hallucination types (YES)
 * - Does NOT generate content (NO)
 * - Does NOT execute benchmarks (NO - that's benchmark-runner)
 * - Does NOT compare models (NO - that's model-comparator)
 * - Does NOT enforce policies (NO - that's policy agents)
 *
 * Deployed as: Google Cloud Edge Function
 * Part of: LLM-Test-Bench unified GCP service
 */

import { randomUUID } from 'crypto';
import { z } from 'zod';
import {
  // Contracts
  DecisionEvent,
  AgentError,
  validateInput,
  hashInputs,
} from '../contracts';

import {
  getRuVectorClient,
  createTelemetryEmitter,
  TelemetryEmitter,
} from '../services';

// =============================================================================
// AGENT METADATA
// =============================================================================

export const HALLUCINATION_DETECTOR_AGENT = {
  agent_id: 'hallucination-detector',
  agent_version: '1.0.0',
  decision_type: 'hallucination_detection',
} as const;

// =============================================================================
// TYPES
// =============================================================================

export interface EdgeFunctionRequest {
  body: unknown;
  headers: Record<string, string>;
  method: string;
  path: string;
}

export interface EdgeFunctionResponse {
  statusCode: number;
  headers: Record<string, string>;
  body: string;
}

interface ExecutionContext {
  executionId: string;
  startedAt: Date;
  telemetry: TelemetryEmitter;
  constraintsApplied: string[];
}

// =============================================================================
// INPUT SCHEMAS
// =============================================================================

/**
 * A single claim to be analyzed for hallucination
 */
export const ClaimSchema = z.object({
  /** Unique identifier for the claim */
  claim_id: z.string().min(1),

  /** The actual claim text to analyze */
  claim_text: z.string().min(1),

  /** Optional: source of the claim (e.g., LLM output, document section) */
  source: z.string().optional(),

  /** Optional: additional context about the claim */
  metadata: z.record(z.unknown()).optional(),
});

export type Claim = z.infer<typeof ClaimSchema>;

/**
 * Reference context used to validate claims
 */
export const ReferenceContextSchema = z.object({
  /** Unique identifier for the reference */
  reference_id: z.string().min(1),

  /** The reference content text */
  content: z.string().min(1),

  /** Type of reference material */
  content_type: z.enum([
    'document',
    'knowledge_base',
    'api_response',
    'database_result',
    'user_provided',
    'web_source',
    'other',
  ]).default('document'),

  /** Optional: title or name of the reference */
  title: z.string().optional(),

  /** Optional: URL or path to the original source */
  source_url: z.string().optional(),

  /** Optional: timestamp of when the reference was created/retrieved */
  retrieved_at: z.string().datetime().optional(),

  /** Optional: additional metadata */
  metadata: z.record(z.unknown()).optional(),
});

export type ReferenceContext = z.infer<typeof ReferenceContextSchema>;

/**
 * Detection configuration options
 */
export const DetectionConfigSchema = z.object({
  /** Minimum similarity threshold for considering a claim supported (0-1) */
  similarity_threshold: z.number().min(0).max(1).default(0.7),

  /** Enable semantic similarity analysis (uses embeddings) */
  enable_semantic_analysis: z.boolean().default(true),

  /** Enable keyword/entity matching */
  enable_keyword_matching: z.boolean().default(true),

  /** Enable contradiction detection */
  enable_contradiction_detection: z.boolean().default(true),

  /** Case sensitivity for text matching */
  case_sensitive: z.boolean().default(false),

  /** Maximum number of claims to process */
  max_claims: z.number().int().positive().default(100),

  /** Timeout for analysis in milliseconds */
  timeout_ms: z.number().int().positive().default(30000),

  /** Include detailed analysis in output */
  include_detailed_analysis: z.boolean().default(true),
});

export type DetectionConfig = z.infer<typeof DetectionConfigSchema>;

/**
 * Main input schema for Hallucination Detector Agent
 */
export const HallucinationDetectorInputSchema = z.object({
  /** Claims to analyze for hallucinations */
  claims: z.array(ClaimSchema).min(1).max(1000),

  /** Reference context(s) to validate claims against */
  reference_contexts: z.array(ReferenceContextSchema).min(1),

  /** Detection configuration */
  detection_config: DetectionConfigSchema.optional(),

  /** Optional: caller context */
  caller_id: z.string().optional(),

  /** Optional: correlation ID for tracing */
  correlation_id: z.string().uuid().optional(),
});

export type HallucinationDetectorInput = z.infer<typeof HallucinationDetectorInputSchema>;

// =============================================================================
// OUTPUT SCHEMAS
// =============================================================================

/**
 * Types of hallucinations that can be detected
 */
export const HallucinationType = z.enum([
  'fabrication',      // Claim not present in any reference context
  'exaggeration',     // Claim overstates or embellishes what's in context
  'misattribution',   // Claim attributes information to wrong source
  'contradiction',    // Claim directly contradicts reference context
  'unsupported',      // Claim cannot be verified (insufficient context)
  'partial_support',  // Claim partially supported but contains unverified elements
]);

export type HallucinationTypeValue = z.infer<typeof HallucinationType>;

/**
 * Evidence supporting the hallucination classification
 */
export const EvidenceSchema = z.object({
  /** Reference ID that this evidence comes from */
  reference_id: z.string(),

  /** Relevant excerpt from the reference */
  relevant_excerpt: z.string().optional(),

  /** Similarity score between claim and reference (0-1) */
  similarity_score: z.number().min(0).max(1),

  /** Keywords found in both claim and reference */
  matched_keywords: z.array(z.string()).optional(),

  /** Keywords in claim but not in reference */
  unmatched_keywords: z.array(z.string()).optional(),

  /** Whether this reference contradicts the claim */
  is_contradictory: z.boolean(),

  /** Explanation of the match/mismatch */
  explanation: z.string().optional(),
});

export type Evidence = z.infer<typeof EvidenceSchema>;

/**
 * Result for a single claim analysis
 */
export const HallucinationClaimResultSchema = z.object({
  /** Claim identifier */
  claim_id: z.string(),

  /** Original claim text */
  claim_text: z.string(),

  /** Whether a hallucination was detected */
  is_hallucination: z.boolean(),

  /** Type of hallucination (if detected) */
  hallucination_type: HallucinationType.nullable(),

  /** Confidence score for the classification (0-1) */
  confidence: z.number().min(0).max(1),

  /** Overall support score from references (0-1, 1 = fully supported) */
  support_score: z.number().min(0).max(1),

  /** Evidence from each reference context */
  evidence: z.array(EvidenceSchema),

  /** Best matching reference ID (if any) */
  best_matching_reference: z.string().nullable(),

  /** Severity of the hallucination (if detected) */
  severity: z.enum(['low', 'medium', 'high', 'critical']).nullable(),

  /** Human-readable summary of the analysis */
  summary: z.string(),

  /** Analysis timestamp */
  analyzed_at: z.string().datetime(),
});

export type HallucinationClaimResult = z.infer<typeof HallucinationClaimResultSchema>;

/**
 * Aggregated statistics for the detection run
 */
export const DetectionStatsSchema = z.object({
  /** Total claims analyzed */
  total_claims: z.number().int().nonnegative(),

  /** Claims classified as hallucinations */
  hallucinations_detected: z.number().int().nonnegative(),

  /** Claims fully supported by references */
  fully_supported: z.number().int().nonnegative(),

  /** Claims partially supported */
  partially_supported: z.number().int().nonnegative(),

  /** Hallucination rate (hallucinations / total) */
  hallucination_rate: z.number().min(0).max(1),

  /** Average confidence across all classifications */
  avg_confidence: z.number().min(0).max(1),

  /** Average support score across all claims */
  avg_support_score: z.number().min(0).max(1),

  /** Breakdown by hallucination type */
  by_type: z.object({
    fabrication: z.number().int().nonnegative(),
    exaggeration: z.number().int().nonnegative(),
    misattribution: z.number().int().nonnegative(),
    contradiction: z.number().int().nonnegative(),
    unsupported: z.number().int().nonnegative(),
    partial_support: z.number().int().nonnegative(),
  }),

  /** Breakdown by severity */
  by_severity: z.object({
    low: z.number().int().nonnegative(),
    medium: z.number().int().nonnegative(),
    high: z.number().int().nonnegative(),
    critical: z.number().int().nonnegative(),
  }),
});

export type DetectionStats = z.infer<typeof DetectionStatsSchema>;

/**
 * Main output schema for Hallucination Detector Agent
 */
export const HallucinationDetectorOutputSchema = z.object({
  /** Unique detection run identifier */
  detection_id: z.string().uuid(),

  /** Individual claim results */
  results: z.array(HallucinationClaimResultSchema),

  /** Aggregated statistics */
  stats: DetectionStatsSchema,

  /** Reference contexts used */
  references_used: z.array(z.object({
    reference_id: z.string(),
    title: z.string().optional(),
    content_type: z.string(),
  })),

  /** Detection configuration used */
  detection_config_used: DetectionConfigSchema,

  /** Timing */
  started_at: z.string().datetime(),
  completed_at: z.string().datetime(),
  duration_ms: z.number().nonnegative(),
});

export type HallucinationDetectorOutput = z.infer<typeof HallucinationDetectorOutputSchema>;

/**
 * Hallucination Detector Decision Event
 */
export const HallucinationDetectorDecisionEventSchema = z.object({
  agent_id: z.string(),
  agent_version: z.string(),
  decision_type: z.literal('hallucination_detection'),
  decision_id: z.string().uuid(),
  inputs_hash: z.string().length(64),
  inputs_summary: z.record(z.unknown()).optional(),
  outputs: HallucinationDetectorOutputSchema,
  confidence: z.number().min(0).max(1),
  confidence_factors: z.array(z.object({
    factor: z.string(),
    weight: z.number().min(0).max(1),
    value: z.number().min(0).max(1),
  })).optional(),
  constraints_applied: z.array(z.string()),
  execution_ref: z.object({
    execution_id: z.string().uuid(),
    trace_id: z.string().uuid().optional(),
    span_id: z.string().optional(),
    parent_span_id: z.string().optional(),
  }),
  timestamp: z.string().datetime(),
  duration_ms: z.number().nonnegative(),
});

export type HallucinationDetectorDecisionEvent = z.infer<typeof HallucinationDetectorDecisionEventSchema>;

// =============================================================================
// CONSTRAINTS
// =============================================================================

export const VALID_CONSTRAINTS = [
  'max_claims_exceeded',           // Claim count exceeded limit
  'timeout_exceeded',              // Processing timeout reached
  'semantic_analysis_unavailable', // Embeddings not available
  'reference_context_empty',       // Reference context has no usable content
  'low_confidence_classification', // Classification confidence below threshold
  'partial_analysis_only',         // Some claims could not be fully analyzed
] as const;

// =============================================================================
// MAIN HANDLER
// =============================================================================

/**
 * Edge Function Handler for Hallucination Detector Agent
 *
 * This is the main entry point for the agent.
 * Deployed as a Google Cloud Edge Function.
 */
export async function handler(
  request: EdgeFunctionRequest
): Promise<EdgeFunctionResponse> {
  const executionId = randomUUID();
  const startedAt = new Date();

  // Initialize telemetry
  const telemetry = createTelemetryEmitter(
    HALLUCINATION_DETECTOR_AGENT.agent_id,
    HALLUCINATION_DETECTOR_AGENT.agent_version,
    executionId
  );

  const context: ExecutionContext = {
    executionId,
    startedAt,
    telemetry,
    constraintsApplied: [],
  };

  try {
    // Emit invocation telemetry
    telemetry.emitInvoked();

    // Handle only POST requests
    if (request.method !== 'POST') {
      return createErrorResponse(405, 'Method Not Allowed');
    }

    // Parse and validate input
    const inputValidation = validateInput(HallucinationDetectorInputSchema, request.body);
    if (!inputValidation.success) {
      telemetry.emitValidationFailed('input', (inputValidation as { success: false; error: AgentError }).error.message);
      await telemetry.flush();

      return createErrorResponse(400, 'Validation Error', (inputValidation as { success: false; error: AgentError }).error);
    }

    const input = inputValidation.data;

    // Execute hallucination detection
    const output = await detectHallucinations(input, context);

    // Calculate confidence
    const overallConfidence = calculateOverallConfidence(output);

    // Create DecisionEvent
    const decisionEvent = await createDecisionEvent(
      input,
      output,
      overallConfidence,
      context
    );

    // Persist DecisionEvent (async, non-blocking)
    const ruVectorClient = getRuVectorClient();
    await ruVectorClient.persistDecisionEvent(decisionEvent);

    // Emit completion telemetry
    telemetry.emitDecision(decisionEvent.decision_id, overallConfidence);
    telemetry.emitCompleted({
      duration_ms: Date.now() - startedAt.getTime(),
      success_count: output.stats.total_claims - output.stats.hallucinations_detected,
      failure_count: output.stats.hallucinations_detected,
    });

    // Flush telemetry
    await telemetry.flush();

    // Return success response
    return createSuccessResponse(output, decisionEvent.decision_id);

  } catch (err) {
    // Handle unexpected errors
    const error = err instanceof Error ? err : new Error(String(err));

    telemetry.emitError('EXECUTION_ERROR', error.message);
    await telemetry.flush();

    return createErrorResponse(500, 'Internal Server Error', {
      code: 'EXECUTION_ERROR',
      message: error.message,
      recoverable: false,
      timestamp: new Date().toISOString(),
    });
  }
}

// =============================================================================
// CORE DETECTION LOGIC
// =============================================================================

async function detectHallucinations(
  input: HallucinationDetectorInput,
  context: ExecutionContext
): Promise<HallucinationDetectorOutput> {
  const config = input.detection_config ?? {
    similarity_threshold: 0.7,
    enable_semantic_analysis: true,
    enable_keyword_matching: true,
    enable_contradiction_detection: true,
    case_sensitive: false,
    max_claims: 100,
    timeout_ms: 30000,
    include_detailed_analysis: true,
  };

  const results: HallucinationClaimResult[] = [];
  const startTime = context.startedAt;

  // Apply max_claims constraint
  let claimsToProcess = input.claims;
  if (input.claims.length > config.max_claims) {
    claimsToProcess = input.claims.slice(0, config.max_claims);
    context.constraintsApplied.push('max_claims_exceeded');
    context.telemetry.emitConstraintApplied(
      'max_claims_exceeded',
      `Processing ${config.max_claims} of ${input.claims.length} claims`
    );
  }

  // Preprocess reference contexts
  const preprocessedRefs = preprocessReferenceContexts(input.reference_contexts, config);

  // Analyze each claim
  for (const claim of claimsToProcess) {
    // Check timeout constraint
    const elapsed = Date.now() - startTime.getTime();
    if (elapsed >= config.timeout_ms) {
      context.constraintsApplied.push('timeout_exceeded');
      context.telemetry.emitConstraintApplied(
        'timeout_exceeded',
        `Elapsed: ${elapsed}ms, Max: ${config.timeout_ms}ms`
      );
      break;
    }

    const result = await analyzeClaimAgainstContext(
      claim,
      preprocessedRefs,
      input.reference_contexts,
      config
    );

    results.push(result);
  }

  const completedAt = new Date();

  // Calculate aggregated stats
  const stats = calculateDetectionStats(results);

  // Build output
  const output: HallucinationDetectorOutput = {
    detection_id: context.executionId,
    results,
    stats,
    references_used: input.reference_contexts.map(ref => ({
      reference_id: ref.reference_id,
      title: ref.title,
      content_type: ref.content_type,
    })),
    detection_config_used: config,
    started_at: startTime.toISOString(),
    completed_at: completedAt.toISOString(),
    duration_ms: completedAt.getTime() - startTime.getTime(),
  };

  return output;
}

interface PreprocessedReference {
  reference_id: string;
  content: string;
  normalizedContent: string;
  keywords: Set<string>;
  sentences: string[];
}

function preprocessReferenceContexts(
  references: ReferenceContext[],
  config: DetectionConfig
): PreprocessedReference[] {
  return references.map(ref => {
    const normalizedContent = config.case_sensitive
      ? ref.content
      : ref.content.toLowerCase();

    // Extract keywords (simple word tokenization)
    const keywords = new Set(
      normalizedContent
        .split(/\W+/)
        .filter(word => word.length > 2)
    );

    // Split into sentences
    const sentences = ref.content.split(/[.!?]+/).filter(s => s.trim().length > 0);

    return {
      reference_id: ref.reference_id,
      content: ref.content,
      normalizedContent,
      keywords,
      sentences,
    };
  });
}

async function analyzeClaimAgainstContext(
  claim: Claim,
  preprocessedRefs: PreprocessedReference[],
  originalRefs: ReferenceContext[],
  config: DetectionConfig
): Promise<HallucinationClaimResult> {
  const evidence: Evidence[] = [];
  let bestSimilarity = 0;
  let bestMatchingRef: string | null = null;
  let hasContradiction = false;

  const normalizedClaim = config.case_sensitive
    ? claim.claim_text
    : claim.claim_text.toLowerCase();

  // Extract keywords from claim
  const claimKeywords = new Set(
    normalizedClaim.split(/\W+/).filter(word => word.length > 2)
  );

  // Analyze against each reference
  for (const ref of preprocessedRefs) {
    // Keyword matching
    const matchedKeywords: string[] = [];
    const unmatchedKeywords: string[] = [];

    if (config.enable_keyword_matching) {
      Array.from(claimKeywords).forEach(keyword => {
        if (ref.keywords.has(keyword)) {
          matchedKeywords.push(keyword);
        } else {
          unmatchedKeywords.push(keyword);
        }
      });
    }

    // Calculate similarity score
    const keywordSimilarity = claimKeywords.size > 0
      ? matchedKeywords.length / claimKeywords.size
      : 0;

    // Semantic similarity (simplified - would use embeddings in production)
    let semanticSimilarity = 0;
    if (config.enable_semantic_analysis) {
      semanticSimilarity = calculateSemanticSimilarity(normalizedClaim, ref.normalizedContent);
    }

    // Combined similarity
    const similarityScore = config.enable_semantic_analysis
      ? (keywordSimilarity * 0.4 + semanticSimilarity * 0.6)
      : keywordSimilarity;

    // Find most relevant excerpt
    const relevantExcerpt = findRelevantExcerpt(claim.claim_text, ref.sentences);

    // Contradiction detection
    let isContradictory = false;
    if (config.enable_contradiction_detection) {
      isContradictory = detectContradiction(normalizedClaim, ref.normalizedContent);
      if (isContradictory) {
        hasContradiction = true;
      }
    }

    // Build evidence entry
    evidence.push({
      reference_id: ref.reference_id,
      relevant_excerpt: relevantExcerpt,
      similarity_score: similarityScore,
      matched_keywords: matchedKeywords.length > 0 ? matchedKeywords : undefined,
      unmatched_keywords: unmatchedKeywords.length > 0 ? unmatchedKeywords : undefined,
      is_contradictory: isContradictory,
      explanation: generateExplanation(similarityScore, matchedKeywords, unmatchedKeywords, isContradictory),
    });

    // Track best match
    if (similarityScore > bestSimilarity) {
      bestSimilarity = similarityScore;
      bestMatchingRef = ref.reference_id;
    }
  }

  // Classify the hallucination
  const classification = classifyHallucination(
    bestSimilarity,
    hasContradiction,
    evidence,
    config.similarity_threshold
  );

  // Calculate confidence for this classification
  const claimConfidence = calculateClaimConfidence(evidence, classification);

  // Determine severity
  const severity = classification.isHallucination
    ? determineSeverity(classification.type!, bestSimilarity)
    : null;

  return {
    claim_id: claim.claim_id,
    claim_text: claim.claim_text,
    is_hallucination: classification.isHallucination,
    hallucination_type: classification.type,
    confidence: claimConfidence,
    support_score: bestSimilarity,
    evidence,
    best_matching_reference: bestMatchingRef,
    severity,
    summary: generateSummary(classification, bestSimilarity, bestMatchingRef),
    analyzed_at: new Date().toISOString(),
  };
}

function calculateSemanticSimilarity(text1: string, text2: string): number {
  // Simplified semantic similarity using character n-grams
  // In production, this would use actual embeddings (e.g., from OpenAI, Cohere, etc.)

  const getNGrams = (text: string, n: number): Set<string> => {
    const ngrams = new Set<string>();
    for (let i = 0; i <= text.length - n; i++) {
      ngrams.add(text.slice(i, i + n));
    }
    return ngrams;
  };

  const ngrams1 = getNGrams(text1, 3);
  const ngrams2 = getNGrams(text2, 3);

  if (ngrams1.size === 0 || ngrams2.size === 0) return 0;

  let intersection = 0;
  Array.from(ngrams1).forEach(gram => {
    if (ngrams2.has(gram)) {
      intersection++;
    }
  });

  // Jaccard similarity
  const union = ngrams1.size + ngrams2.size - intersection;
  return union > 0 ? intersection / union : 0;
}

function findRelevantExcerpt(claim: string, sentences: string[]): string | undefined {
  if (sentences.length === 0) return undefined;

  const claimLower = claim.toLowerCase();
  let bestMatch = '';
  let bestScore = 0;

  for (const sentence of sentences) {
    const sentenceLower = sentence.toLowerCase();
    const score = calculateSemanticSimilarity(claimLower, sentenceLower);

    if (score > bestScore) {
      bestScore = score;
      bestMatch = sentence.trim();
    }
  }

  return bestScore > 0.1 ? bestMatch : undefined;
}

function detectContradiction(claim: string, reference: string): boolean {
  // Simplified contradiction detection using negation patterns
  // In production, this would use NLI models or more sophisticated logic

  const negationPatterns = [
    /\bnot\b/,
    /\bnever\b/,
    /\bno\b/,
    /\bwithout\b/,
    /\bcan't\b/,
    /\bcannot\b/,
    /\bwon't\b/,
    /\bdoesn't\b/,
    /\bdon't\b/,
    /\bisn't\b/,
    /\baren't\b/,
    /\bwasn't\b/,
    /\bweren't\b/,
  ];

  const claimHasNegation = negationPatterns.some(p => p.test(claim));
  const refHasNegation = negationPatterns.some(p => p.test(reference));

  // Very simplified: if claim has negation but reference doesn't (or vice versa)
  // and they share significant content, might be a contradiction
  // This is a placeholder - real implementation would be much more sophisticated
  if (claimHasNegation !== refHasNegation) {
    const similarity = calculateSemanticSimilarity(claim, reference);
    return similarity > 0.3; // Threshold for considering it a potential contradiction
  }

  return false;
}

function generateExplanation(
  similarity: number,
  matched: string[],
  unmatched: string[],
  isContradictory: boolean
): string {
  const parts: string[] = [];

  if (isContradictory) {
    parts.push('Potential contradiction detected.');
  }

  if (similarity >= 0.7) {
    parts.push(`Strong support (${(similarity * 100).toFixed(0)}% similarity).`);
  } else if (similarity >= 0.4) {
    parts.push(`Partial support (${(similarity * 100).toFixed(0)}% similarity).`);
  } else {
    parts.push(`Weak support (${(similarity * 100).toFixed(0)}% similarity).`);
  }

  if (matched.length > 0) {
    parts.push(`Matched keywords: ${matched.slice(0, 5).join(', ')}.`);
  }

  if (unmatched.length > 0 && unmatched.length <= 5) {
    parts.push(`Unmatched keywords: ${unmatched.join(', ')}.`);
  } else if (unmatched.length > 5) {
    parts.push(`${unmatched.length} keywords not found in reference.`);
  }

  return parts.join(' ');
}

interface Classification {
  isHallucination: boolean;
  type: HallucinationTypeValue | null;
}

function classifyHallucination(
  bestSimilarity: number,
  hasContradiction: boolean,
  evidence: Evidence[],
  threshold: number
): Classification {
  // Priority 1: Contradiction
  if (hasContradiction) {
    return {
      isHallucination: true,
      type: 'contradiction',
    };
  }

  // Priority 2: Fabrication (very low similarity)
  if (bestSimilarity < 0.2) {
    return {
      isHallucination: true,
      type: 'fabrication',
    };
  }

  // Priority 3: Unsupported (low similarity)
  if (bestSimilarity < 0.4) {
    return {
      isHallucination: true,
      type: 'unsupported',
    };
  }

  // Priority 4: Partial support (below threshold but some match)
  if (bestSimilarity < threshold) {
    return {
      isHallucination: true,
      type: 'partial_support',
    };
  }

  // Priority 5: Exaggeration check (high similarity but with unmatched content)
  const avgUnmatchedRatio = evidence.reduce((sum, e) => {
    const unmatched = e.unmatched_keywords?.length ?? 0;
    const matched = e.matched_keywords?.length ?? 0;
    const total = matched + unmatched;
    return sum + (total > 0 ? unmatched / total : 0);
  }, 0) / evidence.length;

  if (bestSimilarity >= threshold && avgUnmatchedRatio > 0.4) {
    return {
      isHallucination: true,
      type: 'exaggeration',
    };
  }

  // Not a hallucination - claim is supported
  return {
    isHallucination: false,
    type: null,
  };
}

function calculateClaimConfidence(evidence: Evidence[], classification: Classification): number {
  if (evidence.length === 0) return 0.5;

  // Higher confidence when:
  // 1. Multiple references agree
  // 2. Similarity scores are consistent
  // 3. Classification is clear-cut

  const similarities = evidence.map(e => e.similarity_score);
  const avgSimilarity = similarities.reduce((a, b) => a + b, 0) / similarities.length;

  // Variance in similarity scores (lower = more confident)
  const variance = similarities.reduce((sum, s) => sum + Math.pow(s - avgSimilarity, 2), 0) / similarities.length;
  const consistencyScore = 1 - Math.min(1, variance * 4);

  // Classification clarity
  let clarityScore: number;
  if (classification.isHallucination) {
    // More confident for clear fabrications (very low similarity) or contradictions
    if (classification.type === 'fabrication' || classification.type === 'contradiction') {
      clarityScore = 0.9;
    } else if (classification.type === 'unsupported') {
      clarityScore = 0.7;
    } else {
      clarityScore = 0.6;
    }
  } else {
    // More confident when well-supported
    clarityScore = avgSimilarity > 0.8 ? 0.9 : 0.75;
  }

  // Number of references factor
  const referenceFactor = Math.min(1, evidence.length / 3);

  // Weighted combination
  return Math.min(1, Math.max(0,
    consistencyScore * 0.3 +
    clarityScore * 0.5 +
    referenceFactor * 0.2
  ));
}

function determineSeverity(
  type: HallucinationTypeValue,
  supportScore: number
): 'low' | 'medium' | 'high' | 'critical' {
  // Severity based on type and support score
  if (type === 'contradiction') {
    return 'critical'; // Contradictions are always severe
  }

  if (type === 'fabrication') {
    return supportScore < 0.1 ? 'critical' : 'high';
  }

  if (type === 'misattribution') {
    return 'high';
  }

  if (type === 'exaggeration') {
    return supportScore < 0.5 ? 'medium' : 'low';
  }

  if (type === 'unsupported') {
    return supportScore < 0.3 ? 'high' : 'medium';
  }

  // partial_support
  return 'low';
}

function generateSummary(
  classification: Classification,
  similarity: number,
  bestRef: string | null
): string {
  if (!classification.isHallucination) {
    return `Claim is supported by reference context with ${(similarity * 100).toFixed(0)}% confidence.`;
  }

  const typeDescriptions: Record<HallucinationTypeValue, string> = {
    fabrication: 'Claim appears to be fabricated with no basis in reference context.',
    exaggeration: 'Claim exaggerates or embellishes information from reference context.',
    misattribution: 'Claim incorrectly attributes information to a source.',
    contradiction: 'Claim directly contradicts information in reference context.',
    unsupported: 'Claim cannot be verified against available reference context.',
    partial_support: 'Claim is only partially supported by reference context.',
  };

  return typeDescriptions[classification.type!];
}

function calculateDetectionStats(results: HallucinationClaimResult[]): DetectionStats {
  const total = results.length;

  if (total === 0) {
    return {
      total_claims: 0,
      hallucinations_detected: 0,
      fully_supported: 0,
      partially_supported: 0,
      hallucination_rate: 0,
      avg_confidence: 0,
      avg_support_score: 0,
      by_type: {
        fabrication: 0,
        exaggeration: 0,
        misattribution: 0,
        contradiction: 0,
        unsupported: 0,
        partial_support: 0,
      },
      by_severity: {
        low: 0,
        medium: 0,
        high: 0,
        critical: 0,
      },
    };
  }

  const hallucinations = results.filter(r => r.is_hallucination);
  const fullySupported = results.filter(r => !r.is_hallucination && r.support_score >= 0.7);
  const partiallySupported = results.filter(r => !r.is_hallucination && r.support_score < 0.7);

  const byType = {
    fabrication: 0,
    exaggeration: 0,
    misattribution: 0,
    contradiction: 0,
    unsupported: 0,
    partial_support: 0,
  };

  const bySeverity = {
    low: 0,
    medium: 0,
    high: 0,
    critical: 0,
  };

  for (const result of hallucinations) {
    if (result.hallucination_type) {
      byType[result.hallucination_type]++;
    }
    if (result.severity) {
      bySeverity[result.severity]++;
    }
  }

  return {
    total_claims: total,
    hallucinations_detected: hallucinations.length,
    fully_supported: fullySupported.length,
    partially_supported: partiallySupported.length,
    hallucination_rate: hallucinations.length / total,
    avg_confidence: results.reduce((s, r) => s + r.confidence, 0) / total,
    avg_support_score: results.reduce((s, r) => s + r.support_score, 0) / total,
    by_type: byType,
    by_severity: bySeverity,
  };
}

// =============================================================================
// CONFIDENCE CALCULATION
// =============================================================================

export const CONFIDENCE_FACTORS = {
  sample_size: {
    description: 'Number of claims analyzed',
    weight: 0.2,
  },
  reference_coverage: {
    description: 'Quality and coverage of reference contexts',
    weight: 0.25,
  },
  classification_consistency: {
    description: 'Consistency of hallucination classifications',
    weight: 0.25,
  },
  average_claim_confidence: {
    description: 'Average confidence across individual claims',
    weight: 0.3,
  },
} as const;

function calculateOverallConfidence(output: HallucinationDetectorOutput): number {
  if (output.results.length === 0) return 0;

  const factors: Array<{ factor: string; weight: number; value: number }> = [];

  // Sample size factor (logarithmic, capped at 100 claims)
  const sampleSizeValue = Math.min(1, Math.log10(output.results.length + 1) / 2);
  factors.push({
    factor: 'sample_size',
    weight: CONFIDENCE_FACTORS.sample_size.weight,
    value: sampleSizeValue,
  });

  // Reference coverage factor
  const refCount = output.references_used.length;
  const refCoverageValue = Math.min(1, refCount / 5); // Assume 5+ refs is optimal
  factors.push({
    factor: 'reference_coverage',
    weight: CONFIDENCE_FACTORS.reference_coverage.weight,
    value: refCoverageValue,
  });

  // Classification consistency (inverse of hallucination rate variance from extreme)
  // Higher confidence when results are clearly hallucinated or clearly not
  const hallucinationRate = output.stats.hallucination_rate;
  const consistencyValue = 1 - 4 * Math.abs(hallucinationRate - 0.5) * Math.abs(hallucinationRate - 0.5);
  // This peaks at 0 and 1, lowest at 0.5 (most uncertain)
  factors.push({
    factor: 'classification_consistency',
    weight: CONFIDENCE_FACTORS.classification_consistency.weight,
    value: Math.max(0.3, 1 - consistencyValue), // Invert so extreme rates = high confidence
  });

  // Average claim confidence
  factors.push({
    factor: 'average_claim_confidence',
    weight: CONFIDENCE_FACTORS.average_claim_confidence.weight,
    value: output.stats.avg_confidence,
  });

  // Calculate weighted confidence
  const confidence = factors.reduce(
    (sum, f) => sum + f.weight * f.value,
    0
  );

  return Math.min(1, Math.max(0, confidence));
}

// =============================================================================
// DECISION EVENT CREATION
// =============================================================================

async function createDecisionEvent(
  input: HallucinationDetectorInput,
  output: HallucinationDetectorOutput,
  confidence: number,
  context: ExecutionContext
): Promise<DecisionEvent> {
  const inputsHash = await hashInputs(input);

  return {
    agent_id: HALLUCINATION_DETECTOR_AGENT.agent_id,
    agent_version: HALLUCINATION_DETECTOR_AGENT.agent_version,
    decision_type: HALLUCINATION_DETECTOR_AGENT.decision_type,
    decision_id: randomUUID(),
    inputs_hash: inputsHash,
    inputs_summary: {
      total_claims: input.claims.length,
      reference_count: input.reference_contexts.length,
    },
    outputs: output,
    confidence,
    confidence_factors: [
      { factor: 'sample_size', weight: 0.2, value: Math.min(1, Math.log10(output.results.length + 1) / 2) },
      { factor: 'reference_coverage', weight: 0.25, value: Math.min(1, output.references_used.length / 5) },
      { factor: 'avg_claim_confidence', weight: 0.3, value: output.stats.avg_confidence },
    ],
    constraints_applied: context.constraintsApplied,
    execution_ref: {
      execution_id: context.executionId,
    },
    timestamp: new Date().toISOString(),
    duration_ms: Date.now() - context.startedAt.getTime(),
  };
}

// =============================================================================
// RESPONSE HELPERS
// =============================================================================

function createSuccessResponse(
  output: HallucinationDetectorOutput,
  decisionId: string
): EdgeFunctionResponse {
  return {
    statusCode: 200,
    headers: {
      'Content-Type': 'application/json',
      'X-Decision-Id': decisionId,
      'X-Agent-Id': HALLUCINATION_DETECTOR_AGENT.agent_id,
      'X-Agent-Version': HALLUCINATION_DETECTOR_AGENT.agent_version,
    },
    body: JSON.stringify({
      success: true,
      decision_id: decisionId,
      data: output,
    }),
  };
}

function createErrorResponse(
  statusCode: number,
  message: string,
  error?: AgentError
): EdgeFunctionResponse {
  return {
    statusCode,
    headers: {
      'Content-Type': 'application/json',
      'X-Agent-Id': HALLUCINATION_DETECTOR_AGENT.agent_id,
      'X-Agent-Version': HALLUCINATION_DETECTOR_AGENT.agent_version,
    },
    body: JSON.stringify({
      success: false,
      error: error ?? {
        code: statusCode === 400 ? 'VALIDATION_ERROR' : 'EXECUTION_ERROR',
        message,
        recoverable: statusCode < 500,
        timestamp: new Date().toISOString(),
      },
    }),
  };
}

// =============================================================================
// EXPORTS
// =============================================================================

export { HALLUCINATION_DETECTOR_AGENT as default };

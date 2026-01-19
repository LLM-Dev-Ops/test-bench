/**
 * Faithfulness Verification Agent - Edge Function Handler
 *
 * AGENT PURPOSE:
 * Measure alignment between model output and supplied source documents.
 * The agent evaluates whether the generated content faithfully represents
 * the information in the source documents without hallucination, distortion,
 * or unsupported claims.
 *
 * This agent:
 * - Verifies faithfulness of model outputs against sources (YES)
 * - Identifies hallucinations and unsupported claims (YES)
 * - Calculates faithfulness scores with evidence (YES)
 * - Does NOT generate new content (NO)
 * - Does NOT compare models (NO)
 * - Does NOT orchestrate workflows (NO)
 *
 * Deployed as: Google Cloud Edge Function
 * Part of: LLM-Test-Bench unified GCP service
 */

import { randomUUID } from 'crypto';
import {
  // Contracts
  FaithfulnessVerificationInputSchema,
  FaithfulnessVerificationOutputSchema,
  FaithfulnessVerificationDecisionEventSchema,
  DecisionEvent,
  AgentError,
  validateInput,
  hashInputs,
  // Constants
  FAITHFULNESS_VERIFICATION_AGENT,
  FAITHFULNESS_VALID_CONSTRAINTS,
  calculateFaithfulnessConfidence,
  // Types
  FaithfulnessVerificationInput,
  FaithfulnessVerificationOutput,
  SourceDocument,
  VerificationModelOutput,
  VerificationConfig,
  VerificationProviderConfig,
  ClaimVerification,
  Hallucination,
  Contradiction,
  Evidence,
  FaithfulnessScores,
} from '../contracts';

import {
  getRuVectorClient,
  createTelemetryEmitter,
  TelemetryEmitter,
} from '../services';

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

interface ExtractedClaim {
  id: string;
  text: string;
  type: ClaimVerification['claim_type'];
  startOffset?: number;
  endOffset?: number;
}

// =============================================================================
// MAIN HANDLER
// =============================================================================

/**
 * Edge Function Handler for Faithfulness Verification Agent
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
    FAITHFULNESS_VERIFICATION_AGENT.agent_id,
    FAITHFULNESS_VERIFICATION_AGENT.agent_version,
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
    const inputValidation = validateInput(FaithfulnessVerificationInputSchema, request.body);
    if (inputValidation.success === false) {
      telemetry.emitValidationFailed('input', inputValidation.error.message);
      await telemetry.flush();

      return createErrorResponse(400, 'Validation Error', inputValidation.error);
    }

    const input = inputValidation.data;

    // Validate source size
    const totalSourceSize = input.sources.reduce((sum, s) => sum + s.content.length, 0);
    if (totalSourceSize > 500000) { // 500KB limit
      context.constraintsApplied.push('source_too_large');
      telemetry.emitConstraintApplied('source_too_large', `Size: ${totalSourceSize} bytes`);
    }

    // Validate output size
    if (input.output.content.length > 100000) { // 100KB limit
      context.constraintsApplied.push('output_too_large');
      telemetry.emitConstraintApplied('output_too_large', `Size: ${input.output.content.length} bytes`);
    }

    // Execute faithfulness verification
    const output = await verifyFaithfulness(input, context);

    // Calculate confidence
    const { confidence, factors } = calculateFaithfulnessConfidence(output);

    // Create DecisionEvent
    const decisionEvent = await createDecisionEvent(
      input,
      output,
      confidence,
      factors,
      context
    );

    // Persist DecisionEvent (async, non-blocking)
    const ruVectorClient = getRuVectorClient();
    await ruVectorClient.persistDecisionEvent(decisionEvent);

    // Emit completion telemetry
    telemetry.emitDecision(decisionEvent.decision_id, confidence);
    telemetry.emitCompleted({
      duration_ms: Date.now() - startedAt.getTime(),
      faithfulness_score: output.faithfulness_scores.overall,
      total_claims: output.summary.total_claims,
      hallucination_count: output.summary.total_hallucinations,
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
// CORE VERIFICATION LOGIC
// =============================================================================

async function verifyFaithfulness(
  input: FaithfulnessVerificationInput,
  context: ExecutionContext
): Promise<FaithfulnessVerificationOutput> {
  const config: VerificationConfig = {
    faithfulness_threshold: 0.7,
    extract_claims: true,
    detect_hallucinations: true,
    detect_contradictions: true,
    granularity: 'claim',
    max_claims: 50,
    include_evidence: true,
    method: 'hybrid',
    ...input.config,
  };

  // Extract claims from model output
  const extractedClaims = await extractClaims(input.output, config, context);

  // Check max claims constraint
  if (extractedClaims.length > config.max_claims) {
    context.constraintsApplied.push('max_claims_exceeded');
    context.telemetry.emitConstraintApplied(
      'max_claims_exceeded',
      `Found ${extractedClaims.length}, max ${config.max_claims}`
    );
    extractedClaims.splice(config.max_claims);
  }

  // Verify each claim against sources
  const claims: ClaimVerification[] = await Promise.all(
    extractedClaims.map(claim => verifyClaim(claim, input.sources, config, input.provider))
  );

  // Detect hallucinations
  let hallucinations: Hallucination[] = [];
  if (config.detect_hallucinations) {
    hallucinations = await detectHallucinations(
      input.output,
      input.sources,
      claims,
      config,
      input.provider
    );
  }

  // Detect contradictions
  let contradictions: Contradiction[] = [];
  if (config.detect_contradictions) {
    contradictions = await detectContradictions(
      input.output,
      input.sources,
      claims,
      config,
      input.provider
    );
  }

  // Calculate faithfulness scores
  const faithfulnessScores = calculateFaithfulnessScores(
    claims,
    hallucinations,
    contradictions,
    input.sources.length
  );

  const completedAt = new Date();

  // Build summary
  const summary = {
    total_claims: claims.length,
    supported_claims: claims.filter(c => c.verdict === 'supported').length,
    partially_supported_claims: claims.filter(c => c.verdict === 'partially_supported').length,
    unsupported_claims: claims.filter(c => c.verdict === 'not_supported').length,
    contradicted_claims: claims.filter(c => c.verdict === 'contradicted').length,
    unverifiable_claims: claims.filter(c => c.verdict === 'unverifiable').length,
    total_hallucinations: hallucinations.length,
    total_contradictions: contradictions.length,
    sources_used: input.sources.length,
  };

  // Build output
  const output: FaithfulnessVerificationOutput = {
    execution_id: context.executionId,
    output_id: input.output.output_id,
    started_at: context.startedAt.toISOString(),
    completed_at: completedAt.toISOString(),
    duration_ms: completedAt.getTime() - context.startedAt.getTime(),
    is_faithful: faithfulnessScores.overall >= config.faithfulness_threshold,
    faithfulness_scores: faithfulnessScores,
    claims: config.extract_claims ? claims : undefined,
    hallucinations: config.detect_hallucinations ? hallucinations : undefined,
    contradictions: config.detect_contradictions ? contradictions : undefined,
    summary,
    config,
    constraints_applied: context.constraintsApplied,
  };

  return output;
}

// =============================================================================
// CLAIM EXTRACTION
// =============================================================================

async function extractClaims(
  output: VerificationModelOutput,
  config: VerificationConfig,
  context: ExecutionContext
): Promise<ExtractedClaim[]> {
  const claims: ExtractedClaim[] = [];
  const content = output.content;

  // Split based on granularity
  let segments: Array<{ text: string; start: number; end: number }> = [];

  switch (config.granularity) {
    case 'document':
      segments = [{ text: content, start: 0, end: content.length }];
      break;

    case 'paragraph':
      segments = splitIntoParagraphs(content);
      break;

    case 'sentence':
      segments = splitIntoSentences(content);
      break;

    case 'claim':
    default:
      // Extract atomic claims from sentences
      const sentences = splitIntoSentences(content);
      for (const sentence of sentences) {
        const sentenceClaims = extractAtomicClaims(sentence.text);
        for (const claim of sentenceClaims) {
          segments.push({
            text: claim,
            start: sentence.start,
            end: sentence.end,
          });
        }
      }
      break;
  }

  // Create claim objects
  for (let i = 0; i < segments.length; i++) {
    const segment = segments[i];
    if (segment.text.trim().length < 10) continue; // Skip very short segments

    claims.push({
      id: `claim-${i + 1}`,
      text: segment.text.trim(),
      type: classifyClaimType(segment.text),
      startOffset: segment.start,
      endOffset: segment.end,
    });
  }

  return claims;
}

function splitIntoParagraphs(text: string): Array<{ text: string; start: number; end: number }> {
  const paragraphs: Array<{ text: string; start: number; end: number }> = [];
  const regex = /\n\s*\n/g;
  let lastEnd = 0;
  let match;

  while ((match = regex.exec(text)) !== null) {
    if (match.index > lastEnd) {
      paragraphs.push({
        text: text.slice(lastEnd, match.index),
        start: lastEnd,
        end: match.index,
      });
    }
    lastEnd = match.index + match[0].length;
  }

  if (lastEnd < text.length) {
    paragraphs.push({
      text: text.slice(lastEnd),
      start: lastEnd,
      end: text.length,
    });
  }

  return paragraphs;
}

function splitIntoSentences(text: string): Array<{ text: string; start: number; end: number }> {
  const sentences: Array<{ text: string; start: number; end: number }> = [];
  // Simple sentence splitter - handles common cases
  const regex = /[^.!?]*[.!?]+/g;
  let match;

  while ((match = regex.exec(text)) !== null) {
    sentences.push({
      text: match[0].trim(),
      start: match.index,
      end: match.index + match[0].length,
    });
  }

  // Handle remaining text without sentence-ending punctuation
  const lastEnd = sentences.length > 0 ? sentences[sentences.length - 1].end : 0;
  if (lastEnd < text.length) {
    const remaining = text.slice(lastEnd).trim();
    if (remaining.length > 0) {
      sentences.push({
        text: remaining,
        start: lastEnd,
        end: text.length,
      });
    }
  }

  return sentences;
}

function extractAtomicClaims(sentence: string): string[] {
  // Split compound sentences into atomic claims
  // This is a simplified version - a real implementation would use NLP
  const claims: string[] = [];

  // Split on conjunctions and semicolons
  const parts = sentence.split(/(?:,\s*(?:and|but|or|however|although)\s+)|(?:;\s*)/i);

  for (const part of parts) {
    const trimmed = part.trim();
    if (trimmed.length >= 10) {
      claims.push(trimmed);
    }
  }

  // If no splits, return the original sentence
  if (claims.length === 0 && sentence.trim().length >= 10) {
    claims.push(sentence.trim());
  }

  return claims;
}

function classifyClaimType(claim: string): ClaimVerification['claim_type'] {
  const lowerClaim = claim.toLowerCase();

  // Numerical claim detection
  if (/\d+(?:\.\d+)?(?:\s*%|\s*percent|\s*million|\s*billion)?/i.test(claim)) {
    return 'numerical';
  }

  // Temporal claim detection
  if (/\b(?:in\s+\d{4}|yesterday|today|tomorrow|last\s+\w+|next\s+\w+|before|after|during)\b/i.test(lowerClaim)) {
    return 'temporal';
  }

  // Causal claim detection
  if (/\b(?:because|therefore|thus|hence|causes?|leads?\s+to|results?\s+in|due\s+to)\b/i.test(lowerClaim)) {
    return 'causal';
  }

  // Comparison claim detection
  if (/\b(?:more|less|better|worse|greater|smaller|larger|faster|slower|than|compared)\b/i.test(lowerClaim)) {
    return 'comparison';
  }

  // Opinion detection
  if (/\b(?:i\s+think|i\s+believe|in\s+my\s+opinion|probably|possibly|might|may\s+be|seems?\s+to)\b/i.test(lowerClaim)) {
    return 'opinion';
  }

  // Inference detection
  if (/\b(?:suggests?|implies?|indicates?|means\s+that|shows?\s+that)\b/i.test(lowerClaim)) {
    return 'inference';
  }

  // Default to factual
  return 'factual';
}

// =============================================================================
// CLAIM VERIFICATION
// =============================================================================

async function verifyClaim(
  claim: ExtractedClaim,
  sources: SourceDocument[],
  config: VerificationConfig,
  provider?: VerificationProviderConfig
): Promise<ClaimVerification> {
  // Find relevant evidence from sources
  const evidence = findEvidence(claim.text, sources, config);

  // Determine verdict based on evidence
  const { verdict, confidence, explanation } = determineVerdict(
    claim.text,
    evidence,
    config.method
  );

  return {
    claim_id: claim.id,
    claim_text: claim.text,
    claim_type: claim.type,
    verdict,
    confidence,
    evidence: config.include_evidence ? evidence : undefined,
    explanation,
    start_offset: claim.startOffset,
    end_offset: claim.endOffset,
  };
}

function findEvidence(
  claimText: string,
  sources: SourceDocument[],
  config: VerificationConfig
): Evidence[] {
  const evidence: Evidence[] = [];
  const claimTokens = tokenize(claimText);

  for (const source of sources) {
    // Find relevant passages in source
    const passages = findRelevantPassages(source.content, claimTokens);

    for (const passage of passages) {
      const relevanceScore = calculateSemanticSimilarity(claimText, passage.text);

      if (relevanceScore >= 0.3) { // Minimum relevance threshold
        evidence.push({
          document_id: source.document_id,
          text: passage.text,
          start_offset: passage.start,
          end_offset: passage.end,
          relevance_score: relevanceScore,
        });
      }
    }
  }

  // Sort by relevance and limit
  return evidence
    .sort((a, b) => b.relevance_score - a.relevance_score)
    .slice(0, 5);
}

function findRelevantPassages(
  content: string,
  claimTokens: string[]
): Array<{ text: string; start: number; end: number }> {
  const passages: Array<{ text: string; start: number; end: number }> = [];
  const sentences = splitIntoSentences(content);

  for (const sentence of sentences) {
    const sentenceTokens = tokenize(sentence.text);
    const overlap = claimTokens.filter(t => sentenceTokens.includes(t)).length;
    const overlapRatio = overlap / Math.max(claimTokens.length, 1);

    if (overlapRatio >= 0.2) { // At least 20% token overlap
      passages.push(sentence);
    }
  }

  return passages;
}

function tokenize(text: string): string[] {
  return text
    .toLowerCase()
    .replace(/[^\w\s]/g, '')
    .split(/\s+/)
    .filter(t => t.length > 2); // Filter out very short tokens
}

function calculateSemanticSimilarity(text1: string, text2: string): number {
  // Simplified similarity using Jaccard similarity on tokens
  // A real implementation would use embeddings
  const tokens1 = new Set(tokenize(text1));
  const tokens2 = new Set(tokenize(text2));

  if (tokens1.size === 0 || tokens2.size === 0) return 0;

  const intersection = new Set(Array.from(tokens1).filter(t => tokens2.has(t)));
  const union = new Set(Array.from(tokens1).concat(Array.from(tokens2)));

  return intersection.size / union.size;
}

function determineVerdict(
  claimText: string,
  evidence: Evidence[],
  method: string
): { verdict: ClaimVerification['verdict']; confidence: number; explanation: string } {
  if (evidence.length === 0) {
    return {
      verdict: 'not_supported',
      confidence: 0.7,
      explanation: 'No relevant evidence found in source documents.',
    };
  }

  const maxRelevance = Math.max(...evidence.map(e => e.relevance_score));
  const avgRelevance = evidence.reduce((sum, e) => sum + e.relevance_score, 0) / evidence.length;

  // Check for contradictions (simplified)
  const hasContradiction = evidence.some(e =>
    containsNegation(e.text, claimText) ||
    containsContradictoryNumbers(e.text, claimText)
  );

  if (hasContradiction) {
    return {
      verdict: 'contradicted',
      confidence: 0.8,
      explanation: 'Evidence contains information that contradicts the claim.',
    };
  }

  // Determine support level
  if (maxRelevance >= 0.7) {
    return {
      verdict: 'supported',
      confidence: Math.min(0.95, maxRelevance),
      explanation: `Claim is strongly supported by ${evidence.length} evidence passage(s).`,
    };
  } else if (maxRelevance >= 0.5) {
    return {
      verdict: 'partially_supported',
      confidence: maxRelevance,
      explanation: `Claim is partially supported; some aspects may not be fully covered.`,
    };
  } else if (maxRelevance >= 0.3) {
    return {
      verdict: 'partially_supported',
      confidence: maxRelevance,
      explanation: `Weak support found; claim may extend beyond source information.`,
    };
  }

  return {
    verdict: 'unverifiable',
    confidence: 0.5,
    explanation: 'Insufficient evidence to verify or refute the claim.',
  };
}

function containsNegation(evidence: string, claim: string): boolean {
  const negationWords = ['not', 'never', 'no', 'none', 'neither', 'cannot', 'doesn\'t', 'don\'t', 'isn\'t', 'aren\'t'];
  const evidenceTokens = evidence.toLowerCase().split(/\s+/);
  const claimTokens = claim.toLowerCase().split(/\s+/);

  // Check if evidence has negation where claim doesn't, or vice versa
  const evidenceHasNegation = evidenceTokens.some(t => negationWords.includes(t));
  const claimHasNegation = claimTokens.some(t => negationWords.includes(t));

  return evidenceHasNegation !== claimHasNegation;
}

function containsContradictoryNumbers(evidence: string, claim: string): boolean {
  const extractNumbers = (text: string): number[] => {
    const matches = text.match(/\d+(?:\.\d+)?/g);
    return matches ? matches.map(Number) : [];
  };

  const evidenceNumbers = extractNumbers(evidence);
  const claimNumbers = extractNumbers(claim);

  // Check if numbers differ significantly
  for (const claimNum of claimNumbers) {
    for (const evidenceNum of evidenceNumbers) {
      // If both numbers are in similar context and differ by more than 10%
      const diff = Math.abs(claimNum - evidenceNum) / Math.max(claimNum, evidenceNum, 1);
      if (diff > 0.1 && diff < 1) {
        return true;
      }
    }
  }

  return false;
}

// =============================================================================
// HALLUCINATION DETECTION
// =============================================================================

async function detectHallucinations(
  output: VerificationModelOutput,
  sources: SourceDocument[],
  claims: ClaimVerification[],
  config: VerificationConfig,
  provider?: VerificationProviderConfig
): Promise<Hallucination[]> {
  const hallucinations: Hallucination[] = [];

  // Identify unsupported claims as potential hallucinations
  const unsupportedClaims = claims.filter(c =>
    c.verdict === 'not_supported' || c.verdict === 'unverifiable'
  );

  for (const claim of unsupportedClaims) {
    const hallucinationType = classifyHallucination(claim);

    if (hallucinationType) {
      hallucinations.push({
        hallucination_id: `hal-${hallucinations.length + 1}`,
        text: claim.claim_text,
        hallucination_type: hallucinationType.type,
        severity: hallucinationType.severity,
        confidence: 1 - claim.confidence,
        explanation: hallucinationType.explanation,
        start_offset: claim.start_offset,
        end_offset: claim.end_offset,
      });
    }
  }

  // Check for fabricated entities (names, dates, etc.)
  const fabricatedEntities = detectFabricatedEntities(output.content, sources);
  for (const entity of fabricatedEntities) {
    hallucinations.push({
      hallucination_id: `hal-${hallucinations.length + 1}`,
      ...entity,
    });
  }

  return hallucinations;
}

function classifyHallucination(
  claim: ClaimVerification
): { type: Hallucination['hallucination_type']; severity: Hallucination['severity']; explanation: string } | null {
  if (claim.verdict !== 'not_supported' && claim.verdict !== 'unverifiable') {
    return null;
  }

  // Classify based on claim type and content
  switch (claim.claim_type) {
    case 'numerical':
      return {
        type: 'fabrication',
        severity: 'major',
        explanation: 'Numerical claim not found in source documents.',
      };

    case 'factual':
      if (claim.confidence < 0.3) {
        return {
          type: 'fabrication',
          severity: 'critical',
          explanation: 'Factual claim appears to be fabricated.',
        };
      }
      return {
        type: 'unsupported_inference',
        severity: 'major',
        explanation: 'Factual claim lacks supporting evidence.',
      };

    case 'inference':
      return {
        type: 'unsupported_inference',
        severity: 'minor',
        explanation: 'Inference not supported by source evidence.',
      };

    case 'temporal':
      return {
        type: 'fabrication',
        severity: 'major',
        explanation: 'Temporal information not found in sources.',
      };

    case 'causal':
      return {
        type: 'unsupported_inference',
        severity: 'major',
        explanation: 'Causal relationship not established in sources.',
      };

    default:
      return {
        type: 'unsupported_inference',
        severity: 'minor',
        explanation: 'Claim not adequately supported by sources.',
      };
  }
}

function detectFabricatedEntities(
  outputContent: string,
  sources: SourceDocument[]
): Array<Omit<Hallucination, 'hallucination_id'>> {
  const fabricated: Array<Omit<Hallucination, 'hallucination_id'>> = [];

  // Extract potential named entities from output (simplified)
  const potentialEntities = extractNamedEntities(outputContent);

  // Check if entities exist in sources
  const sourceContent = sources.map(s => s.content.toLowerCase()).join(' ');

  for (const entity of potentialEntities) {
    if (!sourceContent.includes(entity.text.toLowerCase())) {
      fabricated.push({
        text: entity.text,
        hallucination_type: 'fabrication',
        severity: entity.type === 'person' ? 'critical' : 'major',
        confidence: 0.7,
        explanation: `${entity.type} "${entity.text}" not found in source documents.`,
        start_offset: entity.start,
        end_offset: entity.end,
      });
    }
  }

  return fabricated;
}

function extractNamedEntities(
  text: string
): Array<{ text: string; type: string; start: number; end: number }> {
  const entities: Array<{ text: string; type: string; start: number; end: number }> = [];

  // Simple pattern-based entity extraction
  // A real implementation would use NER

  // Capitalized multi-word phrases (potential names/organizations)
  const capitalizedRegex = /(?:[A-Z][a-z]+(?:\s+[A-Z][a-z]+)+)/g;
  let match;
  while ((match = capitalizedRegex.exec(text)) !== null) {
    entities.push({
      text: match[0],
      type: 'entity',
      start: match.index,
      end: match.index + match[0].length,
    });
  }

  return entities.slice(0, 10); // Limit entities checked
}

// =============================================================================
// CONTRADICTION DETECTION
// =============================================================================

async function detectContradictions(
  output: VerificationModelOutput,
  sources: SourceDocument[],
  claims: ClaimVerification[],
  config: VerificationConfig,
  provider?: VerificationProviderConfig
): Promise<Contradiction[]> {
  const contradictions: Contradiction[] = [];

  // Check claims marked as contradicted
  const contradictedClaims = claims.filter(c => c.verdict === 'contradicted');

  for (const claim of contradictedClaims) {
    const evidence = claim.evidence?.[0];
    if (evidence) {
      contradictions.push({
        contradiction_id: `con-${contradictions.length + 1}`,
        output_text: claim.claim_text,
        source_text: evidence.text,
        source_document_id: evidence.document_id,
        contradiction_type: classifyContradictionType(claim.claim_text, evidence.text),
        severity: determineContradictionSeverity(claim),
        confidence: claim.confidence,
        explanation: `Output contradicts source: "${evidence.text.substring(0, 100)}..."`,
      });
    }
  }

  return contradictions;
}

function classifyContradictionType(
  outputText: string,
  sourceText: string
): Contradiction['contradiction_type'] {
  // Check for numerical contradiction
  if (containsContradictoryNumbers(sourceText, outputText)) {
    return 'numerical';
  }

  // Check for direct negation
  if (containsNegation(sourceText, outputText)) {
    return 'direct';
  }

  // Check for temporal contradiction
  const temporalPatterns = /\b(?:before|after|during|in\s+\d{4})\b/i;
  if (temporalPatterns.test(outputText) && temporalPatterns.test(sourceText)) {
    return 'temporal';
  }

  // Default to implicit
  return 'implicit';
}

function determineContradictionSeverity(
  claim: ClaimVerification
): Contradiction['severity'] {
  switch (claim.claim_type) {
    case 'factual':
    case 'numerical':
      return 'critical';
    case 'temporal':
    case 'causal':
      return 'major';
    default:
      return 'minor';
  }
}

// =============================================================================
// FAITHFULNESS SCORING
// =============================================================================

function calculateFaithfulnessScores(
  claims: ClaimVerification[],
  hallucinations: Hallucination[],
  contradictions: Contradiction[],
  sourceCount: number
): FaithfulnessScores {
  const totalClaims = claims.length;

  // Calculate claim support rate
  const supportedClaims = claims.filter(c =>
    c.verdict === 'supported' || c.verdict === 'partially_supported'
  ).length;
  const claimSupportRate = totalClaims > 0 ? supportedClaims / totalClaims : 1;

  // Calculate hallucination rate (inverse for scoring)
  const hallucinationRate = totalClaims > 0
    ? hallucinations.length / totalClaims
    : 0;

  // Calculate contradiction rate
  const contradictionRate = totalClaims > 0
    ? contradictions.length / totalClaims
    : 0;

  // Calculate coverage score (how well sources are utilized)
  const evidenceUsed = new Set(
    claims.flatMap(c => c.evidence?.map(e => e.document_id) || [])
  ).size;
  const coverageScore = sourceCount > 0 ? evidenceUsed / sourceCount : 0;

  // Calculate overall score
  const factors = [
    { factor: 'claim_support', weight: 0.40, value: claimSupportRate, description: 'Claims supported by sources' },
    { factor: 'no_hallucination', weight: 0.30, value: 1 - hallucinationRate, description: 'Absence of hallucinations' },
    { factor: 'no_contradiction', weight: 0.20, value: 1 - contradictionRate, description: 'Absence of contradictions' },
    { factor: 'source_coverage', weight: 0.10, value: coverageScore, description: 'Source document coverage' },
  ];

  const overall = Math.min(1, Math.max(0,
    factors.reduce((sum, f) => sum + f.weight * f.value, 0)
  ));

  return {
    overall,
    claim_support_rate: claimSupportRate,
    hallucination_rate: hallucinationRate,
    contradiction_rate: contradictionRate,
    coverage_score: coverageScore,
    factors,
  };
}

// =============================================================================
// DECISION EVENT CREATION
// =============================================================================

async function createDecisionEvent(
  input: FaithfulnessVerificationInput,
  output: FaithfulnessVerificationOutput,
  confidence: number,
  confidenceFactors: Array<{ factor: string; weight: number; value: number }>,
  context: ExecutionContext
): Promise<DecisionEvent> {
  const inputsHash = await hashInputs({
    sources: input.sources.map(s => ({ id: s.document_id, length: s.content.length })),
    output_id: input.output.output_id,
    config: input.config,
  });

  return {
    agent_id: FAITHFULNESS_VERIFICATION_AGENT.agent_id,
    agent_version: FAITHFULNESS_VERIFICATION_AGENT.agent_version,
    decision_type: FAITHFULNESS_VERIFICATION_AGENT.decision_type,
    decision_id: randomUUID(),
    inputs_hash: inputsHash,
    inputs_summary: {
      output_id: input.output.output_id,
      source_count: input.sources.length,
      output_length: input.output.content.length,
    },
    outputs: output,
    confidence,
    confidence_factors: confidenceFactors,
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
  output: FaithfulnessVerificationOutput,
  decisionId: string
): EdgeFunctionResponse {
  return {
    statusCode: 200,
    headers: {
      'Content-Type': 'application/json',
      'X-Decision-Id': decisionId,
      'X-Agent-Id': FAITHFULNESS_VERIFICATION_AGENT.agent_id,
      'X-Agent-Version': FAITHFULNESS_VERIFICATION_AGENT.agent_version,
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
      'X-Agent-Id': FAITHFULNESS_VERIFICATION_AGENT.agent_id,
      'X-Agent-Version': FAITHFULNESS_VERIFICATION_AGENT.agent_version,
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

export { FAITHFULNESS_VERIFICATION_AGENT };

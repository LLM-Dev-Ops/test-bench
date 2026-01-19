/**
 * Golden Dataset Validator Agent - Edge Function Handler
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
 * - Does NOT generate content (NO)
 * - Does NOT train models (NO)
 * - Does NOT execute benchmarks (NO - that's benchmark-runner)
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

export const GOLDEN_DATASET_VALIDATOR_AGENT = {
  agent_id: 'golden-dataset-validator',
  agent_version: '1.0.0',
  decision_type: 'golden_dataset_validation',
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
 * A single sample from the golden dataset
 */
export const GoldenSampleSchema = z.object({
  sample_id: z.string().min(1),
  input: z.string().min(1),
  golden_output: z.string().min(1),
  category: z.string().optional(),
  difficulty: z.enum(['easy', 'medium', 'hard', 'expert']).optional(),
  tags: z.array(z.string()).optional(),
  metadata: z.record(z.unknown()).optional(),
});

export type GoldenSample = z.infer<typeof GoldenSampleSchema>;

/**
 * Model output to validate
 */
export const GoldenValidatorModelOutputSchema = z.object({
  sample_id: z.string().min(1),
  model_output: z.string(),
  model_id: z.string().optional(),
  provider: z.string().optional(),
  generated_at: z.string().datetime().optional(),
  metadata: z.record(z.unknown()).optional(),
});

export type GoldenValidatorModelOutput = z.infer<typeof GoldenValidatorModelOutputSchema>;

/**
 * Validation configuration
 */
export const ValidationConfigSchema = z.object({
  enable_exact_match: z.boolean().default(true),
  case_insensitive: z.boolean().default(false),
  enable_semantic_similarity: z.boolean().default(true),
  semantic_similarity_threshold: z.number().min(0).max(1).default(0.85),
  enable_keyword_analysis: z.boolean().default(true),
  keyword_overlap_threshold: z.number().min(0).max(1).default(0.7),
  enable_structural_similarity: z.boolean().default(false),
  numeric_tolerance: z.number().nonnegative().default(0.001),
  trim_whitespace: z.boolean().default(true),
  normalize_unicode: z.boolean().default(true),
  max_samples: z.number().int().positive().default(1000),
  timeout_ms: z.number().int().positive().default(60000),
  include_detailed_analysis: z.boolean().default(true),
});

export type ValidationConfig = z.infer<typeof ValidationConfigSchema>;

/**
 * Main input schema for Golden Dataset Validator Agent
 */
export const GoldenDatasetValidatorInputSchema = z.object({
  golden_samples: z.array(GoldenSampleSchema).min(1).max(10000),
  model_outputs: z.array(GoldenValidatorModelOutputSchema).min(1).max(10000),
  validation_config: ValidationConfigSchema.optional(),
  dataset: z.object({
    name: z.string().min(1),
    version: z.string().optional(),
    description: z.string().optional(),
    source: z.string().optional(),
  }).optional(),
  caller_id: z.string().optional(),
  correlation_id: z.string().uuid().optional(),
}).refine(
  (data) => {
    const goldenIds = new Set(data.golden_samples.map(s => s.sample_id));
    return data.model_outputs.every(o => goldenIds.has(o.sample_id));
  },
  { message: 'All model_outputs must have corresponding sample_ids in golden_samples' }
);

export type GoldenDatasetValidatorInput = z.infer<typeof GoldenDatasetValidatorInputSchema>;

// =============================================================================
// OUTPUT SCHEMAS
// =============================================================================

export const MatchType = z.enum([
  'exact_match',
  'semantic_match',
  'partial_match',
  'structural_match',
  'no_match',
  'error',
]);

export type MatchTypeValue = z.infer<typeof MatchType>;

export const ValidationSeverity = z.enum(['pass', 'warning', 'fail', 'critical']);
export type ValidationSeverityValue = z.infer<typeof ValidationSeverity>;

export const SampleValidationResultSchema = z.object({
  sample_id: z.string(),
  passed: z.boolean(),
  match_type: MatchType,
  severity: ValidationSeverity,
  confidence: z.number().min(0).max(1),
  exact_match: z.boolean(),
  semantic_similarity: z.number().min(0).max(1).nullable(),
  keyword_overlap: z.number().min(0).max(1).nullable(),
  structural_similarity: z.number().min(0).max(1).nullable(),
  golden_output: z.string(),
  model_output: z.string(),
  diff_summary: z.object({
    chars_added: z.number().nonnegative(),
    chars_removed: z.number().nonnegative(),
    words_added: z.number().nonnegative(),
    words_removed: z.number().nonnegative(),
    key_differences: z.array(z.string()).optional(),
  }).optional(),
  explanation: z.string(),
  validated_at: z.string().datetime(),
  category: z.string().optional(),
  metadata: z.record(z.unknown()).optional(),
});

export type SampleValidationResult = z.infer<typeof SampleValidationResultSchema>;

export const ValidationStatsSchema = z.object({
  total_samples: z.number().int().nonnegative(),
  passed: z.number().int().nonnegative(),
  failed: z.number().int().nonnegative(),
  pass_rate: z.number().min(0).max(1),
  exact_matches: z.number().int().nonnegative(),
  exact_match_rate: z.number().min(0).max(1),
  semantic_matches: z.number().int().nonnegative(),
  semantic_match_rate: z.number().min(0).max(1),
  partial_matches: z.number().int().nonnegative(),
  no_matches: z.number().int().nonnegative(),
  errors: z.number().int().nonnegative(),
  avg_semantic_similarity: z.number().min(0).max(1),
  avg_keyword_overlap: z.number().min(0).max(1),
  avg_confidence: z.number().min(0).max(1),
  by_match_type: z.object({
    exact_match: z.number().int().nonnegative(),
    semantic_match: z.number().int().nonnegative(),
    partial_match: z.number().int().nonnegative(),
    structural_match: z.number().int().nonnegative(),
    no_match: z.number().int().nonnegative(),
    error: z.number().int().nonnegative(),
  }),
  by_severity: z.object({
    pass: z.number().int().nonnegative(),
    warning: z.number().int().nonnegative(),
    fail: z.number().int().nonnegative(),
    critical: z.number().int().nonnegative(),
  }),
  by_category: z.record(z.object({
    total: z.number().int().nonnegative(),
    passed: z.number().int().nonnegative(),
    pass_rate: z.number().min(0).max(1),
  })).optional(),
});

export type ValidationStats = z.infer<typeof ValidationStatsSchema>;

export const GoldenDatasetValidatorOutputSchema = z.object({
  validation_id: z.string().uuid(),
  results: z.array(SampleValidationResultSchema),
  stats: ValidationStatsSchema,
  dataset: z.object({
    name: z.string(),
    version: z.string().optional(),
    sample_count: z.number().int().nonnegative(),
  }),
  validation_config_used: ValidationConfigSchema,
  model_info: z.object({
    model_id: z.string().optional(),
    provider: z.string().optional(),
    output_count: z.number().int().nonnegative(),
  }).optional(),
  quality_assessment: z.object({
    grade: z.enum(['A', 'B', 'C', 'D', 'F']),
    score: z.number().min(0).max(100),
    summary: z.string(),
    recommendations: z.array(z.string()).optional(),
  }),
  started_at: z.string().datetime(),
  completed_at: z.string().datetime(),
  duration_ms: z.number().nonnegative(),
});

export type GoldenDatasetValidatorOutput = z.infer<typeof GoldenDatasetValidatorOutputSchema>;

// =============================================================================
// CONSTRAINTS
// =============================================================================

export const VALID_CONSTRAINTS = [
  'max_samples_exceeded',
  'timeout_exceeded',
  'semantic_analysis_unavailable',
  'sample_mismatch',
  'invalid_sample_format',
  'memory_limit_exceeded',
  'low_confidence_result',
  'partial_validation_only',
] as const;

// =============================================================================
// MAIN HANDLER
// =============================================================================

/**
 * Edge Function Handler for Golden Dataset Validator Agent
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
    GOLDEN_DATASET_VALIDATOR_AGENT.agent_id,
    GOLDEN_DATASET_VALIDATOR_AGENT.agent_version,
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
    const inputValidation = validateInput(GoldenDatasetValidatorInputSchema, request.body);
    if (!inputValidation.success) {
      telemetry.emitValidationFailed('input', (inputValidation as { success: false; error: AgentError }).error.message);
      await telemetry.flush();

      return createErrorResponse(400, 'Validation Error', (inputValidation as { success: false; error: AgentError }).error);
    }

    const input = inputValidation.data;

    // Execute validation
    const output = await validateGoldenDataset(input, context);

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
      success_count: output.stats.passed,
      failure_count: output.stats.failed,
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
// CORE VALIDATION LOGIC
// =============================================================================

async function validateGoldenDataset(
  input: GoldenDatasetValidatorInput,
  context: ExecutionContext
): Promise<GoldenDatasetValidatorOutput> {
  const config: ValidationConfig = {
    enable_exact_match: true,
    case_insensitive: false,
    enable_semantic_similarity: true,
    semantic_similarity_threshold: 0.85,
    enable_keyword_analysis: true,
    keyword_overlap_threshold: 0.7,
    enable_structural_similarity: false,
    numeric_tolerance: 0.001,
    trim_whitespace: true,
    normalize_unicode: true,
    max_samples: 1000,
    timeout_ms: 60000,
    include_detailed_analysis: true,
    ...input.validation_config,
  };

  const results: SampleValidationResult[] = [];
  const startTime = context.startedAt;

  // Build golden samples lookup
  const goldenMap = new Map<string, GoldenSample>();
  for (const sample of input.golden_samples) {
    goldenMap.set(sample.sample_id, sample);
  }

  // Apply max_samples constraint
  let outputsToProcess = input.model_outputs;
  if (input.model_outputs.length > config.max_samples) {
    outputsToProcess = input.model_outputs.slice(0, config.max_samples);
    context.constraintsApplied.push('max_samples_exceeded');
    context.telemetry.emitConstraintApplied(
      'max_samples_exceeded',
      `Processing ${config.max_samples} of ${input.model_outputs.length} samples`
    );
  }

  // Validate each sample
  for (const modelOutput of outputsToProcess) {
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

    const goldenSample = goldenMap.get(modelOutput.sample_id);
    if (!goldenSample) {
      context.constraintsApplied.push('sample_mismatch');
      continue;
    }

    const result = await validateSample(goldenSample, modelOutput, config);
    results.push(result);
  }

  const completedAt = new Date();

  // Calculate aggregated stats
  const stats = calculateValidationStats(results);

  // Calculate quality assessment
  const qualityAssessment = calculateQualityAssessment(stats);

  // Extract model info
  const modelInfo = extractModelInfo(input.model_outputs);

  // Build output
  const output: GoldenDatasetValidatorOutput = {
    validation_id: context.executionId,
    results,
    stats,
    dataset: {
      name: input.dataset?.name || 'unnamed',
      version: input.dataset?.version,
      sample_count: input.golden_samples.length,
    },
    validation_config_used: config,
    model_info: modelInfo,
    quality_assessment: qualityAssessment,
    started_at: startTime.toISOString(),
    completed_at: completedAt.toISOString(),
    duration_ms: completedAt.getTime() - startTime.getTime(),
  };

  return output;
}

async function validateSample(
  goldenSample: GoldenSample,
  modelOutput: GoldenValidatorModelOutput,
  config: ValidationConfig
): Promise<SampleValidationResult> {
  const golden = preprocessText(goldenSample.golden_output, config);
  const model = preprocessText(modelOutput.model_output, config);

  // Check exact match
  const exactMatch = golden === model;

  // Calculate semantic similarity
  let semanticSimilarity: number | null = null;
  if (config.enable_semantic_similarity) {
    semanticSimilarity = calculateSemanticSimilarity(golden, model);
  }

  // Calculate keyword overlap
  let keywordOverlap: number | null = null;
  if (config.enable_keyword_analysis) {
    keywordOverlap = calculateKeywordOverlap(golden, model);
  }

  // Calculate structural similarity (placeholder)
  let structuralSimilarity: number | null = null;
  if (config.enable_structural_similarity) {
    structuralSimilarity = calculateStructuralSimilarity(golden, model);
  }

  // Classify match type
  const matchType = classifyMatchType(
    exactMatch,
    semanticSimilarity,
    keywordOverlap,
    structuralSimilarity,
    config
  );

  // Determine if passed
  const passed = matchType === 'exact_match' ||
    matchType === 'semantic_match' ||
    matchType === 'structural_match';

  // Determine severity
  const severity = determineSeverity(matchType, semanticSimilarity, keywordOverlap);

  // Calculate confidence
  const confidence = calculateSampleConfidence(
    exactMatch,
    semanticSimilarity,
    keywordOverlap,
    matchType,
    config
  );

  // Calculate diff summary
  const diffSummary = calculateDiffSummary(goldenSample.golden_output, modelOutput.model_output);

  // Generate explanation
  const explanation = generateExplanation(matchType, exactMatch, semanticSimilarity, keywordOverlap, passed);

  return {
    sample_id: goldenSample.sample_id,
    passed,
    match_type: matchType,
    severity,
    confidence,
    exact_match: exactMatch,
    semantic_similarity: semanticSimilarity,
    keyword_overlap: keywordOverlap,
    structural_similarity: structuralSimilarity,
    golden_output: goldenSample.golden_output,
    model_output: modelOutput.model_output,
    diff_summary: diffSummary,
    explanation,
    validated_at: new Date().toISOString(),
    category: goldenSample.category,
    metadata: goldenSample.metadata,
  };
}

function preprocessText(text: string, config: ValidationConfig): string {
  let processed = text;

  if (config.trim_whitespace) {
    processed = processed.trim();
  }

  if (config.normalize_unicode) {
    processed = processed.normalize('NFC');
  }

  if (config.case_insensitive) {
    processed = processed.toLowerCase();
  }

  return processed;
}

function calculateSemanticSimilarity(text1: string, text2: string): number {
  // Simplified semantic similarity using character n-grams
  // In production, this would use actual embeddings (e.g., from OpenAI, Cohere, etc.)

  if (text1 === text2) return 1.0;
  if (!text1 || !text2) return 0.0;

  const getNGrams = (text: string, n: number): Set<string> => {
    const ngrams = new Set<string>();
    const normalized = text.toLowerCase();
    for (let i = 0; i <= normalized.length - n; i++) {
      ngrams.add(normalized.slice(i, i + n));
    }
    return ngrams;
  };

  // Use multiple n-gram sizes for better accuracy
  let totalSimilarity = 0;
  const weights = [0.2, 0.3, 0.3, 0.2];
  const sizes = [2, 3, 4, 5];

  for (let i = 0; i < sizes.length; i++) {
    const n = sizes[i];
    const ngrams1 = getNGrams(text1, n);
    const ngrams2 = getNGrams(text2, n);

    if (ngrams1.size === 0 || ngrams2.size === 0) continue;

    let intersection = 0;
    Array.from(ngrams1).forEach(gram => {
      if (ngrams2.has(gram)) {
        intersection++;
      }
    });

    const union = ngrams1.size + ngrams2.size - intersection;
    const similarity = union > 0 ? intersection / union : 0;
    totalSimilarity += similarity * weights[i];
  }

  return Math.min(1, Math.max(0, totalSimilarity));
}

function calculateKeywordOverlap(text1: string, text2: string): number {
  const extractKeywords = (text: string): Set<string> => {
    return new Set(
      text.toLowerCase()
        .split(/\W+/)
        .filter(word => word.length > 2)
    );
  };

  const keywords1 = extractKeywords(text1);
  const keywords2 = extractKeywords(text2);

  if (keywords1.size === 0 && keywords2.size === 0) return 1.0;
  if (keywords1.size === 0 || keywords2.size === 0) return 0.0;

  let intersection = 0;
  Array.from(keywords1).forEach(keyword => {
    if (keywords2.has(keyword)) {
      intersection++;
    }
  });

  // Use Jaccard similarity
  const union = keywords1.size + keywords2.size - intersection;
  return union > 0 ? intersection / union : 0;
}

function calculateStructuralSimilarity(text1: string, text2: string): number {
  // Placeholder for structural similarity
  // This would analyze JSON/XML structure, bullet points, numbered lists, etc.

  // Simple length-based structural similarity
  const len1 = text1.length;
  const len2 = text2.length;

  if (len1 === 0 && len2 === 0) return 1.0;
  if (len1 === 0 || len2 === 0) return 0.0;

  const lengthRatio = Math.min(len1, len2) / Math.max(len1, len2);

  // Check for similar paragraph structure
  const paragraphs1 = text1.split(/\n\n+/).length;
  const paragraphs2 = text2.split(/\n\n+/).length;
  const paragraphRatio = Math.min(paragraphs1, paragraphs2) / Math.max(paragraphs1, paragraphs2);

  return (lengthRatio * 0.5 + paragraphRatio * 0.5);
}

function classifyMatchType(
  exactMatch: boolean,
  semanticSimilarity: number | null,
  keywordOverlap: number | null,
  structuralSimilarity: number | null,
  config: ValidationConfig
): MatchTypeValue {
  if (exactMatch) {
    return 'exact_match';
  }

  if (semanticSimilarity !== null && semanticSimilarity >= config.semantic_similarity_threshold) {
    return 'semantic_match';
  }

  if (structuralSimilarity !== null && structuralSimilarity >= 0.9) {
    return 'structural_match';
  }

  if (keywordOverlap !== null && keywordOverlap >= config.keyword_overlap_threshold) {
    return 'partial_match';
  }

  if (semanticSimilarity !== null && semanticSimilarity >= 0.5) {
    return 'partial_match';
  }

  return 'no_match';
}

function determineSeverity(
  matchType: MatchTypeValue,
  semanticSimilarity: number | null,
  keywordOverlap: number | null
): ValidationSeverityValue {
  if (matchType === 'exact_match' || matchType === 'semantic_match') {
    return 'pass';
  }

  if (matchType === 'structural_match') {
    return 'pass';
  }

  if (matchType === 'partial_match') {
    return 'warning';
  }

  if (matchType === 'no_match') {
    const similarity = semanticSimilarity ?? keywordOverlap ?? 0;
    if (similarity < 0.2) {
      return 'critical';
    }
    return 'fail';
  }

  if (matchType === 'error') {
    return 'critical';
  }

  return 'fail';
}

function calculateSampleConfidence(
  exactMatch: boolean,
  semanticSimilarity: number | null,
  keywordOverlap: number | null,
  matchType: MatchTypeValue,
  config: ValidationConfig
): number {
  if (exactMatch) {
    return 0.99;
  }

  let confidence = 0.5;

  if (semanticSimilarity !== null) {
    if (semanticSimilarity >= 0.95) {
      confidence = 0.95;
    } else if (semanticSimilarity >= config.semantic_similarity_threshold) {
      confidence = 0.75 + (semanticSimilarity - config.semantic_similarity_threshold) * 0.5;
    } else if (semanticSimilarity >= 0.5) {
      confidence = 0.5 + semanticSimilarity * 0.3;
    } else {
      confidence = 0.4 + semanticSimilarity * 0.2;
    }
  } else if (keywordOverlap !== null) {
    confidence = 0.4 + keywordOverlap * 0.4;
  }

  // Boost for clear results
  if (matchType === 'no_match' || matchType === 'exact_match') {
    confidence = Math.min(1, confidence + 0.1);
  }

  return Math.min(1, Math.max(0, confidence));
}

function calculateDiffSummary(golden: string, model: string): {
  chars_added: number;
  chars_removed: number;
  words_added: number;
  words_removed: number;
  key_differences?: string[];
} {
  const goldenWords = new Set(golden.split(/\s+/));
  const modelWords = new Set(model.split(/\s+/));

  let wordsAdded = 0;
  let wordsRemoved = 0;

  Array.from(modelWords).forEach(word => {
    if (!goldenWords.has(word)) {
      wordsAdded++;
    }
  });

  Array.from(goldenWords).forEach(word => {
    if (!modelWords.has(word)) {
      wordsRemoved++;
    }
  });

  const charsAdded = Math.max(0, model.length - golden.length);
  const charsRemoved = Math.max(0, golden.length - model.length);

  // Identify key differences
  const keyDifferences: string[] = [];
  if (golden.length !== model.length) {
    keyDifferences.push(`Length differs: golden=${golden.length}, model=${model.length}`);
  }
  if (wordsAdded > 5) {
    keyDifferences.push(`${wordsAdded} words added in model output`);
  }
  if (wordsRemoved > 5) {
    keyDifferences.push(`${wordsRemoved} words missing from model output`);
  }

  return {
    chars_added: charsAdded,
    chars_removed: charsRemoved,
    words_added: wordsAdded,
    words_removed: wordsRemoved,
    key_differences: keyDifferences.length > 0 ? keyDifferences : undefined,
  };
}

function generateExplanation(
  matchType: MatchTypeValue,
  exactMatch: boolean,
  semanticSimilarity: number | null,
  keywordOverlap: number | null,
  passed: boolean
): string {
  if (exactMatch) {
    return 'Model output exactly matches the golden reference.';
  }

  const parts: string[] = [];

  if (passed) {
    parts.push('Validation passed.');
  } else {
    parts.push('Validation failed.');
  }

  switch (matchType) {
    case 'semantic_match':
      parts.push(`High semantic similarity (${((semanticSimilarity ?? 0) * 100).toFixed(1)}%) indicates equivalent meaning.`);
      break;
    case 'partial_match':
      parts.push(`Partial match detected with ${((semanticSimilarity ?? keywordOverlap ?? 0) * 100).toFixed(1)}% similarity.`);
      break;
    case 'structural_match':
      parts.push('Structure matches but content differs.');
      break;
    case 'no_match':
      parts.push(`Low similarity (${((semanticSimilarity ?? keywordOverlap ?? 0) * 100).toFixed(1)}%) indicates significant deviation from golden output.`);
      break;
    case 'error':
      parts.push('Error occurred during validation.');
      break;
  }

  if (keywordOverlap !== null) {
    parts.push(`Keyword overlap: ${(keywordOverlap * 100).toFixed(1)}%.`);
  }

  return parts.join(' ');
}

function calculateValidationStats(results: SampleValidationResult[]): ValidationStats {
  const total = results.length;

  if (total === 0) {
    return {
      total_samples: 0,
      passed: 0,
      failed: 0,
      pass_rate: 0,
      exact_matches: 0,
      exact_match_rate: 0,
      semantic_matches: 0,
      semantic_match_rate: 0,
      partial_matches: 0,
      no_matches: 0,
      errors: 0,
      avg_semantic_similarity: 0,
      avg_keyword_overlap: 0,
      avg_confidence: 0,
      by_match_type: {
        exact_match: 0,
        semantic_match: 0,
        partial_match: 0,
        structural_match: 0,
        no_match: 0,
        error: 0,
      },
      by_severity: {
        pass: 0,
        warning: 0,
        fail: 0,
        critical: 0,
      },
    };
  }

  const passed = results.filter(r => r.passed);
  const failed = results.filter(r => !r.passed);

  const byMatchType = {
    exact_match: 0,
    semantic_match: 0,
    partial_match: 0,
    structural_match: 0,
    no_match: 0,
    error: 0,
  };

  const bySeverity = {
    pass: 0,
    warning: 0,
    fail: 0,
    critical: 0,
  };

  const byCategory: Record<string, { total: number; passed: number; pass_rate: number }> = {};

  let totalSemanticSim = 0;
  let semanticCount = 0;
  let totalKeywordOverlap = 0;
  let keywordCount = 0;
  let totalConfidence = 0;

  for (const result of results) {
    byMatchType[result.match_type]++;
    bySeverity[result.severity]++;

    if (result.semantic_similarity !== null) {
      totalSemanticSim += result.semantic_similarity;
      semanticCount++;
    }

    if (result.keyword_overlap !== null) {
      totalKeywordOverlap += result.keyword_overlap;
      keywordCount++;
    }

    totalConfidence += result.confidence;

    if (result.category) {
      if (!byCategory[result.category]) {
        byCategory[result.category] = { total: 0, passed: 0, pass_rate: 0 };
      }
      byCategory[result.category].total++;
      if (result.passed) {
        byCategory[result.category].passed++;
      }
    }
  }

  // Calculate pass rates for categories
  for (const category of Object.keys(byCategory)) {
    const cat = byCategory[category];
    cat.pass_rate = cat.total > 0 ? cat.passed / cat.total : 0;
  }

  return {
    total_samples: total,
    passed: passed.length,
    failed: failed.length,
    pass_rate: passed.length / total,
    exact_matches: byMatchType.exact_match,
    exact_match_rate: byMatchType.exact_match / total,
    semantic_matches: byMatchType.semantic_match,
    semantic_match_rate: byMatchType.semantic_match / total,
    partial_matches: byMatchType.partial_match,
    no_matches: byMatchType.no_match,
    errors: byMatchType.error,
    avg_semantic_similarity: semanticCount > 0 ? totalSemanticSim / semanticCount : 0,
    avg_keyword_overlap: keywordCount > 0 ? totalKeywordOverlap / keywordCount : 0,
    avg_confidence: totalConfidence / total,
    by_match_type: byMatchType,
    by_severity: bySeverity,
    by_category: Object.keys(byCategory).length > 0 ? byCategory : undefined,
  };
}

function calculateQualityAssessment(stats: ValidationStats): {
  grade: 'A' | 'B' | 'C' | 'D' | 'F';
  score: number;
  summary: string;
  recommendations?: string[];
} {
  // Calculate quality score (0-100)
  const passRateWeight = 0.4;
  const exactMatchWeight = 0.3;
  const semanticSimWeight = 0.2;
  const confidenceWeight = 0.1;

  const score = Math.round(
    (stats.pass_rate * passRateWeight +
      stats.exact_match_rate * exactMatchWeight +
      stats.avg_semantic_similarity * semanticSimWeight +
      stats.avg_confidence * confidenceWeight) * 100
  );

  // Determine grade
  let grade: 'A' | 'B' | 'C' | 'D' | 'F';
  if (score >= 90) grade = 'A';
  else if (score >= 80) grade = 'B';
  else if (score >= 70) grade = 'C';
  else if (score >= 60) grade = 'D';
  else grade = 'F';

  // Generate summary
  let summary: string;
  if (grade === 'A') {
    summary = 'Excellent performance. Model outputs closely match golden references.';
  } else if (grade === 'B') {
    summary = 'Good performance. Most outputs match golden references with minor deviations.';
  } else if (grade === 'C') {
    summary = 'Acceptable performance. Some outputs deviate from golden references.';
  } else if (grade === 'D') {
    summary = 'Below average performance. Significant deviations from golden references.';
  } else {
    summary = 'Poor performance. Model outputs frequently fail to match golden references.';
  }

  // Generate recommendations
  const recommendations: string[] = [];
  if (stats.exact_match_rate < 0.5) {
    recommendations.push('Consider fine-tuning to improve exact match rate.');
  }
  if (stats.avg_semantic_similarity < 0.7) {
    recommendations.push('Review prompts to improve semantic alignment with expected outputs.');
  }
  if (stats.no_matches > stats.total_samples * 0.2) {
    recommendations.push('Investigate samples with no matches for systematic issues.');
  }
  if (stats.by_severity.critical > 0) {
    recommendations.push(`Address ${stats.by_severity.critical} critical failures immediately.`);
  }

  return {
    grade,
    score,
    summary,
    recommendations: recommendations.length > 0 ? recommendations : undefined,
  };
}

function extractModelInfo(outputs: GoldenValidatorModelOutput[]): {
  model_id?: string;
  provider?: string;
  output_count: number;
} | undefined {
  if (outputs.length === 0) return undefined;

  // Use the first output's model info as representative
  const first = outputs[0];
  return {
    model_id: first.model_id,
    provider: first.provider,
    output_count: outputs.length,
  };
}

// =============================================================================
// CONFIDENCE CALCULATION
// =============================================================================

export const CONFIDENCE_FACTORS = {
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

function calculateOverallConfidence(output: GoldenDatasetValidatorOutput): number {
  if (output.results.length === 0) return 0;

  const factors: Array<{ factor: string; weight: number; value: number }> = [];

  // Sample size factor (logarithmic, capped at 100 samples)
  const sampleSizeValue = Math.min(1, Math.log10(output.results.length + 1) / 2);
  factors.push({
    factor: 'sample_size',
    weight: CONFIDENCE_FACTORS.sample_size.weight,
    value: sampleSizeValue,
  });

  // Match consistency factor (based on variance in match types)
  const passRate = output.stats.pass_rate;
  const consistencyValue = 1 - 4 * Math.abs(passRate - 0.5) * Math.abs(passRate - 0.5);
  factors.push({
    factor: 'match_consistency',
    weight: CONFIDENCE_FACTORS.match_consistency.weight,
    value: Math.max(0.3, 1 - consistencyValue),
  });

  // Semantic coverage factor
  const semanticCoverageValue = output.stats.avg_semantic_similarity > 0 ? 0.9 : 0.5;
  factors.push({
    factor: 'semantic_coverage',
    weight: CONFIDENCE_FACTORS.semantic_coverage.weight,
    value: semanticCoverageValue,
  });

  // Result clarity factor
  const clarityValue = output.stats.avg_confidence;
  factors.push({
    factor: 'result_clarity',
    weight: CONFIDENCE_FACTORS.result_clarity.weight,
    value: clarityValue,
  });

  // Validation coverage factor
  const errorRate = output.stats.errors / output.stats.total_samples;
  const coverageValue = 1 - errorRate;
  factors.push({
    factor: 'validation_coverage',
    weight: CONFIDENCE_FACTORS.validation_coverage.weight,
    value: coverageValue,
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
  input: GoldenDatasetValidatorInput,
  output: GoldenDatasetValidatorOutput,
  confidence: number,
  context: ExecutionContext
): Promise<DecisionEvent> {
  const inputsHash = await hashInputs(input);

  return {
    agent_id: GOLDEN_DATASET_VALIDATOR_AGENT.agent_id,
    agent_version: GOLDEN_DATASET_VALIDATOR_AGENT.agent_version,
    decision_type: GOLDEN_DATASET_VALIDATOR_AGENT.decision_type,
    decision_id: randomUUID(),
    inputs_hash: inputsHash,
    inputs_summary: {
      golden_sample_count: input.golden_samples.length,
      model_output_count: input.model_outputs.length,
      dataset_name: input.dataset?.name,
    },
    outputs: output,
    confidence,
    confidence_factors: [
      { factor: 'sample_size', weight: 0.20, value: Math.min(1, Math.log10(output.results.length + 1) / 2) },
      { factor: 'semantic_coverage', weight: 0.20, value: output.stats.avg_semantic_similarity > 0 ? 0.9 : 0.5 },
      { factor: 'result_clarity', weight: 0.20, value: output.stats.avg_confidence },
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
  output: GoldenDatasetValidatorOutput,
  decisionId: string
): EdgeFunctionResponse {
  return {
    statusCode: 200,
    headers: {
      'Content-Type': 'application/json',
      'X-Decision-Id': decisionId,
      'X-Agent-Id': GOLDEN_DATASET_VALIDATOR_AGENT.agent_id,
      'X-Agent-Version': GOLDEN_DATASET_VALIDATOR_AGENT.agent_version,
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
      'X-Agent-Id': GOLDEN_DATASET_VALIDATOR_AGENT.agent_id,
      'X-Agent-Version': GOLDEN_DATASET_VALIDATOR_AGENT.agent_version,
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

export { GOLDEN_DATASET_VALIDATOR_AGENT as default };

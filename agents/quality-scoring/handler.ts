/**
 * Quality Scoring Agent - Edge Function Handler
 *
 * AGENT PURPOSE:
 * Compute normalized quality scores for model outputs using deterministic
 * scoring profiles. Produces consistent, reproducible quality assessments
 * based on configurable evaluation criteria.
 *
 * This agent:
 * - Evaluates output quality (YES)
 * - Applies deterministic scoring profiles (YES)
 * - Normalizes scores to 0-1 range (YES)
 * - Aggregates multiple quality dimensions (YES)
 * - Does NOT execute benchmarks (NO)
 * - Does NOT compare models (NO)
 * - Does NOT enforce policies (NO)
 * - Does NOT orchestrate workflows (NO)
 *
 * Deployed as: Google Cloud Edge Function
 * Part of: LLM-Test-Bench unified GCP service
 */

import { randomUUID } from 'crypto';
import {
  // Contracts
  QualityScoringInputSchema,
  QualityScoringOutputSchema,
  QualityScoringDecisionEventSchema,
  DecisionEvent,
  AgentError,
  validateInput,
  hashInputs,
  // Constants
  QUALITY_SCORING_AGENT,
  VALID_SCORING_CONSTRAINTS,
  calculateQualityConfidence,
  // Types
  QualityScoringInput,
  QualityScoringOutput,
  ModelOutput,
  ScoringProfile,
  QualityDimension,
  EvaluationConfig,
  OutputScore,
  DimensionScore,
  ModelQualityStats,
  ScoringSummary,
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

// =============================================================================
// MAIN HANDLER
// =============================================================================

/**
 * Edge Function Handler for Quality Scoring Agent
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
    QUALITY_SCORING_AGENT.agent_id,
    QUALITY_SCORING_AGENT.agent_version,
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
    const inputValidation = validateInput(QualityScoringInputSchema, request.body);
    if (inputValidation.success === false) {
      const validationError = (inputValidation as { success: false; error: AgentError }).error;
      telemetry.emitValidationFailed('input', validationError.message);
      await telemetry.flush();

      return createErrorResponse(400, 'Validation Error', validationError);
    }

    const input = (inputValidation as { success: true; data: QualityScoringInput }).data;

    // Execute quality scoring
    const output = await executeQualityScoring(input, context);

    // Calculate confidence
    const { confidence, factors } = calculateQualityConfidence(output, input.scoring_profile);

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
      outputs_scored: output.scores.length,
      avg_score: output.summary.overall_avg_score,
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
// CORE EXECUTION LOGIC
// =============================================================================

async function executeQualityScoring(
  input: QualityScoringInput,
  context: ExecutionContext
): Promise<QualityScoringOutput> {
  const evaluationConfig: EvaluationConfig = {
    include_dimension_scores: true,
    include_breakdown: true,
    case_sensitive: false,
    normalize_whitespace: true,
    fail_fast_on_threshold: false,
    parallel_evaluation: true,
    ...input.evaluation_config,
  };

  const profile = input.scoring_profile;
  const outputs = input.outputs;

  // Check batch size constraint
  if (outputs.length > 1000) {
    context.constraintsApplied.push('max_outputs_exceeded');
    context.telemetry.emitConstraintApplied(
      'max_outputs_exceeded',
      `Received ${outputs.length} outputs, max is 1000`
    );
  }

  // Score all outputs
  const scores: OutputScore[] = [];

  if (evaluationConfig.parallel_evaluation && outputs.length > 1) {
    // Parallel evaluation
    const scorePromises = outputs.map(output =>
      scoreOutput(output, profile, evaluationConfig, context)
    );
    const results = await Promise.all(scorePromises);
    scores.push(...results);
  } else {
    // Sequential evaluation
    for (const output of outputs) {
      const score = await scoreOutput(output, profile, evaluationConfig, context);
      scores.push(score);

      // Fail-fast if configured
      if (evaluationConfig.fail_fast_on_threshold && !score.overall_passed) {
        context.constraintsApplied.push('threshold_breach_detected');
        context.telemetry.emitConstraintApplied(
          'threshold_breach_detected',
          `Output ${output.output_id} failed threshold check`
        );
        break;
      }
    }
  }

  const completedAt = new Date();

  // Calculate model-level statistics
  const modelStats = calculateModelStats(scores);

  // Calculate summary
  const summary = calculateSummary(scores, modelStats);

  // Build output
  const output: QualityScoringOutput = {
    scoring_id: context.executionId,
    profile_id: profile.profile_id,
    profile_name: profile.name,
    scores,
    model_stats: modelStats,
    summary,
    evaluation_config_used: evaluationConfig,
    started_at: context.startedAt.toISOString(),
    completed_at: completedAt.toISOString(),
    duration_ms: completedAt.getTime() - context.startedAt.getTime(),
  };

  return output;
}

// =============================================================================
// SCORING LOGIC
// =============================================================================

async function scoreOutput(
  output: ModelOutput,
  profile: ScoringProfile,
  config: EvaluationConfig,
  context: ExecutionContext
): Promise<OutputScore> {
  const dimensionScores: DimensionScore[] = [];

  for (const dimension of profile.dimensions) {
    const score = await scoreDimension(output, dimension, config, context);
    dimensionScores.push(score);
  }

  // Calculate composite score based on normalization method
  const compositeScore = calculateCompositeScore(dimensionScores, profile.normalization);

  // Count passed dimensions
  const dimensionsPassed = dimensionScores.filter(d => d.passed).length;

  // Determine overall pass status
  const overallPassed = dimensionScores.every(d => d.passed);

  return {
    output_id: output.output_id,
    provider_name: output.provider_name,
    model_id: output.model_id,
    composite_score: compositeScore,
    dimension_scores: dimensionScores,
    dimensions_passed: dimensionsPassed,
    dimensions_total: dimensionScores.length,
    pass_rate: dimensionsPassed / dimensionScores.length,
    overall_passed: overallPassed,
    scored_at: new Date().toISOString(),
  };
}

async function scoreDimension(
  output: ModelOutput,
  dimension: QualityDimension,
  config: EvaluationConfig,
  context: ExecutionContext
): Promise<DimensionScore> {
  let rawScore: number;
  let details: Record<string, unknown> = {};

  const content = normalizeContent(output.content, config);
  const expected = dimension.expected
    ? normalizeContent(dimension.expected, config)
    : output.expected_output
      ? normalizeContent(output.expected_output, config)
      : undefined;

  try {
    switch (dimension.scoring_method) {
      case 'exact_match':
        rawScore = scoreExactMatch(content, expected);
        details = { matched: rawScore === 1.0 };
        break;

      case 'contains':
        rawScore = scoreContains(content, expected);
        details = { contains: rawScore === 1.0 };
        break;

      case 'regex_match':
        const regexResult = scoreRegexMatch(content, expected);
        rawScore = regexResult.score;
        details = regexResult.details;
        break;

      case 'semantic_similarity':
        // Semantic similarity requires embeddings - may not be available
        try {
          rawScore = await scoreSemanticSimilarity(content, expected);
          details = { method: 'embedding_cosine' };
        } catch {
          context.constraintsApplied.push('semantic_similarity_unavailable');
          context.telemetry.emitConstraintApplied(
            'semantic_similarity_unavailable',
            'Falling back to keyword matching'
          );
          rawScore = scoreContains(content, expected);
          details = { fallback: 'contains' };
        }
        break;

      case 'length_ratio':
        const lengthResult = scoreLengthRatio(content, expected);
        rawScore = lengthResult.score;
        details = lengthResult.details;
        break;

      case 'keyword_presence':
        const keywordResult = scoreKeywordPresence(content, dimension.keywords ?? []);
        rawScore = keywordResult.score;
        details = keywordResult.details;
        break;

      case 'format_compliance':
        const formatResult = scoreFormatCompliance(content, dimension.format_type ?? 'json');
        rawScore = formatResult.score;
        details = formatResult.details;
        break;

      case 'custom_evaluator':
        // Custom evaluator would call external function
        try {
          rawScore = await scoreCustomEvaluator(content, dimension.evaluator_ref, context);
          details = { evaluator: dimension.evaluator_ref };
        } catch (err) {
          context.constraintsApplied.push('custom_evaluator_failed');
          context.telemetry.emitConstraintApplied(
            'custom_evaluator_failed',
            `Evaluator ${dimension.evaluator_ref} failed: ${err}`
          );
          rawScore = 0;
          details = { error: String(err) };
        }
        break;

      default:
        rawScore = 0;
        details = { error: `Unknown scoring method: ${dimension.scoring_method}` };
    }
  } catch (err) {
    rawScore = 0;
    details = { error: String(err) };
  }

  // Apply inversion if configured
  if (dimension.invert) {
    rawScore = 1 - rawScore;
  }

  // Clamp to 0-1 range
  rawScore = Math.max(0, Math.min(1, rawScore));

  const weightedScore = rawScore * dimension.weight;
  const passed = rawScore >= dimension.pass_threshold;

  return {
    dimension_id: dimension.dimension_id,
    name: dimension.name,
    raw_score: rawScore,
    weighted_score: weightedScore,
    weight: dimension.weight,
    passed,
    scoring_method: dimension.scoring_method,
    details,
  };
}

// =============================================================================
// SCORING METHODS
// =============================================================================

function normalizeContent(content: string, config: EvaluationConfig): string {
  let normalized = content;

  if (!config.case_sensitive) {
    normalized = normalized.toLowerCase();
  }

  if (config.normalize_whitespace) {
    normalized = normalized.replace(/\s+/g, ' ').trim();
  }

  return normalized;
}

function scoreExactMatch(content: string, expected?: string): number {
  if (!expected) return 0;
  return content === expected ? 1.0 : 0.0;
}

function scoreContains(content: string, expected?: string): number {
  if (!expected) return 0;
  return content.includes(expected) ? 1.0 : 0.0;
}

function scoreRegexMatch(content: string, pattern?: string): { score: number; details: Record<string, unknown> } {
  if (!pattern) return { score: 0, details: { error: 'No pattern provided' } };

  try {
    const regex = new RegExp(pattern, 'i');
    const matches = content.match(regex);
    return {
      score: matches ? 1.0 : 0.0,
      details: { matched: !!matches, pattern },
    };
  } catch (err) {
    return {
      score: 0,
      details: { error: `Invalid regex: ${err}` },
    };
  }
}

async function scoreSemanticSimilarity(content: string, expected?: string): Promise<number> {
  if (!expected) return 0;

  // This would integrate with an embedding service
  // For now, use a simple word overlap as placeholder
  const contentWords = new Set(content.toLowerCase().split(/\s+/));
  const expectedWordsArray = expected.toLowerCase().split(/\s+/);
  const expectedWords = new Set(expectedWordsArray);

  let intersection = 0;
  expectedWordsArray.forEach(word => {
    if (contentWords.has(word)) intersection++;
  });

  const allWords = [...Array.from(contentWords), ...expectedWordsArray];
  const union = new Set(allWords).size;
  return union > 0 ? intersection / union : 0;
}

function scoreLengthRatio(content: string, expected?: string): { score: number; details: Record<string, unknown> } {
  if (!expected || expected.length === 0) {
    return { score: 0, details: { error: 'No expected content for length comparison' } };
  }

  const ratio = content.length / expected.length;
  // Score is 1.0 when ratio is 1.0, decreases as it deviates
  const score = Math.max(0, 1 - Math.abs(1 - ratio));

  return {
    score,
    details: {
      content_length: content.length,
      expected_length: expected.length,
      ratio,
    },
  };
}

function scoreKeywordPresence(content: string, keywords: string[]): { score: number; details: Record<string, unknown> } {
  if (keywords.length === 0) {
    return { score: 0, details: { error: 'No keywords provided' } };
  }

  const foundKeywords: string[] = [];
  const missingKeywords: string[] = [];

  for (const keyword of keywords) {
    if (content.toLowerCase().includes(keyword.toLowerCase())) {
      foundKeywords.push(keyword);
    } else {
      missingKeywords.push(keyword);
    }
  }

  const score = foundKeywords.length / keywords.length;

  return {
    score,
    details: {
      found: foundKeywords,
      missing: missingKeywords,
      total_keywords: keywords.length,
    },
  };
}

function scoreFormatCompliance(content: string, formatType: string): { score: number; details: Record<string, unknown> } {
  let isValid = false;
  let parseError: string | undefined;

  switch (formatType) {
    case 'json':
      try {
        JSON.parse(content);
        isValid = true;
      } catch (err) {
        parseError = String(err);
      }
      break;

    case 'xml':
      // Simple XML validation (checks for balanced tags)
      isValid = /^<[^>]+>[\s\S]*<\/[^>]+>$/.test(content.trim());
      break;

    case 'yaml':
      // Simple YAML check (key: value pattern)
      isValid = /^[\w-]+:\s*.+/m.test(content);
      break;

    case 'markdown':
      // Check for markdown elements
      isValid = /^#|^\*|\[.+\]\(.+\)|```/.test(content);
      break;

    case 'code':
      // Check for code-like patterns (functions, variables, etc.)
      isValid = /function|const|let|var|class|def |import |from /.test(content);
      break;

    default:
      parseError = `Unknown format type: ${formatType}`;
  }

  return {
    score: isValid ? 1.0 : 0.0,
    details: {
      format_type: formatType,
      is_valid: isValid,
      parse_error: parseError,
    },
  };
}

async function scoreCustomEvaluator(
  content: string,
  evaluatorRef?: string,
  context?: ExecutionContext
): Promise<number> {
  if (!evaluatorRef) {
    throw new Error('No evaluator reference provided');
  }

  // In production, this would call an external evaluator service
  // For now, throw to trigger the constraint
  throw new Error('Custom evaluators not yet implemented');
}

// =============================================================================
// COMPOSITE SCORE CALCULATION
// =============================================================================

function calculateCompositeScore(
  dimensionScores: DimensionScore[],
  normalization: 'weighted_sum' | 'harmonic_mean' | 'geometric_mean'
): number {
  if (dimensionScores.length === 0) return 0;

  const scores = dimensionScores.map(d => d.raw_score);
  const weights = dimensionScores.map(d => d.weight);

  switch (normalization) {
    case 'weighted_sum':
      return dimensionScores.reduce((sum, d) => sum + d.weighted_score, 0);

    case 'harmonic_mean':
      const nonZeroScores = scores.filter(s => s > 0);
      if (nonZeroScores.length === 0) return 0;
      const sumOfReciprocals = nonZeroScores.reduce((sum, s) => sum + (1 / s), 0);
      return nonZeroScores.length / sumOfReciprocals;

    case 'geometric_mean':
      const product = scores.reduce((prod, s, i) => prod * Math.pow(s || 0.001, weights[i]), 1);
      return Math.pow(product, 1);

    default:
      return dimensionScores.reduce((sum, d) => sum + d.weighted_score, 0);
  }
}

// =============================================================================
// STATISTICS CALCULATION
// =============================================================================

function calculateModelStats(scores: OutputScore[]): ModelQualityStats[] {
  // Group by provider/model
  const groups = new Map<string, OutputScore[]>();

  for (const score of scores) {
    const key = `${score.provider_name}:${score.model_id}`;
    const group = groups.get(key) ?? [];
    group.push(score);
    groups.set(key, group);
  }

  // Calculate stats for each group
  const stats: ModelQualityStats[] = [];

  Array.from(groups.entries()).forEach(([key, groupScores]) => {
    const [providerName, modelId] = key.split(':');
    const compositeScores = groupScores.map(s => s.composite_score).sort((a, b) => a - b);

    // Calculate average dimension scores
    const avgDimensionScores: Record<string, number> = {};
    if (groupScores.length > 0 && groupScores[0].dimension_scores) {
      for (const dim of groupScores[0].dimension_scores) {
        const dimScores = groupScores.map(
          s => s.dimension_scores.find(d => d.dimension_id === dim.dimension_id)?.raw_score ?? 0
        );
        avgDimensionScores[dim.dimension_id] = mean(dimScores);
      }
    }

    stats.push({
      provider_name: providerName,
      model_id: modelId,
      outputs_scored: groupScores.length,
      avg_composite_score: mean(compositeScores),
      min_composite_score: Math.min(...compositeScores),
      max_composite_score: Math.max(...compositeScores),
      stddev_composite_score: stddev(compositeScores),
      p50_composite_score: percentile(compositeScores, 50),
      p95_composite_score: percentile(compositeScores, 95),
      p99_composite_score: percentile(compositeScores, 99),
      outputs_passed: groupScores.filter(s => s.overall_passed).length,
      overall_pass_rate: groupScores.filter(s => s.overall_passed).length / groupScores.length,
      avg_dimension_scores: avgDimensionScores,
    });
  });

  return stats;
}

function calculateSummary(scores: OutputScore[], modelStats: ModelQualityStats[]): ScoringSummary {
  const compositeScores = scores.map(s => s.composite_score);

  // Find best model
  let bestModel: { provider_name: string; model_id: string; avg_score: number } | null = null;
  if (modelStats.length > 0) {
    const sorted = [...modelStats].sort((a, b) => b.avg_composite_score - a.avg_composite_score);
    bestModel = {
      provider_name: sorted[0].provider_name,
      model_id: sorted[0].model_id,
      avg_score: sorted[0].avg_composite_score,
    };
  }

  // Calculate score distribution
  const scoreDistribution = {
    excellent: compositeScores.filter(s => s >= 0.9).length,
    good: compositeScores.filter(s => s >= 0.7 && s < 0.9).length,
    fair: compositeScores.filter(s => s >= 0.5 && s < 0.7).length,
    poor: compositeScores.filter(s => s >= 0.3 && s < 0.5).length,
    failed: compositeScores.filter(s => s < 0.3).length,
  };

  return {
    total_outputs_scored: scores.length,
    total_models_evaluated: modelStats.length,
    overall_avg_score: mean(compositeScores),
    overall_pass_rate: scores.filter(s => s.overall_passed).length / scores.length,
    best_model: bestModel,
    score_distribution: scoreDistribution,
  };
}

// =============================================================================
// STATISTICAL HELPERS
// =============================================================================

function percentile(sorted: number[], p: number): number {
  if (sorted.length === 0) return 0;
  const index = Math.ceil((p / 100) * sorted.length) - 1;
  return sorted[Math.max(0, index)];
}

function mean(values: number[]): number {
  if (values.length === 0) return 0;
  return values.reduce((a, b) => a + b, 0) / values.length;
}

function stddev(values: number[]): number {
  if (values.length === 0) return 0;
  const avg = mean(values);
  const squareDiffs = values.map(v => Math.pow(v - avg, 2));
  return Math.sqrt(mean(squareDiffs));
}

// =============================================================================
// DECISION EVENT CREATION
// =============================================================================

async function createDecisionEvent(
  input: QualityScoringInput,
  output: QualityScoringOutput,
  confidence: number,
  factors: Array<{ factor: string; weight: number; value: number }>,
  context: ExecutionContext
): Promise<DecisionEvent> {
  const inputsHash = await hashInputs(input);

  return {
    agent_id: QUALITY_SCORING_AGENT.agent_id,
    agent_version: QUALITY_SCORING_AGENT.agent_version,
    decision_type: QUALITY_SCORING_AGENT.decision_type,
    decision_id: randomUUID(),
    inputs_hash: inputsHash,
    inputs_summary: {
      profile_id: input.scoring_profile.profile_id,
      output_count: input.outputs.length,
      dimension_count: input.scoring_profile.dimensions.length,
    },
    outputs: output,
    confidence,
    confidence_factors: factors,
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
  output: QualityScoringOutput,
  decisionId: string
): EdgeFunctionResponse {
  return {
    statusCode: 200,
    headers: {
      'Content-Type': 'application/json',
      'X-Decision-Id': decisionId,
      'X-Agent-Id': QUALITY_SCORING_AGENT.agent_id,
      'X-Agent-Version': QUALITY_SCORING_AGENT.agent_version,
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
      'X-Agent-Id': QUALITY_SCORING_AGENT.agent_id,
      'X-Agent-Version': QUALITY_SCORING_AGENT.agent_version,
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

export { QUALITY_SCORING_AGENT };

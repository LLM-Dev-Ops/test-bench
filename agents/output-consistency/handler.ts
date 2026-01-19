/**
 * Output Consistency Agent - Edge Function Handler
 *
 * AGENT PURPOSE:
 * Measure consistency across repeated executions of identical prompts.
 * Produces deterministic consistency metrics by analyzing output variations
 * when the same prompt is executed multiple times against the same model.
 *
 * This agent:
 * - Measures output consistency (YES)
 * - Calculates similarity/variance metrics (YES)
 * - Detects semantic drift across executions (YES)
 * - Aggregates consistency scores by model/prompt (YES)
 * - Does NOT execute prompts (NO)
 * - Does NOT compare different models (NO)
 * - Does NOT enforce policies (NO)
 * - Does NOT orchestrate workflows (NO)
 *
 * Deployed as: Google Cloud Edge Function
 * Part of: LLM-Test-Bench unified GCP service
 */

import { randomUUID } from 'crypto';
import {
  // Contracts
  OutputConsistencyInputSchema,
  OutputConsistencyOutputSchema,
  OutputConsistencyDecisionEventSchema,
  DecisionEvent,
  AgentError,
  validateInput,
  hashInputs,
  // Constants
  OUTPUT_CONSISTENCY_AGENT,
  VALID_CONSISTENCY_CONSTRAINTS,
  calculateConsistencyConfidence,
  // Types
  OutputConsistencyInput,
  OutputConsistencyOutput,
  ConsistencyConfig,
  PromptExecutionGroup,
  ExecutionOutput,
  GroupConsistencyResult,
  ModelConsistencyStats,
  ConsistencySummary,
  TokenAnalysis,
  CharVariance,
  SimilarityScores,
  SimilarityMethod,
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
 * Edge Function Handler for Output Consistency Agent
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
    OUTPUT_CONSISTENCY_AGENT.agent_id,
    OUTPUT_CONSISTENCY_AGENT.agent_version,
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
    const inputValidation = validateInput(OutputConsistencyInputSchema, request.body);
    if (inputValidation.success === false) {
      const validationError = (inputValidation as { success: false; error: AgentError }).error;
      telemetry.emitValidationFailed('input', validationError.message);
      await telemetry.flush();

      return createErrorResponse(400, 'Validation Error', validationError);
    }

    const input = (inputValidation as { success: true; data: OutputConsistencyInput }).data;

    // Execute consistency analysis
    const output = await executeConsistencyAnalysis(input, context);

    // Calculate confidence
    const config: ConsistencyConfig = {
      similarity_method: 'jaccard_tokens',
      ngram_size: 3,
      consistency_threshold: 0.85,
      normalize_whitespace: true,
      case_sensitive: false,
      trim_content: true,
      include_token_analysis: true,
      include_char_variance: false,
      compute_pairwise_matrix: false,
      ...input.config,
    };
    const { confidence, factors } = calculateConsistencyConfidence(output, config);

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
      groups_analyzed: output.results.length,
      avg_consistency: output.summary.overall_avg_consistency,
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

async function executeConsistencyAnalysis(
  input: OutputConsistencyInput,
  context: ExecutionContext
): Promise<OutputConsistencyOutput> {
  const config: ConsistencyConfig = {
    similarity_method: 'jaccard_tokens',
    ngram_size: 3,
    consistency_threshold: 0.85,
    normalize_whitespace: true,
    case_sensitive: false,
    trim_content: true,
    include_token_analysis: true,
    include_char_variance: false,
    compute_pairwise_matrix: false,
    ...input.config,
  };

  const groups = input.execution_groups;

  // Check batch size constraint
  if (groups.length > 500) {
    context.constraintsApplied.push('max_groups_exceeded');
    context.telemetry.emitConstraintApplied(
      'max_groups_exceeded',
      `Received ${groups.length} groups, max is 500`
    );
  }

  // Analyze all groups
  const results: GroupConsistencyResult[] = [];

  for (const group of groups) {
    const result = await analyzeGroup(group, config, context);
    results.push(result);
  }

  const completedAt = new Date();

  // Calculate model-level statistics
  const modelStats = calculateModelStats(results);

  // Calculate summary
  const summary = calculateSummary(results, modelStats, config);

  // Build output
  const output: OutputConsistencyOutput = {
    analysis_id: context.executionId,
    results,
    model_stats: modelStats,
    summary,
    config_used: config,
    started_at: context.startedAt.toISOString(),
    completed_at: completedAt.toISOString(),
    duration_ms: completedAt.getTime() - context.startedAt.getTime(),
  };

  return output;
}

// =============================================================================
// GROUP ANALYSIS
// =============================================================================

async function analyzeGroup(
  group: PromptExecutionGroup,
  config: ConsistencyConfig,
  context: ExecutionContext
): Promise<GroupConsistencyResult> {
  const outputs = group.outputs;

  // Normalize all outputs
  const normalizedOutputs = outputs.map(o => normalizeContent(o.content, config));

  // Check for identical outputs
  const allIdentical = normalizedOutputs.every(o => o === normalizedOutputs[0]);
  if (allIdentical) {
    context.constraintsApplied.push('identical_outputs_detected');
    context.telemetry.emitConstraintApplied(
      'identical_outputs_detected',
      `All ${outputs.length} outputs in group ${group.group_id} are identical`
    );
  }

  // Check for very short outputs
  const avgLength = normalizedOutputs.reduce((sum, o) => sum + o.length, 0) / normalizedOutputs.length;
  if (avgLength < 10) {
    context.constraintsApplied.push('outputs_too_short');
    context.telemetry.emitConstraintApplied(
      'outputs_too_short',
      `Average output length is ${avgLength.toFixed(1)} characters`
    );
  }

  // Compute primary similarity score
  const primaryScore = computeSimilarity(normalizedOutputs, config.similarity_method, config);

  // Compute additional method scores if requested
  let additionalScores: Record<string, number> | undefined;
  if (config.additional_methods && config.additional_methods.length > 0) {
    additionalScores = {};
    for (const method of config.additional_methods) {
      if (method !== config.similarity_method) {
        additionalScores[method] = computeSimilarity(normalizedOutputs, method, config);
      }
    }
  }

  const similarityScores: SimilarityScores = {
    primary_score: primaryScore,
    primary_method: config.similarity_method,
    additional_scores: additionalScores,
  };

  // Token analysis
  let tokenAnalysis: TokenAnalysis | undefined;
  if (config.include_token_analysis) {
    tokenAnalysis = computeTokenAnalysis(normalizedOutputs);
  }

  // Character variance
  let charVariance: CharVariance | undefined;
  if (config.include_char_variance) {
    charVariance = computeCharVariance(normalizedOutputs);
  }

  // Pairwise similarity matrix
  let pairwiseSimilarities: number[] | undefined;
  if (config.compute_pairwise_matrix && normalizedOutputs.length <= 50) {
    pairwiseSimilarities = computePairwiseMatrix(normalizedOutputs, config);
  } else if (config.compute_pairwise_matrix && normalizedOutputs.length > 50) {
    context.constraintsApplied.push('pairwise_matrix_too_large');
    context.telemetry.emitConstraintApplied(
      'pairwise_matrix_too_large',
      `${normalizedOutputs.length} outputs would produce ${(normalizedOutputs.length * (normalizedOutputs.length - 1)) / 2} pairs`
    );
  }

  // Find representative and most divergent outputs
  const { representativeIndex, divergentIndex, maxDivergence } = findOutliers(
    normalizedOutputs,
    config
  );

  // Compute prompt hash
  const promptHash = await computePromptHash(group.prompt);

  return {
    group_id: group.group_id,
    provider_name: group.provider_name,
    model_id: group.model_id,
    output_count: outputs.length,
    consistency_score: primaryScore,
    is_consistent: primaryScore >= config.consistency_threshold,
    similarity_scores: similarityScores,
    token_analysis: tokenAnalysis,
    char_variance: charVariance,
    pairwise_similarities: pairwiseSimilarities,
    representative_output_index: representativeIndex,
    most_divergent_output_index: divergentIndex,
    max_divergence_score: maxDivergence,
    prompt_hash: promptHash,
    analyzed_at: new Date().toISOString(),
  };
}

// =============================================================================
// SIMILARITY COMPUTATION
// =============================================================================

function normalizeContent(content: string, config: ConsistencyConfig): string {
  let normalized = content;

  if (config.trim_content) {
    normalized = normalized.trim();
  }

  if (!config.case_sensitive) {
    normalized = normalized.toLowerCase();
  }

  if (config.normalize_whitespace) {
    normalized = normalized.replace(/\s+/g, ' ');
  }

  return normalized;
}

function computeSimilarity(
  outputs: string[],
  method: SimilarityMethod,
  config: ConsistencyConfig
): number {
  if (outputs.length < 2) return 1.0;

  switch (method) {
    case 'exact_match':
      return computeExactMatchSimilarity(outputs);

    case 'normalized_levenshtein':
      return computeLevenshteinSimilarity(outputs);

    case 'jaccard_tokens':
      return computeJaccardTokenSimilarity(outputs);

    case 'character_ngram':
      return computeNgramSimilarity(outputs, config.ngram_size, 'char');

    case 'word_ngram':
      return computeNgramSimilarity(outputs, config.ngram_size, 'word');

    case 'cosine_tfidf':
      return computeTfidfSimilarity(outputs);

    case 'semantic_embedding':
      // Would require embedding service - fall back to Jaccard
      return computeJaccardTokenSimilarity(outputs);

    default:
      return computeJaccardTokenSimilarity(outputs);
  }
}

function computeExactMatchSimilarity(outputs: string[]): number {
  const first = outputs[0];
  const allMatch = outputs.every(o => o === first);
  return allMatch ? 1.0 : 0.0;
}

function computeLevenshteinSimilarity(outputs: string[]): number {
  // Compute average pairwise Levenshtein similarity
  let totalSimilarity = 0;
  let pairs = 0;

  for (let i = 0; i < outputs.length; i++) {
    for (let j = i + 1; j < outputs.length; j++) {
      const distance = levenshteinDistance(outputs[i], outputs[j]);
      const maxLen = Math.max(outputs[i].length, outputs[j].length);
      const similarity = maxLen > 0 ? 1 - distance / maxLen : 1;
      totalSimilarity += similarity;
      pairs++;
    }
  }

  return pairs > 0 ? totalSimilarity / pairs : 1.0;
}

function levenshteinDistance(a: string, b: string): number {
  const m = a.length;
  const n = b.length;

  // Early exit for empty strings
  if (m === 0) return n;
  if (n === 0) return m;

  // Use two rows for space efficiency
  let prev = Array.from({ length: n + 1 }, (_, i) => i);
  let curr = new Array(n + 1);

  for (let i = 1; i <= m; i++) {
    curr[0] = i;
    for (let j = 1; j <= n; j++) {
      const cost = a[i - 1] === b[j - 1] ? 0 : 1;
      curr[j] = Math.min(
        prev[j] + 1,      // deletion
        curr[j - 1] + 1,  // insertion
        prev[j - 1] + cost // substitution
      );
    }
    [prev, curr] = [curr, prev];
  }

  return prev[n];
}

function computeJaccardTokenSimilarity(outputs: string[]): number {
  // Tokenize all outputs
  const tokenSets = outputs.map(o => new Set(o.split(/\s+/).filter(t => t.length > 0)));

  // Compute intersection and union across all sets
  const allTokens = new Set<string>();
  const tokenCounts = new Map<string, number>();

  for (const tokenSet of tokenSets) {
    Array.from(tokenSet).forEach(token => {
      allTokens.add(token);
      tokenCounts.set(token, (tokenCounts.get(token) ?? 0) + 1);
    });
  }

  // Intersection: tokens present in ALL outputs
  let intersection = 0;
  Array.from(tokenCounts.entries()).forEach(([token, count]) => {
    if (count === outputs.length) {
      intersection++;
    }
  });

  const union = allTokens.size;
  return union > 0 ? intersection / union : 1.0;
}

function computeNgramSimilarity(
  outputs: string[],
  n: number,
  type: 'char' | 'word'
): number {
  // Generate n-grams for each output
  const ngramSets = outputs.map(o => new Set(generateNgrams(o, n, type)));

  // Compute average pairwise Jaccard
  let totalSimilarity = 0;
  let pairs = 0;

  for (let i = 0; i < ngramSets.length; i++) {
    for (let j = i + 1; j < ngramSets.length; j++) {
      const setI = Array.from(ngramSets[i]);
      const setJ = Array.from(ngramSets[j]);
      const intersection = new Set(setI.filter(x => ngramSets[j].has(x)));
      const unionArr = [...setI, ...setJ];
      const union = new Set(unionArr);
      const similarity = union.size > 0 ? intersection.size / union.size : 1;
      totalSimilarity += similarity;
      pairs++;
    }
  }

  return pairs > 0 ? totalSimilarity / pairs : 1.0;
}

function generateNgrams(text: string, n: number, type: 'char' | 'word'): string[] {
  const ngrams: string[] = [];

  if (type === 'char') {
    for (let i = 0; i <= text.length - n; i++) {
      ngrams.push(text.slice(i, i + n));
    }
  } else {
    const words = text.split(/\s+/).filter(w => w.length > 0);
    for (let i = 0; i <= words.length - n; i++) {
      ngrams.push(words.slice(i, i + n).join(' '));
    }
  }

  return ngrams;
}

function computeTfidfSimilarity(outputs: string[]): number {
  // Simple TF-IDF cosine similarity
  const documents = outputs.map(o => o.split(/\s+/).filter(t => t.length > 0));

  // Build vocabulary
  const vocab = new Set<string>();
  for (const doc of documents) {
    for (const term of doc) {
      vocab.add(term);
    }
  }
  const vocabArray = Array.from(vocab);

  // Compute TF-IDF vectors
  const vectors: number[][] = [];
  const docCount = documents.length;

  for (const doc of documents) {
    const tf = new Map<string, number>();
    for (const term of doc) {
      tf.set(term, (tf.get(term) ?? 0) + 1);
    }

    const vector: number[] = [];
    for (const term of vocabArray) {
      const termFreq = (tf.get(term) ?? 0) / doc.length;
      const docsWithTerm = documents.filter(d => d.includes(term)).length;
      const idf = Math.log((docCount + 1) / (docsWithTerm + 1)) + 1;
      vector.push(termFreq * idf);
    }
    vectors.push(vector);
  }

  // Compute average pairwise cosine similarity
  let totalSimilarity = 0;
  let pairs = 0;

  for (let i = 0; i < vectors.length; i++) {
    for (let j = i + 1; j < vectors.length; j++) {
      const similarity = cosineSimilarity(vectors[i], vectors[j]);
      totalSimilarity += similarity;
      pairs++;
    }
  }

  return pairs > 0 ? totalSimilarity / pairs : 1.0;
}

function cosineSimilarity(a: number[], b: number[]): number {
  let dotProduct = 0;
  let normA = 0;
  let normB = 0;

  for (let i = 0; i < a.length; i++) {
    dotProduct += a[i] * b[i];
    normA += a[i] * a[i];
    normB += b[i] * b[i];
  }

  const denominator = Math.sqrt(normA) * Math.sqrt(normB);
  return denominator > 0 ? dotProduct / denominator : 0;
}

// =============================================================================
// TOKEN AND CHARACTER ANALYSIS
// =============================================================================

function computeTokenAnalysis(outputs: string[]): TokenAnalysis {
  const tokenCounts = outputs.map(o => o.split(/\s+/).filter(t => t.length > 0).length);
  const tokenSets = outputs.map(o => new Set(o.split(/\s+/).filter(t => t.length > 0)));

  // Common tokens (intersection)
  const allTokenSets = tokenSets.slice();
  let commonTokens = new Set(Array.from(allTokenSets[0]));
  for (let i = 1; i < allTokenSets.length; i++) {
    commonTokens = new Set(Array.from(commonTokens).filter(x => allTokenSets[i].has(x)));
  }

  // Total unique tokens (union)
  const totalUniqueTokens = new Set<string>();
  for (const tokenSet of tokenSets) {
    Array.from(tokenSet).forEach(token => {
      totalUniqueTokens.add(token);
    });
  }

  const avg = mean(tokenCounts);
  const std = stddev(tokenCounts);

  return {
    avg_token_count: avg,
    min_token_count: Math.min(...tokenCounts),
    max_token_count: Math.max(...tokenCounts),
    stddev_token_count: std,
    cv_token_count: avg > 0 ? std / avg : 0,
    common_token_count: commonTokens.size,
    total_unique_tokens: totalUniqueTokens.size,
  };
}

function computeCharVariance(outputs: string[]): CharVariance {
  const charCounts = outputs.map(o => o.length);
  const avg = mean(charCounts);
  const std = stddev(charCounts);

  // Compute edit distances
  const editDistances: number[] = [];
  for (let i = 0; i < outputs.length; i++) {
    for (let j = i + 1; j < outputs.length; j++) {
      editDistances.push(levenshteinDistance(outputs[i], outputs[j]));
    }
  }

  return {
    avg_char_count: avg,
    stddev_char_count: std,
    cv_char_count: avg > 0 ? std / avg : 0,
    avg_edit_distance: editDistances.length > 0 ? mean(editDistances) : 0,
    max_edit_distance: editDistances.length > 0 ? Math.max(...editDistances) : 0,
  };
}

function computePairwiseMatrix(outputs: string[], config: ConsistencyConfig): number[] {
  const similarities: number[] = [];

  for (let i = 0; i < outputs.length; i++) {
    for (let j = i + 1; j < outputs.length; j++) {
      const similarity = computeSimilarity([outputs[i], outputs[j]], config.similarity_method, config);
      similarities.push(similarity);
    }
  }

  return similarities;
}

function findOutliers(
  outputs: string[],
  config: ConsistencyConfig
): { representativeIndex: number; divergentIndex: number; maxDivergence: number } {
  if (outputs.length < 2) {
    return { representativeIndex: 0, divergentIndex: 0, maxDivergence: 0 };
  }

  // Compute average similarity of each output to all others
  const avgSimilarities: number[] = [];

  for (let i = 0; i < outputs.length; i++) {
    let totalSimilarity = 0;
    for (let j = 0; j < outputs.length; j++) {
      if (i !== j) {
        totalSimilarity += computeSimilarity([outputs[i], outputs[j]], config.similarity_method, config);
      }
    }
    avgSimilarities.push(totalSimilarity / (outputs.length - 1));
  }

  // Representative: highest average similarity (closest to centroid)
  let representativeIndex = 0;
  let maxAvgSimilarity = avgSimilarities[0];
  for (let i = 1; i < avgSimilarities.length; i++) {
    if (avgSimilarities[i] > maxAvgSimilarity) {
      maxAvgSimilarity = avgSimilarities[i];
      representativeIndex = i;
    }
  }

  // Most divergent: lowest average similarity
  let divergentIndex = 0;
  let minAvgSimilarity = avgSimilarities[0];
  for (let i = 1; i < avgSimilarities.length; i++) {
    if (avgSimilarities[i] < minAvgSimilarity) {
      minAvgSimilarity = avgSimilarities[i];
      divergentIndex = i;
    }
  }

  return {
    representativeIndex,
    divergentIndex,
    maxDivergence: 1 - minAvgSimilarity,
  };
}

async function computePromptHash(prompt: string): Promise<string> {
  const encoder = new TextEncoder();
  const data = encoder.encode(prompt);
  const hashBuffer = await crypto.subtle.digest('SHA-256', data);
  const hashArray = Array.from(new Uint8Array(hashBuffer));
  return hashArray.map(b => b.toString(16).padStart(2, '0')).join('').slice(0, 16);
}

// =============================================================================
// STATISTICS CALCULATION
// =============================================================================

function calculateModelStats(results: GroupConsistencyResult[]): ModelConsistencyStats[] {
  // Group by provider/model
  const groups = new Map<string, GroupConsistencyResult[]>();

  for (const result of results) {
    const key = `${result.provider_name}:${result.model_id}`;
    const group = groups.get(key) ?? [];
    group.push(result);
    groups.set(key, group);
  }

  // Calculate stats for each group
  const stats: ModelConsistencyStats[] = [];

  Array.from(groups.entries()).forEach(([key, groupResults]) => {
    const [providerName, modelId] = key.split(':');
    const consistencyScores = groupResults.map(r => r.consistency_score).sort((a, b) => a - b);
    const totalOutputs = groupResults.reduce((sum, r) => sum + r.output_count, 0);

    // Token variance
    let avgTokenVariance: number | undefined;
    const tokenVariances = groupResults
      .filter(r => r.token_analysis)
      .map(r => r.token_analysis!.cv_token_count);
    if (tokenVariances.length > 0) {
      avgTokenVariance = mean(tokenVariances);
    }

    stats.push({
      provider_name: providerName,
      model_id: modelId,
      groups_analyzed: groupResults.length,
      outputs_analyzed: totalOutputs,
      avg_consistency_score: mean(consistencyScores),
      min_consistency_score: Math.min(...consistencyScores),
      max_consistency_score: Math.max(...consistencyScores),
      stddev_consistency_score: stddev(consistencyScores),
      p50_consistency_score: percentile(consistencyScores, 50),
      p95_consistency_score: percentile(consistencyScores, 95),
      p99_consistency_score: percentile(consistencyScores, 99),
      consistent_groups: groupResults.filter(r => r.is_consistent).length,
      consistency_rate: groupResults.filter(r => r.is_consistent).length / groupResults.length,
      avg_token_variance: avgTokenVariance,
    });
  });

  return stats;
}

function calculateSummary(
  results: GroupConsistencyResult[],
  modelStats: ModelConsistencyStats[],
  config: ConsistencyConfig
): ConsistencySummary {
  const consistencyScores = results.map(r => r.consistency_score);
  const totalOutputs = results.reduce((sum, r) => sum + r.output_count, 0);

  // Find most/least consistent models
  let mostConsistentModel: { provider_name: string; model_id: string; avg_score: number } | null = null;
  let leastConsistentModel: { provider_name: string; model_id: string; avg_score: number } | null = null;

  if (modelStats.length > 0) {
    const sorted = [...modelStats].sort((a, b) => b.avg_consistency_score - a.avg_consistency_score);
    mostConsistentModel = {
      provider_name: sorted[0].provider_name,
      model_id: sorted[0].model_id,
      avg_score: sorted[0].avg_consistency_score,
    };
    leastConsistentModel = {
      provider_name: sorted[sorted.length - 1].provider_name,
      model_id: sorted[sorted.length - 1].model_id,
      avg_score: sorted[sorted.length - 1].avg_consistency_score,
    };
  }

  // Calculate distribution
  const consistencyDistribution = {
    highly_consistent: consistencyScores.filter(s => s >= 0.95).length,
    consistent: consistencyScores.filter(s => s >= 0.85 && s < 0.95).length,
    moderate: consistencyScores.filter(s => s >= 0.70 && s < 0.85).length,
    inconsistent: consistencyScores.filter(s => s >= 0.50 && s < 0.70).length,
    highly_inconsistent: consistencyScores.filter(s => s < 0.50).length,
  };

  return {
    total_groups_analyzed: results.length,
    total_outputs_analyzed: totalOutputs,
    total_models_evaluated: modelStats.length,
    overall_avg_consistency: mean(consistencyScores),
    overall_consistency_rate: results.filter(r => r.is_consistent).length / results.length,
    most_consistent_model: mostConsistentModel,
    least_consistent_model: leastConsistentModel,
    consistency_distribution: consistencyDistribution,
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
  input: OutputConsistencyInput,
  output: OutputConsistencyOutput,
  confidence: number,
  factors: Array<{ factor: string; weight: number; value: number }>,
  context: ExecutionContext
): Promise<DecisionEvent> {
  const inputsHash = await hashInputs(input);

  return {
    agent_id: OUTPUT_CONSISTENCY_AGENT.agent_id,
    agent_version: OUTPUT_CONSISTENCY_AGENT.agent_version,
    decision_type: OUTPUT_CONSISTENCY_AGENT.decision_type,
    decision_id: randomUUID(),
    inputs_hash: inputsHash,
    inputs_summary: {
      groups_count: input.execution_groups.length,
      total_outputs: input.execution_groups.reduce((sum, g) => sum + g.outputs.length, 0),
      similarity_method: output.config_used.similarity_method,
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
  output: OutputConsistencyOutput,
  decisionId: string
): EdgeFunctionResponse {
  return {
    statusCode: 200,
    headers: {
      'Content-Type': 'application/json',
      'X-Decision-Id': decisionId,
      'X-Agent-Id': OUTPUT_CONSISTENCY_AGENT.agent_id,
      'X-Agent-Version': OUTPUT_CONSISTENCY_AGENT.agent_version,
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
      'X-Agent-Id': OUTPUT_CONSISTENCY_AGENT.agent_id,
      'X-Agent-Version': OUTPUT_CONSISTENCY_AGENT.agent_version,
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

export { OUTPUT_CONSISTENCY_AGENT };

/**
 * Prompt Sensitivity Agent - Edge Function Handler
 *
 * AGENT PURPOSE:
 * Analyze how sensitive an LLM is to prompt variations by testing different
 * perturbations of a base prompt and measuring response variance.
 *
 * This agent:
 * - Generates/uses prompt perturbations (YES)
 * - Measures response variance (YES)
 * - Does NOT compare models (NO)
 * - Does NOT optimize prompts (NO)
 *
 * Deployed as: Google Cloud Edge Function
 * Part of: LLM-Test-Bench unified GCP service
 */

import { randomUUID } from 'crypto';
import {
  // Contracts
  PromptSensitivityInputSchema,
  PromptSensitivityOutputSchema,
  PromptSensitivityDecisionEventSchema,
  DecisionEvent,
  AgentError,
  validateInput,
  hashInputs,
  // Constants
  PROMPT_SENSITIVITY_AGENT,
  PROMPT_SENSITIVITY_VALID_CONSTRAINTS,
  calculateSensitivityConfidence,
  // Types
  PromptSensitivityInput,
  PromptSensitivityOutput,
  SensitivityProviderConfig,
  PerturbationType,
  PerturbationResult,
  PerturbationRun,
  PerturbationAggregateMetrics,
  OverallSensitivity,
  SensitivityTokenUsage,
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
 * Edge Function Handler for Prompt Sensitivity Agent
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
    PROMPT_SENSITIVITY_AGENT.agent_id,
    PROMPT_SENSITIVITY_AGENT.agent_version,
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
    const inputValidation = validateInput(PromptSensitivityInputSchema, request.body);
    if (!inputValidation.success) {
      const validationError = (inputValidation as { success: false; error: AgentError }).error;
      telemetry.emitValidationFailed('input', validationError.message);
      await telemetry.flush();

      return createErrorResponse(400, 'Validation Error', validationError);
    }

    const input = (inputValidation as { success: true; data: PromptSensitivityInput }).data;

    // Execute sensitivity analysis
    const output = await executeSensitivityAnalysis(input, context);

    // Calculate confidence
    const overallConfidence = calculateSensitivityConfidence(output);

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
      success_count: output.successful_runs,
      failure_count: output.failed_runs,
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

async function executeSensitivityAnalysis(
  input: PromptSensitivityInput,
  context: ExecutionContext
): Promise<PromptSensitivityOutput> {
  const executionConfig = input.execution_config ?? {
    concurrency: 5,
    timeout_ms: 300000,
    save_responses: true,
  };

  const samplingConfig = input.sampling_config;
  const perturbationConfig = input.perturbation_config;
  const analysisConfig = input.analysis_config ?? {
    similarity_metric: 'cosine' as const,
    compute_embeddings: true,
    statistical_tests: ['variance' as const, 'anova' as const],
  };

  const startTime = context.startedAt;
  const warnings: string[] = [];

  // Generate perturbations
  const perturbations = await generatePerturbations(
    input.base_prompt,
    perturbationConfig,
    context
  );

  if (perturbations.length === 0) {
    context.constraintsApplied.push('perturbation_generation_failed');
    context.telemetry.emitConstraintApplied(
      'perturbation_generation_failed',
      'No perturbations could be generated'
    );
    warnings.push('No perturbations generated, using base prompt only');
  }

  // Execute baseline runs (multiple runs of base prompt)
  const baseResponseSamples: string[] = [];
  for (let i = 0; i < samplingConfig.runs_per_perturbation; i++) {
    // Check timeout
    if (executionConfig.timeout_ms) {
      const elapsed = Date.now() - startTime.getTime();
      if (elapsed >= executionConfig.timeout_ms) {
        context.constraintsApplied.push('max_duration_exceeded');
        context.telemetry.emitConstraintApplied(
          'max_duration_exceeded',
          `Elapsed: ${elapsed}ms, Max: ${executionConfig.timeout_ms}ms`
        );
        break;
      }
    }

    const result = await executePrompt(
      input.provider,
      input.base_prompt,
      samplingConfig
    );

    if (result.success && result.content) {
      baseResponseSamples.push(result.content);
    }
  }

  // Execute perturbations
  const perturbationResults: PerturbationResult[] = [];
  let totalRuns = baseResponseSamples.length;
  let successfulRuns = baseResponseSamples.length;
  let failedRuns = 0;

  for (const perturbation of perturbations) {
    // Check timeout
    if (executionConfig.timeout_ms) {
      const elapsed = Date.now() - startTime.getTime();
      if (elapsed >= executionConfig.timeout_ms) {
        context.constraintsApplied.push('max_duration_exceeded');
        context.telemetry.emitConstraintApplied(
          'max_duration_exceeded',
          `Stopped at perturbation ${perturbationResults.length + 1}/${perturbations.length}`
        );
        break;
      }
    }

    const runs: PerturbationRun[] = [];

    // Execute multiple runs per perturbation
    for (let i = 0; i < samplingConfig.runs_per_perturbation; i++) {
      totalRuns++;

      const result = await executePrompt(
        input.provider,
        perturbation.prompt,
        samplingConfig
      );

      const run: PerturbationRun = {
        run_id: randomUUID(),
        response: result.content ?? '',
        latency_ms: result.latency_ms,
        tokens_used: result.tokens_used ?? { prompt_tokens: 0, completion_tokens: 0, total_tokens: 0 },
        success: result.success,
        error_message: result.error_message,
      };

      runs.push(run);

      if (result.success) {
        successfulRuns++;
      } else {
        failedRuns++;
      }
    }

    // Calculate aggregate metrics for this perturbation
    const aggregateMetrics = calculateAggregateMetrics(runs, baseResponseSamples);

    perturbationResults.push({
      type: perturbation.type,
      perturbation_id: perturbation.id,
      perturbation_prompt: perturbation.prompt,
      runs,
      aggregate_metrics: aggregateMetrics,
    });
  }

  const completedAt = new Date();

  // Calculate overall sensitivity
  const overallSensitivity = calculateOverallSensitivity(
    perturbationResults,
    baseResponseSamples
  );

  // Build output
  const output: PromptSensitivityOutput = {
    execution_id: context.executionId,
    base_prompt: input.base_prompt,
    provider_name: input.provider.provider_name,
    model_id: input.provider.model_id,
    started_at: startTime.toISOString(),
    completed_at: completedAt.toISOString(),
    total_duration_ms: completedAt.getTime() - startTime.getTime(),
    perturbation_results: perturbationResults,
    overall_sensitivity: overallSensitivity,
    base_response_samples: baseResponseSamples,
    total_perturbations: perturbations.length,
    total_runs: totalRuns,
    successful_runs: successfulRuns,
    failed_runs: failedRuns,
    warnings,
  };

  return output;
}

// =============================================================================
// PERTURBATION GENERATION
// =============================================================================

interface GeneratedPerturbation {
  id: string;
  type: PerturbationType;
  prompt: string;
  description?: string;
}

async function generatePerturbations(
  basePrompt: string,
  config: typeof PromptSensitivityInputSchema._type.perturbation_config,
  context: ExecutionContext
): Promise<GeneratedPerturbation[]> {
  const perturbations: GeneratedPerturbation[] = [];

  // Use custom perturbations if provided
  if (config.custom_perturbations && config.custom_perturbations.length > 0) {
    for (const custom of config.custom_perturbations) {
      perturbations.push({
        id: randomUUID(),
        type: custom.type,
        prompt: custom.prompt,
        description: custom.description,
      });
    }
  }

  // Auto-generate perturbations if enabled
  if (config.auto_generate) {
    for (const type of config.types) {
      for (let i = 0; i < config.perturbations_per_type; i++) {
        const generated = generateSinglePerturbation(basePrompt, type, i);
        perturbations.push({
          id: randomUUID(),
          type,
          prompt: generated,
        });
      }
    }
  }

  return perturbations;
}

function generateSinglePerturbation(
  basePrompt: string,
  type: PerturbationType,
  index: number
): string {
  // Simple rule-based perturbation generation
  // In production, this would use more sophisticated NLP techniques

  switch (type) {
    case 'paraphrase':
      return paraphrasePrompt(basePrompt, index);

    case 'instruction_rephrase':
      return rephraseInstruction(basePrompt, index);

    case 'format_change':
      return changeFormat(basePrompt, index);

    case 'tone_shift':
      return shiftTone(basePrompt, index);

    case 'detail_expansion':
      return expandDetails(basePrompt, index);

    case 'detail_reduction':
      return reduceDetails(basePrompt, index);

    case 'order_change':
      return reorderComponents(basePrompt, index);

    case 'emphasis_change':
      return changeEmphasis(basePrompt, index);

    default:
      return basePrompt;
  }
}

// Simple perturbation functions (rule-based)
function paraphrasePrompt(prompt: string, index: number): string {
  const prefixes = [
    'Please ',
    'Could you ',
    'I would like you to ',
  ];
  const prefix = prefixes[index % prefixes.length] ?? '';
  return prefix + prompt.charAt(0).toLowerCase() + prompt.slice(1);
}

function rephraseInstruction(prompt: string, index: number): string {
  const variations = [
    prompt,
    prompt + '\n\nPlease be specific.',
    'Task: ' + prompt,
  ];
  return variations[index % variations.length] ?? prompt;
}

function changeFormat(prompt: string, index: number): string {
  const formats = [
    prompt,
    `# Request\n\n${prompt}`,
    `Question: ${prompt}\n\nAnswer:`,
  ];
  return formats[index % formats.length] ?? prompt;
}

function shiftTone(prompt: string, index: number): string {
  const tones = [
    prompt,
    'Formally speaking, ' + prompt,
    'Hey, ' + prompt.toLowerCase(),
  ];
  return tones[index % tones.length] ?? prompt;
}

function expandDetails(prompt: string, index: number): string {
  const expansions = [
    prompt,
    prompt + ' Please provide a detailed explanation.',
    prompt + ' Be comprehensive in your response.',
  ];
  return expansions[index % expansions.length] ?? prompt;
}

function reduceDetails(prompt: string, index: number): string {
  // Simple reduction: take first sentence
  const sentences = prompt.split(/[.!?]+/);
  if (sentences.length > 1 && index === 0) {
    return sentences[0] + '.';
  }
  return prompt;
}

function reorderComponents(prompt: string, index: number): string {
  // Simple reorder: reverse sentences
  const sentences = prompt.split(/([.!?]+)/);
  if (sentences.length > 2 && index === 0) {
    return sentences.reverse().join('').trim();
  }
  return prompt;
}

function changeEmphasis(prompt: string, index: number): string {
  const emphasis = [
    prompt,
    `IMPORTANT: ${prompt}`,
    `${prompt} (This is critical)`,
  ];
  return emphasis[index % emphasis.length] ?? prompt;
}

// =============================================================================
// PROVIDER ABSTRACTION
// =============================================================================

interface ProviderResponse {
  content?: string;
  latency_ms: number;
  tokens_used?: SensitivityTokenUsage;
  success: boolean;
  error_message?: string;
}

async function executePrompt(
  provider: SensitivityProviderConfig,
  prompt: string,
  samplingConfig: typeof PromptSensitivityInputSchema._type.sampling_config
): Promise<ProviderResponse> {
  const startTime = performance.now();

  try {
    const response = await callProvider(provider, prompt, samplingConfig);
    const endTime = performance.now();

    return {
      content: response.content,
      latency_ms: endTime - startTime,
      tokens_used: response.usage,
      success: true,
    };

  } catch (err) {
    const endTime = performance.now();
    const error = err instanceof Error ? err : new Error(String(err));

    return {
      latency_ms: endTime - startTime,
      success: false,
      error_message: error.message,
    };
  }
}

interface ProviderCallResponse {
  content: string;
  usage?: SensitivityTokenUsage;
}

async function callProvider(
  provider: SensitivityProviderConfig,
  prompt: string,
  samplingConfig: typeof PromptSensitivityInputSchema._type.sampling_config
): Promise<ProviderCallResponse> {
  // This would integrate with the existing provider infrastructure
  // from /core/src/providers/

  const apiKey = await resolveApiKey(provider.api_key_ref);
  const baseUrl = provider.base_url ?? getDefaultBaseUrl(provider.provider_name);

  const requestBody = {
    model: provider.model_id,
    messages: [{ role: 'user', content: prompt }],
    max_tokens: samplingConfig.max_tokens,
    temperature: samplingConfig.temperature,
    top_p: samplingConfig.top_p,
  };

  const controller = new AbortController();
  const timeout = setTimeout(() => controller.abort(), provider.timeout_ms);

  try {
    const response = await fetch(`${baseUrl}/v1/chat/completions`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${apiKey}`,
      },
      body: JSON.stringify(requestBody),
      signal: controller.signal,
    });

    clearTimeout(timeout);

    if (!response.ok) {
      throw new Error(`Provider error: ${response.status} ${await response.text()}`);
    }

    const data = await response.json();

    return {
      content: data.choices?.[0]?.message?.content ?? '',
      usage: data.usage ? {
        prompt_tokens: data.usage.prompt_tokens,
        completion_tokens: data.usage.completion_tokens,
        total_tokens: data.usage.total_tokens,
      } : undefined,
    };

  } finally {
    clearTimeout(timeout);
  }
}

async function resolveApiKey(apiKeyRef?: string): Promise<string> {
  if (!apiKeyRef) {
    throw new Error('API key reference required');
  }

  // In production, this would fetch from a secret manager
  // For now, check environment variables
  const envVar = `${apiKeyRef.toUpperCase().replace(/-/g, '_')}_API_KEY`;
  const key = process.env[envVar];

  if (!key) {
    throw new Error(`API key not found for ref: ${apiKeyRef}`);
  }

  return key;
}

function getDefaultBaseUrl(providerName: string): string {
  const baseUrls: Record<string, string> = {
    openai: 'https://api.openai.com',
    anthropic: 'https://api.anthropic.com',
    google: 'https://generativelanguage.googleapis.com',
    mistral: 'https://api.mistral.ai',
    groq: 'https://api.groq.com/openai',
    together: 'https://api.together.xyz',
    perplexity: 'https://api.perplexity.ai',
  };

  return baseUrls[providerName] ?? '';
}

// =============================================================================
// VARIANCE CALCULATION
// =============================================================================

function calculateAggregateMetrics(
  runs: PerturbationRun[],
  baseResponses: string[]
): PerturbationAggregateMetrics {
  const successfulRuns = runs.filter(r => r.success);
  const responses = successfulRuns.map(r => r.response);

  // Calculate response variance (how much responses differ from each other)
  const responseVariance = calculateResponseVariance(responses);

  // Calculate semantic similarity to base responses
  const semanticSimilarity = calculateSemanticSimilarity(responses, baseResponses);

  // Calculate latency stats
  const latencies = successfulRuns.map(r => r.latency_ms);
  const avgLatency = latencies.length > 0 ? mean(latencies) : 0;
  const latencyVariance = latencies.length > 0 ? variance(latencies) : 0;

  // Calculate token stats
  const totalTokens = successfulRuns.reduce((sum, r) => sum + r.tokens_used.total_tokens, 0);
  const avgTokens = successfulRuns.length > 0 ? totalTokens / successfulRuns.length : 0;

  return {
    response_variance: responseVariance,
    semantic_similarity_to_base: semanticSimilarity,
    average_latency_ms: avgLatency,
    latency_variance: latencyVariance,
    average_tokens: avgTokens,
    success_rate: runs.length > 0 ? successfulRuns.length / runs.length : 0,
  };
}

function calculateResponseVariance(responses: string[]): number {
  if (responses.length < 2) return 0;

  // Calculate pairwise similarities and return 1 - average similarity
  const similarities: number[] = [];

  for (let i = 0; i < responses.length; i++) {
    for (let j = i + 1; j < responses.length; j++) {
      similarities.push(calculateTextSimilarity(responses[i], responses[j]));
    }
  }

  const avgSimilarity = similarities.length > 0 ? mean(similarities) : 1;
  return 1 - avgSimilarity; // Convert similarity to variance
}

function calculateSemanticSimilarity(responses: string[], baseResponses: string[]): number {
  if (responses.length === 0 || baseResponses.length === 0) return 0;

  const similarities: number[] = [];

  // Compare each perturbation response to each base response
  for (const response of responses) {
    for (const baseResponse of baseResponses) {
      similarities.push(calculateTextSimilarity(response, baseResponse));
    }
  }

  return similarities.length > 0 ? mean(similarities) : 0;
}

function calculateTextSimilarity(text1: string, text2: string): number {
  // Simple Jaccard similarity (word overlap)
  const words1 = new Set(text1.toLowerCase().split(/\s+/));
  const words2 = new Set(text2.toLowerCase().split(/\s+/));

  const words1Array = Array.from(words1);
  const words2Array = Array.from(words2);
  const intersection = new Set(words1Array.filter(w => words2.has(w)));
  const union = new Set([...words1Array, ...words2Array]);

  return union.size > 0 ? intersection.size / union.size : 0;
}

// =============================================================================
// OVERALL SENSITIVITY CALCULATION
// =============================================================================

function calculateOverallSensitivity(
  perturbationResults: PerturbationResult[],
  baseResponses: string[]
): OverallSensitivity {
  if (perturbationResults.length === 0) {
    // No perturbations - return default
    return {
      variance_score: 0,
      most_sensitive_perturbation: 'paraphrase' as PerturbationType,
      least_sensitive_perturbation: 'paraphrase' as PerturbationType,
      confidence_interval: { lower: 0, upper: 0 },
      statistical_significance: false,
    };
  }

  // Calculate overall variance score (average across all perturbations)
  const variances = perturbationResults.map(r => r.aggregate_metrics.response_variance);
  const varianceScore = mean(variances);

  // Find most and least sensitive perturbations
  const sortedByVariance = [...perturbationResults].sort(
    (a, b) => b.aggregate_metrics.response_variance - a.aggregate_metrics.response_variance
  );

  const mostSensitive = sortedByVariance[0];
  const leastSensitive = sortedByVariance[sortedByVariance.length - 1];

  // Calculate confidence interval (simple ±2 std dev)
  const stdDev = Math.sqrt(variance(variances));
  const marginOfError = 2 * stdDev / Math.sqrt(variances.length);

  const confidenceInterval = {
    lower: Math.max(0, varianceScore - marginOfError),
    upper: Math.min(1, varianceScore + marginOfError),
  };

  // Statistical significance (simple threshold check)
  const statisticalSignificance = stdDev < 0.2 && variances.length >= 3;

  return {
    variance_score: varianceScore,
    most_sensitive_perturbation: mostSensitive?.type ?? 'paraphrase' as PerturbationType,
    least_sensitive_perturbation: leastSensitive?.type ?? 'paraphrase' as PerturbationType,
    confidence_interval: confidenceInterval,
    statistical_significance: statisticalSignificance,
    p_value: undefined, // Would require proper statistical test
    degrees_of_freedom: perturbationResults.length - 1,
  };
}

// =============================================================================
// STATISTICS HELPERS
// =============================================================================

function mean(values: number[]): number {
  if (values.length === 0) return 0;
  return values.reduce((a, b) => a + b, 0) / values.length;
}

function variance(values: number[]): number {
  if (values.length === 0) return 0;
  const avg = mean(values);
  const squareDiffs = values.map(v => Math.pow(v - avg, 2));
  return mean(squareDiffs);
}

// =============================================================================
// DECISION EVENT CREATION
// =============================================================================

async function createDecisionEvent(
  input: PromptSensitivityInput,
  output: PromptSensitivityOutput,
  confidence: number,
  context: ExecutionContext
): Promise<DecisionEvent> {
  const inputsHash = await hashInputs(input);

  return {
    agent_id: PROMPT_SENSITIVITY_AGENT.agent_id,
    agent_version: PROMPT_SENSITIVITY_AGENT.agent_version,
    decision_type: PROMPT_SENSITIVITY_AGENT.decision_type,
    decision_id: randomUUID(),
    inputs_hash: inputsHash,
    inputs_summary: {
      provider: input.provider.provider_name,
      model: input.provider.model_id,
      perturbation_types: input.perturbation_config.types,
      perturbation_count: output.total_perturbations,
    },
    outputs: output,
    confidence,
    confidence_factors: [
      { factor: 'execution_success_rate', weight: 0.2, value: output.successful_runs / output.total_runs },
      { factor: 'sample_size', weight: 0.3, value: Math.min(1, output.total_runs / 100) },
      { factor: 'statistical_significance', weight: 0.25, value: output.overall_sensitivity.statistical_significance ? 1 : 0.5 },
      { factor: 'variance_consistency', weight: 0.25, value: 1 - output.overall_sensitivity.variance_score },
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
  output: PromptSensitivityOutput,
  decisionId: string
): EdgeFunctionResponse {
  return {
    statusCode: 200,
    headers: {
      'Content-Type': 'application/json',
      'X-Decision-Id': decisionId,
      'X-Agent-Id': PROMPT_SENSITIVITY_AGENT.agent_id,
      'X-Agent-Version': PROMPT_SENSITIVITY_AGENT.agent_version,
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
      'X-Agent-Id': PROMPT_SENSITIVITY_AGENT.agent_id,
      'X-Agent-Version': PROMPT_SENSITIVITY_AGENT.agent_version,
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

export { PROMPT_SENSITIVITY_AGENT };

/**
 * Benchmark Runner Agent - Edge Function Handler
 *
 * AGENT PURPOSE:
 * Execute deterministic benchmark suites against one or more LLMs, producing
 * reproducible performance, quality, latency, and cost metrics.
 *
 * This agent:
 * - Executes benchmarks (YES)
 * - Does NOT compare models (NO)
 * - Does NOT score regressions (NO)
 * - Does NOT rank outputs (NO)
 *
 * Deployed as: Google Cloud Edge Function
 * Part of: LLM-Test-Bench unified GCP service
 */

import { randomUUID } from 'crypto';
import {
  // Contracts
  BenchmarkRunnerInputSchema,
  BenchmarkRunnerOutputSchema,
  BenchmarkRunnerDecisionEventSchema,
  DecisionEvent,
  AgentError,
  validateInput,
  hashInputs,
  // Constants
  BENCHMARK_RUNNER_AGENT,
  VALID_CONSTRAINTS,
  calculateConfidence,
  // Types
  BenchmarkRunnerInput,
  BenchmarkRunnerOutput,
  BenchmarkProviderConfig,
  BenchmarkTestCase,
  TestExecutionResult,
  AggregatedStats,
  LatencyMetrics,
  TokenUsage,
  CostMetrics,
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
 * Edge Function Handler for Benchmark Runner Agent
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
    BENCHMARK_RUNNER_AGENT.agent_id,
    BENCHMARK_RUNNER_AGENT.agent_version,
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
    const inputValidation = validateInput(BenchmarkRunnerInputSchema, request.body);
    if (!inputValidation.success) {
      telemetry.emitValidationFailed('input', inputValidation.error.message);
      await telemetry.flush();

      return createErrorResponse(400, 'Validation Error', inputValidation.error);
    }

    const input = inputValidation.data;

    // Execute benchmark
    const output = await executeBenchmark(input, context);

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
      success_count: output.successful_executions,
      failure_count: output.failed_executions,
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

async function executeBenchmark(
  input: BenchmarkRunnerInput,
  context: ExecutionContext
): Promise<BenchmarkRunnerOutput> {
  const executionConfig = input.execution_config ?? {
    concurrency: 1,
    warm_up_runs: 0,
    iterations_per_test: 1,
    save_responses: true,
    fail_fast: false,
  };

  const results: TestExecutionResult[] = [];
  let startTime = context.startedAt;

  // Warm-up runs (if configured)
  if (executionConfig.warm_up_runs > 0) {
    await runWarmUp(input, executionConfig.warm_up_runs);
  }

  // Execute tests for each provider
  for (const provider of input.providers) {
    for (const testCase of input.suite.test_cases) {
      for (let iteration = 0; iteration < executionConfig.iterations_per_test; iteration++) {
        // Check max duration constraint
        if (executionConfig.max_duration_ms) {
          const elapsed = Date.now() - startTime.getTime();
          if (elapsed >= executionConfig.max_duration_ms) {
            context.constraintsApplied.push('max_duration_exceeded');
            context.telemetry.emitConstraintApplied(
              'max_duration_exceeded',
              `Elapsed: ${elapsed}ms, Max: ${executionConfig.max_duration_ms}ms`
            );
            break;
          }
        }

        const result = await executeTestCase(
          provider,
          testCase,
          iteration,
          executionConfig.save_responses
        );

        results.push(result);

        // Fail-fast constraint
        if (!result.success && executionConfig.fail_fast) {
          context.constraintsApplied.push('fail_fast_triggered');
          context.telemetry.emitConstraintApplied(
            'fail_fast_triggered',
            `Test ${testCase.test_id} failed on iteration ${iteration}`
          );
          break;
        }
      }
    }
  }

  const completedAt = new Date();

  // Calculate aggregated stats
  const aggregatedStats = calculateAggregatedStats(results);

  // Build output
  const output: BenchmarkRunnerOutput = {
    execution_id: context.executionId,
    suite_id: input.suite.suite_id,
    suite_name: input.suite.suite_name,
    started_at: startTime.toISOString(),
    completed_at: completedAt.toISOString(),
    total_duration_ms: completedAt.getTime() - startTime.getTime(),
    total_tests: input.suite.test_cases.length,
    total_executions: results.length,
    successful_executions: results.filter(r => r.success).length,
    failed_executions: results.filter(r => !r.success).length,
    results,
    aggregated_stats: aggregatedStats,
    execution_config: executionConfig,
    providers_tested: input.providers.map(p => ({
      provider_name: p.provider_name,
      model_id: p.model_id,
    })),
  };

  return output;
}

async function runWarmUp(
  input: BenchmarkRunnerInput,
  warmUpRuns: number
): Promise<void> {
  // Run warm-up with first provider and first test case
  const provider = input.providers[0];
  const testCase = input.suite.test_cases[0];

  for (let i = 0; i < warmUpRuns; i++) {
    await executeTestCase(provider, testCase, i, false);
  }
}

async function executeTestCase(
  provider: BenchmarkProviderConfig,
  testCase: BenchmarkTestCase,
  iteration: number,
  saveResponse: boolean
): Promise<TestExecutionResult> {
  const startedAt = new Date();

  try {
    // Execute LLM call via provider
    const response = await callProvider(provider, testCase);

    const completedAt = new Date();
    const latencyMs = completedAt.getTime() - startedAt.getTime();

    return {
      test_id: testCase.test_id,
      iteration,
      provider_name: provider.provider_name,
      model_id: provider.model_id,
      success: true,
      response_content: saveResponse ? response.content : undefined,
      finish_reason: response.finish_reason,
      latency: {
        total_ms: latencyMs,
        time_to_first_token_ms: response.time_to_first_token_ms,
        tokens_per_second: response.tokens_per_second,
      },
      token_usage: response.usage,
      cost: response.cost,
      started_at: startedAt.toISOString(),
      completed_at: completedAt.toISOString(),
    };

  } catch (err) {
    const completedAt = new Date();
    const error = err instanceof Error ? err : new Error(String(err));

    return {
      test_id: testCase.test_id,
      iteration,
      provider_name: provider.provider_name,
      model_id: provider.model_id,
      success: false,
      error_message: error.message,
      latency: {
        total_ms: completedAt.getTime() - startedAt.getTime(),
      },
      started_at: startedAt.toISOString(),
      completed_at: completedAt.toISOString(),
    };
  }
}

// =============================================================================
// PROVIDER ABSTRACTION
// =============================================================================

interface ProviderResponse {
  content: string;
  finish_reason?: string;
  time_to_first_token_ms?: number;
  tokens_per_second?: number;
  usage?: TokenUsage;
  cost?: CostMetrics;
}

async function callProvider(
  provider: BenchmarkProviderConfig,
  testCase: BenchmarkTestCase
): Promise<ProviderResponse> {
  // This would integrate with the existing provider infrastructure
  // from /core/src/providers/

  const apiKey = await resolveApiKey(provider.api_key_ref);
  const baseUrl = provider.base_url ?? getDefaultBaseUrl(provider.provider_name);

  const requestBody = {
    model: provider.model_id,
    messages: [{ role: 'user', content: testCase.prompt }],
    max_tokens: testCase.max_tokens,
    temperature: testCase.temperature,
    top_p: testCase.top_p,
    stop: testCase.stop_sequences,
  };

  const startTime = performance.now();
  let firstTokenTime: number | undefined;

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
    const endTime = performance.now();
    const totalMs = endTime - startTime;

    const completionTokens = data.usage?.completion_tokens ?? 0;
    const tokensPerSecond = completionTokens > 0 ? (completionTokens / (totalMs / 1000)) : undefined;

    return {
      content: data.choices?.[0]?.message?.content ?? '',
      finish_reason: data.choices?.[0]?.finish_reason,
      time_to_first_token_ms: firstTokenTime,
      tokens_per_second: tokensPerSecond,
      usage: data.usage ? {
        prompt_tokens: data.usage.prompt_tokens,
        completion_tokens: data.usage.completion_tokens,
        total_tokens: data.usage.total_tokens,
      } : undefined,
      cost: calculateCost(provider, data.usage),
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

function calculateCost(
  provider: BenchmarkProviderConfig,
  usage?: { prompt_tokens: number; completion_tokens: number; total_tokens: number }
): CostMetrics | undefined {
  if (!usage) return undefined;

  // Cost per 1K tokens (placeholder - would be loaded from config)
  const costTable: Record<string, { input: number; output: number }> = {
    'gpt-4o': { input: 0.0025, output: 0.01 },
    'gpt-4o-mini': { input: 0.00015, output: 0.0006 },
    'claude-3-5-sonnet-20241022': { input: 0.003, output: 0.015 },
    'claude-3-5-haiku-20241022': { input: 0.0008, output: 0.004 },
  };

  const pricing = costTable[provider.model_id] ?? { input: 0, output: 0 };

  const inputCost = (usage.prompt_tokens / 1000) * pricing.input;
  const outputCost = (usage.completion_tokens / 1000) * pricing.output;

  return {
    input_cost_usd: inputCost,
    output_cost_usd: outputCost,
    total_cost_usd: inputCost + outputCost,
  };
}

// =============================================================================
// STATISTICS CALCULATION
// =============================================================================

function calculateAggregatedStats(results: TestExecutionResult[]): AggregatedStats[] {
  // Group by provider/model
  const groups = new Map<string, TestExecutionResult[]>();

  for (const result of results) {
    const key = `${result.provider_name}:${result.model_id}`;
    const group = groups.get(key) ?? [];
    group.push(result);
    groups.set(key, group);
  }

  // Calculate stats for each group
  const stats: AggregatedStats[] = [];

  for (const [key, groupResults] of groups) {
    const [providerName, modelId] = key.split(':');
    const successful = groupResults.filter(r => r.success);
    const latencies = successful.map(r => r.latency.total_ms).sort((a, b) => a - b);

    const totalTokens = groupResults.reduce(
      (sum, r) => sum + (r.token_usage?.total_tokens ?? 0),
      0
    );

    const totalCost = groupResults.reduce(
      (sum, r) => sum + (r.cost?.total_cost_usd ?? 0),
      0
    );

    stats.push({
      provider_name: providerName,
      model_id: modelId,
      total_executions: groupResults.length,
      successful_executions: successful.length,
      failed_executions: groupResults.length - successful.length,
      success_rate: successful.length / groupResults.length,
      latency_p50_ms: percentile(latencies, 50),
      latency_p95_ms: percentile(latencies, 95),
      latency_p99_ms: percentile(latencies, 99),
      latency_mean_ms: mean(latencies),
      latency_min_ms: Math.min(...latencies) || 0,
      latency_max_ms: Math.max(...latencies) || 0,
      latency_stddev_ms: stddev(latencies),
      total_tokens: totalTokens,
      avg_tokens_per_request: groupResults.length > 0 ? totalTokens / groupResults.length : 0,
      total_cost_usd: totalCost,
      avg_cost_per_request_usd: groupResults.length > 0 ? totalCost / groupResults.length : 0,
      avg_tokens_per_second: calculateAvgTokensPerSecond(successful),
    });
  }

  return stats;
}

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

function calculateAvgTokensPerSecond(results: TestExecutionResult[]): number {
  const withTps = results.filter(r => r.latency.tokens_per_second !== undefined);
  if (withTps.length === 0) return 0;

  return mean(withTps.map(r => r.latency.tokens_per_second!));
}

// =============================================================================
// CONFIDENCE CALCULATION
// =============================================================================

function calculateOverallConfidence(output: BenchmarkRunnerOutput): number {
  if (output.aggregated_stats.length === 0) return 0;

  // Average confidence across all provider/model combinations
  const confidences = output.aggregated_stats.map(stats => calculateConfidence(stats));
  return mean(confidences);
}

// =============================================================================
// DECISION EVENT CREATION
// =============================================================================

async function createDecisionEvent(
  input: BenchmarkRunnerInput,
  output: BenchmarkRunnerOutput,
  confidence: number,
  context: ExecutionContext
): Promise<DecisionEvent> {
  const inputsHash = await hashInputs(input);

  return {
    agent_id: BENCHMARK_RUNNER_AGENT.agent_id,
    agent_version: BENCHMARK_RUNNER_AGENT.agent_version,
    decision_type: BENCHMARK_RUNNER_AGENT.decision_type,
    decision_id: randomUUID(),
    inputs_hash: inputsHash,
    inputs_summary: {
      suite_id: input.suite.suite_id,
      provider_count: input.providers.length,
      test_count: input.suite.test_cases.length,
    },
    outputs: output,
    confidence,
    confidence_factors: [
      { factor: 'execution_success_rate', weight: 0.4, value: output.successful_executions / output.total_executions },
      { factor: 'sample_size', weight: 0.2, value: Math.min(1, output.total_executions / 100) },
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
  output: BenchmarkRunnerOutput,
  decisionId: string
): EdgeFunctionResponse {
  return {
    statusCode: 200,
    headers: {
      'Content-Type': 'application/json',
      'X-Decision-Id': decisionId,
      'X-Agent-Id': BENCHMARK_RUNNER_AGENT.agent_id,
      'X-Agent-Version': BENCHMARK_RUNNER_AGENT.agent_version,
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
      'X-Agent-Id': BENCHMARK_RUNNER_AGENT.agent_id,
      'X-Agent-Version': BENCHMARK_RUNNER_AGENT.agent_version,
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

export { BENCHMARK_RUNNER_AGENT };

/**
 * Regression Detection Agent - Edge Function Handler
 *
 * AGENT PURPOSE:
 * Detect statistically significant regressions between baseline and candidate
 * benchmark runs. Performs comparative analysis to identify performance degradation
 * and produces severity classifications with confidence scores.
 *
 * This agent:
 * - Detects regressions between baseline and candidate runs (YES)
 * - Classifies regression severity (YES)
 * - Calculates statistical significance (YES)
 * - Does NOT execute benchmarks (NO)
 * - Does NOT compare/rank models (NO)
 * - Does NOT orchestrate workflows (NO)
 *
 * Deployed as: Google Cloud Edge Function
 * Part of: LLM-Test-Bench unified GCP service
 */

import { randomUUID } from 'crypto';
import {
  // Contracts
  RegressionDetectionInputSchema,
  RegressionDetectionOutputSchema,
  RegressionDetectionDecisionEventSchema,
  DecisionEvent,
  AgentError,
  validateInput,
  hashInputs,
  // Constants
  REGRESSION_DETECTION_AGENT,
  VALID_REGRESSION_CONSTRAINTS,
  calculateRegressionConfidence,
  calculateCohensD,
  welchTTest,
  interpretEffectSize,
  // Types
  RegressionDetectionInput,
  RegressionDetectionOutput,
  RegressionThresholds,
  StatisticalConfig,
  RegressionMetricsConfig,
  ModelRegressionResult,
  MetricRegression,
  RegressionSummary,
  RegressionSeverity,
  StatisticalTestResult,
  RegressionConstraint,
  AggregatedStats,
  BenchmarkRunnerOutput,
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
  constraintsApplied: RegressionConstraint[];
}

// =============================================================================
// DEFAULT CONFIGURATIONS
// =============================================================================

const DEFAULT_THRESHOLDS: RegressionThresholds = {
  latency: { critical: 0.50, major: 0.25, minor: 0.10 },
  throughput: { critical: 0.50, major: 0.25, minor: 0.10 },
  success_rate: { critical: 0.10, major: 0.05, minor: 0.02 },
  cost: { critical: 0.50, major: 0.25, minor: 0.10 },
};

const DEFAULT_STATISTICAL_CONFIG: StatisticalConfig = {
  confidence_level: 0.95,
  min_sample_size: 5,
  use_welch_t_test: true,
  use_mann_whitney: false,
  effect_size_threshold: 0.5,
};

const DEFAULT_METRICS_CONFIG: RegressionMetricsConfig = {
  analyze_latency: true,
  analyze_throughput: true,
  analyze_success_rate: true,
  analyze_cost: true,
  latency_percentile: 'p95',
};

// =============================================================================
// MAIN HANDLER
// =============================================================================

/**
 * Edge Function Handler for Regression Detection Agent
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
    REGRESSION_DETECTION_AGENT.agent_id,
    REGRESSION_DETECTION_AGENT.agent_version,
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
    const inputValidation = validateInput(RegressionDetectionInputSchema, request.body);
    if (!inputValidation.success) {
      const validationError = inputValidation.error;
      telemetry.emitValidationFailed('input', validationError.message);
      await telemetry.flush();

      return createErrorResponse(400, 'Validation Error', validationError);
    }

    const input = inputValidation.data;

    // Perform regression detection
    const output = await detectRegressions(input, context);

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
      success_count: output.summary.models_with_regressions === 0 ? 1 : 0,
      failure_count: output.summary.models_with_regressions,
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
// CORE REGRESSION DETECTION LOGIC
// =============================================================================

async function detectRegressions(
  input: RegressionDetectionInput,
  context: ExecutionContext
): Promise<RegressionDetectionOutput> {
  const thresholds = input.thresholds ?? DEFAULT_THRESHOLDS;
  const statisticalConfig = input.statistical_config ?? DEFAULT_STATISTICAL_CONFIG;
  const metricsConfig = input.metrics_config ?? DEFAULT_METRICS_CONFIG;

  // Extract and aggregate stats from all runs
  const baselineStats = aggregateRunStats(input.baseline_runs);
  const candidateStats = aggregateRunStats(input.candidate_runs);

  // Find common models (normalize filter to required fields)
  const normalizedFilter = input.model_filter?.map(f => ({
    provider_name: f.provider_name ?? '',
    model_id: f.model_id ?? '',
  })).filter(f => f.provider_name && f.model_id);
  const commonModels = findCommonModels(baselineStats, candidateStats, normalizedFilter);

  if (commonModels.length === 0) {
    context.constraintsApplied.push('no_common_models');
    context.telemetry.emitConstraintApplied(
      'no_common_models',
      'No models found in both baseline and candidate runs'
    );
  }

  // Check sample sizes
  if (input.baseline_runs.length === 1) {
    context.constraintsApplied.push('single_baseline_run');
    context.telemetry.emitConstraintApplied('single_baseline_run');
  }
  if (input.candidate_runs.length === 1) {
    context.constraintsApplied.push('single_candidate_run');
    context.telemetry.emitConstraintApplied('single_candidate_run');
  }

  // Analyze each model
  const modelResults: ModelRegressionResult[] = [];

  for (const { provider_name, model_id } of commonModels) {
    const baseline = baselineStats.get(`${provider_name}:${model_id}`)!;
    const candidate = candidateStats.get(`${provider_name}:${model_id}`)!;

    const result = analyzeModelRegression(
      provider_name,
      model_id,
      baseline,
      candidate,
      thresholds,
      statisticalConfig,
      metricsConfig,
      context
    );

    modelResults.push(result);
  }

  // Calculate summary
  const summary = calculateSummary(modelResults, input.baseline_runs, input.candidate_runs);

  const output: RegressionDetectionOutput = {
    detection_id: randomUUID(),
    detected_at: new Date().toISOString(),
    summary,
    model_results: modelResults,
    thresholds_used: thresholds,
    statistical_config_used: statisticalConfig,
    metrics_config_used: metricsConfig,
    analysis_duration_ms: Date.now() - context.startedAt.getTime(),
  };

  return output;
}

interface AggregatedModelStats {
  provider_name: string;
  model_id: string;
  latency_p50_values: number[];
  latency_p95_values: number[];
  latency_p99_values: number[];
  throughput_values: number[];
  success_rate_values: number[];
  cost_values: number[];
  execution_ids: string[];
  total_executions: number;
}

function aggregateRunStats(
  runs: BenchmarkRunnerOutput[]
): Map<string, AggregatedModelStats> {
  const statsMap = new Map<string, AggregatedModelStats>();

  for (const run of runs) {
    for (const stats of run.aggregated_stats) {
      const key = `${stats.provider_name}:${stats.model_id}`;

      if (!statsMap.has(key)) {
        statsMap.set(key, {
          provider_name: stats.provider_name,
          model_id: stats.model_id,
          latency_p50_values: [],
          latency_p95_values: [],
          latency_p99_values: [],
          throughput_values: [],
          success_rate_values: [],
          cost_values: [],
          execution_ids: [],
          total_executions: 0,
        });
      }

      const agg = statsMap.get(key)!;
      agg.latency_p50_values.push(stats.latency_p50_ms);
      agg.latency_p95_values.push(stats.latency_p95_ms);
      agg.latency_p99_values.push(stats.latency_p99_ms);
      agg.throughput_values.push(stats.avg_tokens_per_second ?? 0);
      agg.success_rate_values.push(stats.success_rate);
      agg.cost_values.push(stats.avg_cost_per_request_usd);
      agg.execution_ids.push(run.execution_id);
      agg.total_executions += stats.total_executions;
    }
  }

  return statsMap;
}

function findCommonModels(
  baseline: Map<string, AggregatedModelStats>,
  candidate: Map<string, AggregatedModelStats>,
  filter?: Array<{ provider_name: string; model_id: string }>
): Array<{ provider_name: string; model_id: string }> {
  const common: Array<{ provider_name: string; model_id: string }> = [];

  const baselineKeys = Array.from(baseline.keys());
  for (const key of baselineKeys) {
    if (candidate.has(key)) {
      const [provider_name, model_id] = key.split(':');

      // Apply filter if specified
      if (filter && filter.length > 0) {
        const matches = filter.some(
          f => f.provider_name === provider_name && f.model_id === model_id
        );
        if (!matches) continue;
      }

      common.push({ provider_name, model_id });
    }
  }

  return common;
}

function analyzeModelRegression(
  provider_name: string,
  model_id: string,
  baseline: AggregatedModelStats,
  candidate: AggregatedModelStats,
  thresholds: RegressionThresholds,
  statisticalConfig: StatisticalConfig,
  metricsConfig: RegressionMetricsConfig,
  context: ExecutionContext
): ModelRegressionResult {
  const metricRegressions: MetricRegression[] = [];

  // Helper to normalize thresholds with defaults
  const normalizeThreshold = (t?: { critical?: number; major?: number; minor?: number }) => ({
    critical: t?.critical ?? 0.50,
    major: t?.major ?? 0.25,
    minor: t?.minor ?? 0.10,
  });

  // Analyze latency regression
  if (metricsConfig.analyze_latency) {
    const latencyValues = getLatencyValues(baseline, candidate, metricsConfig.latency_percentile);
    const latencyRegression = analyzeMetricRegression(
      'latency',
      latencyValues.baseline,
      latencyValues.candidate,
      normalizeThreshold(thresholds.latency),
      statisticalConfig,
      'ms',
      true // Higher is worse
    );
    if (latencyRegression) {
      metricRegressions.push(latencyRegression);
    }
  }

  // Analyze throughput regression
  if (metricsConfig.analyze_throughput) {
    const throughputRegression = analyzeMetricRegression(
      'throughput',
      baseline.throughput_values,
      candidate.throughput_values,
      normalizeThreshold(thresholds.throughput),
      statisticalConfig,
      'tokens/s',
      false // Lower is worse
    );
    if (throughputRegression) {
      metricRegressions.push(throughputRegression);
    }
  }

  // Analyze success rate regression
  if (metricsConfig.analyze_success_rate) {
    const successRateThresholds = {
      critical: thresholds.success_rate?.critical ?? 0.10,
      major: thresholds.success_rate?.major ?? 0.05,
      minor: thresholds.success_rate?.minor ?? 0.02,
    };
    const successRateRegression = analyzeMetricRegression(
      'success_rate',
      baseline.success_rate_values,
      candidate.success_rate_values,
      successRateThresholds,
      statisticalConfig,
      '%',
      false, // Lower is worse
      true   // Use absolute thresholds
    );
    if (successRateRegression) {
      metricRegressions.push(successRateRegression);
    }
  }

  // Analyze cost regression
  if (metricsConfig.analyze_cost) {
    const costRegression = analyzeMetricRegression(
      'cost',
      baseline.cost_values,
      candidate.cost_values,
      normalizeThreshold(thresholds.cost),
      statisticalConfig,
      'USD',
      true // Higher is worse
    );
    if (costRegression) {
      metricRegressions.push(costRegression);
    }
  }

  // Determine overall severity
  const overallSeverity = determineOverallSeverity(metricRegressions);
  const hasRegression = metricRegressions.some(m => m.is_regression);
  const regressionCount = metricRegressions.filter(m => m.is_regression).length;

  // Generate summary
  const summary = generateResultSummary(
    provider_name,
    model_id,
    overallSeverity,
    metricRegressions.filter(m => m.is_regression)
  );

  return {
    provider_name,
    model_id,
    overall_severity: overallSeverity,
    has_regression: hasRegression,
    regression_count: regressionCount,
    metric_regressions: metricRegressions,
    summary,
    baseline_execution_ids: baseline.execution_ids,
    candidate_execution_ids: candidate.execution_ids,
  };
}

function getLatencyValues(
  baseline: AggregatedModelStats,
  candidate: AggregatedModelStats,
  percentile: 'p50' | 'p95' | 'p99'
): { baseline: number[]; candidate: number[] } {
  switch (percentile) {
    case 'p50':
      return { baseline: baseline.latency_p50_values, candidate: candidate.latency_p50_values };
    case 'p95':
      return { baseline: baseline.latency_p95_values, candidate: candidate.latency_p95_values };
    case 'p99':
      return { baseline: baseline.latency_p99_values, candidate: candidate.latency_p99_values };
  }
}

function analyzeMetricRegression(
  metricName: 'latency' | 'throughput' | 'success_rate' | 'cost',
  baselineValues: number[],
  candidateValues: number[],
  thresholds: { critical: number; major: number; minor: number },
  statisticalConfig: StatisticalConfig,
  unit: string,
  higherIsWorse: boolean,
  useAbsoluteThreshold = false
): MetricRegression | null {
  if (baselineValues.length === 0 || candidateValues.length === 0) {
    return null;
  }

  // Calculate statistics
  const baselineMean = mean(baselineValues);
  const baselineStddev = stddev(baselineValues);
  const candidateMean = mean(candidateValues);
  const candidateStddev = stddev(candidateValues);

  // Calculate changes
  const absoluteChange = candidateMean - baselineMean;
  let percentageChange = baselineMean !== 0
    ? ((candidateMean - baselineMean) / baselineMean)
    : 0;

  // Determine direction
  let changeDirection: 'improved' | 'degraded' | 'unchanged';
  if (Math.abs(percentageChange) < 0.01) {
    changeDirection = 'unchanged';
  } else if (higherIsWorse) {
    changeDirection = percentageChange > 0 ? 'degraded' : 'improved';
  } else {
    changeDirection = percentageChange < 0 ? 'degraded' : 'improved';
  }

  // Perform statistical test
  const testResult = performStatisticalTest(
    baselineMean,
    candidateMean,
    baselineStddev,
    candidateStddev,
    baselineValues.length,
    candidateValues.length,
    statisticalConfig
  );

  // Determine severity
  const severity = determineSeverity(
    percentageChange,
    absoluteChange,
    thresholds,
    higherIsWorse,
    testResult.is_significant,
    useAbsoluteThreshold
  );

  // Is this a regression?
  const isRegression =
    changeDirection === 'degraded' &&
    testResult.is_significant &&
    severity !== 'none';

  return {
    metric_name: metricName,
    baseline_value: baselineMean,
    baseline_stddev: baselineStddev,
    baseline_sample_count: baselineValues.length,
    candidate_value: candidateMean,
    candidate_stddev: candidateStddev,
    candidate_sample_count: candidateValues.length,
    absolute_change: absoluteChange,
    percentage_change: percentageChange,
    change_direction: changeDirection,
    statistical_test: testResult,
    severity,
    is_regression: isRegression,
    unit,
  };
}

function performStatisticalTest(
  mean1: number,
  mean2: number,
  stddev1: number,
  stddev2: number,
  n1: number,
  n2: number,
  config: StatisticalConfig
): StatisticalTestResult {
  // Perform Welch's t-test
  const { t, df, p } = welchTTest(mean1, mean2, stddev1, stddev2, n1, n2);

  // Calculate effect size (Cohen's d)
  const effectSize = calculateCohensD(mean1, mean2, stddev1, stddev2, n1, n2);
  const effectSizeInterpretation = interpretEffectSize(effectSize);

  // Determine significance
  const alpha = 1 - config.confidence_level;
  const isSignificant = p < alpha;

  return {
    test_name: 'welch_t_test',
    statistic: t,
    p_value: p,
    is_significant: isSignificant,
    effect_size: effectSize,
    effect_size_interpretation: effectSizeInterpretation,
    degrees_of_freedom: df,
  };
}

function determineSeverity(
  percentageChange: number,
  absoluteChange: number,
  thresholds: { critical: number; major: number; minor: number },
  higherIsWorse: boolean,
  isSignificant: boolean,
  useAbsolute: boolean
): RegressionSeverity {
  if (!isSignificant) {
    return 'none';
  }

  // Get the magnitude to compare
  const magnitude = useAbsolute
    ? Math.abs(absoluteChange)
    : Math.abs(percentageChange);

  // Check if this is a degradation
  const isDegradation = higherIsWorse
    ? percentageChange > 0
    : percentageChange < 0;

  if (!isDegradation) {
    return 'none';
  }

  // Classify severity
  if (magnitude >= thresholds.critical) {
    return 'critical';
  } else if (magnitude >= thresholds.major) {
    return 'major';
  } else if (magnitude >= thresholds.minor) {
    return 'minor';
  }

  return 'none';
}

function determineOverallSeverity(
  regressions: MetricRegression[]
): RegressionSeverity {
  const severities = regressions.map(r => r.severity);

  if (severities.includes('critical')) return 'critical';
  if (severities.includes('major')) return 'major';
  if (severities.includes('minor')) return 'minor';
  return 'none';
}

function generateResultSummary(
  provider_name: string,
  model_id: string,
  severity: RegressionSeverity,
  regressions: MetricRegression[]
): string {
  if (regressions.length === 0) {
    return `No statistically significant regressions detected for ${provider_name}/${model_id}.`;
  }

  const regressedMetrics = regressions
    .map(r => `${r.metric_name} (${(r.percentage_change * 100).toFixed(1)}%)`)
    .join(', ');

  return `${severity.toUpperCase()} regression detected for ${provider_name}/${model_id}: ${regressedMetrics}`;
}

function calculateSummary(
  results: ModelRegressionResult[],
  baselineRuns: BenchmarkRunnerOutput[],
  candidateRuns: BenchmarkRunnerOutput[]
): RegressionSummary {
  const modelsWithRegressions = results.filter(r => r.has_regression);
  const modelsWithCritical = results.filter(r => r.overall_severity === 'critical');
  const modelsWithMajor = results.filter(r => r.overall_severity === 'major');
  const modelsWithMinor = results.filter(r => r.overall_severity === 'minor');

  const worstSeverity = determineOverallSeverity(
    results.flatMap(r => r.metric_regressions)
  );

  const baselineExecutions = baselineRuns.reduce(
    (sum, r) => sum + r.total_executions,
    0
  );
  const candidateExecutions = candidateRuns.reduce(
    (sum, r) => sum + r.total_executions,
    0
  );

  let summaryText: string;
  if (modelsWithRegressions.length === 0) {
    summaryText = `No regressions detected across ${results.length} model(s).`;
  } else {
    summaryText = `Detected regressions in ${modelsWithRegressions.length} of ${results.length} model(s). ` +
      `Severity breakdown: ${modelsWithCritical.length} critical, ${modelsWithMajor.length} major, ${modelsWithMinor.length} minor.`;
  }

  return {
    total_models_analyzed: results.length,
    models_with_regressions: modelsWithRegressions.length,
    models_with_critical: modelsWithCritical.length,
    models_with_major: modelsWithMajor.length,
    models_with_minor: modelsWithMinor.length,
    worst_severity: worstSeverity,
    total_baseline_executions: baselineExecutions,
    total_candidate_executions: candidateExecutions,
    any_regressions_detected: modelsWithRegressions.length > 0,
    summary_text: summaryText,
  };
}

// =============================================================================
// STATISTICS UTILITIES
// =============================================================================

function mean(values: number[]): number {
  if (values.length === 0) return 0;
  return values.reduce((a, b) => a + b, 0) / values.length;
}

function stddev(values: number[]): number {
  if (values.length < 2) return 0;
  const avg = mean(values);
  const squareDiffs = values.map(v => Math.pow(v - avg, 2));
  return Math.sqrt(mean(squareDiffs));
}

// =============================================================================
// CONFIDENCE CALCULATION
// =============================================================================

function calculateOverallConfidence(output: RegressionDetectionOutput): number {
  if (output.model_results.length === 0) return 0;

  const totalBaselineExecutions = output.summary.total_baseline_executions;
  const totalCandidateExecutions = output.summary.total_candidate_executions;

  // Average confidence across all models
  const confidences = output.model_results.map(result =>
    calculateRegressionConfidence(result, totalBaselineExecutions, totalCandidateExecutions).confidence
  );

  return mean(confidences);
}

// =============================================================================
// DECISION EVENT CREATION
// =============================================================================

async function createDecisionEvent(
  input: RegressionDetectionInput,
  output: RegressionDetectionOutput,
  confidence: number,
  context: ExecutionContext
): Promise<DecisionEvent> {
  const inputsHash = await hashInputs(input);

  const confidenceResult = output.model_results.length > 0
    ? calculateRegressionConfidence(
        output.model_results[0],
        output.summary.total_baseline_executions,
        output.summary.total_candidate_executions
      )
    : { confidence: 0, factors: [] };

  return {
    agent_id: REGRESSION_DETECTION_AGENT.agent_id,
    agent_version: REGRESSION_DETECTION_AGENT.agent_version,
    decision_type: REGRESSION_DETECTION_AGENT.decision_type,
    decision_id: randomUUID(),
    inputs_hash: inputsHash,
    inputs_summary: {
      baseline_runs_count: input.baseline_runs.length,
      candidate_runs_count: input.candidate_runs.length,
      models_analyzed: output.summary.total_models_analyzed,
    },
    outputs: output as unknown as Record<string, unknown>,
    confidence,
    confidence_factors: confidenceResult.factors,
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
  output: RegressionDetectionOutput,
  decisionId: string
): EdgeFunctionResponse {
  return {
    statusCode: 200,
    headers: {
      'Content-Type': 'application/json',
      'X-Decision-Id': decisionId,
      'X-Agent-Id': REGRESSION_DETECTION_AGENT.agent_id,
      'X-Agent-Version': REGRESSION_DETECTION_AGENT.agent_version,
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
      'X-Agent-Id': REGRESSION_DETECTION_AGENT.agent_id,
      'X-Agent-Version': REGRESSION_DETECTION_AGENT.agent_version,
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

export { REGRESSION_DETECTION_AGENT };

/**
 * Regression Detection Agent Contract Schemas
 *
 * AGENT PURPOSE:
 * Detect statistically significant regressions between baseline and candidate
 * benchmark runs. Performs comparative analysis to identify performance degradation
 * and produces severity classifications with confidence scores.
 *
 * This agent:
 * - Detects regressions between baseline and candidate runs (YES)
 * - Classifies regression severity (critical, major, minor, none) (YES)
 * - Calculates statistical significance (YES)
 * - Does NOT execute benchmarks (NO - that's benchmark-runner)
 * - Does NOT compare/rank models (NO - that's model-comparator)
 * - Does NOT enforce policies (NO - that's policy agents)
 * - Does NOT orchestrate workflows (NO - that's orchestrator)
 * - Does NOT recommend fixes (NO - that's separate agents)
 *
 * decision_type: "regression_detection"
 */

import { z } from 'zod';
import { DecisionEventSchema } from './base';
import { BenchmarkRunnerOutputSchema, AggregatedStatsSchema } from './benchmark-runner';

// =============================================================================
// AGENT METADATA
// =============================================================================

export const REGRESSION_DETECTION_AGENT = {
  agent_id: 'regression-detection',
  agent_version: '1.0.0',
  decision_type: 'regression_detection',
} as const;

// =============================================================================
// INPUT SCHEMAS
// =============================================================================

/**
 * Regression detection thresholds for each severity level
 * Thresholds represent percentage degradation (0.05 = 5%)
 */
export const RegressionThresholdsSchema = z.object({
  /** Latency degradation thresholds (percentage increase) */
  latency: z.object({
    critical: z.number().min(0).max(1).default(0.50), // 50%+ increase
    major: z.number().min(0).max(1).default(0.25),    // 25%+ increase
    minor: z.number().min(0).max(1).default(0.10),    // 10%+ increase
  }).default({}),

  /** Throughput degradation thresholds (percentage decrease) */
  throughput: z.object({
    critical: z.number().min(0).max(1).default(0.50), // 50%+ decrease
    major: z.number().min(0).max(1).default(0.25),    // 25%+ decrease
    minor: z.number().min(0).max(1).default(0.10),    // 10%+ decrease
  }).default({}),

  /** Success rate degradation thresholds (absolute decrease) */
  success_rate: z.object({
    critical: z.number().min(0).max(1).default(0.10), // 10%+ decrease
    major: z.number().min(0).max(1).default(0.05),    // 5%+ decrease
    minor: z.number().min(0).max(1).default(0.02),    // 2%+ decrease
  }).default({}),

  /** Cost increase thresholds (percentage increase) */
  cost: z.object({
    critical: z.number().min(0).max(1).default(0.50), // 50%+ increase
    major: z.number().min(0).max(1).default(0.25),    // 25%+ increase
    minor: z.number().min(0).max(1).default(0.10),    // 10%+ increase
  }).default({}),
});

export type RegressionThresholds = z.infer<typeof RegressionThresholdsSchema>;

/**
 * Statistical significance configuration
 */
export const StatisticalConfigSchema = z.object({
  /** Confidence level for statistical tests (0.95 = 95% confidence) */
  confidence_level: z.number().min(0.5).max(0.999).default(0.95),

  /** Minimum sample size required for statistical significance */
  min_sample_size: z.number().int().positive().default(5),

  /** Use Welch's t-test for unequal variances */
  use_welch_t_test: z.boolean().default(true),

  /** Use Mann-Whitney U test for non-normal distributions */
  use_mann_whitney: z.boolean().default(false),

  /** Effect size threshold (Cohen's d) for practical significance */
  effect_size_threshold: z.number().min(0).default(0.5),
});

export type StatisticalConfig = z.infer<typeof StatisticalConfigSchema>;

/**
 * Metrics to analyze for regressions
 */
export const RegressionMetricsConfigSchema = z.object({
  /** Analyze latency regressions */
  analyze_latency: z.boolean().default(true),

  /** Analyze throughput regressions */
  analyze_throughput: z.boolean().default(true),

  /** Analyze success rate regressions */
  analyze_success_rate: z.boolean().default(true),

  /** Analyze cost regressions */
  analyze_cost: z.boolean().default(true),

  /** Latency percentile to analyze (p50, p95, p99) */
  latency_percentile: z.enum(['p50', 'p95', 'p99']).default('p95'),
});

export type RegressionMetricsConfig = z.infer<typeof RegressionMetricsConfigSchema>;

/**
 * Main input schema for Regression Detection Agent
 */
export const RegressionDetectionInputSchema = z.object({
  /**
   * Baseline benchmark run(s) - the reference performance
   * Multiple runs are combined for statistical analysis
   */
  baseline_runs: z.array(BenchmarkRunnerOutputSchema).min(1),

  /**
   * Candidate benchmark run(s) - the performance to compare
   * Multiple runs are combined for statistical analysis
   */
  candidate_runs: z.array(BenchmarkRunnerOutputSchema).min(1),

  /**
   * Thresholds for regression severity classification
   */
  thresholds: RegressionThresholdsSchema.optional(),

  /**
   * Statistical significance configuration
   */
  statistical_config: StatisticalConfigSchema.optional(),

  /**
   * Metrics to analyze for regressions
   */
  metrics_config: RegressionMetricsConfigSchema.optional(),

  /**
   * Optional: Limit analysis to specific provider/model combinations
   */
  model_filter: z.array(z.object({
    provider_name: z.string(),
    model_id: z.string(),
  })).optional(),

  /** Optional: caller context */
  caller_id: z.string().optional(),

  /** Optional: correlation ID for tracing */
  correlation_id: z.string().uuid().optional(),
});

export type RegressionDetectionInput = z.infer<typeof RegressionDetectionInputSchema>;

// =============================================================================
// OUTPUT SCHEMAS
// =============================================================================

/**
 * Severity levels for regressions
 */
export const RegressionSeveritySchema = z.enum([
  'critical',  // Severe performance degradation
  'major',     // Significant performance degradation
  'minor',     // Noticeable performance degradation
  'none',      // No regression detected
]);

export type RegressionSeverity = z.infer<typeof RegressionSeveritySchema>;

/**
 * Statistical test results
 */
export const StatisticalTestResultSchema = z.object({
  /** Test performed (t-test, welch-t-test, mann-whitney) */
  test_name: z.string(),

  /** Test statistic value */
  statistic: z.number(),

  /** P-value */
  p_value: z.number(),

  /** Whether the result is statistically significant */
  is_significant: z.boolean(),

  /** Effect size (Cohen's d or r) */
  effect_size: z.number(),

  /** Effect size interpretation (small, medium, large) */
  effect_size_interpretation: z.enum(['negligible', 'small', 'medium', 'large']),

  /** Degrees of freedom (if applicable) */
  degrees_of_freedom: z.number().optional(),
});

export type StatisticalTestResult = z.infer<typeof StatisticalTestResultSchema>;

/**
 * Individual metric regression result
 */
export const MetricRegressionSchema = z.object({
  /** Metric name */
  metric_name: z.enum(['latency', 'throughput', 'success_rate', 'cost']),

  /** Baseline value (mean) */
  baseline_value: z.number(),

  /** Baseline standard deviation */
  baseline_stddev: z.number(),

  /** Baseline sample count */
  baseline_sample_count: z.number().int().nonnegative(),

  /** Candidate value (mean) */
  candidate_value: z.number(),

  /** Candidate standard deviation */
  candidate_stddev: z.number(),

  /** Candidate sample count */
  candidate_sample_count: z.number().int().nonnegative(),

  /** Absolute change */
  absolute_change: z.number(),

  /** Percentage change (positive = worse for latency/cost, negative = worse for throughput/success_rate) */
  percentage_change: z.number(),

  /** Direction of change */
  change_direction: z.enum(['improved', 'degraded', 'unchanged']),

  /** Statistical test result */
  statistical_test: StatisticalTestResultSchema,

  /** Regression severity */
  severity: RegressionSeveritySchema,

  /** Whether this is a statistically significant regression */
  is_regression: z.boolean(),

  /** Unit of measurement */
  unit: z.string(),
});

export type MetricRegression = z.infer<typeof MetricRegressionSchema>;

/**
 * Single model regression result
 */
export const ModelRegressionResultSchema = z.object({
  /** Provider name */
  provider_name: z.string(),

  /** Model identifier */
  model_id: z.string(),

  /** Overall severity (worst of all metrics) */
  overall_severity: RegressionSeveritySchema,

  /** Whether any statistically significant regression was detected */
  has_regression: z.boolean(),

  /** Number of regressions detected across all metrics */
  regression_count: z.number().int().nonnegative(),

  /** Individual metric regressions */
  metric_regressions: z.array(MetricRegressionSchema),

  /** Summary of key findings */
  summary: z.string(),

  /** Baseline execution IDs used */
  baseline_execution_ids: z.array(z.string().uuid()),

  /** Candidate execution IDs used */
  candidate_execution_ids: z.array(z.string().uuid()),
});

export type ModelRegressionResult = z.infer<typeof ModelRegressionResultSchema>;

/**
 * Aggregate regression summary
 */
export const RegressionSummarySchema = z.object({
  /** Total models analyzed */
  total_models_analyzed: z.number().int().nonnegative(),

  /** Models with regressions detected */
  models_with_regressions: z.number().int().nonnegative(),

  /** Models with critical regressions */
  models_with_critical: z.number().int().nonnegative(),

  /** Models with major regressions */
  models_with_major: z.number().int().nonnegative(),

  /** Models with minor regressions */
  models_with_minor: z.number().int().nonnegative(),

  /** Overall worst severity across all models */
  worst_severity: RegressionSeveritySchema,

  /** Total baseline executions analyzed */
  total_baseline_executions: z.number().int().nonnegative(),

  /** Total candidate executions analyzed */
  total_candidate_executions: z.number().int().nonnegative(),

  /** Whether any statistically significant regressions were detected */
  any_regressions_detected: z.boolean(),

  /** Brief textual summary */
  summary_text: z.string(),
});

export type RegressionSummary = z.infer<typeof RegressionSummarySchema>;

/**
 * Main output schema for Regression Detection Agent
 */
export const RegressionDetectionOutputSchema = z.object({
  /** Unique detection ID */
  detection_id: z.string().uuid(),

  /** When the detection was performed */
  detected_at: z.string().datetime(),

  /** Aggregate summary */
  summary: RegressionSummarySchema,

  /** Per-model regression results */
  model_results: z.array(ModelRegressionResultSchema),

  /** Thresholds used */
  thresholds_used: RegressionThresholdsSchema,

  /** Statistical configuration used */
  statistical_config_used: StatisticalConfigSchema,

  /** Metrics configuration used */
  metrics_config_used: RegressionMetricsConfigSchema,

  /** Duration of analysis in milliseconds */
  analysis_duration_ms: z.number().nonnegative(),
});

export type RegressionDetectionOutput = z.infer<typeof RegressionDetectionOutputSchema>;

// =============================================================================
// DECISION EVENT SCHEMA (SPECIALIZED)
// =============================================================================

/**
 * Regression Detection Decision Event
 * Extends base DecisionEvent with regression-specific outputs
 */
export const RegressionDetectionDecisionEventSchema = DecisionEventSchema.extend({
  decision_type: z.literal('regression_detection'),
  outputs: RegressionDetectionOutputSchema,
});

export type RegressionDetectionDecisionEvent = z.infer<typeof RegressionDetectionDecisionEventSchema>;

// =============================================================================
// CLI CONTRACT
// =============================================================================

/**
 * CLI invocation shape for Regression Detection Agent
 */
export const RegressionDetectionCLIArgsSchema = z.object({
  /** Path to baseline benchmark results JSON file */
  baseline_file: z.string().optional(),

  /** Path to candidate benchmark results JSON file */
  candidate_file: z.string().optional(),

  /** JSON string for baseline results */
  baseline_json: z.string().optional(),

  /** JSON string for candidate results */
  candidate_json: z.string().optional(),

  /** Read input from stdin (expects { baseline_runs, candidate_runs }) */
  input_stdin: z.boolean().optional(),

  /** Path to thresholds configuration file */
  thresholds_file: z.string().optional(),

  /** Output format */
  output_format: z.enum(['json', 'table', 'summary']).default('json'),

  /** Output file path */
  output_file: z.string().optional(),

  /** Verbose output */
  verbose: z.boolean().default(false),

  /** Quiet mode (only output if regressions detected) */
  quiet: z.boolean().default(false),

  /** Dry run (validate inputs only) */
  dry_run: z.boolean().default(false),

  /** Fail if any regressions detected (for CI/CD) */
  fail_on_regression: z.boolean().default(false),

  /** Minimum severity to fail on (critical, major, minor) */
  fail_severity: z.enum(['critical', 'major', 'minor']).default('major'),
});

export type RegressionDetectionCLIArgs = z.infer<typeof RegressionDetectionCLIArgsSchema>;

// =============================================================================
// CONSTRAINTS & NON-RESPONSIBILITIES
// =============================================================================

/**
 * Constraints that MAY be applied during detection
 */
export const VALID_REGRESSION_CONSTRAINTS = [
  'insufficient_sample_size',     // Not enough data for statistical significance
  'mismatched_models',            // Baseline and candidate have different models
  'no_common_models',             // No models appear in both baseline and candidate
  'missing_metrics',              // Some metrics unavailable for analysis
  'statistical_test_failed',      // Statistical test could not be performed
  'single_baseline_run',          // Only one baseline run (limited confidence)
  'single_candidate_run',         // Only one candidate run (limited confidence)
  'high_variance',                // High variance in data reduces confidence
] as const;

export type RegressionConstraint = typeof VALID_REGRESSION_CONSTRAINTS[number];

/**
 * Explicit non-responsibilities - this agent MUST NOT:
 */
export const REGRESSION_DETECTION_NON_RESPONSIBILITIES = [
  'execute_benchmarks',           // No benchmark execution (that's benchmark-runner)
  'compare_rank_models',          // No model ranking (that's model-comparator)
  'recommend_fixes',              // No fix recommendations (separate agent)
  'predict_future_regressions',   // No predictive analysis
  'enforce_policy',               // No policy decisions
  'orchestrate_workflows',        // No workflow orchestration
  'call_other_agents',            // No direct agent-to-agent calls
  'store_api_keys',               // Never persist API keys
  'modify_benchmark_results',     // No mutation of input data
  'rollback_deployments',         // No deployment actions
] as const;

// =============================================================================
// CONFIDENCE SCORING
// =============================================================================

/**
 * Factors that contribute to regression detection confidence scoring
 */
export const REGRESSION_CONFIDENCE_FACTORS = {
  sample_size: {
    description: 'Total number of executions across baseline and candidate',
    weight: 0.30,
  },
  statistical_significance: {
    description: 'Strength of statistical evidence (1 - p-value)',
    weight: 0.30,
  },
  effect_size: {
    description: 'Magnitude of the detected effect (normalized Cohen\'s d)',
    weight: 0.20,
  },
  data_consistency: {
    description: 'Low variance in measurements (inverse of coefficient of variation)',
    weight: 0.15,
  },
  metric_coverage: {
    description: 'Percentage of metrics successfully analyzed',
    weight: 0.05,
  },
} as const;

/**
 * Calculate confidence score for a regression detection result
 */
export function calculateRegressionConfidence(
  result: ModelRegressionResult,
  baselineTotalExecutions: number,
  candidateTotalExecutions: number
): { confidence: number; factors: Array<{ factor: string; weight: number; value: number }> } {
  const factors: Array<{ factor: string; weight: number; value: number }> = [];

  // Sample size factor (logarithmic scale)
  const totalSamples = baselineTotalExecutions + candidateTotalExecutions;
  const sampleSizeValue = Math.min(1, Math.log10(totalSamples + 1) / 3);
  factors.push({
    factor: 'sample_size',
    weight: REGRESSION_CONFIDENCE_FACTORS.sample_size.weight,
    value: sampleSizeValue,
  });

  // Statistical significance factor (average 1 - p-value)
  const significanceValues = result.metric_regressions
    .map(m => 1 - m.statistical_test.p_value)
    .filter(v => !isNaN(v));
  const avgSignificance = significanceValues.length > 0
    ? significanceValues.reduce((a, b) => a + b, 0) / significanceValues.length
    : 0.5;
  factors.push({
    factor: 'statistical_significance',
    weight: REGRESSION_CONFIDENCE_FACTORS.statistical_significance.weight,
    value: avgSignificance,
  });

  // Effect size factor (normalized)
  const effectSizes = result.metric_regressions
    .map(m => Math.min(1, Math.abs(m.statistical_test.effect_size) / 2))
    .filter(v => !isNaN(v));
  const avgEffectSize = effectSizes.length > 0
    ? effectSizes.reduce((a, b) => a + b, 0) / effectSizes.length
    : 0.5;
  factors.push({
    factor: 'effect_size',
    weight: REGRESSION_CONFIDENCE_FACTORS.effect_size.weight,
    value: avgEffectSize,
  });

  // Data consistency factor (inverse of average coefficient of variation)
  const cvValues = result.metric_regressions
    .filter(m => m.baseline_value !== 0)
    .map(m => Math.max(0, 1 - (m.baseline_stddev / Math.abs(m.baseline_value))));
  const avgConsistency = cvValues.length > 0
    ? cvValues.reduce((a, b) => a + b, 0) / cvValues.length
    : 0.5;
  factors.push({
    factor: 'data_consistency',
    weight: REGRESSION_CONFIDENCE_FACTORS.data_consistency.weight,
    value: Math.min(1, avgConsistency),
  });

  // Metric coverage factor
  const expectedMetrics = 4; // latency, throughput, success_rate, cost
  const metricCoverage = Math.min(1, result.metric_regressions.length / expectedMetrics);
  factors.push({
    factor: 'metric_coverage',
    weight: REGRESSION_CONFIDENCE_FACTORS.metric_coverage.weight,
    value: metricCoverage,
  });

  // Calculate weighted confidence
  const confidence = factors.reduce(
    (sum, f) => sum + f.weight * f.value,
    0
  );

  return {
    confidence: Math.min(1, Math.max(0, confidence)),
    factors,
  };
}

// =============================================================================
// CORE BUNDLE CONSUMERS
// =============================================================================

/**
 * Core bundles that may consume this agent's output
 */
export const REGRESSION_ALLOWED_CONSUMERS = [
  'llm-orchestrator',         // For CI/CD integration
  'llm-observatory',          // For monitoring/alerting
  'llm-analytics',            // For trend analysis
  'llm-test-bench-ui',        // For visualization
  'deployment-gate',          // For deployment decisions
  'alert-manager',            // For regression alerts
] as const;

// =============================================================================
// VERSIONING RULES
// =============================================================================

export const REGRESSION_VERSIONING_RULES = {
  major: 'Breaking changes to input/output schemas or detection algorithm',
  minor: 'New metrics, new statistical tests, new config options',
  patch: 'Bug fixes, threshold adjustments, documentation',
} as const;

// =============================================================================
// STATISTICAL UTILITIES
// =============================================================================

/**
 * Interpret effect size according to Cohen's conventions
 */
export function interpretEffectSize(d: number): 'negligible' | 'small' | 'medium' | 'large' {
  const absD = Math.abs(d);
  if (absD < 0.2) return 'negligible';
  if (absD < 0.5) return 'small';
  if (absD < 0.8) return 'medium';
  return 'large';
}

/**
 * Calculate Cohen's d effect size
 */
export function calculateCohensD(
  mean1: number,
  mean2: number,
  stddev1: number,
  stddev2: number,
  n1: number,
  n2: number
): number {
  // Pooled standard deviation
  const pooledStdDev = Math.sqrt(
    ((n1 - 1) * stddev1 * stddev1 + (n2 - 1) * stddev2 * stddev2) /
    (n1 + n2 - 2)
  );

  if (pooledStdDev === 0) return 0;

  return (mean2 - mean1) / pooledStdDev;
}

/**
 * Perform Welch's t-test
 */
export function welchTTest(
  mean1: number,
  mean2: number,
  stddev1: number,
  stddev2: number,
  n1: number,
  n2: number
): { t: number; df: number; p: number } {
  // Welch's t-statistic
  const se1 = (stddev1 * stddev1) / n1;
  const se2 = (stddev2 * stddev2) / n2;
  const se = Math.sqrt(se1 + se2);

  if (se === 0) {
    return { t: 0, df: n1 + n2 - 2, p: 1 };
  }

  const t = (mean2 - mean1) / se;

  // Welch-Satterthwaite degrees of freedom
  const df = Math.pow(se1 + se2, 2) / (
    (se1 * se1) / (n1 - 1) + (se2 * se2) / (n2 - 1)
  );

  // Approximate p-value using normal distribution for large df
  // For proper implementation, use t-distribution
  const p = 2 * (1 - normalCDF(Math.abs(t)));

  return { t, df, p };
}

/**
 * Standard normal CDF approximation
 */
function normalCDF(x: number): number {
  const a1 = 0.254829592;
  const a2 = -0.284496736;
  const a3 = 1.421413741;
  const a4 = -1.453152027;
  const a5 = 1.061405429;
  const p = 0.3275911;

  const sign = x < 0 ? -1 : 1;
  x = Math.abs(x) / Math.sqrt(2);

  const t = 1.0 / (1.0 + p * x);
  const y = 1.0 - (((((a5 * t + a4) * t) + a3) * t + a2) * t + a1) * t * Math.exp(-x * x);

  return 0.5 * (1.0 + sign * y);
}

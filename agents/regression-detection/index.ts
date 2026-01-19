/**
 * Regression Detection Agent
 *
 * Detect statistically significant regressions between baseline and candidate
 * benchmark runs. Performs comparative analysis to identify performance degradation
 * and produces severity classifications with confidence scores.
 */

export { handler, REGRESSION_DETECTION_AGENT } from './handler';

// Re-export contracts for consumers
export {
  RegressionDetectionInputSchema,
  RegressionDetectionOutputSchema,
  RegressionDetectionDecisionEventSchema,
  RegressionDetectionCLIArgsSchema,
  RegressionThresholdsSchema,
  StatisticalConfigSchema,
  RegressionMetricsConfigSchema,
  RegressionSeveritySchema,
  VALID_REGRESSION_CONSTRAINTS,
  REGRESSION_DETECTION_NON_RESPONSIBILITIES,
  REGRESSION_ALLOWED_CONSUMERS,
  REGRESSION_CONFIDENCE_FACTORS,
  calculateRegressionConfidence,
  calculateCohensD,
  welchTTest,
  interpretEffectSize,
  type RegressionDetectionInput,
  type RegressionDetectionOutput,
  type RegressionThresholds,
  type StatisticalConfig,
  type RegressionMetricsConfig,
  type RegressionSeverity,
  type ModelRegressionResult,
  type MetricRegression,
  type RegressionSummary,
  type StatisticalTestResult,
} from '../contracts';

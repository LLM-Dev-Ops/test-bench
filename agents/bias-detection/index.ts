/**
 * Bias Detection Agent
 *
 * Detect demographic, cultural, or systemic bias in model outputs.
 * Identifies gender, racial, cultural, socioeconomic, age, disability,
 * religious, and other forms of systematic unfairness.
 */

export { handler, BIAS_DETECTION_AGENT } from './handler';
export { executeCLI, CLI_COMMAND_SPEC, SMOKE_TEST_COMMANDS } from './cli';

// Re-export contracts for consumers
export {
  BiasDetectionInputSchema,
  BiasDetectionOutputSchema,
  BiasDetectionDecisionEventSchema,
  BiasDetectionCLIArgsSchema,
  BiasDetectionConfigSchema,
  BiasDetectionStatsSchema,
  BiasSampleResultSchema,
  DetectedBiasSchema,
  BiasEvidenceSchema,
  BiasTypeSchema,
  BiasSeveritySchema,
  BiasDirectionSchema,
  TextSampleSchema,
  DemographicContextSchema,
  BIAS_DETECTION_VALID_CONSTRAINTS,
  BIAS_DETECTION_NON_RESPONSIBILITIES,
  BIAS_DETECTION_ALLOWED_CONSUMERS,
  BIAS_DETECTION_CONFIDENCE_FACTORS,
  BIAS_DETECTION_VERSIONING_RULES,
  BIAS_DETECTION_FAILURE_MODES,
  calculateBiasConfidence,
  type BiasDetectionInput,
  type BiasDetectionOutput,
  type BiasDetectionConfig,
  type BiasDetectionStats,
  type BiasSampleResult,
  type DetectedBias,
  type BiasEvidence,
  type BiasType,
  type BiasSeverity,
  type BiasDirection,
  type TextSample,
  type DemographicContext,
  type BiasDetectionCLIArgs,
  type BiasDetectionConstraint,
} from '../contracts/schemas/bias-detection';

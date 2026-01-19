/**
 * Golden Dataset Validator Agent - Public Exports
 *
 * Validates model outputs against canonical, human-verified datasets.
 */

// Handler exports
export {
  handler,
  GOLDEN_DATASET_VALIDATOR_AGENT,
  CONFIDENCE_FACTORS,
  VALID_CONSTRAINTS,
  // Types
  type EdgeFunctionRequest,
  type EdgeFunctionResponse,
  type GoldenSample,
  type GoldenValidatorModelOutput,
  type ValidationConfig,
  type GoldenDatasetValidatorInput,
  type SampleValidationResult,
  type ValidationStats,
  type GoldenDatasetValidatorOutput,
  type MatchTypeValue,
  type ValidationSeverityValue,
  // Schemas
  GoldenSampleSchema,
  GoldenValidatorModelOutputSchema,
  ValidationConfigSchema,
  GoldenDatasetValidatorInputSchema,
  SampleValidationResultSchema,
  ValidationStatsSchema,
  GoldenDatasetValidatorOutputSchema,
  MatchType,
  ValidationSeverity,
} from './handler';

// CLI exports
export {
  CLI_COMMAND_SPEC,
  executeCLI,
  generateOpenAPISpec,
} from './cli';

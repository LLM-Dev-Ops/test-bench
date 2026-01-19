/**
 * Hallucination Detector Agent
 *
 * Detect unsupported or fabricated claims relative to provided reference context.
 * Identifies fabrication, exaggeration, misattribution, contradiction, and unsupported claims.
 */

export { handler, HALLUCINATION_DETECTOR_AGENT } from './handler';
export { executeCLI, CLI_COMMAND_SPEC } from './cli';

// Re-export contracts for consumers
export {
  HallucinationDetectorInputSchema,
  HallucinationDetectorOutputSchema,
  HallucinationDetectorDecisionEventSchema,
  HallucinationDetectorCLIArgsSchema,
  HallucinationClaimResultSchema,
  HallucinationTypeSchema,
  EvidenceReferenceSchema,
  DetectionConfigSchema,
  ReferenceSourceSchema,
  DetectionSummarySchema,
  VALID_CONSTRAINTS,
  NON_RESPONSIBILITIES,
  ALLOWED_CONSUMERS,
  CONFIDENCE_FACTORS,
  calculateConfidence,
  type HallucinationDetectorInput,
  type HallucinationDetectorOutput,
  type HallucinationClaimResult,
  type HallucinationType,
  type EvidenceReference,
  type DetectionConfig,
  type ReferenceSource,
  type DetectionSummary,
} from '../contracts';

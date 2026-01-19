/**
 * Prompt Sensitivity Agent
 *
 * Measure output variance under controlled prompt perturbations,
 * producing sensitivity analysis metrics for prompt engineering workflows.
 */

export { handler, PROMPT_SENSITIVITY_AGENT } from './handler';

// Re-export contracts for consumers
export {
  PromptSensitivityInputSchema,
  PromptSensitivityOutputSchema,
  PromptSensitivityDecisionEventSchema,
  PROMPT_SENSITIVITY_VALID_CONSTRAINTS,
  PROMPT_SENSITIVITY_NON_RESPONSIBILITIES,
  PROMPT_SENSITIVITY_ALLOWED_CONSUMERS,
  calculateSensitivityConfidence,
  type PromptSensitivityInput,
  type PromptSensitivityOutput,
  type SensitivityProviderConfig,
  type PerturbationType,
  type PerturbationResult,
  type PerturbationRun,
  type PerturbationConfig,
  type SamplingConfig,
  type OverallSensitivity,
} from '../contracts';

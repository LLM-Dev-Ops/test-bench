/**
 * Stress Test Agent
 *
 * Evaluate model robustness under extreme input, load, or adversarial conditions.
 * Produces metrics on failure modes, degradation patterns, and recovery behavior.
 */

export { handler, STRESS_TEST_AGENT } from './handler';

// Re-export contracts for consumers
export {
  StressTestInputSchema,
  StressTestOutputSchema,
  StressTestDecisionEventSchema,
  STRESS_TEST_VALID_CONSTRAINTS,
  STRESS_TEST_NON_RESPONSIBILITIES,
  STRESS_TEST_ALLOWED_CONSUMERS,
  calculateStressTestConfidence,
  FAILURE_MODE_METADATA,
  type StressTestInput,
  type StressTestOutput,
  type StressTestProviderConfig,
  type StressTestScenario,
  type StressTestExecutionConfig,
  type StressRequestResult,
  type ScenarioResult,
  type ProviderRobustnessSummary,
  type BreakingPoint,
  type FailureMode,
  type StressTestType,
} from '../contracts';

/**
 * Execution span module for the Agentics Foundational Execution Unit.
 *
 * This module ensures this repository NEVER executes silently.
 * All externally-invoked operations must emit agent-level execution spans
 * and integrate into a hierarchical ExecutionGraph produced by a Core.
 *
 * @packageDocumentation
 */

// Core types
export type {
  SpanType,
  SpanStatus,
  ExecutionContext,
  ExecutionSpan,
  ExecutionArtifact,
  ExecutionResult,
} from './types.js';
export { generateSpanId, nowISO, REPO_NAME } from './types.js';

// Span management
export {
  SpanManager,
  validateExecutionContext,
  ExecutionContextError,
  ExecutionInvariantError,
} from './span-manager.js';

// Instrumented entry points
export { InstrumentedTestBench, executeWithSpans } from './instrumented.js';

// Policy gate
export { PolicyGate, PolicyGateError } from './policy-gate.js';
export type { PolicyCheckResult } from './policy-gate.js';
export {
  SimulationIntentSchema,
  GateDispositionSchema,
  PolicyRuleSchema,
  ExecutionPolicySchema,
  ExemptionRecordSchema,
  ExemptionRequestSchema,
} from './policy.js';
export type {
  SimulationIntent,
  GateDisposition,
  PolicyRule,
  ExecutionPolicy,
  ExemptionRecord,
  ExemptionRequest,
} from './policy.js';
export { DEFAULT_EXECUTION_POLICY } from './default-policy.js';

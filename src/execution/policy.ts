/**
 * Policy gate types for the Agentics Foundational Execution Unit.
 *
 * Defines the declarative policy that classifies simulation intents
 * as mandatory, recommended, or optional execution dependencies.
 * When a mandatory intent is bypassed, an ExemptionRecord must be
 * produced as a first-class artifact in the execution graph.
 */

import { z } from 'zod';

// =============================================================================
// SIMULATION INTENT
// =============================================================================

/**
 * All recognized simulation intents.
 * Union of InstrumentedTestBench method intents and stress test types.
 */
export const SimulationIntentSchema = z.enum([
  // InstrumentedTestBench methods
  'benchmark',
  'compare',
  'evaluate',
  'complete',
  'optimize',
  // Stress test types
  'load_ramp',
  'spike',
  'soak',
  'extreme_input',
  'adversarial',
  'rate_limit_probe',
  'timeout_boundary',
  'token_limit',
  'context_overflow',
  'malformed_request',
]);

export type SimulationIntent = z.infer<typeof SimulationIntentSchema>;

// =============================================================================
// GATE DISPOSITION
// =============================================================================

/**
 * Gate disposition for a given intent.
 * - 'mandatory': MUST run through InstrumentedTestBench; bypass is an error
 *   unless an exemption is explicitly granted.
 * - 'recommended': Should run instrumented, but bypass does not fail.
 * - 'optional': No policy enforcement.
 */
export const GateDispositionSchema = z.enum([
  'mandatory',
  'recommended',
  'optional',
]);

export type GateDisposition = z.infer<typeof GateDispositionSchema>;

// =============================================================================
// POLICY RULES
// =============================================================================

/**
 * A single policy rule binding an intent to a gate disposition.
 */
export const PolicyRuleSchema = z.object({
  intent: SimulationIntentSchema,
  disposition: GateDispositionSchema,
  rationale: z.string().min(1),
});

export type PolicyRule = z.infer<typeof PolicyRuleSchema>;

// =============================================================================
// EXECUTION POLICY
// =============================================================================

/**
 * The complete execution policy. Declarative, JSON-serializable,
 * inspectable at rest.
 */
export const ExecutionPolicySchema = z.object({
  policy_version: z.string().regex(/^\d+\.\d+\.\d+$/),
  updated_at: z.string().datetime(),
  description: z.string(),
  rules: z.array(PolicyRuleSchema).refine(
    (rules: Array<{ intent: string; disposition: string; rationale: string }>) => {
      const intents = rules.map((r: { intent: string }) => r.intent);
      return new Set(intents).size === intents.length;
    },
    { message: 'Duplicate intent in policy rules' }
  ),
  default_disposition: GateDispositionSchema,
});

export type ExecutionPolicy = z.infer<typeof ExecutionPolicySchema>;

// =============================================================================
// EXEMPTION RECORDS
// =============================================================================

/**
 * An exemption record. Created when a mandatory intent is explicitly
 * waived. This is a FIRST-CLASS record in the execution graph,
 * attached as an artifact to the relevant span.
 */
export const ExemptionRecordSchema = z.object({
  intent: SimulationIntentSchema,
  reason: z.string().min(1),
  granted_by: z.string().min(1),
  granted_at: z.string().datetime(),
  expires_at: z.string().datetime().optional(),
  execution_id: z.string(),
  parent_span_id: z.string(),
});

export type ExemptionRecord = z.infer<typeof ExemptionRecordSchema>;

/**
 * Exemption request passed by callers who need to bypass a mandatory gate.
 */
export const ExemptionRequestSchema = z.object({
  intent: SimulationIntentSchema,
  reason: z.string().min(1),
  granted_by: z.string().min(1),
});

export type ExemptionRequest = z.infer<typeof ExemptionRequestSchema>;

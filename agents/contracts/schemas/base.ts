/**
 * Base schemas for all Agentics Dev platform agents.
 * These schemas define the canonical structures that ALL agents must use.
 */

import { z } from 'zod';

// =============================================================================
// CORE IDENTITY SCHEMAS
// =============================================================================

/**
 * Agent identifier with version
 */
export const AgentIdentifierSchema = z.object({
  agent_id: z.string().regex(/^[a-z][a-z0-9-]*[a-z0-9]$/, 'Agent ID must be kebab-case'),
  agent_version: z.string().regex(/^\d+\.\d+\.\d+$/, 'Version must be semver format'),
});

/**
 * Execution reference for tracing
 */
export const ExecutionRefSchema = z.object({
  execution_id: z.string().uuid(),
  trace_id: z.string().uuid().optional(),
  span_id: z.string().optional(),
  parent_span_id: z.string().optional(),
});

// =============================================================================
// DECISION EVENT SCHEMA (REQUIRED FOR ALL AGENTS)
// =============================================================================

/**
 * DecisionEvent - Every agent MUST emit exactly ONE of these per invocation.
 * This is the canonical record persisted to ruvector-service.
 */
export const DecisionEventSchema = z.object({
  // Identity
  agent_id: z.string(),
  agent_version: z.string(),

  // Decision metadata
  decision_type: z.string(),
  decision_id: z.string().uuid(),

  // Inputs (hashed for privacy/size)
  inputs_hash: z.string().length(64, 'SHA-256 hash required'),
  inputs_summary: z.record(z.unknown()).optional(),

  // Outputs
  outputs: z.record(z.unknown()),

  // Confidence and constraints
  confidence: z.number().min(0).max(1),
  confidence_factors: z.array(z.object({
    factor: z.string(),
    weight: z.number().min(0).max(1),
    value: z.number().min(0).max(1),
  })).optional(),

  constraints_applied: z.array(z.string()),

  // Execution context
  execution_ref: ExecutionRefSchema,

  // Timing
  timestamp: z.string().datetime(),
  duration_ms: z.number().nonnegative(),

  // Error state (if any)
  error: z.object({
    code: z.string(),
    message: z.string(),
    recoverable: z.boolean(),
  }).optional(),
});

export type DecisionEvent = z.infer<typeof DecisionEventSchema>;

// =============================================================================
// TELEMETRY SCHEMA
// =============================================================================

/**
 * Telemetry event compatible with LLM-Observatory
 */
export const TelemetryEventSchema = z.object({
  event_type: z.enum([
    'agent_invoked',
    'agent_completed',
    'agent_error',
    'decision_emitted',
    'validation_failed',
    'constraint_applied',
  ]),
  agent_id: z.string(),
  agent_version: z.string(),
  execution_ref: ExecutionRefSchema,
  timestamp: z.string().datetime(),

  // Metrics
  metrics: z.record(z.number()).optional(),

  // Labels for filtering
  labels: z.record(z.string()).optional(),

  // Additional context
  context: z.record(z.unknown()).optional(),
});

export type TelemetryEvent = z.infer<typeof TelemetryEventSchema>;

// =============================================================================
// ERROR SCHEMAS
// =============================================================================

export const AgentErrorSchema = z.object({
  code: z.enum([
    'VALIDATION_ERROR',
    'EXECUTION_ERROR',
    'TIMEOUT_ERROR',
    'PROVIDER_ERROR',
    'CONFIGURATION_ERROR',
    'PERSISTENCE_ERROR',
    'UNKNOWN_ERROR',
  ]),
  message: z.string(),
  details: z.record(z.unknown()).optional(),
  recoverable: z.boolean(),
  timestamp: z.string().datetime(),
});

export type AgentError = z.infer<typeof AgentErrorSchema>;

// =============================================================================
// VALIDATION UTILITIES
// =============================================================================

/**
 * Validate input against a schema, returning a standardized result
 */
export function validateInput<T>(
  schema: z.ZodSchema<T>,
  input: unknown
): { success: true; data: T } | { success: false; error: AgentError } {
  const result = schema.safeParse(input);

  if (result.success) {
    return { success: true, data: result.data };
  }

  return {
    success: false,
    error: {
      code: 'VALIDATION_ERROR',
      message: 'Input validation failed',
      details: {
        issues: result.error.issues.map(i => ({
          path: i.path.join('.'),
          message: i.message,
        })),
      },
      recoverable: true,
      timestamp: new Date().toISOString(),
    },
  };
}

/**
 * Create SHA-256 hash of inputs for DecisionEvent
 */
export async function hashInputs(inputs: unknown): Promise<string> {
  const encoder = new TextEncoder();
  const data = encoder.encode(JSON.stringify(inputs));
  const hashBuffer = await crypto.subtle.digest('SHA-256', data);
  const hashArray = Array.from(new Uint8Array(hashBuffer));
  return hashArray.map(b => b.toString(16).padStart(2, '0')).join('');
}

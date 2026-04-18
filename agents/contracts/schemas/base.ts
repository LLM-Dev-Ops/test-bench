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
 * Agent Identity Schema - REQUIRED for all DecisionEvents
 * Phase 1 / Layer 1 standardization
 */
export const AgentIdentitySchema = z.object({
  source_agent: z.string().min(1, 'source_agent is required'),
  domain: z.string().min(1, 'domain is required'),
  phase: z.enum(['phase1', 'phase2', 'phase3']),
  layer: z.enum(['layer1', 'layer2', 'layer3']),
});

export type AgentIdentity = z.infer<typeof AgentIdentitySchema>;

/**
 * DecisionEvent - Every agent MUST emit exactly ONE of these per invocation.
 * This is the canonical record persisted to ruvector-service.
 *
 * IMPORTANT: DecisionEvents MUST emit SIGNALS, NOT conclusions.
 * - Include: event_type, confidence (0-1), evidence_refs
 * - Do NOT include: summaries, recommendations, synthesis
 */
export const DecisionEventSchema = z.object({
  // Identity (REQUIRED - No anonymous agents)
  agent_id: z.string(),
  agent_version: z.string(),

  // Agent Identity Standardization (PHASE 1 / LAYER 1 REQUIREMENT)
  source_agent: z.string().min(1, 'source_agent is required'),
  domain: z.string().min(1, 'domain is required'),
  phase: z.enum(['phase1', 'phase2', 'phase3']),
  layer: z.enum(['layer1', 'layer2', 'layer3']),

  // Decision metadata
  decision_type: z.string(),
  decision_id: z.string().uuid(),

  // Event type for signal classification
  event_type: z.string().optional(),

  // Evidence references (for signal traceability)
  evidence_refs: z.array(z.string()).optional(),

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

// =============================================================================
// AGENT IDENTITY HELPERS
// =============================================================================

type Phase = 'phase1' | 'phase2' | 'phase3';
type Layer = 'layer1' | 'layer2' | 'layer3';

function coercePhase(v: string | undefined): Phase {
  return v === 'phase2' || v === 'phase3' ? v : 'phase1';
}

function coerceLayer(v: string | undefined): Layer {
  return v === 'layer2' || v === 'layer3' ? v : 'layer1';
}

/**
 * Resolve the AgentIdentity fields (source_agent, domain, phase, layer)
 * required on every DecisionEvent. Pulls from AGENT_* env vars set at
 * deploy time, falling back to safe defaults.
 */
export function getAgentIdentity(agentId: string): AgentIdentity {
  return {
    source_agent: process.env.AGENT_NAME || agentId,
    domain: process.env.AGENT_DOMAIN || 'llm-test-bench',
    phase: coercePhase(process.env.AGENT_PHASE),
    layer: coerceLayer(process.env.AGENT_LAYER),
  };
}

/**
 * Clamp a numeric value into [0, 1], coercing NaN/undefined to `fallback`.
 * Used for confidence and confidence_factors where n<2 sample math can
 * produce NaN (0/0 from stddev/mean ratios).
 */
export function sanitizeConfidence(value: number | undefined | null, fallback = 0): number {
  if (value === null || value === undefined || !Number.isFinite(value)) {
    return fallback;
  }
  if (value < 0) return 0;
  if (value > 1) return 1;
  return value;
}

/**
 * Coerce possibly-NaN statistics for optional numeric output fields.
 * Returns null when value is non-finite (so JSON serialization produces
 * `null` rather than `NaN`, which is not valid JSON).
 */
export function finiteOrNull(value: number | undefined | null): number | null {
  if (value === null || value === undefined || !Number.isFinite(value)) {
    return null;
  }
  return value;
}

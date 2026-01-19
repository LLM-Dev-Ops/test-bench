/**
 * Telemetry Service
 *
 * Emits telemetry events compatible with LLM-Observatory.
 * All agents MUST emit telemetry for observability.
 */

import {
  TelemetryEvent,
  TelemetryEventSchema,
  ExecutionRefSchema,
} from '../contracts';
import { getRuVectorClient } from './ruvector-client';

// =============================================================================
// TYPES
// =============================================================================

export type TelemetryEventType =
  | 'agent_invoked'
  | 'agent_completed'
  | 'agent_error'
  | 'decision_emitted'
  | 'validation_failed'
  | 'constraint_applied';

export interface TelemetryContext {
  agent_id: string;
  agent_version: string;
  execution_id: string;
  trace_id?: string;
  span_id?: string;
  parent_span_id?: string;
}

export interface TelemetryMetrics {
  duration_ms?: number;
  tokens_used?: number;
  cost_usd?: number;
  success_count?: number;
  failure_count?: number;
  latency_p50_ms?: number;
  latency_p95_ms?: number;
  [key: string]: number | undefined;
}

export interface TelemetryLabels {
  provider?: string;
  model?: string;
  suite_id?: string;
  environment?: string;
  [key: string]: string | undefined;
}

// =============================================================================
// TELEMETRY EMITTER
// =============================================================================

export class TelemetryEmitter {
  private context: TelemetryContext;
  private events: TelemetryEvent[] = [];
  private flushed = false;

  constructor(context: TelemetryContext) {
    this.context = context;
  }

  /**
   * Emit a telemetry event
   */
  emit(
    eventType: TelemetryEventType,
    options: {
      metrics?: TelemetryMetrics;
      labels?: TelemetryLabels;
      context?: Record<string, unknown>;
    } = {}
  ): void {
    const event: TelemetryEvent = {
      event_type: eventType,
      agent_id: this.context.agent_id,
      agent_version: this.context.agent_version,
      execution_ref: {
        execution_id: this.context.execution_id,
        trace_id: this.context.trace_id,
        span_id: this.context.span_id,
        parent_span_id: this.context.parent_span_id,
      },
      timestamp: new Date().toISOString(),
      metrics: this.cleanMetrics(options.metrics),
      labels: this.cleanLabels(options.labels),
      context: options.context,
    };

    // Validate before storing
    const validation = TelemetryEventSchema.safeParse(event);
    if (!validation.success) {
      console.error('[Telemetry] Invalid event:', validation.error);
      return;
    }

    this.events.push(validation.data);
  }

  /**
   * Emit agent invocation event
   */
  emitInvoked(labels?: TelemetryLabels): void {
    this.emit('agent_invoked', { labels });
  }

  /**
   * Emit agent completion event with metrics
   */
  emitCompleted(metrics: TelemetryMetrics, labels?: TelemetryLabels): void {
    this.emit('agent_completed', { metrics, labels });
  }

  /**
   * Emit agent error event
   */
  emitError(
    errorCode: string,
    errorMessage: string,
    context?: Record<string, unknown>
  ): void {
    this.emit('agent_error', {
      labels: { error_code: errorCode },
      context: { error_message: errorMessage, ...context },
    });
  }

  /**
   * Emit decision emitted event
   */
  emitDecision(decisionId: string, confidence: number): void {
    this.emit('decision_emitted', {
      metrics: { confidence },
      labels: { decision_id: decisionId },
    });
  }

  /**
   * Emit validation failure event
   */
  emitValidationFailed(
    field: string,
    message: string,
    context?: Record<string, unknown>
  ): void {
    this.emit('validation_failed', {
      labels: { validation_field: field },
      context: { validation_message: message, ...context },
    });
  }

  /**
   * Emit constraint applied event
   */
  emitConstraintApplied(constraint: string, reason?: string): void {
    this.emit('constraint_applied', {
      labels: { constraint },
      context: reason ? { reason } : undefined,
    });
  }

  /**
   * Flush all events to ruvector-service
   */
  async flush(): Promise<void> {
    if (this.flushed) {
      console.warn('[Telemetry] Already flushed');
      return;
    }

    const client = getRuVectorClient();

    // Fire-and-forget all events
    await Promise.allSettled(
      this.events.map(event => client.persistTelemetryEvent(event))
    );

    this.flushed = true;
    this.events = [];
  }

  /**
   * Get all collected events (for testing/debugging)
   */
  getEvents(): TelemetryEvent[] {
    return [...this.events];
  }

  // ===========================================================================
  // PRIVATE HELPERS
  // ===========================================================================

  private cleanMetrics(
    metrics?: TelemetryMetrics
  ): Record<string, number> | undefined {
    if (!metrics) return undefined;

    const cleaned: Record<string, number> = {};
    for (const [key, value] of Object.entries(metrics)) {
      if (typeof value === 'number' && !isNaN(value)) {
        cleaned[key] = value;
      }
    }

    return Object.keys(cleaned).length > 0 ? cleaned : undefined;
  }

  private cleanLabels(
    labels?: TelemetryLabels
  ): Record<string, string> | undefined {
    if (!labels) return undefined;

    const cleaned: Record<string, string> = {};
    for (const [key, value] of Object.entries(labels)) {
      if (typeof value === 'string' && value.length > 0) {
        cleaned[key] = value;
      }
    }

    return Object.keys(cleaned).length > 0 ? cleaned : undefined;
  }
}

// =============================================================================
// FACTORY FUNCTION
// =============================================================================

export function createTelemetryEmitter(
  agentId: string,
  agentVersion: string,
  executionId: string
): TelemetryEmitter {
  return new TelemetryEmitter({
    agent_id: agentId,
    agent_version: agentVersion,
    execution_id: executionId,
  });
}

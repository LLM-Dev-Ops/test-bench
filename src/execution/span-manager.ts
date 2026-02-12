/**
 * Span manager for the Agentics Foundational Execution Unit.
 *
 * Manages the lifecycle of execution spans, enforces invariants,
 * and produces the final ExecutionResult.
 *
 * Invariants enforced:
 * - parent_span_id must be present and non-empty
 * - At least one agent span must be emitted for a valid execution
 * - Agents must not share spans
 * - Repo span is created on entry, closed on exit
 * - All spans are append-only and causally ordered
 */

import type {
  ExecutionContext,
  ExecutionSpan,
  ExecutionArtifact,
  ExecutionResult,
  SpanStatus,
} from './types.js';
import { generateSpanId, nowISO, REPO_NAME } from './types.js';
import type { ExemptionRecord } from './policy.js';

/**
 * Validates that an ExecutionContext is structurally valid.
 * Rejects execution if parent_span_id is missing.
 */
export function validateExecutionContext(ctx: unknown): ctx is ExecutionContext {
  if (!ctx || typeof ctx !== 'object') return false;
  const c = ctx as Record<string, unknown>;
  return (
    typeof c['execution_id'] === 'string' &&
    c['execution_id'].length > 0 &&
    typeof c['parent_span_id'] === 'string' &&
    c['parent_span_id'].length > 0
  );
}

/**
 * Error thrown when execution context is missing or invalid.
 */
export class ExecutionContextError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'ExecutionContextError';
  }
}

/**
 * Error thrown when execution invariants are violated.
 */
export class ExecutionInvariantError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'ExecutionInvariantError';
  }
}

/**
 * Manages execution spans for a single repo-level invocation.
 *
 * Usage:
 *   const mgr = SpanManager.create(ctx);
 *   const agentSpan = mgr.startAgent('benchmark-runner');
 *   // ... do work ...
 *   mgr.addArtifact({ reference: 'results.json', artifact_type: 'benchmark', agent_span_id: agentSpan.span_id });
 *   mgr.endAgent(agentSpan.span_id, 'completed');
 *   const result = mgr.finalize(data);
 */
export class SpanManager {
  private readonly repoSpan: ExecutionSpan;
  private readonly agentSpans: Map<string, ExecutionSpan> = new Map();
  private readonly artifacts: ExecutionArtifact[] = [];
  private readonly exemptions: ExemptionRecord[] = [];
  private readonly activeAgents: Set<string> = new Set();
  private finalized = false;

  private constructor(ctx: ExecutionContext) {
    this.repoSpan = {
      type: 'repo',
      repo_name: REPO_NAME,
      span_id: generateSpanId(),
      parent_span_id: ctx.parent_span_id,
      start_time: nowISO(),
      status: 'running',
    };
  }

  /**
   * Create a new SpanManager for an incoming execution.
   * Rejects if parent_span_id is missing or invalid.
   */
  static create(ctx: unknown): SpanManager {
    if (!validateExecutionContext(ctx)) {
      throw new ExecutionContextError(
        'Execution rejected: parent_span_id is missing or invalid. ' +
          'Every externally-invoked operation must provide a valid ExecutionContext ' +
          'with execution_id and parent_span_id.'
      );
    }
    return new SpanManager(ctx);
  }

  /** Get the repo-level span ID */
  get repoSpanId(): string {
    return this.repoSpan.span_id;
  }

  /** Get the execution_id from the parent context */
  get executionId(): string {
    // Stored implicitly via the parent relationship
    return this.repoSpan.parent_span_id;
  }

  /**
   * Start a new agent-level span.
   * Each agent gets its own unique span. Agents must not share spans.
   */
  startAgent(agentName: string): ExecutionSpan {
    if (this.finalized) {
      throw new ExecutionInvariantError('Cannot start agent after finalization');
    }

    const span: ExecutionSpan = {
      type: 'agent',
      agent_name: agentName,
      repo_name: REPO_NAME,
      span_id: generateSpanId(),
      parent_span_id: this.repoSpan.span_id,
      start_time: nowISO(),
      status: 'running',
    };

    this.agentSpans.set(span.span_id, span);
    this.activeAgents.add(span.span_id);
    return { ...span };
  }

  /**
   * End an agent-level span.
   */
  endAgent(
    spanId: string,
    status: 'completed' | 'failed',
    failureReasons?: string[]
  ): void {
    const span = this.agentSpans.get(spanId);
    if (!span) {
      throw new ExecutionInvariantError(`Unknown agent span: ${spanId}`);
    }
    if (span.status !== 'running') {
      throw new ExecutionInvariantError(
        `Agent span ${spanId} already ended with status ${span.status}`
      );
    }

    span.end_time = nowISO();
    span.status = status;
    if (failureReasons) {
      span.failure_reasons = failureReasons;
    }
    this.activeAgents.delete(spanId);
  }

  /**
   * Attach an artifact to an agent span.
   * Artifacts must never be attached directly to the repo span.
   */
  addArtifact(artifact: ExecutionArtifact): void {
    if (!this.agentSpans.has(artifact.agent_span_id)) {
      throw new ExecutionInvariantError(
        `Cannot attach artifact to unknown agent span: ${artifact.agent_span_id}. ` +
          'Artifacts must be attached to an existing agent-level span.'
      );
    }
    this.artifacts.push(artifact);
  }

  /**
   * Record a policy exemption for this execution.
   * Exemptions appear in the final ExecutionResult.exemptions array.
   */
  addExemption(record: ExemptionRecord): void {
    this.exemptions.push(record);
  }

  /**
   * Finalize the execution and produce the result.
   *
   * Enforces:
   * - At least one agent span was emitted
   * - No agents are still running
   * - All invariants are satisfied
   */
  finalize<T>(data?: T, forceStatus?: SpanStatus): ExecutionResult<T> {
    if (this.finalized) {
      throw new ExecutionInvariantError('Already finalized');
    }
    this.finalized = true;

    const validationErrors: string[] = [];

    // Enforce: at least one agent span must exist
    if (this.agentSpans.size === 0) {
      validationErrors.push(
        'No agent-level spans were emitted. Execution is INVALID.'
      );
    }

    // Enforce: no running agents
    if (this.activeAgents.size > 0) {
      const running = Array.from(this.activeAgents)
        .map((id) => this.agentSpans.get(id)?.agent_name ?? id)
        .join(', ');
      validationErrors.push(
        `Agents still running at finalization: ${running}. ` +
          'All agent spans must be ended before finalization.'
      );
      // Force-end them as failed
      for (const id of this.activeAgents) {
        this.endAgent(id, 'failed', [
          'Agent was still running at finalization',
        ]);
      }
    }

    // Check if any agent executed without a span (structural check)
    // This is enforced by the API: you cannot execute logic without calling startAgent

    // Determine repo span status
    const anyFailed = Array.from(this.agentSpans.values()).some(
      (s) => s.status === 'failed'
    );
    const repoStatus: SpanStatus =
      forceStatus ??
      (validationErrors.length > 0
        ? 'failed'
        : anyFailed
          ? 'failed'
          : 'completed');

    this.repoSpan.end_time = nowISO();
    this.repoSpan.status = repoStatus;
    if (repoStatus === 'failed') {
      this.repoSpan.failure_reasons = validationErrors.length > 0
        ? validationErrors
        : Array.from(this.agentSpans.values())
            .filter((s) => s.status === 'failed')
            .flatMap((s) => s.failure_reasons ?? ['Agent failed']);
    }

    const valid = validationErrors.length === 0;

    return {
      repo_span: { ...this.repoSpan },
      agent_spans: Array.from(this.agentSpans.values()).map((s) => ({ ...s })),
      artifacts: [...this.artifacts],
      data: valid ? data : undefined,
      valid,
      validation_errors: validationErrors.length > 0 ? validationErrors : undefined,
      exemptions: this.exemptions.length > 0 ? [...this.exemptions] : undefined,
    };
  }

  /**
   * Finalize as failed with explicit reasons.
   * Still returns all emitted spans per the contract.
   */
  finalizeFailure(reasons: string[]): ExecutionResult<never> {
    if (!this.finalized) {
      // End any running agents
      for (const id of this.activeAgents) {
        this.endAgent(id, 'failed', reasons);
      }
    }

    this.finalized = true;
    this.repoSpan.end_time = nowISO();
    this.repoSpan.status = 'failed';
    this.repoSpan.failure_reasons = reasons;

    return {
      repo_span: { ...this.repoSpan },
      agent_spans: Array.from(this.agentSpans.values()).map((s) => ({ ...s })),
      artifacts: [...this.artifacts],
      valid: false,
      validation_errors: reasons,
      exemptions: this.exemptions.length > 0 ? [...this.exemptions] : undefined,
    };
  }
}

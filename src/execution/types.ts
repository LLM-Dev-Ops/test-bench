/**
 * Execution span types for the Agentics Foundational Execution Unit.
 *
 * This module defines the core types that enable this repository to emit
 * agent-level execution spans and integrate into a hierarchical
 * ExecutionGraph produced by a Core.
 *
 * Span hierarchy invariant:
 *   Core
 *     └─ Repo (this repo)
 *         └─ Agent (one or more)
 */

import { randomUUID } from 'crypto';
import type { ExemptionRecord } from './policy.js';

/** Span type discriminator */
export type SpanType = 'repo' | 'agent';

/** Span status */
export type SpanStatus = 'running' | 'completed' | 'failed';

/**
 * Execution context provided by the calling Core.
 * Every externally-invoked operation MUST receive this.
 */
export interface ExecutionContext {
  /** Unique identifier for this execution graph */
  execution_id: string;

  /** Span ID from the Core that parents this repo's span */
  parent_span_id: string;
}

/**
 * A single execution span within the graph.
 * JSON-serializable, append-only, causally ordered via parent_span_id.
 */
export interface ExecutionSpan {
  /** Discriminator: "repo" or "agent" */
  type: SpanType;

  /** Unique span identifier */
  span_id: string;

  /** Parent span (Core span for repo, repo span for agents) */
  parent_span_id: string;

  /** ISO-8601 start time */
  start_time: string;

  /** ISO-8601 end time (set on completion/failure) */
  end_time?: string;

  /** Current status */
  status: SpanStatus;

  /** Repo-level: name of this repository */
  repo_name?: string;

  /** Agent-level: name of the agent that executed */
  agent_name?: string;

  /** Failure reasons (populated when status is "failed") */
  failure_reasons?: string[];
}

/**
 * An artifact produced during execution.
 * MUST be attached to the agent-level span that produced it.
 */
export interface ExecutionArtifact {
  /** Stable reference: ID, URI, hash, or filename */
  reference: string;

  /** Artifact type descriptor */
  artifact_type: string;

  /** Span ID of the agent that produced this artifact */
  agent_span_id: string;

  /** Optional metadata */
  metadata?: Record<string, unknown>;
}

/**
 * Complete execution result returned by this repo.
 * Includes the repo-level span, nested agent-level spans,
 * and artifacts with evidence attached at correct levels.
 */
export interface ExecutionResult<T = unknown> {
  /** The repo-level span */
  repo_span: ExecutionSpan;

  /** Nested agent-level spans (one per agent that executed) */
  agent_spans: ExecutionSpan[];

  /** Artifacts produced, each attached to an agent span */
  artifacts: ExecutionArtifact[];

  /** The operation's return data (if successful) */
  data?: T;

  /** Whether the execution is valid per invariants */
  valid: boolean;

  /** Validation errors (if invalid) */
  validation_errors?: string[];

  /** Policy exemptions granted during this execution (if any) */
  exemptions?: ExemptionRecord[];
}

/** Generate a new span ID */
export function generateSpanId(): string {
  return randomUUID();
}

/** Get current ISO-8601 timestamp */
export function nowISO(): string {
  return new Date().toISOString();
}

/** The canonical name for this repository */
export const REPO_NAME = 'llm-test-bench';

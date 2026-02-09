/**
 * Tests for the Agentics Foundational Execution Unit instrumentation.
 *
 * Verifies all invariants:
 * - parent_span_id must be present
 * - At least one agent span must be emitted
 * - Agents must not share spans
 * - Artifacts attached to correct agent spans
 * - Repo span parents to Core span
 * - Agent spans parent to repo span
 * - No silent execution allowed
 */

import { describe, it, expect } from 'vitest';
import {
  SpanManager,
  validateExecutionContext,
  ExecutionContextError,
  ExecutionInvariantError,
  REPO_NAME,
} from '../src/execution/index.js';
import type { ExecutionContext, ExecutionSpan } from '../src/execution/index.js';

const validCtx: ExecutionContext = {
  execution_id: 'exec-test-001',
  parent_span_id: 'core-span-001',
};

describe('ExecutionContext validation', () => {
  it('accepts valid context', () => {
    expect(validateExecutionContext(validCtx)).toBe(true);
  });

  it('rejects null', () => {
    expect(validateExecutionContext(null)).toBe(false);
  });

  it('rejects undefined', () => {
    expect(validateExecutionContext(undefined)).toBe(false);
  });

  it('rejects empty execution_id', () => {
    expect(
      validateExecutionContext({ execution_id: '', parent_span_id: 'x' })
    ).toBe(false);
  });

  it('rejects empty parent_span_id', () => {
    expect(
      validateExecutionContext({ execution_id: 'x', parent_span_id: '' })
    ).toBe(false);
  });

  it('rejects missing fields', () => {
    expect(validateExecutionContext({})).toBe(false);
    expect(validateExecutionContext({ execution_id: 'x' })).toBe(false);
    expect(validateExecutionContext({ parent_span_id: 'x' })).toBe(false);
  });

  it('rejects non-string fields', () => {
    expect(
      validateExecutionContext({ execution_id: 123, parent_span_id: 'x' })
    ).toBe(false);
  });
});

describe('SpanManager creation', () => {
  it('creates successfully with valid context', () => {
    const mgr = SpanManager.create(validCtx);
    expect(mgr).toBeDefined();
    expect(mgr.repoSpanId).toBeTruthy();
  });

  it('throws ExecutionContextError with missing parent_span_id', () => {
    expect(() =>
      SpanManager.create({ execution_id: 'x', parent_span_id: '' })
    ).toThrow(ExecutionContextError);
  });

  it('throws ExecutionContextError with null context', () => {
    expect(() => SpanManager.create(null)).toThrow(ExecutionContextError);
  });

  it('error message mentions parent_span_id', () => {
    try {
      SpanManager.create({ execution_id: 'x', parent_span_id: '' });
      expect.fail('should have thrown');
    } catch (e) {
      expect((e as Error).message).toContain('parent_span_id');
    }
  });
});

describe('Agent span lifecycle', () => {
  it('creates agent span with correct parent', () => {
    const mgr = SpanManager.create(validCtx);
    const agent = mgr.startAgent('test-agent');

    expect(agent.type).toBe('agent');
    expect(agent.agent_name).toBe('test-agent');
    expect(agent.repo_name).toBe(REPO_NAME);
    expect(agent.status).toBe('running');
    expect(agent.parent_span_id).toBe(mgr.repoSpanId);
    expect(agent.span_id).toBeTruthy();
    expect(agent.start_time).toBeTruthy();
  });

  it('each agent gets a unique span', () => {
    const mgr = SpanManager.create(validCtx);
    const a1 = mgr.startAgent('agent-1');
    const a2 = mgr.startAgent('agent-2');

    expect(a1.span_id).not.toBe(a2.span_id);
  });

  it('ends agent span successfully', () => {
    const mgr = SpanManager.create(validCtx);
    const agent = mgr.startAgent('agent');
    mgr.endAgent(agent.span_id, 'completed');

    const result = mgr.finalize('ok');
    const found = result.agent_spans.find(
      (s) => s.span_id === agent.span_id
    );
    expect(found?.status).toBe('completed');
    expect(found?.end_time).toBeTruthy();
  });

  it('ends agent span as failed with reasons', () => {
    const mgr = SpanManager.create(validCtx);
    const agent = mgr.startAgent('agent');
    mgr.endAgent(agent.span_id, 'failed', ['something broke']);

    const result = mgr.finalize(undefined, 'failed');
    const found = result.agent_spans.find(
      (s) => s.span_id === agent.span_id
    );
    expect(found?.status).toBe('failed');
    expect(found?.failure_reasons).toContain('something broke');
  });

  it('throws on unknown span_id', () => {
    const mgr = SpanManager.create(validCtx);
    mgr.startAgent('agent');
    expect(() => mgr.endAgent('nonexistent', 'completed')).toThrow(
      ExecutionInvariantError
    );
  });

  it('throws on double-end', () => {
    const mgr = SpanManager.create(validCtx);
    const agent = mgr.startAgent('agent');
    mgr.endAgent(agent.span_id, 'completed');
    expect(() => mgr.endAgent(agent.span_id, 'completed')).toThrow(
      ExecutionInvariantError
    );
  });
});

describe('Artifact attachment', () => {
  it('attaches artifact to valid agent span', () => {
    const mgr = SpanManager.create(validCtx);
    const agent = mgr.startAgent('producer');

    mgr.addArtifact({
      reference: 'output.json',
      artifact_type: 'results',
      agent_span_id: agent.span_id,
      metadata: { size: 1024 },
    });

    mgr.endAgent(agent.span_id, 'completed');
    const result = mgr.finalize('ok');

    expect(result.artifacts).toHaveLength(1);
    expect(result.artifacts[0].reference).toBe('output.json');
    expect(result.artifacts[0].agent_span_id).toBe(agent.span_id);
  });

  it('throws when attaching to unknown agent span', () => {
    const mgr = SpanManager.create(validCtx);
    mgr.startAgent('real-agent');

    expect(() =>
      mgr.addArtifact({
        reference: 'bad.json',
        artifact_type: 'results',
        agent_span_id: 'nonexistent',
      })
    ).toThrow(ExecutionInvariantError);
  });

  it('multiple artifacts on same agent', () => {
    const mgr = SpanManager.create(validCtx);
    const agent = mgr.startAgent('producer');

    mgr.addArtifact({
      reference: 'a.json',
      artifact_type: 'data',
      agent_span_id: agent.span_id,
    });
    mgr.addArtifact({
      reference: 'b.json',
      artifact_type: 'metrics',
      agent_span_id: agent.span_id,
    });

    mgr.endAgent(agent.span_id, 'completed');
    const result = mgr.finalize(null);
    expect(result.artifacts).toHaveLength(2);
  });
});

describe('Finalization and enforcement', () => {
  it('valid execution: one agent completed', () => {
    const mgr = SpanManager.create(validCtx);
    const agent = mgr.startAgent('worker');
    mgr.endAgent(agent.span_id, 'completed');

    const result = mgr.finalize('success');
    expect(result.valid).toBe(true);
    expect(result.data).toBe('success');
    expect(result.repo_span.status).toBe('completed');
    expect(result.repo_span.repo_name).toBe(REPO_NAME);
    expect(result.repo_span.type).toBe('repo');
    expect(result.agent_spans).toHaveLength(1);
    expect(result.validation_errors).toBeUndefined();
  });

  it('INVALID: no agent spans emitted', () => {
    const mgr = SpanManager.create(validCtx);
    const result = mgr.finalize('data');

    expect(result.valid).toBe(false);
    expect(result.data).toBeUndefined();
    expect(result.repo_span.status).toBe('failed');
    expect(result.validation_errors).toBeDefined();
    expect(result.validation_errors?.some((e) => e.includes('No agent-level spans'))).toBe(true);
  });

  it('INVALID: running agents at finalization', () => {
    const mgr = SpanManager.create(validCtx);
    mgr.startAgent('still-running');

    const result = mgr.finalize('data');
    // Still valid because agent span exists, but agents were force-ended
    expect(result.repo_span.status).toBe('failed');
    expect(result.agent_spans[0].status).toBe('failed');
  });

  it('failure propagates from agent to repo', () => {
    const mgr = SpanManager.create(validCtx);
    const agent = mgr.startAgent('failing');
    mgr.endAgent(agent.span_id, 'failed', ['crash']);

    const result = mgr.finalize(undefined);
    expect(result.repo_span.status).toBe('failed');
    expect(result.repo_span.failure_reasons).toBeDefined();
  });

  it('finalizeFailure marks everything as failed', () => {
    const mgr = SpanManager.create(validCtx);
    mgr.startAgent('doomed');

    const result = mgr.finalizeFailure(['fatal error']);
    expect(result.valid).toBe(false);
    expect(result.repo_span.status).toBe('failed');
    expect(result.repo_span.failure_reasons).toContain('fatal error');
  });

  it('throws on double finalization', () => {
    const mgr = SpanManager.create(validCtx);
    const agent = mgr.startAgent('agent');
    mgr.endAgent(agent.span_id, 'completed');
    mgr.finalize('ok');

    expect(() => mgr.finalize('again')).toThrow(ExecutionInvariantError);
  });

  it('cannot start agent after finalization', () => {
    const mgr = SpanManager.create(validCtx);
    const agent = mgr.startAgent('agent');
    mgr.endAgent(agent.span_id, 'completed');
    mgr.finalize('ok');

    expect(() => mgr.startAgent('late-agent')).toThrow(ExecutionInvariantError);
  });
});

describe('Causal ordering (parent_span_id chain)', () => {
  it('repo span parents to Core span', () => {
    const mgr = SpanManager.create(validCtx);
    const agent = mgr.startAgent('x');
    mgr.endAgent(agent.span_id, 'completed');

    const result = mgr.finalize(null);
    expect(result.repo_span.parent_span_id).toBe('core-span-001');
  });

  it('agent spans parent to repo span', () => {
    const mgr = SpanManager.create(validCtx);
    const a1 = mgr.startAgent('a1');
    const a2 = mgr.startAgent('a2');
    mgr.endAgent(a1.span_id, 'completed');
    mgr.endAgent(a2.span_id, 'completed');

    const result = mgr.finalize(null);
    for (const agentSpan of result.agent_spans) {
      expect(agentSpan.parent_span_id).toBe(result.repo_span.span_id);
    }
  });
});

describe('JSON serialization', () => {
  it('result is JSON-serializable without loss', () => {
    const mgr = SpanManager.create(validCtx);
    const agent = mgr.startAgent('serializer');
    mgr.addArtifact({
      reference: 'data.bin',
      artifact_type: 'binary',
      agent_span_id: agent.span_id,
      metadata: { hash: 'abc123' },
    });
    mgr.endAgent(agent.span_id, 'completed');

    const result = mgr.finalize({ key: 'value' });
    const json = JSON.stringify(result);
    const parsed = JSON.parse(json);

    expect(parsed.valid).toBe(true);
    expect(parsed.repo_span.type).toBe('repo');
    expect(parsed.agent_spans).toHaveLength(1);
    expect(parsed.agent_spans[0].type).toBe('agent');
    expect(parsed.artifacts).toHaveLength(1);
    expect(parsed.data).toEqual({ key: 'value' });
  });
});

describe('Multiple agents', () => {
  it('supports multiple concurrent agents', () => {
    const mgr = SpanManager.create(validCtx);
    const agents: ExecutionSpan[] = [];

    for (let i = 0; i < 5; i++) {
      agents.push(mgr.startAgent(`agent-${i}`));
    }

    for (const agent of agents) {
      mgr.endAgent(agent.span_id, 'completed');
    }

    const result = mgr.finalize('done');
    expect(result.valid).toBe(true);
    expect(result.agent_spans).toHaveLength(5);

    // All span IDs are unique
    const ids = new Set(result.agent_spans.map((s) => s.span_id));
    expect(ids.size).toBe(5);
  });

  it('partial failure: some agents succeed, some fail', () => {
    const mgr = SpanManager.create(validCtx);
    const ok = mgr.startAgent('ok-agent');
    const bad = mgr.startAgent('bad-agent');

    mgr.endAgent(ok.span_id, 'completed');
    mgr.endAgent(bad.span_id, 'failed', ['oops']);

    const result = mgr.finalize('partial');
    expect(result.valid).toBe(true); // valid because spans exist
    expect(result.repo_span.status).toBe('failed'); // failed because an agent failed
    expect(result.agent_spans).toHaveLength(2);
  });
});

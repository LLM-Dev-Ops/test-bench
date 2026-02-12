/**
 * Tests for the policy-gated execution dependency.
 *
 * Verifies:
 * - Default policy classifies intents correctly
 * - Mandatory intents are blocked without context or exemption
 * - Exemptions are recorded as first-class artifacts in the execution graph
 * - PolicyGate.enforce() works with InstrumentedTestBench
 * - Policy and exemption records are JSON-serializable
 */

import { describe, it, expect } from 'vitest';
import {
  PolicyGate,
  PolicyGateError,
  DEFAULT_EXECUTION_POLICY,
  SpanManager,
} from '../src/execution/index.js';
import type {
  SimulationIntent,
  ExecutionPolicy,
  ExemptionRequest,
  ExemptionRecord,
} from '../src/execution/index.js';
import type { ExecutionContext } from '../src/execution/index.js';

const validCtx: ExecutionContext = {
  execution_id: 'exec-test-001',
  parent_span_id: 'core-span-001',
};

// =============================================================================
// DEFAULT POLICY CLASSIFICATION
// =============================================================================

describe('Default policy classification', () => {
  const gate = new PolicyGate();
  const policy = gate.getPolicy();

  it('has correct version', () => {
    expect(policy.policy_version).toBe('1.0.0');
  });

  it('has 15 rules', () => {
    expect(policy.rules.length).toBe(15);
  });

  const mandatoryIntents: SimulationIntent[] = [
    'benchmark',
    'compare',
    'evaluate',
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
  ];

  for (const intent of mandatoryIntents) {
    it(`marks '${intent}' as mandatory`, () => {
      const rule = policy.rules.find((r) => r.intent === intent);
      expect(rule).toBeDefined();
      expect(rule!.disposition).toBe('mandatory');
      expect(rule!.rationale.length).toBeGreaterThan(0);
    });
  }

  const recommendedIntents: SimulationIntent[] = ['complete', 'optimize'];

  for (const intent of recommendedIntents) {
    it(`marks '${intent}' as recommended`, () => {
      const rule = policy.rules.find((r) => r.intent === intent);
      expect(rule).toBeDefined();
      expect(rule!.disposition).toBe('recommended');
    });
  }

  it('default disposition is recommended', () => {
    expect(policy.default_disposition).toBe('recommended');
  });

  it('has no duplicate intents', () => {
    const intents = policy.rules.map((r) => r.intent);
    expect(new Set(intents).size).toBe(intents.length);
  });
});

// =============================================================================
// POLICY GATE CHECKS
// =============================================================================

describe('PolicyGate.check()', () => {
  const gate = new PolicyGate();

  it('allows mandatory intent WITH execution context', () => {
    const result = gate.check('benchmark', validCtx);
    expect(result.allowed).toBe(true);
    expect(result.disposition).toBe('mandatory');
    expect(result.exemption).toBeUndefined();
    expect(result.error).toBeUndefined();
  });

  it('blocks mandatory intent WITHOUT context and WITHOUT exemption', () => {
    const result = gate.check('benchmark', null);
    expect(result.allowed).toBe(false);
    expect(result.disposition).toBe('mandatory');
    expect(result.error).toBeDefined();
    expect(result.error).toContain("Intent 'benchmark'");
    expect(result.error).toContain('mandatory');
  });

  it('allows mandatory intent WITHOUT context but WITH valid exemption', () => {
    const exemption: ExemptionRequest = {
      intent: 'benchmark',
      reason: 'Emergency hotfix validation, no Core available',
      granted_by: 'operator:alice',
    };
    const result = gate.check('benchmark', null, exemption);
    expect(result.allowed).toBe(true);
    expect(result.disposition).toBe('mandatory');
    expect(result.exemption).toBeDefined();
    expect(result.exemption!.intent).toBe('benchmark');
    expect(result.exemption!.reason).toBe(exemption.reason);
    expect(result.exemption!.granted_by).toBe(exemption.granted_by);
    expect(result.exemption!.execution_id).toBe('EXEMPTED');
    expect(result.exemption!.parent_span_id).toBe('EXEMPTED');
  });

  it('allows recommended intent without context', () => {
    const result = gate.check('complete', null);
    expect(result.allowed).toBe(true);
    expect(result.disposition).toBe('recommended');
  });

  it('allows optional intent without context', () => {
    // Use a custom policy with an optional intent
    const customPolicy: ExecutionPolicy = {
      ...DEFAULT_EXECUTION_POLICY,
      rules: [
        {
          intent: 'benchmark',
          disposition: 'optional',
          rationale: 'test',
        },
      ],
    };
    const customGate = new PolicyGate(customPolicy);
    const result = customGate.check('benchmark', null);
    expect(result.allowed).toBe(true);
    expect(result.disposition).toBe('optional');
  });

  it('blocks all stress test intents without context', () => {
    const stressIntents: SimulationIntent[] = [
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
    ];

    for (const intent of stressIntents) {
      const result = gate.check(intent, null);
      expect(result.allowed).toBe(false);
      expect(result.error).toContain(intent);
    }
  });
});

// =============================================================================
// POLICY GATE ENFORCEMENT
// =============================================================================

describe('PolicyGate.enforce()', () => {
  const gate = new PolicyGate();

  it('returns SpanManager for valid context', () => {
    const { mgr } = gate.enforce('benchmark', validCtx);
    expect(mgr).toBeDefined();
    expect(mgr.repoSpanId).toBeTruthy();
  });

  it('works for all intent types with valid context', () => {
    const intents: SimulationIntent[] = [
      'benchmark',
      'compare',
      'evaluate',
      'complete',
      'optimize',
    ];
    for (const intent of intents) {
      const { mgr } = gate.enforce(intent, validCtx);
      expect(mgr).toBeDefined();
    }
  });
});

// =============================================================================
// EXEMPTION RECORDS IN EXECUTION GRAPH
// =============================================================================

describe('Exemption recording', () => {
  const gate = new PolicyGate();

  it('records exemption as artifact and in exemptions array', () => {
    const exemption: ExemptionRequest = {
      intent: 'benchmark',
      reason: 'CI pipeline emergency bypass',
      granted_by: 'system:ci-override',
    };

    const checkResult = gate.check('benchmark', null, exemption);
    expect(checkResult.allowed).toBe(true);
    expect(checkResult.exemption).toBeDefined();

    // Now simulate recording the exemption on a SpanManager
    const mgr = SpanManager.create(validCtx);
    const agent = mgr.startAgent('policy-gate-exemption');
    gate.recordExemption(mgr, agent.span_id, checkResult.exemption!);
    mgr.endAgent(agent.span_id, 'completed');
    const result = mgr.finalize(null);

    // Exemption appears in artifacts
    const exemptionArtifacts = result.artifacts.filter(
      (a) => a.artifact_type === 'policy_exemption'
    );
    expect(exemptionArtifacts).toHaveLength(1);
    expect(exemptionArtifacts[0].agent_span_id).toBe(agent.span_id);

    // Exemption appears in result.exemptions
    expect(result.exemptions).toBeDefined();
    expect(result.exemptions).toHaveLength(1);
    expect(result.exemptions![0].intent).toBe('benchmark');
    expect(result.exemptions![0].reason).toBe(exemption.reason);
    expect(result.exemptions![0].granted_by).toBe(exemption.granted_by);
  });

  it('no exemptions array when none recorded', () => {
    const mgr = SpanManager.create(validCtx);
    const agent = mgr.startAgent('normal-agent');
    mgr.endAgent(agent.span_id, 'completed');
    const result = mgr.finalize('ok');
    expect(result.exemptions).toBeUndefined();
  });
});

// =============================================================================
// CUSTOM POLICY
// =============================================================================

describe('Custom policy override', () => {
  it('accepts custom policy', () => {
    const customPolicy: ExecutionPolicy = {
      policy_version: '2.0.0',
      updated_at: '2026-02-12T12:00:00.000Z',
      description: 'Custom policy for testing',
      rules: [
        {
          intent: 'benchmark',
          disposition: 'optional',
          rationale: 'test override',
        },
        {
          intent: 'complete',
          disposition: 'mandatory',
          rationale: 'strict mode',
        },
      ],
      default_disposition: 'optional',
    };

    const gate = new PolicyGate(customPolicy);
    const policy = gate.getPolicy();
    expect(policy.policy_version).toBe('2.0.0');

    // benchmark is now optional
    const benchResult = gate.check('benchmark', null);
    expect(benchResult.allowed).toBe(true);

    // complete is now mandatory
    const completeResult = gate.check('complete', null);
    expect(completeResult.allowed).toBe(false);
  });
});

// =============================================================================
// SERIALIZATION
// =============================================================================

describe('Serialization', () => {
  it('policy is JSON-serializable', () => {
    const gate = new PolicyGate();
    const policy = gate.getPolicy();
    const json = JSON.stringify(policy);
    const parsed = JSON.parse(json);
    expect(parsed.policy_version).toBe('1.0.0');
    expect(parsed.rules).toHaveLength(15);
    expect(parsed.default_disposition).toBe('recommended');
  });

  it('exemption record is JSON-serializable', () => {
    const gate = new PolicyGate();
    const result = gate.check('benchmark', null, {
      intent: 'benchmark',
      reason: 'test',
      granted_by: 'tester',
    });
    const json = JSON.stringify(result.exemption);
    const parsed = JSON.parse(json);
    expect(parsed.intent).toBe('benchmark');
    expect(parsed.execution_id).toBe('EXEMPTED');
  });

  it('execution result with exemption is JSON-serializable', () => {
    const gate = new PolicyGate();
    const checkResult = gate.check('benchmark', null, {
      intent: 'benchmark',
      reason: 'test',
      granted_by: 'tester',
    });

    const mgr = SpanManager.create(validCtx);
    const agent = mgr.startAgent('test-agent');
    gate.recordExemption(mgr, agent.span_id, checkResult.exemption!);
    mgr.endAgent(agent.span_id, 'completed');
    const execResult = mgr.finalize({ status: 'ok' });

    const json = JSON.stringify(execResult);
    const parsed = JSON.parse(json);
    expect(parsed.exemptions).toHaveLength(1);
    expect(parsed.artifacts.some((a: any) => a.artifact_type === 'policy_exemption')).toBe(true);
    expect(parsed.valid).toBe(true);
  });
});

// =============================================================================
// ERROR TYPES
// =============================================================================

describe('PolicyGateError', () => {
  it('has correct name and properties', () => {
    const err = new PolicyGateError('test error', 'benchmark', 'mandatory');
    expect(err.name).toBe('PolicyGateError');
    expect(err.intent).toBe('benchmark');
    expect(err.disposition).toBe('mandatory');
    expect(err.message).toBe('test error');
    expect(err instanceof Error).toBe(true);
  });
});

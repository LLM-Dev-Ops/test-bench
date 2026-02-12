/**
 * PolicyGate enforcer for the Agentics Foundational Execution Unit.
 *
 * Checks the execution policy before any simulation intent proceeds.
 * For mandatory intents, either an ExecutionContext must be present or
 * an ExemptionRequest must be provided. Exemptions are recorded as
 * first-class artifacts in the execution graph.
 */

import type { ExecutionContext, ExecutionArtifact } from './types.js';
import type {
  SimulationIntent,
  GateDisposition,
  ExecutionPolicy,
  ExemptionRequest,
  ExemptionRecord,
} from './policy.js';
import { SpanManager } from './span-manager.js';
import { DEFAULT_EXECUTION_POLICY } from './default-policy.js';
import { nowISO } from './types.js';

/**
 * Error thrown when a mandatory policy gate is not satisfied.
 */
export class PolicyGateError extends Error {
  public readonly intent: SimulationIntent;
  public readonly disposition: GateDisposition;

  constructor(
    message: string,
    intent: SimulationIntent,
    disposition: GateDisposition,
  ) {
    super(message);
    this.name = 'PolicyGateError';
    this.intent = intent;
    this.disposition = disposition;
  }
}

/**
 * Result of a policy gate check.
 */
export interface PolicyCheckResult {
  allowed: boolean;
  disposition: GateDisposition;
  exemption?: ExemptionRecord;
  error?: string;
}

/**
 * PolicyGate sits between callers and InstrumentedTestBench.
 *
 * It checks the intent against the policy BEFORE execution proceeds.
 * If the intent is mandatory and no ExecutionContext is provided,
 * callers must supply an ExemptionRequest -- otherwise the check fails.
 */
export class PolicyGate {
  private readonly policy: ExecutionPolicy;

  constructor(policy?: ExecutionPolicy) {
    this.policy = policy ?? DEFAULT_EXECUTION_POLICY;
  }

  /** Get the current policy (for inspection / serialization). */
  getPolicy(): Readonly<ExecutionPolicy> {
    return this.policy;
  }

  /**
   * Check whether an intent may proceed without an ExecutionContext.
   *
   * @param intent - The simulation intent being requested
   * @param ctx - The execution context (null if caller is trying to bypass)
   * @param exemption - An optional exemption request if ctx is null
   */
  check(
    intent: SimulationIntent,
    ctx: ExecutionContext | null,
    exemption?: ExemptionRequest,
  ): PolicyCheckResult {
    const rule = this.policy.rules.find((r: { intent: string }) => r.intent === intent);
    const disposition = rule?.disposition ?? this.policy.default_disposition;

    // If context is present, always allowed (properly instrumented path)
    if (ctx !== null) {
      return { allowed: true, disposition };
    }

    // No context -- evaluate disposition
    switch (disposition) {
      case 'optional':
        return { allowed: true, disposition };

      case 'recommended':
        return { allowed: true, disposition };

      case 'mandatory': {
        if (!exemption) {
          return {
            allowed: false,
            disposition,
            error:
              `Intent '${intent}' has mandatory policy gate. ` +
              `Provide an ExecutionContext or an ExemptionRequest. ` +
              `Rationale: ${rule?.rationale ?? 'policy default'}`,
          };
        }

        const record: ExemptionRecord = {
          intent,
          reason: exemption.reason,
          granted_by: exemption.granted_by,
          granted_at: nowISO(),
          execution_id: 'EXEMPTED',
          parent_span_id: 'EXEMPTED',
        };

        return { allowed: true, disposition, exemption: record };
      }

      default:
        return { allowed: true, disposition };
    }
  }

  /**
   * Enforce the policy gate for an instrumented execution.
   *
   * Called by InstrumentedTestBench before delegating to the inner
   * LLMTestBench. Since ctx is always provided on this path,
   * this validates and creates the SpanManager.
   */
  enforce(
    intent: SimulationIntent,
    ctx: ExecutionContext,
  ): { mgr: SpanManager } {
    const result = this.check(intent, ctx);
    if (!result.allowed) {
      throw new PolicyGateError(
        result.error ?? 'Policy gate check failed',
        intent,
        result.disposition,
      );
    }
    return { mgr: SpanManager.create(ctx) };
  }

  /**
   * Record an exemption as a first-class artifact on a SpanManager.
   */
  recordExemption(
    mgr: SpanManager,
    agentSpanId: string,
    exemption: ExemptionRecord,
  ): void {
    const artifact: ExecutionArtifact = {
      reference: `policy-exemption-${exemption.intent}-${Date.now()}.json`,
      artifact_type: 'policy_exemption',
      agent_span_id: agentSpanId,
      metadata: { ...exemption },
    };
    mgr.addArtifact(artifact);
    mgr.addExemption(exemption);
  }
}

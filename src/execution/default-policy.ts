/**
 * Default execution policy for LLM-Test-Bench.
 *
 * This is the canonical policy enforced when no override is provided.
 * Benchmark, compare, evaluate, and all stress test intents are mandatory.
 * Complete and optimize are recommended but not hard-gated.
 */

import type { ExecutionPolicy } from './policy.js';

export const DEFAULT_EXECUTION_POLICY: ExecutionPolicy = {
  policy_version: '1.0.0',
  updated_at: '2026-02-12T00:00:00.000Z',
  description:
    'Default policy: benchmark, compare, evaluate, and all stress test intents ' +
    'are mandatory execution dependencies. Complete and optimize are recommended.',
  rules: [
    // Core simulation intents - mandatory
    {
      intent: 'benchmark',
      disposition: 'mandatory',
      rationale: 'Benchmark results must appear in execution graph for auditability',
    },
    {
      intent: 'compare',
      disposition: 'mandatory',
      rationale: 'Model comparisons are decision-critical and must be traced',
    },
    {
      intent: 'evaluate',
      disposition: 'mandatory',
      rationale: 'Evaluations are quality gates and must be traced',
    },
    // Core intents - recommended
    {
      intent: 'complete',
      disposition: 'recommended',
      rationale: 'Completions should be traced but may be used for ad-hoc exploration',
    },
    {
      intent: 'optimize',
      disposition: 'recommended',
      rationale: 'Optimization is advisory; should be traced but not hard-gated',
    },
    // Stress test intents - ALL mandatory
    {
      intent: 'load_ramp',
      disposition: 'mandatory',
      rationale: 'Load ramp results are critical for capacity planning',
    },
    {
      intent: 'spike',
      disposition: 'mandatory',
      rationale: 'Spike test results inform resilience decisions',
    },
    {
      intent: 'soak',
      disposition: 'mandatory',
      rationale: 'Soak test results inform stability decisions',
    },
    {
      intent: 'extreme_input',
      disposition: 'mandatory',
      rationale: 'Extreme input handling must be auditable',
    },
    {
      intent: 'adversarial',
      disposition: 'mandatory',
      rationale: 'Adversarial test results are security-critical',
    },
    {
      intent: 'rate_limit_probe',
      disposition: 'mandatory',
      rationale: 'Rate limit probing affects operational parameters',
    },
    {
      intent: 'timeout_boundary',
      disposition: 'mandatory',
      rationale: 'Timeout boundary results set SLA parameters',
    },
    {
      intent: 'token_limit',
      disposition: 'mandatory',
      rationale: 'Token limit results inform context window policies',
    },
    {
      intent: 'context_overflow',
      disposition: 'mandatory',
      rationale: 'Context overflow results are input validation critical',
    },
    {
      intent: 'malformed_request',
      disposition: 'mandatory',
      rationale: 'Error handling behavior must be auditable',
    },
  ],
  default_disposition: 'recommended',
};

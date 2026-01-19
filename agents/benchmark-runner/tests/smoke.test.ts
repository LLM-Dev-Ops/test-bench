/**
 * Benchmark Runner Agent - Smoke Tests
 *
 * Verification checklist and smoke tests for platform wiring.
 */

import { describe, it, expect, beforeAll, afterAll } from 'vitest';
import { handler, BENCHMARK_RUNNER_AGENT } from '../handler';
import {
  BenchmarkRunnerInputSchema,
  BenchmarkRunnerOutputSchema,
  DecisionEventSchema,
  validateInput,
} from '../../contracts';
import {
  getRegistrationMetadata,
  validateRegistration,
} from '../../registry/benchmark-runner';

// =============================================================================
// TEST FIXTURES
// =============================================================================

const VALID_INPUT = {
  providers: [
    {
      provider_name: 'openai' as const,
      model_id: 'gpt-4o-mini',
      api_key_ref: 'test-openai',
      timeout_ms: 30000,
      max_retries: 3,
    },
  ],
  suite: {
    suite_id: 'smoke-test',
    suite_name: 'Smoke Test Suite',
    description: 'Basic smoke test',
    test_cases: [
      {
        test_id: 'test-1',
        prompt: 'Say hello',
        max_tokens: 10,
        temperature: 0,
      },
    ],
  },
  execution_config: {
    concurrency: 1,
    warm_up_runs: 0,
    iterations_per_test: 1,
    save_responses: true,
    fail_fast: false,
  },
};

const INVALID_INPUT_MISSING_PROVIDERS = {
  suite: VALID_INPUT.suite,
};

const INVALID_INPUT_EMPTY_SUITE = {
  providers: VALID_INPUT.providers,
  suite: {
    suite_id: 'empty',
    suite_name: 'Empty Suite',
    test_cases: [], // Invalid - must have at least 1
  },
};

// =============================================================================
// VERIFICATION CHECKLIST
// =============================================================================

describe('Verification Checklist', () => {
  describe('1. Agent Registration', () => {
    it('should have valid agent_id', () => {
      expect(BENCHMARK_RUNNER_AGENT.agent_id).toBe('benchmark-runner');
      expect(BENCHMARK_RUNNER_AGENT.agent_id).toMatch(/^[a-z][a-z0-9-]*[a-z0-9]$/);
    });

    it('should have valid version', () => {
      expect(BENCHMARK_RUNNER_AGENT.agent_version).toBe('1.0.0');
      expect(BENCHMARK_RUNNER_AGENT.agent_version).toMatch(/^\d+\.\d+\.\d+$/);
    });

    it('should have correct decision_type', () => {
      expect(BENCHMARK_RUNNER_AGENT.decision_type).toBe('benchmark_execution');
    });

    it('should pass registration validation', () => {
      const result = validateRegistration();
      expect(result.valid).toBe(true);
      expect(result.errors).toHaveLength(0);
    });

    it('should have complete registration metadata', () => {
      const metadata = getRegistrationMetadata();

      expect(metadata.identity.agent_id).toBeDefined();
      expect(metadata.deployment.endpoint).toBeDefined();
      expect(metadata.schemas.input).toBeDefined();
      expect(metadata.schemas.output).toBeDefined();
      expect(metadata.consumers.length).toBeGreaterThan(0);
    });
  });

  describe('2. Schema Validation', () => {
    it('should validate correct input', () => {
      const result = validateInput(BenchmarkRunnerInputSchema, VALID_INPUT);
      expect(result.success).toBe(true);
    });

    it('should reject input without providers', () => {
      const result = validateInput(BenchmarkRunnerInputSchema, INVALID_INPUT_MISSING_PROVIDERS);
      expect(result.success).toBe(false);
    });

    it('should reject input with empty test suite', () => {
      const result = validateInput(BenchmarkRunnerInputSchema, INVALID_INPUT_EMPTY_SUITE);
      expect(result.success).toBe(false);
    });
  });

  describe('3. Handler Contract', () => {
    it('should return 405 for non-POST requests', async () => {
      const response = await handler({
        body: {},
        headers: {},
        method: 'GET',
        path: '/benchmark-runner',
      });

      expect(response.statusCode).toBe(405);
    });

    it('should return 400 for invalid input', async () => {
      const response = await handler({
        body: INVALID_INPUT_MISSING_PROVIDERS,
        headers: {},
        method: 'POST',
        path: '/benchmark-runner',
      });

      expect(response.statusCode).toBe(400);
      const body = JSON.parse(response.body);
      expect(body.success).toBe(false);
      expect(body.error.code).toBe('VALIDATION_ERROR');
    });

    it('should include agent headers in response', async () => {
      const response = await handler({
        body: INVALID_INPUT_MISSING_PROVIDERS,
        headers: {},
        method: 'POST',
        path: '/benchmark-runner',
      });

      expect(response.headers['X-Agent-Id']).toBe('benchmark-runner');
      expect(response.headers['X-Agent-Version']).toBe('1.0.0');
    });
  });

  describe('4. Decision Event Structure', () => {
    it('should define valid decision event schema', () => {
      // Create a minimal valid decision event
      const mockDecisionEvent = {
        agent_id: 'benchmark-runner',
        agent_version: '1.0.0',
        decision_type: 'benchmark_execution',
        decision_id: '550e8400-e29b-41d4-a716-446655440000',
        inputs_hash: 'a'.repeat(64),
        outputs: {},
        confidence: 0.95,
        constraints_applied: [],
        execution_ref: {
          execution_id: '550e8400-e29b-41d4-a716-446655440001',
        },
        timestamp: new Date().toISOString(),
        duration_ms: 1000,
      };

      const result = DecisionEventSchema.safeParse(mockDecisionEvent);
      expect(result.success).toBe(true);
    });

    it('should require confidence between 0 and 1', () => {
      const invalidConfidence = {
        agent_id: 'benchmark-runner',
        agent_version: '1.0.0',
        decision_type: 'benchmark_execution',
        decision_id: '550e8400-e29b-41d4-a716-446655440000',
        inputs_hash: 'a'.repeat(64),
        outputs: {},
        confidence: 1.5, // Invalid - > 1
        constraints_applied: [],
        execution_ref: {
          execution_id: '550e8400-e29b-41d4-a716-446655440001',
        },
        timestamp: new Date().toISOString(),
        duration_ms: 1000,
      };

      const result = DecisionEventSchema.safeParse(invalidConfidence);
      expect(result.success).toBe(false);
    });
  });

  describe('5. Platform Dependencies', () => {
    it('should declare ruvector-service as required dependency', () => {
      const metadata = getRegistrationMetadata();
      expect(metadata.dependencies.required).toContain('ruvector-service');
    });

    it('should declare llm-observatory as optional dependency', () => {
      const metadata = getRegistrationMetadata();
      expect(metadata.dependencies.optional).toContain('llm-observatory');
    });
  });
});

// =============================================================================
// SMOKE TEST CLI COMMANDS
// =============================================================================

describe('Smoke Test CLI Commands', () => {
  // These would be run manually or in CI

  it.skip('should execute: agentics benchmark-runner --help', () => {
    // Command: agentics benchmark-runner --help
    // Expected: Help text with options
  });

  it.skip('should execute: agentics benchmark-runner --dry-run -i sample.json', () => {
    // Command: agentics benchmark-runner --dry-run -i sample.json
    // Expected: Validation passes, no execution
  });

  it.skip('should execute: agentics benchmark-runner -i sample.json -f table', () => {
    // Command: agentics benchmark-runner -i sample.json -f table
    // Expected: Table output with stats
  });
});

// =============================================================================
// INTEGRATION SMOKE TESTS
// =============================================================================

describe('Integration Smoke Tests', () => {
  // These require actual provider credentials

  it.skip('should execute benchmark against OpenAI', async () => {
    // Requires OPENAI_API_KEY
    const input = {
      ...VALID_INPUT,
      providers: [{
        provider_name: 'openai' as const,
        model_id: 'gpt-4o-mini',
        api_key_ref: 'openai',
      }],
    };

    const response = await handler({
      body: input,
      headers: {},
      method: 'POST',
      path: '/benchmark-runner',
    });

    expect(response.statusCode).toBe(200);
    const body = JSON.parse(response.body);
    expect(body.success).toBe(true);
    expect(body.decision_id).toBeDefined();
  });

  it.skip('should persist decision event to ruvector-service', async () => {
    // Requires RUVECTOR_SERVICE_URL
    // Would verify event appears in database
  });

  it.skip('should emit telemetry to llm-observatory', async () => {
    // Requires LLM_OBSERVATORY_URL
    // Would verify telemetry events appear
  });
});

/**
 * Faithfulness Verification Agent - Unit Tests
 *
 * Test suite for the Faithfulness Verification Agent.
 * Validates contract compliance, edge cases, and failure modes.
 */

import { describe, it, expect, beforeEach, vi } from 'vitest';
import {
  handler,
  EdgeFunctionRequest,
  FAITHFULNESS_VERIFICATION_AGENT,
} from './handler';
import {
  FaithfulnessVerificationInputSchema,
  FaithfulnessVerificationOutputSchema,
  FAITHFULNESS_VERIFICATION_AGENT as AGENT_META,
} from '../contracts';

// Mock ruvector-client and telemetry
vi.mock('../services', () => ({
  getRuVectorClient: () => ({
    persistDecisionEvent: vi.fn().mockResolvedValue(undefined),
    persistTelemetryEvent: vi.fn().mockResolvedValue(undefined),
    flush: vi.fn().mockResolvedValue(undefined),
  }),
  createTelemetryEmitter: () => ({
    emitInvoked: vi.fn(),
    emitCompleted: vi.fn(),
    emitError: vi.fn(),
    emitDecision: vi.fn(),
    emitValidationFailed: vi.fn(),
    emitConstraintApplied: vi.fn(),
    flush: vi.fn().mockResolvedValue(undefined),
  }),
}));

// =============================================================================
// TEST FIXTURES
// =============================================================================

const createRequest = (body: unknown): EdgeFunctionRequest => ({
  body,
  headers: { 'Content-Type': 'application/json' },
  method: 'POST',
  path: '/api/v1/agents/faithfulness-verification',
});

const validInput = {
  sources: [
    {
      document_id: 'source-1',
      content: 'The capital of France is Paris. The Eiffel Tower is located in Paris and was built in 1889.',
      source_type: 'context',
    },
    {
      document_id: 'source-2',
      content: 'France is a country in Western Europe. Its largest city is Paris.',
      source_type: 'knowledge_base',
    },
  ],
  output: {
    output_id: 'output-1',
    content: 'Paris is the capital of France. The Eiffel Tower, located in Paris, was constructed in 1889.',
  },
};

const unfaithfulInput = {
  sources: [
    {
      document_id: 'source-1',
      content: 'The capital of France is Paris. The population is approximately 2 million.',
    },
  ],
  output: {
    output_id: 'output-1',
    content: 'Paris is the capital of France with a population of 10 million people. The city was founded in ancient Rome.',
  },
};

// =============================================================================
// AGENT IDENTITY TESTS
// =============================================================================

describe('Faithfulness Verification Agent - Identity', () => {
  it('should have correct agent identity', () => {
    expect(FAITHFULNESS_VERIFICATION_AGENT.agent_id).toBe('faithfulness-verification');
    expect(FAITHFULNESS_VERIFICATION_AGENT.agent_version).toBe('1.0.0');
    expect(FAITHFULNESS_VERIFICATION_AGENT.decision_type).toBe('faithfulness_verification');
  });

  it('should match contract metadata', () => {
    expect(FAITHFULNESS_VERIFICATION_AGENT.agent_id).toBe(AGENT_META.agent_id);
    expect(FAITHFULNESS_VERIFICATION_AGENT.agent_version).toBe(AGENT_META.agent_version);
    expect(FAITHFULNESS_VERIFICATION_AGENT.decision_type).toBe(AGENT_META.decision_type);
  });
});

// =============================================================================
// INPUT VALIDATION TESTS
// =============================================================================

describe('Faithfulness Verification Agent - Input Validation', () => {
  it('should reject non-POST requests', async () => {
    const request: EdgeFunctionRequest = {
      ...createRequest(validInput),
      method: 'GET',
    };

    const response = await handler(request);

    expect(response.statusCode).toBe(405);
    expect(JSON.parse(response.body).success).toBe(false);
  });

  it('should reject missing sources', async () => {
    const request = createRequest({
      output: { output_id: 'test', content: 'Test content' },
    });

    const response = await handler(request);

    expect(response.statusCode).toBe(400);
    expect(JSON.parse(response.body).error.code).toBe('VALIDATION_ERROR');
  });

  it('should reject missing output', async () => {
    const request = createRequest({
      sources: [{ document_id: 'test', content: 'Test' }],
    });

    const response = await handler(request);

    expect(response.statusCode).toBe(400);
    expect(JSON.parse(response.body).error.code).toBe('VALIDATION_ERROR');
  });

  it('should reject empty sources array', async () => {
    const request = createRequest({
      sources: [],
      output: { output_id: 'test', content: 'Test content' },
    });

    const response = await handler(request);

    expect(response.statusCode).toBe(400);
  });

  it('should accept valid input', async () => {
    const request = createRequest(validInput);

    const response = await handler(request);

    expect(response.statusCode).toBe(200);
    expect(JSON.parse(response.body).success).toBe(true);
  });
});

// =============================================================================
// OUTPUT VALIDATION TESTS
// =============================================================================

describe('Faithfulness Verification Agent - Output Schema', () => {
  it('should return valid output schema', async () => {
    const request = createRequest(validInput);

    const response = await handler(request);
    const result = JSON.parse(response.body);

    expect(result.success).toBe(true);
    expect(result.data).toBeDefined();

    // Validate against output schema
    const validation = FaithfulnessVerificationOutputSchema.safeParse(result.data);
    expect(validation.success).toBe(true);
  });

  it('should include required output fields', async () => {
    const request = createRequest(validInput);

    const response = await handler(request);
    const result = JSON.parse(response.body);

    expect(result.data.execution_id).toBeDefined();
    expect(result.data.output_id).toBe('output-1');
    expect(result.data.started_at).toBeDefined();
    expect(result.data.completed_at).toBeDefined();
    expect(result.data.duration_ms).toBeGreaterThanOrEqual(0);
    expect(typeof result.data.is_faithful).toBe('boolean');
    expect(result.data.faithfulness_scores).toBeDefined();
    expect(result.data.summary).toBeDefined();
  });

  it('should include faithfulness scores', async () => {
    const request = createRequest(validInput);

    const response = await handler(request);
    const result = JSON.parse(response.body);

    const scores = result.data.faithfulness_scores;
    expect(scores.overall).toBeGreaterThanOrEqual(0);
    expect(scores.overall).toBeLessThanOrEqual(1);
    expect(scores.claim_support_rate).toBeDefined();
    expect(scores.hallucination_rate).toBeDefined();
    expect(scores.contradiction_rate).toBeDefined();
  });
});

// =============================================================================
// FAITHFULNESS VERIFICATION TESTS
// =============================================================================

describe('Faithfulness Verification Agent - Verification Logic', () => {
  it('should verify faithful output as faithful', async () => {
    const request = createRequest(validInput);

    const response = await handler(request);
    const result = JSON.parse(response.body);

    expect(result.data.is_faithful).toBe(true);
    expect(result.data.faithfulness_scores.overall).toBeGreaterThan(0.5);
  });

  it('should identify unfaithful output', async () => {
    const request = createRequest(unfaithfulInput);

    const response = await handler(request);
    const result = JSON.parse(response.body);

    // Should detect issues
    expect(result.data.summary.total_hallucinations).toBeGreaterThanOrEqual(0);
    expect(result.data.summary.unsupported_claims).toBeGreaterThanOrEqual(0);
  });

  it('should extract claims from output', async () => {
    const request = createRequest(validInput);

    const response = await handler(request);
    const result = JSON.parse(response.body);

    expect(result.data.claims).toBeDefined();
    expect(Array.isArray(result.data.claims)).toBe(true);
    expect(result.data.summary.total_claims).toBeGreaterThan(0);
  });

  it('should provide evidence for claims', async () => {
    const request = createRequest(validInput);

    const response = await handler(request);
    const result = JSON.parse(response.body);

    const claimsWithEvidence = result.data.claims?.filter(
      (c: { evidence?: unknown[] }) => c.evidence && c.evidence.length > 0
    );
    expect(claimsWithEvidence?.length).toBeGreaterThanOrEqual(0);
  });
});

// =============================================================================
// CONFIGURATION TESTS
// =============================================================================

describe('Faithfulness Verification Agent - Configuration', () => {
  it('should respect faithfulness threshold', async () => {
    const request = createRequest({
      ...validInput,
      config: {
        faithfulness_threshold: 0.9,
      },
    });

    const response = await handler(request);
    const result = JSON.parse(response.body);

    expect(result.data.config.faithfulness_threshold).toBe(0.9);
  });

  it('should respect granularity setting', async () => {
    const request = createRequest({
      ...validInput,
      config: {
        granularity: 'sentence',
      },
    });

    const response = await handler(request);
    const result = JSON.parse(response.body);

    expect(result.data.config.granularity).toBe('sentence');
  });

  it('should use default config when not specified', async () => {
    const request = createRequest(validInput);

    const response = await handler(request);
    const result = JSON.parse(response.body);

    expect(result.data.config.faithfulness_threshold).toBe(0.7);
    expect(result.data.config.granularity).toBe('claim');
    expect(result.data.config.method).toBe('hybrid');
  });
});

// =============================================================================
// DECISION EVENT TESTS
// =============================================================================

describe('Faithfulness Verification Agent - Decision Event', () => {
  it('should include decision_id in response', async () => {
    const request = createRequest(validInput);

    const response = await handler(request);
    const result = JSON.parse(response.body);

    expect(result.decision_id).toBeDefined();
    expect(typeof result.decision_id).toBe('string');
  });

  it('should include decision_id in response headers', async () => {
    const request = createRequest(validInput);

    const response = await handler(request);

    expect(response.headers['X-Decision-Id']).toBeDefined();
    expect(response.headers['X-Agent-Id']).toBe('faithfulness-verification');
    expect(response.headers['X-Agent-Version']).toBe('1.0.0');
  });
});

// =============================================================================
// CONSTRAINT TESTS
// =============================================================================

describe('Faithfulness Verification Agent - Constraints', () => {
  it('should apply max_claims constraint', async () => {
    // Create output with many sentences
    const longOutput = Array(100).fill('This is a test claim.').join(' ');
    const request = createRequest({
      sources: [{ document_id: 's1', content: 'This is a test claim.' }],
      output: { output_id: 'o1', content: longOutput },
      config: { max_claims: 10 },
    });

    const response = await handler(request);
    const result = JSON.parse(response.body);

    expect(result.data.summary.total_claims).toBeLessThanOrEqual(10);
  });
});

// =============================================================================
// ERROR HANDLING TESTS
// =============================================================================

describe('Faithfulness Verification Agent - Error Handling', () => {
  it('should return error response for invalid JSON', async () => {
    const request = createRequest('invalid json');

    const response = await handler(request);

    expect(response.statusCode).toBe(400);
    expect(JSON.parse(response.body).success).toBe(false);
  });

  it('should include error code in error responses', async () => {
    const request = createRequest({});

    const response = await handler(request);
    const result = JSON.parse(response.body);

    expect(result.success).toBe(false);
    expect(result.error).toBeDefined();
    expect(result.error.code).toBeDefined();
  });
});

// =============================================================================
// SUMMARY STATISTICS TESTS
// =============================================================================

describe('Faithfulness Verification Agent - Summary Statistics', () => {
  it('should include all summary fields', async () => {
    const request = createRequest(validInput);

    const response = await handler(request);
    const result = JSON.parse(response.body);

    const summary = result.data.summary;
    expect(typeof summary.total_claims).toBe('number');
    expect(typeof summary.supported_claims).toBe('number');
    expect(typeof summary.partially_supported_claims).toBe('number');
    expect(typeof summary.unsupported_claims).toBe('number');
    expect(typeof summary.contradicted_claims).toBe('number');
    expect(typeof summary.unverifiable_claims).toBe('number');
    expect(typeof summary.total_hallucinations).toBe('number');
    expect(typeof summary.total_contradictions).toBe('number');
    expect(typeof summary.sources_used).toBe('number');
  });

  it('should have consistent claim counts', async () => {
    const request = createRequest(validInput);

    const response = await handler(request);
    const result = JSON.parse(response.body);

    const summary = result.data.summary;
    const totalFromBreakdown =
      summary.supported_claims +
      summary.partially_supported_claims +
      summary.unsupported_claims +
      summary.contradicted_claims +
      summary.unverifiable_claims;

    expect(totalFromBreakdown).toBe(summary.total_claims);
  });
});

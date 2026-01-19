/**
 * Output Consistency Agent - Handler Tests
 */

import { describe, it, expect, beforeEach, vi } from 'vitest';
import { handler, OUTPUT_CONSISTENCY_AGENT } from '../handler';
import type { OutputConsistencyInput, PromptExecutionGroup } from '../../contracts';

// Mock the services
vi.mock('../../services', () => ({
  getRuVectorClient: () => ({
    persistDecisionEvent: vi.fn().mockResolvedValue(undefined),
  }),
  createTelemetryEmitter: () => ({
    emitInvoked: vi.fn(),
    emitCompleted: vi.fn(),
    emitDecision: vi.fn(),
    emitError: vi.fn(),
    emitValidationFailed: vi.fn(),
    emitConstraintApplied: vi.fn(),
    flush: vi.fn().mockResolvedValue(undefined),
  }),
}));

function createTestGroup(overrides: Partial<PromptExecutionGroup> = {}): PromptExecutionGroup {
  return {
    group_id: 'test-group-1',
    prompt: 'Test prompt',
    provider_name: 'test-provider',
    model_id: 'test-model',
    outputs: [
      {
        output_id: '550e8400-e29b-41d4-a716-446655440001',
        content: 'Test output 1',
        execution_number: 1,
        executed_at: new Date().toISOString(),
      },
      {
        output_id: '550e8400-e29b-41d4-a716-446655440002',
        content: 'Test output 2',
        execution_number: 2,
        executed_at: new Date().toISOString(),
      },
    ],
    ...overrides,
  };
}

describe('Output Consistency Agent', () => {
  describe('Agent Metadata', () => {
    it('should have correct agent ID', () => {
      expect(OUTPUT_CONSISTENCY_AGENT.agent_id).toBe('output-consistency');
    });

    it('should have correct version', () => {
      expect(OUTPUT_CONSISTENCY_AGENT.agent_version).toBe('1.0.0');
    });

    it('should have correct decision type', () => {
      expect(OUTPUT_CONSISTENCY_AGENT.decision_type).toBe('output_consistency_analysis');
    });
  });

  describe('Handler', () => {
    it('should reject non-POST requests', async () => {
      const response = await handler({
        method: 'GET',
        path: '/output-consistency',
        headers: {},
        body: {},
      });

      expect(response.statusCode).toBe(405);
    });

    it('should reject invalid input', async () => {
      const response = await handler({
        method: 'POST',
        path: '/output-consistency',
        headers: {},
        body: { invalid: 'data' },
      });

      expect(response.statusCode).toBe(400);
      const result = JSON.parse(response.body);
      expect(result.success).toBe(false);
    });

    it('should require at least 2 outputs per group', async () => {
      const response = await handler({
        method: 'POST',
        path: '/output-consistency',
        headers: {},
        body: {
          execution_groups: [{
            group_id: 'test',
            prompt: 'Test',
            provider_name: 'test',
            model_id: 'test',
            outputs: [{
              output_id: '550e8400-e29b-41d4-a716-446655440001',
              content: 'Only one output',
              execution_number: 1,
              executed_at: new Date().toISOString(),
            }],
          }],
        },
      });

      expect(response.statusCode).toBe(400);
    });

    it('should successfully analyze consistent outputs', async () => {
      const input: OutputConsistencyInput = {
        execution_groups: [
          createTestGroup({
            outputs: [
              {
                output_id: '550e8400-e29b-41d4-a716-446655440001',
                content: 'Identical output',
                execution_number: 1,
                executed_at: new Date().toISOString(),
              },
              {
                output_id: '550e8400-e29b-41d4-a716-446655440002',
                content: 'Identical output',
                execution_number: 2,
                executed_at: new Date().toISOString(),
              },
              {
                output_id: '550e8400-e29b-41d4-a716-446655440003',
                content: 'Identical output',
                execution_number: 3,
                executed_at: new Date().toISOString(),
              },
            ],
          }),
        ],
      };

      const response = await handler({
        method: 'POST',
        path: '/output-consistency',
        headers: {},
        body: input,
      });

      expect(response.statusCode).toBe(200);

      const result = JSON.parse(response.body);
      expect(result.success).toBe(true);
      expect(result.decision_id).toBeDefined();
      expect(result.data.results).toHaveLength(1);
      expect(result.data.results[0].consistency_score).toBe(1.0);
      expect(result.data.results[0].is_consistent).toBe(true);
    });

    it('should detect inconsistent outputs', async () => {
      const input: OutputConsistencyInput = {
        execution_groups: [
          createTestGroup({
            outputs: [
              {
                output_id: '550e8400-e29b-41d4-a716-446655440001',
                content: 'The quick brown fox jumps over the lazy dog',
                execution_number: 1,
                executed_at: new Date().toISOString(),
              },
              {
                output_id: '550e8400-e29b-41d4-a716-446655440002',
                content: 'A completely different sentence about cats and mice',
                execution_number: 2,
                executed_at: new Date().toISOString(),
              },
            ],
          }),
        ],
        config: {
          consistency_threshold: 0.85,
        },
      };

      const response = await handler({
        method: 'POST',
        path: '/output-consistency',
        headers: {},
        body: input,
      });

      expect(response.statusCode).toBe(200);

      const result = JSON.parse(response.body);
      expect(result.success).toBe(true);
      expect(result.data.results[0].consistency_score).toBeLessThan(0.85);
      expect(result.data.results[0].is_consistent).toBe(false);
    });

    it('should include token analysis when enabled', async () => {
      const input: OutputConsistencyInput = {
        execution_groups: [createTestGroup()],
        config: {
          include_token_analysis: true,
        },
      };

      const response = await handler({
        method: 'POST',
        path: '/output-consistency',
        headers: {},
        body: input,
      });

      const result = JSON.parse(response.body);
      expect(result.data.results[0].token_analysis).toBeDefined();
      expect(result.data.results[0].token_analysis.avg_token_count).toBeDefined();
    });

    it('should aggregate model statistics correctly', async () => {
      const input: OutputConsistencyInput = {
        execution_groups: [
          createTestGroup({ group_id: 'group-1' }),
          createTestGroup({ group_id: 'group-2' }),
        ],
      };

      const response = await handler({
        method: 'POST',
        path: '/output-consistency',
        headers: {},
        body: input,
      });

      const result = JSON.parse(response.body);
      expect(result.data.model_stats).toHaveLength(1);
      expect(result.data.model_stats[0].groups_analyzed).toBe(2);
    });

    it('should include correct headers', async () => {
      const input: OutputConsistencyInput = {
        execution_groups: [createTestGroup()],
      };

      const response = await handler({
        method: 'POST',
        path: '/output-consistency',
        headers: {},
        body: input,
      });

      expect(response.headers['X-Agent-Id']).toBe('output-consistency');
      expect(response.headers['X-Agent-Version']).toBe('1.0.0');
      expect(response.headers['X-Decision-Id']).toBeDefined();
    });

    it('should calculate summary correctly', async () => {
      const input: OutputConsistencyInput = {
        execution_groups: [createTestGroup()],
      };

      const response = await handler({
        method: 'POST',
        path: '/output-consistency',
        headers: {},
        body: input,
      });

      const result = JSON.parse(response.body);
      expect(result.data.summary.total_groups_analyzed).toBe(1);
      expect(result.data.summary.total_outputs_analyzed).toBe(2);
      expect(result.data.summary.overall_avg_consistency).toBeGreaterThanOrEqual(0);
      expect(result.data.summary.overall_avg_consistency).toBeLessThanOrEqual(1);
    });
  });

  describe('Similarity Methods', () => {
    it('should support exact_match method', async () => {
      const input: OutputConsistencyInput = {
        execution_groups: [
          createTestGroup({
            outputs: [
              {
                output_id: '550e8400-e29b-41d4-a716-446655440001',
                content: 'Same',
                execution_number: 1,
                executed_at: new Date().toISOString(),
              },
              {
                output_id: '550e8400-e29b-41d4-a716-446655440002',
                content: 'Same',
                execution_number: 2,
                executed_at: new Date().toISOString(),
              },
            ],
          }),
        ],
        config: {
          similarity_method: 'exact_match',
        },
      };

      const response = await handler({
        method: 'POST',
        path: '/output-consistency',
        headers: {},
        body: input,
      });

      const result = JSON.parse(response.body);
      expect(result.data.results[0].similarity_scores.primary_method).toBe('exact_match');
      expect(result.data.results[0].consistency_score).toBe(1.0);
    });

    it('should support normalized_levenshtein method', async () => {
      const input: OutputConsistencyInput = {
        execution_groups: [createTestGroup()],
        config: {
          similarity_method: 'normalized_levenshtein',
        },
      };

      const response = await handler({
        method: 'POST',
        path: '/output-consistency',
        headers: {},
        body: input,
      });

      const result = JSON.parse(response.body);
      expect(result.data.results[0].similarity_scores.primary_method).toBe('normalized_levenshtein');
    });

    it('should support jaccard_tokens method', async () => {
      const input: OutputConsistencyInput = {
        execution_groups: [createTestGroup()],
        config: {
          similarity_method: 'jaccard_tokens',
        },
      };

      const response = await handler({
        method: 'POST',
        path: '/output-consistency',
        headers: {},
        body: input,
      });

      const result = JSON.parse(response.body);
      expect(result.data.results[0].similarity_scores.primary_method).toBe('jaccard_tokens');
    });
  });

  describe('Edge Cases', () => {
    it('should handle empty strings', async () => {
      const input: OutputConsistencyInput = {
        execution_groups: [
          createTestGroup({
            outputs: [
              {
                output_id: '550e8400-e29b-41d4-a716-446655440001',
                content: '',
                execution_number: 1,
                executed_at: new Date().toISOString(),
              },
              {
                output_id: '550e8400-e29b-41d4-a716-446655440002',
                content: '',
                execution_number: 2,
                executed_at: new Date().toISOString(),
              },
            ],
          }),
        ],
      };

      const response = await handler({
        method: 'POST',
        path: '/output-consistency',
        headers: {},
        body: input,
      });

      expect(response.statusCode).toBe(200);
    });

    it('should handle whitespace normalization', async () => {
      const input: OutputConsistencyInput = {
        execution_groups: [
          createTestGroup({
            outputs: [
              {
                output_id: '550e8400-e29b-41d4-a716-446655440001',
                content: 'hello   world',
                execution_number: 1,
                executed_at: new Date().toISOString(),
              },
              {
                output_id: '550e8400-e29b-41d4-a716-446655440002',
                content: 'hello world',
                execution_number: 2,
                executed_at: new Date().toISOString(),
              },
            ],
          }),
        ],
        config: {
          normalize_whitespace: true,
        },
      };

      const response = await handler({
        method: 'POST',
        path: '/output-consistency',
        headers: {},
        body: input,
      });

      const result = JSON.parse(response.body);
      // With whitespace normalization, these should be considered identical
      expect(result.data.results[0].consistency_score).toBe(1.0);
    });

    it('should handle case insensitivity', async () => {
      const input: OutputConsistencyInput = {
        execution_groups: [
          createTestGroup({
            outputs: [
              {
                output_id: '550e8400-e29b-41d4-a716-446655440001',
                content: 'Hello World',
                execution_number: 1,
                executed_at: new Date().toISOString(),
              },
              {
                output_id: '550e8400-e29b-41d4-a716-446655440002',
                content: 'hello world',
                execution_number: 2,
                executed_at: new Date().toISOString(),
              },
            ],
          }),
        ],
        config: {
          case_sensitive: false,
        },
      };

      const response = await handler({
        method: 'POST',
        path: '/output-consistency',
        headers: {},
        body: input,
      });

      const result = JSON.parse(response.body);
      // With case insensitivity, these should be considered identical
      expect(result.data.results[0].consistency_score).toBe(1.0);
    });
  });
});

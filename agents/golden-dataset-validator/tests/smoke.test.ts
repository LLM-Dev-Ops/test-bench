/**
 * Golden Dataset Validator Agent - Smoke Tests
 *
 * Basic integration tests to verify the agent works correctly.
 */

import { describe, it, expect, beforeEach } from 'vitest';
import { randomUUID } from 'crypto';
import {
  handler,
  GOLDEN_DATASET_VALIDATOR_AGENT,
  GoldenDatasetValidatorInputSchema,
  GoldenDatasetValidatorOutputSchema,
  type EdgeFunctionRequest,
  type GoldenDatasetValidatorInput,
} from '../handler';

describe('Golden Dataset Validator Agent', () => {
  describe('Agent Metadata', () => {
    it('should have correct agent_id', () => {
      expect(GOLDEN_DATASET_VALIDATOR_AGENT.agent_id).toBe('golden-dataset-validator');
    });

    it('should have correct agent_version', () => {
      expect(GOLDEN_DATASET_VALIDATOR_AGENT.agent_version).toMatch(/^\d+\.\d+\.\d+$/);
    });

    it('should have correct decision_type', () => {
      expect(GOLDEN_DATASET_VALIDATOR_AGENT.decision_type).toBe('golden_dataset_validation');
    });
  });

  describe('Input Validation', () => {
    it('should validate valid input', () => {
      const input: GoldenDatasetValidatorInput = {
        golden_samples: [
          {
            sample_id: 'sample-1',
            input: 'What is 2+2?',
            golden_output: '4',
          },
        ],
        model_outputs: [
          {
            sample_id: 'sample-1',
            model_output: '4',
          },
        ],
        dataset: {
          name: 'test-dataset',
        },
      };

      const result = GoldenDatasetValidatorInputSchema.safeParse(input);
      expect(result.success).toBe(true);
    });

    it('should reject empty golden_samples', () => {
      const input = {
        golden_samples: [],
        model_outputs: [
          {
            sample_id: 'sample-1',
            model_output: '4',
          },
        ],
      };

      const result = GoldenDatasetValidatorInputSchema.safeParse(input);
      expect(result.success).toBe(false);
    });

    it('should reject empty model_outputs', () => {
      const input = {
        golden_samples: [
          {
            sample_id: 'sample-1',
            input: 'What is 2+2?',
            golden_output: '4',
          },
        ],
        model_outputs: [],
      };

      const result = GoldenDatasetValidatorInputSchema.safeParse(input);
      expect(result.success).toBe(false);
    });

    it('should reject mismatched sample_ids', () => {
      const input = {
        golden_samples: [
          {
            sample_id: 'sample-1',
            input: 'What is 2+2?',
            golden_output: '4',
          },
        ],
        model_outputs: [
          {
            sample_id: 'sample-999', // Does not exist
            model_output: '4',
          },
        ],
      };

      const result = GoldenDatasetValidatorInputSchema.safeParse(input);
      expect(result.success).toBe(false);
    });
  });

  describe('Handler', () => {
    it('should reject non-POST requests', async () => {
      const request: EdgeFunctionRequest = {
        method: 'GET',
        path: '/golden-dataset-validator',
        headers: {},
        body: {},
      };

      const response = await handler(request);
      expect(response.statusCode).toBe(405);
    });

    it('should reject invalid input', async () => {
      const request: EdgeFunctionRequest = {
        method: 'POST',
        path: '/golden-dataset-validator',
        headers: { 'Content-Type': 'application/json' },
        body: { invalid: 'data' },
      };

      const response = await handler(request);
      expect(response.statusCode).toBe(400);

      const result = JSON.parse(response.body);
      expect(result.success).toBe(false);
      expect(result.error.code).toBe('VALIDATION_ERROR');
    });

    it('should process exact match correctly', async () => {
      const input: GoldenDatasetValidatorInput = {
        golden_samples: [
          {
            sample_id: 'exact-1',
            input: 'What is 2+2?',
            golden_output: '4',
          },
        ],
        model_outputs: [
          {
            sample_id: 'exact-1',
            model_output: '4',
          },
        ],
        dataset: {
          name: 'exact-match-test',
        },
      };

      const request: EdgeFunctionRequest = {
        method: 'POST',
        path: '/golden-dataset-validator',
        headers: { 'Content-Type': 'application/json' },
        body: input,
      };

      const response = await handler(request);
      expect(response.statusCode).toBe(200);

      const result = JSON.parse(response.body);
      expect(result.success).toBe(true);
      expect(result.data.stats.exact_matches).toBe(1);
      expect(result.data.stats.pass_rate).toBe(1);
      expect(result.data.results[0].match_type).toBe('exact_match');
    });

    it('should detect semantic match', async () => {
      const input: GoldenDatasetValidatorInput = {
        golden_samples: [
          {
            sample_id: 'semantic-1',
            input: 'What is the capital of France?',
            golden_output: 'The capital of France is Paris.',
          },
        ],
        model_outputs: [
          {
            sample_id: 'semantic-1',
            model_output: 'Paris is the capital of France.',
          },
        ],
        dataset: {
          name: 'semantic-match-test',
        },
      };

      const request: EdgeFunctionRequest = {
        method: 'POST',
        path: '/golden-dataset-validator',
        headers: { 'Content-Type': 'application/json' },
        body: input,
      };

      const response = await handler(request);
      expect(response.statusCode).toBe(200);

      const result = JSON.parse(response.body);
      expect(result.success).toBe(true);
      // Should detect high semantic similarity due to shared keywords
      expect(result.data.results[0].semantic_similarity).toBeGreaterThan(0.5);
    });

    it('should detect no match', async () => {
      const input: GoldenDatasetValidatorInput = {
        golden_samples: [
          {
            sample_id: 'no-match-1',
            input: 'What is 2+2?',
            golden_output: '4',
          },
        ],
        model_outputs: [
          {
            sample_id: 'no-match-1',
            model_output: 'The sky is blue and grass is green.',
          },
        ],
        dataset: {
          name: 'no-match-test',
        },
      };

      const request: EdgeFunctionRequest = {
        method: 'POST',
        path: '/golden-dataset-validator',
        headers: { 'Content-Type': 'application/json' },
        body: input,
      };

      const response = await handler(request);
      expect(response.statusCode).toBe(200);

      const result = JSON.parse(response.body);
      expect(result.success).toBe(true);
      expect(result.data.results[0].match_type).toBe('no_match');
      expect(result.data.results[0].passed).toBe(false);
    });

    it('should process multiple samples', async () => {
      const input: GoldenDatasetValidatorInput = {
        golden_samples: [
          { sample_id: 'multi-1', input: 'Q1', golden_output: 'Answer 1' },
          { sample_id: 'multi-2', input: 'Q2', golden_output: 'Answer 2' },
          { sample_id: 'multi-3', input: 'Q3', golden_output: 'Answer 3' },
        ],
        model_outputs: [
          { sample_id: 'multi-1', model_output: 'Answer 1' }, // Exact match
          { sample_id: 'multi-2', model_output: 'Answer 2' }, // Exact match
          { sample_id: 'multi-3', model_output: 'Wrong answer' }, // No match
        ],
        dataset: {
          name: 'multi-sample-test',
        },
      };

      const request: EdgeFunctionRequest = {
        method: 'POST',
        path: '/golden-dataset-validator',
        headers: { 'Content-Type': 'application/json' },
        body: input,
      };

      const response = await handler(request);
      expect(response.statusCode).toBe(200);

      const result = JSON.parse(response.body);
      expect(result.success).toBe(true);
      expect(result.data.stats.total_samples).toBe(3);
      expect(result.data.stats.exact_matches).toBe(2);
      expect(result.data.stats.passed).toBeGreaterThanOrEqual(2);
    });

    it('should include response headers', async () => {
      const input: GoldenDatasetValidatorInput = {
        golden_samples: [
          { sample_id: 'header-1', input: 'Test', golden_output: 'Test' },
        ],
        model_outputs: [
          { sample_id: 'header-1', model_output: 'Test' },
        ],
      };

      const request: EdgeFunctionRequest = {
        method: 'POST',
        path: '/golden-dataset-validator',
        headers: { 'Content-Type': 'application/json' },
        body: input,
      };

      const response = await handler(request);
      expect(response.headers['X-Agent-Id']).toBe(GOLDEN_DATASET_VALIDATOR_AGENT.agent_id);
      expect(response.headers['X-Agent-Version']).toBe(GOLDEN_DATASET_VALIDATOR_AGENT.agent_version);
      expect(response.headers['X-Decision-Id']).toBeDefined();
    });

    it('should handle case-insensitive matching', async () => {
      const input: GoldenDatasetValidatorInput = {
        golden_samples: [
          { sample_id: 'case-1', input: 'Test', golden_output: 'HELLO WORLD' },
        ],
        model_outputs: [
          { sample_id: 'case-1', model_output: 'hello world' },
        ],
        validation_config: {
          case_insensitive: true,
        },
      };

      const request: EdgeFunctionRequest = {
        method: 'POST',
        path: '/golden-dataset-validator',
        headers: { 'Content-Type': 'application/json' },
        body: input,
      };

      const response = await handler(request);
      expect(response.statusCode).toBe(200);

      const result = JSON.parse(response.body);
      expect(result.success).toBe(true);
      expect(result.data.results[0].exact_match).toBe(true);
    });
  });

  describe('Output Validation', () => {
    it('should produce valid output schema', async () => {
      const input: GoldenDatasetValidatorInput = {
        golden_samples: [
          { sample_id: 'schema-1', input: 'Test', golden_output: 'Output' },
        ],
        model_outputs: [
          { sample_id: 'schema-1', model_output: 'Output' },
        ],
        dataset: {
          name: 'schema-test',
          version: '1.0',
        },
      };

      const request: EdgeFunctionRequest = {
        method: 'POST',
        path: '/golden-dataset-validator',
        headers: { 'Content-Type': 'application/json' },
        body: input,
      };

      const response = await handler(request);
      const result = JSON.parse(response.body);

      // Validate output schema
      const outputValidation = GoldenDatasetValidatorOutputSchema.safeParse(result.data);
      expect(outputValidation.success).toBe(true);
    });

    it('should include quality assessment', async () => {
      const input: GoldenDatasetValidatorInput = {
        golden_samples: [
          { sample_id: 'qa-1', input: 'Test', golden_output: 'Output' },
        ],
        model_outputs: [
          { sample_id: 'qa-1', model_output: 'Output' },
        ],
      };

      const request: EdgeFunctionRequest = {
        method: 'POST',
        path: '/golden-dataset-validator',
        headers: { 'Content-Type': 'application/json' },
        body: input,
      };

      const response = await handler(request);
      const result = JSON.parse(response.body);

      expect(result.data.quality_assessment).toBeDefined();
      expect(result.data.quality_assessment.grade).toMatch(/^[ABCDF]$/);
      expect(result.data.quality_assessment.score).toBeGreaterThanOrEqual(0);
      expect(result.data.quality_assessment.score).toBeLessThanOrEqual(100);
      expect(result.data.quality_assessment.summary).toBeDefined();
    });

    it('should include statistics breakdown', async () => {
      const input: GoldenDatasetValidatorInput = {
        golden_samples: [
          { sample_id: 'stats-1', input: 'Test', golden_output: 'Output', category: 'cat-a' },
          { sample_id: 'stats-2', input: 'Test', golden_output: 'Output', category: 'cat-b' },
        ],
        model_outputs: [
          { sample_id: 'stats-1', model_output: 'Output' },
          { sample_id: 'stats-2', model_output: 'Wrong' },
        ],
      };

      const request: EdgeFunctionRequest = {
        method: 'POST',
        path: '/golden-dataset-validator',
        headers: { 'Content-Type': 'application/json' },
        body: input,
      };

      const response = await handler(request);
      const result = JSON.parse(response.body);

      expect(result.data.stats.by_match_type).toBeDefined();
      expect(result.data.stats.by_severity).toBeDefined();
      expect(result.data.stats.by_category).toBeDefined();
    });
  });

  describe('Constraints', () => {
    it('should apply max_samples constraint', async () => {
      // Create more samples than max
      const goldenSamples = Array.from({ length: 20 }, (_, i) => ({
        sample_id: `max-${i}`,
        input: `Question ${i}`,
        golden_output: `Answer ${i}`,
      }));

      const modelOutputs = goldenSamples.map(s => ({
        sample_id: s.sample_id,
        model_output: s.golden_output,
      }));

      const input: GoldenDatasetValidatorInput = {
        golden_samples: goldenSamples,
        model_outputs: modelOutputs,
        validation_config: {
          max_samples: 5, // Limit to 5
        },
      };

      const request: EdgeFunctionRequest = {
        method: 'POST',
        path: '/golden-dataset-validator',
        headers: { 'Content-Type': 'application/json' },
        body: input,
      };

      const response = await handler(request);
      const result = JSON.parse(response.body);

      expect(result.success).toBe(true);
      expect(result.data.results.length).toBe(5);
    });
  });
});

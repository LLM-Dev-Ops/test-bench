/**
 * Synthetic Data Generator Agent - Unit Tests
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { handler, SYNTHETIC_DATA_GENERATOR_AGENT } from '../handler';

// Mock services
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

describe('Synthetic Data Generator Agent', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  afterEach(() => {
    vi.resetAllMocks();
  });

  describe('Agent Metadata', () => {
    it('should have correct agent_id', () => {
      expect(SYNTHETIC_DATA_GENERATOR_AGENT.agent_id).toBe('synthetic-data-generator');
    });

    it('should have correct agent_version', () => {
      expect(SYNTHETIC_DATA_GENERATOR_AGENT.agent_version).toBe('1.0.0');
    });

    it('should have correct decision_type', () => {
      expect(SYNTHETIC_DATA_GENERATOR_AGENT.decision_type).toBe('synthetic_data_generation');
    });
  });

  describe('HTTP Method Validation', () => {
    it('should reject GET requests with 405', async () => {
      const response = await handler({
        method: 'GET',
        path: '/synthetic-data-generator',
        headers: {},
        body: {},
      });

      expect(response.statusCode).toBe(405);
      expect(JSON.parse(response.body).success).toBe(false);
    });

    it('should reject PUT requests with 405', async () => {
      const response = await handler({
        method: 'PUT',
        path: '/synthetic-data-generator',
        headers: {},
        body: {},
      });

      expect(response.statusCode).toBe(405);
    });

    it('should reject DELETE requests with 405', async () => {
      const response = await handler({
        method: 'DELETE',
        path: '/synthetic-data-generator',
        headers: {},
        body: {},
      });

      expect(response.statusCode).toBe(405);
    });
  });

  describe('Input Validation', () => {
    it('should reject empty body with 400', async () => {
      const response = await handler({
        method: 'POST',
        path: '/synthetic-data-generator',
        headers: { 'Content-Type': 'application/json' },
        body: {},
      });

      expect(response.statusCode).toBe(400);
      const body = JSON.parse(response.body);
      expect(body.success).toBe(false);
      expect(body.error.code).toBe('VALIDATION_ERROR');
    });

    it('should reject missing data_type', async () => {
      const response = await handler({
        method: 'POST',
        path: '/synthetic-data-generator',
        headers: { 'Content-Type': 'application/json' },
        body: {
          generation_strategy: 'template_based',
          count: 10,
        },
      });

      expect(response.statusCode).toBe(400);
    });

    it('should reject missing generation_strategy', async () => {
      const response = await handler({
        method: 'POST',
        path: '/synthetic-data-generator',
        headers: { 'Content-Type': 'application/json' },
        body: {
          data_type: 'qa_pair',
          count: 10,
        },
      });

      expect(response.statusCode).toBe(400);
    });

    it('should reject missing count', async () => {
      const response = await handler({
        method: 'POST',
        path: '/synthetic-data-generator',
        headers: { 'Content-Type': 'application/json' },
        body: {
          data_type: 'qa_pair',
          generation_strategy: 'template_based',
        },
      });

      expect(response.statusCode).toBe(400);
    });

    it('should reject count exceeding maximum', async () => {
      const response = await handler({
        method: 'POST',
        path: '/synthetic-data-generator',
        headers: { 'Content-Type': 'application/json' },
        body: {
          data_type: 'qa_pair',
          generation_strategy: 'template_based',
          count: 100000, // Exceeds max of 10000
        },
      });

      expect(response.statusCode).toBe(400);
    });

    it('should reject invalid data_type', async () => {
      const response = await handler({
        method: 'POST',
        path: '/synthetic-data-generator',
        headers: { 'Content-Type': 'application/json' },
        body: {
          data_type: 'invalid_type',
          generation_strategy: 'template_based',
          count: 10,
        },
      });

      expect(response.statusCode).toBe(400);
    });

    it('should reject invalid generation_strategy', async () => {
      const response = await handler({
        method: 'POST',
        path: '/synthetic-data-generator',
        headers: { 'Content-Type': 'application/json' },
        body: {
          data_type: 'qa_pair',
          generation_strategy: 'invalid_strategy',
          count: 10,
        },
      });

      expect(response.statusCode).toBe(400);
    });
  });

  describe('Successful Generation', () => {
    it('should generate QA pairs successfully', async () => {
      const response = await handler({
        method: 'POST',
        path: '/synthetic-data-generator',
        headers: { 'Content-Type': 'application/json' },
        body: {
          data_type: 'qa_pair',
          generation_strategy: 'template_based',
          count: 5,
        },
      });

      expect(response.statusCode).toBe(200);
      const body = JSON.parse(response.body);
      expect(body.success).toBe(true);
      expect(body.decision_id).toBeDefined();
      expect(body.data.generated_items).toHaveLength(5);
      expect(body.data.generation_stats.generated_count).toBe(5);
    });

    it('should generate text prompts successfully', async () => {
      const response = await handler({
        method: 'POST',
        path: '/synthetic-data-generator',
        headers: { 'Content-Type': 'application/json' },
        body: {
          data_type: 'text_prompt',
          generation_strategy: 'template_based',
          count: 3,
        },
      });

      expect(response.statusCode).toBe(200);
      const body = JSON.parse(response.body);
      expect(body.success).toBe(true);
      expect(body.data.generated_items).toHaveLength(3);
    });

    it('should generate coding tasks successfully', async () => {
      const response = await handler({
        method: 'POST',
        path: '/synthetic-data-generator',
        headers: { 'Content-Type': 'application/json' },
        body: {
          data_type: 'coding_task',
          generation_strategy: 'progressive_difficulty',
          count: 3,
          coding_config: {
            languages: ['python'],
            include_test_cases: true,
            test_case_count: 3,
          },
        },
      });

      expect(response.statusCode).toBe(200);
      const body = JSON.parse(response.body);
      expect(body.success).toBe(true);
      expect(body.data.generated_items).toHaveLength(3);
    });

    it('should generate conversations successfully', async () => {
      const response = await handler({
        method: 'POST',
        path: '/synthetic-data-generator',
        headers: { 'Content-Type': 'application/json' },
        body: {
          data_type: 'multi_turn_conversation',
          generation_strategy: 'template_based',
          count: 2,
          conversation_config: {
            min_turns: 2,
            max_turns: 5,
          },
        },
      });

      expect(response.statusCode).toBe(200);
      const body = JSON.parse(response.body);
      expect(body.success).toBe(true);
      expect(body.data.generated_items).toHaveLength(2);
    });
  });

  describe('Determinism with Random Seed', () => {
    it('should produce identical results with same seed', async () => {
      const input = {
        data_type: 'qa_pair',
        generation_strategy: 'template_based',
        count: 5,
        random_seed: 42,
      };

      const response1 = await handler({
        method: 'POST',
        path: '/synthetic-data-generator',
        headers: {},
        body: input,
      });

      const response2 = await handler({
        method: 'POST',
        path: '/synthetic-data-generator',
        headers: {},
        body: input,
      });

      const body1 = JSON.parse(response1.body);
      const body2 = JSON.parse(response2.body);

      expect(body1.data.generated_items.length).toBe(body2.data.generated_items.length);

      // Compare content of generated items (excluding unique IDs and timestamps)
      for (let i = 0; i < body1.data.generated_items.length; i++) {
        expect(body1.data.generated_items[i].content).toEqual(
          body2.data.generated_items[i].content
        );
      }
    });

    it('should produce different results with different seeds', async () => {
      const response1 = await handler({
        method: 'POST',
        path: '/synthetic-data-generator',
        headers: {},
        body: {
          data_type: 'text_prompt',
          generation_strategy: 'template_based',
          count: 5,
          random_seed: 42,
        },
      });

      const response2 = await handler({
        method: 'POST',
        path: '/synthetic-data-generator',
        headers: {},
        body: {
          data_type: 'text_prompt',
          generation_strategy: 'template_based',
          count: 5,
          random_seed: 999,
        },
      });

      const body1 = JSON.parse(response1.body);
      const body2 = JSON.parse(response2.body);

      // At least some items should differ
      let hasDifference = false;
      for (let i = 0; i < body1.data.generated_items.length; i++) {
        if (
          JSON.stringify(body1.data.generated_items[i].content) !==
          JSON.stringify(body2.data.generated_items[i].content)
        ) {
          hasDifference = true;
          break;
        }
      }
      expect(hasDifference).toBe(true);
    });
  });

  describe('Generation Strategies', () => {
    const strategies = [
      'template_based',
      'variation',
      'distribution_aware',
      'edge_case',
      'adversarial',
      'combinatorial',
      'progressive_difficulty',
      'cross_domain',
    ];

    strategies.forEach((strategy) => {
      it(`should handle ${strategy} strategy`, async () => {
        const response = await handler({
          method: 'POST',
          path: '/synthetic-data-generator',
          headers: {},
          body: {
            data_type: 'text_prompt',
            generation_strategy: strategy,
            count: 3,
          },
        });

        expect(response.statusCode).toBe(200);
        const body = JSON.parse(response.body);
        expect(body.success).toBe(true);
        expect(body.data.generated_items.length).toBeGreaterThan(0);
      });
    });
  });

  describe('Data Types', () => {
    const dataTypes = [
      'text_prompt',
      'qa_pair',
      'multi_turn_conversation',
      'coding_task',
      'summarization',
      'creative_writing',
      'classification',
      'entity_extraction',
      'translation',
      'reasoning_chain',
    ];

    dataTypes.forEach((dataType) => {
      it(`should generate ${dataType} data type`, async () => {
        const response = await handler({
          method: 'POST',
          path: '/synthetic-data-generator',
          headers: {},
          body: {
            data_type: dataType,
            generation_strategy: 'template_based',
            count: 2,
          },
        });

        expect(response.statusCode).toBe(200);
        const body = JSON.parse(response.body);
        expect(body.success).toBe(true);
        expect(body.data.generated_items[0].data_type).toBe(dataType);
      });
    });
  });

  describe('Difficulty Distribution', () => {
    it('should respect difficulty distribution', async () => {
      const response = await handler({
        method: 'POST',
        path: '/synthetic-data-generator',
        headers: {},
        body: {
          data_type: 'qa_pair',
          generation_strategy: 'distribution_aware',
          count: 100,
          difficulty_distribution: {
            easy: 0.5,
            medium: 0.3,
            hard: 0.2,
          },
        },
      });

      expect(response.statusCode).toBe(200);
      const body = JSON.parse(response.body);
      expect(body.success).toBe(true);
      expect(body.data.distribution_analysis.difficulty_actual).toBeDefined();
    });
  });

  describe('Quality Metrics', () => {
    it('should include quality metrics in output', async () => {
      const response = await handler({
        method: 'POST',
        path: '/synthetic-data-generator',
        headers: {},
        body: {
          data_type: 'qa_pair',
          generation_strategy: 'template_based',
          count: 10,
        },
      });

      expect(response.statusCode).toBe(200);
      const body = JSON.parse(response.body);

      expect(body.data.quality_metrics).toBeDefined();
      expect(body.data.quality_metrics.avg_length_chars).toBeGreaterThan(0);
      expect(body.data.quality_metrics.avg_token_count).toBeGreaterThan(0);
      expect(body.data.quality_metrics.avg_complexity_score).toBeGreaterThanOrEqual(0);
      expect(body.data.quality_metrics.constraint_satisfaction_rate).toBeGreaterThanOrEqual(0);
      expect(body.data.quality_metrics.unique_items_rate).toBeGreaterThanOrEqual(0);
    });

    it('should include distribution analysis in output', async () => {
      const response = await handler({
        method: 'POST',
        path: '/synthetic-data-generator',
        headers: {},
        body: {
          data_type: 'text_prompt',
          generation_strategy: 'template_based',
          count: 10,
        },
      });

      expect(response.statusCode).toBe(200);
      const body = JSON.parse(response.body);

      expect(body.data.distribution_analysis).toBeDefined();
      expect(body.data.distribution_analysis.length_distribution).toBeDefined();
      expect(body.data.distribution_analysis.length_distribution.min).toBeDefined();
      expect(body.data.distribution_analysis.length_distribution.max).toBeDefined();
      expect(body.data.distribution_analysis.length_distribution.mean).toBeDefined();
    });
  });

  describe('Response Headers', () => {
    it('should include X-Decision-Id header', async () => {
      const response = await handler({
        method: 'POST',
        path: '/synthetic-data-generator',
        headers: {},
        body: {
          data_type: 'qa_pair',
          generation_strategy: 'template_based',
          count: 1,
        },
      });

      expect(response.headers['X-Decision-Id']).toBeDefined();
    });

    it('should include X-Agent-Id header', async () => {
      const response = await handler({
        method: 'POST',
        path: '/synthetic-data-generator',
        headers: {},
        body: {
          data_type: 'qa_pair',
          generation_strategy: 'template_based',
          count: 1,
        },
      });

      expect(response.headers['X-Agent-Id']).toBe('synthetic-data-generator');
    });

    it('should include X-Agent-Version header', async () => {
      const response = await handler({
        method: 'POST',
        path: '/synthetic-data-generator',
        headers: {},
        body: {
          data_type: 'qa_pair',
          generation_strategy: 'template_based',
          count: 1,
        },
      });

      expect(response.headers['X-Agent-Version']).toBe('1.0.0');
    });
  });

  describe('Generation Stats', () => {
    it('should track generation statistics', async () => {
      const response = await handler({
        method: 'POST',
        path: '/synthetic-data-generator',
        headers: {},
        body: {
          data_type: 'qa_pair',
          generation_strategy: 'template_based',
          count: 10,
        },
      });

      expect(response.statusCode).toBe(200);
      const body = JSON.parse(response.body);

      expect(body.data.generation_stats).toBeDefined();
      expect(body.data.generation_stats.requested_count).toBe(10);
      expect(body.data.generation_stats.generated_count).toBe(10);
      expect(body.data.generation_stats.failed_count).toBe(0);
      expect(body.data.generation_stats.duplicate_count).toBe(0);
      expect(body.data.generation_stats.strategy_distribution).toBeDefined();
    });
  });

  describe('Input Config Summary', () => {
    it('should include input config summary in output', async () => {
      const response = await handler({
        method: 'POST',
        path: '/synthetic-data-generator',
        headers: {},
        body: {
          data_type: 'coding_task',
          generation_strategy: 'progressive_difficulty',
          count: 5,
        },
      });

      expect(response.statusCode).toBe(200);
      const body = JSON.parse(response.body);

      expect(body.data.input_config_summary).toBeDefined();
      expect(body.data.input_config_summary.data_type).toBe('coding_task');
      expect(body.data.input_config_summary.generation_strategy).toBe('progressive_difficulty');
      expect(body.data.input_config_summary.requested_count).toBe(5);
    });
  });
});

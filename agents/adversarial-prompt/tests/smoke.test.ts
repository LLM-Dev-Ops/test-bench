/**
 * Adversarial Prompt Agent - Smoke Tests
 *
 * Basic tests to verify the agent is working correctly.
 */

import { describe, it, expect, beforeEach, vi } from 'vitest';
import { handler, ADVERSARIAL_PROMPT_AGENT } from '../handler';
import {
  AdversarialPromptInputSchema,
  AdversarialPromptOutputSchema,
  type AdversarialPromptInput,
  type AdversarialPromptOutput,
} from '../../contracts';

// Mock the ruvector client
vi.mock('../../services', () => ({
  getRuVectorClient: () => ({
    persistDecisionEvent: vi.fn().mockResolvedValue(undefined),
    flush: vi.fn().mockResolvedValue(undefined),
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

describe('Adversarial Prompt Agent', () => {
  describe('Agent Metadata', () => {
    it('should have correct agent ID', () => {
      expect(ADVERSARIAL_PROMPT_AGENT.agent_id).toBe('adversarial-prompt');
    });

    it('should have correct decision type', () => {
      expect(ADVERSARIAL_PROMPT_AGENT.decision_type).toBe('adversarial_prompt_generation');
    });

    it('should have valid semver version', () => {
      expect(ADVERSARIAL_PROMPT_AGENT.agent_version).toMatch(/^\d+\.\d+\.\d+$/);
    });
  });

  describe('Input Validation', () => {
    it('should reject non-POST requests', async () => {
      const response = await handler({
        body: {},
        headers: {},
        method: 'GET',
        path: '/adversarial-prompt',
      });

      expect(response.statusCode).toBe(405);
    });

    it('should reject invalid input', async () => {
      const response = await handler({
        body: { invalid: 'data' },
        headers: {},
        method: 'POST',
        path: '/adversarial-prompt',
      });

      expect(response.statusCode).toBe(400);
      const result = JSON.parse(response.body);
      expect(result.success).toBe(false);
      expect(result.error.code).toBe('VALIDATION_ERROR');
    });

    it('should accept minimal valid input', async () => {
      const input: AdversarialPromptInput = {
        categories: ['prompt_injection'],
        severities: ['low'],
        count_per_category: 1,
        total_max_count: 10,
        strategy: 'template_based',
        target_model_types: ['general'],
        language: 'en',
        include_benign_variants: true,
        safety_ceiling: 'high',
        purpose: 'stress_testing',
      };

      const validation = AdversarialPromptInputSchema.safeParse(input);
      expect(validation.success).toBe(true);
    });

    it('should validate all adversarial categories', () => {
      const categories = [
        'prompt_injection',
        'jailbreak_attempt',
        'encoding_attacks',
        'format_confusion',
        'system_prompt_extraction',
      ];

      for (const category of categories) {
        const input: AdversarialPromptInput = {
          categories: [category as any],
          severities: ['low'],
          count_per_category: 1,
          total_max_count: 10,
          strategy: 'template_based',
          target_model_types: ['general'],
          language: 'en',
          include_benign_variants: false,
          safety_ceiling: 'medium',
          purpose: 'stress_testing',
        };

        const validation = AdversarialPromptInputSchema.safeParse(input);
        expect(validation.success).toBe(true);
      }
    });
  });

  describe('Prompt Generation', () => {
    it('should generate prompts for single category', async () => {
      const input: AdversarialPromptInput = {
        categories: ['prompt_injection'],
        severities: ['low'],
        count_per_category: 3,
        total_max_count: 10,
        strategy: 'template_based',
        target_model_types: ['general'],
        language: 'en',
        include_benign_variants: true,
        safety_ceiling: 'high',
        purpose: 'stress_testing',
      };

      const response = await handler({
        body: input,
        headers: {},
        method: 'POST',
        path: '/adversarial-prompt',
      });

      expect(response.statusCode).toBe(200);

      const result = JSON.parse(response.body);
      expect(result.success).toBe(true);
      expect(result.data.prompts).toBeDefined();
      expect(result.data.prompts.length).toBeGreaterThan(0);
    });

    it('should generate prompts for multiple categories', async () => {
      const input: AdversarialPromptInput = {
        categories: ['prompt_injection', 'encoding_attacks', 'jailbreak_attempt'],
        severities: ['low', 'medium'],
        count_per_category: 2,
        total_max_count: 20,
        strategy: 'template_based',
        target_model_types: ['general'],
        language: 'en',
        include_benign_variants: false,
        safety_ceiling: 'high',
        purpose: 'security_audit',
      };

      const response = await handler({
        body: input,
        headers: {},
        method: 'POST',
        path: '/adversarial-prompt',
      });

      expect(response.statusCode).toBe(200);

      const result = JSON.parse(response.body);
      expect(result.success).toBe(true);
      expect(result.data.prompts.length).toBeGreaterThan(0);

      // Verify multiple categories are represented
      const categories = new Set(result.data.prompts.map((p: any) => p.category));
      expect(categories.size).toBeGreaterThan(1);
    });

    it('should respect total_max_count limit', async () => {
      const maxCount = 5;
      const input: AdversarialPromptInput = {
        categories: ['prompt_injection', 'encoding_attacks', 'jailbreak_attempt'],
        severities: ['low', 'medium', 'high'],
        count_per_category: 10, // Would generate many more than maxCount
        total_max_count: maxCount,
        strategy: 'template_based',
        target_model_types: ['general'],
        language: 'en',
        include_benign_variants: false,
        safety_ceiling: 'high',
        purpose: 'stress_testing',
      };

      const response = await handler({
        body: input,
        headers: {},
        method: 'POST',
        path: '/adversarial-prompt',
      });

      expect(response.statusCode).toBe(200);

      const result = JSON.parse(response.body);
      expect(result.data.prompts.length).toBeLessThanOrEqual(maxCount);
    });

    it('should apply safety ceiling', async () => {
      const input: AdversarialPromptInput = {
        categories: ['prompt_injection'],
        severities: ['low', 'medium', 'high', 'critical'],
        count_per_category: 2,
        total_max_count: 50,
        strategy: 'template_based',
        target_model_types: ['general'],
        language: 'en',
        include_benign_variants: false,
        safety_ceiling: 'medium', // Should filter out high and critical
        purpose: 'stress_testing',
      };

      const response = await handler({
        body: input,
        headers: {},
        method: 'POST',
        path: '/adversarial-prompt',
      });

      expect(response.statusCode).toBe(200);

      const result = JSON.parse(response.body);
      expect(result.success).toBe(true);

      // Verify no prompts exceed the ceiling
      for (const prompt of result.data.prompts) {
        expect(['low', 'medium']).toContain(prompt.severity);
      }

      // Verify constraint was applied
      expect(result.data.constraints_applied).toContain('severity_ceiling_applied');
    });
  });

  describe('Output Validation', () => {
    it('should return valid output schema', async () => {
      const input: AdversarialPromptInput = {
        categories: ['prompt_injection'],
        severities: ['low'],
        count_per_category: 1,
        total_max_count: 5,
        strategy: 'template_based',
        target_model_types: ['general'],
        language: 'en',
        include_benign_variants: true,
        safety_ceiling: 'high',
        purpose: 'stress_testing',
      };

      const response = await handler({
        body: input,
        headers: {},
        method: 'POST',
        path: '/adversarial-prompt',
      });

      const result = JSON.parse(response.body);
      expect(result.success).toBe(true);

      // Validate output against schema
      const validation = AdversarialPromptOutputSchema.safeParse(result.data);
      expect(validation.success).toBe(true);
    });

    it('should include required response headers', async () => {
      const input: AdversarialPromptInput = {
        categories: ['prompt_injection'],
        severities: ['low'],
        count_per_category: 1,
        total_max_count: 5,
        strategy: 'template_based',
        target_model_types: ['general'],
        language: 'en',
        include_benign_variants: false,
        safety_ceiling: 'high',
        purpose: 'stress_testing',
      };

      const response = await handler({
        body: input,
        headers: {},
        method: 'POST',
        path: '/adversarial-prompt',
      });

      expect(response.headers['Content-Type']).toBe('application/json');
      expect(response.headers['X-Agent-Id']).toBe(ADVERSARIAL_PROMPT_AGENT.agent_id);
      expect(response.headers['X-Agent-Version']).toBe(ADVERSARIAL_PROMPT_AGENT.agent_version);
      expect(response.headers['X-Decision-Id']).toBeDefined();
    });

    it('should return decision ID in response body', async () => {
      const input: AdversarialPromptInput = {
        categories: ['prompt_injection'],
        severities: ['low'],
        count_per_category: 1,
        total_max_count: 5,
        strategy: 'template_based',
        target_model_types: ['general'],
        language: 'en',
        include_benign_variants: false,
        safety_ceiling: 'high',
        purpose: 'stress_testing',
      };

      const response = await handler({
        body: input,
        headers: {},
        method: 'POST',
        path: '/adversarial-prompt',
      });

      const result = JSON.parse(response.body);
      expect(result.decision_id).toBeDefined();
      expect(result.decision_id).toMatch(/^[0-9a-f-]{36}$/i); // UUID format
    });
  });

  describe('Generated Prompt Properties', () => {
    it('should include prompt hash', async () => {
      const input: AdversarialPromptInput = {
        categories: ['encoding_attacks'],
        severities: ['low'],
        count_per_category: 1,
        total_max_count: 5,
        strategy: 'template_based',
        target_model_types: ['general'],
        language: 'en',
        include_benign_variants: false,
        safety_ceiling: 'high',
        purpose: 'stress_testing',
      };

      const response = await handler({
        body: input,
        headers: {},
        method: 'POST',
        path: '/adversarial-prompt',
      });

      const result = JSON.parse(response.body);
      for (const prompt of result.data.prompts) {
        expect(prompt.prompt_hash).toBeDefined();
        expect(prompt.prompt_hash).toHaveLength(64); // SHA-256 hex
      }
    });

    it('should include attack vector and expected behavior', async () => {
      const input: AdversarialPromptInput = {
        categories: ['prompt_injection'],
        severities: ['low'],
        count_per_category: 1,
        total_max_count: 5,
        strategy: 'template_based',
        target_model_types: ['general'],
        language: 'en',
        include_benign_variants: false,
        safety_ceiling: 'high',
        purpose: 'stress_testing',
      };

      const response = await handler({
        body: input,
        headers: {},
        method: 'POST',
        path: '/adversarial-prompt',
      });

      const result = JSON.parse(response.body);
      for (const prompt of result.data.prompts) {
        expect(prompt.attack_vector).toBeDefined();
        expect(prompt.expected_behavior).toBeDefined();
        expect(prompt.failure_indicators).toBeDefined();
        expect(Array.isArray(prompt.failure_indicators)).toBe(true);
      }
    });

    it('should include benign variants when requested', async () => {
      const input: AdversarialPromptInput = {
        categories: ['prompt_injection'],
        severities: ['low'],
        count_per_category: 2,
        total_max_count: 10,
        strategy: 'template_based',
        target_model_types: ['general'],
        language: 'en',
        include_benign_variants: true,
        safety_ceiling: 'high',
        purpose: 'stress_testing',
      };

      const response = await handler({
        body: input,
        headers: {},
        method: 'POST',
        path: '/adversarial-prompt',
      });

      const result = JSON.parse(response.body);
      const promptsWithBenign = result.data.prompts.filter((p: any) => p.benign_variant);
      expect(promptsWithBenign.length).toBeGreaterThan(0);
    });

    it('should calculate complexity score', async () => {
      const input: AdversarialPromptInput = {
        categories: ['prompt_injection'],
        severities: ['low', 'medium', 'high'],
        count_per_category: 1,
        total_max_count: 10,
        strategy: 'template_based',
        target_model_types: ['general'],
        language: 'en',
        include_benign_variants: false,
        safety_ceiling: 'high',
        purpose: 'stress_testing',
      };

      const response = await handler({
        body: input,
        headers: {},
        method: 'POST',
        path: '/adversarial-prompt',
      });

      const result = JSON.parse(response.body);
      for (const prompt of result.data.prompts) {
        expect(prompt.complexity_score).toBeGreaterThanOrEqual(0);
        expect(prompt.complexity_score).toBeLessThanOrEqual(1);
      }
    });
  });

  describe('Quality Metrics', () => {
    it('should include quality metrics in output', async () => {
      const input: AdversarialPromptInput = {
        categories: ['prompt_injection', 'encoding_attacks'],
        severities: ['low', 'medium'],
        count_per_category: 3,
        total_max_count: 20,
        strategy: 'template_based',
        target_model_types: ['general'],
        language: 'en',
        include_benign_variants: false,
        safety_ceiling: 'high',
        purpose: 'stress_testing',
      };

      const response = await handler({
        body: input,
        headers: {},
        method: 'POST',
        path: '/adversarial-prompt',
      });

      const result = JSON.parse(response.body);
      const metrics = result.data.quality_metrics;

      expect(metrics).toBeDefined();
      expect(metrics.total_generated).toBeGreaterThan(0);
      expect(metrics.diversity_score).toBeGreaterThanOrEqual(0);
      expect(metrics.diversity_score).toBeLessThanOrEqual(1);
      expect(metrics.category_coverage).toBeGreaterThanOrEqual(0);
      expect(metrics.category_coverage).toBeLessThanOrEqual(1);
    });

    it('should include category summaries', async () => {
      const input: AdversarialPromptInput = {
        categories: ['prompt_injection', 'encoding_attacks'],
        severities: ['low'],
        count_per_category: 2,
        total_max_count: 20,
        strategy: 'template_based',
        target_model_types: ['general'],
        language: 'en',
        include_benign_variants: false,
        safety_ceiling: 'high',
        purpose: 'stress_testing',
      };

      const response = await handler({
        body: input,
        headers: {},
        method: 'POST',
        path: '/adversarial-prompt',
      });

      const result = JSON.parse(response.body);
      expect(result.data.category_summaries).toBeDefined();
      expect(result.data.category_summaries.length).toBeGreaterThan(0);

      for (const summary of result.data.category_summaries) {
        expect(summary.category).toBeDefined();
        expect(summary.total_generated).toBeGreaterThanOrEqual(0);
        expect(summary.by_severity).toBeDefined();
      }
    });
  });

  describe('Timing and Metadata', () => {
    it('should include timing information', async () => {
      const input: AdversarialPromptInput = {
        categories: ['prompt_injection'],
        severities: ['low'],
        count_per_category: 1,
        total_max_count: 5,
        strategy: 'template_based',
        target_model_types: ['general'],
        language: 'en',
        include_benign_variants: false,
        safety_ceiling: 'high',
        purpose: 'stress_testing',
      };

      const response = await handler({
        body: input,
        headers: {},
        method: 'POST',
        path: '/adversarial-prompt',
      });

      const result = JSON.parse(response.body);
      expect(result.data.started_at).toBeDefined();
      expect(result.data.completed_at).toBeDefined();
      expect(result.data.duration_ms).toBeGreaterThanOrEqual(0);
    });

    it('should echo request summary', async () => {
      const input: AdversarialPromptInput = {
        categories: ['prompt_injection', 'encoding_attacks'],
        severities: ['low', 'medium'],
        count_per_category: 1,
        total_max_count: 10,
        strategy: 'template_based',
        target_model_types: ['general'],
        language: 'en',
        include_benign_variants: false,
        safety_ceiling: 'high',
        purpose: 'red_team_testing',
      };

      const response = await handler({
        body: input,
        headers: {},
        method: 'POST',
        path: '/adversarial-prompt',
      });

      const result = JSON.parse(response.body);
      expect(result.data.request_summary).toBeDefined();
      expect(result.data.request_summary.categories_requested).toContain('prompt_injection');
      expect(result.data.request_summary.severities_requested).toContain('low');
      expect(result.data.request_summary.strategy_used).toBe('template_based');
      expect(result.data.request_summary.purpose).toBe('red_team_testing');
    });
  });
});

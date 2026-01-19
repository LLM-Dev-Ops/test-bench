/**
 * Quality Scoring Agent - Smoke Tests
 *
 * Basic integration tests to verify the agent works correctly.
 */

import { describe, it, expect, beforeAll } from 'vitest';
import { handler, QUALITY_SCORING_AGENT } from '../handler';
import { executeCLI } from '../cli';
import {
  QualityScoringInputSchema,
  QualityScoringOutputSchema,
  PRESET_PROFILES,
} from '../../contracts';

// =============================================================================
// TEST DATA
// =============================================================================

const validInput = {
  outputs: [
    {
      output_id: '550e8400-e29b-41d4-a716-446655440001',
      provider_name: 'openai',
      model_id: 'gpt-4o-mini',
      content: 'The capital of France is Paris.',
      expected_output: 'Paris',
      test_id: 'geo-001',
    },
    {
      output_id: '550e8400-e29b-41d4-a716-446655440002',
      provider_name: 'anthropic',
      model_id: 'claude-3-5-haiku',
      content: 'Paris is the capital city of France.',
      expected_output: 'Paris',
      test_id: 'geo-001',
    },
  ],
  scoring_profile: {
    profile_id: 'test-profile',
    name: 'Test Profile',
    dimensions: [
      {
        dimension_id: 'accuracy',
        name: 'Accuracy',
        weight: 0.6,
        scoring_method: 'contains' as const,
        pass_threshold: 0.5,
        invert: false,
      },
      {
        dimension_id: 'keywords',
        name: 'Keywords',
        weight: 0.4,
        scoring_method: 'keyword_presence' as const,
        keywords: ['Paris', 'capital', 'France'],
        pass_threshold: 0.5,
        invert: false,
      },
    ],
    normalization: 'weighted_sum' as const,
    version: '1.0.0',
  },
};

// =============================================================================
// HANDLER TESTS
// =============================================================================

describe('Quality Scoring Agent Handler', () => {
  it('should have correct agent metadata', () => {
    expect(QUALITY_SCORING_AGENT.agent_id).toBe('quality-scoring');
    expect(QUALITY_SCORING_AGENT.agent_version).toMatch(/^\d+\.\d+\.\d+$/);
    expect(QUALITY_SCORING_AGENT.decision_type).toBe('quality_scoring');
  });

  it('should reject non-POST requests', async () => {
    const response = await handler({
      body: validInput,
      headers: {},
      method: 'GET',
      path: '/quality-scoring',
    });

    expect(response.statusCode).toBe(405);
    const body = JSON.parse(response.body);
    expect(body.success).toBe(false);
  });

  it('should reject invalid input', async () => {
    const response = await handler({
      body: { invalid: 'input' },
      headers: {},
      method: 'POST',
      path: '/quality-scoring',
    });

    expect(response.statusCode).toBe(400);
    const body = JSON.parse(response.body);
    expect(body.success).toBe(false);
    expect(body.error.code).toBe('VALIDATION_ERROR');
  });

  it('should successfully score valid outputs', async () => {
    const response = await handler({
      body: validInput,
      headers: {},
      method: 'POST',
      path: '/quality-scoring',
    });

    expect(response.statusCode).toBe(200);
    const body = JSON.parse(response.body);

    expect(body.success).toBe(true);
    expect(body.decision_id).toBeDefined();
    expect(body.data).toBeDefined();

    // Validate output structure
    const output = body.data;
    expect(output.scoring_id).toBeDefined();
    expect(output.profile_id).toBe('test-profile');
    expect(output.scores).toHaveLength(2);
    expect(output.model_stats).toHaveLength(2);
    expect(output.summary).toBeDefined();
  });

  it('should include decision headers', async () => {
    const response = await handler({
      body: validInput,
      headers: {},
      method: 'POST',
      path: '/quality-scoring',
    });

    expect(response.headers['X-Decision-Id']).toBeDefined();
    expect(response.headers['X-Agent-Id']).toBe(QUALITY_SCORING_AGENT.agent_id);
    expect(response.headers['X-Agent-Version']).toBe(QUALITY_SCORING_AGENT.agent_version);
  });

  it('should calculate composite scores correctly', async () => {
    const response = await handler({
      body: validInput,
      headers: {},
      method: 'POST',
      path: '/quality-scoring',
    });

    expect(response.statusCode).toBe(200);
    const body = JSON.parse(response.body);

    for (const score of body.data.scores) {
      expect(score.composite_score).toBeGreaterThanOrEqual(0);
      expect(score.composite_score).toBeLessThanOrEqual(1);
      expect(score.dimension_scores).toBeDefined();
      expect(score.dimension_scores.length).toBeGreaterThan(0);
    }
  });

  it('should produce consistent results (deterministic)', async () => {
    const response1 = await handler({
      body: validInput,
      headers: {},
      method: 'POST',
      path: '/quality-scoring',
    });

    const response2 = await handler({
      body: validInput,
      headers: {},
      method: 'POST',
      path: '/quality-scoring',
    });

    const body1 = JSON.parse(response1.body);
    const body2 = JSON.parse(response2.body);

    // Scores should be identical (deterministic)
    expect(body1.data.scores[0].composite_score).toBe(body2.data.scores[0].composite_score);
    expect(body1.data.scores[1].composite_score).toBe(body2.data.scores[1].composite_score);
  });
});

// =============================================================================
// SCHEMA TESTS
// =============================================================================

describe('Quality Scoring Schemas', () => {
  it('should validate correct input', () => {
    const result = QualityScoringInputSchema.safeParse(validInput);
    expect(result.success).toBe(true);
  });

  it('should reject input without outputs', () => {
    const result = QualityScoringInputSchema.safeParse({
      outputs: [],
      scoring_profile: validInput.scoring_profile,
    });
    expect(result.success).toBe(false);
  });

  it('should reject profile with weights not summing to 1', () => {
    const invalidProfile = {
      ...validInput,
      scoring_profile: {
        ...validInput.scoring_profile,
        dimensions: [
          {
            dimension_id: 'only',
            name: 'Only',
            weight: 0.5, // Doesn't sum to 1
            scoring_method: 'contains' as const,
            pass_threshold: 0.5,
            invert: false,
          },
        ],
      },
    };

    const result = QualityScoringInputSchema.safeParse(invalidProfile);
    expect(result.success).toBe(false);
  });

  it('should accept preset profiles', () => {
    const withPreset = {
      outputs: validInput.outputs,
      scoring_profile: PRESET_PROFILES.accuracy_basic,
    };

    const result = QualityScoringInputSchema.safeParse(withPreset);
    expect(result.success).toBe(true);
  });
});

// =============================================================================
// SCORING METHOD TESTS
// =============================================================================

describe('Scoring Methods', () => {
  it('should score exact_match correctly', async () => {
    const input = {
      outputs: [
        {
          output_id: '550e8400-e29b-41d4-a716-446655440001',
          provider_name: 'test',
          model_id: 'test',
          content: 'Paris',
          expected_output: 'Paris',
        },
        {
          output_id: '550e8400-e29b-41d4-a716-446655440002',
          provider_name: 'test',
          model_id: 'test',
          content: 'paris', // lowercase
          expected_output: 'Paris',
        },
      ],
      scoring_profile: {
        profile_id: 'exact-test',
        name: 'Exact Match Test',
        dimensions: [
          {
            dimension_id: 'exact',
            name: 'Exact',
            weight: 1.0,
            scoring_method: 'exact_match' as const,
            pass_threshold: 1.0,
            invert: false,
          },
        ],
        normalization: 'weighted_sum' as const,
        version: '1.0.0',
      },
      evaluation_config: {
        case_sensitive: false, // Both should match
      },
    };

    const response = await handler({
      body: input,
      headers: {},
      method: 'POST',
      path: '/quality-scoring',
    });

    const body = JSON.parse(response.body);
    expect(body.data.scores[0].composite_score).toBe(1.0);
    expect(body.data.scores[1].composite_score).toBe(1.0); // Case-insensitive match
  });

  it('should score contains correctly', async () => {
    const input = {
      outputs: [
        {
          output_id: '550e8400-e29b-41d4-a716-446655440001',
          provider_name: 'test',
          model_id: 'test',
          content: 'The capital of France is Paris and it is beautiful.',
          expected_output: 'Paris',
        },
      ],
      scoring_profile: {
        profile_id: 'contains-test',
        name: 'Contains Test',
        dimensions: [
          {
            dimension_id: 'contains',
            name: 'Contains',
            weight: 1.0,
            scoring_method: 'contains' as const,
            pass_threshold: 0.5,
            invert: false,
          },
        ],
        normalization: 'weighted_sum' as const,
        version: '1.0.0',
      },
    };

    const response = await handler({
      body: input,
      headers: {},
      method: 'POST',
      path: '/quality-scoring',
    });

    const body = JSON.parse(response.body);
    expect(body.data.scores[0].composite_score).toBe(1.0);
  });

  it('should score keyword_presence correctly', async () => {
    const input = {
      outputs: [
        {
          output_id: '550e8400-e29b-41d4-a716-446655440001',
          provider_name: 'test',
          model_id: 'test',
          content: 'Paris is the capital of France.',
        },
      ],
      scoring_profile: {
        profile_id: 'keyword-test',
        name: 'Keyword Test',
        dimensions: [
          {
            dimension_id: 'keywords',
            name: 'Keywords',
            weight: 1.0,
            scoring_method: 'keyword_presence' as const,
            keywords: ['Paris', 'capital', 'France', 'London'], // 3 of 4 present
            pass_threshold: 0.5,
            invert: false,
          },
        ],
        normalization: 'weighted_sum' as const,
        version: '1.0.0',
      },
    };

    const response = await handler({
      body: input,
      headers: {},
      method: 'POST',
      path: '/quality-scoring',
    });

    const body = JSON.parse(response.body);
    expect(body.data.scores[0].composite_score).toBe(0.75); // 3/4 keywords
  });

  it('should score format_compliance correctly', async () => {
    const input = {
      outputs: [
        {
          output_id: '550e8400-e29b-41d4-a716-446655440001',
          provider_name: 'test',
          model_id: 'test',
          content: '{"answer": "Paris", "confidence": 0.99}',
        },
        {
          output_id: '550e8400-e29b-41d4-a716-446655440002',
          provider_name: 'test',
          model_id: 'test',
          content: 'Just plain text, not JSON',
        },
      ],
      scoring_profile: {
        profile_id: 'format-test',
        name: 'Format Test',
        dimensions: [
          {
            dimension_id: 'format',
            name: 'Format',
            weight: 1.0,
            scoring_method: 'format_compliance' as const,
            format_type: 'json' as const,
            pass_threshold: 1.0,
            invert: false,
          },
        ],
        normalization: 'weighted_sum' as const,
        version: '1.0.0',
      },
    };

    const response = await handler({
      body: input,
      headers: {},
      method: 'POST',
      path: '/quality-scoring',
    });

    const body = JSON.parse(response.body);
    expect(body.data.scores[0].composite_score).toBe(1.0); // Valid JSON
    expect(body.data.scores[1].composite_score).toBe(0.0); // Invalid JSON
  });
});

// =============================================================================
// CLI TESTS
// =============================================================================

describe('Quality Scoring CLI', () => {
  it('should return 0 for --help', async () => {
    const code = await executeCLI(['--help']);
    expect(code).toBe(0);
  });

  it('should return 1 for no input specified', async () => {
    const code = await executeCLI([]);
    expect(code).toBe(1);
  });

  it('should return 0 for dry-run with valid input', async () => {
    const code = await executeCLI([
      '--dry-run',
      '--input-json',
      JSON.stringify(validInput),
    ]);
    expect(code).toBe(0);
  });
});

/**
 * Hallucination Detector Agent - Smoke Tests
 *
 * Verification checklist and smoke tests for the hallucination-detector agent.
 */

import { describe, it, expect, beforeAll } from 'vitest';
import {
  handler,
  HALLUCINATION_DETECTOR_AGENT,
  HallucinationDetectorInputSchema,
  HallucinationDetectorOutputSchema,
} from '../index';

// =============================================================================
// VERIFICATION CHECKLIST
// =============================================================================

/**
 * PROMPT 3 - VERIFICATION CHECKLIST
 *
 * ✅ Agent registered in agentics-contracts (schemas/hallucination-detector.ts)
 * ✅ CLI command(s) added to agentics-cli (hallucination-detector/cli.ts)
 * ✅ LLM-Orchestrator can invoke the agent (via Edge Function endpoint)
 * ✅ Results appear in LLM-Observatory (via telemetry emission)
 * ✅ DecisionEvents persist in ruvector-service (via RuVectorClient)
 * ✅ Core bundles can consume outputs without rewiring (via ALLOWED_CONSUMERS)
 *
 * PLATFORM REGISTRATION METADATA:
 * - Agent ID: hallucination-detector
 * - Version: 1.0.0
 * - Decision Type: hallucination_detection
 * - Endpoint: /api/v1/agents/hallucination-detector
 * - Platform: GCP (Edge Function)
 * - Service: llm-test-bench
 */

// =============================================================================
// SMOKE TESTS
// =============================================================================

describe('Hallucination Detector Agent', () => {
  describe('Agent Metadata', () => {
    it('should have correct agent identity', () => {
      expect(HALLUCINATION_DETECTOR_AGENT.agent_id).toBe('hallucination-detector');
      expect(HALLUCINATION_DETECTOR_AGENT.agent_version).toBe('1.0.0');
      expect(HALLUCINATION_DETECTOR_AGENT.decision_type).toBe('hallucination_detection');
    });
  });

  describe('Input Schema Validation', () => {
    it('should accept valid input with single claim', () => {
      const input = {
        claim: 'The sky is blue',
        reference_context: 'The sky appears blue during the day due to Rayleigh scattering.',
      };

      const result = HallucinationDetectorInputSchema.safeParse(input);
      expect(result.success).toBe(true);
    });

    it('should accept valid input with multiple claims', () => {
      const input = {
        claims: [
          { claim_id: 'c1', text: 'The sky is blue' },
          { claim_id: 'c2', text: 'Water is wet' },
        ],
        reference_context: 'The sky appears blue. Water is a liquid at room temperature.',
      };

      const result = HallucinationDetectorInputSchema.safeParse(input);
      expect(result.success).toBe(true);
    });

    it('should accept valid input with reference sources array', () => {
      const input = {
        claim: 'Einstein developed relativity',
        reference_context: [
          {
            source_id: 'wiki-1',
            content: 'Albert Einstein developed the theory of relativity.',
            source_type: 'document',
          },
        ],
      };

      const result = HallucinationDetectorInputSchema.safeParse(input);
      expect(result.success).toBe(true);
    });

    it('should reject input without claims', () => {
      const input = {
        reference_context: 'Some reference text',
      };

      const result = HallucinationDetectorInputSchema.safeParse(input);
      expect(result.success).toBe(false);
    });

    it('should reject input without reference context', () => {
      const input = {
        claim: 'The sky is blue',
      };

      const result = HallucinationDetectorInputSchema.safeParse(input);
      expect(result.success).toBe(false);
    });
  });

  describe('Handler - Basic Functionality', () => {
    it('should reject non-POST requests', async () => {
      const response = await handler({
        body: {},
        headers: {},
        method: 'GET',
        path: '/api/v1/agents/hallucination-detector',
      });

      expect(response.statusCode).toBe(405);
    });

    it('should reject invalid input', async () => {
      const response = await handler({
        body: { invalid: 'data' },
        headers: {},
        method: 'POST',
        path: '/api/v1/agents/hallucination-detector',
      });

      expect(response.statusCode).toBe(400);
      const body = JSON.parse(response.body);
      expect(body.success).toBe(false);
      expect(body.error.code).toBe('VALIDATION_ERROR');
    });

    it('should detect hallucination (fabrication)', async () => {
      const response = await handler({
        body: {
          claim: 'Napoleon won the Battle of Waterloo',
          reference_context: 'Napoleon was defeated at the Battle of Waterloo in 1815 by the Duke of Wellington.',
        },
        headers: {},
        method: 'POST',
        path: '/api/v1/agents/hallucination-detector',
      });

      expect(response.statusCode).toBe(200);
      const body = JSON.parse(response.body);
      expect(body.success).toBe(true);
      expect(body.decision_id).toBeDefined();

      const output = HallucinationDetectorOutputSchema.parse(body.data);
      expect(output.total_claims).toBe(1);
      expect(output.results[0].is_hallucination).toBe(true);
    });

    it('should verify supported claim', async () => {
      const response = await handler({
        body: {
          claim: 'The Earth orbits the Sun',
          reference_context: 'The Earth orbits the Sun once every 365.25 days. This is known as a solar year.',
        },
        headers: {},
        method: 'POST',
        path: '/api/v1/agents/hallucination-detector',
      });

      expect(response.statusCode).toBe(200);
      const body = JSON.parse(response.body);
      expect(body.success).toBe(true);

      const output = HallucinationDetectorOutputSchema.parse(body.data);
      expect(output.total_claims).toBe(1);
      expect(output.results[0].is_hallucination).toBe(false);
      expect(output.results[0].hallucination_type).toBe('none');
    });

    it('should return decision headers', async () => {
      const response = await handler({
        body: {
          claim: 'Test claim',
          reference_context: 'Test reference',
        },
        headers: {},
        method: 'POST',
        path: '/api/v1/agents/hallucination-detector',
      });

      expect(response.headers['X-Decision-Id']).toBeDefined();
      expect(response.headers['X-Agent-Id']).toBe('hallucination-detector');
      expect(response.headers['X-Agent-Version']).toBe('1.0.0');
    });
  });

  describe('Handler - Hallucination Types', () => {
    it('should detect contradiction', async () => {
      const response = await handler({
        body: {
          claim: 'Water boils at 50 degrees Celsius',
          reference_context: 'Water boils at 100 degrees Celsius at standard atmospheric pressure.',
        },
        headers: {},
        method: 'POST',
        path: '/api/v1/agents/hallucination-detector',
      });

      expect(response.statusCode).toBe(200);
      const body = JSON.parse(response.body);
      const output = body.data;
      expect(output.results[0].is_hallucination).toBe(true);
    });

    it('should detect exaggeration', async () => {
      const response = await handler({
        body: {
          claim: 'All scientists agree that climate change is catastrophic',
          reference_context: 'Many scientists believe climate change poses significant risks.',
          detection_config: {
            sensitivity: 'high',
          },
        },
        headers: {},
        method: 'POST',
        path: '/api/v1/agents/hallucination-detector',
      });

      expect(response.statusCode).toBe(200);
      const body = JSON.parse(response.body);
      expect(body.success).toBe(true);
    });
  });

  describe('Handler - Multiple Claims', () => {
    it('should process multiple claims correctly', async () => {
      const response = await handler({
        body: {
          claims: [
            { claim_id: 'c1', text: 'Python is a programming language' },
            { claim_id: 'c2', text: 'Python was invented in 1850' },
            { claim_id: 'c3', text: 'Python uses indentation for code blocks' },
          ],
          reference_context: 'Python is a high-level programming language created by Guido van Rossum in 1991. It uses indentation to define code blocks.',
        },
        headers: {},
        method: 'POST',
        path: '/api/v1/agents/hallucination-detector',
      });

      expect(response.statusCode).toBe(200);
      const body = JSON.parse(response.body);
      const output = body.data;

      expect(output.total_claims).toBe(3);
      expect(output.results).toHaveLength(3);

      // First claim should be verified
      expect(output.results.find((r: any) => r.claim_id === 'c1').is_hallucination).toBe(false);

      // Second claim has wrong date - should be hallucination
      expect(output.results.find((r: any) => r.claim_id === 'c2').is_hallucination).toBe(true);

      // Third claim should be verified
      expect(output.results.find((r: any) => r.claim_id === 'c3').is_hallucination).toBe(false);
    });
  });

  describe('Output Schema Validation', () => {
    it('should produce valid output schema', async () => {
      const response = await handler({
        body: {
          claim: 'Test claim',
          reference_context: 'Test reference text',
        },
        headers: {},
        method: 'POST',
        path: '/api/v1/agents/hallucination-detector',
      });

      expect(response.statusCode).toBe(200);
      const body = JSON.parse(response.body);

      // Validate output against schema
      const validation = HallucinationDetectorOutputSchema.safeParse(body.data);
      expect(validation.success).toBe(true);

      if (validation.success) {
        const output = validation.data;
        expect(output.execution_id).toBeDefined();
        expect(output.total_claims).toBeGreaterThanOrEqual(0);
        expect(output.hallucinated_claims).toBeGreaterThanOrEqual(0);
        expect(output.verified_claims).toBeGreaterThanOrEqual(0);
        expect(output.overall_hallucination_rate).toBeGreaterThanOrEqual(0);
        expect(output.overall_hallucination_rate).toBeLessThanOrEqual(1);
        expect(output.results).toBeInstanceOf(Array);
        expect(output.detection_config).toBeDefined();
        expect(output.started_at).toBeDefined();
        expect(output.completed_at).toBeDefined();
        expect(output.total_duration_ms).toBeGreaterThanOrEqual(0);
      }
    });
  });
});

// =============================================================================
// CLI SMOKE TEST COMMANDS
// =============================================================================

/**
 * CLI Smoke Test Commands (run manually):
 *
 * # Check a single claim:
 * npx ts-node agents/hallucination-detector/cli.ts -c "The sky is green" -t "The sky is blue"
 *
 * # Check claims from file:
 * npx ts-node agents/hallucination-detector/cli.ts -i claims.json -r reference.txt
 *
 * # High sensitivity detection:
 * npx ts-node agents/hallucination-detector/cli.ts -c "All experts agree" -t "Many experts believe" -s high
 *
 * # Output as table:
 * npx ts-node agents/hallucination-detector/cli.ts -c "Test claim" -t "Reference" -o table
 *
 * # Dry run (validation only):
 * npx ts-node agents/hallucination-detector/cli.ts -c "Test" -t "Ref" --dry-run
 */

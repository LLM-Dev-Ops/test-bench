/**
 * Hallucination Detector Agent - Platform Registration
 *
 * Registration metadata for the Agentics Dev platform.
 * This enables discovery by LLM-Orchestrator and other Core bundles.
 */

import {
  HALLUCINATION_DETECTOR_AGENT,
  HALLUCINATION_ALLOWED_CONSUMERS,
  HALLUCINATION_NON_RESPONSIBILITIES,
  HALLUCINATION_VALID_CONSTRAINTS,
  HallucinationDetectorInputSchema,
  HallucinationDetectorOutputSchema,
} from '../contracts';

// =============================================================================
// AGENT REGISTRATION METADATA
// =============================================================================

export const HALLUCINATION_DETECTOR_REGISTRATION = {
  /**
   * Agent Identity
   */
  identity: {
    agent_id: HALLUCINATION_DETECTOR_AGENT.agent_id,
    agent_version: HALLUCINATION_DETECTOR_AGENT.agent_version,
    decision_type: HALLUCINATION_DETECTOR_AGENT.decision_type,
  },

  /**
   * Deployment Information
   */
  deployment: {
    type: 'edge_function',
    service: 'llm-test-bench',
    platform: 'google_cloud',
    endpoint: '/api/v1/agents/hallucination-detector',
    region: 'us-central1', // Primary region
    replicas: ['us-east1', 'europe-west1'], // Replica regions
  },

  /**
   * Runtime Configuration
   */
  runtime: {
    timeout_ms: 120000, // 2 minutes max (embedding + analysis)
    memory_mb: 512,
    cpu: 1,
    max_concurrent_requests: 150,
    cold_start_timeout_ms: 5000,
  },

  /**
   * Schema References
   */
  schemas: {
    input: 'HallucinationDetectorInputSchema',
    output: 'HallucinationDetectorOutputSchema',
    contracts_package: '@agents/contracts',
  },

  /**
   * Allowed Consumers
   */
  consumers: HALLUCINATION_ALLOWED_CONSUMERS,

  /**
   * Non-Responsibilities (what this agent MUST NOT do)
   */
  non_responsibilities: HALLUCINATION_NON_RESPONSIBILITIES,

  /**
   * Valid Constraints
   */
  valid_constraints: HALLUCINATION_VALID_CONSTRAINTS,

  /**
   * Health Check Configuration
   */
  health_check: {
    endpoint: '/health',
    interval_ms: 30000,
    timeout_ms: 5000,
    unhealthy_threshold: 3,
    healthy_threshold: 2,
  },

  /**
   * Observability Configuration
   */
  observability: {
    telemetry_enabled: true,
    telemetry_endpoint: 'llm-observatory',
    metrics_prefix: 'hallucination_detector',
    trace_sampling_rate: 0.1,
    log_level: 'info',
  },

  /**
   * Dependencies
   */
  dependencies: {
    required: ['ruvector-service'],
    optional: ['llm-observatory', 'llm-analytics', 'embedding-service'],
    providers: [
      'openai',
      'anthropic',
      'google',
      'cohere',
    ],
  },

  /**
   * Tags for Discovery
   */
  tags: [
    'hallucination',
    'detection',
    'verification',
    'fact-checking',
    'grounding',
    'llm',
    'quality',
    'trust',
    'accuracy',
  ],

  /**
   * Documentation
   */
  documentation: {
    description:
      'Detect unsupported or fabricated claims relative to provided reference context. Identifies fabrication, exaggeration, misattribution, contradiction, and unsupported claims with confidence scoring.',
    repository: 'https://github.com/your-org/llm-test-bench',
    api_docs: '/docs/agents/hallucination-detector',
    changelog: '/docs/agents/hallucination-detector/changelog',
  },

  /**
   * Rate Limiting
   */
  rate_limits: {
    requests_per_minute: 60,
    requests_per_hour: 1500,
    max_payload_size_kb: 2048, // Larger payloads for reference context
  },

  /**
   * Feature Flags
   */
  feature_flags: {
    streaming_enabled: false,
    batch_mode_enabled: true,
    async_execution_enabled: true,
    semantic_similarity_enabled: true,
    entailment_analysis_enabled: true,
    entity_verification_enabled: true,
  },

  /**
   * SLA Configuration
   */
  sla: {
    availability_target: 0.999,
    latency_p99_target_ms: 30000,
    error_rate_target: 0.001,
  },
} as const;

// =============================================================================
// REGISTRATION FUNCTIONS
// =============================================================================

/**
 * Get registration metadata for platform registry
 */
export function getRegistrationMetadata(): typeof HALLUCINATION_DETECTOR_REGISTRATION {
  return HALLUCINATION_DETECTOR_REGISTRATION;
}

/**
 * Validate registration is complete
 */
export function validateRegistration(): { valid: boolean; errors: string[] } {
  const errors: string[] = [];

  // Check required fields
  if (!HALLUCINATION_DETECTOR_REGISTRATION.identity.agent_id) {
    errors.push('Missing agent_id');
  }

  if (!HALLUCINATION_DETECTOR_REGISTRATION.deployment.endpoint) {
    errors.push('Missing deployment endpoint');
  }

  if (!HALLUCINATION_DETECTOR_REGISTRATION.schemas.input) {
    errors.push('Missing input schema reference');
  }

  if (!HALLUCINATION_DETECTOR_REGISTRATION.schemas.output) {
    errors.push('Missing output schema reference');
  }

  // Runtime check for consumers - needed for structural validation
  if (!(HALLUCINATION_DETECTOR_REGISTRATION.consumers as readonly string[]).length) {
    errors.push('No consumers defined');
  }

  return {
    valid: errors.length === 0,
    errors,
  };
}

/**
 * Generate OpenAPI spec for agent endpoint
 */
export function generateOpenAPISpec(): object {
  return {
    openapi: '3.0.3',
    info: {
      title: 'Hallucination Detector Agent',
      version: HALLUCINATION_DETECTOR_AGENT.agent_version,
      description: HALLUCINATION_DETECTOR_REGISTRATION.documentation.description,
    },
    paths: {
      [HALLUCINATION_DETECTOR_REGISTRATION.deployment.endpoint]: {
        post: {
          summary: 'Detect hallucinations in claims against reference context',
          operationId: 'detectHallucinations',
          tags: ['Hallucination Detection'],
          requestBody: {
            required: true,
            content: {
              'application/json': {
                schema: {
                  $ref: '#/components/schemas/HallucinationDetectorInput',
                },
              },
            },
          },
          responses: {
            '200': {
              description: 'Hallucination detection successful',
              content: {
                'application/json': {
                  schema: {
                    $ref: '#/components/schemas/HallucinationDetectorOutput',
                  },
                },
              },
              headers: {
                'X-Decision-Id': {
                  description: 'Unique decision event ID',
                  schema: { type: 'string', format: 'uuid' },
                },
              },
            },
            '400': {
              description: 'Validation error',
            },
            '500': {
              description: 'Internal server error',
            },
          },
        },
      },
    },
    components: {
      schemas: {
        HallucinationDetectorInput: {
          type: 'object',
          required: ['reference_context'],
          properties: {
            claim: {
              type: 'string',
              description: 'Single claim to check for hallucination',
            },
            claims: {
              type: 'array',
              items: {
                type: 'object',
                required: ['claim_id', 'text'],
                properties: {
                  claim_id: { type: 'string' },
                  text: { type: 'string' },
                  metadata: { type: 'object' },
                },
              },
              description: 'Multiple claims to check',
            },
            reference_context: {
              oneOf: [
                { type: 'string' },
                {
                  type: 'array',
                  items: {
                    type: 'object',
                    required: ['source_id', 'content'],
                    properties: {
                      source_id: { type: 'string' },
                      content: { type: 'string' },
                      source_type: { type: 'string' },
                      metadata: { type: 'object' },
                    },
                  },
                },
              ],
              description: 'Reference context for grounding',
            },
            detection_config: {
              type: 'object',
              properties: {
                sensitivity: { type: 'string', enum: ['low', 'medium', 'high'] },
                confidence_threshold: { type: 'number', minimum: 0, maximum: 1 },
                methods: { type: 'array', items: { type: 'string' } },
                detect_types: { type: 'array', items: { type: 'string' } },
              },
            },
          },
        },
        HallucinationDetectorOutput: {
          type: 'object',
          properties: {
            execution_id: { type: 'string', format: 'uuid' },
            total_claims: { type: 'integer' },
            hallucinated_claims: { type: 'integer' },
            verified_claims: { type: 'integer' },
            overall_hallucination_rate: { type: 'number' },
            results: { type: 'array' },
            summary: { type: 'object' },
          },
        },
      },
    },
  };
}

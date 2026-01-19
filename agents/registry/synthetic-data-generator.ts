/**
 * Synthetic Data Generator Agent - Platform Registration
 *
 * Registration metadata for the Agentics Dev platform.
 * This enables discovery by LLM-Orchestrator and other Core bundles.
 */

import {
  SYNTHETIC_DATA_GENERATOR_AGENT,
  SYNTHETIC_DATA_ALLOWED_CONSUMERS,
  SYNTHETIC_DATA_NON_RESPONSIBILITIES,
  SYNTHETIC_DATA_VALID_CONSTRAINTS,
  SyntheticDataGeneratorInputSchema,
  SyntheticDataGeneratorOutputSchema,
} from '../contracts';

// =============================================================================
// AGENT REGISTRATION METADATA
// =============================================================================

export const SYNTHETIC_DATA_GENERATOR_REGISTRATION = {
  /**
   * Agent Identity
   */
  identity: {
    agent_id: SYNTHETIC_DATA_GENERATOR_AGENT.agent_id,
    agent_version: SYNTHETIC_DATA_GENERATOR_AGENT.agent_version,
    decision_type: SYNTHETIC_DATA_GENERATOR_AGENT.decision_type,
  },

  /**
   * Deployment Information
   */
  deployment: {
    type: 'edge_function',
    service: 'llm-test-bench',
    platform: 'google_cloud',
    endpoint: '/api/v1/agents/synthetic-data-generator',
    region: 'us-central1', // Primary region
    replicas: ['us-east1', 'europe-west1'], // Replica regions
  },

  /**
   * Runtime Configuration
   */
  runtime: {
    timeout_ms: 300000, // 5 minutes max
    memory_mb: 1024,    // Higher for large generations
    cpu: 2,
    max_concurrent_requests: 50,
    cold_start_timeout_ms: 5000,
  },

  /**
   * Schema References
   */
  schemas: {
    input: 'SyntheticDataGeneratorInputSchema',
    output: 'SyntheticDataGeneratorOutputSchema',
    contracts_package: '@agents/contracts',
  },

  /**
   * Allowed Consumers
   */
  consumers: SYNTHETIC_DATA_ALLOWED_CONSUMERS,

  /**
   * Non-Responsibilities (what this agent MUST NOT do)
   */
  non_responsibilities: SYNTHETIC_DATA_NON_RESPONSIBILITIES,

  /**
   * Valid Constraints
   */
  valid_constraints: SYNTHETIC_DATA_VALID_CONSTRAINTS,

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
    metrics_prefix: 'synthetic_data_generator',
    trace_sampling_rate: 0.1,
    log_level: 'info',
  },

  /**
   * Dependencies
   */
  dependencies: {
    required: ['ruvector-service'],
    optional: ['llm-observatory', 'llm-analytics'],
    providers: [], // No external providers needed - pure algorithmic generation
  },

  /**
   * Tags for Discovery
   */
  tags: [
    'synthetic-data',
    'testing',
    'benchmark',
    'data-generation',
    'test-data',
    'qa-pairs',
    'coding-tasks',
  ],

  /**
   * Documentation
   */
  documentation: {
    description:
      'Generate synthetic datasets for testing, benchmarking, and stress evaluation of LLM systems using pure algorithmic generation (no LLM calls).',
    repository: 'https://github.com/your-org/llm-test-bench',
    api_docs: '/docs/agents/synthetic-data-generator',
    changelog: '/docs/agents/synthetic-data-generator/changelog',
  },

  /**
   * Rate Limiting
   */
  rate_limits: {
    requests_per_minute: 30,
    requests_per_hour: 500,
    max_payload_size_kb: 2048,
    max_items_per_request: 10000,
  },

  /**
   * Feature Flags
   */
  feature_flags: {
    streaming_enabled: false,
    batch_mode_enabled: true,
    async_execution_enabled: true,
    deterministic_mode_enabled: true,
  },

  /**
   * SLA Configuration
   */
  sla: {
    availability_target: 0.999,
    latency_p99_target_ms: 120000, // 2 minutes for large generations
    error_rate_target: 0.001,
  },
} as const;

// =============================================================================
// REGISTRATION FUNCTIONS
// =============================================================================

/**
 * Get registration metadata for platform registry
 */
export function getRegistrationMetadata(): typeof SYNTHETIC_DATA_GENERATOR_REGISTRATION {
  return SYNTHETIC_DATA_GENERATOR_REGISTRATION;
}

/**
 * Validate registration is complete
 */
export function validateRegistration(): { valid: boolean; errors: string[] } {
  const errors: string[] = [];

  // Check required fields
  if (!SYNTHETIC_DATA_GENERATOR_REGISTRATION.identity.agent_id) {
    errors.push('Missing agent_id');
  }

  if (!SYNTHETIC_DATA_GENERATOR_REGISTRATION.deployment.endpoint) {
    errors.push('Missing deployment endpoint');
  }

  if (!SYNTHETIC_DATA_GENERATOR_REGISTRATION.schemas.input) {
    errors.push('Missing input schema reference');
  }

  if (!SYNTHETIC_DATA_GENERATOR_REGISTRATION.schemas.output) {
    errors.push('Missing output schema reference');
  }

  // consumers is a readonly tuple, so length check is statically verified
  // This check is kept for runtime safety if the type is ever changed
  if ((SYNTHETIC_DATA_GENERATOR_REGISTRATION.consumers as readonly string[]).length === 0) {
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
      title: 'Synthetic Data Generator Agent',
      version: SYNTHETIC_DATA_GENERATOR_AGENT.agent_version,
      description: SYNTHETIC_DATA_GENERATOR_REGISTRATION.documentation.description,
    },
    paths: {
      [SYNTHETIC_DATA_GENERATOR_REGISTRATION.deployment.endpoint]: {
        post: {
          summary: 'Generate synthetic data',
          operationId: 'generateSyntheticData',
          tags: ['Synthetic Data Generator'],
          requestBody: {
            required: true,
            content: {
              'application/json': {
                schema: {
                  $ref: '#/components/schemas/SyntheticDataGeneratorInput',
                },
              },
            },
          },
          responses: {
            '200': {
              description: 'Data generation successful',
              content: {
                'application/json': {
                  schema: {
                    $ref: '#/components/schemas/SyntheticDataGeneratorOutput',
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
  };
}

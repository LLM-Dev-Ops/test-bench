/**
 * Golden Dataset Validator Agent - Platform Registration
 *
 * Registration metadata for the Agentics Dev platform.
 * This enables discovery by LLM-Orchestrator and other Core bundles.
 */

import {
  GOLDEN_DATASET_VALIDATOR_AGENT,
  GOLDEN_DATASET_ALLOWED_CONSUMERS,
  GOLDEN_DATASET_NON_RESPONSIBILITIES,
  GOLDEN_DATASET_VALID_CONSTRAINTS,
} from '../contracts';

// =============================================================================
// AGENT REGISTRATION METADATA
// =============================================================================

export const GOLDEN_DATASET_VALIDATOR_REGISTRATION = {
  /**
   * Agent Identity
   */
  identity: {
    agent_id: GOLDEN_DATASET_VALIDATOR_AGENT.agent_id,
    agent_version: GOLDEN_DATASET_VALIDATOR_AGENT.agent_version,
    decision_type: GOLDEN_DATASET_VALIDATOR_AGENT.decision_type,
  },

  /**
   * Deployment Information
   */
  deployment: {
    type: 'edge_function',
    service: 'llm-test-bench',
    platform: 'google_cloud',
    endpoint: '/api/v1/agents/golden-dataset-validator',
    region: 'us-central1', // Primary region
    replicas: ['us-east1', 'europe-west1'], // Replica regions
  },

  /**
   * Runtime Configuration
   */
  runtime: {
    timeout_ms: 300000, // 5 minutes max
    memory_mb: 512,
    cpu: 1,
    max_concurrent_requests: 100,
    cold_start_timeout_ms: 5000,
  },

  /**
   * Schema References
   */
  schemas: {
    input: 'GoldenDatasetValidatorInputSchema',
    output: 'GoldenDatasetValidatorOutputSchema',
    contracts_package: '@agents/contracts',
  },

  /**
   * Allowed Consumers
   */
  consumers: GOLDEN_DATASET_ALLOWED_CONSUMERS,

  /**
   * Non-Responsibilities (what this agent MUST NOT do)
   */
  non_responsibilities: GOLDEN_DATASET_NON_RESPONSIBILITIES,

  /**
   * Valid Constraints
   */
  valid_constraints: GOLDEN_DATASET_VALID_CONSTRAINTS,

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
    metrics_prefix: 'golden_dataset_validator',
    trace_sampling_rate: 0.1,
    log_level: 'info',
  },

  /**
   * Dependencies
   */
  dependencies: {
    required: ['ruvector-service'],
    optional: ['llm-observatory', 'llm-analytics'],
    providers: [], // This agent does not call external LLM providers
  },

  /**
   * Tags for Discovery
   */
  tags: [
    'validation',
    'golden-dataset',
    'quality',
    'accuracy',
    'testing',
    'llm',
    'evaluation',
    'semantic-similarity',
  ],

  /**
   * Documentation
   */
  documentation: {
    description:
      'Validate model outputs against canonical, human-verified datasets. Compares LLM responses to golden reference answers to measure accuracy, semantic similarity, and output quality.',
    repository: 'https://github.com/your-org/llm-test-bench',
    api_docs: '/docs/agents/golden-dataset-validator',
    changelog: '/docs/agents/golden-dataset-validator/changelog',
  },

  /**
   * Rate Limiting
   */
  rate_limits: {
    requests_per_minute: 60,
    requests_per_hour: 1000,
    max_payload_size_kb: 5120, // 5MB for large datasets
  },

  /**
   * Feature Flags
   */
  feature_flags: {
    streaming_enabled: false,
    batch_mode_enabled: true,
    async_execution_enabled: true,
    cost_tracking_enabled: false, // No external API costs
  },

  /**
   * SLA Configuration
   */
  sla: {
    availability_target: 0.999,
    latency_p99_target_ms: 60000,
    error_rate_target: 0.001,
  },
} as const;

// =============================================================================
// REGISTRATION FUNCTIONS
// =============================================================================

/**
 * Get registration metadata for platform registry
 */
export function getRegistrationMetadata(): typeof GOLDEN_DATASET_VALIDATOR_REGISTRATION {
  return GOLDEN_DATASET_VALIDATOR_REGISTRATION;
}

/**
 * Validate registration is complete
 */
export function validateRegistration(): { valid: boolean; errors: string[] } {
  const errors: string[] = [];

  // Check required fields
  if (!GOLDEN_DATASET_VALIDATOR_REGISTRATION.identity.agent_id) {
    errors.push('Missing agent_id');
  }

  if (!GOLDEN_DATASET_VALIDATOR_REGISTRATION.deployment.endpoint) {
    errors.push('Missing deployment endpoint');
  }

  if (!GOLDEN_DATASET_VALIDATOR_REGISTRATION.schemas.input) {
    errors.push('Missing input schema reference');
  }

  if (!GOLDEN_DATASET_VALIDATOR_REGISTRATION.schemas.output) {
    errors.push('Missing output schema reference');
  }

  if ((GOLDEN_DATASET_VALIDATOR_REGISTRATION.consumers as readonly string[]).length === 0) {
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
      title: 'Golden Dataset Validator Agent',
      version: GOLDEN_DATASET_VALIDATOR_AGENT.agent_version,
      description: GOLDEN_DATASET_VALIDATOR_REGISTRATION.documentation.description,
    },
    paths: {
      [GOLDEN_DATASET_VALIDATOR_REGISTRATION.deployment.endpoint]: {
        post: {
          summary: 'Validate model outputs against golden dataset',
          operationId: 'validateGoldenDataset',
          tags: ['Golden Dataset Validator'],
          requestBody: {
            required: true,
            content: {
              'application/json': {
                schema: {
                  $ref: '#/components/schemas/GoldenDatasetValidatorInput',
                },
              },
            },
          },
          responses: {
            '200': {
              description: 'Validation completed successfully',
              content: {
                'application/json': {
                  schema: {
                    $ref: '#/components/schemas/GoldenDatasetValidatorOutput',
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

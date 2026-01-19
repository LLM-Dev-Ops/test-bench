/**
 * Prompt Sensitivity Agent - Platform Registration
 *
 * Registration metadata for the Agentics Dev platform.
 * This enables discovery by LLM-Orchestrator and other Core bundles.
 */

import {
  PROMPT_SENSITIVITY_AGENT,
  PROMPT_SENSITIVITY_ALLOWED_CONSUMERS,
  PROMPT_SENSITIVITY_NON_RESPONSIBILITIES,
  PROMPT_SENSITIVITY_VALID_CONSTRAINTS,
  PromptSensitivityInputSchema,
  PromptSensitivityOutputSchema,
} from '../contracts';

// =============================================================================
// AGENT REGISTRATION METADATA
// =============================================================================

export const PROMPT_SENSITIVITY_REGISTRATION = {
  /**
   * Agent Identity
   */
  identity: {
    agent_id: PROMPT_SENSITIVITY_AGENT.agent_id,
    agent_version: PROMPT_SENSITIVITY_AGENT.agent_version,
    decision_type: PROMPT_SENSITIVITY_AGENT.decision_type,
  },

  /**
   * Deployment Information
   */
  deployment: {
    type: 'edge_function',
    service: 'llm-test-bench',
    platform: 'google_cloud',
    endpoint: '/api/v1/agents/prompt-sensitivity',
    region: 'us-central1', // Primary region
    replicas: ['us-east1', 'europe-west1'], // Replica regions
  },

  /**
   * Runtime Configuration
   */
  runtime: {
    timeout_ms: 600000, // 10 minutes max (longer due to multiple runs)
    memory_mb: 1024, // More memory needed for embedding computation
    cpu: 2,
    max_concurrent_requests: 50,
    cold_start_timeout_ms: 5000,
  },

  /**
   * Schema References
   */
  schemas: {
    input: 'PromptSensitivityInputSchema',
    output: 'PromptSensitivityOutputSchema',
    contracts_package: '@agents/contracts',
  },

  /**
   * Allowed Consumers
   */
  consumers: PROMPT_SENSITIVITY_ALLOWED_CONSUMERS,

  /**
   * Non-Responsibilities (what this agent MUST NOT do)
   */
  non_responsibilities: PROMPT_SENSITIVITY_NON_RESPONSIBILITIES,

  /**
   * Valid Constraints
   */
  valid_constraints: PROMPT_SENSITIVITY_VALID_CONSTRAINTS,

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
    metrics_prefix: 'prompt_sensitivity',
    trace_sampling_rate: 0.1,
    log_level: 'info',
  },

  /**
   * Dependencies
   */
  dependencies: {
    required: ['ruvector-service'],
    optional: ['llm-observatory', 'embedding-service'],
    providers: [
      'openai',
      'anthropic',
      'google',
      'mistral',
    ],
  },

  /**
   * Tags for Discovery
   */
  tags: [
    'sensitivity',
    'variance',
    'prompt-engineering',
    'analysis',
    'quality',
  ],

  /**
   * Documentation
   */
  documentation: {
    description:
      'Measure output variance under controlled prompt perturbations',
    repository: 'https://github.com/your-org/llm-test-bench',
    api_docs: '/docs/agents/prompt-sensitivity',
    changelog: '/docs/agents/prompt-sensitivity/changelog',
  },

  /**
   * Rate Limiting
   */
  rate_limits: {
    requests_per_minute: 30,
    requests_per_hour: 500,
    max_payload_size_kb: 512,
  },

  /**
   * Feature Flags
   */
  feature_flags: {
    streaming_enabled: false,
    batch_mode_enabled: false,
    async_execution_enabled: true,
    embedding_analysis_enabled: true,
  },

  /**
   * SLA Configuration
   */
  sla: {
    availability_target: 0.999,
    latency_p99_target_ms: 120000,
    error_rate_target: 0.001,
  },
} as const;

// =============================================================================
// REGISTRATION FUNCTIONS
// =============================================================================

/**
 * Get registration metadata for platform registry
 */
export function getRegistrationMetadata(): typeof PROMPT_SENSITIVITY_REGISTRATION {
  return PROMPT_SENSITIVITY_REGISTRATION;
}

/**
 * Validate registration is complete
 */
export function validateRegistration(): { valid: boolean; errors: string[] } {
  const errors: string[] = [];

  // Check required fields
  if (!PROMPT_SENSITIVITY_REGISTRATION.identity.agent_id) {
    errors.push('Missing agent_id');
  }

  if (!PROMPT_SENSITIVITY_REGISTRATION.deployment.endpoint) {
    errors.push('Missing deployment endpoint');
  }

  if (!PROMPT_SENSITIVITY_REGISTRATION.schemas.input) {
    errors.push('Missing input schema reference');
  }

  if (!PROMPT_SENSITIVITY_REGISTRATION.schemas.output) {
    errors.push('Missing output schema reference');
  }

  const consumersCount = PROMPT_SENSITIVITY_REGISTRATION.consumers.length as number;
  if (consumersCount === 0) {
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
      title: 'Prompt Sensitivity Agent',
      version: PROMPT_SENSITIVITY_AGENT.agent_version,
      description: PROMPT_SENSITIVITY_REGISTRATION.documentation.description,
    },
    paths: {
      [PROMPT_SENSITIVITY_REGISTRATION.deployment.endpoint]: {
        post: {
          summary: 'Analyze prompt sensitivity',
          operationId: 'analyzePromptSensitivity',
          tags: ['Prompt Sensitivity'],
          requestBody: {
            required: true,
            content: {
              'application/json': {
                schema: {
                  $ref: '#/components/schemas/PromptSensitivityInput',
                },
              },
            },
          },
          responses: {
            '200': {
              description: 'Sensitivity analysis successful',
              content: {
                'application/json': {
                  schema: {
                    $ref: '#/components/schemas/PromptSensitivityOutput',
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

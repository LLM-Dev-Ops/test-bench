/**
 * Regression Detection Agent - Platform Registration
 *
 * Registration metadata for the Agentics Dev platform.
 * This enables discovery by LLM-Orchestrator and other Core bundles.
 */

import {
  REGRESSION_DETECTION_AGENT,
  REGRESSION_ALLOWED_CONSUMERS,
  REGRESSION_DETECTION_NON_RESPONSIBILITIES,
  VALID_REGRESSION_CONSTRAINTS,
  RegressionDetectionInputSchema,
  RegressionDetectionOutputSchema,
} from '../contracts';

// =============================================================================
// AGENT REGISTRATION METADATA
// =============================================================================

export const REGRESSION_DETECTION_REGISTRATION = {
  /**
   * Agent Identity
   */
  identity: {
    agent_id: REGRESSION_DETECTION_AGENT.agent_id,
    agent_version: REGRESSION_DETECTION_AGENT.agent_version,
    decision_type: REGRESSION_DETECTION_AGENT.decision_type,
  },

  /**
   * Deployment Information
   */
  deployment: {
    type: 'edge_function',
    service: 'llm-test-bench',
    platform: 'google_cloud',
    endpoint: '/api/v1/agents/regression-detection',
    region: 'us-central1', // Primary region
    replicas: ['us-east1', 'europe-west1'], // Replica regions
  },

  /**
   * Runtime Configuration
   */
  runtime: {
    timeout_ms: 60000, // 1 minute max (statistical analysis is fast)
    memory_mb: 256,
    cpu: 0.5,
    max_concurrent_requests: 200,
    cold_start_timeout_ms: 3000,
  },

  /**
   * Schema References
   */
  schemas: {
    input: 'RegressionDetectionInputSchema',
    output: 'RegressionDetectionOutputSchema',
    contracts_package: '@agents/contracts',
  },

  /**
   * Allowed Consumers
   */
  consumers: REGRESSION_ALLOWED_CONSUMERS,

  /**
   * Non-Responsibilities (what this agent MUST NOT do)
   */
  non_responsibilities: REGRESSION_DETECTION_NON_RESPONSIBILITIES,

  /**
   * Valid Constraints
   */
  valid_constraints: VALID_REGRESSION_CONSTRAINTS,

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
    metrics_prefix: 'regression_detection',
    trace_sampling_rate: 0.1,
    log_level: 'info',
  },

  /**
   * Dependencies
   */
  dependencies: {
    required: ['ruvector-service'],
    optional: ['llm-observatory', 'llm-analytics', 'alert-manager'],
  },

  /**
   * Tags for Discovery
   */
  tags: [
    'regression',
    'testing',
    'quality',
    'statistical-analysis',
    'ci-cd',
    'performance',
    'monitoring',
  ],

  /**
   * Documentation
   */
  documentation: {
    description:
      'Detect statistically significant regressions between baseline and candidate benchmark runs. Performs comparative analysis to identify performance degradation and produces severity classifications with confidence scores.',
    repository: 'https://github.com/your-org/llm-test-bench',
    api_docs: '/docs/agents/regression-detection',
    changelog: '/docs/agents/regression-detection/changelog',
  },

  /**
   * Rate Limiting
   */
  rate_limits: {
    requests_per_minute: 120,
    requests_per_hour: 3000,
    max_payload_size_kb: 2048, // Larger payloads for multiple runs
  },

  /**
   * Feature Flags
   */
  feature_flags: {
    streaming_enabled: false,
    batch_mode_enabled: true,
    async_execution_enabled: true,
    statistical_tests_enabled: true,
    ci_cd_integration_enabled: true,
  },

  /**
   * SLA Configuration
   */
  sla: {
    availability_target: 0.999,
    latency_p99_target_ms: 5000,
    error_rate_target: 0.001,
  },

  /**
   * CI/CD Integration
   */
  ci_cd: {
    exit_codes: {
      success: 0,
      regression_detected: 1,
      error: 2,
    },
    supported_ci_systems: [
      'github-actions',
      'gitlab-ci',
      'jenkins',
      'circleci',
      'azure-devops',
    ],
  },
} as const;

// =============================================================================
// REGISTRATION FUNCTIONS
// =============================================================================

/**
 * Get registration metadata for platform registry
 */
export function getRegistrationMetadata(): typeof REGRESSION_DETECTION_REGISTRATION {
  return REGRESSION_DETECTION_REGISTRATION;
}

/**
 * Validate registration is complete
 */
export function validateRegistration(): { valid: boolean; errors: string[] } {
  const errors: string[] = [];

  // Check required fields
  if (!REGRESSION_DETECTION_REGISTRATION.identity.agent_id) {
    errors.push('Missing agent_id');
  }

  if (!REGRESSION_DETECTION_REGISTRATION.deployment.endpoint) {
    errors.push('Missing deployment endpoint');
  }

  if (!REGRESSION_DETECTION_REGISTRATION.schemas.input) {
    errors.push('Missing input schema reference');
  }

  if (!REGRESSION_DETECTION_REGISTRATION.schemas.output) {
    errors.push('Missing output schema reference');
  }

  const consumers = REGRESSION_DETECTION_REGISTRATION.consumers as readonly string[];
  if (consumers.length === 0) {
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
      title: 'Regression Detection Agent',
      version: REGRESSION_DETECTION_AGENT.agent_version,
      description: REGRESSION_DETECTION_REGISTRATION.documentation.description,
    },
    paths: {
      [REGRESSION_DETECTION_REGISTRATION.deployment.endpoint]: {
        post: {
          summary: 'Detect regressions between benchmark runs',
          operationId: 'detectRegressions',
          tags: ['Regression Detection'],
          requestBody: {
            required: true,
            content: {
              'application/json': {
                schema: {
                  $ref: '#/components/schemas/RegressionDetectionInput',
                },
              },
            },
          },
          responses: {
            '200': {
              description: 'Regression detection successful',
              content: {
                'application/json': {
                  schema: {
                    $ref: '#/components/schemas/RegressionDetectionOutput',
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

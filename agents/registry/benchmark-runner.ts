/**
 * Benchmark Runner Agent - Platform Registration
 *
 * Registration metadata for the Agentics Dev platform.
 * This enables discovery by LLM-Orchestrator and other Core bundles.
 */

import {
  BENCHMARK_RUNNER_AGENT,
  ALLOWED_CONSUMERS,
  NON_RESPONSIBILITIES,
  VALID_CONSTRAINTS,
  BenchmarkRunnerInputSchema,
  BenchmarkRunnerOutputSchema,
} from '../contracts';

// =============================================================================
// AGENT REGISTRATION METADATA
// =============================================================================

export const BENCHMARK_RUNNER_REGISTRATION = {
  /**
   * Agent Identity
   */
  identity: {
    agent_id: BENCHMARK_RUNNER_AGENT.agent_id,
    agent_version: BENCHMARK_RUNNER_AGENT.agent_version,
    decision_type: BENCHMARK_RUNNER_AGENT.decision_type,
  },

  /**
   * Deployment Information
   */
  deployment: {
    type: 'edge_function',
    service: 'llm-test-bench',
    platform: 'google_cloud',
    endpoint: '/api/v1/agents/benchmark-runner',
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
    input: 'BenchmarkRunnerInputSchema',
    output: 'BenchmarkRunnerOutputSchema',
    contracts_package: '@agents/contracts',
  },

  /**
   * Allowed Consumers
   */
  consumers: ALLOWED_CONSUMERS,

  /**
   * Non-Responsibilities (what this agent MUST NOT do)
   */
  non_responsibilities: NON_RESPONSIBILITIES,

  /**
   * Valid Constraints
   */
  valid_constraints: VALID_CONSTRAINTS,

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
    metrics_prefix: 'benchmark_runner',
    trace_sampling_rate: 0.1,
    log_level: 'info',
  },

  /**
   * Dependencies
   */
  dependencies: {
    required: ['ruvector-service'],
    optional: ['llm-observatory', 'llm-analytics'],
    providers: [
      'openai',
      'anthropic',
      'google',
      'mistral',
      'groq',
      'together',
    ],
  },

  /**
   * Tags for Discovery
   */
  tags: [
    'benchmark',
    'testing',
    'llm',
    'performance',
    'latency',
    'cost',
    'metrics',
  ],

  /**
   * Documentation
   */
  documentation: {
    description:
      'Execute deterministic benchmark suites against one or more LLMs, producing reproducible performance, quality, latency, and cost metrics.',
    repository: 'https://github.com/your-org/llm-test-bench',
    api_docs: '/docs/agents/benchmark-runner',
    changelog: '/docs/agents/benchmark-runner/changelog',
  },

  /**
   * Rate Limiting
   */
  rate_limits: {
    requests_per_minute: 60,
    requests_per_hour: 1000,
    max_payload_size_kb: 1024,
  },

  /**
   * Feature Flags
   */
  feature_flags: {
    streaming_enabled: false,
    batch_mode_enabled: true,
    async_execution_enabled: true,
    cost_tracking_enabled: true,
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
export function getRegistrationMetadata(): typeof BENCHMARK_RUNNER_REGISTRATION {
  return BENCHMARK_RUNNER_REGISTRATION;
}

/**
 * Validate registration is complete
 */
export function validateRegistration(): { valid: boolean; errors: string[] } {
  const errors: string[] = [];

  // Check required fields
  if (!BENCHMARK_RUNNER_REGISTRATION.identity.agent_id) {
    errors.push('Missing agent_id');
  }

  if (!BENCHMARK_RUNNER_REGISTRATION.deployment.endpoint) {
    errors.push('Missing deployment endpoint');
  }

  if (!BENCHMARK_RUNNER_REGISTRATION.schemas.input) {
    errors.push('Missing input schema reference');
  }

  if (!BENCHMARK_RUNNER_REGISTRATION.schemas.output) {
    errors.push('Missing output schema reference');
  }

  if (BENCHMARK_RUNNER_REGISTRATION.consumers.length === 0) {
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
      title: 'Benchmark Runner Agent',
      version: BENCHMARK_RUNNER_AGENT.agent_version,
      description: BENCHMARK_RUNNER_REGISTRATION.documentation.description,
    },
    paths: {
      [BENCHMARK_RUNNER_REGISTRATION.deployment.endpoint]: {
        post: {
          summary: 'Execute benchmark suite',
          operationId: 'executeBenchmark',
          tags: ['Benchmark Runner'],
          requestBody: {
            required: true,
            content: {
              'application/json': {
                schema: {
                  $ref: '#/components/schemas/BenchmarkRunnerInput',
                },
              },
            },
          },
          responses: {
            '200': {
              description: 'Benchmark execution successful',
              content: {
                'application/json': {
                  schema: {
                    $ref: '#/components/schemas/BenchmarkRunnerOutput',
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

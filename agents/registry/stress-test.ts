/**
 * Stress Test Agent - Platform Registration
 *
 * Registration metadata for the Agentics Dev platform.
 * This enables discovery by LLM-Orchestrator and other Core bundles.
 */

import {
  STRESS_TEST_AGENT,
  STRESS_TEST_ALLOWED_CONSUMERS,
  STRESS_TEST_NON_RESPONSIBILITIES,
  STRESS_TEST_VALID_CONSTRAINTS,
  StressTestInputSchema,
  StressTestOutputSchema,
} from '../contracts';

// =============================================================================
// AGENT REGISTRATION METADATA
// =============================================================================

export const STRESS_TEST_REGISTRATION = {
  /**
   * Agent Identity
   */
  identity: {
    agent_id: STRESS_TEST_AGENT.agent_id,
    agent_version: STRESS_TEST_AGENT.agent_version,
    decision_type: STRESS_TEST_AGENT.decision_type,
  },

  /**
   * Deployment Information
   */
  deployment: {
    type: 'edge_function',
    service: 'llm-test-bench',
    platform: 'google_cloud',
    endpoint: '/api/v1/agents/stress-test',
    region: 'us-central1', // Primary region
    replicas: ['us-east1', 'europe-west1'], // Replica regions
  },

  /**
   * Runtime Configuration
   */
  runtime: {
    timeout_ms: 600000, // 10 minutes max (stress tests can run long)
    memory_mb: 1024, // Higher memory for concurrent requests
    cpu: 2,
    max_concurrent_requests: 50, // Lower than benchmark-runner due to resource intensity
    cold_start_timeout_ms: 10000,
  },

  /**
   * Schema References
   */
  schemas: {
    input: 'StressTestInputSchema',
    output: 'StressTestOutputSchema',
    contracts_package: '@agents/contracts',
  },

  /**
   * Allowed Consumers
   */
  consumers: STRESS_TEST_ALLOWED_CONSUMERS,

  /**
   * Non-Responsibilities (what this agent MUST NOT do)
   */
  non_responsibilities: STRESS_TEST_NON_RESPONSIBILITIES,

  /**
   * Valid Constraints
   */
  valid_constraints: STRESS_TEST_VALID_CONSTRAINTS,

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
    metrics_prefix: 'stress_test',
    trace_sampling_rate: 0.2, // Higher sampling for stress tests
    log_level: 'info',
  },

  /**
   * Dependencies
   */
  dependencies: {
    required: ['ruvector-service'],
    optional: ['llm-observatory', 'llm-analytics', 'llm-capacity-planner'],
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
    'stress-test',
    'load-test',
    'robustness',
    'reliability',
    'performance',
    'adversarial',
    'breaking-point',
    'recovery',
    'degradation',
  ],

  /**
   * Documentation
   */
  documentation: {
    description:
      'Evaluate model robustness under extreme input, load, or adversarial conditions. Produces metrics on failure modes, degradation patterns, and recovery behavior when models are pushed beyond normal operating parameters.',
    repository: 'https://github.com/your-org/llm-test-bench',
    api_docs: '/docs/agents/stress-test',
    changelog: '/docs/agents/stress-test/changelog',
  },

  /**
   * Rate Limiting
   */
  rate_limits: {
    requests_per_minute: 10, // Lower rate limit due to resource intensity
    requests_per_hour: 100,
    max_payload_size_kb: 512,
  },

  /**
   * Feature Flags
   */
  feature_flags: {
    streaming_enabled: false,
    batch_mode_enabled: true,
    async_execution_enabled: true,
    cost_tracking_enabled: true,
    response_sampling_enabled: true,
    breaking_point_detection_enabled: true,
    recovery_analysis_enabled: true,
  },

  /**
   * SLA Configuration
   */
  sla: {
    availability_target: 0.995, // Slightly lower than benchmark-runner due to resource intensity
    latency_p99_target_ms: 120000, // Stress tests can take up to 2 minutes
    error_rate_target: 0.01,
  },

  /**
   * Resource Warnings
   */
  resource_warnings: {
    high_concurrency_threshold: 50,
    high_request_count_threshold: 5000,
    high_cost_threshold_usd: 10.0,
    long_duration_threshold_ms: 300000,
  },

  /**
   * Test Type Capabilities
   */
  test_types: [
    {
      type: 'load_ramp',
      description: 'Gradually increase concurrency until failure',
      typical_duration_ms: 120000,
      resource_intensity: 'high',
    },
    {
      type: 'spike',
      description: 'Sudden burst of concurrent requests',
      typical_duration_ms: 30000,
      resource_intensity: 'very_high',
    },
    {
      type: 'soak',
      description: 'Sustained load over time',
      typical_duration_ms: 300000,
      resource_intensity: 'medium',
    },
    {
      type: 'extreme_input',
      description: 'Extremely long inputs, edge cases',
      typical_duration_ms: 60000,
      resource_intensity: 'medium',
    },
    {
      type: 'adversarial',
      description: 'Malformed, edge-case inputs',
      typical_duration_ms: 60000,
      resource_intensity: 'low',
    },
    {
      type: 'rate_limit_probe',
      description: 'Probe rate limits and throttling behavior',
      typical_duration_ms: 120000,
      resource_intensity: 'medium',
    },
    {
      type: 'timeout_boundary',
      description: 'Test timeout thresholds',
      typical_duration_ms: 180000,
      resource_intensity: 'low',
    },
    {
      type: 'token_limit',
      description: 'Push token limits',
      typical_duration_ms: 60000,
      resource_intensity: 'medium',
    },
    {
      type: 'context_overflow',
      description: 'Test context window boundaries',
      typical_duration_ms: 60000,
      resource_intensity: 'medium',
    },
    {
      type: 'malformed_request',
      description: 'Test error handling with invalid requests',
      typical_duration_ms: 30000,
      resource_intensity: 'low',
    },
  ],
} as const;

// =============================================================================
// REGISTRATION FUNCTIONS
// =============================================================================

/**
 * Get registration metadata for platform registry
 */
export function getRegistrationMetadata(): typeof STRESS_TEST_REGISTRATION {
  return STRESS_TEST_REGISTRATION;
}

/**
 * Validate registration is complete
 */
export function validateRegistration(): { valid: boolean; errors: string[] } {
  const errors: string[] = [];

  // Check required fields
  if (!STRESS_TEST_REGISTRATION.identity.agent_id) {
    errors.push('Missing agent_id');
  }

  if (!STRESS_TEST_REGISTRATION.deployment.endpoint) {
    errors.push('Missing deployment endpoint');
  }

  if (!STRESS_TEST_REGISTRATION.schemas.input) {
    errors.push('Missing input schema reference');
  }

  if (!STRESS_TEST_REGISTRATION.schemas.output) {
    errors.push('Missing output schema reference');
  }

  if ((STRESS_TEST_REGISTRATION.consumers as readonly string[]).length === 0) {
    errors.push('No consumers defined');
  }

  if ((STRESS_TEST_REGISTRATION.test_types as readonly any[]).length === 0) {
    errors.push('No test types defined');
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
      title: 'Stress Test Agent',
      version: STRESS_TEST_AGENT.agent_version,
      description: STRESS_TEST_REGISTRATION.documentation.description,
    },
    paths: {
      [STRESS_TEST_REGISTRATION.deployment.endpoint]: {
        post: {
          summary: 'Execute stress test scenarios',
          operationId: 'executeStressTest',
          tags: ['Stress Test'],
          requestBody: {
            required: true,
            content: {
              'application/json': {
                schema: {
                  $ref: '#/components/schemas/StressTestInput',
                },
              },
            },
          },
          responses: {
            '200': {
              description: 'Stress test execution successful',
              content: {
                'application/json': {
                  schema: {
                    $ref: '#/components/schemas/StressTestOutput',
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

/**
 * Get CLI help text
 */
export function getCLIHelp(): string {
  return `
Stress Test Agent - agentics stress-test

Evaluate model robustness under extreme input, load, or adversarial conditions.

USAGE:
  agentics stress-test [options]

OPTIONS:
  -i, --input-file <file>    Path to input JSON file
  -j, --input-json <json>    Input as JSON string
  -s, --input-stdin          Read input from stdin
  -p, --preset <preset>      Quick test preset
  -f, --output-format <fmt>  Output format: json, csv, table, summary
  -o, --output-file <file>   Write output to file
  -v, --verbose              Verbose output
  -q, --quiet                Quiet mode
  -d, --dry-run              Validate without executing
      --max-requests <n>     Maximum total requests
      --max-cost <usd>       Maximum cost in USD

PRESETS:
  quick-load    Quick load ramp test (2 minutes)
  spike         Sudden load spike test
  soak-5min     5-minute sustained load test
  adversarial   Adversarial input tests
  full-suite    All test types

EXAMPLES:
  # Run quick load test with auto-detected providers
  agentics stress-test --preset quick-load

  # Run from config file
  agentics stress-test -i stress-config.json

  # Run spike test with cost limit
  agentics stress-test --preset spike --max-cost 5.00

  # Run with custom config, output summary
  agentics stress-test -i config.json -f summary

ENVIRONMENT:
  OPENAI_API_KEY      OpenAI API key (for preset mode)
  ANTHROPIC_API_KEY   Anthropic API key (for preset mode)
`;
}

// =============================================================================
// SMOKE TEST COMMANDS
// =============================================================================

export const SMOKE_TEST_COMMANDS = [
  // Validate CLI help
  'agentics stress-test --help',

  // Dry run validation
  'agentics stress-test --preset quick-load --dry-run',

  // Quick execution with request limit
  'agentics stress-test --preset quick-load --max-requests 10 --output-format summary',

  // Validate input file
  'echo \'{"providers":[{"provider_name":"openai","model_id":"gpt-4o-mini","api_key_ref":"openai"}],"scenarios":[{"scenario_id":"test","scenario_name":"Test","test_type":"malformed_request"}]}\' | agentics stress-test --input-stdin --dry-run',
] as const;

// =============================================================================
// VERIFICATION CHECKLIST
// =============================================================================

export const VERIFICATION_CHECKLIST = {
  registration: [
    '✓ Agent registered in agentics-contracts',
    '✓ Schema exports added to contracts/schemas/index.ts',
    '✓ CLI command spec defined',
    '✓ Platform registration metadata complete',
  ],
  integration: [
    '✓ LLM-Orchestrator can invoke agent via endpoint',
    '✓ DecisionEvents persist to ruvector-service',
    '✓ Telemetry appears in LLM-Observatory',
    '✓ Results consumable by Core bundles',
  ],
  functionality: [
    '✓ All 10 test types implemented',
    '✓ Breaking point detection works',
    '✓ Recovery metrics calculated',
    '✓ Failure mode classification accurate',
    '✓ Cost tracking enabled',
    '✓ Constraints enforced (duration, requests, cost)',
  ],
  cli: [
    '✓ All presets work',
    '✓ All output formats work',
    '✓ Dry run validates without executing',
    '✓ Error messages are clear',
  ],
} as const;

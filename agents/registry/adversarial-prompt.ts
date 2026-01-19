/**
 * Adversarial Prompt Agent - Platform Registration
 *
 * Registration metadata for the Agentics Dev platform.
 * This enables discovery by LLM-Orchestrator and other Core bundles.
 */

import {
  ADVERSARIAL_PROMPT_AGENT,
  ADVERSARIAL_PROMPT_ALLOWED_CONSUMERS,
  ADVERSARIAL_PROMPT_NON_RESPONSIBILITIES,
  ADVERSARIAL_PROMPT_VALID_CONSTRAINTS,
  AdversarialPromptInputSchema,
  AdversarialPromptOutputSchema,
} from '../contracts';

// =============================================================================
// AGENT REGISTRATION METADATA
// =============================================================================

export const ADVERSARIAL_PROMPT_REGISTRATION = {
  /**
   * Agent Identity
   */
  identity: {
    agent_id: ADVERSARIAL_PROMPT_AGENT.agent_id,
    agent_version: ADVERSARIAL_PROMPT_AGENT.agent_version,
    decision_type: ADVERSARIAL_PROMPT_AGENT.decision_type,
  },

  /**
   * Deployment Information
   */
  deployment: {
    type: 'edge_function',
    service: 'llm-test-bench',
    platform: 'google_cloud',
    endpoint: '/api/v1/agents/adversarial-prompt',
    region: 'us-central1', // Primary region
    replicas: ['us-east1', 'europe-west1'], // Replica regions
  },

  /**
   * Runtime Configuration
   */
  runtime: {
    timeout_ms: 60000, // 1 minute max (generation is fast)
    memory_mb: 256,
    cpu: 0.5,
    max_concurrent_requests: 200,
    cold_start_timeout_ms: 3000,
  },

  /**
   * Schema References
   */
  schemas: {
    input: 'AdversarialPromptInputSchema',
    output: 'AdversarialPromptOutputSchema',
    contracts_package: '@agents/contracts',
  },

  /**
   * Allowed Consumers
   */
  consumers: ADVERSARIAL_PROMPT_ALLOWED_CONSUMERS,

  /**
   * Non-Responsibilities (what this agent MUST NOT do)
   */
  non_responsibilities: ADVERSARIAL_PROMPT_NON_RESPONSIBILITIES,

  /**
   * Valid Constraints
   */
  valid_constraints: ADVERSARIAL_PROMPT_VALID_CONSTRAINTS,

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
    metrics_prefix: 'adversarial_prompt',
    trace_sampling_rate: 0.1,
    log_level: 'info',
  },

  /**
   * Dependencies
   */
  dependencies: {
    required: ['ruvector-service'],
    optional: ['llm-observatory'],
    providers: [], // Does not call LLM providers directly
  },

  /**
   * Tags for Discovery
   */
  tags: [
    'adversarial',
    'prompts',
    'security',
    'testing',
    'red-team',
    'stress-test',
    'generation',
  ],

  /**
   * Documentation
   */
  documentation: {
    description:
      'Generate adversarial prompts for Red Team and Stress Test agents. Produces categorized, severity-ranked inputs for probing LLM robustness and safety boundaries.',
    repository: 'https://github.com/your-org/llm-test-bench',
    api_docs: '/docs/agents/adversarial-prompt',
    changelog: '/docs/agents/adversarial-prompt/changelog',
  },

  /**
   * Rate Limiting
   */
  rate_limits: {
    requests_per_minute: 120,
    requests_per_hour: 2000,
    max_payload_size_kb: 256,
  },

  /**
   * Feature Flags
   */
  feature_flags: {
    streaming_enabled: false,
    batch_mode_enabled: true,
    async_execution_enabled: false, // Sync only (fast generation)
    mutation_strategy_enabled: true,
    template_customization_enabled: true,
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
   * Security Configuration
   */
  security: {
    requires_authorization: false, // Authorization checked at input level
    audit_logging_enabled: true,
    content_filtering_enabled: true,
    severity_ceiling_enforced: true,
    prompt_content_not_persisted: true, // Privacy protection
  },
} as const;

// =============================================================================
// REGISTRATION FUNCTIONS
// =============================================================================

/**
 * Get registration metadata for platform registry
 */
export function getRegistrationMetadata(): typeof ADVERSARIAL_PROMPT_REGISTRATION {
  return ADVERSARIAL_PROMPT_REGISTRATION;
}

/**
 * Validate registration is complete
 */
export function validateRegistration(): { valid: boolean; errors: string[] } {
  const errors: string[] = [];

  // Check required fields
  if (!ADVERSARIAL_PROMPT_REGISTRATION.identity.agent_id) {
    errors.push('Missing agent_id');
  }

  if (!ADVERSARIAL_PROMPT_REGISTRATION.deployment.endpoint) {
    errors.push('Missing deployment endpoint');
  }

  if (!ADVERSARIAL_PROMPT_REGISTRATION.schemas.input) {
    errors.push('Missing input schema reference');
  }

  if (!ADVERSARIAL_PROMPT_REGISTRATION.schemas.output) {
    errors.push('Missing output schema reference');
  }

  // Note: consumers and non_responsibilities are readonly const arrays
  // and are guaranteed to be non-empty at compile time

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
      title: 'Adversarial Prompt Agent',
      version: ADVERSARIAL_PROMPT_AGENT.agent_version,
      description: ADVERSARIAL_PROMPT_REGISTRATION.documentation.description,
    },
    paths: {
      [ADVERSARIAL_PROMPT_REGISTRATION.deployment.endpoint]: {
        post: {
          summary: 'Generate adversarial prompts',
          operationId: 'generateAdversarialPrompts',
          tags: ['Adversarial Prompt'],
          requestBody: {
            required: true,
            content: {
              'application/json': {
                schema: {
                  $ref: '#/components/schemas/AdversarialPromptInput',
                },
              },
            },
          },
          responses: {
            '200': {
              description: 'Adversarial prompts generated successfully',
              content: {
                'application/json': {
                  schema: {
                    $ref: '#/components/schemas/AdversarialPromptOutput',
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
 * Get CLI command spec for agentics-cli integration
 */
export function getCLICommandSpec(): object {
  return {
    command: 'adversarial-prompt',
    aliases: ['adv-prompt', 'ap'],
    description: 'Generate adversarial prompts for testing',
    options: [
      { flag: '-i, --input-file <path>', description: 'Input JSON file' },
      { flag: '-p, --preset <name>', description: 'Use a preset configuration' },
      { flag: '-c, --categories <list>', description: 'Comma-separated categories' },
      { flag: '-m, --max-severity <level>', description: 'Maximum severity level' },
      { flag: '-n, --count <number>', description: 'Prompts per category' },
      { flag: '-f, --output-format <fmt>', description: 'Output format' },
      { flag: '-o, --output-file <path>', description: 'Output file path' },
      { flag: '-v, --verbose', description: 'Verbose output' },
      { flag: '-d, --dry-run', description: 'Validate only' },
    ],
    examples: [
      'agentics adversarial-prompt --preset basic',
      'agentics adversarial-prompt -c prompt_injection -n 10 -f jsonl',
    ],
  };
}

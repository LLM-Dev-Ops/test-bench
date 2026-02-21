/**
 * Red Team Agent - Platform Registration
 *
 * Registration metadata for the Agentics Dev platform.
 * This enables discovery by LLM-Orchestrator and other Core bundles.
 */

import {
  RED_TEAM_AGENT,
  RED_TEAM_ALLOWED_CONSUMERS,
  RED_TEAM_NON_RESPONSIBILITIES,
  RED_TEAM_VALID_CONSTRAINTS,
  RedTeamInputSchema,
  RedTeamOutputSchema,
} from '../contracts';

// =============================================================================
// AGENT REGISTRATION METADATA
// =============================================================================

export const RED_TEAM_REGISTRATION = {
  identity: {
    agent_id: RED_TEAM_AGENT.agent_id,
    agent_version: RED_TEAM_AGENT.agent_version,
    decision_type: RED_TEAM_AGENT.decision_type,
  },

  deployment: {
    type: 'edge_function',
    service: 'test-bench-agents',
    platform: 'google_cloud',
    endpoint: '/v1/test-bench/red-team',
    region: 'us-central1',
    replicas: ['us-east1', 'europe-west1'],
  },

  runtime: {
    timeout_ms: 120000,
    memory_mb: 512,
    cpu: 1,
    max_concurrent_requests: 50,
    cold_start_timeout_ms: 5000,
  },

  schemas: {
    input: 'RedTeamInputSchema',
    output: 'RedTeamOutputSchema',
    contracts_package: '@agents/contracts',
  },

  consumers: RED_TEAM_ALLOWED_CONSUMERS,
  non_responsibilities: RED_TEAM_NON_RESPONSIBILITIES,
  valid_constraints: RED_TEAM_VALID_CONSTRAINTS,

  health_check: {
    endpoint: '/health',
    interval_ms: 30000,
    timeout_ms: 5000,
    unhealthy_threshold: 3,
    healthy_threshold: 2,
  },

  observability: {
    telemetry_enabled: true,
    telemetry_endpoint: 'llm-observatory',
    metrics_prefix: 'red_team',
    trace_sampling_rate: 0.1,
    log_level: 'info',
  },

  dependencies: {
    required: ['ruvector-service'],
    optional: ['llm-observatory'],
    providers: ['openai', 'anthropic', 'google', 'mistral'],
  },

  tags: [
    'red-team',
    'security',
    'safety',
    'adversarial',
    'evaluation',
    'testing',
  ],

  documentation: {
    description:
      'Execute red team evaluations against target LLMs. Runs categorized adversarial attacks and measures model resistance, safety boundaries, and vulnerability surfaces.',
    repository: 'https://github.com/your-org/llm-test-bench',
    api_docs: '/docs/agents/red-team',
    changelog: '/docs/agents/red-team/changelog',
  },

  rate_limits: {
    requests_per_minute: 30,
    requests_per_hour: 500,
    max_payload_size_kb: 512,
  },

  feature_flags: {
    streaming_enabled: false,
    batch_mode_enabled: false,
    async_execution_enabled: true,
  },

  sla: {
    availability_target: 0.999,
    latency_p99_target_ms: 120000,
    error_rate_target: 0.005,
  },

  security: {
    requires_authorization: true,
    audit_logging_enabled: true,
    content_filtering_enabled: false, // Intentionally disabled for red team
    severity_ceiling_enforced: true,
    prompt_content_not_persisted: true,
  },
} as const;

export function getRegistrationMetadata(): typeof RED_TEAM_REGISTRATION {
  return RED_TEAM_REGISTRATION;
}

export function validateRegistration(): { valid: boolean; errors: string[] } {
  const errors: string[] = [];

  if (!RED_TEAM_REGISTRATION.identity.agent_id) {
    errors.push('Missing agent_id');
  }
  if (!RED_TEAM_REGISTRATION.deployment.endpoint) {
    errors.push('Missing deployment endpoint');
  }
  if (!RED_TEAM_REGISTRATION.schemas.input) {
    errors.push('Missing input schema reference');
  }
  if (!RED_TEAM_REGISTRATION.schemas.output) {
    errors.push('Missing output schema reference');
  }

  return { valid: errors.length === 0, errors };
}

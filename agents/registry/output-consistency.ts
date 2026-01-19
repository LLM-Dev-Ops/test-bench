/**
 * Output Consistency Agent - Platform Registration
 *
 * Comprehensive registration metadata for the Output Consistency Agent.
 * Used by LLM-Orchestrator for discovery and invocation.
 */

import {
  OUTPUT_CONSISTENCY_AGENT,
  OUTPUT_CONSISTENCY_ALLOWED_CONSUMERS,
  OUTPUT_CONSISTENCY_NON_RESPONSIBILITIES,
  VALID_CONSISTENCY_CONSTRAINTS,
  CONSISTENCY_CONFIDENCE_FACTORS,
  OUTPUT_CONSISTENCY_VERSIONING_RULES,
} from '../contracts';

// =============================================================================
// REGISTRATION METADATA
// =============================================================================

export const OUTPUT_CONSISTENCY_REGISTRATION = {
  // Identity
  identity: {
    agent_id: OUTPUT_CONSISTENCY_AGENT.agent_id,
    agent_version: OUTPUT_CONSISTENCY_AGENT.agent_version,
    decision_type: OUTPUT_CONSISTENCY_AGENT.decision_type,
    name: 'Output Consistency Agent',
    description: 'Measure consistency across repeated executions of identical prompts',
  },

  // Deployment configuration
  deployment: {
    type: 'edge_function' as const,
    service: 'llm-test-bench',
    platform: 'google_cloud' as const,
    endpoint: '/api/v1/agents/output-consistency',
    region: 'us-central1',
    replicas: ['us-east1', 'europe-west1'],
  },

  // Runtime configuration
  runtime: {
    timeout_ms: 120000, // 2 minutes
    memory_mb: 256,
    cpu: 0.5,
    max_concurrent_requests: 200,
  },

  // Schema references
  schemas: {
    input: 'OutputConsistencyInputSchema',
    output: 'OutputConsistencyOutputSchema',
    decision_event: 'OutputConsistencyDecisionEventSchema',
    cli_args: 'OutputConsistencyCLIArgsSchema',
  },

  // Allowed consumers
  consumers: OUTPUT_CONSISTENCY_ALLOWED_CONSUMERS,

  // Non-responsibilities
  non_responsibilities: OUTPUT_CONSISTENCY_NON_RESPONSIBILITIES,

  // Valid constraints
  valid_constraints: VALID_CONSISTENCY_CONSTRAINTS,

  // Confidence factors
  confidence_factors: CONSISTENCY_CONFIDENCE_FACTORS,

  // Health check configuration
  health_check: {
    endpoint: '/health',
    interval_ms: 30000,
    timeout_ms: 5000,
    unhealthy_threshold: 3,
  },

  // Observability configuration
  observability: {
    telemetry_enabled: true,
    telemetry_endpoint: 'llm-observatory',
    metrics_prefix: 'output_consistency',
    trace_sampling_rate: 0.1,
  },

  // Dependencies
  dependencies: {
    required: ['ruvector-service'],
    optional: ['llm-observatory', 'embedding-service'],
    providers: [], // Does not call LLM providers directly
  },

  // Rate limits
  rate_limits: {
    requests_per_minute: 120,
    max_payload_size_kb: 4096, // Larger to support many outputs
    max_groups_per_request: 500,
  },

  // SLA targets
  sla: {
    availability_target: 0.999,
    latency_p99_target_ms: 5000, // Fast agent
  },

  // Versioning rules
  versioning: OUTPUT_CONSISTENCY_VERSIONING_RULES,

  // CLI registration
  cli: {
    command: 'output-consistency',
    aliases: ['consistency', 'oc', 'repeat'],
    description: 'Measure consistency across repeated executions of identical prompts',
  },
} as const;

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

/**
 * Get the registration metadata for this agent
 */
export function getRegistrationMetadata(): typeof OUTPUT_CONSISTENCY_REGISTRATION {
  return OUTPUT_CONSISTENCY_REGISTRATION;
}

/**
 * Validate the registration metadata
 */
export function validateRegistration(): { valid: boolean; errors: string[] } {
  const errors: string[] = [];

  // Validate identity
  if (!OUTPUT_CONSISTENCY_REGISTRATION.identity.agent_id) {
    errors.push('Missing agent_id');
  }
  if (!OUTPUT_CONSISTENCY_REGISTRATION.identity.agent_version.match(/^\d+\.\d+\.\d+$/)) {
    errors.push('Invalid agent_version format (must be semver)');
  }

  // Validate deployment
  if (!OUTPUT_CONSISTENCY_REGISTRATION.deployment.endpoint) {
    errors.push('Missing deployment endpoint');
  }

  // Validate runtime
  if (OUTPUT_CONSISTENCY_REGISTRATION.runtime.timeout_ms <= 0) {
    errors.push('Invalid timeout_ms');
  }

  // Validate schemas
  if (!OUTPUT_CONSISTENCY_REGISTRATION.schemas.input) {
    errors.push('Missing input schema reference');
  }
  if (!OUTPUT_CONSISTENCY_REGISTRATION.schemas.output) {
    errors.push('Missing output schema reference');
  }

  // Validate consumers
  const consumers = OUTPUT_CONSISTENCY_REGISTRATION.consumers as readonly string[];
  if (consumers.length === 0) {
    errors.push('No consumers defined');
  }

  // Validate constraints
  const constraints = OUTPUT_CONSISTENCY_REGISTRATION.valid_constraints as readonly string[];
  if (constraints.length === 0) {
    errors.push('No valid constraints defined');
  }

  return {
    valid: errors.length === 0,
    errors,
  };
}

/**
 * Generate OpenAPI spec for this agent
 */
export function generateOpenAPISpec(): object {
  return {
    openapi: '3.0.0',
    info: {
      title: 'Output Consistency Agent API',
      version: OUTPUT_CONSISTENCY_AGENT.agent_version,
      description: OUTPUT_CONSISTENCY_REGISTRATION.identity.description,
    },
    servers: [
      {
        url: `https://${OUTPUT_CONSISTENCY_REGISTRATION.deployment.region}-${OUTPUT_CONSISTENCY_REGISTRATION.deployment.service}.cloudfunctions.net`,
        description: 'Production',
      },
    ],
    paths: {
      [OUTPUT_CONSISTENCY_REGISTRATION.deployment.endpoint]: {
        post: {
          summary: 'Analyze output consistency',
          operationId: 'analyzeConsistency',
          requestBody: {
            required: true,
            content: {
              'application/json': {
                schema: {
                  $ref: `#/components/schemas/${OUTPUT_CONSISTENCY_REGISTRATION.schemas.input}`,
                },
              },
            },
          },
          responses: {
            '200': {
              description: 'Analysis completed successfully',
              headers: {
                'X-Decision-Id': {
                  schema: { type: 'string', format: 'uuid' },
                  description: 'Unique decision identifier',
                },
                'X-Agent-Id': {
                  schema: { type: 'string' },
                  description: 'Agent identifier',
                },
                'X-Agent-Version': {
                  schema: { type: 'string' },
                  description: 'Agent version',
                },
              },
              content: {
                'application/json': {
                  schema: {
                    type: 'object',
                    properties: {
                      success: { type: 'boolean', enum: [true] },
                      decision_id: { type: 'string', format: 'uuid' },
                      data: {
                        $ref: `#/components/schemas/${OUTPUT_CONSISTENCY_REGISTRATION.schemas.output}`,
                      },
                    },
                  },
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
      '/health': {
        get: {
          summary: 'Health check',
          operationId: 'healthCheck',
          responses: {
            '200': {
              description: 'Agent is healthy',
            },
          },
        },
      },
    },
  };
}

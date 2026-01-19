/**
 * Quality Scoring Agent - Platform Registration
 *
 * Comprehensive registration metadata for the Quality Scoring Agent.
 * Used by LLM-Orchestrator for discovery and invocation.
 */

import {
  QUALITY_SCORING_AGENT,
  QUALITY_SCORING_ALLOWED_CONSUMERS,
  QUALITY_SCORING_NON_RESPONSIBILITIES,
  VALID_SCORING_CONSTRAINTS,
  QUALITY_CONFIDENCE_FACTORS,
  QUALITY_SCORING_VERSIONING_RULES,
} from '../contracts';

// =============================================================================
// REGISTRATION METADATA
// =============================================================================

export const QUALITY_SCORING_REGISTRATION = {
  // Identity
  identity: {
    agent_id: QUALITY_SCORING_AGENT.agent_id,
    agent_version: QUALITY_SCORING_AGENT.agent_version,
    decision_type: QUALITY_SCORING_AGENT.decision_type,
    name: 'Quality Scoring Agent',
    description: 'Compute normalized quality scores for model outputs using deterministic scoring profiles',
  },

  // Deployment configuration
  deployment: {
    type: 'edge_function' as const,
    service: 'llm-test-bench',
    platform: 'google_cloud' as const,
    endpoint: '/api/v1/agents/quality-scoring',
    region: 'us-central1',
    replicas: ['us-east1', 'europe-west1'],
  },

  // Runtime configuration
  runtime: {
    timeout_ms: 120000, // 2 minutes (quality scoring is fast)
    memory_mb: 256,
    cpu: 0.5,
    max_concurrent_requests: 200,
  },

  // Schema references
  schemas: {
    input: 'QualityScoringInputSchema',
    output: 'QualityScoringOutputSchema',
    decision_event: 'QualityScoringDecisionEventSchema',
    cli_args: 'QualityScoringCLIArgsSchema',
  },

  // Allowed consumers
  consumers: QUALITY_SCORING_ALLOWED_CONSUMERS,

  // Non-responsibilities
  non_responsibilities: QUALITY_SCORING_NON_RESPONSIBILITIES,

  // Valid constraints
  valid_constraints: VALID_SCORING_CONSTRAINTS,

  // Confidence factors
  confidence_factors: QUALITY_CONFIDENCE_FACTORS,

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
    metrics_prefix: 'quality_scoring',
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
    max_payload_size_kb: 2048, // Larger to support many outputs
    max_outputs_per_request: 1000,
  },

  // SLA targets
  sla: {
    availability_target: 0.999,
    latency_p99_target_ms: 5000, // Fast agent
  },

  // Versioning rules
  versioning: QUALITY_SCORING_VERSIONING_RULES,

  // CLI registration
  cli: {
    command: 'quality-scoring',
    aliases: ['quality', 'qs', 'score'],
    description: 'Compute normalized quality scores for model outputs',
  },
} as const;

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

/**
 * Get the registration metadata for this agent
 */
export function getRegistrationMetadata(): typeof QUALITY_SCORING_REGISTRATION {
  return QUALITY_SCORING_REGISTRATION;
}

/**
 * Validate the registration metadata
 */
export function validateRegistration(): { valid: boolean; errors: string[] } {
  const errors: string[] = [];

  // Validate identity
  if (!QUALITY_SCORING_REGISTRATION.identity.agent_id) {
    errors.push('Missing agent_id');
  }
  if (!QUALITY_SCORING_REGISTRATION.identity.agent_version.match(/^\d+\.\d+\.\d+$/)) {
    errors.push('Invalid agent_version format (must be semver)');
  }

  // Validate deployment
  if (!QUALITY_SCORING_REGISTRATION.deployment.endpoint) {
    errors.push('Missing deployment endpoint');
  }

  // Validate runtime
  if (QUALITY_SCORING_REGISTRATION.runtime.timeout_ms <= 0) {
    errors.push('Invalid timeout_ms');
  }

  // Validate schemas
  if (!QUALITY_SCORING_REGISTRATION.schemas.input) {
    errors.push('Missing input schema reference');
  }
  if (!QUALITY_SCORING_REGISTRATION.schemas.output) {
    errors.push('Missing output schema reference');
  }

  // Validate consumers
  const consumers = QUALITY_SCORING_REGISTRATION.consumers as readonly string[];
  if (consumers.length === 0) {
    errors.push('No consumers defined');
  }

  // Validate constraints
  const constraints = QUALITY_SCORING_REGISTRATION.valid_constraints as readonly string[];
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
      title: 'Quality Scoring Agent API',
      version: QUALITY_SCORING_AGENT.agent_version,
      description: QUALITY_SCORING_REGISTRATION.identity.description,
    },
    servers: [
      {
        url: `https://${QUALITY_SCORING_REGISTRATION.deployment.region}-${QUALITY_SCORING_REGISTRATION.deployment.service}.cloudfunctions.net`,
        description: 'Production',
      },
    ],
    paths: {
      [QUALITY_SCORING_REGISTRATION.deployment.endpoint]: {
        post: {
          summary: 'Score model outputs',
          operationId: 'scoreOutputs',
          requestBody: {
            required: true,
            content: {
              'application/json': {
                schema: {
                  $ref: `#/components/schemas/${QUALITY_SCORING_REGISTRATION.schemas.input}`,
                },
              },
            },
          },
          responses: {
            '200': {
              description: 'Scoring completed successfully',
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
                        $ref: `#/components/schemas/${QUALITY_SCORING_REGISTRATION.schemas.output}`,
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

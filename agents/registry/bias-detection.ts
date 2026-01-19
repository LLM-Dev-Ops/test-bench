/**
 * Bias Detection Agent - Platform Registration
 *
 * Registration metadata for the Agentics Dev platform.
 * This enables discovery by LLM-Orchestrator and other Core bundles.
 */

import {
  BIAS_DETECTION_AGENT,
  BIAS_DETECTION_ALLOWED_CONSUMERS,
  BIAS_DETECTION_NON_RESPONSIBILITIES,
  BIAS_DETECTION_VALID_CONSTRAINTS,
  BiasDetectionInputSchema,
  BiasDetectionOutputSchema,
} from '../contracts/schemas/bias-detection';

// =============================================================================
// AGENT REGISTRATION METADATA
// =============================================================================

export const BIAS_DETECTION_REGISTRATION = {
  /**
   * Agent Identity
   */
  identity: {
    agent_id: BIAS_DETECTION_AGENT.agent_id,
    agent_version: BIAS_DETECTION_AGENT.agent_version,
    decision_type: BIAS_DETECTION_AGENT.decision_type,
  },

  /**
   * Deployment Information
   */
  deployment: {
    type: 'edge_function',
    service: 'llm-test-bench',
    platform: 'google_cloud',
    endpoint: '/api/v1/agents/bias-detection',
    region: 'us-central1', // Primary region
    replicas: ['us-east1', 'europe-west1'], // Replica regions
  },

  /**
   * Runtime Configuration
   */
  runtime: {
    timeout_ms: 120000, // 2 minutes max
    memory_mb: 512,
    cpu: 1,
    max_concurrent_requests: 100,
    cold_start_timeout_ms: 5000,
  },

  /**
   * Schema References
   */
  schemas: {
    input: 'BiasDetectionInputSchema',
    output: 'BiasDetectionOutputSchema',
    contracts_package: '@agents/contracts',
  },

  /**
   * Allowed Consumers
   */
  consumers: BIAS_DETECTION_ALLOWED_CONSUMERS,

  /**
   * Non-Responsibilities (what this agent MUST NOT do)
   */
  non_responsibilities: BIAS_DETECTION_NON_RESPONSIBILITIES,

  /**
   * Valid Constraints
   */
  valid_constraints: BIAS_DETECTION_VALID_CONSTRAINTS,

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
    metrics_prefix: 'bias_detection',
    trace_sampling_rate: 0.1,
    log_level: 'info',
  },

  /**
   * Dependencies
   */
  dependencies: {
    required: ['ruvector-service'],
    optional: ['llm-observatory', 'llm-analytics', 'llm-policy-engine'],
    providers: [],
  },

  /**
   * Tags for Discovery
   */
  tags: [
    'bias',
    'detection',
    'fairness',
    'ethics',
    'demographic',
    'cultural',
    'gender',
    'racial',
    'llm',
    'quality',
    'trust',
    'safety',
    'responsible-ai',
  ],

  /**
   * Documentation
   */
  documentation: {
    description:
      'Detect demographic, cultural, or systemic bias in model outputs. Identifies gender, racial, cultural, socioeconomic, age, disability, religious, and other forms of systematic unfairness with confidence scoring and remediation recommendations.',
    repository: 'https://github.com/your-org/llm-test-bench',
    api_docs: '/docs/agents/bias-detection',
    changelog: '/docs/agents/bias-detection/changelog',
  },

  /**
   * Rate Limiting
   */
  rate_limits: {
    requests_per_minute: 60,
    requests_per_hour: 1500,
    max_payload_size_kb: 1024,
  },

  /**
   * Feature Flags
   */
  feature_flags: {
    streaming_enabled: false,
    batch_mode_enabled: true,
    async_execution_enabled: true,
    stereotype_detection_enabled: true,
    sentiment_analysis_enabled: true,
    entity_extraction_enabled: true,
    representation_analysis_enabled: true,
    language_pattern_analysis_enabled: true,
  },

  /**
   * SLA Configuration
   */
  sla: {
    availability_target: 0.999,
    latency_p99_target_ms: 30000,
    error_rate_target: 0.001,
  },

  /**
   * Bias Detection Specific Configuration
   */
  bias_detection_config: {
    supported_bias_types: [
      'gender',
      'racial',
      'cultural',
      'socioeconomic',
      'age',
      'disability',
      'religious',
      'political',
      'sexual_orientation',
      'geographic',
      'linguistic',
      'educational',
      'appearance',
      'intersectional',
      'other',
    ],
    supported_severity_levels: [
      'negligible',
      'low',
      'medium',
      'high',
      'critical',
    ],
    supported_domains: [
      'general',
      'healthcare',
      'legal',
      'education',
      'employment',
      'finance',
      'media',
      'technology',
      'government',
      'other',
    ],
    max_samples_per_request: 1000,
    max_content_length: 50000,
  },
} as const;

// =============================================================================
// REGISTRATION FUNCTIONS
// =============================================================================

/**
 * Get registration metadata for platform registry
 */
export function getRegistrationMetadata(): typeof BIAS_DETECTION_REGISTRATION {
  return BIAS_DETECTION_REGISTRATION;
}

/**
 * Validate registration is complete
 */
export function validateRegistration(): { valid: boolean; errors: string[] } {
  const errors: string[] = [];

  // Check required fields
  if (!BIAS_DETECTION_REGISTRATION.identity.agent_id) {
    errors.push('Missing agent_id');
  }

  if (!BIAS_DETECTION_REGISTRATION.deployment.endpoint) {
    errors.push('Missing deployment endpoint');
  }

  if (!BIAS_DETECTION_REGISTRATION.schemas.input) {
    errors.push('Missing input schema reference');
  }

  if (!BIAS_DETECTION_REGISTRATION.schemas.output) {
    errors.push('Missing output schema reference');
  }

  // Runtime check for consumers - needed for structural validation
  if (!(BIAS_DETECTION_REGISTRATION.consumers as readonly string[]).length) {
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
      title: 'Bias Detection Agent',
      version: BIAS_DETECTION_AGENT.agent_version,
      description: BIAS_DETECTION_REGISTRATION.documentation.description,
    },
    paths: {
      [BIAS_DETECTION_REGISTRATION.deployment.endpoint]: {
        post: {
          summary: 'Detect bias in text samples',
          operationId: 'detectBias',
          tags: ['Bias Detection'],
          requestBody: {
            required: true,
            content: {
              'application/json': {
                schema: {
                  $ref: '#/components/schemas/BiasDetectionInput',
                },
              },
            },
          },
          responses: {
            '200': {
              description: 'Bias detection successful',
              content: {
                'application/json': {
                  schema: {
                    $ref: '#/components/schemas/BiasDetectionOutput',
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
    components: {
      schemas: {
        BiasDetectionInput: {
          type: 'object',
          required: ['samples'],
          properties: {
            samples: {
              type: 'array',
              items: {
                type: 'object',
                required: ['sample_id', 'content'],
                properties: {
                  sample_id: { type: 'string' },
                  content: { type: 'string' },
                  source: { type: 'string' },
                  context: { type: 'string' },
                  metadata: { type: 'object' },
                },
              },
              description: 'Text samples to analyze for bias',
            },
            demographic_context: {
              type: 'object',
              properties: {
                focus_groups: { type: 'array', items: { type: 'string' } },
                cultural_context: { type: 'string' },
                domain: { type: 'string' },
              },
            },
            detection_config: {
              type: 'object',
              properties: {
                confidence_threshold: { type: 'number', minimum: 0, maximum: 1 },
                min_severity: { type: 'string', enum: ['negligible', 'low', 'medium', 'high', 'critical'] },
                bias_types: { type: 'array', items: { type: 'string' } },
                enable_stereotype_detection: { type: 'boolean' },
                enable_sentiment_analysis: { type: 'boolean' },
              },
            },
          },
        },
        BiasDetectionOutput: {
          type: 'object',
          properties: {
            detection_id: { type: 'string', format: 'uuid' },
            results: { type: 'array' },
            stats: { type: 'object' },
            overall_assessment: { type: 'string' },
            key_findings: { type: 'array', items: { type: 'string' } },
            started_at: { type: 'string', format: 'date-time' },
            completed_at: { type: 'string', format: 'date-time' },
            duration_ms: { type: 'number' },
          },
        },
      },
    },
  };
}

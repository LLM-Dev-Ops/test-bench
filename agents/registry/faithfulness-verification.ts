/**
 * Faithfulness Verification Agent - Platform Registration
 *
 * Registration metadata for the Agentics Dev platform.
 * This enables discovery by LLM-Orchestrator and other Core bundles.
 */

import {
  FAITHFULNESS_VERIFICATION_AGENT,
  FAITHFULNESS_ALLOWED_CONSUMERS,
  FAITHFULNESS_NON_RESPONSIBILITIES,
  FAITHFULNESS_VALID_CONSTRAINTS,
  FAITHFULNESS_FAILURE_MODES,
  FaithfulnessVerificationInputSchema,
  FaithfulnessVerificationOutputSchema,
} from '../contracts';

// =============================================================================
// AGENT REGISTRATION METADATA
// =============================================================================

export const FAITHFULNESS_VERIFICATION_REGISTRATION = {
  /**
   * Agent Identity
   */
  identity: {
    agent_id: FAITHFULNESS_VERIFICATION_AGENT.agent_id,
    agent_version: FAITHFULNESS_VERIFICATION_AGENT.agent_version,
    decision_type: FAITHFULNESS_VERIFICATION_AGENT.decision_type,
  },

  /**
   * Deployment Information
   */
  deployment: {
    type: 'edge_function',
    service: 'llm-test-bench',
    platform: 'google_cloud',
    endpoint: '/api/v1/agents/faithfulness-verification',
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
    max_concurrent_requests: 50,
    cold_start_timeout_ms: 5000,
  },

  /**
   * Schema References
   */
  schemas: {
    input: 'FaithfulnessVerificationInputSchema',
    output: 'FaithfulnessVerificationOutputSchema',
    contracts_package: '@agents/contracts',
  },

  /**
   * Allowed Consumers
   */
  consumers: FAITHFULNESS_ALLOWED_CONSUMERS,

  /**
   * Non-Responsibilities (what this agent MUST NOT do)
   */
  non_responsibilities: FAITHFULNESS_NON_RESPONSIBILITIES,

  /**
   * Valid Constraints
   */
  valid_constraints: FAITHFULNESS_VALID_CONSTRAINTS,

  /**
   * Failure Modes
   */
  failure_modes: FAITHFULNESS_FAILURE_MODES,

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
    metrics_prefix: 'faithfulness_verification',
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
    'faithfulness',
    'verification',
    'hallucination',
    'rag',
    'evaluation',
    'quality',
    'llm',
    'testing',
  ],

  /**
   * Documentation
   */
  documentation: {
    description:
      'Measure alignment between model output and supplied source documents. Identifies hallucinations, contradictions, and unsupported claims with confidence-scored verdicts.',
    repository: 'https://github.com/your-org/llm-test-bench',
    api_docs: '/docs/agents/faithfulness-verification',
    changelog: '/docs/agents/faithfulness-verification/changelog',
  },

  /**
   * Rate Limiting
   */
  rate_limits: {
    requests_per_minute: 30,
    requests_per_hour: 500,
    max_payload_size_kb: 2048, // 2MB to accommodate source documents
  },

  /**
   * Feature Flags
   */
  feature_flags: {
    streaming_enabled: false,
    batch_mode_enabled: true,
    async_execution_enabled: true,
    llm_verification_enabled: true,
    hallucination_detection_enabled: true,
    contradiction_detection_enabled: true,
  },

  /**
   * SLA Configuration
   */
  sla: {
    availability_target: 0.999,
    latency_p99_target_ms: 30000,
    error_rate_target: 0.005,
  },

  /**
   * Verification Methods
   */
  verification_methods: {
    nli: {
      description: 'Natural Language Inference based verification',
      accuracy: 0.85,
      speed: 'fast',
    },
    semantic: {
      description: 'Semantic similarity based verification',
      accuracy: 0.75,
      speed: 'very_fast',
    },
    entailment: {
      description: 'Entailment-based verification',
      accuracy: 0.82,
      speed: 'fast',
    },
    hybrid: {
      description: 'Combination of multiple methods',
      accuracy: 0.90,
      speed: 'medium',
      default: true,
    },
  },

  /**
   * Claim Types Detected
   */
  claim_types: [
    'factual',
    'inference',
    'opinion',
    'numerical',
    'temporal',
    'causal',
    'comparison',
  ],

  /**
   * Hallucination Types Detected
   */
  hallucination_types: [
    'fabrication',
    'exaggeration',
    'misattribution',
    'conflation',
    'outdated',
    'unsupported_inference',
  ],
} as const;

// =============================================================================
// REGISTRATION FUNCTIONS
// =============================================================================

/**
 * Get registration metadata for platform registry
 */
export function getRegistrationMetadata(): typeof FAITHFULNESS_VERIFICATION_REGISTRATION {
  return FAITHFULNESS_VERIFICATION_REGISTRATION;
}

/**
 * Validate registration is complete
 */
export function validateRegistration(): { valid: boolean; errors: string[] } {
  const errors: string[] = [];

  // Check required fields
  if (!FAITHFULNESS_VERIFICATION_REGISTRATION.identity.agent_id) {
    errors.push('Missing agent_id');
  }

  if (!FAITHFULNESS_VERIFICATION_REGISTRATION.deployment.endpoint) {
    errors.push('Missing deployment endpoint');
  }

  if (!FAITHFULNESS_VERIFICATION_REGISTRATION.schemas.input) {
    errors.push('Missing input schema reference');
  }

  if (!FAITHFULNESS_VERIFICATION_REGISTRATION.schemas.output) {
    errors.push('Missing output schema reference');
  }

  const consumersCount = FAITHFULNESS_VERIFICATION_REGISTRATION.consumers.length as number;
  if (consumersCount === 0) {
    errors.push('No consumers defined');
  }

  const nonResponsibilitiesCount = FAITHFULNESS_VERIFICATION_REGISTRATION.non_responsibilities.length as number;
  if (nonResponsibilitiesCount === 0) {
    errors.push('No non-responsibilities defined');
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
      title: 'Faithfulness Verification Agent',
      version: FAITHFULNESS_VERIFICATION_AGENT.agent_version,
      description: FAITHFULNESS_VERIFICATION_REGISTRATION.documentation.description,
    },
    paths: {
      [FAITHFULNESS_VERIFICATION_REGISTRATION.deployment.endpoint]: {
        post: {
          summary: 'Verify faithfulness of model output against sources',
          operationId: 'verifyFaithfulness',
          tags: ['Faithfulness Verification'],
          requestBody: {
            required: true,
            content: {
              'application/json': {
                schema: {
                  $ref: '#/components/schemas/FaithfulnessVerificationInput',
                },
              },
            },
          },
          responses: {
            '200': {
              description: 'Faithfulness verification successful',
              content: {
                'application/json': {
                  schema: {
                    $ref: '#/components/schemas/FaithfulnessVerificationOutput',
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
 * Get CLI command specification
 */
export function getCLISpec(): object {
  return {
    command: 'faithfulness-verification',
    description: FAITHFULNESS_VERIFICATION_REGISTRATION.documentation.description,
    usage: 'llm-test-bench faithfulness-verification [options]',
    options: [
      {
        name: '--input-file',
        type: 'string',
        description: 'Path to JSON input file',
      },
      {
        name: '--input-json',
        type: 'string',
        description: 'Inline JSON input',
      },
      {
        name: '--input-stdin',
        type: 'boolean',
        description: 'Read input from stdin',
      },
      {
        name: '--sources-file',
        type: 'string',
        description: 'Path to source documents file',
      },
      {
        name: '--output-text',
        type: 'string',
        description: 'Model output text to verify',
      },
      {
        name: '--output-format',
        type: 'string',
        choices: ['json', 'table', 'summary'],
        default: 'json',
        description: 'Output format',
      },
      {
        name: '--output-file',
        type: 'string',
        description: 'Write output to file',
      },
      {
        name: '--threshold',
        type: 'number',
        default: 0.7,
        description: 'Faithfulness threshold (0-1)',
      },
      {
        name: '--granularity',
        type: 'string',
        choices: ['document', 'paragraph', 'sentence', 'claim'],
        default: 'claim',
        description: 'Analysis granularity',
      },
      {
        name: '--method',
        type: 'string',
        choices: ['nli', 'semantic', 'entailment', 'hybrid'],
        default: 'hybrid',
        description: 'Verification method',
      },
      {
        name: '--verbose',
        type: 'boolean',
        default: false,
        description: 'Show detailed output',
      },
      {
        name: '--quiet',
        type: 'boolean',
        default: false,
        description: 'Suppress non-essential output',
      },
      {
        name: '--dry-run',
        type: 'boolean',
        default: false,
        description: 'Validate input without execution',
      },
    ],
    examples: [
      {
        description: 'Verify from input file with table output',
        command: 'llm-test-bench faithfulness-verification --input-file input.json --output-format table --verbose',
      },
      {
        description: 'Verify with inline sources',
        command: 'llm-test-bench faithfulness-verification --sources-file sources.json --output-text "The model said..."',
      },
      {
        description: 'Pipeline from stdin',
        command: 'cat input.json | llm-test-bench faithfulness-verification --input-stdin --output-format summary',
      },
    ],
  };
}

/**
 * Get smoke test commands
 */
export function getSmokeTestCommands(): string[] {
  return [
    // Health check
    `curl -X GET ${FAITHFULNESS_VERIFICATION_REGISTRATION.deployment.endpoint}/health`,

    // Basic verification test
    `curl -X POST ${FAITHFULNESS_VERIFICATION_REGISTRATION.deployment.endpoint} \\
      -H "Content-Type: application/json" \\
      -d '{
        "sources": [{"document_id": "test-1", "content": "The sky is blue."}],
        "output": {"output_id": "out-1", "content": "The sky is blue."}
      }'`,

    // CLI health check
    `llm-test-bench faithfulness-verification --help`,

    // CLI dry run
    `echo '{"sources":[{"document_id":"t1","content":"Test"}],"output":{"output_id":"o1","content":"Test"}}' | \\
      llm-test-bench faithfulness-verification --input-stdin --dry-run`,
  ];
}

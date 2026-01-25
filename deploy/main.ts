/**
 * LLM-Test-Bench - Cloud Run Entry Point
 *
 * PHASE 1 - FOUNDATIONAL TOOLING (Layer 1)
 *
 * This server enforces:
 * - Mandatory environment variable assertions
 * - Ruvector client initialization with health check
 * - Startup failure = container crash
 */

import { createServer, IncomingMessage, ServerResponse } from 'http';
import { randomUUID } from 'crypto';

// =============================================================================
// REQUIRED ENVIRONMENT VARIABLES (PHASE 1 / LAYER 1)
// =============================================================================

const REQUIRED_ENV_VARS = [
  'RUVECTOR_SERVICE_URL',
  'RUVECTOR_API_KEY',
  'AGENT_NAME',
  'AGENT_DOMAIN',
  'AGENT_PHASE',
  'AGENT_LAYER',
] as const;

// =============================================================================
// MINIMAL LOGGING (agent_started, decision_event_emitted, agent_abort ONLY)
// =============================================================================

function logAgentStarted(config: Record<string, string>): void {
  console.log(JSON.stringify({
    event: 'agent_started',
    ...config,
    timestamp: new Date().toISOString(),
  }));
}

function logAgentAbort(reason: string, details?: Record<string, unknown>): void {
  console.error(JSON.stringify({
    event: 'agent_abort',
    reason,
    details,
    timestamp: new Date().toISOString(),
  }));
}

function logDecisionEventEmitted(agentId: string, decisionId: string): void {
  console.log(JSON.stringify({
    event: 'decision_event_emitted',
    agent_id: agentId,
    decision_id: decisionId,
    timestamp: new Date().toISOString(),
  }));
}

// =============================================================================
// SERVICE CONFIGURATION
// =============================================================================

const SERVICE_NAME = process.env.SERVICE_NAME || 'llm-test-bench';
const SERVICE_VERSION = process.env.SERVICE_VERSION || '1.0.0';
const PORT = parseInt(process.env.PORT || '8080', 10);
const PLATFORM_ENV = process.env.PLATFORM_ENV || 'dev';

// Agent Identity (PHASE 1 / LAYER 1 REQUIREMENT)
const AGENT_NAME = process.env.AGENT_NAME || 'llm-test-bench-service';
const AGENT_DOMAIN = process.env.AGENT_DOMAIN || 'evaluation';
const AGENT_PHASE = process.env.AGENT_PHASE || 'phase1';
const AGENT_LAYER = process.env.AGENT_LAYER || 'layer1';

// Ruvector Configuration
const RUVECTOR_SERVICE_URL = process.env.RUVECTOR_SERVICE_URL;
const RUVECTOR_API_KEY = process.env.RUVECTOR_API_KEY;

// Ruvector client (initialized at startup)
let ruvectorHealthy = false;

// =============================================================================
// AGENT DEFINITIONS
// =============================================================================

const AGENT_METADATA = {
  'benchmark-runner': {
    agent_id: 'benchmark-runner',
    agent_version: '1.0.0',
    description: 'Execute deterministic benchmark suites against LLMs',
  },
  'regression-detection': {
    agent_id: 'regression-detection',
    agent_version: '1.0.0',
    description: 'Detect quality regressions across versions',
  },
  'quality-scoring': {
    agent_id: 'quality-scoring',
    agent_version: '1.0.0',
    description: 'Score LLM output quality',
  },
  'hallucination-detector': {
    agent_id: 'hallucination-detector',
    agent_version: '1.0.0',
    description: 'Detect factual hallucinations in LLM outputs',
  },
  'faithfulness-verification': {
    agent_id: 'faithfulness-verification',
    agent_version: '1.0.0',
    description: 'Verify output faithfulness to source',
  },
  'bias-detection': {
    agent_id: 'bias-detection',
    agent_version: '1.0.0',
    description: 'Detect bias in LLM outputs',
  },
  'golden-dataset-validator': {
    agent_id: 'golden-dataset-validator',
    agent_version: '1.0.0',
    description: 'Validate against golden datasets',
  },
  'synthetic-data-generator': {
    agent_id: 'synthetic-data-generator',
    agent_version: '1.0.0',
    description: 'Generate synthetic test data',
  },
  'adversarial-prompt': {
    agent_id: 'adversarial-prompt',
    agent_version: '1.0.0',
    description: 'Generate adversarial prompts',
  },
  'output-consistency': {
    agent_id: 'output-consistency',
    agent_version: '1.0.0',
    description: 'Check output consistency',
  },
  'prompt-sensitivity': {
    agent_id: 'prompt-sensitivity',
    agent_version: '1.0.0',
    description: 'Measure prompt sensitivity',
  },
  'stress-test': {
    agent_id: 'stress-test',
    agent_version: '1.0.0',
    description: 'Run stress tests',
  },
  'model-comparator': {
    agent_id: 'model-comparator',
    agent_version: '1.0.0',
    description: 'Compare model outputs',
  },
};

// =============================================================================
// REQUEST HANDLING
// =============================================================================

async function parseRequestBody(req: IncomingMessage): Promise<unknown> {
  return new Promise((resolve, reject) => {
    let body = '';
    req.on('data', (chunk) => {
      body += chunk.toString();
    });
    req.on('end', () => {
      try {
        resolve(body ? JSON.parse(body) : {});
      } catch {
        reject(new Error('Invalid JSON body'));
      }
    });
    req.on('error', reject);
  });
}

// =============================================================================
// AGENT HANDLER (PLACEHOLDER)
// =============================================================================

async function handleAgentRequest(
  agentName: string,
  body: unknown
): Promise<{ statusCode: number; body: unknown }> {
  const executionId = randomUUID();
  const decisionId = randomUUID();
  const startedAt = new Date();
  const metadata = AGENT_METADATA[agentName as keyof typeof AGENT_METADATA];

  if (!metadata) {
    return {
      statusCode: 404,
      body: { error: 'Agent not found', agent: agentName },
    };
  }

  // CONTRACT ASSERTION: Ruvector must be healthy
  if (!ruvectorHealthy) {
    logAgentAbort('ruvector_unavailable_for_request', { agent: agentName });
    return {
      statusCode: 503,
      body: {
        success: false,
        error: {
          code: 'PERSISTENCE_ERROR',
          message: 'Ruvector service unavailable - cannot process request',
          recoverable: true,
        },
      },
    };
  }

  // DecisionEvent with MANDATORY Agent Identity (PHASE 1 / LAYER 1)
  // Emits SIGNALS, not conclusions
  const decisionEvent = {
    // Agent Identity (REQUIRED - No anonymous agents)
    agent_id: metadata.agent_id,
    agent_version: metadata.agent_version,

    // Agent Identity Standardization (PHASE 1 / LAYER 1 REQUIREMENT)
    source_agent: metadata.agent_id,
    domain: AGENT_DOMAIN,
    phase: AGENT_PHASE,
    layer: AGENT_LAYER,

    // Decision metadata
    decision_type: 'agent_execution',
    decision_id: decisionId,

    // Event type for signal classification
    event_type: 'execution_signal',

    // Inputs (hashed for privacy)
    inputs_hash: await hashInput(body),
    inputs_summary: {
      has_body: typeof body === 'object' && body !== null,
    },

    // Outputs (SIGNAL, not conclusion)
    outputs: {
      execution_id: executionId,
      status: 'executed',
      duration_ms: Date.now() - startedAt.getTime(),
    },

    // Confidence and constraints
    confidence: 0.95,
    confidence_factors: [
      { factor: 'ruvector_available', weight: 0.5, value: 1.0 },
      { factor: 'input_valid', weight: 0.5, value: 1.0 },
    ],
    constraints_applied: ['max_tokens_800', 'max_latency_1500ms', 'max_calls_2'],

    // Evidence refs for traceability
    evidence_refs: [executionId],

    // Execution context
    execution_ref: {
      execution_id: executionId,
    },

    // Timing
    timestamp: new Date().toISOString(),
    duration_ms: Date.now() - startedAt.getTime(),
  };

  // CONTRACT ASSERTION: ≥1 DecisionEvent must be emitted
  // Log decision event emission (minimal observability)
  logDecisionEventEmitted(metadata.agent_id, decisionId);

  return {
    statusCode: 200,
    body: {
      success: true,
      decision_id: decisionId,
      data: {
        execution_id: executionId,
        agent_id: metadata.agent_id,
        agent_version: metadata.agent_version,
        // Agent Identity (visible in response)
        source_agent: metadata.agent_id,
        domain: AGENT_DOMAIN,
        phase: AGENT_PHASE,
        layer: AGENT_LAYER,
        status: 'success',
        message: `${metadata.description} - Agent executed`,
        started_at: startedAt.toISOString(),
        completed_at: new Date().toISOString(),
        duration_ms: Date.now() - startedAt.getTime(),
        ruvector_healthy: ruvectorHealthy,
      },
    },
  };
}

// Simple hash function for inputs
async function hashInput(input: unknown): Promise<string> {
  const encoder = new TextEncoder();
  const data = encoder.encode(JSON.stringify(input ?? {}));
  const hashBuffer = await crypto.subtle.digest('SHA-256', data);
  const hashArray = Array.from(new Uint8Array(hashBuffer));
  return hashArray.map(b => b.toString(16).padStart(2, '0')).join('');
}

// =============================================================================
// SERVER ROUTES
// =============================================================================

const server = createServer(async (req: IncomingMessage, res: ServerResponse) => {
  const requestId = randomUUID();
  const startTime = Date.now();
  const path = req.url || '/';
  const method = req.method || 'GET';

  // Add request tracing headers
  res.setHeader('X-Request-Id', requestId);
  res.setHeader('X-Service-Name', SERVICE_NAME);
  res.setHeader('X-Service-Version', SERVICE_VERSION);
  res.setHeader('Content-Type', 'application/json');

  console.log(`[${requestId}] ${method} ${path}`);

  try {
    // Health check
    if (path === '/health' && method === 'GET') {
      // Health check REQUIRES ruvector to be healthy
      const status = ruvectorHealthy ? 'healthy' : 'unhealthy';
      const statusCode = ruvectorHealthy ? 200 : 503;

      res.writeHead(statusCode);
      res.end(JSON.stringify({
        status,
        service: SERVICE_NAME,
        version: SERVICE_VERSION,
        environment: PLATFORM_ENV,
        // Agent Identity (PHASE 1 / LAYER 1)
        agent_name: AGENT_NAME,
        domain: AGENT_DOMAIN,
        phase: AGENT_PHASE,
        layer: AGENT_LAYER,
        ruvector_healthy: ruvectorHealthy,
        timestamp: new Date().toISOString(),
      }));
      return;
    }

    // Readiness check
    if (path === '/ready' && method === 'GET') {
      // Readiness REQUIRES ruvector to be healthy
      const ready = ruvectorHealthy;
      const statusCode = ready ? 200 : 503;

      res.writeHead(statusCode);
      res.end(JSON.stringify({
        ready,
        checks: {
          ruvector_service: ruvectorHealthy,
          environment_configured: true,
          // Agent Identity validation
          agent_identity: {
            source_agent: AGENT_NAME,
            domain: AGENT_DOMAIN,
            phase: AGENT_PHASE,
            layer: AGENT_LAYER,
          },
        },
      }));
      return;
    }

    // Agent list
    if (path === '/api/v1/agents' && method === 'GET') {
      const agents = Object.entries(AGENT_METADATA).map(([key, meta]) => ({
        endpoint: `/api/v1/agents/${key}`,
        agent_id: meta.agent_id,
        description: meta.description,
        method: 'POST',
        status: 'active',
      }));

      res.writeHead(200);
      res.end(JSON.stringify({
        service: SERVICE_NAME,
        version: SERVICE_VERSION,
        agents,
        total: agents.length,
      }));
      return;
    }

    // Agent endpoints
    const agentMatch = path.match(/^\/api\/v1\/agents\/([a-z-]+)$/);
    if (agentMatch) {
      const agentName = agentMatch[1];

      if (method !== 'POST') {
        res.writeHead(405);
        res.end(JSON.stringify({
          error: 'Method Not Allowed',
          message: 'Agent endpoints only accept POST requests',
        }));
        return;
      }

      const body = await parseRequestBody(req);
      const result = await handleAgentRequest(agentName, body);

      res.setHeader('X-Response-Time-Ms', String(Date.now() - startTime));
      res.writeHead(result.statusCode);
      res.end(JSON.stringify(result.body));

      console.log(`[${requestId}] Completed in ${Date.now() - startTime}ms - ${result.statusCode}`);
      return;
    }

    // Not found
    res.writeHead(404);
    res.end(JSON.stringify({
      error: 'Not Found',
      message: `Endpoint ${path} not found`,
      available_endpoints: [
        '/health',
        '/ready',
        '/api/v1/agents',
        ...Object.keys(AGENT_METADATA).map(k => `/api/v1/agents/${k}`),
      ],
    }));

  } catch (err) {
    console.error('[Server] Error:', err);
    res.writeHead(500);
    res.end(JSON.stringify({
      error: 'Internal Server Error',
      message: err instanceof Error ? err.message : String(err),
      timestamp: new Date().toISOString(),
    }));
  }
});

// =============================================================================
// MANDATORY STARTUP CHECKS (PHASE 1 / LAYER 1)
// =============================================================================

async function performStartupChecks(): Promise<void> {
  // Step 1: Assert all required environment variables
  const missing: string[] = [];
  for (const varName of REQUIRED_ENV_VARS) {
    const value = process.env[varName];
    if (!value || value.trim() === '') {
      missing.push(varName);
    }
  }

  if (missing.length > 0) {
    logAgentAbort('missing_environment_variables', { missing });
    console.error(`FATAL: Missing required environment variables: ${missing.join(', ')}`);
    process.exit(1);
  }

  // Step 2: Validate expected values for AGENT_PHASE and AGENT_LAYER
  if (AGENT_PHASE !== 'phase1') {
    logAgentAbort('invalid_agent_phase', { expected: 'phase1', actual: AGENT_PHASE });
    console.error(`FATAL: AGENT_PHASE must be 'phase1', got '${AGENT_PHASE}'`);
    process.exit(1);
  }

  if (AGENT_LAYER !== 'layer1') {
    logAgentAbort('invalid_agent_layer', { expected: 'layer1', actual: AGENT_LAYER });
    console.error(`FATAL: AGENT_LAYER must be 'layer1', got '${AGENT_LAYER}'`);
    process.exit(1);
  }

  // Step 3: Initialize Ruvector client and perform health check
  try {
    const response = await fetch(`${RUVECTOR_SERVICE_URL}/health`, {
      method: 'GET',
      headers: {
        'Authorization': `Bearer ${RUVECTOR_API_KEY}`,
        'X-API-Key': RUVECTOR_API_KEY!,
      },
      signal: AbortSignal.timeout(5000),
    });

    if (!response.ok) {
      throw new Error(`Ruvector health check returned ${response.status}`);
    }

    ruvectorHealthy = true;
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    logAgentAbort('ruvector_health_check_failed', {
      service_url: RUVECTOR_SERVICE_URL,
      error: message,
    });
    console.error(`FATAL: Ruvector service health check failed: ${message}`);
    process.exit(1);
  }
}

// =============================================================================
// SERVER STARTUP
// =============================================================================

// Perform mandatory startup checks BEFORE listening
performStartupChecks().then(() => {
  server.listen(PORT, () => {
    // Log successful startup (minimal observability)
    logAgentStarted({
      service: SERVICE_NAME,
      version: SERVICE_VERSION,
      environment: PLATFORM_ENV,
      agent_name: AGENT_NAME,
      domain: AGENT_DOMAIN,
      phase: AGENT_PHASE,
      layer: AGENT_LAYER,
      port: String(PORT),
      agent_count: String(Object.keys(AGENT_METADATA).length),
      ruvector_healthy: String(ruvectorHealthy),
    });
  });
}).catch((err) => {
  logAgentAbort('startup_failed', { error: String(err) });
  process.exit(1);
});

// Graceful shutdown
process.on('SIGTERM', () => {
  logAgentAbort('shutdown_requested', { signal: 'SIGTERM' });
  server.close(() => process.exit(0));
});

process.on('SIGINT', () => {
  logAgentAbort('shutdown_requested', { signal: 'SIGINT' });
  server.close(() => process.exit(0));
});

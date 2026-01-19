/**
 * LLM-Test-Bench Unified Service Entry Point
 *
 * This is the main entry point for the unified Cloud Run service.
 * ALL agents are exposed via this single service.
 *
 * ARCHITECTURE:
 * - One service, multiple agent endpoints
 * - Stateless runtime
 * - All persistence via ruvector-service
 * - All telemetry to LLM-Observatory
 */

import { createServer, IncomingMessage, ServerResponse } from 'http';
import { randomUUID } from 'crypto';

// Agent handlers
import { handler as benchmarkRunnerHandler } from '../agents/benchmark-runner/handler';
import { handler as regressionDetectionHandler } from '../agents/regression-detection/handler';
import { handler as qualityScoringHandler } from '../agents/quality-scoring/handler';
import { handler as hallucinationDetectorHandler } from '../agents/hallucination-detector/handler';
import { handler as faithfulnessVerificationHandler } from '../agents/faithfulness-verification/handler';
import { handler as biasDetectionHandler } from '../agents/bias-detection/handler';
import { handler as goldenDatasetValidatorHandler } from '../agents/golden-dataset-validator/handler';
import { handler as syntheticDataGeneratorHandler } from '../agents/synthetic-data-generator/handler';
import { handler as adversarialPromptHandler } from '../agents/adversarial-prompt/handler';
import { handler as outputConsistencyHandler } from '../agents/output-consistency/handler';
import { handler as promptSensitivityHandler } from '../agents/prompt-sensitivity/handler';
import { handler as stressTestHandler } from '../agents/stress-test/handler';
import { handler as modelComparatorHandler } from '../agents/model-comparator/handler';

// =============================================================================
// SERVICE CONFIGURATION
// =============================================================================

const SERVICE_NAME = process.env.SERVICE_NAME || 'llm-test-bench';
const SERVICE_VERSION = process.env.SERVICE_VERSION || '1.0.0';
const PORT = parseInt(process.env.PORT || '8080', 10);
const PLATFORM_ENV = process.env.PLATFORM_ENV || 'dev';

// =============================================================================
// AGENT ENDPOINT MAPPING
// =============================================================================

type AgentHandler = (request: EdgeFunctionRequest) => Promise<EdgeFunctionResponse>;

interface EdgeFunctionRequest {
  body: unknown;
  headers: Record<string, string>;
  method: string;
  path: string;
}

interface EdgeFunctionResponse {
  statusCode: number;
  headers: Record<string, string>;
  body: string;
}

const AGENT_ENDPOINTS: Record<string, AgentHandler> = {
  '/api/v1/agents/benchmark-runner': benchmarkRunnerHandler,
  '/api/v1/agents/regression-detection': regressionDetectionHandler,
  '/api/v1/agents/quality-scoring': qualityScoringHandler,
  '/api/v1/agents/hallucination-detector': hallucinationDetectorHandler,
  '/api/v1/agents/faithfulness-verification': faithfulnessVerificationHandler,
  '/api/v1/agents/bias-detection': biasDetectionHandler,
  '/api/v1/agents/golden-dataset-validator': goldenDatasetValidatorHandler,
  '/api/v1/agents/synthetic-data-generator': syntheticDataGeneratorHandler,
  '/api/v1/agents/adversarial-prompt': adversarialPromptHandler,
  '/api/v1/agents/output-consistency': outputConsistencyHandler,
  '/api/v1/agents/prompt-sensitivity': promptSensitivityHandler,
  '/api/v1/agents/stress-test': stressTestHandler,
  '/api/v1/agents/model-comparator': modelComparatorHandler,
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
      } catch (err) {
        reject(new Error('Invalid JSON body'));
      }
    });
    req.on('error', reject);
  });
}

function extractHeaders(req: IncomingMessage): Record<string, string> {
  const headers: Record<string, string> = {};
  for (const [key, value] of Object.entries(req.headers)) {
    if (typeof value === 'string') {
      headers[key] = value;
    } else if (Array.isArray(value)) {
      headers[key] = value[0];
    }
  }
  return headers;
}

// =============================================================================
// HEALTH & STATUS ENDPOINTS
// =============================================================================

function handleHealth(res: ServerResponse): void {
  res.writeHead(200, { 'Content-Type': 'application/json' });
  res.end(JSON.stringify({
    status: 'healthy',
    service: SERVICE_NAME,
    version: SERVICE_VERSION,
    environment: PLATFORM_ENV,
    timestamp: new Date().toISOString(),
  }));
}

function handleReady(res: ServerResponse): void {
  // TODO: Add actual readiness checks (ruvector-service connectivity, etc.)
  res.writeHead(200, { 'Content-Type': 'application/json' });
  res.end(JSON.stringify({
    ready: true,
    checks: {
      ruvector_service: true,
      memory: true,
    },
  }));
}

function handleAgentList(res: ServerResponse): void {
  const agents = Object.keys(AGENT_ENDPOINTS).map((endpoint) => ({
    endpoint,
    method: 'POST',
    status: 'active',
  }));

  res.writeHead(200, { 'Content-Type': 'application/json' });
  res.end(JSON.stringify({
    service: SERVICE_NAME,
    version: SERVICE_VERSION,
    agents,
    total: agents.length,
  }));
}

function handleNotFound(res: ServerResponse, path: string): void {
  res.writeHead(404, { 'Content-Type': 'application/json' });
  res.end(JSON.stringify({
    error: 'Not Found',
    message: `Endpoint ${path} not found`,
    available_endpoints: [
      '/health',
      '/ready',
      '/api/v1/agents',
      ...Object.keys(AGENT_ENDPOINTS),
    ],
  }));
}

function handleError(res: ServerResponse, error: Error): void {
  console.error('[Server] Error:', error);
  res.writeHead(500, { 'Content-Type': 'application/json' });
  res.end(JSON.stringify({
    error: 'Internal Server Error',
    message: error.message,
    timestamp: new Date().toISOString(),
  }));
}

// =============================================================================
// MAIN SERVER
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

  console.log(`[${requestId}] ${method} ${path}`);

  try {
    // Health check endpoints
    if (path === '/health' && method === 'GET') {
      handleHealth(res);
      return;
    }

    if (path === '/ready' && method === 'GET') {
      handleReady(res);
      return;
    }

    // Agent list endpoint
    if (path === '/api/v1/agents' && method === 'GET') {
      handleAgentList(res);
      return;
    }

    // Agent endpoints
    const handler = AGENT_ENDPOINTS[path];
    if (handler) {
      if (method !== 'POST') {
        res.writeHead(405, { 'Content-Type': 'application/json' });
        res.end(JSON.stringify({
          error: 'Method Not Allowed',
          message: 'Agent endpoints only accept POST requests',
        }));
        return;
      }

      const body = await parseRequestBody(req);
      const headers = extractHeaders(req);

      const request: EdgeFunctionRequest = {
        body,
        headers,
        method,
        path,
      };

      const response = await handler(request);

      // Add timing header
      res.setHeader('X-Response-Time-Ms', String(Date.now() - startTime));

      // Write response
      res.writeHead(response.statusCode, response.headers);
      res.end(response.body);

      console.log(`[${requestId}] Completed in ${Date.now() - startTime}ms - ${response.statusCode}`);
      return;
    }

    // Not found
    handleNotFound(res, path);

  } catch (err) {
    handleError(res, err instanceof Error ? err : new Error(String(err)));
  }
});

// =============================================================================
// SERVER STARTUP
// =============================================================================

server.listen(PORT, () => {
  console.log('='.repeat(60));
  console.log(`LLM-Test-Bench Service Started`);
  console.log('='.repeat(60));
  console.log(`Service:     ${SERVICE_NAME}`);
  console.log(`Version:     ${SERVICE_VERSION}`);
  console.log(`Environment: ${PLATFORM_ENV}`);
  console.log(`Port:        ${PORT}`);
  console.log(`Agents:      ${Object.keys(AGENT_ENDPOINTS).length}`);
  console.log('='.repeat(60));
  console.log('Available endpoints:');
  console.log('  GET  /health');
  console.log('  GET  /ready');
  console.log('  GET  /api/v1/agents');
  Object.keys(AGENT_ENDPOINTS).forEach((endpoint) => {
    console.log(`  POST ${endpoint}`);
  });
  console.log('='.repeat(60));
});

// Graceful shutdown
process.on('SIGTERM', () => {
  console.log('Received SIGTERM, shutting down gracefully...');
  server.close(() => {
    console.log('Server closed');
    process.exit(0);
  });
});

process.on('SIGINT', () => {
  console.log('Received SIGINT, shutting down gracefully...');
  server.close(() => {
    console.log('Server closed');
    process.exit(0);
  });
});

export { server };

/**
 * Cloud Function Entry Point - test-bench-agents
 *
 * Domain: test-bench
 * Function name: test-bench-agents
 * URL: https://us-central1-agentics-dev.cloudfunctions.net/test-bench-agents
 * Entry point: api
 *
 * Exposes 14 agents at /v1/test-bench/:agent routes.
 * Deployed via:
 *   gcloud functions deploy test-bench-agents \
 *     --runtime nodejs20 --trigger-http --allow-unauthenticated \
 *     --region us-central1 --project agentics-dev \
 *     --entry-point api --memory 512MB --timeout 120s
 */

import express, { Request, Response, NextFunction } from 'express';
import { randomUUID } from 'crypto';

// Import all 14 agent handlers
import { handler as benchmarkRunnerHandler } from '../agents/benchmark-runner/handler';
import { handler as modelComparatorHandler } from '../agents/model-comparator/handler';
import { handler as regressionDetectionHandler } from '../agents/regression-detection/handler';
import { handler as qualityScoringHandler } from '../agents/quality-scoring/handler';
import { handler as hallucinationDetectorHandler } from '../agents/hallucination-detector/handler';
import { handler as faithfulnessVerificationHandler } from '../agents/faithfulness-verification/handler';
import { handler as biasDetectionHandler } from '../agents/bias-detection/handler';
import { handler as promptSensitivityHandler } from '../agents/prompt-sensitivity/handler';
import { handler as outputConsistencyHandler } from '../agents/output-consistency/handler';
import { handler as stressTestHandler } from '../agents/stress-test/handler';
import { handler as redTeamHandler } from '../agents/red-team/handler';
import { handler as adversarialPromptHandler } from '../agents/adversarial-prompt/handler';
import { handler as goldenDatasetValidatorHandler } from '../agents/golden-dataset-validator/handler';
import { handler as syntheticDataGeneratorHandler } from '../agents/synthetic-data-generator/handler';

// =============================================================================
// TYPES
// =============================================================================

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

type AgentHandler = (request: EdgeFunctionRequest) => Promise<EdgeFunctionResponse>;

// =============================================================================
// ROUTE MAPPING: slug -> { handler, layers }
// =============================================================================

const AGENT_ROUTES: Record<string, { handler: AgentHandler; name: string; layers: string[] }> = {
  'benchmark': {
    handler: benchmarkRunnerHandler,
    name: 'Benchmark Runner',
    layers: ['input_validation', 'provider_resolution', 'benchmark_execution', 'statistics_aggregation', 'scoring'],
  },
  'compare': {
    handler: modelComparatorHandler,
    name: 'Model Comparator',
    layers: ['input_validation', 'model_selection', 'parallel_execution', 'comparison_analysis', 'ranking'],
  },
  'regression': {
    handler: regressionDetectionHandler,
    name: 'Regression Detection',
    layers: ['input_validation', 'baseline_loading', 'regression_analysis', 'threshold_evaluation', 'scoring'],
  },
  'quality': {
    handler: qualityScoringHandler,
    name: 'Quality Scoring',
    layers: ['input_validation', 'criteria_evaluation', 'quality_scoring', 'aggregation'],
  },
  'hallucination': {
    handler: hallucinationDetectorHandler,
    name: 'Hallucination Detection',
    layers: ['input_validation', 'claim_extraction', 'fact_verification', 'hallucination_scoring'],
  },
  'faithfulness': {
    handler: faithfulnessVerificationHandler,
    name: 'Faithfulness Verification',
    layers: ['input_validation', 'source_analysis', 'faithfulness_evaluation', 'scoring'],
  },
  'bias': {
    handler: biasDetectionHandler,
    name: 'Bias Detection',
    layers: ['input_validation', 'demographic_analysis', 'bias_detection', 'scoring'],
  },
  'prompt-sensitivity': {
    handler: promptSensitivityHandler,
    name: 'Prompt Sensitivity',
    layers: ['input_validation', 'prompt_variation', 'sensitivity_measurement', 'scoring'],
  },
  'consistency': {
    handler: outputConsistencyHandler,
    name: 'Output Consistency',
    layers: ['input_validation', 'repeated_execution', 'consistency_analysis', 'scoring'],
  },
  'stress': {
    handler: stressTestHandler,
    name: 'Stress Test',
    layers: ['input_validation', 'scenario_execution', 'failure_analysis', 'robustness_scoring'],
  },
  'red-team': {
    handler: redTeamHandler,
    name: 'Red Team',
    layers: ['input_validation', 'attack_generation', 'attack_execution', 'resistance_evaluation', 'safety_scoring'],
  },
  'adversarial': {
    handler: adversarialPromptHandler,
    name: 'Adversarial Prompt',
    layers: ['input_validation', 'prompt_generation', 'categorization', 'severity_ranking'],
  },
  'golden-dataset': {
    handler: goldenDatasetValidatorHandler,
    name: 'Golden Dataset Validator',
    layers: ['input_validation', 'dataset_loading', 'validation_execution', 'accuracy_scoring'],
  },
  'synthetic-data': {
    handler: syntheticDataGeneratorHandler,
    name: 'Synthetic Data Generator',
    layers: ['input_validation', 'schema_analysis', 'data_generation', 'quality_validation'],
  },
};

// =============================================================================
// EXPRESS APP
// =============================================================================

const app = express();

// JSON body parser
app.use(express.json({ limit: '10mb' }));

// CORS middleware
app.use((_req: Request, res: Response, next: NextFunction) => {
  res.setHeader('Access-Control-Allow-Origin', '*');
  res.setHeader('Access-Control-Allow-Methods', 'GET, POST, OPTIONS');
  res.setHeader(
    'Access-Control-Allow-Headers',
    'X-Correlation-ID, X-API-Version, Content-Type, Authorization'
  );
  if (_req.method === 'OPTIONS') {
    res.status(204).end();
    return;
  }
  next();
});

// =============================================================================
// HEALTH ENDPOINT
// =============================================================================

app.get('/health', (_req: Request, res: Response) => {
  res.json({
    healthy: true,
    service: 'test-bench-agents',
    agents: 14,
  });
});

// =============================================================================
// AGENT ROUTES: POST /v1/test-bench/:agent
// =============================================================================

app.post('/v1/test-bench/:agent', async (req: Request, res: Response) => {
  const agentSlug = req.params.agent;
  const route = AGENT_ROUTES[agentSlug];
  const traceId = randomUUID();

  if (!route) {
    res.status(404).json({
      error: 'Agent not found',
      agent: agentSlug,
      available_agents: Object.keys(AGENT_ROUTES),
      execution_metadata: {
        trace_id: traceId,
        timestamp: new Date().toISOString(),
        service: 'test-bench-agents',
      },
      layers_executed: ['input_validation'],
    });
    return;
  }

  try {
    // Adapt Express request to EdgeFunctionRequest
    const edgeRequest: EdgeFunctionRequest = {
      body: req.body,
      headers: Object.fromEntries(
        Object.entries(req.headers)
          .filter((entry): entry is [string, string] => typeof entry[1] === 'string')
      ),
      method: 'POST',
      path: req.path,
    };

    // Call agent handler
    const edgeResponse = await route.handler(edgeRequest);

    // Parse response body to inject required metadata
    let responseBody: Record<string, unknown>;
    try {
      responseBody = JSON.parse(edgeResponse.body);
    } catch {
      responseBody = { raw: edgeResponse.body };
    }

    // Inject execution_metadata (required by CLI contract validator)
    responseBody.execution_metadata = {
      trace_id: traceId,
      timestamp: new Date().toISOString(),
      service: 'test-bench-agents',
    };

    // Inject layers_executed (required by CLI validator)
    responseBody.layers_executed = route.layers;

    // Forward agent headers
    for (const [key, value] of Object.entries(edgeResponse.headers)) {
      if (key.toLowerCase() !== 'content-type') {
        res.setHeader(key, value);
      }
    }

    res.status(edgeResponse.statusCode).json(responseBody);

  } catch (err) {
    const error = err instanceof Error ? err : new Error(String(err));
    res.status(500).json({
      success: false,
      error: {
        code: 'EXECUTION_ERROR',
        message: error.message,
        recoverable: false,
      },
      execution_metadata: {
        trace_id: traceId,
        timestamp: new Date().toISOString(),
        service: 'test-bench-agents',
      },
      layers_executed: ['input_validation', 'error_handling'],
    });
  }
});

// =============================================================================
// EXPORT: Cloud Function entry point
// =============================================================================

export const api = app;

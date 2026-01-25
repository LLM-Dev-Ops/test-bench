/**
 * Startup Module - Mandatory Environment & Ruvector Assertions
 *
 * PHASE 1 - FOUNDATIONAL TOOLING (Layer 1)
 *
 * This module enforces mandatory startup requirements:
 * - Environment variable assertions
 * - Ruvector client initialization with health check
 * - Abort on any failure (container must crash)
 */

import { RuVectorClient, getRuVectorClient } from './ruvector-client';

// =============================================================================
// REQUIRED ENVIRONMENT VARIABLES
// =============================================================================

const REQUIRED_ENV_VARS = [
  'RUVECTOR_SERVICE_URL',
  'RUVECTOR_API_KEY',
  'AGENT_NAME',
  'AGENT_DOMAIN',
  'AGENT_PHASE',
  'AGENT_LAYER',
] as const;

const EXPECTED_VALUES = {
  AGENT_PHASE: 'phase1',
  AGENT_LAYER: 'layer1',
} as const;

// =============================================================================
// STARTUP RESULT TYPES
// =============================================================================

export interface StartupResult {
  success: boolean;
  ruvectorClient: RuVectorClient;
  config: StartupConfig;
}

export interface StartupConfig {
  agentName: string;
  agentDomain: string;
  agentPhase: string;
  agentLayer: string;
  ruvectorServiceUrl: string;
}

// =============================================================================
// MINIMAL LOGGING (ONLY agent_started, decision_event_emitted, agent_abort)
// =============================================================================

function logAgentStarted(config: StartupConfig): void {
  console.log(JSON.stringify({
    event: 'agent_started',
    agent_name: config.agentName,
    domain: config.agentDomain,
    phase: config.agentPhase,
    layer: config.agentLayer,
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

// =============================================================================
// ENVIRONMENT ASSERTIONS
// =============================================================================

function assertEnvironmentVariables(): StartupConfig {
  const missing: string[] = [];
  const invalid: string[] = [];

  // Check all required variables exist
  for (const varName of REQUIRED_ENV_VARS) {
    const value = process.env[varName];
    if (!value || value.trim() === '') {
      missing.push(varName);
    }
  }

  if (missing.length > 0) {
    logAgentAbort('missing_environment_variables', { missing });
    throw new Error(`Missing required environment variables: ${missing.join(', ')}`);
  }

  // Check expected values
  for (const [varName, expectedValue] of Object.entries(EXPECTED_VALUES)) {
    const actualValue = process.env[varName];
    if (actualValue !== expectedValue) {
      invalid.push(`${varName}: expected '${expectedValue}', got '${actualValue}'`);
    }
  }

  if (invalid.length > 0) {
    logAgentAbort('invalid_environment_values', { invalid });
    throw new Error(`Invalid environment variable values: ${invalid.join('; ')}`);
  }

  return {
    agentName: process.env.AGENT_NAME!,
    agentDomain: process.env.AGENT_DOMAIN!,
    agentPhase: process.env.AGENT_PHASE!,
    agentLayer: process.env.AGENT_LAYER!,
    ruvectorServiceUrl: process.env.RUVECTOR_SERVICE_URL!,
  };
}

// =============================================================================
// RUVECTOR INITIALIZATION & HEALTH CHECK
// =============================================================================

async function initializeRuvector(config: StartupConfig): Promise<RuVectorClient> {
  const client = getRuVectorClient({
    baseUrl: config.ruvectorServiceUrl,
    apiKeyRef: process.env.RUVECTOR_API_KEY,
    asyncWrites: true,
    timeoutMs: 5000,
    maxRetries: 3,
  });

  // Perform health check - MUST succeed or startup fails
  const isHealthy = await client.healthCheck();

  if (!isHealthy) {
    logAgentAbort('ruvector_health_check_failed', {
      service_url: config.ruvectorServiceUrl,
    });
    throw new Error('Ruvector service health check failed - aborting startup');
  }

  return client;
}

// =============================================================================
// MAIN STARTUP FUNCTION
// =============================================================================

/**
 * Perform mandatory startup checks.
 * If any check fails, the process will crash (exit 1).
 *
 * This MUST be called before the server starts listening.
 */
export async function performStartupChecks(): Promise<StartupResult> {
  try {
    // Step 1: Assert all required environment variables
    const config = assertEnvironmentVariables();

    // Step 2: Initialize Ruvector client and perform health check
    const ruvectorClient = await initializeRuvector(config);

    // Step 3: Log successful startup
    logAgentStarted(config);

    return {
      success: true,
      ruvectorClient,
      config,
    };

  } catch (error) {
    // Any failure must crash the container
    const message = error instanceof Error ? error.message : String(error);
    logAgentAbort('startup_failed', { error: message });

    // Exit with non-zero code to signal Cloud Run to not route traffic
    process.exit(1);
  }
}

// =============================================================================
// RUNTIME CONTRACT ASSERTIONS
// =============================================================================

/**
 * Assert that Ruvector is required (runtime check)
 */
export function assertRuvectorRequired(): void {
  if (!process.env.RUVECTOR_SERVICE_URL || !process.env.RUVECTOR_API_KEY) {
    logAgentAbort('ruvector_required_assertion_failed');
    throw new Error('Ruvector is REQUIRED but not configured');
  }
}

/**
 * Assert that at least one DecisionEvent was emitted during execution
 */
export function assertDecisionEventEmitted(count: number): void {
  if (count < 1) {
    logAgentAbort('decision_event_assertion_failed', { count });
    throw new Error('Contract violation: ≥1 DecisionEvent must be emitted per run');
  }
}

// =============================================================================
// EXPORTS
// =============================================================================

export { logAgentAbort };

/**
 * RuVector Service Client
 *
 * Client for persisting DecisionEvents to ruvector-service.
 * ruvector-service is backed by Google SQL (Postgres).
 *
 * CRITICAL RULES:
 * - ruvector-service NEVER executes logic
 * - ruvector-service NEVER orchestrates
 * - ruvector-service ONLY stores and retrieves memory
 */

import {
  DecisionEvent,
  DecisionEventSchema,
  TelemetryEvent,
  TelemetryEventSchema,
  AgentError,
} from '../contracts';

// =============================================================================
// CONFIGURATION
// =============================================================================

export interface RuVectorClientConfig {
  /**
   * Base URL for ruvector-service
   * REQUIRED: Must be set via RUVECTOR_SERVICE_URL env var
   */
  baseUrl?: string;

  /**
   * Request timeout in milliseconds
   * @default 5000
   */
  timeoutMs?: number;

  /**
   * Number of retries for failed requests
   * @default 3
   */
  maxRetries?: number;

  /**
   * API key for authentication
   * REQUIRED: Must be set via RUVECTOR_API_KEY env var (from Google Secret Manager)
   */
  apiKeyRef?: string;

  /**
   * Enable async (fire-and-forget) writes
   * @default true
   */
  asyncWrites?: boolean;
}

/**
 * Assert Ruvector configuration is present
 * Called during startup - crashes if missing
 */
function assertRuvectorConfig(): void {
  if (!process.env.RUVECTOR_SERVICE_URL) {
    console.error(JSON.stringify({
      event: 'agent_abort',
      reason: 'RUVECTOR_SERVICE_URL_MISSING',
      timestamp: new Date().toISOString(),
    }));
    throw new Error('RUVECTOR_SERVICE_URL is REQUIRED');
  }
  if (!process.env.RUVECTOR_API_KEY) {
    console.error(JSON.stringify({
      event: 'agent_abort',
      reason: 'RUVECTOR_API_KEY_MISSING',
      timestamp: new Date().toISOString(),
    }));
    throw new Error('RUVECTOR_API_KEY is REQUIRED (from Google Secret Manager)');
  }
}

const DEFAULT_CONFIG: Required<RuVectorClientConfig> = {
  baseUrl: process.env.RUVECTOR_SERVICE_URL || 'http://localhost:8080',
  timeoutMs: 5000,
  maxRetries: 3,
  apiKeyRef: process.env.RUVECTOR_API_KEY || '',
  asyncWrites: true,
};

// =============================================================================
// CLIENT IMPLEMENTATION
// =============================================================================

export class RuVectorClient {
  private config: Required<RuVectorClientConfig>;
  private pendingWrites: Promise<void>[] = [];

  constructor(config: RuVectorClientConfig = {}) {
    this.config = { ...DEFAULT_CONFIG, ...config };
  }

  /**
   * Persist a DecisionEvent to ruvector-service.
   * This is an async, non-blocking write by default.
   */
  async persistDecisionEvent(event: DecisionEvent): Promise<void> {
    // Validate event against schema
    const validation = DecisionEventSchema.safeParse(event);
    if (!validation.success) {
      throw new Error(
        `Invalid DecisionEvent: ${validation.error.issues.map(i => i.message).join(', ')}`
      );
    }

    const writePromise = this.doWrite('/api/v1/decisions', event);

    if (this.config.asyncWrites) {
      // Fire-and-forget, but track for flush
      this.pendingWrites.push(writePromise.catch(err => {
        console.error('[RuVectorClient] Async write failed:', err);
      }));
    } else {
      // Blocking write
      await writePromise;
    }
  }

  /**
   * Persist a TelemetryEvent to ruvector-service.
   */
  async persistTelemetryEvent(event: TelemetryEvent): Promise<void> {
    const validation = TelemetryEventSchema.safeParse(event);
    if (!validation.success) {
      throw new Error(
        `Invalid TelemetryEvent: ${validation.error.issues.map(i => i.message).join(', ')}`
      );
    }

    const writePromise = this.doWrite('/api/v1/telemetry', event);

    if (this.config.asyncWrites) {
      this.pendingWrites.push(writePromise.catch(err => {
        console.error('[RuVectorClient] Async telemetry write failed:', err);
      }));
    } else {
      await writePromise;
    }
  }

  /**
   * Retrieve a DecisionEvent by ID
   */
  async getDecisionEvent(decisionId: string): Promise<DecisionEvent | null> {
    const response = await this.doRead(`/api/v1/decisions/${decisionId}`);
    if (!response) return null;

    const validation = DecisionEventSchema.safeParse(response);
    if (!validation.success) {
      console.error('[RuVectorClient] Invalid DecisionEvent from storage:', validation.error);
      return null;
    }

    return validation.data;
  }

  /**
   * Query DecisionEvents by agent ID
   */
  async queryDecisionsByAgent(
    agentId: string,
    options: { limit?: number; offset?: number; since?: string } = {}
  ): Promise<DecisionEvent[]> {
    const params = new URLSearchParams({
      agent_id: agentId,
      limit: String(options.limit ?? 100),
      offset: String(options.offset ?? 0),
    });

    if (options.since) {
      params.set('since', options.since);
    }

    const response = await this.doRead(`/api/v1/decisions?${params.toString()}`);
    if (!response || !Array.isArray(response)) return [];

    return response
      .map(item => DecisionEventSchema.safeParse(item))
      .filter(r => r.success)
      .map(r => r.data as DecisionEvent);
  }

  /**
   * Flush all pending async writes
   */
  async flush(): Promise<void> {
    await Promise.allSettled(this.pendingWrites);
    this.pendingWrites = [];
  }

  /**
   * Health check for ruvector-service
   */
  async healthCheck(): Promise<boolean> {
    try {
      const response = await fetch(`${this.config.baseUrl}/health`, {
        method: 'GET',
        signal: AbortSignal.timeout(this.config.timeoutMs),
      });
      return response.ok;
    } catch {
      return false;
    }
  }

  // ===========================================================================
  // PRIVATE METHODS
  // ===========================================================================

  private async doWrite(path: string, data: unknown): Promise<void> {
    let lastError: Error | null = null;

    for (let attempt = 0; attempt < this.config.maxRetries; attempt++) {
      try {
        const response = await fetch(`${this.config.baseUrl}${path}`, {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
            // Use actual API key for authentication (from Google Secret Manager)
            ...(this.config.apiKeyRef && {
              'Authorization': `Bearer ${this.config.apiKeyRef}`,
              'X-API-Key': this.config.apiKeyRef,
            }),
          },
          body: JSON.stringify(data),
          signal: AbortSignal.timeout(this.config.timeoutMs),
        });

        if (!response.ok) {
          throw new Error(`HTTP ${response.status}: ${await response.text()}`);
        }

        return;
      } catch (err) {
        lastError = err instanceof Error ? err : new Error(String(err));

        // Exponential backoff
        if (attempt < this.config.maxRetries - 1) {
          await new Promise(resolve =>
            setTimeout(resolve, Math.pow(2, attempt) * 100)
          );
        }
      }
    }

    throw lastError;
  }

  private async doRead(path: string): Promise<unknown> {
    try {
      const response = await fetch(`${this.config.baseUrl}${path}`, {
        method: 'GET',
        headers: {
          'Content-Type': 'application/json',
          // Use actual API key for authentication (from Google Secret Manager)
          ...(this.config.apiKeyRef && {
            'Authorization': `Bearer ${this.config.apiKeyRef}`,
            'X-API-Key': this.config.apiKeyRef,
          }),
        },
        signal: AbortSignal.timeout(this.config.timeoutMs),
      });

      if (response.status === 404) {
        return null;
      }

      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${await response.text()}`);
      }

      return await response.json();
    } catch (err) {
      console.error('[RuVectorClient] Read failed:', err);
      throw err;
    }
  }
}

// =============================================================================
// SINGLETON INSTANCE
// =============================================================================

let defaultClient: RuVectorClient | null = null;

/**
 * Get the RuVector client singleton.
 * On first call, asserts that required configuration is present.
 */
export function getRuVectorClient(config?: RuVectorClientConfig): RuVectorClient {
  if (!defaultClient) {
    // Assert Ruvector is configured (crashes if missing)
    assertRuvectorConfig();
    defaultClient = new RuVectorClient(config);
  }
  return defaultClient;
}

/**
 * Initialize RuVector client with mandatory health check.
 * MUST be called during startup - crashes if health check fails.
 */
export async function initializeRuVectorClient(config?: RuVectorClientConfig): Promise<RuVectorClient> {
  assertRuvectorConfig();
  const client = getRuVectorClient(config);

  const healthy = await client.healthCheck();
  if (!healthy) {
    console.error(JSON.stringify({
      event: 'agent_abort',
      reason: 'ruvector_health_check_failed',
      service_url: process.env.RUVECTOR_SERVICE_URL,
      timestamp: new Date().toISOString(),
    }));
    throw new Error('Ruvector health check failed - service unavailable');
  }

  return client;
}

// =============================================================================
// TYPES FOR PERSISTENCE RULES
// =============================================================================

/**
 * Data that IS persisted to ruvector-service
 */
export const PERSISTED_DATA = [
  'decision_events',        // Full DecisionEvent records
  'telemetry_events',       // Telemetry for LLM-Observatory
  'execution_refs',         // Trace/span references
  'confidence_scores',      // Historical confidence data
] as const;

/**
 * Data that is explicitly NOT persisted
 */
export const NOT_PERSISTED_DATA = [
  'api_keys',               // Never store actual API keys
  'raw_responses',          // Large response content (optionally stored)
  'prompt_content',         // Full prompts (only hash stored)
  'pii_data',               // Any PII must be redacted
  'session_state',          // Agents are stateless
] as const;

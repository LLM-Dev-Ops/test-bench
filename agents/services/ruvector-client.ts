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
   * @default process.env.RUVECTOR_SERVICE_URL || 'http://localhost:8080'
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
   * API key for authentication (reference, not actual key)
   */
  apiKeyRef?: string;

  /**
   * Enable async (fire-and-forget) writes
   * @default true
   */
  asyncWrites?: boolean;
}

const DEFAULT_CONFIG: Required<RuVectorClientConfig> = {
  baseUrl: process.env.RUVECTOR_SERVICE_URL || 'http://localhost:8080',
  timeoutMs: 5000,
  maxRetries: 3,
  apiKeyRef: process.env.RUVECTOR_API_KEY_REF || '',
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
            ...(this.config.apiKeyRef && {
              'X-API-Key-Ref': this.config.apiKeyRef,
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
          ...(this.config.apiKeyRef && {
            'X-API-Key-Ref': this.config.apiKeyRef,
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

export function getRuVectorClient(config?: RuVectorClientConfig): RuVectorClient {
  if (!defaultClient) {
    defaultClient = new RuVectorClient(config);
  }
  return defaultClient;
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

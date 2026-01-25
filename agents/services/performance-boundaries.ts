/**
 * Performance Boundaries - Conservative Defaults
 *
 * PHASE 1 - FOUNDATIONAL TOOLING (Layer 1)
 *
 * Enforces execution boundaries to prevent resource exhaustion:
 * - MAX_TOKENS: 800
 * - MAX_LATENCY_MS: 1500
 * - MAX_CALLS_PER_RUN: 2
 */

// =============================================================================
// PERFORMANCE LIMITS
// =============================================================================

export const PERFORMANCE_LIMITS = {
  MAX_TOKENS: 800,
  MAX_LATENCY_MS: 1500,
  MAX_CALLS_PER_RUN: 2,
} as const;

// =============================================================================
// EXECUTION TRACKER
// =============================================================================

export class ExecutionTracker {
  private startTime: number;
  private callCount: number = 0;
  private totalTokens: number = 0;
  private aborted: boolean = false;
  private abortReason?: string;

  constructor() {
    this.startTime = Date.now();
  }

  /**
   * Check if execution should continue or abort
   */
  shouldContinue(): boolean {
    if (this.aborted) return false;

    // Check latency
    const elapsed = Date.now() - this.startTime;
    if (elapsed >= PERFORMANCE_LIMITS.MAX_LATENCY_MS) {
      this.abort('max_latency_exceeded', {
        elapsed_ms: elapsed,
        limit_ms: PERFORMANCE_LIMITS.MAX_LATENCY_MS,
      });
      return false;
    }

    // Check call count
    if (this.callCount >= PERFORMANCE_LIMITS.MAX_CALLS_PER_RUN) {
      this.abort('max_calls_exceeded', {
        call_count: this.callCount,
        limit: PERFORMANCE_LIMITS.MAX_CALLS_PER_RUN,
      });
      return false;
    }

    return true;
  }

  /**
   * Record an LLM call
   */
  recordCall(tokens: number = 0): boolean {
    this.callCount++;
    this.totalTokens += tokens;

    // Check token limit
    if (this.totalTokens > PERFORMANCE_LIMITS.MAX_TOKENS) {
      this.abort('max_tokens_exceeded', {
        total_tokens: this.totalTokens,
        limit: PERFORMANCE_LIMITS.MAX_TOKENS,
      });
      return false;
    }

    return this.shouldContinue();
  }

  /**
   * Abort execution with reason
   */
  private abort(reason: string, details: Record<string, unknown>): void {
    this.aborted = true;
    this.abortReason = reason;

    console.error(JSON.stringify({
      event: 'agent_abort',
      reason,
      details,
      timestamp: new Date().toISOString(),
    }));
  }

  /**
   * Get execution metrics
   */
  getMetrics(): ExecutionMetrics {
    return {
      elapsed_ms: Date.now() - this.startTime,
      call_count: this.callCount,
      total_tokens: this.totalTokens,
      aborted: this.aborted,
      abort_reason: this.abortReason,
    };
  }

  /**
   * Check remaining budget
   */
  getRemainingBudget(): ExecutionBudget {
    const elapsed = Date.now() - this.startTime;
    return {
      remaining_latency_ms: Math.max(0, PERFORMANCE_LIMITS.MAX_LATENCY_MS - elapsed),
      remaining_calls: Math.max(0, PERFORMANCE_LIMITS.MAX_CALLS_PER_RUN - this.callCount),
      remaining_tokens: Math.max(0, PERFORMANCE_LIMITS.MAX_TOKENS - this.totalTokens),
    };
  }

  /**
   * Check if execution was aborted
   */
  isAborted(): boolean {
    return this.aborted;
  }
}

// =============================================================================
// TYPES
// =============================================================================

export interface ExecutionMetrics {
  elapsed_ms: number;
  call_count: number;
  total_tokens: number;
  aborted: boolean;
  abort_reason?: string;
}

export interface ExecutionBudget {
  remaining_latency_ms: number;
  remaining_calls: number;
  remaining_tokens: number;
}

// =============================================================================
// GUARD FUNCTIONS
// =============================================================================

/**
 * Create a guarded fetch that respects performance boundaries
 */
export function createBoundedFetch(tracker: ExecutionTracker) {
  return async function boundedFetch(
    url: string,
    options?: RequestInit
  ): Promise<Response> {
    if (!tracker.shouldContinue()) {
      throw new Error(`Execution aborted: ${tracker.getMetrics().abort_reason}`);
    }

    const budget = tracker.getRemainingBudget();
    const controller = new AbortController();
    const timeout = setTimeout(
      () => controller.abort(),
      budget.remaining_latency_ms
    );

    try {
      const response = await fetch(url, {
        ...options,
        signal: controller.signal,
      });

      return response;
    } finally {
      clearTimeout(timeout);
    }
  };
}

/**
 * Validate that request parameters respect token limits
 */
export function validateTokenBudget(
  requestedTokens: number,
  tracker: ExecutionTracker
): { valid: boolean; allowed_tokens: number } {
  const budget = tracker.getRemainingBudget();
  const allowed = Math.min(requestedTokens, budget.remaining_tokens);

  return {
    valid: allowed > 0,
    allowed_tokens: allowed,
  };
}

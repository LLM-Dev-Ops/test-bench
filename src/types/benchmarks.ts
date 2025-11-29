/**
 * Benchmark types and interfaces
 *
 * This module defines TypeScript types for benchmarking LLM providers.
 */

/**
 * Benchmark configuration
 */
export interface BenchmarkConfig {
  /** Number of concurrent requests */
  concurrency?: number;

  /** Whether to save individual responses */
  saveResponses?: boolean;

  /** Output file path for results */
  outputPath?: string;

  /** Maximum duration for the benchmark (milliseconds) */
  maxDuration?: number;

  /** Whether to show progress bar */
  showProgress?: boolean;
}

/**
 * Latency metrics in milliseconds
 */
export interface LatencyMetrics {
  /** Median latency (P50) */
  p50: number;

  /** 95th percentile latency */
  p95: number;

  /** 99th percentile latency */
  p99: number;

  /** Average latency */
  mean: number;

  /** Minimum latency */
  min: number;

  /** Maximum latency */
  max: number;

  /** Standard deviation */
  stdDev: number;
}

/**
 * Token usage metrics
 */
export interface TokenMetrics {
  /** Total prompt tokens across all requests */
  totalPromptTokens: number;

  /** Total completion tokens across all requests */
  totalCompletionTokens: number;

  /** Total tokens used */
  totalTokens: number;

  /** Average tokens per request */
  averageTokensPerRequest: number;

  /** Average tokens per second */
  tokensPerSecond: number;
}

/**
 * Cost metrics in USD
 */
export interface CostMetrics {
  /** Total estimated cost */
  totalCost: number;

  /** Average cost per request */
  averageCostPerRequest: number;

  /** Input token cost */
  inputCost: number;

  /** Output token cost */
  outputCost: number;
}

/**
 * Benchmark summary statistics
 */
export interface BenchmarkSummary {
  /** Total number of requests */
  totalRequests: number;

  /** Number of successful requests */
  successfulRequests: number;

  /** Number of failed requests */
  failedRequests: number;

  /** Success rate (0.0 - 1.0) */
  successRate: number;

  /** Latency metrics */
  latency: LatencyMetrics;

  /** Token usage metrics */
  tokens: TokenMetrics;

  /** Cost metrics (if available) */
  cost?: CostMetrics;

  /** Total duration in milliseconds */
  totalDuration: number;

  /** Requests per second */
  requestsPerSecond: number;
}

/**
 * Individual benchmark result
 */
export interface BenchmarkResult {
  /** Request ID */
  id: string;

  /** Prompt used */
  prompt: string;

  /** Response content */
  response?: string;

  /** Whether the request succeeded */
  success: boolean;

  /** Error message (if failed) */
  error?: string;

  /** Latency in milliseconds */
  latencyMs: number;

  /** Token usage */
  tokens?: {
    prompt: number;
    completion: number;
    total: number;
  };

  /** Estimated cost in USD */
  cost?: number;

  /** Timestamp */
  timestamp: string;
}

/**
 * Complete benchmark results
 */
export interface BenchmarkResults {
  /** Benchmark metadata */
  metadata: {
    /** Provider name */
    provider: string;

    /** Model name */
    model: string;

    /** Start timestamp */
    startTime: string;

    /** End timestamp */
    endTime: string;

    /** Configuration used */
    config: BenchmarkConfig;
  };

  /** Summary statistics */
  summary: BenchmarkSummary;

  /** Individual results (if saveResponses was enabled) */
  results?: BenchmarkResult[];
}

/**
 * Comparison result for multiple models
 */
export interface ComparisonResult {
  /** Comparison metadata */
  metadata: {
    /** Models compared */
    models: Array<{
      provider: string;
      model: string;
    }>;

    /** Timestamp */
    timestamp: string;

    /** Number of test cases */
    testCases: number;
  };

  /** Results for each model */
  modelResults: Array<{
    provider: string;
    model: string;
    summary: BenchmarkSummary;
  }>;

  /** Winner determination (if applicable) */
  winner?: {
    provider: string;
    model: string;
    reason: string;
  };
}

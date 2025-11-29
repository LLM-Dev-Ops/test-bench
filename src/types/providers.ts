/**
 * LLM Provider types and interfaces
 *
 * This module defines TypeScript types for interacting with various LLM providers.
 * These types map to the Rust core library's provider implementations.
 */

/**
 * Supported LLM provider names
 */
export type ProviderName =
  | 'openai'
  | 'anthropic'
  | 'google'
  | 'mistral'
  | 'azure-openai'
  | 'bedrock'
  | 'cohere'
  | 'groq'
  | 'huggingface'
  | 'ollama'
  | 'perplexity'
  | 'replicate'
  | 'together';

/**
 * Configuration for an LLM provider
 */
export interface ProviderConfig {
  /** Provider name */
  name: ProviderName;

  /** Environment variable name containing the API key */
  apiKeyEnv: string;

  /** Base URL for the provider's API */
  baseUrl: string;

  /** Default model to use if not specified in requests */
  defaultModel: string;

  /** Request timeout in seconds */
  timeoutSeconds?: number;

  /** Maximum number of retry attempts */
  maxRetries?: number;

  /** Whether this provider is enabled */
  enabled?: boolean;
}

/**
 * Completion request parameters
 */
export interface CompletionRequest {
  /** Model identifier (e.g., "gpt-4", "claude-opus-4") */
  model: string;

  /** Input prompt or message */
  prompt: string;

  /** Maximum tokens to generate */
  maxTokens?: number;

  /** Sampling temperature (0.0 - 2.0) */
  temperature?: number;

  /** Nucleus sampling parameter (0.0 - 1.0) */
  topP?: number;

  /** Sequences where the model should stop generating */
  stop?: string[];

  /** Enable streaming responses */
  stream?: boolean;
}

/**
 * Reason why the model stopped generating
 */
export type FinishReason = 'stop' | 'length' | 'content_filter' | 'tool_calls' | 'error';

/**
 * Token usage statistics
 */
export interface TokenUsage {
  /** Number of tokens in the prompt */
  promptTokens: number;

  /** Number of tokens in the completion */
  completionTokens: number;

  /** Total tokens used (prompt + completion) */
  totalTokens: number;
}

/**
 * Completion response from an LLM provider
 */
export interface CompletionResponse {
  /** The generated text content */
  content: string;

  /** Model used for generation */
  model: string;

  /** Why the model stopped generating */
  finishReason: FinishReason;

  /** Token usage statistics */
  usage: TokenUsage;

  /** Response timestamp */
  timestamp: string;

  /** Response latency in milliseconds */
  latencyMs: number;

  /** Estimated cost in USD (if available) */
  estimatedCost?: number;
}

/**
 * Streaming chunk from a completion stream
 */
export interface CompletionChunk {
  /** Incremental text content */
  content: string;

  /** Whether this is the final chunk */
  done: boolean;

  /** Finish reason (only present in final chunk) */
  finishReason?: FinishReason;

  /** Token usage (only present in final chunk) */
  usage?: TokenUsage;
}

/**
 * Model information
 */
export interface ModelInfo {
  /** Model identifier */
  id: string;

  /** Provider name */
  provider: ProviderName;

  /** Human-readable model name */
  name: string;

  /** Model description */
  description?: string;

  /** Context window size in tokens */
  contextWindow?: number;

  /** Maximum output tokens */
  maxOutputTokens?: number;

  /** Input cost per 1M tokens (USD) */
  inputCostPerMillion?: number;

  /** Output cost per 1M tokens (USD) */
  outputCostPerMillion?: number;

  /** Model capabilities */
  capabilities?: {
    streaming?: boolean;
    vision?: boolean;
    audio?: boolean;
    functionCalling?: boolean;
    json?: boolean;
  };
}

/**
 * Provider error
 */
export interface ProviderError {
  /** Error type */
  type: 'rate_limit' | 'authentication' | 'network' | 'invalid_request' | 'provider_error' | 'timeout';

  /** Error message */
  message: string;

  /** HTTP status code (if applicable) */
  statusCode?: number;

  /** Provider-specific error details */
  details?: unknown;
}

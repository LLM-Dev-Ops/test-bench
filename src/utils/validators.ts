/**
 * Validation utilities
 *
 * Provides validation functions for SDK inputs.
 */

import { z } from 'zod';
import type {
  CompletionRequest,
  ProviderConfig,
  BenchmarkConfig,
  EvaluationConfig,
} from '../types/index.js';

/**
 * Zod schema for provider configuration
 */
const providerConfigSchema = z.object({
  name: z.enum([
    'openai',
    'anthropic',
    'google',
    'mistral',
    'azure-openai',
    'bedrock',
    'cohere',
    'groq',
    'huggingface',
    'ollama',
    'perplexity',
    'replicate',
    'together',
  ]),
  apiKeyEnv: z.string().min(1),
  baseUrl: z.string().url(),
  defaultModel: z.string().min(1),
  timeoutSeconds: z.number().positive().optional(),
  maxRetries: z.number().int().nonnegative().optional(),
  enabled: z.boolean().optional(),
});

/**
 * Zod schema for completion request
 */
const completionRequestSchema = z.object({
  model: z.string().min(1),
  prompt: z.string().min(1),
  maxTokens: z.number().int().positive().optional(),
  temperature: z.number().min(0).max(2).optional(),
  topP: z.number().min(0).max(1).optional(),
  stop: z.array(z.string()).optional(),
  stream: z.boolean().optional(),
});

/**
 * Zod schema for benchmark configuration
 */
const benchmarkConfigSchema = z.object({
  concurrency: z.number().int().positive().optional(),
  saveResponses: z.boolean().optional(),
  outputPath: z.string().optional(),
  maxDuration: z.number().int().positive().optional(),
  showProgress: z.boolean().optional(),
});

/**
 * Zod schema for evaluation configuration
 */
const evaluationConfigSchema = z.object({
  evaluators: z.array(
    z.enum([
      'perplexity',
      'coherence',
      'relevance',
      'faithfulness',
      'llm-as-judge',
      'readability',
      'sentiment',
      'toxicity',
      'pii-detection',
      'custom',
    ])
  ),
  referenceText: z.string().optional(),
  context: z.array(z.string()).optional(),
  customCriteria: z.array(z.string()).optional(),
  judgeModel: z.string().optional(),
  parallel: z.boolean().optional(),
  cacheResults: z.boolean().optional(),
});

/**
 * Validate provider configuration
 *
 * @param config - Provider configuration to validate
 * @throws {z.ZodError} If validation fails
 */
export function validateProviderConfig(config: unknown): asserts config is ProviderConfig {
  providerConfigSchema.parse(config);
}

/**
 * Validate completion request
 *
 * @param request - Completion request to validate
 * @throws {z.ZodError} If validation fails
 */
export function validateCompletionRequest(request: unknown): asserts request is CompletionRequest {
  completionRequestSchema.parse(request);
}

/**
 * Validate benchmark configuration
 *
 * @param config - Benchmark configuration to validate
 * @throws {z.ZodError} If validation fails
 */
export function validateBenchmarkConfig(config: unknown): asserts config is BenchmarkConfig {
  benchmarkConfigSchema.parse(config);
}

/**
 * Validate evaluation configuration
 *
 * @param config - Evaluation configuration to validate
 * @throws {z.ZodError} If validation fails
 */
export function validateEvaluationConfig(config: unknown): asserts config is EvaluationConfig {
  evaluationConfigSchema.parse(config);
}

/**
 * Check if a value is a valid model identifier
 *
 * @param model - Model identifier to check
 * @returns True if valid, false otherwise
 */
export function isValidModel(model: string): boolean {
  return model.length > 0 && /^[a-z0-9-_.]+$/i.test(model);
}

/**
 * Check if a value is a valid provider name
 *
 * @param provider - Provider name to check
 * @returns True if valid, false otherwise
 */
export function isValidProvider(provider: string): boolean {
  const validProviders = [
    'openai',
    'anthropic',
    'google',
    'mistral',
    'azure-openai',
    'bedrock',
    'cohere',
    'groq',
    'huggingface',
    'ollama',
    'perplexity',
    'replicate',
    'together',
  ];
  return validProviders.includes(provider);
}

/**
 * Provider-specific client implementations
 *
 * Provides convenience methods for working with specific LLM providers.
 */

import type {
  CompletionRequest,
  CompletionResponse,
  ProviderName,
  ModelInfo,
} from '../types/index.js';

import { LLMTestBench } from './llm-test-bench.js';

/**
 * Base provider client
 */
export abstract class ProviderClient {
  protected ltb: LLMTestBench;
  protected providerName: ProviderName;

  constructor(ltb: LLMTestBench, providerName: ProviderName) {
    this.ltb = ltb;
    this.providerName = providerName;
  }

  /**
   * Get available models for this provider
   */
  async listModels(): Promise<ModelInfo[]> {
    return this.ltb.listModels(this.providerName);
  }

  /**
   * Complete a prompt using the provider's default model
   */
  async complete(prompt: string, options?: Partial<CompletionRequest>): Promise<CompletionResponse> {
    const models = await this.listModels();
    if (models.length === 0) {
      throw new Error(`No models available for provider: ${this.providerName}`);
    }

    const firstModel = models[0];
    if (!firstModel) {
      throw new Error(`Unable to retrieve default model for provider: ${this.providerName}`);
    }

    const request: CompletionRequest = {
      model: firstModel.id,
      prompt,
      ...options,
    };

    return this.ltb.complete(request);
  }
}

/**
 * OpenAI provider client
 */
export class OpenAIClient extends ProviderClient {
  constructor(ltb: LLMTestBench) {
    super(ltb, 'openai');
  }

  /**
   * Use GPT-4
   */
  async gpt4(prompt: string, options?: Partial<CompletionRequest>): Promise<CompletionResponse> {
    return this.ltb.complete({
      model: 'gpt-4',
      prompt,
      ...options,
    });
  }

  /**
   * Use GPT-4 Turbo
   */
  async gpt4Turbo(prompt: string, options?: Partial<CompletionRequest>): Promise<CompletionResponse> {
    return this.ltb.complete({
      model: 'gpt-4-turbo',
      prompt,
      ...options,
    });
  }

  /**
   * Use GPT-4o
   */
  async gpt4o(prompt: string, options?: Partial<CompletionRequest>): Promise<CompletionResponse> {
    return this.ltb.complete({
      model: 'gpt-4o',
      prompt,
      ...options,
    });
  }

  /**
   * Use GPT-3.5 Turbo
   */
  async gpt35Turbo(prompt: string, options?: Partial<CompletionRequest>): Promise<CompletionResponse> {
    return this.ltb.complete({
      model: 'gpt-3.5-turbo',
      prompt,
      ...options,
    });
  }
}

/**
 * Anthropic provider client
 */
export class AnthropicClient extends ProviderClient {
  constructor(ltb: LLMTestBench) {
    super(ltb, 'anthropic');
  }

  /**
   * Use Claude Opus 4
   */
  async claudeOpus4(prompt: string, options?: Partial<CompletionRequest>): Promise<CompletionResponse> {
    return this.ltb.complete({
      model: 'claude-opus-4',
      prompt,
      ...options,
    });
  }

  /**
   * Use Claude Sonnet 4.5
   */
  async claudeSonnet45(prompt: string, options?: Partial<CompletionRequest>): Promise<CompletionResponse> {
    return this.ltb.complete({
      model: 'claude-sonnet-4.5',
      prompt,
      ...options,
    });
  }

  /**
   * Use Claude 3.5 Sonnet
   */
  async claude35Sonnet(prompt: string, options?: Partial<CompletionRequest>): Promise<CompletionResponse> {
    return this.ltb.complete({
      model: 'claude-3-5-sonnet-latest',
      prompt,
      ...options,
    });
  }

  /**
   * Use Claude 3.5 Haiku
   */
  async claude35Haiku(prompt: string, options?: Partial<CompletionRequest>): Promise<CompletionResponse> {
    return this.ltb.complete({
      model: 'claude-3-5-haiku-latest',
      prompt,
      ...options,
    });
  }
}

/**
 * Google provider client
 */
export class GoogleClient extends ProviderClient {
  constructor(ltb: LLMTestBench) {
    super(ltb, 'google');
  }

  /**
   * Use Gemini 2.5 Pro
   */
  async gemini25Pro(prompt: string, options?: Partial<CompletionRequest>): Promise<CompletionResponse> {
    return this.ltb.complete({
      model: 'gemini-2.5-pro',
      prompt,
      ...options,
    });
  }

  /**
   * Use Gemini 1.5 Pro
   */
  async gemini15Pro(prompt: string, options?: Partial<CompletionRequest>): Promise<CompletionResponse> {
    return this.ltb.complete({
      model: 'gemini-1.5-pro',
      prompt,
      ...options,
    });
  }

  /**
   * Use Gemini 1.5 Flash
   */
  async gemini15Flash(prompt: string, options?: Partial<CompletionRequest>): Promise<CompletionResponse> {
    return this.ltb.complete({
      model: 'gemini-1.5-flash',
      prompt,
      ...options,
    });
  }
}

/**
 * Factory for creating provider clients
 */
export class ProviderClientFactory {
  constructor(private ltb: LLMTestBench) {}

  /**
   * Get OpenAI client
   */
  openai(): OpenAIClient {
    return new OpenAIClient(this.ltb);
  }

  /**
   * Get Anthropic client
   */
  anthropic(): AnthropicClient {
    return new AnthropicClient(this.ltb);
  }

  /**
   * Get Google client
   */
  google(): GoogleClient {
    return new GoogleClient(this.ltb);
  }
}

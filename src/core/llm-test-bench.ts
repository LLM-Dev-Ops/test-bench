/**
 * Main SDK class for LLM Test Bench
 *
 * Provides a programmatic interface to the LLM Test Bench CLI.
 */

import type {
  SDKConfig,
  CompletionRequest,
  CompletionResponse,
  BenchmarkConfig,
  BenchmarkResults,
  ComparisonResult,
  EvaluationConfig,
  CombinedEvaluationResults,
  ProviderName,
  ModelInfo,
} from '../types/index.js';
import { executeCLI, findCLIPath } from '../utils/cli-executor.js';
import {
  validateCompletionRequest,
  validateBenchmarkConfig,
  validateEvaluationConfig,
  isValidModel,
  isValidProvider,
} from '../utils/validators.js';

/**
 * LLM Test Bench SDK
 *
 * Main entry point for programmatic access to LLM Test Bench functionality.
 *
 * @example
 * ```typescript
 * import { LLMTestBench } from 'llm-test-bench';
 *
 * const ltb = new LLMTestBench({
 *   verbose: true,
 *   timeout: 30000,
 * });
 *
 * // Run a benchmark
 * const results = await ltb.benchmark({
 *   provider: 'openai',
 *   model: 'gpt-4',
 *   prompts: ['Explain quantum computing'],
 * });
 *
 * console.log(results.summary);
 * ```
 */
export class LLMTestBench {
  private cliPath: string;
  private config: Required<SDKConfig>;

  /**
   * Create a new LLM Test Bench SDK instance
   *
   * @param config - SDK configuration
   * @throws {Error} If CLI binary cannot be found
   */
  constructor(config: SDKConfig = {}) {
    const resolvedCliPath = config.cliPath ?? findCLIPath();
    if (!resolvedCliPath) {
      throw new Error(
        'LLM Test Bench CLI not found. Please install it via cargo or npm, or provide cliPath in config.'
      );
    }

    this.cliPath = resolvedCliPath;
    this.config = {
      cliPath: resolvedCliPath,
      workingDir: config.workingDir ?? process.cwd(),
      verbose: config.verbose ?? false,
      env: config.env ?? {},
      timeout: config.timeout ?? 120000, // 2 minutes default
    };
  }

  /**
   * Get the SDK version
   *
   * @returns Promise resolving to version string
   */
  async version(): Promise<string> {
    const result = await executeCLI<{ version: string }>(this.cliPath, {
      args: ['--version'],
      cwd: this.config.workingDir,
      env: this.config.env,
      timeout: 5000,
    });

    if (!result.success) {
      throw new Error(`Failed to get version: ${result.error}`);
    }

    // Parse version from output like "llm-test-bench 0.1.2"
    const match = result.stdout.match(/(\d+\.\d+\.\d+)/);
    const version = match?.[1];
    return version ?? 'unknown';
  }

  /**
   * List all available models
   *
   * @param provider - Optional provider filter
   * @returns Promise resolving to array of model information
   */
  async listModels(provider?: ProviderName): Promise<ModelInfo[]> {
    const args = ['models', 'list'];
    if (provider) {
      if (!isValidProvider(provider)) {
        throw new Error(`Invalid provider: ${provider}`);
      }
      args.push('--provider', provider);
    }
    args.push('--json');

    const result = await executeCLI<{ models: ModelInfo[] }>(this.cliPath, {
      args,
      cwd: this.config.workingDir,
      env: this.config.env,
      timeout: 10000,
      parseJson: true,
    });

    if (!result.success || !result.data) {
      throw new Error(`Failed to list models: ${result.error}`);
    }

    return result.data.models;
  }

  /**
   * Run a benchmark
   *
   * @param options - Benchmark options
   * @returns Promise resolving to benchmark results
   */
  async benchmark(options: {
    provider: ProviderName;
    model: string;
    prompts: string[];
    config?: BenchmarkConfig;
  }): Promise<BenchmarkResults> {
    if (!isValidProvider(options.provider)) {
      throw new Error(`Invalid provider: ${options.provider}`);
    }
    if (!isValidModel(options.model)) {
      throw new Error(`Invalid model: ${options.model}`);
    }
    if (!options.prompts || options.prompts.length === 0) {
      throw new Error('At least one prompt is required');
    }
    if (options.config) {
      validateBenchmarkConfig(options.config);
    }

    const args = [
      'bench',
      '--provider',
      options.provider,
      '--model',
      options.model,
      '--json',
    ];

    // Add prompts
    for (const prompt of options.prompts) {
      args.push('--prompt', prompt);
    }

    // Add config options
    if (options.config) {
      if (options.config.concurrency) {
        args.push('--concurrency', options.config.concurrency.toString());
      }
      if (options.config.saveResponses) {
        args.push('--save-responses');
      }
      if (options.config.outputPath) {
        args.push('--output', options.config.outputPath);
      }
      if (options.config.maxDuration) {
        args.push('--max-duration', options.config.maxDuration.toString());
      }
      if (options.config.showProgress === false) {
        args.push('--no-progress');
      }
    }

    const result = await executeCLI<BenchmarkResults>(this.cliPath, {
      args,
      cwd: this.config.workingDir,
      env: this.config.env,
      timeout: this.config.timeout,
      parseJson: true,
    });

    if (!result.success || !result.data) {
      throw new Error(`Benchmark failed: ${result.error}`);
    }

    return result.data;
  }

  /**
   * Compare multiple models
   *
   * @param options - Comparison options
   * @returns Promise resolving to comparison results
   */
  async compare(options: {
    models: Array<{ provider: ProviderName; model: string }>;
    prompts: string[];
    config?: BenchmarkConfig;
  }): Promise<ComparisonResult> {
    if (!options.models || options.models.length < 2) {
      throw new Error('At least two models are required for comparison');
    }
    if (!options.prompts || options.prompts.length === 0) {
      throw new Error('At least one prompt is required');
    }

    for (const { provider, model } of options.models) {
      if (!isValidProvider(provider)) {
        throw new Error(`Invalid provider: ${provider}`);
      }
      if (!isValidModel(model)) {
        throw new Error(`Invalid model: ${model}`);
      }
    }

    if (options.config) {
      validateBenchmarkConfig(options.config);
    }

    const args = ['compare', '--json'];

    // Add models
    const modelSpecs = options.models.map((m) => `${m.provider}:${m.model}`).join(',');
    args.push('--models', modelSpecs);

    // Add prompts
    for (const prompt of options.prompts) {
      args.push('--prompt', prompt);
    }

    // Add config options
    if (options.config) {
      if (options.config.concurrency) {
        args.push('--concurrency', options.config.concurrency.toString());
      }
      if (options.config.outputPath) {
        args.push('--output', options.config.outputPath);
      }
    }

    const result = await executeCLI<ComparisonResult>(this.cliPath, {
      args,
      cwd: this.config.workingDir,
      env: this.config.env,
      timeout: this.config.timeout,
      parseJson: true,
    });

    if (!result.success || !result.data) {
      throw new Error(`Comparison failed: ${result.error}`);
    }

    return result.data;
  }

  /**
   * Evaluate a response
   *
   * @param text - Text to evaluate
   * @param config - Evaluation configuration
   * @returns Promise resolving to evaluation results
   */
  async evaluate(
    text: string,
    config: EvaluationConfig
  ): Promise<CombinedEvaluationResults> {
    if (!text || text.trim().length === 0) {
      throw new Error('Text to evaluate cannot be empty');
    }

    validateEvaluationConfig(config);

    const args = ['eval', '--text', text, '--json'];

    // Add evaluators
    for (const evaluator of config.evaluators) {
      args.push('--evaluator', evaluator);
    }

    // Add optional parameters
    if (config.referenceText) {
      args.push('--reference', config.referenceText);
    }
    if (config.context && config.context.length > 0) {
      for (const ctx of config.context) {
        args.push('--context', ctx);
      }
    }
    if (config.judgeModel) {
      args.push('--judge-model', config.judgeModel);
    }
    if (config.parallel) {
      args.push('--parallel');
    }
    if (config.cacheResults === false) {
      args.push('--no-cache');
    }

    const result = await executeCLI<CombinedEvaluationResults>(this.cliPath, {
      args,
      cwd: this.config.workingDir,
      env: this.config.env,
      timeout: this.config.timeout,
      parseJson: true,
    });

    if (!result.success || !result.data) {
      throw new Error(`Evaluation failed: ${result.error}`);
    }

    return result.data;
  }

  /**
   * Get a completion from a model
   *
   * @param request - Completion request
   * @returns Promise resolving to completion response
   */
  async complete(request: CompletionRequest): Promise<CompletionResponse> {
    validateCompletionRequest(request);

    const args = ['complete', '--model', request.model, '--prompt', request.prompt, '--json'];

    if (request.maxTokens) {
      args.push('--max-tokens', request.maxTokens.toString());
    }
    if (request.temperature !== undefined) {
      args.push('--temperature', request.temperature.toString());
    }
    if (request.topP !== undefined) {
      args.push('--top-p', request.topP.toString());
    }
    if (request.stop && request.stop.length > 0) {
      for (const stop of request.stop) {
        args.push('--stop', stop);
      }
    }

    const result = await executeCLI<CompletionResponse>(this.cliPath, {
      args,
      cwd: this.config.workingDir,
      env: this.config.env,
      timeout: this.config.timeout,
      parseJson: true,
    });

    if (!result.success || !result.data) {
      throw new Error(`Completion failed: ${result.error}`);
    }

    return result.data;
  }

  /**
   * Optimize model selection
   *
   * @param options - Optimization options
   * @returns Promise resolving to recommended model
   */
  async optimize(options: {
    prompts: string[];
    metric: 'latency' | 'cost' | 'quality';
    maxCost?: number;
    minQuality?: number;
  }): Promise<{ provider: ProviderName; model: string; reason: string }> {
    if (!options.prompts || options.prompts.length === 0) {
      throw new Error('At least one prompt is required');
    }

    const args = ['optimize', '--metric', options.metric, '--json'];

    for (const prompt of options.prompts) {
      args.push('--prompt', prompt);
    }

    if (options.maxCost !== undefined) {
      args.push('--max-cost', options.maxCost.toString());
    }
    if (options.minQuality !== undefined) {
      args.push('--min-quality', options.minQuality.toString());
    }

    const result = await executeCLI<{
      provider: ProviderName;
      model: string;
      reason: string;
    }>(this.cliPath, {
      args,
      cwd: this.config.workingDir,
      env: this.config.env,
      timeout: this.config.timeout,
      parseJson: true,
    });

    if (!result.success || !result.data) {
      throw new Error(`Optimization failed: ${result.error}`);
    }

    return result.data;
  }
}

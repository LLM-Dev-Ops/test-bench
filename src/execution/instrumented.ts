/**
 * Instrumented wrapper for LLMTestBench.
 *
 * Wraps every externally-invoked operation so that it:
 * 1. Requires an ExecutionContext (rejects if parent_span_id is missing)
 * 2. Creates a repo-level span on entry
 * 3. Creates agent-level spans for each operation
 * 4. Attaches artifacts to agent spans
 * 5. Returns ExecutionResult with full span hierarchy
 *
 * This ensures the repository NEVER executes silently.
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
import { LLMTestBench } from '../core/llm-test-bench.js';
import type { ExecutionContext, ExecutionResult } from './types.js';
import type { ExecutionPolicy } from './policy.js';
import { SpanManager } from './span-manager.js';
import { PolicyGate } from './policy-gate.js';

/**
 * Instrumented LLM Test Bench that emits execution spans.
 *
 * This is the primary entry point when this repo operates as a
 * Foundational Execution Unit within the Agentics system.
 *
 * @example
 * ```typescript
 * const bench = new InstrumentedTestBench({ verbose: true });
 *
 * const result = await bench.benchmark(
 *   { execution_id: 'exec-123', parent_span_id: 'core-span-456' },
 *   { provider: 'openai', model: 'gpt-4', prompts: ['Hello'] }
 * );
 *
 * // result.repo_span - the repo-level span
 * // result.agent_spans - agent spans (benchmark-runner)
 * // result.artifacts - benchmark output artifacts
 * // result.data - BenchmarkResults
 * // result.valid - true if all invariants satisfied
 * ```
 */
export class InstrumentedTestBench {
  private readonly inner: LLMTestBench;
  private readonly gate: PolicyGate;

  constructor(config: SDKConfig = {}, policy?: ExecutionPolicy) {
    this.inner = new LLMTestBench(config);
    this.gate = new PolicyGate(policy);
  }

  /**
   * Get version with execution span tracking.
   */
  async version(ctx: ExecutionContext): Promise<ExecutionResult<string>> {
    const { mgr } = this.gate.enforce('benchmark', ctx);
    const agent = mgr.startAgent('version-resolver');

    try {
      const version = await this.inner.version();
      mgr.endAgent(agent.span_id, 'completed');
      return mgr.finalize(version);
    } catch (err) {
      const reason = err instanceof Error ? err.message : String(err);
      mgr.endAgent(agent.span_id, 'failed', [reason]);
      return mgr.finalize<string>(undefined, 'failed');
    }
  }

  /**
   * List models with execution span tracking.
   */
  async listModels(
    ctx: ExecutionContext,
    provider?: ProviderName
  ): Promise<ExecutionResult<ModelInfo[]>> {
    const { mgr } = this.gate.enforce('benchmark', ctx);
    const agent = mgr.startAgent('model-lister');

    try {
      const models = await this.inner.listModels(provider);
      mgr.endAgent(agent.span_id, 'completed');
      return mgr.finalize(models);
    } catch (err) {
      const reason = err instanceof Error ? err.message : String(err);
      mgr.endAgent(agent.span_id, 'failed', [reason]);
      return mgr.finalize<ModelInfo[]>(undefined, 'failed');
    }
  }

  /**
   * Run benchmark with execution span tracking.
   * Emits agent spans for validation, execution, and result collection.
   */
  async benchmark(
    ctx: ExecutionContext,
    options: {
      provider: ProviderName;
      model: string;
      prompts: string[];
      config?: BenchmarkConfig;
    }
  ): Promise<ExecutionResult<BenchmarkResults>> {
    const { mgr } = this.gate.enforce('benchmark', ctx);

    // Agent 1: Input validation
    const validator = mgr.startAgent('input-validator');
    try {
      if (!options.provider || !options.model || !options.prompts?.length) {
        throw new Error('Missing required benchmark parameters');
      }
      mgr.endAgent(validator.span_id, 'completed');
    } catch (err) {
      const reason = err instanceof Error ? err.message : String(err);
      mgr.endAgent(validator.span_id, 'failed', [reason]);
      return mgr.finalize<BenchmarkResults>(undefined, 'failed');
    }

    // Agent 2: Benchmark execution
    const runner = mgr.startAgent('benchmark-runner');
    try {
      const results = await this.inner.benchmark(options);

      // Attach result artifact to the runner agent
      mgr.addArtifact({
        reference: `benchmark-${mgr.repoSpanId}-results.json`,
        artifact_type: 'benchmark_results',
        agent_span_id: runner.span_id,
        metadata: {
          provider: options.provider,
          model: options.model,
          prompt_count: options.prompts.length,
        },
      });

      mgr.endAgent(runner.span_id, 'completed');
      return mgr.finalize(results);
    } catch (err) {
      const reason = err instanceof Error ? err.message : String(err);
      mgr.endAgent(runner.span_id, 'failed', [reason]);
      return mgr.finalize<BenchmarkResults>(undefined, 'failed');
    }
  }

  /**
   * Compare models with execution span tracking.
   */
  async compare(
    ctx: ExecutionContext,
    options: {
      models: Array<{ provider: ProviderName; model: string }>;
      prompts: string[];
      config?: BenchmarkConfig;
    }
  ): Promise<ExecutionResult<ComparisonResult>> {
    const { mgr } = this.gate.enforce('compare', ctx);
    const agent = mgr.startAgent('model-comparator');

    try {
      const result = await this.inner.compare(options);

      mgr.addArtifact({
        reference: `comparison-${mgr.repoSpanId}-results.json`,
        artifact_type: 'comparison_results',
        agent_span_id: agent.span_id,
        metadata: {
          model_count: options.models.length,
          prompt_count: options.prompts.length,
        },
      });

      mgr.endAgent(agent.span_id, 'completed');
      return mgr.finalize(result);
    } catch (err) {
      const reason = err instanceof Error ? err.message : String(err);
      mgr.endAgent(agent.span_id, 'failed', [reason]);
      return mgr.finalize<ComparisonResult>(undefined, 'failed');
    }
  }

  /**
   * Evaluate text with execution span tracking.
   */
  async evaluate(
    ctx: ExecutionContext,
    text: string,
    config: EvaluationConfig
  ): Promise<ExecutionResult<CombinedEvaluationResults>> {
    const { mgr } = this.gate.enforce('evaluate', ctx);
    const agent = mgr.startAgent('evaluator');

    try {
      const result = await this.inner.evaluate(text, config);

      mgr.addArtifact({
        reference: `evaluation-${mgr.repoSpanId}-results.json`,
        artifact_type: 'evaluation_results',
        agent_span_id: agent.span_id,
        metadata: {
          evaluators: config.evaluators,
          text_length: text.length,
        },
      });

      mgr.endAgent(agent.span_id, 'completed');
      return mgr.finalize(result);
    } catch (err) {
      const reason = err instanceof Error ? err.message : String(err);
      mgr.endAgent(agent.span_id, 'failed', [reason]);
      return mgr.finalize<CombinedEvaluationResults>(undefined, 'failed');
    }
  }

  /**
   * Get completion with execution span tracking.
   */
  async complete(
    ctx: ExecutionContext,
    request: CompletionRequest
  ): Promise<ExecutionResult<CompletionResponse>> {
    const { mgr } = this.gate.enforce('complete', ctx);
    const agent = mgr.startAgent('completion-agent');

    try {
      const result = await this.inner.complete(request);

      mgr.addArtifact({
        reference: `completion-${mgr.repoSpanId}-response.json`,
        artifact_type: 'completion_response',
        agent_span_id: agent.span_id,
        metadata: {
          model: request.model,
        },
      });

      mgr.endAgent(agent.span_id, 'completed');
      return mgr.finalize(result);
    } catch (err) {
      const reason = err instanceof Error ? err.message : String(err);
      mgr.endAgent(agent.span_id, 'failed', [reason]);
      return mgr.finalize<CompletionResponse>(undefined, 'failed');
    }
  }

  /**
   * Optimize model selection with execution span tracking.
   */
  async optimize(
    ctx: ExecutionContext,
    options: {
      prompts: string[];
      metric: 'latency' | 'cost' | 'quality';
      maxCost?: number;
      minQuality?: number;
    }
  ): Promise<
    ExecutionResult<{ provider: ProviderName; model: string; reason: string }>
  > {
    const { mgr } = this.gate.enforce('optimize', ctx);
    const agent = mgr.startAgent('optimizer');

    try {
      const result = await this.inner.optimize(options);

      mgr.addArtifact({
        reference: `optimization-${mgr.repoSpanId}-recommendation.json`,
        artifact_type: 'optimization_recommendation',
        agent_span_id: agent.span_id,
        metadata: {
          metric: options.metric,
          prompt_count: options.prompts.length,
        },
      });

      mgr.endAgent(agent.span_id, 'completed');
      return mgr.finalize(result);
    } catch (err) {
      const reason = err instanceof Error ? err.message : String(err);
      mgr.endAgent(agent.span_id, 'failed', [reason]);
      return mgr.finalize<{
        provider: ProviderName;
        model: string;
        reason: string;
      }>(undefined, 'failed');
    }
  }
}

/**
 * Execute an arbitrary async operation with full execution span instrumentation.
 *
 * Use this for ad-hoc operations that need to participate in the
 * execution graph without going through InstrumentedTestBench.
 *
 * @example
 * ```typescript
 * const result = await executeWithSpans(
 *   { execution_id: 'exec-1', parent_span_id: 'core-span-1' },
 *   async (mgr) => {
 *     const agent = mgr.startAgent('custom-agent');
 *     const data = await doWork();
 *     mgr.addArtifact({
 *       reference: 'output.json',
 *       artifact_type: 'custom',
 *       agent_span_id: agent.span_id,
 *     });
 *     mgr.endAgent(agent.span_id, 'completed');
 *     return data;
 *   }
 * );
 * ```
 */
export async function executeWithSpans<T>(
  ctx: ExecutionContext,
  fn: (mgr: SpanManager) => Promise<T>
): Promise<ExecutionResult<T>> {
  const mgr = SpanManager.create(ctx);

  try {
    const data = await fn(mgr);
    return mgr.finalize(data);
  } catch (err) {
    const reason = err instanceof Error ? err.message : String(err);
    return mgr.finalizeFailure([reason]);
  }
}

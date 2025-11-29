/**
 * Evaluator utilities and helpers
 *
 * Provides convenient functions for evaluating LLM responses.
 */

import { LLMTestBench } from '../core/llm-test-bench.js';
import type {
  EvaluationConfig,
  CombinedEvaluationResults,
  EvaluatorType,
} from '../types/index.js';

/**
 * Evaluator helper class
 */
export class Evaluator {
  constructor(private ltb: LLMTestBench) {}

  /**
   * Evaluate text with all available evaluators
   */
  async evaluateAll(text: string, options?: {
    referenceText?: string;
    context?: string[];
  }): Promise<CombinedEvaluationResults> {
    const config: EvaluationConfig = {
      evaluators: [
        'perplexity',
        'coherence',
        'relevance',
        'faithfulness',
        'readability',
        'sentiment',
        'toxicity',
        'pii-detection',
      ],
      ...options,
      parallel: true,
      cacheResults: true,
    };

    return this.ltb.evaluate(text, config);
  }

  /**
   * Evaluate text quality (perplexity, coherence, readability)
   */
  async evaluateQuality(text: string): Promise<CombinedEvaluationResults> {
    const config: EvaluationConfig = {
      evaluators: ['perplexity', 'coherence', 'readability'],
      parallel: true,
    };

    return this.ltb.evaluate(text, config);
  }

  /**
   * Evaluate safety (toxicity, PII detection)
   */
  async evaluateSafety(text: string): Promise<CombinedEvaluationResults> {
    const config: EvaluationConfig = {
      evaluators: ['toxicity', 'pii-detection'],
      parallel: true,
    };

    return this.ltb.evaluate(text, config);
  }

  /**
   * Evaluate accuracy (relevance, faithfulness)
   */
  async evaluateAccuracy(
    text: string,
    options: {
      referenceText?: string;
      context?: string[];
    }
  ): Promise<CombinedEvaluationResults> {
    const config: EvaluationConfig = {
      evaluators: ['relevance', 'faithfulness'],
      ...options,
      parallel: true,
    };

    return this.ltb.evaluate(text, config);
  }

  /**
   * Use LLM as judge to evaluate text
   */
  async llmAsJudge(
    text: string,
    options: {
      criteria: string[];
      judgeModel?: string;
      referenceText?: string;
    }
  ): Promise<CombinedEvaluationResults> {
    const config: EvaluationConfig = {
      evaluators: ['llm-as-judge'],
      customCriteria: options.criteria,
      judgeModel: options.judgeModel ?? 'gpt-4',
      referenceText: options.referenceText,
    };

    return this.ltb.evaluate(text, config);
  }

  /**
   * Evaluate with specific evaluators
   */
  async evaluateWith(
    text: string,
    evaluators: EvaluatorType[],
    options?: {
      referenceText?: string;
      context?: string[];
      judgeModel?: string;
    }
  ): Promise<CombinedEvaluationResults> {
    const config: EvaluationConfig = {
      evaluators,
      ...options,
      parallel: true,
    };

    return this.ltb.evaluate(text, config);
  }
}

/**
 * Create an evaluator instance
 */
export function createEvaluator(ltb: LLMTestBench): Evaluator {
  return new Evaluator(ltb);
}

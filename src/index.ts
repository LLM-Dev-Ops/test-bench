/**
 * LLM Test Bench - TypeScript SDK
 *
 * A comprehensive, production-ready framework for benchmarking, testing,
 * and evaluating Large Language Models.
 *
 * @packageDocumentation
 */

// Export main SDK class
export { LLMTestBench } from './core/llm-test-bench.js';

// Export provider clients
export {
  ProviderClient,
  OpenAIClient,
  AnthropicClient,
  GoogleClient,
  ProviderClientFactory,
} from './core/provider-client.js';

// Export evaluator utilities
export { Evaluator, createEvaluator } from './evaluators/index.js';

// Export all types
export * from './types/index.js';

// Export utilities
export { executeCLI, findCLIPath } from './utils/cli-executor.js';
export {
  validateProviderConfig,
  validateCompletionRequest,
  validateBenchmarkConfig,
  validateEvaluationConfig,
  isValidModel,
  isValidProvider,
} from './utils/validators.js';

/**
 * SDK version
 */
export const VERSION = '0.1.2';

/**
 * Default export for convenience
 */
import { LLMTestBench } from './core/llm-test-bench.js';

export default LLMTestBench;

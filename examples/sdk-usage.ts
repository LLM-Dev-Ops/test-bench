/**
 * Example usage of the LLM Test Bench TypeScript SDK
 *
 * This example demonstrates various ways to use the SDK for
 * benchmarking, evaluating, and comparing LLM models.
 */

import {
  LLMTestBench,
  ProviderClientFactory,
  createEvaluator,
  type BenchmarkConfig,
} from '../src/index.js';

/**
 * Basic example: Run a simple benchmark
 */
async function basicBenchmark() {
  console.log('=== Basic Benchmark ===\n');

  const ltb = new LLMTestBench({
    verbose: true,
    timeout: 60000,
  });

  try {
    const results = await ltb.benchmark({
      provider: 'openai',
      model: 'gpt-4',
      prompts: [
        'Explain quantum computing in simple terms',
        'What is the difference between AI and ML?',
        'Describe the blockchain technology',
      ],
      config: {
        concurrency: 2,
        saveResponses: true,
        showProgress: true,
      },
    });

    console.log('Benchmark Results:');
    console.log(`- Total Requests: ${results.summary.totalRequests}`);
    console.log(`- Success Rate: ${(results.summary.successRate * 100).toFixed(2)}%`);
    console.log(`- Median Latency: ${results.summary.latency.p50.toFixed(2)}ms`);
    console.log(`- P95 Latency: ${results.summary.latency.p95.toFixed(2)}ms`);
    console.log(`- Total Tokens: ${results.summary.tokens.totalTokens}`);
    if (results.summary.cost) {
      console.log(`- Total Cost: $${results.summary.cost.totalCost.toFixed(4)}`);
    }
  } catch (error) {
    console.error('Benchmark failed:', error);
  }
}

/**
 * Compare multiple models
 */
async function compareModels() {
  console.log('\n=== Model Comparison ===\n');

  const ltb = new LLMTestBench();

  try {
    const results = await ltb.compare({
      models: [
        { provider: 'openai', model: 'gpt-4' },
        { provider: 'anthropic', model: 'claude-opus-4' },
        { provider: 'google', model: 'gemini-2.5-pro' },
      ],
      prompts: ['Write a Python function to calculate Fibonacci numbers'],
      config: {
        concurrency: 1,
      },
    });

    console.log('Comparison Results:');
    for (const modelResult of results.modelResults) {
      console.log(`\n${modelResult.provider}:${modelResult.model}`);
      console.log(`  Success Rate: ${(modelResult.summary.successRate * 100).toFixed(2)}%`);
      console.log(`  Median Latency: ${modelResult.summary.latency.p50.toFixed(2)}ms`);
      console.log(`  Total Tokens: ${modelResult.summary.tokens.totalTokens}`);
    }

    if (results.winner) {
      console.log(`\nWinner: ${results.winner.provider}:${results.winner.model}`);
      console.log(`Reason: ${results.winner.reason}`);
    }
  } catch (error) {
    console.error('Comparison failed:', error);
  }
}

/**
 * Use provider-specific clients
 */
async function useProviderClients() {
  console.log('\n=== Provider-Specific Clients ===\n');

  const ltb = new LLMTestBench();
  const factory = new ProviderClientFactory(ltb);

  // OpenAI client
  const openai = factory.openai();
  try {
    const response = await openai.gpt4o('What is TypeScript?', {
      maxTokens: 100,
      temperature: 0.7,
    });
    console.log('GPT-4o Response:', response.content.substring(0, 100) + '...');
    console.log(`Tokens used: ${response.usage.totalTokens}`);
  } catch (error) {
    console.error('OpenAI request failed:', error);
  }

  // Anthropic client
  const anthropic = factory.anthropic();
  try {
    const response = await anthropic.claude35Sonnet('Explain Rust ownership', {
      maxTokens: 150,
    });
    console.log('\nClaude 3.5 Sonnet Response:', response.content.substring(0, 100) + '...');
    console.log(`Tokens used: ${response.usage.totalTokens}`);
  } catch (error) {
    console.error('Anthropic request failed:', error);
  }
}

/**
 * Evaluate responses
 */
async function evaluateResponses() {
  console.log('\n=== Response Evaluation ===\n');

  const ltb = new LLMTestBench();
  const evaluator = createEvaluator(ltb);

  const text = `
    Artificial Intelligence (AI) is transforming our world in unprecedented ways.
    It enables machines to learn from experience and perform tasks that typically
    require human intelligence. Machine learning, a subset of AI, uses algorithms
    to identify patterns in data and make predictions.
  `.trim();

  try {
    // Evaluate quality
    const qualityResults = await evaluator.evaluateQuality(text);
    console.log('Quality Evaluation:');
    console.log(`- Overall Score: ${qualityResults.overallScore.toFixed(2)}`);
    for (const result of qualityResults.results) {
      console.log(`  ${result.type}: ${result.score.toFixed(2)}`);
    }

    // Evaluate safety
    const safetyResults = await evaluator.evaluateSafety(text);
    console.log('\nSafety Evaluation:');
    console.log(`- Overall Score: ${safetyResults.overallScore.toFixed(2)}`);
    for (const result of safetyResults.results) {
      console.log(`  ${result.type}: ${result.score.toFixed(2)}`);
    }
  } catch (error) {
    console.error('Evaluation failed:', error);
  }
}

/**
 * Optimize model selection
 */
async function optimizeModelSelection() {
  console.log('\n=== Model Optimization ===\n');

  const ltb = new LLMTestBench();

  try {
    const recommendation = await ltb.optimize({
      prompts: [
        'Translate this to Spanish: Hello, how are you?',
        'Summarize the plot of Hamlet',
        'Write a haiku about technology',
      ],
      metric: 'cost',
      maxCost: 0.01,
      minQuality: 0.7,
    });

    console.log('Recommended Model:');
    console.log(`- Provider: ${recommendation.provider}`);
    console.log(`- Model: ${recommendation.model}`);
    console.log(`- Reason: ${recommendation.reason}`);
  } catch (error) {
    console.error('Optimization failed:', error);
  }
}

/**
 * List available models
 */
async function listModels() {
  console.log('\n=== Available Models ===\n');

  const ltb = new LLMTestBench();

  try {
    // List all OpenAI models
    const openaiModels = await ltb.listModels('openai');
    console.log(`OpenAI Models (${openaiModels.length}):`);
    for (const model of openaiModels.slice(0, 5)) {
      console.log(`  - ${model.id} (${model.name})`);
      if (model.contextWindow) {
        console.log(`    Context: ${model.contextWindow} tokens`);
      }
    }

    // List all Anthropic models
    const anthropicModels = await ltb.listModels('anthropic');
    console.log(`\nAnthropic Models (${anthropicModels.length}):`);
    for (const model of anthropicModels.slice(0, 5)) {
      console.log(`  - ${model.id} (${model.name})`);
      if (model.contextWindow) {
        console.log(`    Context: ${model.contextWindow} tokens`);
      }
    }
  } catch (error) {
    console.error('Failed to list models:', error);
  }
}

/**
 * Main function to run all examples
 */
async function main() {
  console.log('LLM Test Bench SDK Examples\n');
  console.log('Note: These examples require API keys to be set in environment variables\n');

  // Run examples (comment out ones you don't want to run)
  await listModels();
  // await basicBenchmark();
  // await compareModels();
  // await useProviderClients();
  // await evaluateResponses();
  // await optimizeModelSelection();

  console.log('\n=== Examples Complete ===');
}

// Run if executed directly
if (import.meta.url === `file://${process.argv[1]}`) {
  main().catch(console.error);
}

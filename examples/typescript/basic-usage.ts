/**
 * Basic usage examples for LLM Test Bench
 *
 * This file demonstrates the core functionality of the SDK including:
 * - Configuration setup
 * - Running benchmarks
 * - Comparing models
 * - Analyzing results
 */

import { spawn } from 'child_process';
import { promisify } from 'util';
import { writeFile, mkdir } from 'fs/promises';
import { join } from 'path';

const exec = promisify(spawn);

/**
 * Example 1: Simple benchmark with a single model
 */
async function example1_SimpleBenchmark() {
  console.log('=== Example 1: Simple Benchmark ===\n');

  // Set up API key (would normally come from environment)
  process.env.OPENAI_API_KEY = process.env.OPENAI_API_KEY || 'your-api-key';

  // Run a simple benchmark
  const result = spawn('llm-test-bench', [
    'bench',
    '--provider', 'openai',
    '--model', 'gpt-3.5-turbo',
    '--prompt', 'Explain quantum computing in simple terms',
    '--output', './results/example1.json'
  ], { stdio: 'inherit' });

  return new Promise((resolve, reject) => {
    result.on('close', (code) => {
      if (code === 0) {
        console.log('✓ Benchmark completed successfully\n');
        resolve(code);
      } else {
        console.error(`✗ Benchmark failed with code ${code}\n`);
        reject(new Error(`Process exited with code ${code}`));
      }
    });
  });
}

/**
 * Example 2: Compare multiple models
 */
async function example2_CompareModels() {
  console.log('=== Example 2: Compare Multiple Models ===\n');

  const result = spawn('llm-test-bench', [
    'compare',
    '--models', 'openai:gpt-4,openai:gpt-3.5-turbo,anthropic:claude-3-haiku-20240307',
    '--prompt', 'Write a Python function to calculate fibonacci numbers',
    '--output', './results/comparison.json'
  ], { stdio: 'inherit' });

  return new Promise((resolve, reject) => {
    result.on('close', (code) => {
      if (code === 0) {
        console.log('✓ Model comparison completed\n');
        resolve(code);
      } else {
        reject(new Error(`Process exited with code ${code}`));
      }
    });
  });
}

/**
 * Example 3: Run benchmark with custom configuration
 */
async function example3_CustomConfig() {
  console.log('=== Example 3: Custom Configuration ===\n');

  // Create a custom config file
  const config = `
[general]
log_level = "info"
output_dir = "./results"

[[providers]]
name = "openai"
api_key_env = "OPENAI_API_KEY"
base_url = "https://api.openai.com/v1"
default_model = "gpt-4"
timeout_seconds = 60
max_retries = 3
enabled = true

[[evaluators]]
name = "perplexity"
enabled = true

[[evaluators]]
name = "coherence"
enabled = true
`;

  await mkdir('./config', { recursive: true });
  await writeFile('./config/custom.toml', config);

  const result = spawn('llm-test-bench', [
    'bench',
    '--config', './config/custom.toml',
    '--provider', 'openai',
    '--model', 'gpt-4',
    '--prompt', 'Explain the theory of relativity'
  ], { stdio: 'inherit' });

  return new Promise((resolve, reject) => {
    result.on('close', (code) => {
      if (code === 0) {
        console.log('✓ Custom config benchmark completed\n');
        resolve(code);
      } else {
        reject(new Error(`Process exited with code ${code}`));
      }
    });
  });
}

/**
 * Example 4: Batch processing with dataset
 */
async function example4_BatchProcessing() {
  console.log('=== Example 4: Batch Processing ===\n');

  // Create a dataset file
  const dataset = {
    name: "Sample Dataset",
    version: "1.0",
    prompts: [
      { id: "1", text: "What is machine learning?", category: "technical" },
      { id: "2", text: "Explain neural networks", category: "technical" },
      { id: "3", text: "Write a haiku about code", category: "creative" }
    ]
  };

  await mkdir('./datasets', { recursive: true });
  await writeFile('./datasets/sample.json', JSON.stringify(dataset, null, 2));

  const result = spawn('llm-test-bench', [
    'bench',
    '--provider', 'openai',
    '--model', 'gpt-3.5-turbo',
    '--dataset', './datasets/sample.json',
    '--output', './results/batch.json'
  ], { stdio: 'inherit' });

  return new Promise((resolve, reject) => {
    result.on('close', (code) => {
      if (code === 0) {
        console.log('✓ Batch processing completed\n');
        resolve(code);
      } else {
        reject(new Error(`Process exited with code ${code}`));
      }
    });
  });
}

/**
 * Example 5: Analyze benchmark results
 */
async function example5_AnalyzeResults() {
  console.log('=== Example 5: Analyze Results ===\n');

  const result = spawn('llm-test-bench', [
    'analyze',
    '--results', './results/comparison.json',
    '--format', 'html',
    '--output', './results/analysis.html'
  ], { stdio: 'inherit' });

  return new Promise((resolve, reject) => {
    result.on('close', (code) => {
      if (code === 0) {
        console.log('✓ Analysis completed\n');
        console.log('View results at: ./results/analysis.html\n');
        resolve(code);
      } else {
        reject(new Error(`Process exited with code ${code}`));
      }
    });
  });
}

/**
 * Example 6: Launch interactive dashboard
 */
async function example6_Dashboard() {
  console.log('=== Example 6: Interactive Dashboard ===\n');

  const result = spawn('llm-test-bench', [
    'dashboard',
    '--port', '8080',
    '--results-dir', './results'
  ], { stdio: 'inherit' });

  console.log('Dashboard starting at http://localhost:8080');
  console.log('Press Ctrl+C to stop\n');

  return new Promise((resolve) => {
    result.on('close', (code) => {
      console.log('Dashboard stopped\n');
      resolve(code);
    });
  });
}

/**
 * Example 7: Optimize model selection
 */
async function example7_OptimizeSelection() {
  console.log('=== Example 7: Optimize Model Selection ===\n');

  const result = spawn('llm-test-bench', [
    'optimize',
    '--metric', 'latency',
    '--max-cost', '0.01',
    '--dataset', './datasets/sample.json',
    '--output', './results/optimization.json'
  ], { stdio: 'inherit' });

  return new Promise((resolve, reject) => {
    result.on('close', (code) => {
      if (code === 0) {
        console.log('✓ Optimization completed\n');
        resolve(code);
      } else {
        reject(new Error(`Process exited with code ${code}`));
      }
    });
  });
}

/**
 * Example 8: Evaluation with custom metrics
 */
async function example8_CustomEvaluators() {
  console.log('=== Example 8: Custom Evaluators ===\n');

  const result = spawn('llm-test-bench', [
    'eval',
    '--provider', 'openai',
    '--model', 'gpt-4',
    '--prompt', 'Summarize: The quick brown fox jumps over the lazy dog',
    '--evaluators', 'perplexity,coherence,faithfulness,relevance',
    '--output', './results/evaluation.json'
  ], { stdio: 'inherit' });

  return new Promise((resolve, reject) => {
    result.on('close', (code) => {
      if (code === 0) {
        console.log('✓ Evaluation completed\n');
        resolve(code);
      } else {
        reject(new Error(`Process exited with code ${code}`));
      }
    });
  });
}

/**
 * Main function to run all examples
 */
async function main() {
  console.log('LLM Test Bench - Example Usage\n');
  console.log('================================\n');

  try {
    // Create results directory
    await mkdir('./results', { recursive: true });

    // Run examples sequentially
    // Note: Comment out examples that require API keys or are time-consuming

    // await example1_SimpleBenchmark();
    // await example2_CompareModels();
    // await example3_CustomConfig();
    // await example4_BatchProcessing();
    // await example5_AnalyzeResults();
    // await example6_Dashboard(); // This starts a server
    // await example7_OptimizeSelection();
    // await example8_CustomEvaluators();

    console.log('All examples completed successfully!');
  } catch (error) {
    console.error('Error running examples:', error);
    process.exit(1);
  }
}

// Run main function if this file is executed directly
if (require.main === module) {
  main();
}

// Export examples for testing
export {
  example1_SimpleBenchmark,
  example2_CompareModels,
  example3_CustomConfig,
  example4_BatchProcessing,
  example5_AnalyzeResults,
  example6_Dashboard,
  example7_OptimizeSelection,
  example8_CustomEvaluators
};

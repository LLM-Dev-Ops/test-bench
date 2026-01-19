/**
 * Benchmark Runner Agent - CLI Integration
 *
 * CLI command spec for invoking the Benchmark Runner Agent.
 * This integrates with agentics-cli.
 */

import * as fs from 'fs/promises';
import * as path from 'path';
import { handler, BENCHMARK_RUNNER_AGENT } from './handler';
import {
  BenchmarkRunnerInputSchema,
  BenchmarkRunnerCLIArgsSchema,
  type BenchmarkRunnerCLIArgs,
  type BenchmarkRunnerInput,
} from '../contracts';

// =============================================================================
// CLI COMMAND SPEC
// =============================================================================

export const CLI_COMMAND_SPEC = {
  name: 'benchmark-runner',
  description: 'Execute deterministic benchmark suites against LLMs',
  aliases: ['bench-run', 'br'],
  usage: 'agentics benchmark-runner [options]',
  examples: [
    'agentics benchmark-runner --input-file benchmark.json',
    'agentics benchmark-runner --input-file benchmark.json --output-format csv',
    'cat benchmark.json | agentics benchmark-runner --input-stdin',
    'agentics benchmark-runner --input-json \'{"providers":[...], "suite":{...}}\'',
  ],
  options: [
    {
      name: '--input-file',
      short: '-i',
      description: 'Path to input JSON file',
      type: 'string',
      required: false,
    },
    {
      name: '--input-json',
      short: '-j',
      description: 'Input as JSON string',
      type: 'string',
      required: false,
    },
    {
      name: '--input-stdin',
      short: '-s',
      description: 'Read input from stdin',
      type: 'boolean',
      required: false,
    },
    {
      name: '--output-format',
      short: '-f',
      description: 'Output format: json, csv, table',
      type: 'string',
      default: 'json',
    },
    {
      name: '--output-file',
      short: '-o',
      description: 'Write output to file',
      type: 'string',
      required: false,
    },
    {
      name: '--verbose',
      short: '-v',
      description: 'Verbose output',
      type: 'boolean',
      default: false,
    },
    {
      name: '--quiet',
      short: '-q',
      description: 'Quiet mode (minimal output)',
      type: 'boolean',
      default: false,
    },
    {
      name: '--dry-run',
      short: '-d',
      description: 'Validate input without executing',
      type: 'boolean',
      default: false,
    },
  ],
} as const;

// =============================================================================
// CLI EXECUTOR
// =============================================================================

/**
 * Execute CLI command
 */
export async function executeCLI(args: string[]): Promise<number> {
  const parsedArgs = parseArgs(args);

  // Validate args
  const argsValidation = BenchmarkRunnerCLIArgsSchema.safeParse(parsedArgs);
  if (!argsValidation.success) {
    console.error('Invalid arguments:', argsValidation.error.message);
    return 1;
  }

  const cliArgs = argsValidation.data;

  // Load input
  let input: unknown;
  try {
    input = await loadInput(cliArgs);
  } catch (err) {
    console.error('Failed to load input:', err instanceof Error ? err.message : err);
    return 1;
  }

  // Validate input
  const inputValidation = BenchmarkRunnerInputSchema.safeParse(input);
  if (!inputValidation.success) {
    console.error('Invalid input:');
    for (const issue of inputValidation.error.issues) {
      console.error(`  - ${issue.path.join('.')}: ${issue.message}`);
    }
    return 1;
  }

  // Dry run - just validate
  if (cliArgs.dry_run) {
    if (!cliArgs.quiet) {
      console.log('✓ Input validation passed');
      console.log(`  Providers: ${inputValidation.data.providers.length}`);
      console.log(`  Test cases: ${inputValidation.data.suite.test_cases.length}`);
    }
    return 0;
  }

  // Execute handler
  if (cliArgs.verbose && !cliArgs.quiet) {
    console.log(`Executing ${BENCHMARK_RUNNER_AGENT.agent_id} v${BENCHMARK_RUNNER_AGENT.agent_version}`);
  }

  const response = await handler({
    body: inputValidation.data,
    headers: {},
    method: 'POST',
    path: '/benchmark-runner',
  });

  // Parse response
  const result = JSON.parse(response.body);

  // Handle error
  if (!result.success) {
    console.error('Execution failed:', result.error.message);
    return 1;
  }

  // Format output
  const output = formatOutput(result.data, cliArgs.output_format);

  // Write output
  if (cliArgs.output_file) {
    await fs.writeFile(cliArgs.output_file, output, 'utf-8');
    if (!cliArgs.quiet) {
      console.log(`Output written to: ${cliArgs.output_file}`);
    }
  } else if (!cliArgs.quiet) {
    console.log(output);
  }

  // Print summary
  if (!cliArgs.quiet) {
    console.log(`\nDecision ID: ${result.decision_id}`);
    console.log(`Success rate: ${((result.data.successful_executions / result.data.total_executions) * 100).toFixed(1)}%`);
    console.log(`Duration: ${result.data.total_duration_ms}ms`);
  }

  return 0;
}

// =============================================================================
// HELPERS
// =============================================================================

function parseArgs(args: string[]): Partial<BenchmarkRunnerCLIArgs> {
  const result: Partial<BenchmarkRunnerCLIArgs> = {};
  let i = 0;

  while (i < args.length) {
    const arg = args[i];

    switch (arg) {
      case '--input-file':
      case '-i':
        result.input_file = args[++i];
        break;
      case '--input-json':
      case '-j':
        result.input_json = args[++i];
        break;
      case '--input-stdin':
      case '-s':
        result.input_stdin = true;
        break;
      case '--output-format':
      case '-f':
        result.output_format = args[++i] as 'json' | 'csv' | 'table';
        break;
      case '--output-file':
      case '-o':
        result.output_file = args[++i];
        break;
      case '--verbose':
      case '-v':
        result.verbose = true;
        break;
      case '--quiet':
      case '-q':
        result.quiet = true;
        break;
      case '--dry-run':
      case '-d':
        result.dry_run = true;
        break;
    }

    i++;
  }

  return result;
}

async function loadInput(args: BenchmarkRunnerCLIArgs): Promise<unknown> {
  if (args.input_file) {
    const content = await fs.readFile(args.input_file, 'utf-8');
    return JSON.parse(content);
  }

  if (args.input_json) {
    return JSON.parse(args.input_json);
  }

  if (args.input_stdin) {
    return new Promise((resolve, reject) => {
      let data = '';
      process.stdin.setEncoding('utf-8');
      process.stdin.on('data', chunk => (data += chunk));
      process.stdin.on('end', () => {
        try {
          resolve(JSON.parse(data));
        } catch (err) {
          reject(err);
        }
      });
      process.stdin.on('error', reject);
    });
  }

  throw new Error('No input source specified. Use --input-file, --input-json, or --input-stdin');
}

function formatOutput(data: unknown, format: string): string {
  switch (format) {
    case 'json':
      return JSON.stringify(data, null, 2);

    case 'csv':
      return formatCSV(data);

    case 'table':
      return formatTable(data);

    default:
      return JSON.stringify(data, null, 2);
  }
}

function formatCSV(data: any): string {
  const results = data.results ?? [];
  if (results.length === 0) return '';

  const headers = [
    'test_id',
    'iteration',
    'provider_name',
    'model_id',
    'success',
    'latency_ms',
    'prompt_tokens',
    'completion_tokens',
    'cost_usd',
  ];

  const rows = results.map((r: any) => [
    r.test_id,
    r.iteration,
    r.provider_name,
    r.model_id,
    r.success,
    r.latency.total_ms,
    r.token_usage?.prompt_tokens ?? '',
    r.token_usage?.completion_tokens ?? '',
    r.cost?.total_cost_usd ?? '',
  ]);

  return [headers.join(','), ...rows.map((r: any[]) => r.join(','))].join('\n');
}

function formatTable(data: any): string {
  const stats = data.aggregated_stats ?? [];
  if (stats.length === 0) return 'No results';

  const header = `| Provider | Model | Success Rate | P50 (ms) | P95 (ms) | Mean (ms) | Total Cost |`;
  const separator = `|----------|-------|--------------|----------|----------|-----------|------------|`;

  const rows = stats.map((s: any) =>
    `| ${s.provider_name.padEnd(8)} | ${s.model_id.slice(0, 5).padEnd(5)} | ${(s.success_rate * 100).toFixed(1).padStart(11)}% | ${s.latency_p50_ms.toFixed(0).padStart(8)} | ${s.latency_p95_ms.toFixed(0).padStart(8)} | ${s.latency_mean_ms.toFixed(0).padStart(9)} | $${s.total_cost_usd.toFixed(4).padStart(9)} |`
  );

  return [header, separator, ...rows].join('\n');
}

// =============================================================================
// MAIN ENTRY
// =============================================================================

if (require.main === module) {
  executeCLI(process.argv.slice(2))
    .then(code => process.exit(code))
    .catch(err => {
      console.error('Fatal error:', err);
      process.exit(1);
    });
}

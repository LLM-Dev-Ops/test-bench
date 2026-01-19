/**
 * Prompt Sensitivity Agent - CLI Integration
 *
 * CLI command spec for invoking the Prompt Sensitivity Agent.
 * This integrates with agentics-cli.
 */

import * as fs from 'fs/promises';
import * as path from 'path';
import { handler, PROMPT_SENSITIVITY_AGENT } from './handler';
import {
  PromptSensitivityInputSchema,
  PromptSensitivityCLIArgsSchema,
  type PromptSensitivityCLIArgs,
  type PromptSensitivityInput,
} from '../contracts';

// =============================================================================
// CLI COMMAND SPEC
// =============================================================================

export const CLI_COMMAND_SPEC = {
  name: 'prompt-sensitivity',
  description: 'Measure output variance under controlled prompt perturbations',
  aliases: ['ps', 'sensitivity'],
  usage: 'agentics prompt-sensitivity [options]',
  examples: [
    'agentics prompt-sensitivity --input-file analysis.json',
    'agentics prompt-sensitivity --prompt "Explain quantum computing" --provider openai --model gpt-4o',
    'agentics prompt-sensitivity -i analysis.json -f table',
    'cat analysis.json | agentics prompt-sensitivity -s',
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
      name: '--prompt',
      short: '-p',
      description: 'Base prompt string (convenience for simple cases)',
      type: 'string',
      required: false,
    },
    {
      name: '--provider',
      description: 'Provider name (e.g., openai, anthropic)',
      type: 'string',
      required: false,
    },
    {
      name: '--model',
      short: '-m',
      description: 'Model ID',
      type: 'string',
      required: false,
    },
    {
      name: '--perturbation-types',
      description: 'Comma-separated perturbation types',
      type: 'string',
      required: false,
    },
    {
      name: '--runs',
      short: '-r',
      description: 'Runs per perturbation (default: 3)',
      type: 'number',
      default: 3,
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
  const argsValidation = PromptSensitivityCLIArgsSchema.safeParse(parsedArgs);
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
  const inputValidation = PromptSensitivityInputSchema.safeParse(input);
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
      console.log(`  Base prompt: ${inputValidation.data.base_prompt?.substring(0, 50)}...`);
      console.log(`  Provider: ${inputValidation.data.provider?.provider_name}`);
      console.log(`  Model: ${inputValidation.data.provider?.model_id}`);
      console.log(`  Perturbation types: ${inputValidation.data.perturbation_config?.types?.join(', ') || 'all'}`);
      console.log(`  Runs per perturbation: ${inputValidation.data.sampling_config?.runs_per_perturbation || 3}`);
    }
    return 0;
  }

  // Execute handler
  if (cliArgs.verbose && !cliArgs.quiet) {
    console.log(`Executing ${PROMPT_SENSITIVITY_AGENT.agent_id} v${PROMPT_SENSITIVITY_AGENT.agent_version}`);
  }

  const response = await handler({
    body: inputValidation.data,
    headers: {},
    method: 'POST',
    path: '/prompt-sensitivity',
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
    printSummary(result);
  }

  return 0;
}

// =============================================================================
// HELPERS
// =============================================================================

function parseArgs(args: string[]): Partial<PromptSensitivityCLIArgs> {
  const result: Partial<PromptSensitivityCLIArgs> = {};
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
      case '--prompt':
      case '-p':
        result.base_prompt = args[++i];
        break;
      case '--provider':
        result.provider = args[++i];
        break;
      case '--model':
      case '-m':
        result.model = args[++i];
        break;
      case '--perturbation-types':
        result.perturbation_types = args[++i];
        break;
      case '--runs':
      case '-r':
        result.runs_per_perturbation = parseInt(args[++i], 10);
        break;
      case '--output-format':
      case '-f':
        result.output_format = args[++i] as 'json' | 'table' | 'summary';
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

async function loadInput(args: PromptSensitivityCLIArgs): Promise<unknown> {
  // If convenience args are provided, build input from them
  if (args.base_prompt && args.provider && args.model) {
    const input: Record<string, unknown> = {
      base_prompt: args.base_prompt,
      provider: {
        provider_name: args.provider,
        model_id: args.model,
      },
      perturbation_config: {
        types: ['paraphrase', 'tone_shift'],
        perturbations_per_type: 3,
        auto_generate: true,
      },
      sampling_config: {
        runs_per_perturbation: args.runs_per_perturbation ?? 3,
      },
    };

    if (args.perturbation_types) {
      (input.perturbation_config as Record<string, unknown>).types = args.perturbation_types.split(',').map(t => t.trim());
    }

    return input;
  }

  // Otherwise load from file/json/stdin
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

  throw new Error(
    'No input source specified. Use --input-file, --input-json, --input-stdin, or provide --prompt, --provider, and --model'
  );
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
  const results = data.perturbation_results ?? [];
  if (results.length === 0) return '';

  const headers = [
    'perturbation_type',
    'variance_score',
    'semantic_similarity',
    'mean_latency_ms',
    'runs',
  ];

  const rows = results.map((r: any) => [
    r.perturbation_type,
    r.variance_score.toFixed(4),
    r.semantic_similarity.mean.toFixed(4),
    r.mean_latency_ms.toFixed(2),
    r.runs,
  ]);

  return [headers.join(','), ...rows.map((r: any[]) => r.join(','))].join('\n');
}

function formatTable(data: any): string {
  const results = data.perturbation_results ?? [];
  if (results.length === 0) return 'No results';

  const header = `| Perturbation Type | Variance | Similarity | Latency (ms) | Runs |`;
  const separator = `|-------------------|----------|------------|--------------|------|`;

  const rows = results.map((r: any) =>
    `| ${r.perturbation_type.padEnd(17)} | ${r.variance_score.toFixed(4).padStart(8)} | ${r.semantic_similarity.mean.toFixed(4).padStart(10)} | ${r.mean_latency_ms.toFixed(2).padStart(12)} | ${r.runs.toString().padStart(4)} |`
  );

  return [header, separator, ...rows].join('\n');
}

function printSummary(result: any): void {
  console.log('\nSensitivity Analysis Complete');
  console.log(`Decision ID: ${result.decision_id}`);

  const data = result.data;
  const overallScore = data.overall_sensitivity_score;
  const sensitivityLevel = getSensitivityLevel(overallScore);

  console.log(`Overall Sensitivity: ${overallScore.toFixed(2)} (${sensitivityLevel})`);

  // Find most and least sensitive perturbations
  const perturbations = data.perturbation_results ?? [];
  if (perturbations.length > 0) {
    const sorted = [...perturbations].sort((a, b) => b.variance_score - a.variance_score);
    const mostSensitive = sorted[0];
    const leastSensitive = sorted[sorted.length - 1];

    console.log(`Most Sensitive: ${mostSensitive.perturbation_type} (${mostSensitive.variance_score.toFixed(2)})`);
    console.log(`Least Sensitive: ${leastSensitive.perturbation_type} (${leastSensitive.variance_score.toFixed(2)})`);
  }

  const duration = data.total_duration_ms
    ? (data.total_duration_ms / 1000).toFixed(1)
    : result.metadata?.execution_time_ms
    ? (result.metadata.execution_time_ms / 1000).toFixed(1)
    : 'N/A';

  console.log(`Duration: ${duration}s`);
}

function getSensitivityLevel(score: number): string {
  if (score < 0.3) return 'low';
  if (score < 0.6) return 'moderate';
  return 'high';
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

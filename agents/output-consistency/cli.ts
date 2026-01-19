/**
 * Output Consistency Agent - CLI Integration
 *
 * CLI command spec for invoking the Output Consistency Agent.
 * This integrates with agentics-cli.
 */

import * as fs from 'fs/promises';
import * as path from 'path';
import { handler, OUTPUT_CONSISTENCY_AGENT } from './handler';
import {
  OutputConsistencyInputSchema,
  OutputConsistencyCLIArgsSchema,
  ConsistencyConfigSchema,
  type OutputConsistencyCLIArgs,
  type OutputConsistencyInput,
  type ConsistencyConfig,
} from '../contracts';

// =============================================================================
// CLI COMMAND SPEC
// =============================================================================

export const CLI_COMMAND_SPEC = {
  name: 'output-consistency',
  description: 'Measure consistency across repeated executions of identical prompts',
  aliases: ['consistency', 'oc', 'repeat'],
  usage: 'agentics output-consistency [options]',
  examples: [
    'agentics output-consistency --input-file executions.json',
    'agentics output-consistency --input-file executions.json --similarity-method normalized_levenshtein',
    'agentics output-consistency --input-file executions.json --consistency-threshold 0.9',
    'agentics output-consistency --input-file executions.json --output-format table',
    'cat executions.json | agentics output-consistency --input-stdin',
    'agentics output-consistency --input-json \'{"execution_groups":[...]}\'',
    'agentics output-consistency --dry-run --input-file executions.json',
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
      name: '--similarity-method',
      short: '-m',
      description: 'Similarity method: exact_match, normalized_levenshtein, jaccard_tokens, cosine_tfidf, character_ngram, word_ngram',
      type: 'string',
      default: 'jaccard_tokens',
    },
    {
      name: '--consistency-threshold',
      short: '-t',
      description: 'Minimum consistency score (0-1) to consider consistent',
      type: 'number',
      default: 0.85,
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
  // Handle help
  if (args.includes('--help') || args.includes('-h')) {
    printHelp();
    return 0;
  }

  const parsedArgs = parseArgs(args);

  // Validate args
  const argsValidation = OutputConsistencyCLIArgsSchema.safeParse(parsedArgs);
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

  // Apply config overrides
  if (typeof input === 'object' && input !== null) {
    const inputObj = input as Record<string, unknown>;
    const config = (inputObj.config ?? {}) as Record<string, unknown>;

    if (cliArgs.similarity_method) {
      config.similarity_method = cliArgs.similarity_method;
    }
    if (cliArgs.consistency_threshold !== undefined) {
      config.consistency_threshold = cliArgs.consistency_threshold;
    }

    inputObj.config = config;
  }

  // Validate input
  const inputValidation = OutputConsistencyInputSchema.safeParse(input);
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
      console.log('Input validation passed');
      console.log(`  Execution groups: ${inputValidation.data.execution_groups.length}`);
      const totalOutputs = inputValidation.data.execution_groups.reduce(
        (sum, g) => sum + g.outputs.length,
        0
      );
      console.log(`  Total outputs: ${totalOutputs}`);
      console.log(`  Similarity method: ${inputValidation.data.config?.similarity_method ?? 'jaccard_tokens'}`);
    }
    return 0;
  }

  // Execute handler
  if (cliArgs.verbose && !cliArgs.quiet) {
    console.log(`Executing ${OUTPUT_CONSISTENCY_AGENT.agent_id} v${OUTPUT_CONSISTENCY_AGENT.agent_version}`);
    console.log(`Analyzing ${inputValidation.data.execution_groups.length} execution groups`);
  }

  const response = await handler({
    body: inputValidation.data,
    headers: {},
    method: 'POST',
    path: '/output-consistency',
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
    console.log(`Average Consistency: ${(result.data.summary.overall_avg_consistency * 100).toFixed(1)}%`);
    console.log(`Consistency Rate: ${(result.data.summary.overall_consistency_rate * 100).toFixed(1)}%`);
    console.log(`Duration: ${result.data.duration_ms}ms`);
  }

  return 0;
}

// =============================================================================
// HELPERS
// =============================================================================

function printHelp(): void {
  console.log(`
${CLI_COMMAND_SPEC.description}

Usage: ${CLI_COMMAND_SPEC.usage}

Options:
${CLI_COMMAND_SPEC.options.map(opt =>
  `  ${opt.short}, ${opt.name.padEnd(24)} ${opt.description}${'default' in opt && opt.default !== undefined ? ` (default: ${opt.default})` : ''}`
).join('\n')}

Similarity Methods:
  exact_match             1.0 if all outputs identical, 0.0 otherwise
  normalized_levenshtein  Edit distance normalized to 0-1 (higher = more similar)
  jaccard_tokens          Jaccard similarity on word tokens
  cosine_tfidf            Cosine similarity on TF-IDF vectors
  character_ngram         Character n-gram overlap
  word_ngram              Word n-gram overlap

Examples:
${CLI_COMMAND_SPEC.examples.map(ex => `  ${ex}`).join('\n')}

For more information, see: https://github.com/agentics/output-consistency
`);
}

function parseArgs(args: string[]): Partial<OutputConsistencyCLIArgs> {
  const result: Partial<OutputConsistencyCLIArgs> = {};
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
      case '--similarity-method':
      case '-m':
        result.similarity_method = args[++i] as any;
        break;
      case '--consistency-threshold':
      case '-t':
        result.consistency_threshold = parseFloat(args[++i]);
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

async function loadInput(args: OutputConsistencyCLIArgs): Promise<unknown> {
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
    'group_id',
    'provider_name',
    'model_id',
    'output_count',
    'consistency_score',
    'is_consistent',
    'similarity_method',
    'representative_idx',
    'divergent_idx',
    'max_divergence',
  ];

  const rows = results.map((r: any) => [
    r.group_id,
    r.provider_name,
    r.model_id,
    r.output_count,
    r.consistency_score.toFixed(4),
    r.is_consistent,
    r.similarity_scores.primary_method,
    r.representative_output_index,
    r.most_divergent_output_index,
    r.max_divergence_score.toFixed(4),
  ]);

  return [headers.join(','), ...rows.map((r: any[]) => r.join(','))].join('\n');
}

function formatTable(data: any): string {
  const stats = data.model_stats ?? [];
  if (stats.length === 0) return 'No results';

  const header = `| Provider | Model | Groups | Avg Consistency | Min | Max | Rate |`;
  const separator = `|----------|-------|--------|-----------------|-----|-----|------|`;

  const rows = stats.map((s: any) =>
    `| ${s.provider_name.padEnd(8).slice(0, 8)} | ${s.model_id.slice(0, 5).padEnd(5)} | ${s.groups_analyzed.toString().padStart(6)} | ${(s.avg_consistency_score * 100).toFixed(1).padStart(14)}% | ${(s.min_consistency_score * 100).toFixed(0).padStart(3)}% | ${(s.max_consistency_score * 100).toFixed(0).padStart(3)}% | ${(s.consistency_rate * 100).toFixed(0).padStart(3)}% |`
  );

  // Add summary
  const summary = data.summary;
  const summaryRow = `\nSummary: ${summary.total_groups_analyzed} groups analyzed, ${summary.total_outputs_analyzed} outputs`;
  const avgRow = `Overall Avg Consistency: ${(summary.overall_avg_consistency * 100).toFixed(1)}%`;

  // Add distribution
  const dist = summary.consistency_distribution;
  const distRow = `Distribution: Highly(${dist.highly_consistent}) Consistent(${dist.consistent}) Moderate(${dist.moderate}) Inconsistent(${dist.inconsistent}) Highly Inconsistent(${dist.highly_inconsistent})`;

  // Add best/worst models
  let modelRows = '';
  if (summary.most_consistent_model) {
    modelRows += `\nMost Consistent: ${summary.most_consistent_model.provider_name}/${summary.most_consistent_model.model_id} (${(summary.most_consistent_model.avg_score * 100).toFixed(1)}%)`;
  }
  if (summary.least_consistent_model) {
    modelRows += `\nLeast Consistent: ${summary.least_consistent_model.provider_name}/${summary.least_consistent_model.model_id} (${(summary.least_consistent_model.avg_score * 100).toFixed(1)}%)`;
  }

  return [header, separator, ...rows, summaryRow, avgRow, distRow, modelRows].join('\n');
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

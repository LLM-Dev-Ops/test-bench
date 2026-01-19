/**
 * Quality Scoring Agent - CLI Integration
 *
 * CLI command spec for invoking the Quality Scoring Agent.
 * This integrates with agentics-cli.
 */

import * as fs from 'fs/promises';
import * as path from 'path';
import { handler, QUALITY_SCORING_AGENT } from './handler';
import {
  QualityScoringInputSchema,
  QualityScoringCLIArgsSchema,
  ScoringProfileSchema,
  PRESET_PROFILES,
  type QualityScoringCLIArgs,
  type QualityScoringInput,
  type ScoringProfile,
} from '../contracts';

// =============================================================================
// CLI COMMAND SPEC
// =============================================================================

export const CLI_COMMAND_SPEC = {
  name: 'quality-scoring',
  description: 'Compute normalized quality scores for model outputs using deterministic scoring profiles',
  aliases: ['quality', 'qs', 'score'],
  usage: 'agentics quality-scoring [options]',
  examples: [
    'agentics quality-scoring --input-file outputs.json',
    'agentics quality-scoring --input-file outputs.json --profile-file custom-profile.json',
    'agentics quality-scoring --input-file outputs.json --output-format table',
    'cat outputs.json | agentics quality-scoring --input-stdin',
    'agentics quality-scoring --input-json \'{"outputs":[...], "scoring_profile":{...}}\'',
    'agentics quality-scoring --dry-run --input-file outputs.json',
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
      name: '--profile-file',
      short: '-p',
      description: 'Path to scoring profile JSON file (overrides embedded profile)',
      type: 'string',
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
  // Handle help
  if (args.includes('--help') || args.includes('-h')) {
    printHelp();
    return 0;
  }

  const parsedArgs = parseArgs(args);

  // Validate args
  const argsValidation = QualityScoringCLIArgsSchema.safeParse(parsedArgs);
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

  // Load profile from separate file if specified
  if (cliArgs.profile_file && typeof input === 'object' && input !== null) {
    try {
      const profileContent = await fs.readFile(cliArgs.profile_file, 'utf-8');
      const profile = JSON.parse(profileContent);
      const profileValidation = ScoringProfileSchema.safeParse(profile);
      if (!profileValidation.success) {
        console.error('Invalid scoring profile:');
        for (const issue of profileValidation.error.issues) {
          console.error(`  - ${issue.path.join('.')}: ${issue.message}`);
        }
        return 1;
      }
      (input as any).scoring_profile = profileValidation.data;
    } catch (err) {
      console.error('Failed to load profile:', err instanceof Error ? err.message : err);
      return 1;
    }
  }

  // Validate input
  const inputValidation = QualityScoringInputSchema.safeParse(input);
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
      console.log(`  Outputs: ${inputValidation.data.outputs.length}`);
      console.log(`  Profile: ${inputValidation.data.scoring_profile.name}`);
      console.log(`  Dimensions: ${inputValidation.data.scoring_profile.dimensions.length}`);
    }
    return 0;
  }

  // Execute handler
  if (cliArgs.verbose && !cliArgs.quiet) {
    console.log(`Executing ${QUALITY_SCORING_AGENT.agent_id} v${QUALITY_SCORING_AGENT.agent_version}`);
    console.log(`Scoring ${inputValidation.data.outputs.length} outputs with profile "${inputValidation.data.scoring_profile.name}"`);
  }

  const response = await handler({
    body: inputValidation.data,
    headers: {},
    method: 'POST',
    path: '/quality-scoring',
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
    console.log(`Average Score: ${(result.data.summary.overall_avg_score * 100).toFixed(1)}%`);
    console.log(`Pass Rate: ${(result.data.summary.overall_pass_rate * 100).toFixed(1)}%`);
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
  `  ${opt.short}, ${opt.name.padEnd(18)} ${opt.description}${'default' in opt && opt.default !== undefined ? ` (default: ${opt.default})` : ''}`
).join('\n')}

Examples:
${CLI_COMMAND_SPEC.examples.map(ex => `  ${ex}`).join('\n')}

Preset Profiles:
  accuracy-basic     Simple accuracy scoring (exact match + keywords)
  comprehensive      Multi-dimensional quality assessment

For more information, see: https://github.com/agentics/quality-scoring
`);
}

function parseArgs(args: string[]): Partial<QualityScoringCLIArgs> {
  const result: Partial<QualityScoringCLIArgs> = {};
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
      case '--profile-file':
      case '-p':
        result.profile_file = args[++i];
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

async function loadInput(args: QualityScoringCLIArgs): Promise<unknown> {
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
  const scores = data.scores ?? [];
  if (scores.length === 0) return '';

  const headers = [
    'output_id',
    'provider_name',
    'model_id',
    'composite_score',
    'dimensions_passed',
    'dimensions_total',
    'pass_rate',
    'overall_passed',
  ];

  const rows = scores.map((s: any) => [
    s.output_id,
    s.provider_name,
    s.model_id,
    s.composite_score.toFixed(4),
    s.dimensions_passed,
    s.dimensions_total,
    s.pass_rate.toFixed(4),
    s.overall_passed,
  ]);

  return [headers.join(','), ...rows.map((r: any[]) => r.join(','))].join('\n');
}

function formatTable(data: any): string {
  const stats = data.model_stats ?? [];
  if (stats.length === 0) return 'No results';

  const header = `| Provider | Model | Avg Score | Min | Max | Pass Rate | Outputs |`;
  const separator = `|----------|-------|-----------|-----|-----|-----------|---------|`;

  const rows = stats.map((s: any) =>
    `| ${s.provider_name.padEnd(8)} | ${s.model_id.slice(0, 5).padEnd(5)} | ${(s.avg_composite_score * 100).toFixed(1).padStart(8)}% | ${(s.min_composite_score * 100).toFixed(0).padStart(3)}% | ${(s.max_composite_score * 100).toFixed(0).padStart(3)}% | ${(s.overall_pass_rate * 100).toFixed(1).padStart(8)}% | ${s.outputs_scored.toString().padStart(7)} |`
  );

  // Add summary
  const summary = data.summary;
  const summaryRow = `\nSummary: ${summary.total_outputs_scored} outputs scored, Overall Avg: ${(summary.overall_avg_score * 100).toFixed(1)}%`;

  // Add distribution
  const dist = summary.score_distribution;
  const distRow = `Distribution: Excellent(${dist.excellent}) Good(${dist.good}) Fair(${dist.fair}) Poor(${dist.poor}) Failed(${dist.failed})`;

  return [header, separator, ...rows, summaryRow, distRow].join('\n');
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

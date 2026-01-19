/**
 * Synthetic Data Generator Agent - CLI Integration
 *
 * CLI command spec for invoking the Synthetic Data Generator Agent.
 * This integrates with agentics-cli.
 */

import * as fs from 'fs/promises';
import { handler, SYNTHETIC_DATA_GENERATOR_AGENT } from './handler';
import {
  SyntheticDataGeneratorInputSchema,
  SyntheticDataGeneratorCLIArgsSchema,
  GENERATION_PRESETS,
  type SyntheticDataGeneratorCLIArgs,
  type SyntheticDataGeneratorInput,
  type GenerationPreset,
} from '../contracts';

// =============================================================================
// CLI COMMAND SPEC
// =============================================================================

export const CLI_COMMAND_SPEC = {
  name: 'synthetic-data-generator',
  description: 'Generate synthetic datasets for testing and benchmarking',
  aliases: ['synth-gen', 'sdg'],
  usage: 'agentics synthetic-data-generator [options]',
  examples: [
    'agentics synthetic-data-generator --type qa_pair --count 100',
    'agentics synthetic-data-generator --input-file config.json',
    'agentics synthetic-data-generator --type coding_task --count 50 --strategy progressive_difficulty',
    'agentics synthetic-data-generator --preset qa-benchmark --count 1000',
    'agentics synthetic-data-generator --type text_prompt --count 10 --seed 42',
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
      name: '--type',
      short: '-t',
      description: 'Data type to generate (qa_pair, coding_task, text_prompt, etc.)',
      type: 'string',
      required: false,
    },
    {
      name: '--strategy',
      short: '-s',
      description: 'Generation strategy (template_based, variation, edge_case, etc.)',
      type: 'string',
      default: 'template_based',
    },
    {
      name: '--count',
      short: '-c',
      description: 'Number of items to generate',
      type: 'number',
      required: false,
    },
    {
      name: '--preset',
      short: '-p',
      description: 'Use preset configuration (qa-benchmark, coding-challenge, etc.)',
      type: 'string',
      required: false,
    },
    {
      name: '--output-format',
      short: '-f',
      description: 'Output format: json, jsonl, csv',
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
      name: '--seed',
      description: 'Random seed for reproducibility',
      type: 'number',
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
      description: 'Validate input without generating',
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
  const argsValidation = SyntheticDataGeneratorCLIArgsSchema.safeParse(parsedArgs);
  if (!argsValidation.success) {
    console.error('Invalid arguments:', argsValidation.error.message);
    return 1;
  }

  const cliArgs = argsValidation.data;

  // Build input
  let input: unknown;
  try {
    input = await buildInput(cliArgs);
  } catch (err) {
    console.error('Failed to build input:', err instanceof Error ? err.message : err);
    return 1;
  }

  // Validate input
  const inputValidation = SyntheticDataGeneratorInputSchema.safeParse(input);
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
      console.log('Validation passed');
      console.log(`  Data type: ${inputValidation.data.data_type}`);
      console.log(`  Strategy: ${inputValidation.data.generation_strategy}`);
      console.log(`  Count: ${inputValidation.data.count}`);
      if (inputValidation.data.random_seed !== undefined) {
        console.log(`  Seed: ${inputValidation.data.random_seed}`);
      }
    }
    return 0;
  }

  // Execute handler
  if (cliArgs.verbose && !cliArgs.quiet) {
    console.log(`Executing ${SYNTHETIC_DATA_GENERATOR_AGENT.agent_id} v${SYNTHETIC_DATA_GENERATOR_AGENT.agent_version}`);
    console.log(`Generating ${inputValidation.data.count} ${inputValidation.data.data_type} items...`);
  }

  const response = await handler({
    body: inputValidation.data,
    headers: {},
    method: 'POST',
    path: '/synthetic-data-generator',
  });

  // Parse response
  const result = JSON.parse(response.body);

  // Handle error
  if (!result.success) {
    console.error('Generation failed:', result.error.message);
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
    console.log(`Generated: ${result.data.generation_stats.generated_count}/${result.data.generation_stats.requested_count} items`);
    console.log(`Failed: ${result.data.generation_stats.failed_count}`);
    console.log(`Duplicates: ${result.data.generation_stats.duplicate_count}`);
    console.log(`Duration: ${result.data.duration_ms}ms`);
    console.log(`Unique items rate: ${(result.data.quality_metrics.unique_items_rate * 100).toFixed(1)}%`);
  }

  return 0;
}

// =============================================================================
// HELPERS
// =============================================================================

function parseArgs(args: string[]): Partial<SyntheticDataGeneratorCLIArgs> {
  const result: Partial<SyntheticDataGeneratorCLIArgs> = {};
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
      case '--type':
      case '-t':
        result.type = args[++i];
        break;
      case '--strategy':
      case '-s':
        result.strategy = args[++i];
        break;
      case '--count':
      case '-c':
        result.count = parseInt(args[++i], 10);
        break;
      case '--preset':
      case '-p':
        result.preset = args[++i];
        break;
      case '--output-format':
      case '-f':
        result.output_format = args[++i] as 'json' | 'jsonl' | 'csv';
        break;
      case '--output-file':
      case '-o':
        result.output_file = args[++i];
        break;
      case '--seed':
        result.seed = parseInt(args[++i], 10);
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

async function buildInput(args: SyntheticDataGeneratorCLIArgs): Promise<SyntheticDataGeneratorInput> {
  // If input file provided, load it
  if (args.input_file) {
    const content = await fs.readFile(args.input_file, 'utf-8');
    const parsed = JSON.parse(content);
    // Apply seed override if provided
    if (args.seed !== undefined) {
      parsed.random_seed = args.seed;
    }
    return parsed;
  }

  // If input JSON provided, parse it
  if (args.input_json) {
    const parsed = JSON.parse(args.input_json);
    if (args.seed !== undefined) {
      parsed.random_seed = args.seed;
    }
    return parsed;
  }

  // If preset provided, use it
  if (args.preset) {
    const presetKey = args.preset as GenerationPreset;
    if (!(presetKey in GENERATION_PRESETS)) {
      throw new Error(`Unknown preset: ${args.preset}. Available: ${Object.keys(GENERATION_PRESETS).join(', ')}`);
    }
    const preset = GENERATION_PRESETS[presetKey];
    return {
      data_type: preset.data_type as any,
      generation_strategy: preset.generation_strategy as any,
      count: args.count ?? 100,
      difficulty_distribution: (preset as any).difficulty_distribution,
      constraints: (preset as any).constraints,
      coding_config: (preset as any).coding_config,
      conversation_config: (preset as any).conversation_config,
      random_seed: args.seed,
    };
  }

  // Build from individual args
  if (!args.type) {
    throw new Error('No input source specified. Use --input-file, --input-json, --preset, or --type');
  }

  if (!args.count) {
    throw new Error('Count is required when using --type. Use --count <number>');
  }

  return {
    data_type: args.type as any,
    generation_strategy: (args.strategy ?? 'template_based') as any,
    count: args.count,
    random_seed: args.seed,
  };
}

function formatOutput(data: any, format: string): string {
  switch (format) {
    case 'json':
      return JSON.stringify(data, null, 2);

    case 'jsonl':
      return formatJSONL(data);

    case 'csv':
      return formatCSV(data);

    default:
      return JSON.stringify(data, null, 2);
  }
}

function formatJSONL(data: any): string {
  const items = data.generated_items ?? [];
  return items.map((item: any) => JSON.stringify(item)).join('\n');
}

function formatCSV(data: any): string {
  const items = data.generated_items ?? [];
  if (items.length === 0) return '';

  // Get all unique keys from content
  const contentKeys = new Set<string>();
  for (const item of items) {
    for (const key of Object.keys(item.content || {})) {
      contentKeys.add(key);
    }
  }

  const headers = [
    'item_id',
    'data_type',
    'difficulty_level',
    'strategy_used',
    'length_chars',
    'complexity_score',
    ...Array.from(contentKeys),
  ];

  const rows = items.map((item: any) => {
    const contentValues = Array.from(contentKeys).map(key => {
      const value = item.content?.[key];
      if (value === undefined || value === null) return '';
      if (typeof value === 'object') return JSON.stringify(value);
      return String(value).replace(/"/g, '""');
    });

    return [
      item.item_id,
      item.data_type,
      item.generation_metadata?.difficulty_level ?? '',
      item.generation_metadata?.strategy_used ?? '',
      item.quality_indicators?.length_chars ?? '',
      item.quality_indicators?.complexity_score ?? '',
      ...contentValues,
    ].map(v => `"${v}"`).join(',');
  });

  return [headers.map(h => `"${h}"`).join(','), ...rows].join('\n');
}

// =============================================================================
// HELP TEXT
// =============================================================================

export function printHelp(): void {
  console.log(`
${CLI_COMMAND_SPEC.name} - ${CLI_COMMAND_SPEC.description}

USAGE:
  ${CLI_COMMAND_SPEC.usage}

ALIASES:
  ${CLI_COMMAND_SPEC.aliases.join(', ')}

OPTIONS:
${CLI_COMMAND_SPEC.options.map(opt => {
  const shortOpt = 'short' in opt && opt.short ? `${opt.short}, ` : '    ';
  return `  ${shortOpt}${opt.name.padEnd(20)} ${opt.description}`;
}).join('\n')}

EXAMPLES:
${CLI_COMMAND_SPEC.examples.map(ex => `  ${ex}`).join('\n')}

PRESETS:
  qa-benchmark          QA pairs with difficulty distribution
  coding-challenge      Coding tasks with test cases
  stress-test-prompts   Edge case text prompts
  conversation-dataset  Multi-turn conversations
  adversarial-inputs    Adversarial/tricky prompts

DATA TYPES:
  text_prompt, qa_pair, multi_turn_conversation, coding_task,
  summarization, creative_writing, classification, entity_extraction,
  translation, reasoning_chain

STRATEGIES:
  template_based, variation, distribution_aware, edge_case,
  adversarial, combinatorial, progressive_difficulty, cross_domain
`);
}

// =============================================================================
// MAIN ENTRY
// =============================================================================

if (require.main === module) {
  const args = process.argv.slice(2);

  if (args.includes('--help') || args.includes('-h')) {
    printHelp();
    process.exit(0);
  }

  executeCLI(args)
    .then(code => process.exit(code))
    .catch(err => {
      console.error('Fatal error:', err);
      process.exit(1);
    });
}

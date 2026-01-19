/**
 * Adversarial Prompt Agent - CLI Integration
 *
 * CLI command spec for invoking the Adversarial Prompt Agent.
 * This integrates with agentics-cli.
 */

import * as fs from 'fs/promises';
import * as path from 'path';
import { handler, ADVERSARIAL_PROMPT_AGENT } from './handler';
import {
  AdversarialPromptInputSchema,
  AdversarialPromptCLIArgsSchema,
  AdversarialCategorySchema,
  AdversarialSeveritySchema,
  type AdversarialPromptCLIArgs,
  type AdversarialPromptInput,
  type AdversarialCategory,
  type AdversarialSeverity,
} from '../contracts';

// =============================================================================
// CLI COMMAND SPEC
// =============================================================================

export const CLI_COMMAND_SPEC = {
  name: 'adversarial-prompt',
  description: 'Generate adversarial prompts for Red Team and Stress Test agents',
  aliases: ['adv-prompt', 'ap'],
  usage: 'agentics adversarial-prompt [options]',
  examples: [
    'agentics adversarial-prompt --preset basic',
    'agentics adversarial-prompt --categories prompt_injection,encoding_attacks --count 10',
    'agentics adversarial-prompt --input-file config.json --output-format jsonl -o prompts.jsonl',
    'agentics adversarial-prompt --preset red-team --max-severity high',
    'cat config.json | agentics adversarial-prompt --input-stdin --output-format prompts-only',
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
      name: '--preset',
      short: '-p',
      description: 'Use a predefined preset (basic, comprehensive, red-team, encoding-focus, injection-focus)',
      type: 'string',
      required: false,
    },
    {
      name: '--categories',
      short: '-c',
      description: 'Comma-separated list of adversarial categories',
      type: 'string',
      required: false,
    },
    {
      name: '--max-severity',
      short: '-m',
      description: 'Maximum severity level (low, medium, high, critical)',
      type: 'string',
      default: 'high',
    },
    {
      name: '--count',
      short: '-n',
      description: 'Number of prompts per category',
      type: 'number',
      default: 5,
    },
    {
      name: '--output-format',
      short: '-f',
      description: 'Output format: json, jsonl, csv, prompts-only',
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
      name: '--include-benign',
      description: 'Include benign variants for comparison',
      type: 'boolean',
      default: true,
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
// PRESETS
// =============================================================================

const PRESETS: Record<string, Partial<AdversarialPromptInput>> = {
  basic: {
    categories: ['prompt_injection', 'encoding_attacks'],
    severities: ['low', 'medium'],
    count_per_category: 5,
    strategy: 'template_based',
    purpose: 'stress_testing',
  },
  comprehensive: {
    categories: [
      'prompt_injection',
      'encoding_attacks',
      'jailbreak_attempt',
      'delimiter_attacks',
      'format_confusion',
      'hallucination_triggers',
    ],
    severities: ['low', 'medium', 'high'],
    count_per_category: 10,
    strategy: 'template_based',
    purpose: 'security_audit',
  },
  'red-team': {
    categories: [
      'prompt_injection',
      'jailbreak_attempt',
      'system_prompt_extraction',
      'authority_impersonation',
      'urgency_manipulation',
    ],
    severities: ['medium', 'high'],
    count_per_category: 15,
    strategy: 'combinatorial',
    purpose: 'red_team_testing',
  },
  'encoding-focus': {
    categories: [
      'encoding_attacks',
      'delimiter_attacks',
      'whitespace_exploitation',
      'format_confusion',
    ],
    severities: ['low', 'medium'],
    count_per_category: 10,
    strategy: 'mutation_based',
    purpose: 'stress_testing',
    mutation_config: {
      mutation_types: ['encoding_change', 'whitespace_injection'],
      mutation_rate: 0.5,
    },
  },
  'injection-focus': {
    categories: [
      'prompt_injection',
      'instruction_override',
      'delimiter_attacks',
      'system_prompt_extraction',
    ],
    severities: ['low', 'medium', 'high'],
    count_per_category: 10,
    strategy: 'template_based',
    purpose: 'security_audit',
  },
};

// =============================================================================
// CLI EXECUTOR
// =============================================================================

/**
 * Execute CLI command
 */
export async function executeCLI(args: string[]): Promise<number> {
  const parsedArgs = parseArgs(args);

  // Validate args
  const argsValidation = AdversarialPromptCLIArgsSchema.safeParse(parsedArgs);
  if (!argsValidation.success) {
    console.error('Invalid arguments:', argsValidation.error.message);
    return 1;
  }

  const cliArgs = argsValidation.data;

  // Build input from args or file
  let input: AdversarialPromptInput;
  try {
    input = await buildInput(cliArgs);
  } catch (err) {
    console.error('Failed to build input:', err instanceof Error ? err.message : err);
    return 1;
  }

  // Validate input
  const inputValidation = AdversarialPromptInputSchema.safeParse(input);
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
      console.log(`  Categories: ${inputValidation.data.categories.join(', ')}`);
      console.log(`  Severities: ${inputValidation.data.severities.join(', ')}`);
      console.log(`  Count per category: ${inputValidation.data.count_per_category}`);
      console.log(`  Strategy: ${inputValidation.data.strategy}`);
    }
    return 0;
  }

  // Execute handler
  if (cliArgs.verbose && !cliArgs.quiet) {
    console.log(`Executing ${ADVERSARIAL_PROMPT_AGENT.agent_id} v${ADVERSARIAL_PROMPT_AGENT.agent_version}`);
  }

  const response = await handler({
    body: inputValidation.data,
    headers: {},
    method: 'POST',
    path: '/adversarial-prompt',
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
    console.log(`Prompts generated: ${result.data.prompts.length}`);
    console.log(`Duration: ${result.data.duration_ms}ms`);
    console.log(`Diversity score: ${(result.data.quality_metrics.diversity_score * 100).toFixed(1)}%`);
    console.log(`Category coverage: ${(result.data.quality_metrics.category_coverage * 100).toFixed(1)}%`);

    if (result.data.warnings.length > 0) {
      console.log('\nWarnings:');
      for (const warning of result.data.warnings) {
        console.log(`  - ${warning}`);
      }
    }
  }

  return 0;
}

// =============================================================================
// HELPERS
// =============================================================================

function parseArgs(args: string[]): Partial<AdversarialPromptCLIArgs> {
  const result: Partial<AdversarialPromptCLIArgs> = {};
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
      case '--preset':
      case '-p':
        result.preset = args[++i] as AdversarialPromptCLIArgs['preset'];
        break;
      case '--categories':
      case '-c':
        result.categories = args[++i];
        break;
      case '--max-severity':
      case '-m':
        result.max_severity = args[++i] as AdversarialSeverity;
        break;
      case '--count':
      case '-n':
        result.count = parseInt(args[++i], 10);
        break;
      case '--output-format':
      case '-f':
        result.output_format = args[++i] as 'json' | 'jsonl' | 'csv' | 'prompts-only';
        break;
      case '--output-file':
      case '-o':
        result.output_file = args[++i];
        break;
      case '--include-benign':
        result.include_benign = true;
        break;
      case '--no-include-benign':
        result.include_benign = false;
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

async function buildInput(args: AdversarialPromptCLIArgs): Promise<AdversarialPromptInput> {
  // Start with file/json/stdin input if provided
  let baseInput: Partial<AdversarialPromptInput> = {};

  if (args.input_file) {
    const content = await fs.readFile(args.input_file, 'utf-8');
    baseInput = JSON.parse(content);
  } else if (args.input_json) {
    baseInput = JSON.parse(args.input_json);
  } else if (args.input_stdin) {
    const data = await readStdin();
    baseInput = JSON.parse(data);
  }

  // Apply preset if specified
  if (args.preset && PRESETS[args.preset]) {
    baseInput = { ...PRESETS[args.preset], ...baseInput };
  }

  // Apply CLI overrides
  if (args.categories) {
    const categoryList = args.categories.split(',').map(c => c.trim());
    // Validate categories
    const validCategories: AdversarialCategory[] = [];
    for (const cat of categoryList) {
      const parsed = AdversarialCategorySchema.safeParse(cat);
      if (parsed.success) {
        validCategories.push(parsed.data);
      } else {
        console.warn(`Invalid category "${cat}", skipping`);
      }
    }
    if (validCategories.length > 0) {
      baseInput.categories = validCategories;
    }
  }

  if (args.max_severity) {
    baseInput.safety_ceiling = args.max_severity;
    // Also set severities based on max
    const severityOrder: AdversarialSeverity[] = ['low', 'medium', 'high', 'critical'];
    const maxIndex = severityOrder.indexOf(args.max_severity);
    baseInput.severities = severityOrder.slice(0, maxIndex + 1);
  }

  if (args.count !== undefined) {
    baseInput.count_per_category = args.count;
  }

  if (args.include_benign !== undefined) {
    baseInput.include_benign_variants = args.include_benign;
  }

  // Set defaults for required fields if not provided
  if (!baseInput.categories || baseInput.categories.length === 0) {
    baseInput.categories = ['prompt_injection'];
  }

  if (!baseInput.severities || baseInput.severities.length === 0) {
    baseInput.severities = ['low', 'medium'];
  }

  return baseInput as AdversarialPromptInput;
}

async function readStdin(): Promise<string> {
  return new Promise((resolve, reject) => {
    let data = '';
    process.stdin.setEncoding('utf-8');
    process.stdin.on('data', chunk => (data += chunk));
    process.stdin.on('end', () => resolve(data));
    process.stdin.on('error', reject);
  });
}

function formatOutput(data: any, format: string): string {
  switch (format) {
    case 'json':
      return JSON.stringify(data, null, 2);

    case 'jsonl':
      return formatJSONL(data);

    case 'csv':
      return formatCSV(data);

    case 'prompts-only':
      return formatPromptsOnly(data);

    default:
      return JSON.stringify(data, null, 2);
  }
}

function formatJSONL(data: any): string {
  const prompts = data.prompts ?? [];
  return prompts.map((p: any) => JSON.stringify(p)).join('\n');
}

function formatCSV(data: any): string {
  const prompts = data.prompts ?? [];
  if (prompts.length === 0) return '';

  const headers = [
    'prompt_id',
    'category',
    'severity',
    'attack_vector',
    'estimated_tokens',
    'complexity_score',
    'prompt_text',
  ];

  const rows = prompts.map((p: any) => [
    p.prompt_id,
    p.category,
    p.severity,
    `"${(p.attack_vector || '').replace(/"/g, '""')}"`,
    p.estimated_tokens,
    p.complexity_score.toFixed(2),
    `"${(p.prompt_text || '').replace(/"/g, '""').replace(/\n/g, '\\n')}"`,
  ]);

  return [headers.join(','), ...rows.map((r: any[]) => r.join(','))].join('\n');
}

function formatPromptsOnly(data: any): string {
  const prompts = data.prompts ?? [];
  return prompts.map((p: any, i: number) =>
    `# Prompt ${i + 1} [${p.category}/${p.severity}]\n${p.prompt_text}\n`
  ).join('\n---\n\n');
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

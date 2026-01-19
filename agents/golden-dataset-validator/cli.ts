/**
 * Golden Dataset Validator Agent - CLI Integration
 *
 * CLI command spec for invoking the Golden Dataset Validator Agent.
 * This integrates with agentics-cli.
 */

import * as fs from 'fs/promises';
import * as path from 'path';
import {
  GoldenDatasetValidatorInputSchema,
  GoldenDatasetValidatorCLIArgsSchema,
  GOLDEN_DATASET_VALIDATOR_AGENT,
  type GoldenDatasetValidatorCLIArgs,
  type GoldenDatasetValidatorInput,
  type GoldenDatasetValidatorOutput,
  type SampleValidationResult,
} from '../contracts';

// =============================================================================
// CLI COMMAND SPEC
// =============================================================================

export const CLI_COMMAND_SPEC = {
  name: 'golden-dataset-validator',
  description: 'Validate model outputs against canonical, human-verified datasets',
  aliases: ['golden-validate', 'gdv'],
  usage: 'agentics golden-dataset-validator [options]',
  examples: [
    'agentics golden-dataset-validator --golden-file golden.json --outputs-file outputs.json',
    'agentics golden-dataset-validator --input-file combined.json --output-format summary',
    'agentics golden-dataset-validator --golden-file golden.json --outputs-file outputs.json --similarity-threshold 0.9',
    'cat combined.json | agentics golden-dataset-validator --input-stdin',
    'agentics golden-dataset-validator --input-file data.json --output-format report --output-file report.md',
  ],
  options: [
    {
      name: '--input-file',
      short: '-i',
      description: 'Path to combined input JSON file containing both golden samples and model outputs',
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
      name: '--golden-file',
      short: '-g',
      description: 'Path to golden dataset JSON file',
      type: 'string',
      required: false,
    },
    {
      name: '--golden-url',
      description: 'URL to fetch golden dataset from',
      type: 'string',
      required: false,
    },
    {
      name: '--outputs-file',
      short: '-m',
      description: 'Path to model outputs JSON file',
      type: 'string',
      required: false,
    },
    {
      name: '--outputs-url',
      description: 'URL to fetch model outputs from',
      type: 'string',
      required: false,
    },
    {
      name: '--exact-match-only',
      short: '-e',
      description: 'Only consider exact matches as passing',
      type: 'boolean',
      default: false,
    },
    {
      name: '--case-insensitive',
      short: '-c',
      description: 'Use case-insensitive comparison',
      type: 'boolean',
      default: false,
    },
    {
      name: '--similarity-threshold',
      short: '-t',
      description: 'Semantic similarity threshold (0-1)',
      type: 'number',
      default: 0.85,
    },
    {
      name: '--output-format',
      short: '-f',
      description: 'Output format: json, csv, table, summary, report',
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
    {
      name: '--fail-fast',
      description: 'Exit on first validation failure',
      type: 'boolean',
      default: false,
    },
  ],
} as const;

// =============================================================================
// HANDLER PLACEHOLDER
// =============================================================================

interface EdgeFunctionRequest {
  body: unknown;
  headers: Record<string, string>;
  method: string;
  path: string;
}

interface EdgeFunctionResponse {
  statusCode: number;
  headers: Record<string, string>;
  body: string;
}

// Placeholder handler - actual implementation in handler.ts
async function handler(request: EdgeFunctionRequest): Promise<EdgeFunctionResponse> {
  // Import and delegate to actual handler
  const { handler: actualHandler } = await import('./handler');
  return actualHandler(request);
}

// =============================================================================
// CLI EXECUTOR
// =============================================================================

/**
 * Execute CLI command
 */
export async function executeCLI(args: string[]): Promise<number> {
  const parsedArgs = parseArgs(args);

  // Validate args
  const argsValidation = GoldenDatasetValidatorCLIArgsSchema.safeParse(parsedArgs);
  if (!argsValidation.success) {
    console.error('Invalid arguments:', argsValidation.error.message);
    return 1;
  }

  const cliArgs = argsValidation.data;

  // Load input
  let input: GoldenDatasetValidatorInput;
  try {
    input = await loadInput(cliArgs);
  } catch (err) {
    console.error('Failed to load input:', err instanceof Error ? err.message : err);
    return 1;
  }

  // Validate input
  const inputValidation = GoldenDatasetValidatorInputSchema.safeParse(input);
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
      console.log(`  Golden samples: ${input.golden_samples.length}`);
      console.log(`  Model outputs: ${input.model_outputs.length}`);
      console.log(`  Dataset: ${input.dataset?.name || 'unnamed'}`);
    }
    return 0;
  }

  // Execute handler
  if (cliArgs.verbose && !cliArgs.quiet) {
    console.log(`Executing ${GOLDEN_DATASET_VALIDATOR_AGENT.agent_id} v${GOLDEN_DATASET_VALIDATOR_AGENT.agent_version}`);
  }

  const response = await handler({
    body: inputValidation.data,
    headers: {},
    method: 'POST',
    path: '/golden-dataset-validator',
  });

  // Parse response
  const result = JSON.parse(response.body);

  // Handle error
  if (!result.success) {
    console.error('Execution failed:', result.error?.message || 'Unknown error');
    return 1;
  }

  const data = result.data as GoldenDatasetValidatorOutput;

  // Check for fail-fast
  if (cliArgs.fail_fast && data.stats.failed > 0) {
    console.error(`Validation failed: ${data.stats.failed} failures detected`);
    return 1;
  }

  // Format output
  const output = formatOutput(data, cliArgs.output_format);

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
    console.log(`Grade: ${data.quality_assessment.grade} (${data.quality_assessment.score}/100)`);
    console.log(`Pass Rate: ${(data.stats.pass_rate * 100).toFixed(1)}%`);
    console.log(`Duration: ${data.duration_ms}ms`);
  }

  // Return non-zero if validation failed
  return data.stats.failed > 0 && cliArgs.fail_fast ? 1 : 0;
}

// =============================================================================
// HELPERS
// =============================================================================

function parseArgs(args: string[]): Partial<GoldenDatasetValidatorCLIArgs> {
  const result: Partial<GoldenDatasetValidatorCLIArgs> = {};
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
      case '--golden-file':
      case '-g':
        result.golden_file = args[++i];
        break;
      case '--golden-url':
        result.golden_url = args[++i];
        break;
      case '--outputs-file':
      case '-m':
        result.outputs_file = args[++i];
        break;
      case '--outputs-url':
        result.outputs_url = args[++i];
        break;
      case '--exact-match-only':
      case '-e':
        result.exact_match_only = true;
        break;
      case '--case-insensitive':
      case '-c':
        result.case_insensitive = true;
        break;
      case '--similarity-threshold':
      case '-t':
        result.similarity_threshold = parseFloat(args[++i]);
        break;
      case '--output-format':
      case '-f':
        result.output_format = args[++i] as 'json' | 'csv' | 'table' | 'summary' | 'report';
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
      case '--fail-fast':
        result.fail_fast = true;
        break;
    }

    i++;
  }

  return result;
}

async function loadInput(args: GoldenDatasetValidatorCLIArgs): Promise<GoldenDatasetValidatorInput> {
  let baseInput: Partial<GoldenDatasetValidatorInput> = {};

  // Load combined input from various sources
  if (args.input_file) {
    const content = await fs.readFile(args.input_file, 'utf-8');
    baseInput = JSON.parse(content);
  } else if (args.input_json) {
    baseInput = JSON.parse(args.input_json);
  } else if (args.input_stdin) {
    const data = await readStdin();
    baseInput = JSON.parse(data);
  }

  // Load golden samples separately if provided
  if (args.golden_file) {
    const content = await fs.readFile(args.golden_file, 'utf-8');
    const parsed = JSON.parse(content);
    baseInput.golden_samples = Array.isArray(parsed) ? parsed : parsed.golden_samples || parsed.samples;
  } else if (args.golden_url) {
    const response = await fetch(args.golden_url);
    const parsed = await response.json();
    baseInput.golden_samples = Array.isArray(parsed) ? parsed : parsed.golden_samples || parsed.samples;
  }

  // Load model outputs separately if provided
  if (args.outputs_file) {
    const content = await fs.readFile(args.outputs_file, 'utf-8');
    const parsed = JSON.parse(content);
    baseInput.model_outputs = Array.isArray(parsed) ? parsed : parsed.model_outputs || parsed.outputs;
  } else if (args.outputs_url) {
    const response = await fetch(args.outputs_url);
    const parsed = await response.json();
    baseInput.model_outputs = Array.isArray(parsed) ? parsed : parsed.model_outputs || parsed.outputs;
  }

  // Apply CLI options to validation config
  baseInput.validation_config = {
    ...baseInput.validation_config,
    case_insensitive: args.case_insensitive || baseInput.validation_config?.case_insensitive || false,
    semantic_similarity_threshold: args.similarity_threshold ?? baseInput.validation_config?.semantic_similarity_threshold ?? 0.85,
  };

  if (args.exact_match_only) {
    baseInput.validation_config.enable_semantic_similarity = false;
    baseInput.validation_config.enable_keyword_analysis = false;
  }

  if (!baseInput.golden_samples || baseInput.golden_samples.length === 0) {
    throw new Error('No golden samples provided. Use --golden-file, --input-file, or --input-stdin');
  }

  if (!baseInput.model_outputs || baseInput.model_outputs.length === 0) {
    throw new Error('No model outputs provided. Use --outputs-file, --input-file, or --input-stdin');
  }

  return baseInput as GoldenDatasetValidatorInput;
}

function readStdin(): Promise<string> {
  return new Promise((resolve, reject) => {
    let data = '';
    process.stdin.setEncoding('utf-8');
    process.stdin.on('data', chunk => (data += chunk));
    process.stdin.on('end', () => resolve(data));
    process.stdin.on('error', reject);
  });
}

function formatOutput(data: GoldenDatasetValidatorOutput, format: string): string {
  switch (format) {
    case 'json':
      return JSON.stringify(data, null, 2);

    case 'csv':
      return formatCSV(data);

    case 'table':
      return formatTable(data);

    case 'summary':
      return formatSummary(data);

    case 'report':
      return formatReport(data);

    default:
      return JSON.stringify(data, null, 2);
  }
}

function formatCSV(data: GoldenDatasetValidatorOutput): string {
  const results = data.results ?? [];
  if (results.length === 0) return '';

  const headers = [
    'sample_id',
    'passed',
    'match_type',
    'severity',
    'confidence',
    'semantic_similarity',
    'exact_match',
    'category',
  ];

  const rows = results.map(r => [
    r.sample_id,
    r.passed,
    r.match_type,
    r.severity,
    r.confidence.toFixed(3),
    r.semantic_similarity?.toFixed(3) ?? 'N/A',
    r.exact_match,
    r.category ?? '',
  ]);

  return [headers.join(','), ...rows.map(r => r.join(','))].join('\n');
}

function formatTable(data: GoldenDatasetValidatorOutput): string {
  const results = data.results ?? [];
  if (results.length === 0) return 'No samples validated';

  const header = `| Sample ID | Passed | Match Type | Severity | Confidence | Sim |`;
  const separator = `|-----------|--------|------------|----------|------------|-----|`;

  const rows = results.slice(0, 50).map(r => {
    const passedIcon = r.passed ? 'YES' : 'NO';
    const simScore = r.semantic_similarity !== null ? r.semantic_similarity.toFixed(2) : 'N/A';
    return `| ${r.sample_id.slice(0, 9).padEnd(9)} | ${passedIcon.padEnd(6)} | ${r.match_type.slice(0, 10).padEnd(10)} | ${r.severity.padEnd(8)} | ${r.confidence.toFixed(2).padStart(10)} | ${simScore.padStart(3)} |`;
  });

  if (results.length > 50) {
    rows.push(`| ... and ${results.length - 50} more samples |`);
  }

  return [header, separator, ...rows].join('\n');
}

function formatSummary(data: GoldenDatasetValidatorOutput): string {
  const lines: string[] = [];

  lines.push('='.repeat(60));
  lines.push('GOLDEN DATASET VALIDATION SUMMARY');
  lines.push('='.repeat(60));
  lines.push('');
  lines.push(`Validation ID: ${data.validation_id}`);
  lines.push(`Dataset: ${data.dataset.name}${data.dataset.version ? ` v${data.dataset.version}` : ''}`);
  lines.push(`Started: ${data.started_at}`);
  lines.push(`Completed: ${data.completed_at}`);
  lines.push(`Duration: ${data.duration_ms}ms`);
  lines.push('');
  lines.push('-'.repeat(60));
  lines.push('QUALITY ASSESSMENT');
  lines.push('-'.repeat(60));
  lines.push(`Grade: ${data.quality_assessment.grade}`);
  lines.push(`Score: ${data.quality_assessment.score}/100`);
  lines.push(`Summary: ${data.quality_assessment.summary}`);
  lines.push('');
  lines.push('-'.repeat(60));
  lines.push('STATISTICS');
  lines.push('-'.repeat(60));
  lines.push(`Total Samples: ${data.stats.total_samples}`);
  lines.push(`Passed: ${data.stats.passed} (${(data.stats.pass_rate * 100).toFixed(1)}%)`);
  lines.push(`Failed: ${data.stats.failed}`);
  lines.push('');
  lines.push('Match Types:');
  lines.push(`  Exact Match: ${data.stats.exact_matches} (${(data.stats.exact_match_rate * 100).toFixed(1)}%)`);
  lines.push(`  Semantic Match: ${data.stats.semantic_matches} (${(data.stats.semantic_match_rate * 100).toFixed(1)}%)`);
  lines.push(`  Partial Match: ${data.stats.partial_matches}`);
  lines.push(`  No Match: ${data.stats.no_matches}`);
  lines.push(`  Errors: ${data.stats.errors}`);
  lines.push('');
  lines.push('Severity:');
  lines.push(`  Pass: ${data.stats.by_severity.pass}`);
  lines.push(`  Warning: ${data.stats.by_severity.warning}`);
  lines.push(`  Fail: ${data.stats.by_severity.fail}`);
  lines.push(`  Critical: ${data.stats.by_severity.critical}`);
  lines.push('');
  lines.push('Averages:');
  lines.push(`  Semantic Similarity: ${(data.stats.avg_semantic_similarity * 100).toFixed(1)}%`);
  lines.push(`  Keyword Overlap: ${(data.stats.avg_keyword_overlap * 100).toFixed(1)}%`);
  lines.push(`  Confidence: ${(data.stats.avg_confidence * 100).toFixed(1)}%`);

  if (data.quality_assessment.recommendations && data.quality_assessment.recommendations.length > 0) {
    lines.push('');
    lines.push('-'.repeat(60));
    lines.push('RECOMMENDATIONS');
    lines.push('-'.repeat(60));
    for (const rec of data.quality_assessment.recommendations) {
      lines.push(`  - ${rec}`);
    }
  }

  lines.push('');
  lines.push('='.repeat(60));

  return lines.join('\n');
}

function formatReport(data: GoldenDatasetValidatorOutput): string {
  const lines: string[] = [];

  lines.push('# Golden Dataset Validation Report');
  lines.push('');
  lines.push(`**Validation ID:** ${data.validation_id}`);
  lines.push(`**Date:** ${data.completed_at}`);
  lines.push(`**Duration:** ${data.duration_ms}ms`);
  lines.push('');

  lines.push('## Executive Summary');
  lines.push('');
  lines.push(`| Metric | Value |`);
  lines.push(`|--------|-------|`);
  lines.push(`| Grade | **${data.quality_assessment.grade}** |`);
  lines.push(`| Score | ${data.quality_assessment.score}/100 |`);
  lines.push(`| Pass Rate | ${(data.stats.pass_rate * 100).toFixed(1)}% |`);
  lines.push(`| Exact Match Rate | ${(data.stats.exact_match_rate * 100).toFixed(1)}% |`);
  lines.push('');
  lines.push(`> ${data.quality_assessment.summary}`);
  lines.push('');

  lines.push('## Dataset Information');
  lines.push('');
  lines.push(`- **Name:** ${data.dataset.name}`);
  if (data.dataset.version) {
    lines.push(`- **Version:** ${data.dataset.version}`);
  }
  lines.push(`- **Sample Count:** ${data.dataset.sample_count}`);
  if (data.model_info) {
    lines.push(`- **Model:** ${data.model_info.model_id || 'Unknown'}${data.model_info.provider ? ` (${data.model_info.provider})` : ''}`);
  }
  lines.push('');

  lines.push('## Detailed Statistics');
  lines.push('');
  lines.push('### Match Type Breakdown');
  lines.push('');
  lines.push(`| Match Type | Count | Percentage |`);
  lines.push(`|------------|-------|------------|`);
  lines.push(`| Exact Match | ${data.stats.by_match_type.exact_match} | ${((data.stats.by_match_type.exact_match / data.stats.total_samples) * 100).toFixed(1)}% |`);
  lines.push(`| Semantic Match | ${data.stats.by_match_type.semantic_match} | ${((data.stats.by_match_type.semantic_match / data.stats.total_samples) * 100).toFixed(1)}% |`);
  lines.push(`| Partial Match | ${data.stats.by_match_type.partial_match} | ${((data.stats.by_match_type.partial_match / data.stats.total_samples) * 100).toFixed(1)}% |`);
  lines.push(`| Structural Match | ${data.stats.by_match_type.structural_match} | ${((data.stats.by_match_type.structural_match / data.stats.total_samples) * 100).toFixed(1)}% |`);
  lines.push(`| No Match | ${data.stats.by_match_type.no_match} | ${((data.stats.by_match_type.no_match / data.stats.total_samples) * 100).toFixed(1)}% |`);
  lines.push(`| Error | ${data.stats.by_match_type.error} | ${((data.stats.by_match_type.error / data.stats.total_samples) * 100).toFixed(1)}% |`);
  lines.push('');

  lines.push('### Severity Breakdown');
  lines.push('');
  lines.push(`| Severity | Count |`);
  lines.push(`|----------|-------|`);
  lines.push(`| Pass | ${data.stats.by_severity.pass} |`);
  lines.push(`| Warning | ${data.stats.by_severity.warning} |`);
  lines.push(`| Fail | ${data.stats.by_severity.fail} |`);
  lines.push(`| Critical | ${data.stats.by_severity.critical} |`);
  lines.push('');

  if (data.quality_assessment.recommendations && data.quality_assessment.recommendations.length > 0) {
    lines.push('## Recommendations');
    lines.push('');
    for (const rec of data.quality_assessment.recommendations) {
      lines.push(`- ${rec}`);
    }
    lines.push('');
  }

  // Show top failures
  const failures = data.results.filter(r => !r.passed).slice(0, 10);
  if (failures.length > 0) {
    lines.push('## Top Failures');
    lines.push('');
    for (const failure of failures) {
      lines.push(`### Sample: ${failure.sample_id}`);
      lines.push('');
      lines.push(`- **Match Type:** ${failure.match_type}`);
      lines.push(`- **Severity:** ${failure.severity}`);
      lines.push(`- **Confidence:** ${(failure.confidence * 100).toFixed(1)}%`);
      lines.push(`- **Explanation:** ${failure.explanation}`);
      lines.push('');
    }
  }

  lines.push('---');
  lines.push('');
  lines.push(`*Generated by ${GOLDEN_DATASET_VALIDATOR_AGENT.agent_id} v${GOLDEN_DATASET_VALIDATOR_AGENT.agent_version}*`);

  return lines.join('\n');
}

// =============================================================================
// OPENAPI SPEC GENERATOR
// =============================================================================

/**
 * Generate OpenAPI spec for CLI documentation
 */
export function generateOpenAPISpec(): object {
  return {
    openapi: '3.0.3',
    info: {
      title: 'Golden Dataset Validator Agent CLI',
      version: GOLDEN_DATASET_VALIDATOR_AGENT.agent_version,
      description: 'CLI interface for the Golden Dataset Validator Agent',
    },
    paths: {
      '/cli/golden-dataset-validator': {
        post: {
          summary: 'Execute golden dataset validation via CLI',
          operationId: 'cliValidateGoldenDataset',
          tags: ['CLI'],
          requestBody: {
            required: true,
            content: {
              'application/json': {
                schema: {
                  type: 'object',
                  properties: {
                    args: {
                      type: 'array',
                      items: { type: 'string' },
                      description: 'CLI arguments',
                    },
                  },
                },
              },
            },
          },
          responses: {
            '200': {
              description: 'CLI execution completed',
              content: {
                'application/json': {
                  schema: {
                    type: 'object',
                    properties: {
                      exitCode: { type: 'integer' },
                      stdout: { type: 'string' },
                      stderr: { type: 'string' },
                    },
                  },
                },
              },
            },
          },
        },
      },
    },
  };
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

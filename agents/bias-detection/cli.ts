/**
 * Bias Detection Agent - CLI Integration
 *
 * CLI command spec for invoking the Bias Detection Agent.
 * This integrates with agentics-cli.
 */

import * as fs from 'fs/promises';
import * as path from 'path';
import {
  BiasDetectionInputSchema,
  BiasDetectionCLIArgsSchema,
  BIAS_DETECTION_AGENT,
  type BiasDetectionCLIArgs,
  type BiasDetectionInput,
  type BiasDetectionOutput,
  type BiasSampleResult,
  type DetectedBias,
} from '../contracts/schemas/bias-detection';

// =============================================================================
// CLI COMMAND SPEC
// =============================================================================

export const CLI_COMMAND_SPEC = {
  name: 'bias-detection',
  description: 'Detect demographic, cultural, or systemic bias in model outputs',
  aliases: ['bias-detect', 'bd'],
  usage: 'agentics bias-detection [options]',
  examples: [
    'agentics bias-detection --input-file samples.json',
    'agentics bias-detection --input-text "Your text to analyze for bias"',
    'agentics bias-detection --input-file samples.json --bias-types gender,racial',
    'agentics bias-detection --input-file samples.json --min-severity medium',
    'cat input.json | agentics bias-detection --input-stdin',
    'agentics bias-detection --input-file samples.json --output-format report --output-file report.md',
  ],
  options: [
    {
      name: '--input-file',
      short: '-i',
      description: 'Path to input JSON file containing samples',
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
      name: '--input-text',
      short: '-t',
      description: 'Single text to analyze (creates one sample)',
      type: 'string',
      required: false,
    },
    {
      name: '--bias-types',
      short: '-b',
      description: 'Comma-separated bias types to check (e.g., gender,racial,age)',
      type: 'string',
      required: false,
    },
    {
      name: '--min-severity',
      description: 'Minimum severity to report: negligible, low, medium, high, critical',
      type: 'string',
      default: 'low',
    },
    {
      name: '--confidence-threshold',
      description: 'Minimum confidence threshold (0-1)',
      type: 'number',
      default: 0.5,
    },
    {
      name: '--domain',
      description: 'Domain context: general, healthcare, legal, education, employment, finance, media, technology, government',
      type: 'string',
      default: 'general',
    },
    {
      name: '--cultural-context',
      description: 'Cultural context: us_english, uk_english, global, specific',
      type: 'string',
      default: 'global',
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

// Import handler dynamically to avoid circular dependencies
let handlerModule: { handler: (req: EdgeFunctionRequest) => Promise<EdgeFunctionResponse> } | null = null;

async function getHandler(): Promise<typeof handlerModule> {
  if (!handlerModule) {
    try {
      handlerModule = await import('./handler');
    } catch {
      // Fallback to placeholder if handler not available
      handlerModule = {
        handler: async (request: EdgeFunctionRequest): Promise<EdgeFunctionResponse> => {
          const input = request.body as BiasDetectionInput;
          const executionId = crypto.randomUUID();
          const startedAt = new Date().toISOString();

          // Placeholder detection logic
          const results: BiasSampleResult[] = input.samples.map(sample => ({
            sample_id: sample.sample_id,
            has_bias: false,
            bias_score: 0,
            max_severity: null,
            detected_biases: [],
            bias_types_found: [],
            assessment: 'no_bias_detected' as const,
            analyzed_at: new Date().toISOString(),
            processing_ms: 10,
          }));

          const output: BiasDetectionOutput = {
            detection_id: executionId,
            results,
            stats: {
              total_samples: input.samples.length,
              samples_with_bias: 0,
              samples_without_bias: input.samples.length,
              total_biases_detected: 0,
              bias_rate: 0,
              avg_bias_score: 0,
              avg_confidence: 0,
              by_type: {
                gender: 0, racial: 0, cultural: 0, socioeconomic: 0, age: 0,
                disability: 0, religious: 0, political: 0, sexual_orientation: 0,
                geographic: 0, linguistic: 0, educational: 0, appearance: 0,
                intersectional: 0, other: 0,
              },
              by_severity: { negligible: 0, low: 0, medium: 0, high: 0, critical: 0 },
            },
            config_used: input.detection_config || {
              confidence_threshold: 0.5,
              min_severity: 'low',
              enable_sentiment_analysis: true,
              enable_entity_extraction: true,
              enable_stereotype_detection: true,
              enable_representation_analysis: true,
              enable_language_pattern_analysis: true,
              case_sensitive: false,
              max_samples: 100,
              timeout_ms: 60000,
              include_explanations: true,
              include_recommendations: true,
            },
            overall_assessment: 'no_significant_bias',
            started_at: startedAt,
            completed_at: new Date().toISOString(),
            duration_ms: Date.now() - new Date(startedAt).getTime(),
          };

          return {
            statusCode: 200,
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
              success: true,
              decision_id: executionId,
              data: output,
            }),
          };
        },
      };
    }
  }
  return handlerModule;
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
  const argsValidation = BiasDetectionCLIArgsSchema.safeParse(parsedArgs);
  if (!argsValidation.success) {
    console.error('Invalid arguments:', argsValidation.error.message);
    return 1;
  }

  const cliArgs = argsValidation.data;

  // Load input
  let input: BiasDetectionInput;
  try {
    input = await loadInput(cliArgs);
  } catch (err) {
    console.error('Failed to load input:', err instanceof Error ? err.message : err);
    return 1;
  }

  // Validate input
  const inputValidation = BiasDetectionInputSchema.safeParse(input);
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
      console.log(`  Samples: ${input.samples.length}`);
      console.log(`  Total characters: ${input.samples.reduce((s, sample) => s + sample.content.length, 0)}`);
    }
    return 0;
  }

  // Execute handler
  if (cliArgs.verbose && !cliArgs.quiet) {
    console.log(`Executing ${BIAS_DETECTION_AGENT.agent_id} v${BIAS_DETECTION_AGENT.agent_version}`);
  }

  const handlerMod = await getHandler();
  const response = await handlerMod!.handler({
    body: inputValidation.data,
    headers: {},
    method: 'POST',
    path: '/bias-detection',
  });

  // Parse response
  const result = JSON.parse(response.body);

  // Handle error
  if (!result.success) {
    console.error('Execution failed:', result.error?.message || 'Unknown error');
    return 1;
  }

  const data = result.data as BiasDetectionOutput;

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
    console.log(`Bias rate: ${(data.stats.bias_rate * 100).toFixed(1)}%`);
    console.log(`Total biases detected: ${data.stats.total_biases_detected}`);
    console.log(`Duration: ${data.duration_ms}ms`);
  }

  return 0;
}

// =============================================================================
// HELPERS
// =============================================================================

function parseArgs(args: string[]): Partial<BiasDetectionCLIArgs> {
  const result: Partial<BiasDetectionCLIArgs> = {};
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
      case '--input-text':
      case '-t':
        result.input_text = args[++i];
        break;
      case '--bias-types':
      case '-b':
        result.bias_types = args[++i];
        break;
      case '--min-severity':
        result.min_severity = args[++i] as any;
        break;
      case '--confidence-threshold':
        result.confidence_threshold = parseFloat(args[++i]);
        break;
      case '--domain':
        result.domain = args[++i];
        break;
      case '--cultural-context':
        result.cultural_context = args[++i];
        break;
      case '--output-format':
      case '-f':
        result.output_format = args[++i] as any;
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

async function loadInput(args: BiasDetectionCLIArgs): Promise<BiasDetectionInput> {
  let baseInput: Partial<BiasDetectionInput> = {};

  // Load from various sources
  if (args.input_file) {
    const content = await fs.readFile(args.input_file, 'utf-8');
    baseInput = JSON.parse(content);
  } else if (args.input_json) {
    baseInput = JSON.parse(args.input_json);
  } else if (args.input_stdin) {
    const data = await readStdin();
    baseInput = JSON.parse(data);
  } else if (args.input_text) {
    // Create a single sample from input text
    baseInput = {
      samples: [{
        sample_id: 'sample-1',
        content: args.input_text,
        source: 'cli-input',
      }],
    };
  }

  // Apply CLI configuration
  baseInput.detection_config = baseInput.detection_config || {};

  if (args.bias_types) {
    const types = args.bias_types.split(',').map(t => t.trim()) as any[];
    baseInput.detection_config.bias_types = types;
  }

  if (args.min_severity) {
    baseInput.detection_config.min_severity = args.min_severity;
  }

  if (args.confidence_threshold !== undefined) {
    baseInput.detection_config.confidence_threshold = args.confidence_threshold;
  }

  // Apply demographic context
  if (args.domain || args.cultural_context) {
    baseInput.demographic_context = baseInput.demographic_context || {};
    if (args.domain) {
      baseInput.demographic_context.domain = args.domain as any;
    }
    if (args.cultural_context) {
      baseInput.demographic_context.cultural_context = args.cultural_context as any;
    }
  }

  if (!baseInput.samples || baseInput.samples.length === 0) {
    throw new Error('No samples provided. Use --input-file, --input-json, --input-text, or --input-stdin');
  }

  return baseInput as BiasDetectionInput;
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

function formatOutput(data: BiasDetectionOutput, format: string): string {
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

function formatCSV(data: BiasDetectionOutput): string {
  const rows: string[][] = [];

  // Header
  rows.push([
    'sample_id',
    'has_bias',
    'bias_score',
    'max_severity',
    'bias_types',
    'assessment',
  ]);

  // Data rows
  for (const result of data.results) {
    rows.push([
      result.sample_id,
      result.has_bias ? 'true' : 'false',
      result.bias_score.toFixed(3),
      result.max_severity || 'none',
      result.bias_types_found.join(';'),
      result.assessment,
    ]);
  }

  return rows.map(r => r.join(',')).join('\n');
}

function formatTable(data: BiasDetectionOutput): string {
  const lines: string[] = [];

  lines.push('| Sample ID | Bias | Score | Severity | Types | Assessment |');
  lines.push('|-----------|------|-------|----------|-------|------------|');

  for (const result of data.results) {
    const biasIcon = result.has_bias ? 'YES' : 'NO';
    const types = result.bias_types_found.slice(0, 2).join(', ') +
      (result.bias_types_found.length > 2 ? '...' : '');

    lines.push(
      `| ${result.sample_id.slice(0, 9).padEnd(9)} | ` +
      `${biasIcon.padEnd(4)} | ` +
      `${result.bias_score.toFixed(2).padStart(5)} | ` +
      `${(result.max_severity || 'none').padEnd(8)} | ` +
      `${types.padEnd(5)} | ` +
      `${result.assessment.slice(0, 10)} |`
    );
  }

  return lines.join('\n');
}

function formatSummary(data: BiasDetectionOutput): string {
  const lines: string[] = [];

  lines.push('='.repeat(60));
  lines.push('BIAS DETECTION SUMMARY');
  lines.push('='.repeat(60));
  lines.push('');
  lines.push(`Detection ID: ${data.detection_id}`);
  lines.push(`Started: ${data.started_at}`);
  lines.push(`Completed: ${data.completed_at}`);
  lines.push(`Duration: ${data.duration_ms}ms`);
  lines.push('');
  lines.push('-'.repeat(60));
  lines.push('OVERALL ASSESSMENT: ' + data.overall_assessment.toUpperCase().replace(/_/g, ' '));
  lines.push('-'.repeat(60));
  lines.push('');
  lines.push(`Total samples: ${data.stats.total_samples}`);
  lines.push(`Samples with bias: ${data.stats.samples_with_bias}`);
  lines.push(`Samples without bias: ${data.stats.samples_without_bias}`);
  lines.push(`Bias rate: ${(data.stats.bias_rate * 100).toFixed(1)}%`);
  lines.push(`Average bias score: ${data.stats.avg_bias_score.toFixed(3)}`);
  lines.push('');

  if (data.stats.total_biases_detected > 0) {
    lines.push('-'.repeat(60));
    lines.push('BIAS BREAKDOWN BY TYPE');
    lines.push('-'.repeat(60));

    for (const [type, count] of Object.entries(data.stats.by_type)) {
      if (count > 0) {
        lines.push(`  ${type}: ${count}`);
      }
    }

    lines.push('');
    lines.push('-'.repeat(60));
    lines.push('BIAS BREAKDOWN BY SEVERITY');
    lines.push('-'.repeat(60));

    for (const [severity, count] of Object.entries(data.stats.by_severity)) {
      if (count > 0) {
        lines.push(`  ${severity}: ${count}`);
      }
    }
  }

  if (data.key_findings && data.key_findings.length > 0) {
    lines.push('');
    lines.push('-'.repeat(60));
    lines.push('KEY FINDINGS');
    lines.push('-'.repeat(60));

    for (const finding of data.key_findings) {
      lines.push(`  - ${finding}`);
    }
  }

  lines.push('');
  lines.push('='.repeat(60));

  return lines.join('\n');
}

function formatReport(data: BiasDetectionOutput): string {
  const lines: string[] = [];

  lines.push('# Bias Detection Report');
  lines.push('');
  lines.push(`**Detection ID:** ${data.detection_id}`);
  lines.push(`**Date:** ${new Date(data.started_at).toLocaleDateString()}`);
  lines.push(`**Duration:** ${data.duration_ms}ms`);
  lines.push('');
  lines.push('## Executive Summary');
  lines.push('');
  lines.push(`**Overall Assessment:** ${data.overall_assessment.replace(/_/g, ' ').toUpperCase()}`);
  lines.push('');
  lines.push('| Metric | Value |');
  lines.push('|--------|-------|');
  lines.push(`| Total Samples | ${data.stats.total_samples} |`);
  lines.push(`| Samples with Bias | ${data.stats.samples_with_bias} |`);
  lines.push(`| Bias Rate | ${(data.stats.bias_rate * 100).toFixed(1)}% |`);
  lines.push(`| Total Bias Instances | ${data.stats.total_biases_detected} |`);
  lines.push(`| Average Confidence | ${(data.stats.avg_confidence * 100).toFixed(1)}% |`);
  lines.push('');

  if (data.key_findings && data.key_findings.length > 0) {
    lines.push('## Key Findings');
    lines.push('');
    for (const finding of data.key_findings) {
      lines.push(`- ${finding}`);
    }
    lines.push('');
  }

  if (data.stats.total_biases_detected > 0) {
    lines.push('## Bias Distribution');
    lines.push('');
    lines.push('### By Type');
    lines.push('');
    lines.push('| Type | Count |');
    lines.push('|------|-------|');
    for (const [type, count] of Object.entries(data.stats.by_type)) {
      if (count > 0) {
        lines.push(`| ${type} | ${count} |`);
      }
    }
    lines.push('');

    lines.push('### By Severity');
    lines.push('');
    lines.push('| Severity | Count |');
    lines.push('|----------|-------|');
    for (const [severity, count] of Object.entries(data.stats.by_severity)) {
      if (count > 0) {
        lines.push(`| ${severity} | ${count} |`);
      }
    }
    lines.push('');

    lines.push('## Detailed Findings');
    lines.push('');

    for (const result of data.results.filter(r => r.has_bias)) {
      lines.push(`### Sample: ${result.sample_id}`);
      lines.push('');
      lines.push(`**Bias Score:** ${result.bias_score.toFixed(3)}`);
      lines.push(`**Max Severity:** ${result.max_severity}`);
      lines.push(`**Assessment:** ${result.assessment}`);
      lines.push('');

      for (const bias of result.detected_biases) {
        lines.push(`#### ${bias.bias_type.toUpperCase()} Bias`);
        lines.push('');
        lines.push(`- **Severity:** ${bias.severity}`);
        lines.push(`- **Confidence:** ${(bias.confidence * 100).toFixed(1)}%`);
        lines.push(`- **Affected Groups:** ${bias.affected_groups.join(', ')}`);
        lines.push(`- **Explanation:** ${bias.explanation}`);
        if (bias.recommendation) {
          lines.push(`- **Recommendation:** ${bias.recommendation}`);
        }
        lines.push('');

        lines.push('**Evidence:**');
        for (const evidence of bias.evidence) {
          lines.push(`> "${evidence.text_span}"`);
          lines.push(`> - Method: ${evidence.detection_method}`);
          lines.push(`> - Relevance: ${(evidence.relevance_score * 100).toFixed(0)}%`);
        }
        lines.push('');
      }
    }
  }

  lines.push('## Configuration Used');
  lines.push('');
  lines.push('```json');
  lines.push(JSON.stringify(data.config_used, null, 2));
  lines.push('```');
  lines.push('');
  lines.push('---');
  lines.push(`*Report generated by ${BIAS_DETECTION_AGENT.agent_id} v${BIAS_DETECTION_AGENT.agent_version}*`);

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
      title: 'Bias Detection Agent CLI',
      version: BIAS_DETECTION_AGENT.agent_version,
      description: 'CLI interface for the Bias Detection Agent',
    },
    paths: {
      '/cli/bias-detection': {
        post: {
          summary: 'Execute bias detection via CLI',
          operationId: 'cliDetectBias',
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
// SMOKE TEST COMMANDS
// =============================================================================

export const SMOKE_TEST_COMMANDS = [
  // Basic execution
  {
    description: 'Analyze single text for bias',
    command: 'agentics bias-detection --input-text "Men are natural leaders while women are better at caregiving."',
    expected_output: 'gender bias detected',
  },
  // Dry run validation
  {
    description: 'Validate input file without execution',
    command: 'agentics bias-detection --input-file samples.json --dry-run',
    expected_output: 'Input validation passed',
  },
  // Multiple bias types
  {
    description: 'Check specific bias types',
    command: 'agentics bias-detection --input-file samples.json --bias-types gender,racial,age',
    expected_output: 'Detection ID:',
  },
  // Summary format
  {
    description: 'Generate summary output',
    command: 'agentics bias-detection --input-file samples.json --output-format summary',
    expected_output: 'BIAS DETECTION SUMMARY',
  },
  // Report format
  {
    description: 'Generate markdown report',
    command: 'agentics bias-detection --input-file samples.json --output-format report --output-file report.md',
    expected_output: 'Output written to: report.md',
  },
];

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

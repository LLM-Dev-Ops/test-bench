/**
 * Hallucination Detector Agent - CLI Integration
 *
 * CLI command spec for invoking the Hallucination Detector Agent.
 * This integrates with agentics-cli.
 */

import * as fs from 'fs/promises';
import * as path from 'path';
import {
  HallucinationDetectorInputSchema,
  HallucinationDetectorCLIArgsSchema,
  HALLUCINATION_DETECTOR_AGENT,
  type HallucinationDetectorCLIArgs,
  type HallucinationDetectorInput,
  type HallucinationDetectorOutput,
  type HallucinationClaimResult,
} from '../contracts';

// =============================================================================
// CLI COMMAND SPEC
// =============================================================================

export const CLI_COMMAND_SPEC = {
  name: 'hallucination-detector',
  description: 'Detect unsupported or fabricated claims relative to reference context',
  aliases: ['halluc-detect', 'hd'],
  usage: 'agentics hallucination-detector [options]',
  examples: [
    'agentics hallucination-detector --input-file claims.json --reference-file context.txt',
    'agentics hallucination-detector --input-json \'{"claim":"...","reference_context":"..."}\'',
    'agentics hallucination-detector --input-file claims.json --reference-file context.txt --sensitivity high',
    'cat input.json | agentics hallucination-detector --input-stdin',
    'agentics hallucination-detector --input-file claims.json --reference-file context.txt --output-format table',
  ],
  options: [
    {
      name: '--input-file',
      short: '-i',
      description: 'Path to input JSON file containing claims',
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
      name: '--reference-file',
      short: '-r',
      description: 'Path to reference context file (text or JSON)',
      type: 'string',
      required: false,
    },
    {
      name: '--reference-text',
      short: '-t',
      description: 'Reference context as inline text',
      type: 'string',
      required: false,
    },
    {
      name: '--output-format',
      short: '-f',
      description: 'Output format: json, csv, table, summary',
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
      name: '--sensitivity',
      description: 'Detection sensitivity: low, medium, high',
      type: 'string',
      default: 'medium',
    },
  ],
} as const;

// =============================================================================
// HANDLER PLACEHOLDER
// =============================================================================

// Note: In a real implementation, this would import from './handler'
// For now, we define a placeholder interface for the handler
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

// Placeholder handler - would be implemented in handler.ts
async function handler(request: EdgeFunctionRequest): Promise<EdgeFunctionResponse> {
  // This is a placeholder - actual implementation would be in handler.ts
  const input = request.body as HallucinationDetectorInput;
  const executionId = crypto.randomUUID();
  const startedAt = new Date().toISOString();

  // Placeholder detection logic
  const claims = input.claims || (input.claim ? [{ claim_id: 'claim-1', text: input.claim }] : []);

  const results: HallucinationClaimResult[] = claims.map((c, idx) => ({
    claim_id: c.claim_id || `claim-${idx + 1}`,
    claim_text: c.text,
    is_hallucination: false, // Placeholder
    hallucination_type: 'none' as const,
    confidence: 0.85,
    supporting_evidence: [],
    contradicting_evidence: [],
    explanation: 'Placeholder detection - actual implementation pending',
    method_scores: {
      semantic_similarity: 0.85,
      entailment_analysis: 0.80,
    },
  }));

  const output: HallucinationDetectorOutput = {
    execution_id: executionId,
    total_claims: claims.length,
    hallucinated_claims: results.filter(r => r.is_hallucination).length,
    verified_claims: results.filter(r => !r.is_hallucination).length,
    overall_hallucination_rate: results.filter(r => r.is_hallucination).length / Math.max(1, claims.length),
    results,
    detection_config: input.detection_config || {
      sensitivity: 'medium',
      confidence_threshold: 0.7,
      methods: ['semantic_similarity', 'entailment_analysis'],
      detect_types: ['fabrication', 'exaggeration', 'misattribution', 'contradiction', 'unsupported'],
      max_claim_length: 2000,
      max_reference_length: 50000,
      chunk_overlap: 200,
    },
    started_at: startedAt,
    completed_at: new Date().toISOString(),
    total_duration_ms: Date.now() - new Date(startedAt).getTime(),
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
  const argsValidation = HallucinationDetectorCLIArgsSchema.safeParse(parsedArgs);
  if (!argsValidation.success) {
    console.error('Invalid arguments:', argsValidation.error.message);
    return 1;
  }

  const cliArgs = argsValidation.data;

  // Load input
  let input: HallucinationDetectorInput;
  try {
    input = await loadInput(cliArgs);
  } catch (err) {
    console.error('Failed to load input:', err instanceof Error ? err.message : err);
    return 1;
  }

  // Validate input
  const inputValidation = HallucinationDetectorInputSchema.safeParse(input);
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
      const claimCount = input.claims?.length || (input.claim ? 1 : 0);
      const refLength = typeof input.reference_context === 'string'
        ? input.reference_context.length
        : input.reference_context.reduce((sum, r) => sum + r.content.length, 0);
      console.log('Input validation passed');
      console.log(`  Claims: ${claimCount}`);
      console.log(`  Reference context: ${refLength} characters`);
    }
    return 0;
  }

  // Execute handler
  if (cliArgs.verbose && !cliArgs.quiet) {
    console.log(`Executing ${HALLUCINATION_DETECTOR_AGENT.agent_id} v${HALLUCINATION_DETECTOR_AGENT.agent_version}`);
  }

  const response = await handler({
    body: inputValidation.data,
    headers: {},
    method: 'POST',
    path: '/hallucination-detector',
  });

  // Parse response
  const result = JSON.parse(response.body);

  // Handle error
  if (!result.success) {
    console.error('Execution failed:', result.error?.message || 'Unknown error');
    return 1;
  }

  const data = result.data as HallucinationDetectorOutput;

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
    console.log(`Hallucination rate: ${(data.overall_hallucination_rate * 100).toFixed(1)}%`);
    console.log(`Duration: ${data.total_duration_ms}ms`);
  }

  return 0;
}

// =============================================================================
// HELPERS
// =============================================================================

function parseArgs(args: string[]): Partial<HallucinationDetectorCLIArgs> {
  const result: Partial<HallucinationDetectorCLIArgs> = {};
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
      case '--reference-file':
      case '-r':
        result.reference_file = args[++i];
        break;
      case '--reference-text':
      case '-t':
        // Note: Using reference_url field to store inline text for now
        // In actual implementation, add reference_text to schema
        (result as any).reference_text = args[++i];
        break;
      case '--output-format':
      case '-f':
        result.output_format = args[++i] as 'json' | 'csv' | 'table' | 'summary';
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
      case '--sensitivity':
        result.sensitivity = args[++i] as 'low' | 'medium' | 'high';
        break;
    }

    i++;
  }

  return result;
}

async function loadInput(args: HallucinationDetectorCLIArgs & { reference_text?: string }): Promise<HallucinationDetectorInput> {
  let baseInput: Partial<HallucinationDetectorInput> = {};

  // Load claims from various sources
  if (args.input_file) {
    const content = await fs.readFile(args.input_file, 'utf-8');
    baseInput = JSON.parse(content);
  } else if (args.input_json) {
    baseInput = JSON.parse(args.input_json);
  } else if (args.input_stdin) {
    const data = await readStdin();
    baseInput = JSON.parse(data);
  }

  // Load reference context if provided separately
  if (args.reference_file) {
    const content = await fs.readFile(args.reference_file, 'utf-8');
    // Try to parse as JSON, otherwise use as plain text
    try {
      const parsed = JSON.parse(content);
      baseInput.reference_context = parsed;
    } catch {
      baseInput.reference_context = content;
    }
  } else if (args.reference_text) {
    baseInput.reference_context = args.reference_text;
  }

  // Apply sensitivity setting
  if (args.sensitivity) {
    baseInput.detection_config = {
      ...baseInput.detection_config,
      sensitivity: args.sensitivity,
    };
  }

  // Apply confidence threshold if provided
  if (args.confidence_threshold !== undefined) {
    baseInput.detection_config = {
      ...baseInput.detection_config,
      confidence_threshold: args.confidence_threshold,
    };
  }

  if (!baseInput.claim && (!baseInput.claims || baseInput.claims.length === 0)) {
    throw new Error('No claims provided. Use --input-file, --input-json, or --input-stdin');
  }

  if (!baseInput.reference_context) {
    throw new Error('No reference context provided. Use --reference-file or --reference-text');
  }

  return baseInput as HallucinationDetectorInput;
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

function formatOutput(data: HallucinationDetectorOutput, format: string): string {
  switch (format) {
    case 'json':
      return JSON.stringify(data, null, 2);

    case 'csv':
      return formatCSV(data);

    case 'table':
      return formatTable(data);

    case 'summary':
      return formatSummary(data);

    default:
      return JSON.stringify(data, null, 2);
  }
}

function formatCSV(data: HallucinationDetectorOutput): string {
  const results = data.results ?? [];
  if (results.length === 0) return '';

  const headers = [
    'claim_id',
    'is_hallucination',
    'hallucination_type',
    'confidence',
    'claim_text',
  ];

  const rows = results.map(r => [
    r.claim_id,
    r.is_hallucination,
    r.hallucination_type,
    r.confidence.toFixed(3),
    `"${r.claim_text.replace(/"/g, '""').slice(0, 100)}..."`,
  ]);

  return [headers.join(','), ...rows.map(r => r.join(','))].join('\n');
}

function formatTable(data: HallucinationDetectorOutput): string {
  const results = data.results ?? [];
  if (results.length === 0) return 'No claims analyzed';

  const header = `| Claim ID | Hallucination | Type | Confidence | Claim (truncated) |`;
  const separator = `|----------|---------------|------|------------|-------------------|`;

  const rows = results.map(r => {
    const statusIcon = r.is_hallucination ? 'YES' : 'NO';
    const claimPreview = r.claim_text.slice(0, 30) + (r.claim_text.length > 30 ? '...' : '');
    return `| ${r.claim_id.padEnd(8)} | ${statusIcon.padEnd(13)} | ${r.hallucination_type.padEnd(4)} | ${r.confidence.toFixed(2).padStart(10)} | ${claimPreview.padEnd(17)} |`;
  });

  return [header, separator, ...rows].join('\n');
}

function formatSummary(data: HallucinationDetectorOutput): string {
  const lines: string[] = [];

  lines.push('='.repeat(60));
  lines.push('HALLUCINATION DETECTION SUMMARY');
  lines.push('='.repeat(60));
  lines.push('');
  lines.push(`Execution ID: ${data.execution_id}`);
  lines.push(`Started: ${data.started_at}`);
  lines.push(`Completed: ${data.completed_at}`);
  lines.push(`Duration: ${data.total_duration_ms}ms`);
  lines.push('');
  lines.push('-'.repeat(60));
  lines.push('RESULTS');
  lines.push('-'.repeat(60));
  lines.push(`Total claims: ${data.total_claims}`);
  lines.push(`Verified claims: ${data.verified_claims}`);
  lines.push(`Hallucinated claims: ${data.hallucinated_claims}`);
  lines.push(`Hallucination rate: ${(data.overall_hallucination_rate * 100).toFixed(1)}%`);

  if (data.summary) {
    lines.push('');
    lines.push('By Type:');
    lines.push(`  Fabrication: ${data.summary.by_type.fabrication}`);
    lines.push(`  Exaggeration: ${data.summary.by_type.exaggeration}`);
    lines.push(`  Misattribution: ${data.summary.by_type.misattribution}`);
    lines.push(`  Contradiction: ${data.summary.by_type.contradiction}`);
    lines.push(`  Unsupported: ${data.summary.by_type.unsupported}`);
    lines.push('');
    lines.push('By Confidence:');
    lines.push(`  High (>=0.8): ${data.summary.by_confidence.high}`);
    lines.push(`  Medium (0.5-0.8): ${data.summary.by_confidence.medium}`);
    lines.push(`  Low (<0.5): ${data.summary.by_confidence.low}`);
  }

  if (data.results.some(r => r.is_hallucination)) {
    lines.push('');
    lines.push('-'.repeat(60));
    lines.push('HALLUCINATION DETAILS');
    lines.push('-'.repeat(60));

    for (const result of data.results.filter(r => r.is_hallucination)) {
      lines.push('');
      lines.push(`[${result.hallucination_type.toUpperCase()}] ${result.claim_id}`);
      lines.push(`Claim: "${result.claim_text.slice(0, 100)}${result.claim_text.length > 100 ? '...' : ''}"`);
      lines.push(`Confidence: ${(result.confidence * 100).toFixed(1)}%`);
      lines.push(`Explanation: ${result.explanation}`);
    }
  }

  lines.push('');
  lines.push('='.repeat(60));

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
      title: 'Hallucination Detector Agent CLI',
      version: HALLUCINATION_DETECTOR_AGENT.agent_version,
      description: 'CLI interface for the Hallucination Detector Agent',
    },
    paths: {
      '/cli/hallucination-detector': {
        post: {
          summary: 'Execute hallucination detection via CLI',
          operationId: 'cliDetectHallucinations',
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
    components: {
      schemas: {
        CLIArgs: {
          type: 'object',
          properties: {
            'input-file': {
              type: 'string',
              description: 'Path to input JSON file containing claims',
            },
            'input-json': {
              type: 'string',
              description: 'Input as JSON string',
            },
            'reference-file': {
              type: 'string',
              description: 'Path to reference context file',
            },
            'reference-text': {
              type: 'string',
              description: 'Reference context as inline text',
            },
            'output-format': {
              type: 'string',
              enum: ['json', 'csv', 'table', 'summary'],
              default: 'json',
            },
            'output-file': {
              type: 'string',
              description: 'Write output to file',
            },
            verbose: {
              type: 'boolean',
              default: false,
            },
            quiet: {
              type: 'boolean',
              default: false,
            },
            'dry-run': {
              type: 'boolean',
              default: false,
            },
            sensitivity: {
              type: 'string',
              enum: ['low', 'medium', 'high'],
              default: 'medium',
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

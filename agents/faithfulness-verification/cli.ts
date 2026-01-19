/**
 * Faithfulness Verification Agent - CLI Wrapper
 *
 * Command-line interface for invoking the Faithfulness Verification Agent.
 * This provides a CLI-invokable endpoint as required by the Agentics Dev platform.
 */

import * as fs from 'fs';
import * as path from 'path';
import { handler, EdgeFunctionRequest } from './handler';
import {
  FaithfulnessVerificationInputSchema,
  FaithfulnessVerificationCLIArgsSchema,
  FaithfulnessVerificationInput,
  FaithfulnessVerificationOutput,
  FAITHFULNESS_VERIFICATION_AGENT,
} from '../contracts';

// =============================================================================
// CLI ENTRY POINT
// =============================================================================

interface CLIResult {
  success: boolean;
  decision_id?: string;
  data?: FaithfulnessVerificationOutput;
  error?: {
    code: string;
    message: string;
  };
}

/**
 * Parse command line arguments
 */
function parseArgs(args: string[]): Record<string, string | boolean> {
  const parsed: Record<string, string | boolean> = {};
  let i = 0;

  while (i < args.length) {
    const arg = args[i];

    if (arg.startsWith('--')) {
      const key = arg.slice(2).replace(/-/g, '_');
      const nextArg = args[i + 1];

      if (nextArg && !nextArg.startsWith('--')) {
        parsed[key] = nextArg;
        i += 2;
      } else {
        parsed[key] = true;
        i += 1;
      }
    } else {
      i += 1;
    }
  }

  return parsed;
}

/**
 * Read input from file, JSON string, or stdin
 */
async function readInput(cliArgs: Record<string, string | boolean>): Promise<FaithfulnessVerificationInput> {
  let inputData: unknown;

  if (cliArgs.input_file && typeof cliArgs.input_file === 'string') {
    // Read from file
    const filePath = path.resolve(cliArgs.input_file);
    const content = fs.readFileSync(filePath, 'utf-8');
    inputData = JSON.parse(content);
  } else if (cliArgs.input_json && typeof cliArgs.input_json === 'string') {
    // Parse inline JSON
    inputData = JSON.parse(cliArgs.input_json);
  } else if (cliArgs.input_stdin) {
    // Read from stdin
    const chunks: Buffer[] = [];
    for await (const chunk of process.stdin) {
      chunks.push(chunk);
    }
    const content = Buffer.concat(chunks).toString('utf-8');
    inputData = JSON.parse(content);
  } else if (cliArgs.sources_file && cliArgs.output_text) {
    // Build input from separate source file and output text
    const sourcesPath = path.resolve(cliArgs.sources_file as string);
    const sourcesContent = fs.readFileSync(sourcesPath, 'utf-8');
    const sources = JSON.parse(sourcesContent);

    inputData = {
      sources: Array.isArray(sources) ? sources : [sources],
      output: {
        output_id: 'cli-output-1',
        content: cliArgs.output_text as string,
      },
    };
  } else {
    throw new Error('Input required: use --input-file, --input-json, --input-stdin, or --sources-file with --output-text');
  }

  // Apply CLI config overrides
  const config: Record<string, unknown> = {};
  if (cliArgs.threshold) {
    config.faithfulness_threshold = parseFloat(cliArgs.threshold as string);
  }
  if (cliArgs.granularity) {
    config.granularity = cliArgs.granularity;
  }
  if (cliArgs.method) {
    config.method = cliArgs.method;
  }

  if (Object.keys(config).length > 0) {
    (inputData as Record<string, unknown>).config = {
      ...(inputData as Record<string, unknown>).config as Record<string, unknown>,
      ...config,
    };
  }

  // Validate input
  const validation = FaithfulnessVerificationInputSchema.safeParse(inputData);
  if (!validation.success) {
    throw new Error(`Invalid input: ${validation.error.issues.map(i => `${i.path.join('.')}: ${i.message}`).join(', ')}`);
  }

  return validation.data;
}

/**
 * Format output based on format option
 */
function formatOutput(
  result: CLIResult,
  format: string,
  verbose: boolean
): string {
  switch (format) {
    case 'json':
      return JSON.stringify(result, null, 2);

    case 'table':
      return formatAsTable(result, verbose);

    case 'summary':
      return formatAsSummary(result);

    default:
      return JSON.stringify(result, null, 2);
  }
}

function formatAsTable(result: CLIResult, verbose: boolean): string {
  if (!result.success || !result.data) {
    return `Error: ${result.error?.message || 'Unknown error'}`;
  }

  const data = result.data;
  const lines: string[] = [];

  lines.push('='.repeat(80));
  lines.push(`FAITHFULNESS VERIFICATION REPORT`);
  lines.push(`Decision ID: ${result.decision_id}`);
  lines.push('='.repeat(80));
  lines.push('');

  // Overall result
  lines.push(`Is Faithful: ${data.is_faithful ? 'YES' : 'NO'}`);
  lines.push(`Overall Score: ${(data.faithfulness_scores.overall * 100).toFixed(1)}%`);
  lines.push(`Duration: ${data.duration_ms}ms`);
  lines.push('');

  // Score breakdown
  lines.push('-'.repeat(40));
  lines.push('SCORE BREAKDOWN');
  lines.push('-'.repeat(40));
  lines.push(`Claim Support Rate: ${(data.faithfulness_scores.claim_support_rate * 100).toFixed(1)}%`);
  lines.push(`Hallucination Rate: ${(data.faithfulness_scores.hallucination_rate * 100).toFixed(1)}%`);
  lines.push(`Contradiction Rate: ${(data.faithfulness_scores.contradiction_rate * 100).toFixed(1)}%`);
  lines.push(`Source Coverage:    ${(data.faithfulness_scores.coverage_score * 100).toFixed(1)}%`);
  lines.push('');

  // Summary statistics
  lines.push('-'.repeat(40));
  lines.push('CLAIM STATISTICS');
  lines.push('-'.repeat(40));
  lines.push(`Total Claims:         ${data.summary.total_claims}`);
  lines.push(`Supported:            ${data.summary.supported_claims}`);
  lines.push(`Partially Supported:  ${data.summary.partially_supported_claims}`);
  lines.push(`Unsupported:          ${data.summary.unsupported_claims}`);
  lines.push(`Contradicted:         ${data.summary.contradicted_claims}`);
  lines.push(`Unverifiable:         ${data.summary.unverifiable_claims}`);
  lines.push('');

  // Issues
  if (data.summary.total_hallucinations > 0 || data.summary.total_contradictions > 0) {
    lines.push('-'.repeat(40));
    lines.push('ISSUES DETECTED');
    lines.push('-'.repeat(40));
    lines.push(`Hallucinations: ${data.summary.total_hallucinations}`);
    lines.push(`Contradictions: ${data.summary.total_contradictions}`);
    lines.push('');
  }

  // Verbose: show all claims
  if (verbose && data.claims) {
    lines.push('-'.repeat(40));
    lines.push('CLAIM DETAILS');
    lines.push('-'.repeat(40));
    for (const claim of data.claims) {
      const verdict = claim.verdict.toUpperCase().padEnd(20);
      const conf = `${(claim.confidence * 100).toFixed(0)}%`.padStart(4);
      lines.push(`[${verdict}] (${conf}) ${claim.claim_text.substring(0, 60)}...`);
    }
    lines.push('');
  }

  // Verbose: show hallucinations
  if (verbose && data.hallucinations && data.hallucinations.length > 0) {
    lines.push('-'.repeat(40));
    lines.push('HALLUCINATIONS');
    lines.push('-'.repeat(40));
    for (const hal of data.hallucinations) {
      lines.push(`[${hal.severity.toUpperCase()}] ${hal.hallucination_type}: ${hal.text.substring(0, 60)}...`);
    }
    lines.push('');
  }

  // Verbose: show contradictions
  if (verbose && data.contradictions && data.contradictions.length > 0) {
    lines.push('-'.repeat(40));
    lines.push('CONTRADICTIONS');
    lines.push('-'.repeat(40));
    for (const con of data.contradictions) {
      lines.push(`[${con.severity.toUpperCase()}] ${con.contradiction_type}:`);
      lines.push(`  Output: ${con.output_text.substring(0, 50)}...`);
      lines.push(`  Source: ${con.source_text.substring(0, 50)}...`);
    }
    lines.push('');
  }

  lines.push('='.repeat(80));

  return lines.join('\n');
}

function formatAsSummary(result: CLIResult): string {
  if (!result.success || !result.data) {
    return `Error: ${result.error?.message || 'Unknown error'}`;
  }

  const data = result.data;
  const status = data.is_faithful ? 'FAITHFUL' : 'NOT FAITHFUL';
  const score = (data.faithfulness_scores.overall * 100).toFixed(1);

  return `[${status}] Score: ${score}% | Claims: ${data.summary.total_claims} (${data.summary.supported_claims} supported) | Hallucinations: ${data.summary.total_hallucinations} | Contradictions: ${data.summary.total_contradictions}`;
}

/**
 * Main CLI execution
 */
async function main(): Promise<void> {
  const args = process.argv.slice(2);

  // Show help
  if (args.includes('--help') || args.includes('-h')) {
    console.log(`
Faithfulness Verification Agent CLI

Usage:
  faithfulness-verification [options]

Input Options (one required):
  --input-file <path>     Read input from JSON file
  --input-json <json>     Read input from inline JSON string
  --input-stdin           Read input from stdin
  --sources-file <path>   Source documents file (use with --output-text)
  --output-text <text>    Output text to verify (use with --sources-file)

Output Options:
  --output-format <fmt>   Output format: json, table, summary (default: json)
  --output-file <path>    Write output to file

Configuration Options:
  --threshold <0-1>       Faithfulness threshold (default: 0.7)
  --granularity <level>   Analysis granularity: document, paragraph, sentence, claim
  --method <method>       Verification method: nli, semantic, entailment, hybrid

Other Options:
  --verbose               Show detailed output
  --quiet                 Suppress non-essential output
  --dry-run               Validate input without execution
  --help, -h              Show this help message

Examples:
  # Verify from input file
  faithfulness-verification --input-file input.json --output-format table --verbose

  # Verify with inline sources
  faithfulness-verification --sources-file sources.json --output-text "The model said..."

  # Pipeline from stdin
  cat input.json | faithfulness-verification --input-stdin --output-format summary

Agent: ${FAITHFULNESS_VERIFICATION_AGENT.agent_id} v${FAITHFULNESS_VERIFICATION_AGENT.agent_version}
Decision Type: ${FAITHFULNESS_VERIFICATION_AGENT.decision_type}
`);
    process.exit(0);
  }

  try {
    const cliArgs = parseArgs(args);

    // Validate CLI args
    const cliValidation = FaithfulnessVerificationCLIArgsSchema.safeParse(cliArgs);
    if (!cliValidation.success) {
      console.error(`Invalid arguments: ${cliValidation.error.issues.map(i => i.message).join(', ')}`);
      process.exit(1);
    }

    const validatedArgs = cliValidation.data;

    // Read and validate input
    const input = await readInput(cliArgs);

    // Dry run check
    if (validatedArgs.dry_run) {
      console.log('Dry run: Input validation successful');
      console.log(JSON.stringify(input, null, 2));
      process.exit(0);
    }

    // Create request
    const request: EdgeFunctionRequest = {
      body: input,
      headers: {
        'Content-Type': 'application/json',
      },
      method: 'POST',
      path: '/api/v1/agents/faithfulness-verification',
    };

    // Execute handler
    if (!validatedArgs.quiet) {
      console.error(`Executing ${FAITHFULNESS_VERIFICATION_AGENT.agent_id}...`);
    }

    const response = await handler(request);

    // Parse response
    const result: CLIResult = JSON.parse(response.body);

    // Format output
    const output = formatOutput(
      result,
      validatedArgs.output_format,
      validatedArgs.verbose
    );

    // Write output
    if (validatedArgs.output_file) {
      fs.writeFileSync(validatedArgs.output_file, output);
      if (!validatedArgs.quiet) {
        console.error(`Output written to: ${validatedArgs.output_file}`);
      }
    } else {
      console.log(output);
    }

    // Exit with appropriate code
    process.exit(result.success ? 0 : 1);

  } catch (err) {
    const error = err instanceof Error ? err : new Error(String(err));
    console.error(`Error: ${error.message}`);
    process.exit(1);
  }
}

// Run if executed directly
if (require.main === module) {
  main().catch(err => {
    console.error('Fatal error:', err);
    process.exit(1);
  });
}

// Export for testing
export { main, parseArgs, readInput, formatOutput };

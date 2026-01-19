/**
 * Regression Detection Agent - CLI Integration
 *
 * CLI command spec for invoking the Regression Detection Agent.
 * This integrates with agentics-cli.
 */

import * as fs from 'fs/promises';
import * as path from 'path';
import { handler, REGRESSION_DETECTION_AGENT } from './handler';
import {
  RegressionDetectionInputSchema,
  RegressionDetectionCLIArgsSchema,
  BenchmarkRunnerOutputSchema,
  type RegressionDetectionCLIArgs,
  type RegressionDetectionInput,
  type RegressionDetectionOutput,
  type BenchmarkRunnerOutput,
} from '../contracts';

// =============================================================================
// CLI COMMAND SPEC
// =============================================================================

export const CLI_COMMAND_SPEC = {
  name: 'regression-detection',
  description: 'Detect statistically significant regressions between benchmark runs',
  aliases: ['regress', 'rd'],
  usage: 'agentics regression-detection [options]',
  examples: [
    'agentics regression-detection --baseline-file baseline.json --candidate-file candidate.json',
    'agentics regression-detection --baseline-file baseline.json --candidate-file candidate.json --fail-on-regression',
    'agentics regression-detection --baseline-file baseline.json --candidate-file candidate.json --fail-severity critical',
    'cat input.json | agentics regression-detection --input-stdin',
    'agentics regression-detection --baseline-file baseline.json --candidate-file candidate.json --output-format table',
  ],
  options: [
    {
      name: '--baseline-file',
      short: '-b',
      description: 'Path to baseline benchmark results JSON file',
      type: 'string',
      required: false,
    },
    {
      name: '--candidate-file',
      short: '-c',
      description: 'Path to candidate benchmark results JSON file',
      type: 'string',
      required: false,
    },
    {
      name: '--baseline-json',
      description: 'Baseline results as JSON string',
      type: 'string',
      required: false,
    },
    {
      name: '--candidate-json',
      description: 'Candidate results as JSON string',
      type: 'string',
      required: false,
    },
    {
      name: '--input-stdin',
      short: '-s',
      description: 'Read input from stdin (expects { baseline_runs, candidate_runs })',
      type: 'boolean',
      required: false,
    },
    {
      name: '--thresholds-file',
      short: '-t',
      description: 'Path to custom thresholds configuration file',
      type: 'string',
      required: false,
    },
    {
      name: '--output-format',
      short: '-f',
      description: 'Output format: json, table, summary',
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
      description: 'Quiet mode (only output if regressions detected)',
      type: 'boolean',
      default: false,
    },
    {
      name: '--dry-run',
      short: '-d',
      description: 'Validate inputs without executing',
      type: 'boolean',
      default: false,
    },
    {
      name: '--fail-on-regression',
      description: 'Exit with code 1 if regressions detected (for CI/CD)',
      type: 'boolean',
      default: false,
    },
    {
      name: '--fail-severity',
      description: 'Minimum severity to fail on (critical, major, minor)',
      type: 'string',
      default: 'major',
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
  const argsValidation = RegressionDetectionCLIArgsSchema.safeParse(parsedArgs);
  if (!argsValidation.success) {
    console.error('Invalid arguments:', argsValidation.error.message);
    return 1;
  }

  const cliArgs = argsValidation.data;

  // Load input
  let input: RegressionDetectionInput;
  try {
    input = await loadInput(cliArgs);
  } catch (err) {
    console.error('Failed to load input:', err instanceof Error ? err.message : err);
    return 1;
  }

  // Validate input
  const inputValidation = RegressionDetectionInputSchema.safeParse(input);
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
      console.log(`  Baseline runs: ${input.baseline_runs.length}`);
      console.log(`  Candidate runs: ${input.candidate_runs.length}`);
    }
    return 0;
  }

  // Execute handler
  if (cliArgs.verbose && !cliArgs.quiet) {
    console.log(`Executing ${REGRESSION_DETECTION_AGENT.agent_id} v${REGRESSION_DETECTION_AGENT.agent_version}`);
  }

  const response = await handler({
    body: inputValidation.data,
    headers: {},
    method: 'POST',
    path: '/regression-detection',
  });

  // Parse response
  const result = JSON.parse(response.body);

  // Handle error
  if (!result.success) {
    console.error('Execution failed:', result.error.message);
    return 1;
  }

  const data = result.data as RegressionDetectionOutput;

  // Format output
  const output = formatOutput(data, cliArgs.output_format);

  // Write output (unless quiet mode with no regressions)
  const shouldOutput = !cliArgs.quiet || data.summary.any_regressions_detected;

  if (cliArgs.output_file) {
    await fs.writeFile(cliArgs.output_file, output, 'utf-8');
    if (!cliArgs.quiet) {
      console.log(`Output written to: ${cliArgs.output_file}`);
    }
  } else if (shouldOutput) {
    console.log(output);
  }

  // Print summary (unless quiet)
  if (!cliArgs.quiet) {
    console.log(`\nDecision ID: ${result.decision_id}`);
    console.log(`Analysis Duration: ${data.analysis_duration_ms}ms`);
    console.log(`\n${data.summary.summary_text}`);
  }

  // Check if we should fail on regression
  if (cliArgs.fail_on_regression && data.summary.any_regressions_detected) {
    const severityOrder = ['minor', 'major', 'critical'] as const;
    const failSeverityIndex = severityOrder.indexOf(cliArgs.fail_severity);
    const worstSeverityIndex = severityOrder.indexOf(
      data.summary.worst_severity === 'none' ? 'minor' : data.summary.worst_severity
    );

    if (worstSeverityIndex >= failSeverityIndex) {
      console.error(`\n❌ Regression detected at severity: ${data.summary.worst_severity}`);
      return 1;
    }
  }

  return 0;
}

// =============================================================================
// HELPERS
// =============================================================================

function parseArgs(args: string[]): Partial<RegressionDetectionCLIArgs> {
  const result: Partial<RegressionDetectionCLIArgs> = {};
  let i = 0;

  while (i < args.length) {
    const arg = args[i];

    switch (arg) {
      case '--baseline-file':
      case '-b':
        result.baseline_file = args[++i];
        break;
      case '--candidate-file':
      case '-c':
        result.candidate_file = args[++i];
        break;
      case '--baseline-json':
        result.baseline_json = args[++i];
        break;
      case '--candidate-json':
        result.candidate_json = args[++i];
        break;
      case '--input-stdin':
      case '-s':
        result.input_stdin = true;
        break;
      case '--thresholds-file':
      case '-t':
        result.thresholds_file = args[++i];
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
      case '--fail-on-regression':
        result.fail_on_regression = true;
        break;
      case '--fail-severity':
        result.fail_severity = args[++i] as 'critical' | 'major' | 'minor';
        break;
    }

    i++;
  }

  return result;
}

async function loadInput(args: RegressionDetectionCLIArgs): Promise<RegressionDetectionInput> {
  let baseline_runs: BenchmarkRunnerOutput[] = [];
  let candidate_runs: BenchmarkRunnerOutput[] = [];
  let thresholds: unknown;

  // Load from stdin
  if (args.input_stdin) {
    const data = await readStdin();
    const parsed = JSON.parse(data);
    baseline_runs = parsed.baseline_runs ?? [];
    candidate_runs = parsed.candidate_runs ?? [];
    thresholds = parsed.thresholds;
  } else {
    // Load baseline
    if (args.baseline_file) {
      const content = await fs.readFile(args.baseline_file, 'utf-8');
      const parsed = JSON.parse(content);
      // Handle single run or array of runs
      baseline_runs = Array.isArray(parsed) ? parsed : [parsed];
    } else if (args.baseline_json) {
      const parsed = JSON.parse(args.baseline_json);
      baseline_runs = Array.isArray(parsed) ? parsed : [parsed];
    }

    // Load candidate
    if (args.candidate_file) {
      const content = await fs.readFile(args.candidate_file, 'utf-8');
      const parsed = JSON.parse(content);
      candidate_runs = Array.isArray(parsed) ? parsed : [parsed];
    } else if (args.candidate_json) {
      const parsed = JSON.parse(args.candidate_json);
      candidate_runs = Array.isArray(parsed) ? parsed : [parsed];
    }
  }

  // Load thresholds if specified
  if (args.thresholds_file) {
    const content = await fs.readFile(args.thresholds_file, 'utf-8');
    thresholds = JSON.parse(content);
  }

  if (baseline_runs.length === 0) {
    throw new Error('No baseline runs provided. Use --baseline-file or --baseline-json');
  }

  if (candidate_runs.length === 0) {
    throw new Error('No candidate runs provided. Use --candidate-file or --candidate-json');
  }

  return {
    baseline_runs,
    candidate_runs,
    thresholds: thresholds as RegressionDetectionInput['thresholds'],
  };
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

function formatOutput(data: RegressionDetectionOutput, format: string): string {
  switch (format) {
    case 'json':
      return JSON.stringify(data, null, 2);

    case 'table':
      return formatTable(data);

    case 'summary':
      return formatSummary(data);

    default:
      return JSON.stringify(data, null, 2);
  }
}

function formatTable(data: RegressionDetectionOutput): string {
  const results = data.model_results;
  if (results.length === 0) return 'No models analyzed';

  const header = `| Provider | Model | Severity | Latency | Throughput | Success Rate | Cost |`;
  const separator = `|----------|-------|----------|---------|------------|--------------|------|`;

  const rows = results.map(r => {
    const getChange = (metric: string) => {
      const m = r.metric_regressions.find(mr => mr.metric_name === metric);
      if (!m) return 'N/A';
      const sign = m.percentage_change > 0 ? '+' : '';
      const status = m.is_regression ? '⚠' : '✓';
      return `${status} ${sign}${(m.percentage_change * 100).toFixed(1)}%`;
    };

    const severityIcon = {
      critical: '🔴',
      major: '🟠',
      minor: '🟡',
      none: '🟢',
    }[r.overall_severity];

    return `| ${r.provider_name.padEnd(8)} | ${r.model_id.slice(0, 5).padEnd(5)} | ${severityIcon} ${r.overall_severity.padEnd(7)} | ${getChange('latency').padEnd(7)} | ${getChange('throughput').padEnd(10)} | ${getChange('success_rate').padEnd(12)} | ${getChange('cost').padEnd(4)} |`;
  });

  return [header, separator, ...rows].join('\n');
}

function formatSummary(data: RegressionDetectionOutput): string {
  const lines: string[] = [];

  lines.push('='.repeat(60));
  lines.push('REGRESSION DETECTION SUMMARY');
  lines.push('='.repeat(60));
  lines.push('');
  lines.push(`Detection ID: ${data.detection_id}`);
  lines.push(`Detected at: ${data.detected_at}`);
  lines.push(`Analysis duration: ${data.analysis_duration_ms}ms`);
  lines.push('');
  lines.push('-'.repeat(60));
  lines.push('OVERALL RESULTS');
  lines.push('-'.repeat(60));
  lines.push(`Models analyzed: ${data.summary.total_models_analyzed}`);
  lines.push(`Models with regressions: ${data.summary.models_with_regressions}`);
  lines.push(`  - Critical: ${data.summary.models_with_critical}`);
  lines.push(`  - Major: ${data.summary.models_with_major}`);
  lines.push(`  - Minor: ${data.summary.models_with_minor}`);
  lines.push(`Worst severity: ${data.summary.worst_severity.toUpperCase()}`);
  lines.push('');
  lines.push(`Baseline executions: ${data.summary.total_baseline_executions}`);
  lines.push(`Candidate executions: ${data.summary.total_candidate_executions}`);
  lines.push('');
  lines.push(data.summary.summary_text);

  if (data.model_results.some(r => r.has_regression)) {
    lines.push('');
    lines.push('-'.repeat(60));
    lines.push('REGRESSION DETAILS');
    lines.push('-'.repeat(60));

    for (const result of data.model_results.filter(r => r.has_regression)) {
      lines.push('');
      lines.push(`[${result.overall_severity.toUpperCase()}] ${result.provider_name}/${result.model_id}`);
      lines.push(result.summary);

      for (const metric of result.metric_regressions.filter(m => m.is_regression)) {
        const sign = metric.percentage_change > 0 ? '+' : '';
        lines.push(`  - ${metric.metric_name}: ${sign}${(metric.percentage_change * 100).toFixed(1)}% (p=${metric.statistical_test.p_value.toFixed(4)})`);
      }
    }
  }

  lines.push('');
  lines.push('='.repeat(60));

  return lines.join('\n');
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

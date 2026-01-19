/**
 * Stress Test Agent - CLI Integration
 *
 * CLI command spec for invoking the Stress Test Agent.
 * This integrates with agentics-cli.
 */

import * as fs from 'fs/promises';
import * as path from 'path';
import { handler, STRESS_TEST_AGENT } from './handler';
import {
  StressTestInputSchema,
  StressTestCLIArgsSchema,
  type StressTestCLIArgs,
  type StressTestInput,
  type StressTestScenario,
  type StressTestProviderConfig,
} from '../contracts';

// =============================================================================
// CLI COMMAND SPEC
// =============================================================================

export const CLI_COMMAND_SPEC = {
  name: 'stress-test',
  description: 'Evaluate model robustness under extreme input, load, or adversarial conditions',
  aliases: ['stress', 'st'],
  usage: 'agentics stress-test [options]',
  examples: [
    'agentics stress-test --input-file stress-config.json',
    'agentics stress-test --preset quick-load --output-format summary',
    'agentics stress-test --preset spike --max-requests 1000',
    'agentics stress-test --preset adversarial --output-file results.json',
    'cat config.json | agentics stress-test --input-stdin',
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
      description: 'Quick test preset: quick-load, spike, soak-5min, adversarial, full-suite',
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
      name: '--max-requests',
      description: 'Maximum total requests (safety limit)',
      type: 'number',
      required: false,
    },
    {
      name: '--max-cost',
      description: 'Maximum cost in USD (safety limit)',
      type: 'number',
      required: false,
    },
  ],
} as const;

// =============================================================================
// PRESETS
// =============================================================================

const PRESETS: Record<string, Omit<StressTestInput, 'providers'>> = {
  'quick-load': {
    scenarios: [
      {
        scenario_id: 'quick-load-ramp',
        scenario_name: 'Quick Load Ramp Test',
        description: 'Quickly ramp up concurrency to find limits',
        test_type: 'load_ramp',
        load_ramp_config: {
          initial_concurrency: 1,
          max_concurrency: 50,
          step_size: 10,
          step_duration_ms: 5000,
          requests_per_step: 5,
        },
        base_prompt: 'Respond with a single word: OK',
      },
    ],
    execution_config: {
      max_total_duration_ms: 120000,
      max_total_requests: 500,
      stop_on_critical_failure: true,
      collect_response_samples: false,
      sample_rate: 0.1,
    },
  },
  'spike': {
    scenarios: [
      {
        scenario_id: 'spike-test',
        scenario_name: 'Spike Test',
        description: 'Test response to sudden load spike',
        test_type: 'spike',
        spike_config: {
          baseline_concurrency: 1,
          spike_concurrency: 30,
          spike_duration_ms: 10000,
          recovery_observation_ms: 15000,
        },
        base_prompt: 'Count from 1 to 10.',
      },
    ],
    execution_config: {
      max_total_duration_ms: 60000,
      max_total_requests: 300,
      stop_on_critical_failure: false,
      collect_response_samples: false,
      sample_rate: 0.1,
    },
  },
  'soak-5min': {
    scenarios: [
      {
        scenario_id: 'soak-5min',
        scenario_name: '5-Minute Soak Test',
        description: 'Sustained load over 5 minutes',
        test_type: 'soak',
        soak_config: {
          concurrency: 5,
          duration_ms: 300000,
          request_interval_ms: 2000,
          metrics_sample_interval_ms: 10000,
        },
        base_prompt: 'What is 2 + 2?',
      },
    ],
    execution_config: {
      max_total_duration_ms: 330000,
      max_total_requests: 2000,
      stop_on_critical_failure: true,
      collect_response_samples: false,
      sample_rate: 0.05,
    },
  },
  'adversarial': {
    scenarios: [
      {
        scenario_id: 'adversarial-encoding',
        scenario_name: 'Encoding Tricks Test',
        test_type: 'adversarial',
        adversarial_config: {
          test_categories: ['encoding_tricks', 'repetition', 'boundary_chars'],
          severity_level: 'medium',
          samples_per_category: 3,
        },
      },
      {
        scenario_id: 'adversarial-structure',
        scenario_name: 'Nested Structure Test',
        test_type: 'adversarial',
        adversarial_config: {
          test_categories: ['nested_structures', 'format_confusion'],
          severity_level: 'medium',
          samples_per_category: 3,
        },
      },
    ],
    execution_config: {
      max_total_duration_ms: 120000,
      max_total_requests: 100,
      stop_on_critical_failure: false,
      collect_response_samples: true,
      sample_rate: 1.0,
    },
  },
  'full-suite': {
    scenarios: [
      {
        scenario_id: 'load-ramp',
        scenario_name: 'Load Ramp Test',
        test_type: 'load_ramp',
        load_ramp_config: {
          initial_concurrency: 1,
          max_concurrency: 20,
          step_size: 5,
          step_duration_ms: 5000,
          requests_per_step: 3,
        },
        base_prompt: 'Respond OK.',
      },
      {
        scenario_id: 'extreme-input',
        scenario_name: 'Extreme Input Test',
        test_type: 'extreme_input',
        extreme_input_config: {
          input_sizes: [1000, 5000, 10000],
          character_types: ['ascii', 'unicode'],
          include_edge_cases: true,
        },
      },
      {
        scenario_id: 'rate-limit',
        scenario_name: 'Rate Limit Probe',
        test_type: 'rate_limit_probe',
        rate_limit_probe_config: {
          initial_rps: 1,
          max_rps: 20,
          increment: 5,
          duration_per_level_ms: 3000,
          detect_throttling: true,
        },
      },
      {
        scenario_id: 'malformed',
        scenario_name: 'Malformed Request Test',
        test_type: 'malformed_request',
      },
    ],
    execution_config: {
      max_total_duration_ms: 300000,
      max_total_requests: 500,
      stop_on_critical_failure: false,
      collect_response_samples: true,
      sample_rate: 0.2,
    },
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
  const argsValidation = StressTestCLIArgsSchema.safeParse(parsedArgs);
  if (!argsValidation.success) {
    console.error('Invalid arguments:', argsValidation.error.message);
    return 1;
  }

  const cliArgs = argsValidation.data;

  // Load input
  let input: StressTestInput;
  try {
    input = await loadInput(cliArgs);
  } catch (err) {
    console.error('Failed to load input:', err instanceof Error ? err.message : err);
    return 1;
  }

  // Apply CLI overrides
  if (cliArgs.max_requests) {
    input.execution_config = {
      ...input.execution_config,
      max_total_requests: cliArgs.max_requests,
    };
  }
  if (cliArgs.max_cost_usd) {
    input.execution_config = {
      ...input.execution_config,
      max_total_cost_usd: cliArgs.max_cost_usd,
    };
  }

  // Validate input
  const inputValidation = StressTestInputSchema.safeParse(input);
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
      console.log(`  Providers: ${inputValidation.data.providers.length}`);
      console.log(`  Scenarios: ${inputValidation.data.scenarios.length}`);
      console.log(`  Test types: ${Array.from(new Set(inputValidation.data.scenarios.map(s => s.test_type))).join(', ')}`);
    }
    return 0;
  }

  // Execute handler
  if (cliArgs.verbose && !cliArgs.quiet) {
    console.log(`Executing ${STRESS_TEST_AGENT.agent_id} v${STRESS_TEST_AGENT.agent_version}`);
    console.log(`  Providers: ${inputValidation.data.providers.map(p => `${p.provider_name}/${p.model_id}`).join(', ')}`);
    console.log(`  Scenarios: ${inputValidation.data.scenarios.length}`);
  }

  if (!cliArgs.quiet) {
    console.log('Starting stress test execution...\n');
  }

  const response = await handler({
    body: inputValidation.data,
    headers: {},
    method: 'POST',
    path: '/stress-test',
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
    console.log('\n' + '='.repeat(60));
    console.log('STRESS TEST SUMMARY');
    console.log('='.repeat(60));
    console.log(`Decision ID: ${result.decision_id}`);
    console.log(`Duration: ${(result.data.total_duration_ms / 1000).toFixed(1)}s`);
    console.log(`Total Requests: ${result.data.total_requests}`);
    console.log(`Success Rate: ${(result.data.overall_success_rate * 100).toFixed(1)}%`);

    if (result.data.provider_summaries?.length > 0) {
      console.log('\nProvider Robustness Scores:');
      for (const summary of result.data.provider_summaries) {
        console.log(`  ${summary.provider_name}/${summary.model_id}: ${(summary.robustness_score * 100).toFixed(0)}%`);
        if (summary.warnings?.length > 0) {
          for (const warning of summary.warnings) {
            console.log(`    ⚠ ${warning}`);
          }
        }
      }
    }

    if (result.data.constraints_applied?.length > 0) {
      console.log(`\nConstraints Applied: ${result.data.constraints_applied.join(', ')}`);
    }
  }

  return 0;
}

// =============================================================================
// HELPERS
// =============================================================================

function parseArgs(args: string[]): Partial<StressTestCLIArgs> {
  const result: Partial<StressTestCLIArgs> = {};
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
        result.preset = args[++i] as any;
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
      case '--max-requests':
        result.max_requests = parseInt(args[++i], 10);
        break;
      case '--max-cost':
        result.max_cost_usd = parseFloat(args[++i]);
        break;
    }

    i++;
  }

  return result;
}

async function loadInput(args: StressTestCLIArgs): Promise<StressTestInput> {
  // If using a preset, build the input from the preset
  if (args.preset) {
    const preset = PRESETS[args.preset];
    if (!preset) {
      throw new Error(`Unknown preset: ${args.preset}. Valid presets: ${Object.keys(PRESETS).join(', ')}`);
    }

    // Still need providers from environment or default
    const providers = getDefaultProviders();
    if (providers.length === 0) {
      throw new Error('No providers configured. Set OPENAI_API_KEY, ANTHROPIC_API_KEY, or provide --input-file');
    }

    return {
      providers,
      ...preset,
    };
  }

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

  throw new Error('No input source specified. Use --input-file, --input-json, --input-stdin, or --preset');
}

function getDefaultProviders(): StressTestProviderConfig[] {
  const providers: StressTestProviderConfig[] = [];

  if (process.env.OPENAI_API_KEY) {
    providers.push({
      provider_name: 'openai',
      model_id: 'gpt-4o-mini',
      api_key_ref: 'openai',
      timeout_ms: 60000,
      max_retries: 1,
    });
  }

  if (process.env.ANTHROPIC_API_KEY) {
    providers.push({
      provider_name: 'anthropic',
      model_id: 'claude-3-5-haiku-20241022',
      api_key_ref: 'anthropic',
      timeout_ms: 60000,
      max_retries: 1,
    });
  }

  return providers;
}

function formatOutput(data: any, format: string): string {
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

function formatCSV(data: any): string {
  const results = data.scenario_results ?? [];
  if (results.length === 0) return '';

  const headers = [
    'scenario_id',
    'test_type',
    'provider',
    'model',
    'total_requests',
    'success_rate',
    'latency_p50_ms',
    'latency_p95_ms',
    'breaking_points',
  ];

  const rows = results.map((r: any) => [
    r.scenario_id,
    r.test_type,
    r.provider_name,
    r.model_id,
    r.total_requests,
    (r.success_rate * 100).toFixed(1) + '%',
    r.latency_p50_ms?.toFixed(0) ?? '',
    r.latency_p95_ms?.toFixed(0) ?? '',
    r.breaking_points?.length ?? 0,
  ]);

  return [headers.join(','), ...rows.map((r: any[]) => r.join(','))].join('\n');
}

function formatTable(data: any): string {
  const summaries = data.provider_summaries ?? [];
  if (summaries.length === 0) return 'No results';

  const header = `| Provider      | Model         | Robustness | Max Conc. | Degradation | Warnings |`;
  const separator = `|---------------|---------------|------------|-----------|-------------|----------|`;

  const rows = summaries.map((s: any) =>
    `| ${(s.provider_name || '').padEnd(13)} | ${(s.model_id || '').slice(0, 13).padEnd(13)} | ${((s.robustness_score * 100).toFixed(0) + '%').padStart(10)} | ${String(s.max_sustainable_concurrency ?? 'N/A').padStart(9)} | ${(s.degradation_severity || 'none').padEnd(11)} | ${String(s.warnings?.length ?? 0).padStart(8)} |`
  );

  return [header, separator, ...rows].join('\n');
}

function formatSummary(data: any): string {
  const lines: string[] = [
    '╔══════════════════════════════════════════════════════════════╗',
    '║              STRESS TEST EXECUTION SUMMARY                   ║',
    '╠══════════════════════════════════════════════════════════════╣',
  ];

  lines.push(`║ Execution ID: ${data.execution_id?.slice(0, 36) ?? 'N/A'.padEnd(36)}      ║`);
  lines.push(`║ Duration: ${(data.total_duration_ms / 1000).toFixed(1)}s                                              ║`.slice(0, 65) + '║');
  lines.push(`║ Total Requests: ${data.total_requests ?? 0}                                        ║`.slice(0, 65) + '║');
  lines.push(`║ Success Rate: ${((data.overall_success_rate ?? 0) * 100).toFixed(1)}%                                       ║`.slice(0, 65) + '║');
  lines.push('╠══════════════════════════════════════════════════════════════╣');

  if (data.scenario_results?.length > 0) {
    lines.push('║ SCENARIO RESULTS:                                            ║');
    for (const scenario of data.scenario_results) {
      lines.push(`║   ${scenario.scenario_name?.slice(0, 45) ?? 'Unknown'}                                 ║`.slice(0, 65) + '║');
      lines.push(`║     Type: ${scenario.test_type ?? 'unknown'}, Success: ${((scenario.success_rate ?? 0) * 100).toFixed(1)}%                        ║`.slice(0, 65) + '║');
      if (scenario.breaking_points?.length > 0) {
        lines.push(`║     ⚠ Breaking point(s) detected                            ║`);
      }
    }
  }

  if (data.provider_summaries?.length > 0) {
    lines.push('╠══════════════════════════════════════════════════════════════╣');
    lines.push('║ PROVIDER ROBUSTNESS:                                         ║');
    for (const provider of data.provider_summaries) {
      const score = ((provider.robustness_score ?? 0) * 100).toFixed(0);
      const bar = '█'.repeat(Math.floor((provider.robustness_score ?? 0) * 20)).padEnd(20, '░');
      lines.push(`║   ${provider.provider_name ?? 'unknown'}: [${bar}] ${score}%                   ║`.slice(0, 65) + '║');
    }
  }

  if (data.constraints_applied?.length > 0) {
    lines.push('╠══════════════════════════════════════════════════════════════╣');
    lines.push('║ CONSTRAINTS APPLIED:                                         ║');
    for (const constraint of data.constraints_applied) {
      lines.push(`║   • ${constraint}                                             ║`.slice(0, 65) + '║');
    }
  }

  lines.push('╚══════════════════════════════════════════════════════════════╝');

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

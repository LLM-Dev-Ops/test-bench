# Benchmark Runner Agent

## Purpose Statement

Execute deterministic benchmark suites against one or more LLMs, producing reproducible performance, quality, latency, and cost metrics.

**This agent:**
- ✅ Executes benchmarks
- ❌ Does NOT compare models
- ❌ Does NOT score regressions
- ❌ Does NOT rank outputs

**decision_type:** `benchmark_execution`

---

## Contract Summary

### Input Schema

```typescript
interface BenchmarkRunnerInput {
  providers: BenchmarkProviderConfig[];  // 1+ providers to benchmark
  suite: BenchmarkSuite;                  // Test cases to execute
  execution_config?: ExecutionConfig;     // Concurrency, iterations, etc.
  caller_id?: string;                     // Optional caller identifier
  correlation_id?: string;                // Optional correlation UUID
}
```

### Output Schema

```typescript
interface BenchmarkRunnerOutput {
  execution_id: string;            // UUID for this execution
  suite_id: string;                // Suite identifier
  started_at: string;              // ISO timestamp
  completed_at: string;            // ISO timestamp
  total_duration_ms: number;       // Total execution time
  total_tests: number;             // Number of test cases
  total_executions: number;        // Total iterations executed
  successful_executions: number;   // Successful count
  failed_executions: number;       // Failed count
  results: TestExecutionResult[];  // Per-execution results
  aggregated_stats: AggregatedStats[]; // Per-provider stats
}
```

### DecisionEvent Mapping

| Field | Source |
|-------|--------|
| `agent_id` | `benchmark-runner` (constant) |
| `agent_version` | `1.0.0` (constant) |
| `decision_type` | `benchmark_execution` (constant) |
| `inputs_hash` | SHA-256 of input JSON |
| `outputs` | Full `BenchmarkRunnerOutput` |
| `confidence` | Calculated from success rate, latency consistency |
| `constraints_applied` | List of applied constraints |

---

## CLI Contract

### Command

```bash
agentics benchmark-runner [options]
```

### Options

| Option | Short | Description | Default |
|--------|-------|-------------|---------|
| `--input-file` | `-i` | Path to input JSON file | - |
| `--input-json` | `-j` | Input as JSON string | - |
| `--input-stdin` | `-s` | Read input from stdin | false |
| `--output-format` | `-f` | Output format: json, csv, table | json |
| `--output-file` | `-o` | Write output to file | stdout |
| `--verbose` | `-v` | Verbose output | false |
| `--quiet` | `-q` | Quiet mode | false |
| `--dry-run` | `-d` | Validate input only | false |

### Examples

```bash
# Execute from file
agentics benchmark-runner -i benchmark.json

# Execute with table output
agentics benchmark-runner -i benchmark.json -f table

# Pipe input from stdin
cat benchmark.json | agentics benchmark-runner -s

# Dry run (validation only)
agentics benchmark-runner -i benchmark.json --dry-run

# Save output to file
agentics benchmark-runner -i benchmark.json -o results.json
```

---

## Explicit Non-Responsibilities

This agent MUST NOT:

1. **compare_models** - No model comparison logic
2. **score_regressions** - No regression scoring
3. **rank_outputs** - No ranking/ordering
4. **enforce_policy** - No policy enforcement
5. **orchestrate_workflows** - No workflow orchestration
6. **call_other_agents** - No direct agent-to-agent calls
7. **store_api_keys** - Never persist actual API keys
8. **execute_arbitrary_code** - No code execution beyond LLM calls
9. **bypass_schemas** - Must validate all I/O

---

## Failure Modes

| Error Code | Description | Recoverable |
|------------|-------------|-------------|
| `VALIDATION_ERROR` | Input failed schema validation | Yes |
| `EXECUTION_ERROR` | Benchmark execution failed | Depends |
| `TIMEOUT_ERROR` | Request exceeded timeout | Yes |
| `PROVIDER_ERROR` | LLM provider returned error | Yes |
| `CONFIGURATION_ERROR` | Invalid configuration | Yes |
| `PERSISTENCE_ERROR` | Failed to persist decision | No |

---

## Allowed Consumers

The following Core bundles may consume this agent's output:

- `llm-orchestrator` - For workflow coordination
- `llm-observatory` - For telemetry/monitoring
- `llm-analytics` - For aggregation/analysis
- `llm-test-bench-ui` - For dashboard display

---

## Valid Constraints

Constraints that may be applied during execution:

- `max_duration_exceeded` - Stopped due to time limit
- `max_cost_exceeded` - Stopped due to cost limit
- `rate_limit_applied` - Rate limiting enforced
- `fail_fast_triggered` - Stopped on first failure
- `warm_up_skipped` - Warm-up phase skipped
- `concurrency_limited` - Concurrency reduced
- `provider_unavailable` - Provider not reachable

---

## Confidence Scoring

Confidence is calculated from:

| Factor | Weight | Description |
|--------|--------|-------------|
| `execution_success_rate` | 0.4 | % of successful executions |
| `latency_consistency` | 0.2 | Inverse of stddev/mean |
| `provider_reliability` | 0.2 | Historical provider score |
| `sample_size` | 0.2 | Log scale of iterations |

---

## Versioning Rules

- **Major**: Breaking changes to input/output schemas
- **Minor**: New optional fields, new providers, new metrics
- **Patch**: Bug fixes, performance improvements, documentation

---

## Smoke Test Commands

```bash
# 1. Help text
agentics benchmark-runner --help

# 2. Dry run validation
agentics benchmark-runner --dry-run -i examples/sample-input.json

# 3. Execute with table output
agentics benchmark-runner -i examples/sample-input.json -f table

# 4. Execute with CSV output
agentics benchmark-runner -i examples/sample-input.json -f csv -o results.csv

# 5. Health check endpoint
curl http://localhost:8080/api/v1/agents/benchmark-runner/health
```

---

## Deployment

- **Type**: Google Cloud Edge Function
- **Service**: llm-test-bench
- **Endpoint**: `/api/v1/agents/benchmark-runner`
- **Timeout**: 5 minutes
- **Memory**: 512 MB
- **Region**: us-central1 (primary)

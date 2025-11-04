# CLI Command Reference

Complete reference for all LLM Test Bench CLI commands.

## Table of Contents

- [Global Options](#global-options)
- [Commands](#commands)
  - [test](#test---run-single-test)
  - [bench](#bench---run-benchmarks)
  - [eval](#eval---evaluate-results)
  - [compare](#compare---compare-models)
  - [dashboard](#dashboard---generate-dashboards)
  - [analyze](#analyze---statistical-analysis)
  - [optimize](#optimize---cost-optimization)
  - [config](#config---configuration-management)
  - [completions](#completions---shell-completions)

---

## Global Options

Available for all commands:

```bash
--verbose, -v       Enable verbose output
--no-color          Disable colored output
--help, -h          Print help information
--version, -V       Print version information
```

---

## Commands

### `test` - Run Single Test

Run a single test against an LLM provider.

**Alias:** `t`

#### Usage

```bash
llm-test-bench test [OPTIONS] --provider <PROVIDER> --prompt <PROMPT>
```

#### Options

- `--provider <PROVIDER>` - Provider name (e.g., openai, anthropic)
- `--model <MODEL>` - Model to use (overrides provider default)
- `--prompt <PROMPT>` - Test prompt
- `--expected <EXPECTED>` - Expected output for validation
- `--temperature <TEMP>` - Temperature setting (0.0-2.0)
- `--max-tokens <TOKENS>` - Maximum tokens to generate
- `--timeout <SECONDS>` - Request timeout in seconds
- `--config <PATH>` - Path to custom configuration file

#### Examples

```bash
# Basic test
llm-test-bench test --provider openai --prompt "Explain quantum computing"

# Test with specific model
llm-test-bench test --provider openai --model gpt-4 --prompt "Hello"

# Test with validation
llm-test-bench test --provider anthropic --prompt "2+2" --expected "4"
```

---

### `bench` - Run Benchmarks

Run benchmark tests across multiple providers.

**Alias:** `b`

#### Usage

```bash
llm-test-bench bench [OPTIONS] --dataset <PATH> --providers <PROVIDERS>
```

#### Options

- `--dataset <PATH>` - Path to dataset file (JSON or YAML)
- `--providers <PROVIDERS>` - Comma-separated list of providers
- `--concurrency <N>` - Number of concurrent requests (default: 5)
- `--output <PATH>` - Output directory (default: ./bench-results)
- `--export <FORMAT>` - Export format: json, csv, both (default: both)
- `--continue-on-failure` - Continue on test failure (default: true)
- `--save-responses` - Save raw responses (default: true)
- `--delay <MS>` - Request delay in milliseconds
- `--config <PATH>` - Path to custom configuration file
- `--metrics <METRICS>` - Comma-separated evaluation metrics
- `--judge-model <MODEL>` - Judge model for evaluations
- `--judge-provider <PROVIDER>` - Judge provider
- `--dashboard` - Generate HTML dashboard after benchmark

#### Examples

```bash
# Basic benchmark
llm-test-bench bench --dataset tests.json --providers openai,anthropic

# Benchmark with evaluation
llm-test-bench bench \
  --dataset tests.json \
  --providers openai \
  --metrics faithfulness,relevance \
  --dashboard

# Benchmark with custom concurrency
llm-test-bench bench \
  --dataset tests.json \
  --providers openai \
  --concurrency 10 \
  --delay 100
```

---

### `eval` - Evaluate Results

Evaluate test results with metrics.

**Alias:** `e`

#### Usage

```bash
llm-test-bench eval [OPTIONS] --results <PATH>
```

#### Options

- `--results <PATH>` - Path to results file
- `--metrics <METRICS>` - Comma-separated evaluation metrics
- `--judge-model <MODEL>` - Judge model for evaluations
- `--output <PATH>` - Output file for evaluation results
- `--config <PATH>` - Path to custom configuration file

#### Examples

```bash
# Evaluate results
llm-test-bench eval --results bench-results/openai-results.json

# Evaluate with specific metrics
llm-test-bench eval \
  --results results.json \
  --metrics faithfulness,relevance,coherence
```

---

### `compare` - Compare Models

Compare multiple models on the same prompt or dataset.

**Alias:** `c`

#### Usage

```bash
llm-test-bench compare [OPTIONS] --models <MODELS>
```

#### Options

- `--prompt <PROMPT>` - Single prompt to test (conflicts with --dataset)
- `--dataset <PATH>` - Dataset file for batch comparison
- `--models <MODELS>` - Comma-separated models (format: provider:model)
- `--metrics <METRICS>` - Evaluation metrics (default: faithfulness,relevance)
- `--statistical-tests` - Run statistical significance tests
- `--output <FORMAT>` - Output format: table, json, dashboard (default: table)
- `--output-file <PATH>` - Save results to file
- `--dashboard` - Generate HTML dashboard
- `--config <PATH>` - Path to custom configuration file
- `--concurrency <N>` - Maximum concurrent comparisons (default: 5)

#### Examples

```bash
# Compare two models on a prompt
llm-test-bench compare \
  --prompt "Explain quantum computing" \
  --models openai:gpt-4,anthropic:claude-3-opus

# Compare with statistical tests
llm-test-bench compare \
  --prompt "Test prompt" \
  --models openai:gpt-4,openai:gpt-3.5-turbo,anthropic:claude-3-sonnet \
  --statistical-tests \
  --dashboard

# Batch comparison
llm-test-bench compare \
  --dataset tests.json \
  --models openai:gpt-4,anthropic:claude-3-opus \
  --metrics faithfulness,relevance,coherence \
  --output-file comparison.json
```

---

### `dashboard` - Generate Dashboards

Generate interactive HTML dashboards from results.

**Alias:** `d`

#### Usage

```bash
llm-test-bench dashboard [OPTIONS] --results <FILES> --output <PATH>
```

#### Options

- `--results <FILES>` - Comma-separated result files to visualize
- `--dashboard-type <TYPE>` - Type: benchmark, comparison, analysis, custom (default: benchmark)
- `--theme <THEME>` - Theme: light, dark, auto (default: auto)
- `--output <PATH>` - Output file path (default: dashboard.html)
- `--title <TITLE>` - Dashboard title
- `--include-raw-data` - Include raw data in dashboard
- `--config <PATH>` - Path to custom configuration file

#### Examples

```bash
# Generate benchmark dashboard
llm-test-bench dashboard \
  --results bench-results/*.json \
  --output benchmark.html

# Generate comparison dashboard
llm-test-bench dashboard \
  --results comparison-results.json \
  --dashboard-type comparison \
  --theme dark \
  --output comparison-dashboard.html

# Multiple result files
llm-test-bench dashboard \
  --results results1.json,results2.json,results3.json \
  --title "Multi-Provider Comparison" \
  --output multi-dashboard.html
```

---

### `analyze` - Statistical Analysis

Perform statistical analysis comparing baseline and new results.

**Alias:** `a`

#### Usage

```bash
llm-test-bench analyze [OPTIONS] --baseline <PATH> --comparison <PATH>
```

#### Options

- `--baseline <PATH>` - Baseline results file
- `--comparison <PATH>` - Comparison results file
- `--metric <METRIC>` - Metric to analyze (default: overall)
- `--confidence-level <LEVEL>` - Confidence level: 0.90, 0.95, 0.99 (default: 0.95)
- `--fail-on-regression` - Exit with error code 2 if regression detected
- `--effect-size-threshold <THRESHOLD>` - Effect size threshold (default: 0.2)
- `--output <FORMAT>` - Output format: detailed, summary, json (default: detailed)
- `--report-file <PATH>` - Save report to file
- `--config <PATH>` - Path to custom configuration file

#### Exit Codes

- `0` - Success, no regression
- `1` - Error during analysis
- `2` - Regression detected (with --fail-on-regression)

#### Examples

```bash
# Basic analysis
llm-test-bench analyze \
  --baseline baseline-results.json \
  --comparison new-results.json

# Analysis with regression check
llm-test-bench analyze \
  --baseline v1-results.json \
  --comparison v2-results.json \
  --metric faithfulness \
  --fail-on-regression

# CI/CD integration
llm-test-bench analyze \
  --baseline prod-baseline.json \
  --comparison pr-results.json \
  --confidence-level 0.99 \
  --fail-on-regression \
  --output summary
```

---

### `optimize` - Cost Optimization

Recommend cost-optimized model alternatives.

**Alias:** `o`

#### Usage

```bash
llm-test-bench optimize [OPTIONS] --current-model <MODEL> --monthly-requests <N>
```

#### Options

- `--current-model <MODEL>` - Current model (format: provider:model or model)
- `--quality-threshold <THRESHOLD>` - Quality threshold 0.0-1.0 (default: 0.75)
- `--monthly-requests <N>` - Monthly request volume
- `--history <PATH>` - Historical results for analysis
- `--max-cost-increase <PERCENT>` - Max acceptable cost increase % (default: 10.0)
- `--min-quality <SCORE>` - Minimum required quality score (default: 0.70)
- `--include-experimental` - Include experimental models
- `--output <FORMAT>` - Output format: detailed, summary, json (default: detailed)
- `--report-file <PATH>` - Save optimization report
- `--config <PATH>` - Path to custom configuration file

#### Examples

```bash
# Basic optimization
llm-test-bench optimize \
  --current-model gpt-4 \
  --monthly-requests 100000

# Optimization with constraints
llm-test-bench optimize \
  --current-model openai:gpt-4 \
  --monthly-requests 100000 \
  --quality-threshold 0.85 \
  --max-cost-increase 5.0

# Save detailed report
llm-test-bench optimize \
  --current-model gpt-4 \
  --monthly-requests 50000 \
  --output detailed \
  --report-file optimization-report.json
```

---

### `config` - Configuration Management

Manage configuration files and settings.

#### Subcommands

#### `config show`

Display current configuration.

```bash
llm-test-bench config show [OPTIONS]
```

Options:
- `--format <FORMAT>` - Output format: toml, json (default: toml)
- `--config <PATH>` - Path to configuration file

Example:
```bash
llm-test-bench config show
llm-test-bench config show --format json
```

#### `config init`

Initialize a new configuration file.

```bash
llm-test-bench config init [OPTIONS]
```

Options:
- `--force` - Overwrite existing configuration
- `--path <PATH>` - Custom configuration path

Example:
```bash
llm-test-bench config init
llm-test-bench config init --path ./custom-config.toml
```

#### `config validate`

Validate configuration file.

```bash
llm-test-bench config validate [OPTIONS]
```

Options:
- `--config <PATH>` - Path to configuration file

Example:
```bash
llm-test-bench config validate
llm-test-bench config validate --config ./my-config.toml
```

#### `config path`

Show configuration file path.

```bash
llm-test-bench config path
```

---

### `completions` - Shell Completions

Generate shell completion scripts.

#### Usage

```bash
llm-test-bench completions <SHELL>
```

#### Supported Shells

- `bash`
- `zsh`
- `fish`
- `powershell`
- `elvish`

#### Examples

```bash
# Bash
llm-test-bench completions bash > ~/.local/share/bash-completion/completions/llm-test-bench

# Zsh
llm-test-bench completions zsh > ~/.zfunc/_llm-test-bench

# Fish
llm-test-bench completions fish > ~/.config/fish/completions/llm-test-bench.fish
```

---

## Configuration File

The CLI uses a hierarchical configuration system:

1. CLI Arguments (highest priority)
2. Environment Variables (LLM_TEST_BENCH_ prefix)
3. Config Files (~/.config/llm-test-bench/config.toml)
4. Defaults (lowest priority)

### Configuration Example

```toml
# ~/.config/llm-test-bench/config.toml

[providers.openai]
api_key_env = "OPENAI_API_KEY"
base_url = "https://api.openai.com/v1"
default_model = "gpt-4-turbo"
timeout_seconds = 30
max_retries = 3

[providers.anthropic]
api_key_env = "ANTHROPIC_API_KEY"
base_url = "https://api.anthropic.com/v1"
default_model = "claude-3-sonnet-20240229"
timeout_seconds = 30
max_retries = 3

[benchmarks]
output_dir = "./bench-results"
save_responses = true
parallel_requests = 5
continue_on_failure = true

[evaluation]
metrics = ["perplexity", "faithfulness", "relevance", "latency"]
llm_judge_model = "gpt-4"
confidence_threshold = 0.7
include_explanations = true

[orchestration]
max_parallel_models = 5
comparison_timeout_seconds = 300
routing_strategy = "quality_first"
enable_caching = true

[analytics]
confidence_level = 0.95
effect_size_threshold = 0.2
quality_threshold = 0.75
min_sample_size = 30

[dashboard]
theme = "auto"
chart_colors = ["#3B82F6", "#10B981", "#F59E0B", "#EF4444"]
max_data_points = 1000
enable_interactive = true
```

### Environment Variables

Override configuration with environment variables:

```bash
export LLM_TEST_BENCH_PROVIDERS__OPENAI__DEFAULT_MODEL="gpt-4"
export LLM_TEST_BENCH_BENCHMARKS__PARALLEL_REQUESTS=10
export LLM_TEST_BENCH_EVALUATION__LLM_JUDGE_MODEL="claude-3-opus"
```

---

## Common Workflows

### 1. Benchmark and Compare

```bash
# Run benchmark
llm-test-bench bench \
  --dataset tests.json \
  --providers openai,anthropic \
  --metrics faithfulness,relevance \
  --dashboard

# Compare specific models
llm-test-bench compare \
  --dataset tests.json \
  --models openai:gpt-4,anthropic:claude-3-opus \
  --statistical-tests \
  --output-file comparison.json

# Generate dashboard
llm-test-bench dashboard \
  --results bench-results/*.json,comparison.json \
  --title "Complete Analysis" \
  --output full-dashboard.html
```

### 2. CI/CD Regression Detection

```bash
# In CI pipeline
llm-test-bench bench \
  --dataset regression-tests.json \
  --providers openai \
  --output ./ci-results

llm-test-bench analyze \
  --baseline prod-baseline.json \
  --comparison ci-results/openai-results.json \
  --fail-on-regression \
  --confidence-level 0.99 \
  --output summary

# Exit code 2 if regression detected
```

### 3. Cost Optimization Analysis

```bash
# Analyze current costs
llm-test-bench optimize \
  --current-model gpt-4 \
  --monthly-requests 1000000 \
  --quality-threshold 0.85 \
  --report-file optimization-report.json

# Test recommended alternative
llm-test-bench compare \
  --dataset sample-tests.json \
  --models openai:gpt-4,anthropic:claude-3-sonnet \
  --metrics faithfulness,relevance,coherence \
  --dashboard
```

---

## Troubleshooting

### API Key Not Found

```bash
export OPENAI_API_KEY="your-api-key"
export ANTHROPIC_API_KEY="your-api-key"
```

### Configuration Issues

```bash
# Validate configuration
llm-test-bench config validate

# Show current configuration
llm-test-bench config show

# Reset to defaults
llm-test-bench config init --force
```

### Verbose Output

Enable verbose mode for debugging:

```bash
llm-test-bench --verbose <command> <options>
```

---

## Exit Codes

- `0` - Success
- `1` - General error
- `2` - Regression detected (with --fail-on-regression)
- `3` - Configuration error
- `4` - Invalid input
- `5` - Provider error (API key missing, rate limit, etc.)
- `6` - Cost limit exceeded

---

## Support

For issues and questions:
- GitHub Issues: https://github.com/llm-test-bench/llm-test-bench/issues
- Documentation: https://github.com/llm-test-bench/llm-test-bench/docs

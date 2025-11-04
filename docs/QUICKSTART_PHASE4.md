# Phase 4 CLI - Quick Start Guide

## Installation

```bash
# Clone the repository
git clone https://github.com/llm-test-bench/llm-test-bench
cd llm-test-bench

# Build the project
cargo build --release

# The binary will be at: target/release/llm-test-bench
```

## Setup

### 1. Set API Keys

```bash
export OPENAI_API_KEY="your-openai-key"
export ANTHROPIC_API_KEY="your-anthropic-key"
```

### 2. Initialize Configuration (Optional)

```bash
llm-test-bench config init
```

This creates `~/.config/llm-test-bench/config.toml`

## Basic Usage

### Compare Models

```bash
# Simple comparison
llm-test-bench compare \
  --prompt "Explain machine learning" \
  --models openai:gpt-4,anthropic:claude-3-opus

# With metrics and statistics
llm-test-bench compare \
  --prompt "Explain machine learning" \
  --models openai:gpt-4,anthropic:claude-3-opus \
  --metrics faithfulness,relevance \
  --statistical-tests \
  --dashboard
```

### Generate Dashboard

```bash
# From benchmark results
llm-test-bench dashboard \
  --results bench-results/*.json \
  --output dashboard.html

# Custom themed dashboard
llm-test-bench dashboard \
  --results results.json \
  --theme dark \
  --title "My Analysis" \
  --output my-dashboard.html
```

### Analyze Performance

```bash
# Detect regressions
llm-test-bench analyze \
  --baseline old-results.json \
  --comparison new-results.json

# For CI/CD (exits with code 2 if regression)
llm-test-bench analyze \
  --baseline baseline.json \
  --comparison pr-results.json \
  --fail-on-regression \
  --confidence-level 0.99
```

### Optimize Costs

```bash
# Get recommendations
llm-test-bench optimize \
  --current-model gpt-4 \
  --monthly-requests 100000 \
  --quality-threshold 0.85

# Save detailed report
llm-test-bench optimize \
  --current-model gpt-4 \
  --monthly-requests 100000 \
  --output json \
  --report-file optimization.json
```

## Common Workflows

### 1. Model Selection Workflow

```bash
# Step 1: Compare candidates
llm-test-bench compare \
  --dataset sample-tests.json \
  --models openai:gpt-4,openai:gpt-3.5-turbo,anthropic:claude-3-sonnet \
  --metrics faithfulness,relevance \
  --output-file comparison.json

# Step 2: Analyze costs
llm-test-bench optimize \
  --current-model gpt-4 \
  --monthly-requests 100000

# Step 3: Generate report
llm-test-bench dashboard \
  --results comparison.json \
  --dashboard-type comparison \
  --output selection-report.html
```

### 2. CI/CD Integration

```bash
#!/bin/bash
# ci-regression-check.sh

# Run tests
llm-test-bench bench \
  --dataset regression-tests.json \
  --providers openai \
  --output ./ci-results

# Check for regressions
llm-test-bench analyze \
  --baseline prod-baseline.json \
  --comparison ci-results/openai-results.json \
  --fail-on-regression \
  --confidence-level 0.95

# Generate report (if no regression)
if [ $? -eq 0 ]; then
  llm-test-bench dashboard \
    --results ci-results/*.json \
    --output ci-report.html
fi
```

### 3. Cost Monitoring Workflow

```bash
# Weekly cost analysis
llm-test-bench optimize \
  --current-model gpt-4 \
  --monthly-requests 500000 \
  --report-file "reports/cost-$(date +%Y%m%d).json"

# Compare with alternative
llm-test-bench compare \
  --dataset production-sample.json \
  --models openai:gpt-4,anthropic:claude-3-sonnet \
  --output-file comparison.json

# Generate dashboard
llm-test-bench dashboard \
  --results comparison.json \
  --title "Weekly Cost Analysis" \
  --output "reports/dashboard-$(date +%Y%m%d).html"
```

## Command Aliases

Save typing with short aliases:

```bash
llm-test-bench c      # compare
llm-test-bench d      # dashboard
llm-test-bench a      # analyze
llm-test-bench o      # optimize
llm-test-bench b      # bench
llm-test-bench t      # test
llm-test-bench e      # eval
```

## Output Formats

Most commands support multiple output formats:

```bash
# Table (default, human-readable)
--output table

# JSON (for scripts/APIs)
--output json

# Summary (concise)
--output summary

# Detailed (verbose)
--output detailed
```

## Getting Help

```bash
# General help
llm-test-bench --help

# Command-specific help
llm-test-bench compare --help
llm-test-bench dashboard --help
llm-test-bench analyze --help
llm-test-bench optimize --help

# With verbose output
llm-test-bench --verbose <command>
```

## Configuration

### Environment Variables

Override any config setting:

```bash
# Provider settings
export LLM_TEST_BENCH_PROVIDERS__OPENAI__DEFAULT_MODEL="gpt-4"
export LLM_TEST_BENCH_PROVIDERS__OPENAI__TIMEOUT_SECONDS=60

# Benchmark settings
export LLM_TEST_BENCH_BENCHMARKS__PARALLEL_REQUESTS=10

# Analytics settings
export LLM_TEST_BENCH_ANALYTICS__CONFIDENCE_LEVEL=0.99
```

### Configuration File

Edit `~/.config/llm-test-bench/config.toml`:

```toml
[orchestration]
max_parallel_models = 5
comparison_timeout_seconds = 300

[analytics]
confidence_level = 0.95
quality_threshold = 0.75

[dashboard]
theme = "dark"
enable_interactive = true
```

## Shell Completions

### Bash

```bash
llm-test-bench completions bash > ~/.local/share/bash-completion/completions/llm-test-bench
```

### Zsh

```bash
llm-test-bench completions zsh > ~/.zfunc/_llm-test-bench
```

### Fish

```bash
llm-test-bench completions fish > ~/.config/fish/completions/llm-test-bench.fish
```

## Troubleshooting

### API Key Not Found

```bash
# Check if set
echo $OPENAI_API_KEY

# Set if missing
export OPENAI_API_KEY="sk-..."
```

### Configuration Issues

```bash
# Validate config
llm-test-bench config validate

# Show current config
llm-test-bench config show

# Reset to defaults
llm-test-bench config init --force
```

### Verbose Mode

Enable for debugging:

```bash
llm-test-bench --verbose compare --prompt "test" --models openai:gpt-4,anthropic:claude-3
```

## Exit Codes

- `0` - Success
- `1` - General error
- `2` - Regression detected (with --fail-on-regression)
- `3` - Configuration error
- `4` - Invalid input
- `5` - Provider error
- `6` - Cost limit exceeded

## Tips & Tricks

### 1. Save Common Commands as Scripts

```bash
# compare-models.sh
#!/bin/bash
llm-test-bench compare \
  --models openai:gpt-4,anthropic:claude-3-opus \
  --metrics faithfulness,relevance \
  --statistical-tests \
  --dashboard \
  "$@"
```

### 2. Use Configuration for Defaults

Instead of passing options every time, set defaults in config:

```toml
[orchestration]
max_parallel_models = 10  # Instead of --concurrency 10

[analytics]
confidence_level = 0.99   # Instead of --confidence-level 0.99
```

### 3. Chain Commands

```bash
# Compare, then analyze, then dashboard
llm-test-bench compare \
  --dataset tests.json \
  --models openai:gpt-4,anthropic:claude-3-opus \
  --output-file comparison.json && \
llm-test-bench analyze \
  --baseline baseline.json \
  --comparison comparison.json \
  --report-file analysis.json && \
llm-test-bench dashboard \
  --results comparison.json,analysis.json \
  --output complete-report.html
```

### 4. Use Aliases in Shell

Add to `~/.bashrc` or `~/.zshrc`:

```bash
alias ltb='llm-test-bench'
alias ltbc='llm-test-bench compare'
alias ltbd='llm-test-bench dashboard'
alias ltba='llm-test-bench analyze'
alias ltbo='llm-test-bench optimize'
```

Then:

```bash
ltbc --prompt "test" --models openai:gpt-4,anthropic:claude-3
```

## Examples Library

### Example 1: Quick Model Test

```bash
llm-test-bench compare \
  --prompt "What is 2+2?" \
  --models openai:gpt-4,anthropic:claude-3-opus \
  --output table
```

### Example 2: Quality vs Cost Analysis

```bash
llm-test-bench compare \
  --dataset quality-tests.json \
  --models openai:gpt-4,openai:gpt-3.5-turbo \
  --metrics faithfulness,relevance \
  --statistical-tests \
  --output-file quality-comparison.json

llm-test-bench optimize \
  --current-model gpt-4 \
  --monthly-requests 1000000 \
  --quality-threshold 0.8
```

### Example 3: Regression Testing

```bash
# Before deployment
llm-test-bench bench \
  --dataset production-tests.json \
  --providers openai \
  --output ./baseline-results

# After changes
llm-test-bench bench \
  --dataset production-tests.json \
  --providers openai \
  --output ./new-results

# Check for regressions
llm-test-bench analyze \
  --baseline baseline-results/openai-results.json \
  --comparison new-results/openai-results.json \
  --fail-on-regression
```

## Advanced Features

### Custom Judge Models

```bash
llm-test-bench compare \
  --prompt "Complex reasoning task" \
  --models openai:gpt-4,anthropic:claude-3-opus \
  --metrics faithfulness,relevance \
  --judge-model gpt-4 \
  --judge-provider openai
```

### Batch Processing

```bash
# Process multiple datasets
for dataset in datasets/*.json; do
  llm-test-bench compare \
    --dataset "$dataset" \
    --models openai:gpt-4,anthropic:claude-3-opus \
    --output-file "results/$(basename $dataset)"
done

# Generate combined dashboard
llm-test-bench dashboard \
  --results results/*.json \
  --output combined-dashboard.html
```

### Cost Forecasting

```bash
# Current usage
llm-test-bench optimize \
  --current-model gpt-4 \
  --monthly-requests 100000

# Projected usage
llm-test-bench optimize \
  --current-model gpt-4 \
  --monthly-requests 500000 \
  --quality-threshold 0.85
```

## Next Steps

1. Read the [CLI Reference](CLI_REFERENCE.md) for complete documentation
2. Check [Implementation Summary](PHASE4_INTEGRATION_SUMMARY.md) for technical details
3. Review example datasets in `/datasets`
4. Join the community for support

## Support

- GitHub Issues: https://github.com/llm-test-bench/llm-test-bench/issues
- Documentation: https://github.com/llm-test-bench/llm-test-bench/docs
- Examples: https://github.com/llm-test-bench/llm-test-bench/examples

---

**Last Updated:** 2025-11-04
**Version:** Phase 4 Integration

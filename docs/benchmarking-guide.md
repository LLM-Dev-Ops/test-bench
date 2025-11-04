# LLM Test Bench - Benchmarking Guide

## Table of Contents

1. [Quick Start](#quick-start)
2. [Dataset Format](#dataset-format)
3. [Command Options](#command-options)
4. [Multi-Provider Benchmarking](#multi-provider-benchmarking)
5. [Output Formats](#output-formats)
6. [Best Practices](#best-practices)
7. [Troubleshooting](#troubleshooting)

## Quick Start

### Running Your First Benchmark

```bash
# Run a simple benchmark with one provider
llm-test-bench bench \
  --dataset datasets/examples/quick-start.json \
  --providers openai

# Run with multiple providers
llm-test-bench bench \
  --dataset datasets/examples/coding-tasks.json \
  --providers openai,anthropic \
  --concurrency 10
```

### Expected Output

```
LLM Test Bench - Benchmark Command

▶ Loading dataset...
  ✓ Loaded: quick-start (3 tests)

▶ Benchmarking provider openai (1/1)...
  ✓ Saved JSON: ./bench-results/openai-results.json
  ✓ Saved CSV: ./bench-results/openai-results.csv

Results for openai:
────────────────────────────────────────────────────────────
  ℹ Tests:        3
  ✓ Success:      3 (100.0%)

  ⏱ Avg Duration: 1234ms
  ℹ P50 Latency:  1200ms
  ℹ P95 Latency:  1500ms
  ℹ P99 Latency:  1500ms

  💰 Total Tokens: 245
  💰 Est. Cost:    $0.0123
────────────────────────────────────────────────────────────

✓ Benchmark complete!
Results saved to: ./bench-results
```

## Dataset Format

### JSON Format

```json
{
  "name": "my-benchmark",
  "description": "Description of what this benchmark tests",
  "version": "1.0.0",
  "defaults": {
    "temperature": 0.7,
    "max_tokens": 500
  },
  "test_cases": [
    {
      "id": "test-1",
      "category": "coding",
      "prompt": "Write a {{language}} function to {{task}}",
      "variables": {
        "language": "Python",
        "task": "reverse a string"
      },
      "expected": "def reverse",
      "references": ["[::-1]", "reversed()"],
      "config": {
        "model": "gpt-4",
        "temperature": 0.0
      }
    }
  ]
}
```

### YAML Format

```yaml
name: my-benchmark
description: Description of what this benchmark tests
version: 1.0.0

defaults:
  temperature: 0.7
  max_tokens: 500

test_cases:
  - id: test-1
    category: coding
    prompt: Write a {{language}} function to {{task}}
    variables:
      language: Python
      task: reverse a string
    expected: "def reverse"
    references:
      - "[::-1]"
      - "reversed()"
    config:
      model: gpt-4
      temperature: 0.0
```

### Schema Fields

#### Dataset Level

- **name** (required): Unique identifier for the dataset
- **description** (optional): Human-readable description
- **version** (required): Semantic version (e.g., "1.0.0")
- **defaults** (optional): Default configuration for all tests
- **test_cases** (required): Array of test cases (minimum 1)
- **metadata** (optional): Custom metadata as key-value pairs

#### Test Case Level

- **id** (required): Unique identifier for the test
- **category** (optional): Category or tag for grouping
- **prompt** (required): The prompt text (supports {{variable}} syntax)
- **variables** (optional): Key-value pairs for template substitution
- **expected** (optional): Expected string in the output
- **references** (optional): Array of reference strings for evaluation
- **config** (optional): Per-test configuration overrides
- **metadata** (optional): Test-specific metadata

### Template Variables

Use `{{variable_name}}` syntax in prompts:

```json
{
  "prompt": "Translate '{{text}}' to {{language}}",
  "variables": {
    "text": "Hello, world!",
    "language": "French"
  }
}
```

The prompt will be rendered as:
```
Translate 'Hello, world!' to French
```

## Command Options

### Basic Options

```bash
llm-test-bench bench [OPTIONS] --dataset <PATH> --providers <LIST>
```

### All Options

| Option | Short | Default | Description |
|--------|-------|---------|-------------|
| `--dataset` | `-d` | (required) | Path to dataset file (JSON or YAML) |
| `--providers` | `-p` | (required) | Comma-separated list of providers |
| `--concurrency` | `-c` | 5 | Number of concurrent requests |
| `--output` | `-o` | `./bench-results` | Output directory for results |
| `--export` | `-e` | `both` | Export format: json, csv, or both |
| `--continue-on-failure` | | `true` | Continue if individual tests fail |
| `--save-responses` | | `true` | Save raw responses to disk |
| `--delay` | | (none) | Delay between requests (ms) |
| `--config` | | (auto) | Path to custom config file |
| `--verbose` | `-v` | `false` | Enable verbose output |

### Examples

```bash
# Basic usage
llm-test-bench bench -d dataset.json -p openai

# High concurrency
llm-test-bench bench -d dataset.json -p openai -c 20

# With delay to avoid rate limits
llm-test-bench bench -d dataset.json -p openai --delay 1000

# JSON output only
llm-test-bench bench -d dataset.json -p openai -e json

# Custom output directory
llm-test-bench bench -d dataset.json -p openai -o ./my-results

# Stop on first failure
llm-test-bench bench -d dataset.json -p openai --continue-on-failure=false

# Multiple providers
llm-test-bench bench -d dataset.json -p openai,anthropic -c 10

# Verbose output
llm-test-bench bench -d dataset.json -p openai -v
```

## Multi-Provider Benchmarking

### Running Multiple Providers

```bash
llm-test-bench bench \
  --dataset datasets/coding-tasks.json \
  --providers openai,anthropic \
  --concurrency 5
```

### Output Structure

```
bench-results/
├── openai/
│   ├── test-1.json
│   ├── test-2.json
│   └── ...
├── anthropic/
│   ├── test-1.json
│   ├── test-2.json
│   └── ...
├── openai-results.json
├── openai-results.csv
├── anthropic-results.json
└── anthropic-results.csv
```

### Comparing Results

After running benchmarks, you can compare providers:

```bash
# View summaries
cat bench-results/openai-results.json | jq '.summary'
cat bench-results/anthropic-results.json | jq '.summary'

# Compare in spreadsheet
# Open both CSV files in Excel or Google Sheets
```

## Output Formats

### JSON Output

Complete benchmark results with all metadata:

```json
{
  "dataset_name": "coding-tasks",
  "provider_name": "openai",
  "total_tests": 5,
  "started_at": "2025-11-04T12:00:00Z",
  "completed_at": "2025-11-04T12:01:30Z",
  "total_duration_ms": 90000,
  "results": [
    {
      "test_id": "test-1",
      "category": "coding",
      "status": "success",
      "response": {
        "id": "resp-123",
        "model": "gpt-4",
        "content": "def reverse_string(s):\n    return s[::-1]",
        "usage": {
          "prompt_tokens": 25,
          "completion_tokens": 15,
          "total_tokens": 40
        },
        "finish_reason": "stop",
        "created_at": "2025-11-04T12:00:05Z"
      },
      "duration_ms": 1234,
      "timestamp": "2025-11-04T12:00:05Z"
    }
  ],
  "summary": {
    "total": 5,
    "succeeded": 5,
    "failed": 0,
    "timeout": 0,
    "skipped": 0,
    "success_rate": 1.0,
    "avg_duration_ms": 1200.5,
    "p50_duration_ms": 1180.0,
    "p95_duration_ms": 1450.0,
    "p99_duration_ms": 1500.0,
    "total_tokens": 245,
    "total_cost": 0.0123
  }
}
```

### CSV Output

Tabular format for spreadsheet analysis:

| test_id | category | status | duration_ms | tokens | cost | model | error |
|---------|----------|--------|-------------|--------|------|-------|-------|
| test-1 | coding | Success | 1234 | 40 | 0.002 | gpt-4 | |
| test-2 | coding | Success | 1156 | 35 | 0.0018 | gpt-4 | |

Fields include:
- test_id, category, status
- duration_ms, tokens, cost
- model, prompt_length, response_length
- prompt_tokens, completion_tokens
- finish_reason, error, timestamp

## Best Practices

### Concurrency

**Start Low**
```bash
# Begin with conservative concurrency
llm-test-bench bench -d dataset.json -p openai -c 3
```

**Scale Up Gradually**
```bash
# Increase if no rate limiting
llm-test-bench bench -d dataset.json -p openai -c 10
```

**Monitor for Rate Limits**
- OpenAI: ~3,500 requests/min for GPT-4
- Anthropic: Varies by tier
- Add delays if you hit limits: `--delay 500`

### Delays and Rate Limiting

```bash
# Add 1 second delay between requests
llm-test-bench bench -d dataset.json -p openai --delay 1000

# Good for:
# - Avoiding rate limits
# - Being considerate to APIs
# - Ensuring stable measurements
```

### Dataset Organization

```
datasets/
├── examples/          # Sample datasets
├── production/        # Real benchmarks
│   ├── coding/
│   ├── reasoning/
│   └── summarization/
└── experiments/       # Testing new prompts
```

### Naming Conventions

- **Datasets**: `category-type.json` (e.g., `coding-tasks.json`)
- **Test IDs**: `category-name-variant` (e.g., `fizzbuzz-python-v1`)
- **Categories**: Consistent naming (e.g., `coding`, `reasoning`, `summarization`)

### Cost Management

**Estimate Costs First**
```bash
# Run with small dataset first
llm-test-bench bench -d quick-start.json -p openai

# Check cost in output:
# Est. Cost: $0.0123

# Then scale to full dataset
```

**Use Temperature Wisely**
- Lower temperature (0.0-0.3): More consistent, cheaper
- Higher temperature (0.7-1.0): More creative, variable results

### Reproducibility

**Save Configuration**
```toml
# config.toml
[providers.openai]
default_model = "gpt-4-turbo-preview"
temperature = 0.0
max_tokens = 500
```

**Version Datasets**
```json
{
  "name": "coding-tasks",
  "version": "1.2.0",  # Increment when changing
  ...
}
```

**Use Random Seeds**
```bash
# Not yet supported, coming soon
# --random-seed 42
```

## Troubleshooting

### Common Issues

#### Dataset Not Found
```
Error: Dataset file not found: datasets/my-dataset.json
```

**Solution**: Check the file path and ensure it exists
```bash
ls -la datasets/
llm-test-bench bench -d datasets/examples/quick-start.json -p openai
```

#### Provider Not Configured
```
Error: Provider 'openai' not found in configuration
```

**Solution**: Initialize configuration
```bash
llm-test-bench config init
# Then edit ~/.config/llm-test-bench/config.toml
```

#### Missing API Key
```
Error: Provider error: Invalid API key
```

**Solution**: Set the environment variable
```bash
export OPENAI_API_KEY="sk-..."
export ANTHROPIC_API_KEY="sk-ant-..."
```

#### Rate Limiting
```
Provider error: Rate limit exceeded. Retry after 20 seconds
```

**Solution**: Add delays or reduce concurrency
```bash
llm-test-bench bench -d dataset.json -p openai -c 2 --delay 2000
```

#### Invalid Dataset Schema
```
Error: Dataset validation failed: test_cases: minimum length is 1
```

**Solution**: Ensure dataset has at least one test case
```json
{
  "name": "test",
  "version": "1.0.0",
  "test_cases": [
    {
      "id": "test-1",
      "prompt": "Hello, world!"
    }
  ]
}
```

#### Template Variable Missing
```
Error: Template error: Missing variables: name
```

**Solution**: Provide all required variables
```json
{
  "prompt": "Hello, {{name}}!",
  "variables": {
    "name": "Alice"
  }
}
```

### Debug Mode

```bash
# Enable verbose output
llm-test-bench bench -d dataset.json -p openai -v

# Check raw responses
cat bench-results/openai/test-1.json | jq .
```

### Performance Issues

**Slow Execution**
- Increase concurrency: `-c 10`
- Check network connection
- Verify API endpoint availability

**High Costs**
- Use cheaper models in `config`
- Reduce `max_tokens` in defaults
- Filter dataset to specific categories

**Memory Usage**
- Disable response saving: `--save-responses=false`
- Process smaller batches
- Monitor with `htop` or Task Manager

## Advanced Usage

### Custom Configuration

```bash
llm-test-bench bench \
  --dataset dataset.json \
  --providers openai \
  --config ./custom-config.toml
```

### Filtering by Category

Create filtered datasets:
```bash
# Using jq
cat datasets/coding-tasks.json | \
  jq '.test_cases |= map(select(.category == "algorithms"))' > \
  datasets/algorithms-only.json
```

### Batch Processing

```bash
# Run multiple benchmarks
for dataset in datasets/production/*.json; do
  llm-test-bench bench -d "$dataset" -p openai,anthropic -o "results/$(basename $dataset .json)"
done
```

### Integration with CI/CD

```yaml
# .github/workflows/benchmark.yml
name: LLM Benchmarks
on: [push]
jobs:
  benchmark:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - name: Run benchmarks
        env:
          OPENAI_API_KEY: ${{ secrets.OPENAI_API_KEY }}
        run: |
          llm-test-bench bench \
            -d datasets/regression-tests.json \
            -p openai \
            -o results/
      - name: Upload results
        uses: actions/upload-artifact@v2
        with:
          name: benchmark-results
          path: results/
```

## Example Workflows

### Development Workflow

1. **Create Dataset**
   ```bash
   cp datasets/examples/quick-start.json datasets/my-test.json
   # Edit datasets/my-test.json
   ```

2. **Test Locally**
   ```bash
   llm-test-bench bench -d datasets/my-test.json -p openai -c 1
   ```

3. **Review Results**
   ```bash
   cat bench-results/openai-results.json | jq '.summary'
   ```

4. **Iterate**
   ```bash
   # Adjust prompts, re-run benchmark
   ```

### Production Workflow

1. **Run Full Benchmark**
   ```bash
   llm-test-bench bench \
     -d datasets/production/full-suite.json \
     -p openai,anthropic \
     -c 10 \
     -o results/$(date +%Y%m%d)/
   ```

2. **Export to Spreadsheet**
   ```bash
   # Open results/*-results.csv in Excel/Google Sheets
   ```

3. **Archive Results**
   ```bash
   tar -czf results-$(date +%Y%m%d).tar.gz results/$(date +%Y%m%d)/
   ```

## Further Reading

- [Dataset Schema Reference](./dataset-schema.md)
- [Provider Configuration](./configuration.md)
- [API Reference](./api-reference.md)
- [Contributing Guidelines](../CONTRIBUTING.md)

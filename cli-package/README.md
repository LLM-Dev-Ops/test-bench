# @llm-dev-ops/test-bench-cli

**CLI wrapper for LLM Test Bench** - A production-grade framework for testing and benchmarking Large Language Models.

## Installation

```bash
# Install globally
npm install -g @llm-dev-ops/test-bench-cli

# Or use with npx (no installation required)
npx @llm-dev-ops/test-bench-cli --help
```

## Usage

After installation, you can use the `ltb` command:

```bash
# Show help
ltb --help

# Show version
ltb --version

# Run a benchmark
ltb benchmark --provider openai --model gpt-4 --prompt "Explain quantum computing"

# Compare models
ltb compare --providers openai:gpt-4,anthropic:claude-opus-4 --prompt "Write a poem"

# Run evaluation
ltb evaluate --file responses.json --evaluator coherence
```

## Commands

- `ltb benchmark` - Run benchmarks on LLM models
- `ltb compare` - Compare multiple models
- `ltb evaluate` - Evaluate model responses
- `ltb analyze` - Analyze benchmark results
- `ltb visualize` - Generate visualization dashboards

## Prerequisites

This CLI requires the Rust-based LLM Test Bench binary to be installed:

```bash
# Install via Cargo
cargo install llm-test-bench

# Or download from releases
# https://github.com/LLM-Dev-Ops/test-bench/releases
```

## SDK Package

For programmatic access in TypeScript/JavaScript, use the SDK package:

```bash
npm install @llm-dev-ops/test-bench
```

```typescript
import { LLMTestBench } from '@llm-dev-ops/test-bench';

const bench = new LLMTestBench();
const results = await bench.benchmark({
  provider: 'openai',
  model: 'gpt-4',
  prompts: ['Test prompt']
});
```

## Documentation

- [Full Documentation](https://github.com/LLM-Dev-Ops/test-bench#readme)
- [CLI Reference](https://github.com/LLM-Dev-Ops/test-bench/blob/main/docs/CLI_REFERENCE.md)
- [Configuration Guide](https://github.com/LLM-Dev-Ops/test-bench/blob/main/docs/CONFIGURATION.md)

## License

MIT

## Repository

https://github.com/LLM-Dev-Ops/test-bench

# LLM Test Bench CLI

A production-grade CLI for testing and benchmarking Large Language Model (LLM) applications.

## Phase 1 Implementation Status

This is the **Phase 1 (Milestone 1.3)** implementation of the CLI scaffolding, focusing on command structure and configuration management.

### Completed Features

- Complete Clap-based CLI command structure with derive API
- `config init` command with interactive setup wizard
- Command stubs for `test`, `bench`, and `eval` (full implementation in later phases)
- Shell completion generation for bash, zsh, fish, PowerShell, and elvish
- Comprehensive integration tests with assert_cmd (30 tests, all passing)
- Error handling with anyhow
- Global flags for verbose output and color control

## Installation

### Building from Source

```bash
# Clone the repository
git clone https://github.com/llm-test-bench/llm-test-bench.git
cd llm-test-bench

# Build the CLI
cargo build --release --package llm-test-bench

# The binary will be at: target/release/llm-test-bench
```

### Development Build

```bash
cargo build --package llm-test-bench
```

## Usage

### Basic Commands

```bash
# Show help
llm-test-bench --help

# Show version
llm-test-bench --version

# Enable verbose output (global flag)
llm-test-bench --verbose <COMMAND>

# Disable colored output
llm-test-bench --no-color <COMMAND>
```

### Configuration Management

#### Initialize Configuration

```bash
# Interactive setup wizard
llm-test-bench config init

# Non-interactive mode (uses defaults)
llm-test-bench config init --non-interactive

# Initialize specific provider only
llm-test-bench config init --provider openai
```

The `config init` command will:
1. Guide you through an interactive setup
2. Create a configuration file at `~/.config/llm-test-bench/config.toml`
3. Prompt for API key preferences (environment variable recommended)
4. Configure default models and parameters

#### Show Configuration

```bash
# Display current configuration
llm-test-bench config show

# Show with full TOML content
llm-test-bench config show --verbose
```

#### Validate Configuration

```bash
# Validate default config file
llm-test-bench config validate

# Validate specific config file
llm-test-bench config validate --config /path/to/config.toml
```

### Test Command (Phase 2)

```bash
# Run a single test (stub - coming in Phase 2)
llm-test-bench test openai --prompt "Hello, world!" --model gpt-4

# With additional parameters
llm-test-bench test anthropic \
  --prompt "Explain quantum computing" \
  --model claude-sonnet-4 \
  --temperature 0.7 \
  --max-tokens 1000

# With expected output for validation
llm-test-bench test openai \
  --prompt "What is 2+2?" \
  --expect "4"

# Save results to file
llm-test-bench test openai \
  --prompt "Test prompt" \
  --save ./results.json
```

### Benchmark Command (Phase 3)

```bash
# Run benchmark (stub - coming in Phase 3)
llm-test-bench bench \
  --dataset ./prompts.json \
  --providers openai,anthropic

# With concurrency and iterations
llm-test-bench bench \
  --dataset ./prompts.json \
  --providers openai,anthropic \
  --concurrency 4 \
  --iterations 10

# With caching and output format
llm-test-bench bench \
  --dataset ./dataset.json \
  --providers openai \
  --cache \
  --format html \
  --output ./benchmark-results
```

### Evaluation Command (Phase 4)

```bash
# Evaluate results (stub - coming in Phase 4)
llm-test-bench eval \
  --results ./results.json \
  --metrics accuracy,latency,cost

# With baseline comparison
llm-test-bench eval \
  --results ./results.json \
  --baseline ./baseline.json \
  --metrics all

# With visualizations
llm-test-bench eval \
  --results ./results.json \
  --visualize \
  --format html \
  --output ./evaluation-report
```

### Shell Completions

Generate shell completions for your shell:

```bash
# Bash
llm-test-bench completions bash > ~/.local/share/bash-completion/completions/llm-test-bench

# Zsh
llm-test-bench completions zsh > ~/.zfunc/_llm-test-bench

# Fish
llm-test-bench completions fish > ~/.config/fish/completions/llm-test-bench.fish

# PowerShell
llm-test-bench completions powershell > llm-test-bench.ps1

# Elvish
llm-test-bench completions elvish > ~/.elvish/lib/completions/llm-test-bench.elv
```

## Command Aliases

All major commands have short aliases:

- `test` → `t`
- `bench` → `b`
- `eval` → `e`

```bash
# These are equivalent:
llm-test-bench test openai --prompt "test"
llm-test-bench t openai --prompt "test"
```

## Configuration File Format

The configuration file uses TOML format and is stored at `~/.config/llm-test-bench/config.toml`:

```toml
version = "1.0"

[providers.openai]
type = "openai"
api_key_env = "OPENAI_API_KEY"  # Recommended: use environment variable
base_url = "https://api.openai.com/v1"

[providers.openai.defaults]
model = "gpt-4"
temperature = 0.7
max_tokens = 4096

[providers.anthropic]
type = "anthropic"
api_key_env = "ANTHROPIC_API_KEY"

[providers.anthropic.defaults]
model = "claude-sonnet-4-20250514"
temperature = 0.7

[defaults]
output_dir = "./test-results"
cache_enabled = true
```

## Testing

### Running Tests

```bash
# Run all CLI tests
cargo test --package llm-test-bench

# Run only unit tests
cargo test --package llm-test-bench --lib

# Run only integration tests
cargo test --package llm-test-bench --test integration

# Run with output
cargo test --package llm-test-bench -- --nocapture

# Run specific test
cargo test --package llm-test-bench test_help_command
```

### Test Coverage

Current test suite (Phase 1):
- **6 unit tests**: Argument parsing and configuration serialization
- **24 integration tests**: End-to-end CLI behavior testing

All 30 tests passing.

## Architecture

### CLI Structure

```
cli/
├── src/
│   ├── main.rs                      # Entry point, command routing
│   ├── commands/
│   │   ├── mod.rs                   # Command module declarations
│   │   ├── config.rs                # Config init/show/validate (COMPLETE)
│   │   ├── test.rs                  # Test command stub (Phase 2)
│   │   ├── bench.rs                 # Benchmark command stub (Phase 3)
│   │   └── eval.rs                  # Evaluation command stub (Phase 4)
│   └── lib.rs                       # (Future: shared utilities)
├── tests/
│   └── integration/
│       ├── main.rs                  # Test module entry
│       └── cli_tests.rs             # Integration tests
└── Cargo.toml
```

### Command Flow

```
User Input → Clap Parser → Command Router (main.rs) → Command Handler → Output
                    ↓
            Argument Validation
                    ↓
            Environment Setup
```

### Error Handling

The CLI uses `anyhow` for application-level error handling:

```rust
pub async fn execute(args: Args, verbose: bool) -> Result<()> {
    // Command implementation
    Ok(())
}
```

Errors are caught in `main.rs` and displayed with:
- Error message
- Chain of causes (in verbose mode)
- Appropriate exit code

## Development Guide

### Adding a New Command

1. Create a new file in `cli/src/commands/`:

```rust
// cli/src/commands/mynewcommand.rs
use anyhow::Result;
use clap::Args;

#[derive(Args, Debug)]
pub struct MyNewCommandArgs {
    /// Description of argument
    #[arg(short, long)]
    pub my_arg: String,
}

pub async fn execute(args: MyNewCommandArgs, verbose: bool) -> Result<()> {
    println!("Executing my new command");
    // Implementation here
    Ok(())
}
```

2. Add to `cli/src/commands/mod.rs`:

```rust
pub mod mynewcommand;
```

3. Add to command enum in `cli/src/main.rs`:

```rust
#[derive(Subcommand)]
enum Commands {
    // ... existing commands ...

    /// Description of my new command
    MyNewCommand(mynewcommand::MyNewCommandArgs),
}
```

4. Add to command router:

```rust
let result = match cli.command {
    // ... existing matches ...
    Commands::MyNewCommand(args) => mynewcommand::execute(args, cli.verbose).await,
};
```

5. Write integration tests in `cli/tests/integration/cli_tests.rs`.

### Adding Integration Tests

Use the `assert_cmd` crate for testing:

```rust
#[test]
fn test_my_new_command() {
    cli()
        .arg("mynewcommand")
        .arg("--my-arg")
        .arg("value")
        .assert()
        .success()
        .stdout(predicate::str::contains("expected output"));
}
```

## Phase 2+ Roadmap

### Phase 2: Test Command (Weeks 2-3)
- Implement provider integration (OpenAI, Anthropic)
- Add response generation and streaming
- Implement assertion validation
- Add result formatting and export

### Phase 3: Bench Command (Weeks 4-5)
- Implement multi-provider parallel execution
- Add dataset loading (JSON, YAML, CSV)
- Collect performance metrics
- Generate comparison reports

### Phase 4: Eval Command (Weeks 6-7)
- Implement metrics calculation
- Add baseline comparison
- Generate visualizations
- Create HTML/Markdown reports

## Contributing

When contributing to the CLI:

1. Follow Rust conventions and use `cargo fmt`
2. Add tests for all new features
3. Update this README for new commands
4. Ensure all tests pass: `cargo test --package llm-test-bench`
5. Test the CLI manually with various inputs

## License

MIT License - See LICENSE file

## Related Documentation

- [Main README](../README.md)
- [Architecture Documentation](../docs/ARCHITECTURE.md)
- [Implementation Roadmap](../docs/IMPLEMENTATION_ROADMAP.md)
- [Quick Reference](../docs/QUICK_REFERENCE.md)

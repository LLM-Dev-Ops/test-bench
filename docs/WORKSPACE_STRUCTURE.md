# LLM Test Bench - Workspace Structure Documentation

## Overview

This document describes the Cargo workspace structure for the LLM Test Bench project (Phase 1).

## Workspace Architecture

The project uses a Cargo workspace with three crates organized as follows:

```
llm-test-bench/
├── Cargo.toml              # Workspace root configuration
├── .rustfmt.toml           # Code formatting rules
├── .clippy.toml            # Linting configuration
├── LICENSE-MIT             # MIT license
├── LICENSE-APACHE          # Apache 2.0 license
├── README.md               # Project README
├── cli/                    # Binary crate (CLI interface)
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs
│   │   └── commands/       # CLI command implementations
│   └── tests/              # Integration tests
├── core/                   # Library crate (business logic)
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs
│   │   ├── providers/      # LLM provider implementations
│   │   │   ├── mod.rs
│   │   │   ├── openai.rs
│   │   │   └── anthropic.rs
│   │   ├── evaluators/     # Evaluation metrics
│   │   │   ├── mod.rs
│   │   │   ├── perplexity.rs
│   │   │   ├── faithfulness.rs
│   │   │   ├── relevance.rs
│   │   │   └── coherence.rs
│   │   ├── benchmarks/     # Benchmarking logic
│   │   │   ├── mod.rs
│   │   │   ├── runner.rs
│   │   │   └── reporter.rs
│   │   └── config/         # Configuration management
│   └── tests/              # Unit tests
└── datasets/               # Dataset management crate
    ├── Cargo.toml
    ├── src/
    │   ├── lib.rs
    │   ├── loader.rs       # Dataset loading/saving
    │   └── builtin.rs      # Built-in benchmark datasets
    └── tests/
```

## Crate Descriptions

### 1. CLI Crate (`cli/`)

**Package Name**: `llm-test-bench`
**Binary Name**: `llm-test-bench` (or `ltb` as alias)
**Purpose**: Command-line interface for the test bench

**Key Features**:
- Clap-based CLI with subcommands (test, bench, eval, config)
- Async Tokio runtime for concurrent operations
- Integration with core and datasets crates
- Shell completion generation

**Dependencies**:
- `llm-test-bench-core` - Core business logic
- `llm-test-bench-datasets` - Dataset management
- `clap` - CLI argument parsing
- `tokio` - Async runtime
- `anyhow` - Error handling

### 2. Core Crate (`core/`)

**Package Name**: `llm-test-bench-core`
**Purpose**: Core business logic and provider integrations

**Modules**:

#### `providers/`
LLM provider implementations with a unified trait-based interface:
- `Provider` trait - Common interface for all providers
- `OpenAIProvider` - OpenAI API integration (GPT-4, GPT-3.5)
- `AnthropicProvider` - Anthropic Claude API integration
- Request/response types with comprehensive serialization

#### `evaluators/`
Evaluation metrics for LLM outputs:
- `Evaluator` trait - Common interface for metrics
- `PerplexityEvaluator` - Language model prediction quality
- `FaithfulnessEvaluator` - Factual accuracy measurement
- `RelevanceEvaluator` - Task/prompt alignment scoring
- `CoherenceEvaluator` - Output fluency and consistency

#### `benchmarks/`
Benchmarking infrastructure:
- `BenchmarkRunner` - Executes benchmark suites
- `BenchmarkReporter` - Generates reports
- Latency metrics (P50, P95, P99)
- Token usage tracking

#### `config/`
Configuration management:
- TOML/JSON configuration support
- Environment variable integration
- Validation using serde_valid

**Dependencies**:
- `tokio` - Async runtime
- `reqwest` - HTTP client for API calls
- `serde` / `serde_json` - Serialization
- `async-trait` - Async trait support
- `thiserror` - Structured errors

### 3. Datasets Crate (`datasets/`)

**Package Name**: `llm-test-bench-datasets`
**Purpose**: Dataset management and built-in benchmarks

**Modules**:

#### `loader`
- Load datasets from JSON files
- Save datasets to JSON files
- List available datasets in directories

#### `builtin`
Pre-built benchmark datasets:
- `simple-prompts` - Basic testing (greetings, math, facts)
- `instruction-following` - Instruction adherence testing
- Format validation (JSON, lists, etc.)
- Multi-step reasoning tests

**Key Types**:
- `Dataset` - Collection of test cases with metadata
- `TestCase` - Individual test with prompt, expected output, tags
- `DatasetLoader` - File I/O operations

**Dependencies**:
- `serde` / `serde_json` - Serialization
- `dirs` - Path handling
- `thiserror` - Error types

## Dependency Graph

```
cli (binary)
├── core (library)
│   ├── tokio
│   ├── reqwest
│   ├── serde
│   ├── async-trait
│   └── thiserror
└── datasets (library)
    ├── serde
    └── thiserror

core is independent of datasets
cli depends on both core and datasets
```

## Workspace Configuration

### Shared Dependencies

The workspace root `Cargo.toml` defines shared dependencies that all crates can use:

```toml
[workspace.dependencies]
tokio = { version = "1.40", features = ["full"] }
clap = { version = "4.5", features = ["derive", "env", "color", "suggestions"] }
reqwest = { version = "0.12", features = ["json", "stream", "rustls-tls"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
config = "0.14"
toml = "0.8"
anyhow = "1.0"
thiserror = "1.0"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
```

### Linting Configuration

**Clippy Warnings** (workspace-wide):
- All clippy lints: warn
- Correctness: deny (compilation fails)
- Pedantic: warn
- Nursery: warn
- Cargo: warn

**Rust Lints**:
- Unsafe code: warn
- Missing docs: warn

### Code Formatting

`.rustfmt.toml` configures:
- Rust 2021 edition
- 100 character line width
- Import grouping and reordering
- Consistent formatting across all crates

## Licensing

**Dual License**: MIT OR Apache-2.0

All source files include the license header:

```rust
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
```

## Build Profiles

### Development (`dev`)
- No optimization (`opt-level = 0`)
- Full debug info
- Fast compilation

### Release (`release`)
- LTO enabled
- Single codegen unit
- Stripped binaries
- Maximum optimization (`opt-level = 3`)

### Test (`test`)
- Light optimization (`opt-level = 1`)
- Balance between speed and compile time

## Compilation

To build the workspace:

```bash
cargo build --workspace
```

To check without building:

```bash
cargo check --workspace
```

To run tests:

```bash
cargo test --workspace
```

To run clippy:

```bash
cargo clippy --workspace -- -D warnings
```

To format code:

```bash
cargo fmt --all
```

## Design Decisions

### 1. Workspace vs. Single Crate

**Decision**: Multi-crate workspace

**Rationale**:
- **Separation of Concerns**: CLI, business logic, and datasets are distinct
- **Reusability**: Core and datasets can be used independently
- **Parallel Compilation**: Cargo can build crates in parallel
- **Testing**: Each crate has isolated test suites

### 2. Provider Trait Architecture

**Decision**: Async trait-based abstraction

**Rationale**:
- **Extensibility**: Easy to add new providers
- **Type Safety**: Compile-time guarantees
- **Testability**: Mock implementations for testing
- **Performance**: Async/await for concurrent API calls

### 3. Error Handling Strategy

**Decision**: `thiserror` for libraries, `anyhow` for applications

**Rationale**:
- **Library Errors**: Structured, programmatic (thiserror)
- **Application Errors**: User-facing, contextual (anyhow)
- **Best Practice**: Follows Rust community conventions

### 4. Dual Licensing

**Decision**: MIT OR Apache-2.0

**Rationale**:
- **Rust Standard**: Matches Rust language and ecosystem
- **Flexibility**: Users can choose preferred license
- **Contribution Clarity**: Apache 2.0 has explicit patent grant
- **Adoption**: Broadest compatibility

## Architectural Patterns

### 1. Dependency Inversion

Core defines traits (`Provider`, `Evaluator`) that implementations adhere to.

### 2. Builder Pattern

Used in dataset construction:
```rust
TestCase::new(id, prompt)
    .with_expected_output(output)
    .with_tag(tag)
```

### 3. Factory Pattern

Built-in datasets via factory functions:
```rust
builtin::simple_prompts_dataset()
builtin::instruction_following_dataset()
```

### 4. Repository Pattern

`DatasetLoader` abstracts dataset persistence.

## Next Steps for Other Agents

### Backend Agent
- Implement provider API integrations (OpenAI, Anthropic)
- Add actual HTTP request logic in `providers/`
- Implement evaluation metrics algorithms
- Add caching layer for API responses

### Testing Agent
- Write integration tests for CLI commands
- Add unit tests for providers (with mocks)
- Create benchmark tests using criterion
- Add property-based tests where applicable

### DevOps Agent
- Set up CI/CD pipeline
- Configure automated testing
- Add code coverage reporting
- Set up release automation

### Documentation Agent
- Generate API documentation (`cargo doc`)
- Write user guide and tutorials
- Add inline code examples
- Create migration guides

## Module Status

### ✅ Completed (Phase 1)

- [x] Workspace structure
- [x] Crate scaffolding (cli, core, datasets)
- [x] License files (MIT + Apache-2.0)
- [x] Provider trait definitions
- [x] Evaluator trait definitions
- [x] Dataset structures
- [x] Built-in datasets
- [x] Rustfmt configuration
- [x] Clippy configuration

### 🚧 Stubbed (Future Phases)

- [ ] OpenAI API implementation
- [ ] Anthropic API implementation
- [ ] Evaluation metrics algorithms
- [ ] Benchmark runner logic
- [ ] Configuration file parsing
- [ ] CLI command implementations

## Contact for Questions

For questions about the workspace structure or architecture:
1. Review this document
2. Check inline code documentation
3. Refer to the main ARCHITECTURE.md
4. Consult IMPLEMENTATION_ROADMAP.md

---

**Document Version**: 1.0
**Last Updated**: 2025-11-04
**Architect**: RUST ARCHITECT Agent

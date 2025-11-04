# LLM Test Bench - CLI Command Structure

## Command Hierarchy

```
llm-test-bench [GLOBAL FLAGS] <COMMAND>
│
├── Global Flags
│   ├── -v, --verbose      Enable verbose output
│   ├── --no-color         Disable colored output
│   ├── -h, --help         Print help
│   └── -V, --version      Print version
│
├── test (alias: t) <PROVIDER> [OPTIONS]
│   ├── Required:
│   │   ├── PROVIDER           Provider name (openai, anthropic, local)
│   │   └── -p, --prompt      Prompt text
│   └── Optional:
│       ├── -m, --model       Model name
│       ├── -t, --temperature Temperature (0.0-2.0)
│       ├── --max-tokens      Maximum tokens
│       ├── -c, --config      Config file path
│       ├── --expect          Expected output
│       ├── -o, --output      Output format (json/text/detailed)
│       └── --save            Save results to file
│
├── bench (alias: b) [OPTIONS]
│   ├── Required:
│   │   ├── -d, --dataset     Dataset file path
│   │   └── -p, --providers   Comma-separated provider list
│   └── Optional:
│       ├── -c, --concurrency Number of concurrent requests
│       ├── -i, --iterations  Number of iterations
│       ├── --config          Config file path
│       ├── -o, --output      Output directory
│       ├── -f, --format      Output format (json/csv/html/markdown)
│       ├── --cache           Enable caching
│       ├── --timeout         Timeout in seconds
│       └── --continue-on-error Skip failed providers
│
├── eval (alias: e) [OPTIONS]
│   ├── Required:
│   │   └── -r, --results     Results file path
│   └── Optional:
│       ├── -m, --metrics     Metrics to compute
│       ├── -b, --baseline    Baseline file for comparison
│       ├── -o, --output      Output directory
│       ├── -f, --format      Report format
│       ├── --visualize       Generate charts
│       ├── --threshold       Success rate threshold
│       └── --export-metrics  Export detailed metrics file
│
├── config [SUBCOMMAND]
│   ├── init [OPTIONS]
│   │   ├── --provider            Specific provider to configure
│   │   └── --non-interactive     Skip interactive prompts
│   ├── show
│   └── validate [OPTIONS]
│       └── -c, --config          Config file to validate
│
├── completions <SHELL>
│   ├── bash
│   ├── zsh
│   ├── fish
│   ├── powershell
│   └── elvish
│
└── help [COMMAND]
    ├── test
    ├── bench
    ├── eval
    ├── config
    │   ├── init
    │   ├── show
    │   └── validate
    └── completions
```

## Data Flow Diagram

```
┌─────────────┐
│  User Input │
└──────┬──────┘
       │
       v
┌─────────────────┐
│  Clap Parser    │
│  - Validate args│
│  - Parse flags  │
└──────┬──────────┘
       │
       v
┌─────────────────┐
│ Command Router  │
│  (main.rs)      │
└──────┬──────────┘
       │
       ├────────────┬───────────┬──────────┬────────────┐
       │            │           │          │            │
       v            v           v          v            v
   ┌──────┐   ┌───────┐   ┌──────┐  ┌───────┐   ┌────────────┐
   │ test │   │ bench │   │ eval │  │config │   │completions │
   └──┬───┘   └───┬───┘   └──┬───┘  └───┬───┘   └─────┬──────┘
      │           │           │          │             │
      v           v           v          v             v
   Phase 2     Phase 3     Phase 4   ┌────────┐   ┌────────┐
   (stub)      (stub)      (stub)    │ ACTIVE │   │ ACTIVE │
                                     └────────┘   └────────┘
```

## Configuration Flow

```
┌──────────────────────┐
│ llm-test-bench       │
│ config init          │
└──────────┬───────────┘
           │
           v
┌──────────────────────┐     ┌─────────────────┐
│ Interactive Wizard   │────>│ User Choices    │
│ - Provider selection │     │ - OpenAI        │
│ - API key setup      │     │ - Anthropic     │
│ - Model defaults     │     │ - Local models  │
│ - Parameters         │     └─────────────────┘
└──────────┬───────────┘
           │
           v
┌──────────────────────┐
│ Validation           │
│ - Check paths        │
│ - Validate values    │
│ - Confirm overwrites │
└──────────┬───────────┘
           │
           v
┌──────────────────────┐
│ Write Config File    │
│ ~/.config/llm-test-  │
│ bench/config.toml    │
└──────────┬───────────┘
           │
           v
┌──────────────────────┐
│ Success Output       │
│ - Show config path   │
│ - Show next steps    │
│ - Environment vars   │
└──────────────────────┘
```

## Command Execution Flow

```
┌─────────────────────────────────────────────────────┐
│                    main.rs                          │
│                                                     │
│  ┌──────────────────────────────────────────────┐ │
│  │ 1. Parse CLI arguments (Clap)                │ │
│  └──────────────────┬───────────────────────────┘ │
│                     │                              │
│  ┌──────────────────v───────────────────────────┐ │
│  │ 2. Initialize logging (tracing)              │ │
│  └──────────────────┬───────────────────────────┘ │
│                     │                              │
│  ┌──────────────────v───────────────────────────┐ │
│  │ 3. Handle global flags                       │ │
│  │    - Set color override                      │ │
│  │    - Set verbose mode                        │ │
│  └──────────────────┬───────────────────────────┘ │
│                     │                              │
│  ┌──────────────────v───────────────────────────┐ │
│  │ 4. Route to command handler                  │ │
│  │    - test::execute()                         │ │
│  │    - bench::execute()                        │ │
│  │    - eval::execute()                         │ │
│  │    - config::execute()                       │ │
│  │    - generate_completions()                  │ │
│  └──────────────────┬───────────────────────────┘ │
│                     │                              │
│  ┌──────────────────v───────────────────────────┐ │
│  │ 5. Execute command                           │ │
│  │    Returns: Result<()>                       │ │
│  └──────────────────┬───────────────────────────┘ │
│                     │                              │
│  ┌──────────────────v───────────────────────────┐ │
│  │ 6. Handle result                             │ │
│  │    Success: exit(0)                          │ │
│  │    Error: print error + exit(1)              │ │
│  └──────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────┘
```

## Error Handling Flow

```
Command Execution
       │
       v
  ┌─────────┐
  │ Result? │
  └────┬────┘
       │
   ┌───┴────┐
   │        │
   v        v
┌──────┐ ┌─────────┐
│  Ok  │ │  Err(e) │
└──┬───┘ └────┬────┘
   │          │
   v          v
┌──────┐  ┌──────────────────┐
│Exit 0│  │ Print Error      │
└──────┘  │ "Error: {}"      │
          └────┬─────────────┘
               │
               v
          ┌─────────────┐
          │  Verbose?   │
          └──────┬──────┘
                 │
            ┌────┴────┐
            │         │
            v         v
          ┌───┐   ┌──────────────┐
          │ No│   │ Yes: Print   │
          └─┬─┘   │ cause chain  │
            │     └──────┬───────┘
            │            │
            └────────────┘
                 │
                 v
            ┌─────────┐
            │ Exit 1  │
            └─────────┘
```

## Module Organization

```
cli/
├── src/
│   ├── main.rs
│   │   ├── Cli struct (Clap Parser)
│   │   ├── Commands enum (Subcommands)
│   │   ├── main() function
│   │   │   ├── Parse args
│   │   │   ├── Route commands
│   │   │   └── Handle errors
│   │   └── generate_completions()
│   │
│   └── commands/
│       ├── mod.rs (module declarations)
│       │
│       ├── config.rs
│       │   ├── ConfigCommands enum
│       │   ├── Config/Provider structs
│       │   ├── execute()
│       │   ├── init_config()
│       │   ├── show_config()
│       │   ├── validate_config()
│       │   └── tests
│       │
│       ├── test.rs
│       │   ├── TestArgs struct
│       │   ├── execute() [stub]
│       │   └── tests
│       │
│       ├── bench.rs
│       │   ├── BenchArgs struct
│       │   ├── execute() [stub]
│       │   └── tests
│       │
│       └── eval.rs
│           ├── EvalArgs struct
│           ├── execute() [stub]
│           └── tests
│
└── tests/
    └── integration/
        ├── main.rs
        └── cli_tests.rs (24 integration tests)
```

## Example Usage Patterns

### Pattern 1: Simple Test
```bash
llm-test-bench test openai --prompt "Hello" --model gpt-4
```

### Pattern 2: Benchmark with Options
```bash
llm-test-bench bench \
  --dataset ./data.json \
  --providers openai,anthropic \
  --concurrency 4 \
  --format html
```

### Pattern 3: Evaluation with Baseline
```bash
llm-test-bench eval \
  --results ./results.json \
  --baseline ./baseline.json \
  --metrics accuracy,latency \
  --visualize
```

### Pattern 4: Configuration Management
```bash
# Initialize
llm-test-bench config init --provider openai

# View
llm-test-bench config show --verbose

# Validate
llm-test-bench config validate
```

### Pattern 5: Using Aliases and Verbose
```bash
# Short form with verbose
llm-test-bench -v t openai -p "test" -m gpt-4

# Benchmark with no color (for CI)
llm-test-bench --no-color b -d data.json -p openai
```

## Feature Matrix

| Feature | Phase 1 | Phase 2 | Phase 3 | Phase 4 |
|---------|---------|---------|---------|---------|
| Argument Parsing | ✅ | ✅ | ✅ | ✅ |
| Config Management | ✅ | ✅ | ✅ | ✅ |
| Shell Completions | ✅ | ✅ | ✅ | ✅ |
| Help Documentation | ✅ | ✅ | ✅ | ✅ |
| Error Handling | ✅ | ✅ | ✅ | ✅ |
| Provider Integration | ⏳ | ✅ | ✅ | ✅ |
| Test Execution | ⏳ | ✅ | ✅ | ✅ |
| Benchmarking | ⏳ | ⏳ | ✅ | ✅ |
| Evaluation | ⏳ | ⏳ | ⏳ | ✅ |
| Streaming Output | ⏳ | ✅ | ✅ | ✅ |
| Progress Bars | ⏳ | ✅ | ✅ | ✅ |
| Caching | ⏳ | ✅ | ✅ | ✅ |
| Parallel Execution | ⏳ | ⏳ | ✅ | ✅ |
| Metrics Collection | ⏳ | ⏳ | ✅ | ✅ |
| Report Generation | ⏳ | ⏳ | ✅ | ✅ |
| Visualizations | ⏳ | ⏳ | ⏳ | ✅ |

Legend: ✅ Complete | ⏳ Planned

## Dependencies Graph

```
llm-test-bench (CLI)
│
├── clap 4.5 (CLI framework)
│   └── clap_complete 4.5 (completions)
│
├── inquire 0.7 (interactive prompts)
│   └── crossterm (terminal control)
│
├── anyhow 1.0 (error handling)
│
├── serde 1.0 (serialization)
│   └── toml 0.8 (config format)
│
├── dirs 5.0 (config paths)
│
├── tokio 1.40 (async runtime)
│   └── tracing (logging)
│
└── [test dependencies]
    ├── assert_cmd 2.0
    ├── predicates 3.0
    └── tempfile 3.8
```

---

**Legend:**
- ✅ Implemented and tested
- ⏳ Planned for future phases
- 📋 Documentation complete
- 🧪 Tests passing

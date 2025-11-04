# Configuration System Documentation

## Overview

The LLM Test Bench configuration system provides a flexible, hierarchical approach to managing application settings with proper override precedence, validation, and environment variable support.

## Architecture

### Configuration Hierarchy

Configuration values are loaded and merged in the following order (highest to lowest priority):

1. **CLI Arguments** (highest priority) - _Future implementation_
2. **Environment Variables** (prefixed with `LLM_TEST_BENCH_`)
3. **Config Files** (`~/.config/llm-test-bench/config.toml`)
4. **Defaults** (lowest priority)

### Key Design Principles

- **Type Safety**: All configuration is statically typed using Rust structs with serde
- **Validation**: Schema validation using `serde_valid` ensures all values meet constraints
- **Platform Agnostic**: Automatically uses platform-specific config directories
- **Explicit Defaults**: Clear default values for all configuration options

## Configuration Schema

### Root Configuration

```rust
pub struct Config {
    pub providers: HashMap<String, ProviderConfig>,
    pub benchmarks: BenchmarkConfig,
    pub evaluation: EvaluationConfig,
    pub global_timeout_seconds: Option<u64>,
}
```

### Provider Configuration

Each LLM provider (OpenAI, Anthropic, etc.) has its own configuration:

```rust
pub struct ProviderConfig {
    pub api_key_env: String,          // Environment variable name for API key
    pub base_url: String,              // API base URL
    pub default_model: String,         // Default model to use
    pub timeout_seconds: u64,          // Request timeout (1-300)
    pub max_retries: u32,              // Max retry attempts (0-10)
    pub rate_limit_rpm: Option<u32>,   // Optional rate limit
}
```

**Default Providers:**
- `openai`: OpenAI API configuration
- `anthropic`: Anthropic Claude API configuration

### Benchmark Configuration

```rust
pub struct BenchmarkConfig {
    pub output_dir: PathBuf,           // Results output directory
    pub save_responses: bool,          // Save full LLM responses
    pub parallel_requests: usize,      // Concurrent requests (1-100)
    pub continue_on_failure: bool,     // Continue after failures
    pub random_seed: Option<u64>,      // Reproducibility seed
}
```

### Evaluation Configuration

```rust
pub struct EvaluationConfig {
    pub metrics: Vec<String>,          // Metrics to compute
    pub llm_judge_model: String,       // Model for LLM-as-judge
    pub llm_judge_provider: Option<String>,  // Provider for judge
    pub confidence_threshold: f64,     // Pass threshold (0.0-1.0)
    pub include_explanations: bool,    // Include detailed explanations
}
```

**Available Metrics:**
- `perplexity`: Language model prediction quality
- `faithfulness`: Factual accuracy and hallucination detection
- `relevance`: Task/prompt alignment scoring
- `coherence`: Output fluency and logical consistency
- `latency`: Response time measurement
- `token_efficiency`: Token usage analysis

## Usage Examples

### Loading Configuration

```rust
use llm_test_bench_core::config::ConfigLoader;

// Load from all sources (default)
let config = ConfigLoader::new().load()?;

// Load from specific file
let config = ConfigLoader::new()
    .with_file("/path/to/config.toml")
    .load()?;

// Load without environment variables (testing)
let config = ConfigLoader::new()
    .skip_env()
    .load()?;

// Load without default config file
let config = ConfigLoader::new()
    .skip_default_file()
    .load()?;
```

### Accessing Configuration

```rust
// Access provider configuration
if let Some(openai) = config.providers.get("openai") {
    println!("OpenAI model: {}", openai.default_model);
    println!("API key env: {}", openai.api_key_env);
}

// Access benchmark settings
println!("Parallel requests: {}", config.benchmarks.parallel_requests);
println!("Output dir: {}", config.benchmarks.output_dir.display());

// Access evaluation settings
for metric in &config.evaluation.metrics {
    println!("Metric: {}", metric);
}
```

### Initializing Configuration File

```rust
use llm_test_bench_core::config::init_config_file;

// Create default config file at standard location
let config_path = init_config_file()?;
println!("Created config at: {}", config_path.display());
```

## Environment Variable Mapping

### Naming Convention

Environment variables use the prefix `LLM_TEST_BENCH_` followed by the configuration path with double underscores (`__`) for nesting.

### Examples

#### Provider Settings

```bash
# OpenAI provider
export LLM_TEST_BENCH_PROVIDERS__OPENAI__DEFAULT_MODEL="gpt-4"
export LLM_TEST_BENCH_PROVIDERS__OPENAI__TIMEOUT_SECONDS=60
export LLM_TEST_BENCH_PROVIDERS__OPENAI__MAX_RETRIES=5

# Anthropic provider
export LLM_TEST_BENCH_PROVIDERS__ANTHROPIC__DEFAULT_MODEL="claude-3-opus"
export LLM_TEST_BENCH_PROVIDERS__ANTHROPIC__TIMEOUT_SECONDS=120
```

#### Benchmark Settings

```bash
export LLM_TEST_BENCH_BENCHMARKS__PARALLEL_REQUESTS=10
export LLM_TEST_BENCH_BENCHMARKS__SAVE_RESPONSES=false
export LLM_TEST_BENCH_BENCHMARKS__OUTPUT_DIR="/tmp/bench-results"
export LLM_TEST_BENCH_BENCHMARKS__CONTINUE_ON_FAILURE=true
```

#### Evaluation Settings

```bash
export LLM_TEST_BENCH_EVALUATION__METRICS="latency,faithfulness,relevance"
export LLM_TEST_BENCH_EVALUATION__LLM_JUDGE_MODEL="claude-3-opus"
export LLM_TEST_BENCH_EVALUATION__CONFIDENCE_THRESHOLD=0.8
export LLM_TEST_BENCH_EVALUATION__INCLUDE_EXPLANATIONS=false
```

#### Global Settings

```bash
export LLM_TEST_BENCH_GLOBAL_TIMEOUT_SECONDS=90
```

## Configuration File Format

### Complete Example (TOML)

See [`config.example.toml`](../config.example.toml) for a fully documented configuration file template.

### Minimal Example

```toml
[providers.openai]
api_key_env = "OPENAI_API_KEY"
base_url = "https://api.openai.com/v1"
default_model = "gpt-4-turbo"
timeout_seconds = 30
max_retries = 3

[benchmarks]
output_dir = "./results"
parallel_requests = 5

[evaluation]
metrics = ["latency", "relevance"]
llm_judge_model = "gpt-4"
```

## Platform-Specific Config Locations

The configuration file is automatically searched in platform-specific locations:

- **Linux**: `~/.config/llm-test-bench/config.toml`
- **macOS**: `~/Library/Application Support/llm-test-bench/config.toml`
- **Windows**: `%APPDATA%\llm-test-bench\config.toml`
- **Fallback**: `./config.toml` (current directory)

### Finding Config Directory

```rust
use llm_test_bench_core::config::ConfigLoader;

if let Some(dir) = ConfigLoader::default_config_dir() {
    println!("Config directory: {}", dir.display());
}

if let Some(path) = ConfigLoader::default_config_path() {
    println!("Config file path: {}", path.display());
}
```

## Validation

All configuration values are validated according to these rules:

### Provider Validation
- `api_key_env`: Must be non-empty string
- `base_url`: Must be non-empty string
- `default_model`: Must be non-empty string
- `timeout_seconds`: Must be between 1 and 300
- `max_retries`: Must be ≤ 10

### Benchmark Validation
- `parallel_requests`: Must be between 1 and 100

### Evaluation Validation
- `metrics`: Must contain at least one metric
- `llm_judge_model`: Must be non-empty string
- `confidence_threshold`: Must be between 0.0 and 1.0

### Validation Errors

When validation fails, detailed error messages are provided:

```
Configuration validation failed: timeout_seconds: Maximum value of 300 was exceeded.
```

## Testing

The configuration system includes comprehensive unit tests covering:

1. **Default Configuration**: Verifies defaults are valid and complete
2. **Custom File Loading**: Tests loading from custom configuration files
3. **Environment Variable Overrides**: Tests environment variable precedence
4. **Validation**: Tests constraint validation (min/max values, required fields)
5. **Precedence**: Tests that environment variables override file settings
6. **Serialization**: Tests TOML round-trip serialization

### Running Tests

```bash
cargo test --package llm-test-bench-core --lib config
```

### Test Coverage

- 18 unit tests covering all major functionality
- 12 tests passing (core functionality)
- 6 tests with known limitations (see below)

## Known Limitations

### Environment Variable Support

The current implementation has limitations with nested environment variable overrides due to the `config` crate's handling of complex nested structures. Specifically:

1. **Nested HashMap Overrides**: Environment variables for nested HashMap entries (like `providers.openai.default_model`) may not override correctly in all cases.

2. **List Values**: While basic list support exists, complex list parsing from environment variables has limitations.

**Workaround**: Use configuration files for complex nested settings, and reserve environment variables for simple top-level overrides.

**Future Enhancement**: Consider implementing custom environment variable parsing or upgrading the `config` crate to a version with better nested support.

### Recommended Usage Patterns

1. **Use configuration files** for complex, structured configuration (providers, multiple metrics)
2. **Use environment variables** for simple overrides (timeouts, parallel_requests, output directories)
3. **Use defaults** as a sensible baseline that works out of the box

## API Reference

### ConfigLoader

Builder for loading configuration from multiple sources.

```rust
impl ConfigLoader {
    pub fn new() -> Self
    pub fn with_file<P: AsRef<Path>>(self, path: P) -> Self
    pub fn skip_default_file(self) -> Self
    pub fn skip_env(self) -> Self
    pub fn load(&self) -> Result<Config>
    pub fn default_config_dir() -> Option<PathBuf>
    pub fn default_config_path() -> Option<PathBuf>
}
```

### Initialization

```rust
pub fn init_config_file() -> Result<PathBuf>
```

Creates a default configuration file at the platform-specific location.

### Configuration Structs

- `Config`: Root configuration
- `ProviderConfig`: Per-provider settings
- `BenchmarkConfig`: Benchmark execution settings
- `EvaluationConfig`: Evaluation metrics settings

## Migration Guide

### From Defaults to Custom Configuration

1. Generate example configuration:
   ```bash
   cp config.example.toml ~/.config/llm-test-bench/config.toml
   ```

2. Edit the file to customize settings

3. Set API keys:
   ```bash
   export OPENAI_API_KEY="sk-..."
   export ANTHROPIC_API_KEY="sk-ant-..."
   ```

4. Application will automatically load custom configuration

### Adding New Providers

1. Add provider entry to config file:
   ```toml
   [providers.custom]
   api_key_env = "CUSTOM_API_KEY"
   base_url = "https://api.custom.com/v1"
   default_model = "custom-model"
   timeout_seconds = 30
   max_retries = 3
   ```

2. Set the API key:
   ```bash
   export CUSTOM_API_KEY="your-key"
   ```

3. Provider will be available in the application

## Best Practices

1. **Never commit API keys**: Always use environment variables for secrets
2. **Use version control for config files**: Check in `config.example.toml`, not `config.toml`
3. **Document custom settings**: Add comments explaining why custom values are needed
4. **Test configuration**: Use `ConfigLoader::new().load()` to validate before deployment
5. **Use appropriate timeouts**: Balance between responsiveness and allowing large requests
6. **Monitor rate limits**: Set `rate_limit_rpm` to avoid hitting provider limits

## Troubleshooting

### Configuration Not Loading

1. Check file exists at expected location:
   ```rust
   if let Some(path) = ConfigLoader::default_config_path() {
       println!("Looking for config at: {}", path.display());
   }
   ```

2. Verify TOML syntax is valid:
   ```bash
   # Test parsing
   cargo test --package llm-test-bench-core --lib config::tests::test_load_from_custom_file
   ```

3. Check validation errors in output

### Environment Variables Not Working

1. Verify prefix and separator:
   - Must start with `LLM_TEST_BENCH_`
   - Use `__` (double underscore) for nesting

2. Check variable is set:
   ```bash
   env | grep LLM_TEST_BENCH
   ```

3. Remember current limitations with nested HashMap overrides

### Validation Failures

Review constraint messages and adjust values to meet requirements:
- Timeouts: 1-300 seconds
- Retries: 0-10 attempts
- Parallel requests: 1-100
- Confidence threshold: 0.0-1.0

## Future Enhancements

1. **CLI Argument Integration**: Support for command-line argument overrides (Phase 1, Milestone 1.3)
2. **Configuration Profiles**: Support for multiple configuration profiles (dev, staging, prod)
3. **Hot Reloading**: Watch configuration file for changes and reload automatically
4. **Schema Export**: Generate JSON Schema for IDE autocompletion
5. **Improved Environment Variables**: Better support for complex nested structures
6. **Configuration Validation Tool**: Standalone tool to validate configuration files
7. **Configuration Migration**: Automatic migration between configuration versions

## References

- [config crate documentation](https://docs.rs/config/)
- [serde documentation](https://serde.rs/)
- [serde_valid documentation](https://docs.rs/serde_valid/)
- [TOML specification](https://toml.io/)

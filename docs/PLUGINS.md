# Plugin System Guide

**Enterprise-grade WASM-based plugin system for extensibility**

## Table of Contents

1. [Overview](#overview)
2. [Architecture](#architecture)
3. [Quick Start](#quick-start)
4. [Plugin Types](#plugin-types)
5. [Creating Plugins](#creating-plugins)
6. [Security & Sandboxing](#security--sandboxing)
7. [API Reference](#api-reference)
8. [Best Practices](#best-practices)
9. [Troubleshooting](#troubleshooting)

---

## Overview

The LLM Test Bench plugin system enables secure, high-performance extensibility through WebAssembly (WASM). Plugins can add custom evaluation metrics, provider integrations, data transformations, and more.

### Key Features

- **🔒 Secure**: Sandboxed execution with resource limits
- **⚡ Fast**: Native-speed execution via WASM
- **🌐 Portable**: Write once, run anywhere
- **🛡️ Isolated**: Plugins cannot interfere with each other
- **📦 Lightweight**: Small WASM modules (typically <1MB)
- **🔄 Hot-reloadable**: Load/unload plugins at runtime

### Why WASM?

- **Security**: Sandboxed by design, no direct system access
- **Performance**: Near-native execution speed
- **Portability**: Works across platforms and architectures
- **Size**: Compact binaries optimized for distribution
- **Standard**: W3C standard with broad ecosystem support

---

## Architecture

```text
┌─────────────────────────────────────────────────────────────┐
│                    LLM Test Bench Host                      │
│                                                             │
│  ┌───────────────────────────────────────────────────────┐ │
│  │              Plugin Manager                           │ │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  │ │
│  │  │   Plugin    │  │   Plugin    │  │   Plugin    │  │ │
│  │  │  Registry   │  │   Loader    │  │   Sandbox   │  │ │
│  │  └─────────────┘  └─────────────┘  └─────────────┘  │ │
│  └───────────────────────────────────────────────────────┘ │
│                            │                                │
│                            ▼                                │
│  ┌───────────────────────────────────────────────────────┐ │
│  │              WASM Runtime (wasmtime)                  │ │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  │ │
│  │  │   Plugin A  │  │   Plugin B  │  │   Plugin C  │  │ │
│  │  │  (sandboxed)│  │  (sandboxed)│  │  (sandboxed)│  │ │
│  │  └─────────────┘  └─────────────┘  └─────────────┘  │ │
│  └───────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

### Components

1. **Plugin System**: High-level coordinator
2. **Plugin Manager**: Lifecycle management (load/execute/unload)
3. **Plugin Registry**: Discovery and indexing
4. **Plugin Loader**: File loading and validation
5. **Plugin Sandbox**: Security enforcement
6. **WASM Runtime**: wasmtime-based execution environment
7. **Host Functions**: API exposed to plugins

---

## Quick Start

### Loading a Plugin

```rust
use llm_test_bench_core::plugins::{PluginSystem, PluginInput};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create plugin system
    let plugin_system = PluginSystem::new()?;

    // Load plugin
    let plugin_id = plugin_system
        .load_plugin("plugins/my_evaluator.wasm")
        .await?;

    // Create input
    let input = PluginInput {
        data: serde_json::json!({
            "text": "Hello, world!",
            "expected": "greeting"
        }),
        context: Default::default(),
    };

    // Execute plugin
    let output = plugin_system.execute_plugin(&plugin_id, input).await?;
    println!("Score: {}", output.data["score"]);

    // Unload when done
    plugin_system.unload_plugin(&plugin_id).await?;

    Ok(())
}
```

### Custom Configuration

```rust
use llm_test_bench_core::plugins::{
    PluginSystem, ManagerConfig, RegistryConfig, RuntimeConfig, ResourceLimits
};

// Configure resource limits
let limits = ResourceLimits {
    max_memory_bytes: 128 * 1024 * 1024,  // 128 MB
    max_execution_time_ms: 60_000,         // 60 seconds
    max_instructions: Some(5_000_000_000), // 5 billion instructions
};

// Configure runtime
let runtime_config = RuntimeConfig {
    limits,
    enable_wasi: true,
    ..Default::default()
};

// Configure manager
let manager_config = ManagerConfig {
    runtime_config,
    max_concurrent_plugins: 50,
    cache_dir: Some("./plugin_cache".into()),
};

// Create system with configuration
let plugin_system = PluginSystem::with_config(
    manager_config,
    RegistryConfig::default()
)?;
```

---

## Plugin Types

### 1. Evaluator Plugins

Custom evaluation metrics for LLM outputs.

**Use Cases**:
- Domain-specific scoring
- Custom relevance metrics
- Specialized quality checks
- Multi-criteria evaluation

**Example**: Sentiment accuracy evaluator

### 2. Provider Plugins

Custom LLM provider integrations.

**Use Cases**:
- Internal API wrappers
- Custom model endpoints
- Proprietary LLM services
- Research model access

**Example**: Internal company LLM wrapper

### 3. Transform Plugins

Data transformation and preprocessing.

**Use Cases**:
- Custom tokenization
- Data normalization
- Format conversion
- Feature extraction

**Example**: Domain-specific tokenizer

### 4. Filter Plugins

Result filtering and post-processing.

**Use Cases**:
- Content filtering
- Quality thresholding
- Deduplication
- Ranking adjustment

**Example**: Profanity filter

---

## Creating Plugins

### Project Setup

```bash
# Create new plugin project
cargo new --lib my_plugin
cd my_plugin

# Edit Cargo.toml
```

```toml
[package]
name = "my_plugin"
version = "1.0.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

[profile.release]
opt-level = "z"     # Optimize for size
lto = true          # Link-time optimization
codegen-units = 1   # Better optimization
strip = true        # Strip symbols
```

### Required Exports

Every plugin must export these functions:

```rust
// 1. Metadata - describe the plugin
#[no_mangle]
pub extern "C" fn plugin_metadata(
    output_ptr: *mut u8,
    output_len: *mut usize
) -> i32;

// 2. Initialize - setup plugin state
#[no_mangle]
pub extern "C" fn plugin_init(
    config_ptr: *const u8,
    config_len: usize
) -> i32;

// 3. Execute - main plugin logic
#[no_mangle]
pub extern "C" fn plugin_execute(
    input_ptr: *const u8,
    input_len: usize,
    output_ptr: *mut u8,
    output_len: *mut usize
) -> i32;

// 4. Shutdown - cleanup
#[no_mangle]
pub extern "C" fn plugin_shutdown() -> i32;

// 5. Memory management
#[no_mangle]
pub extern "C" fn plugin_alloc(size: usize) -> *mut u8;

#[no_mangle]
pub extern "C" fn plugin_free(ptr: *mut u8, size: usize);
```

### Complete Example

See `examples/plugin_example.md` for a full working example.

### Building

```bash
# Install WASM target
rustup target add wasm32-unknown-unknown

# Build
cargo build --release --target wasm32-unknown-unknown

# Output: target/wasm32-unknown-unknown/release/my_plugin.wasm

# Optimize (optional)
wasm-opt -Oz input.wasm -o output.wasm
```

---

## Security & Sandboxing

### Resource Limits

All plugins run with enforced limits:

```rust
ResourceLimits {
    max_memory_bytes: 64 * 1024 * 1024,    // 64 MB default
    max_execution_time_ms: 30_000,         // 30 seconds default
    max_instructions: Some(1_000_000_000), // 1 billion instructions
}
```

### Permissions

Plugins have no permissions by default:

```rust
PluginPermissions {
    filesystem: false,          // No file access
    network: false,             // No network access
    env_vars: false,            // No environment variables
    allowed_dirs: vec![],       // No directories accessible
    allowed_hosts: vec![],      // No hosts accessible
}
```

### Granting Permissions

```rust
let mut permissions = PluginPermissions::default();

// Grant filesystem access to specific directory
permissions.filesystem = true;
permissions.allowed_dirs = vec![
    "/tmp/plugin_data".to_string(),
];

// Grant network access to specific hosts
permissions.network = true;
permissions.allowed_hosts = vec![
    "api.example.com".to_string(),
];
```

### Sandbox Enforcement

- Memory limits enforced by WASM runtime
- CPU limits enforced via instruction counting
- Filesystem access controlled via WASI
- Network access controlled via host functions
- No syscalls available to plugins

---

## API Reference

### Plugin Metadata

```rust
struct PluginMetadata {
    name: String,              // Plugin name
    version: String,           // Semver version
    description: String,       // What it does
    author: String,            // Author name
    plugin_type: PluginType,   // evaluator | provider | transform | filter
    capabilities: Vec<String>, // What it can do
    api_version: String,       // Plugin API version
}
```

### Plugin Input/Output

```rust
struct PluginInput {
    data: serde_json::Value,   // Input data
    context: PluginContext,     // Execution context
}

struct PluginOutput {
    data: serde_json::Value,   // Output data
    metadata: OutputMetadata,   // Execution metadata
}

struct OutputMetadata {
    execution_time_ms: u64,
    memory_used_bytes: Option<usize>,
    metrics: HashMap<String, f64>,
}
```

### Host Functions

Functions available to plugins:

```rust
// Logging
host_log(level: i32, message_ptr: u32, message_len: u32) -> i32

// Time
host_current_time_ms() -> i64

// Random
host_random(max: u32) -> u32

// State
host_set_state(key_ptr: u32, key_len: u32, value_ptr: u32, value_len: u32) -> i32
host_get_state(key_ptr: u32, key_len: u32, value_ptr: u32, value_max_len: u32) -> i32
```

### Result Codes

```rust
const RESULT_OK: i32 = 0;
const RESULT_ERROR: i32 = -1;
const RESULT_INVALID_INPUT: i32 = -2;
const RESULT_TIMEOUT: i32 = -3;
const RESULT_OUT_OF_MEMORY: i32 = -4;
```

---

## Best Practices

### 1. Optimize for Size

```toml
[profile.release]
opt-level = "z"       # Optimize for size
lto = true            # Enable LTO
codegen-units = 1     # Single codegen unit
strip = true          # Strip symbols
panic = "abort"       # Smaller panic handler
```

### 2. Validate Input

```rust
pub extern "C" fn plugin_execute(...) -> i32 {
    // Always validate input
    let input: MyInput = match serde_json::from_slice(input_slice) {
        Ok(i) => i,
        Err(_) => return RESULT_INVALID_INPUT,
    };

    // Check input constraints
    if input.text.len() > MAX_TEXT_LENGTH {
        return RESULT_ERROR;
    }

    // ... process input ...
}
```

### 3. Handle Errors Gracefully

```rust
// Don't panic - return error codes
match risky_operation() {
    Ok(result) => result,
    Err(_) => return RESULT_ERROR,
}
```

### 4. Limit Memory Usage

```rust
// Avoid large allocations
const MAX_BUFFER_SIZE: usize = 10 * 1024 * 1024; // 10 MB

if needed_size > MAX_BUFFER_SIZE {
    return RESULT_OUT_OF_MEMORY;
}
```

### 5. Use Appropriate Data Structures

```rust
// Prefer stack over heap when possible
let mut buffer = [0u8; 1024];  // Stack
// vs
let mut buffer = vec![0u8; 1024];  // Heap
```

### 6. Version Properly

```toml
[package]
version = "1.2.3"  # Follow semver

# In metadata
api_version = "1.0.0"  # API compatibility version
```

### 7. Document Thoroughly

```rust
/// Evaluates text sentiment
///
/// # Input
/// - `text`: Text to analyze
/// - `expected`: Expected sentiment (optional)
///
/// # Output
/// - `score`: Sentiment score (0.0 to 1.0)
/// - `sentiment`: "positive" | "negative" | "neutral"
pub extern "C" fn plugin_execute(...) -> i32 {
    // ...
}
```

---

## Troubleshooting

### Plugin Won't Load

**Problem**: `Failed to load WASM module`

**Solutions**:
```bash
# 1. Check it's valid WASM
file my_plugin.wasm
# Should show: WebAssembly (wasm) binary module version 0x1 (MVP)

# 2. Inspect exports
wasm-objdump -x my_plugin.wasm | grep "export"

# 3. Check for required functions
wasm-objdump -x my_plugin.wasm | grep -E "plugin_(init|execute|metadata)"

# 4. Verify no unsupported features
wasm-validate my_plugin.wasm
```

### Plugin Execution Fails

**Problem**: `Plugin execution failed with code: -1`

**Debug**:
```rust
// Add logging in plugin
extern "C" {
    fn host_log(level: i32, ptr: *const u8, len: usize) -> i32;
}

fn log_debug(msg: &str) {
    unsafe {
        host_log(1, msg.as_ptr(), msg.len());
    }
}

pub extern "C" fn plugin_execute(...) -> i32 {
    log_debug("Starting execution");
    // ... your code ...
    log_debug("Execution complete");
    RESULT_OK
}
```

### Memory Issues

**Problem**: `Out of memory` or crashes

**Solutions**:
```rust
// 1. Check allocation size
if size > 1024 * 1024 {  // > 1MB
    return RESULT_OUT_OF_MEMORY;
}

// 2. Reuse buffers
static mut BUFFER: [u8; 4096] = [0; 4096];

// 3. Monitor memory usage
let layout = Layout::from_size_align(size, 8).unwrap();
let ptr = unsafe { alloc(layout) };
if ptr.is_null() {
    return RESULT_OUT_OF_MEMORY;
}
```

### Timeout Issues

**Problem**: `Plugin timeout after 30000ms`

**Solutions**:
```rust
// 1. Increase timeout
let limits = ResourceLimits {
    max_execution_time_ms: 60_000,  // 60 seconds
    ..Default::default()
};

// 2. Optimize plugin code
// - Reduce loop iterations
// - Use efficient algorithms
// - Cache expensive computations

// 3. Process in chunks
for chunk in data.chunks(1000) {
    process_chunk(chunk);
    // Allows timeout check between chunks
}
```

---

## Performance Tips

### 1. Minimize Allocations

```rust
// Bad: Many allocations
for item in items {
    let s = format!("Item: {}", item);  // Allocation per iteration
    process(&s);
}

// Good: Reuse buffer
let mut buffer = String::new();
for item in items {
    buffer.clear();
    write!(&mut buffer, "Item: {}", item).unwrap();
    process(&buffer);
}
```

### 2. Use &str Over String

```rust
// Prefer borrows
fn process(text: &str) { /* ... */ }

// Over owned strings
fn process(text: String) { /* ... */ }
```

### 3. Pre-allocate When Possible

```rust
// Allocate once
let mut results = Vec::with_capacity(input.len());

for item in input {
    results.push(process(item));
}
```

### 4. Optimize Serialization

```rust
// Consider using bincode instead of JSON for large data
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct Data {
    // ... fields ...
}

// Smaller and faster than JSON
let bytes = bincode::serialize(&data)?;
```

---

## Security Checklist

Before deploying plugins in production:

- [ ] Review plugin source code
- [ ] Verify plugin author/signature
- [ ] Test with malicious inputs
- [ ] Set appropriate resource limits
- [ ] Grant minimal permissions
- [ ] Monitor plugin behavior
- [ ] Implement rate limiting
- [ ] Log all plugin executions
- [ ] Have rollback plan
- [ ] Test timeout scenarios
- [ ] Verify memory limits work
- [ ] Check for side effects
- [ ] Audit host function usage
- [ ] Test concurrent execution
- [ ] Validate all outputs

---

## Production Deployment

### Docker Example

```dockerfile
FROM rust:latest as builder
WORKDIR /app

# Copy plugin source
COPY plugins/ ./plugins/

# Build plugins
RUN rustup target add wasm32-unknown-unknown
RUN cd plugins/evaluator && cargo build --release --target wasm32-unknown-unknown

FROM debian:bookworm-slim
WORKDIR /app

# Copy compiled plugins
COPY --from=builder /app/plugins/*/target/wasm32-unknown-unknown/release/*.wasm ./plugins/

# Copy application
COPY --from=builder /app/target/release/llm-test-bench .

CMD ["./llm-test-bench"]
```

### Kubernetes Example

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: plugins
data:
  evaluator.wasm: |
    # Base64-encoded plugin
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: llm-test-bench
spec:
  template:
    spec:
      containers:
      - name: app
        image: llm-test-bench:latest
        volumeMounts:
        - name: plugins
          mountPath: /app/plugins
      volumes:
      - name: plugins
        configMap:
          name: plugins
```

---

## Summary

The LLM Test Bench plugin system provides:

✅ **Secure**: Sandboxed WASM execution
✅ **Fast**: Near-native performance
✅ **Portable**: Write once, run anywhere
✅ **Flexible**: Multiple plugin types
✅ **Safe**: Resource limits enforced
✅ **Enterprise-Ready**: Production-tested

**Next Steps**:
1. [Create your first plugin](examples/plugin_example.md)
2. [Review security guide](SECURITY.md)
3. [Explore advanced features](ADVANCED_PLUGINS.md)
4. [Join the community](https://github.com/llm-test-bench/plugins)

For more information, see the [API documentation](https://docs.rs/llm-test-bench-core).

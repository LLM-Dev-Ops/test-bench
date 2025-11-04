# Plugin Example

This document shows how to create a custom evaluator plugin for LLM Test Bench.

## Plugin Structure

A plugin is a WebAssembly (WASM) module that exports specific functions and follows the plugin API.

### Required Exports

```rust
// Plugin metadata
#[no_mangle]
pub extern "C" fn plugin_metadata(output_ptr: *mut u8, output_len: *mut usize) -> i32;

// Plugin initialization
#[no_mangle]
pub extern "C" fn plugin_init(config_ptr: *const u8, config_len: usize) -> i32;

// Plugin execution
#[no_mangle]
pub extern "C" fn plugin_execute(
    input_ptr: *const u8,
    input_len: usize,
    output_ptr: *mut u8,
    output_len: *mut usize
) -> i32;

// Plugin shutdown
#[no_mangle]
pub extern "C" fn plugin_shutdown() -> i32;

// Memory allocation (required by host)
#[no_mangle]
pub extern "C" fn plugin_alloc(size: usize) -> *mut u8;

// Memory deallocation (required by host)
#[no_mangle]
pub extern "C" fn plugin_free(ptr: *mut u8, size: usize);
```

## Example: Custom Evaluator Plugin

### Cargo.toml

```toml
[package]
name = "my-evaluator-plugin"
version = "1.0.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

[profile.release]
opt-level = "z"       # Optimize for size
lto = true            # Enable link-time optimization
codegen-units = 1     # Better optimization
strip = true          # Strip symbols
```

### src/lib.rs

```rust
use std::alloc::{alloc, dealloc, Layout};
use std::slice;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct PluginMetadata {
    name: String,
    version: String,
    description: String,
    author: String,
    plugin_type: String,
    capabilities: Vec<String>,
    api_version: String,
}

#[derive(Serialize, Deserialize)]
struct EvaluationInput {
    input: String,
    output: String,
    expected: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct EvaluationOutput {
    score: f64,
    explanation: String,
}

// Plugin metadata
#[no_mangle]
pub extern "C" fn plugin_metadata(output_ptr: *mut u8, output_len: *mut usize) -> i32 {
    let metadata = PluginMetadata {
        name: "my-evaluator".to_string(),
        version: "1.0.0".to_string(),
        description: "Custom evaluation metric".to_string(),
        author: "Your Name".to_string(),
        plugin_type: "evaluator".to_string(),
        capabilities: vec!["text_evaluation".to_string()],
        api_version: "1.0.0".to_string(),
    };

    match serde_json::to_vec(&metadata) {
        Ok(json) => {
            unsafe {
                std::ptr::copy_nonoverlapping(json.as_ptr(), output_ptr, json.len());
                *output_len = json.len();
            }
            0 // Success
        }
        Err(_) => -1, // Error
    }
}

// Plugin initialization
#[no_mangle]
pub extern "C" fn plugin_init(_config_ptr: *const u8, _config_len: usize) -> i32 {
    // Initialize plugin state if needed
    0 // Success
}

// Plugin execution
#[no_mangle]
pub extern "C" fn plugin_execute(
    input_ptr: *const u8,
    input_len: usize,
    output_ptr: *mut u8,
    output_len: *mut usize,
) -> i32 {
    // Read input
    let input_slice = unsafe { slice::from_raw_parts(input_ptr, input_len) };

    let input: EvaluationInput = match serde_json::from_slice(input_slice) {
        Ok(i) => i,
        Err(_) => return -2, // Invalid input
    };

    // Perform evaluation (simple example: check if output contains expected)
    let score = if let Some(expected) = &input.expected {
        if input.output.contains(expected) {
            1.0
        } else {
            let similarity = calculate_similarity(&input.output, expected);
            similarity
        }
    } else {
        0.5 // No expected output provided
    };

    let output = EvaluationOutput {
        score,
        explanation: format!("Evaluated with score: {:.2}", score),
    };

    // Write output
    match serde_json::to_vec(&output) {
        Ok(json) => {
            unsafe {
                std::ptr::copy_nonoverlapping(json.as_ptr(), output_ptr, json.len());
                *output_len = json.len();
            }
            0 // Success
        }
        Err(_) => -1, // Error
    }
}

// Plugin shutdown
#[no_mangle]
pub extern "C" fn plugin_shutdown() -> i32 {
    // Cleanup if needed
    0 // Success
}

// Memory allocation
#[no_mangle]
pub extern "C" fn plugin_alloc(size: usize) -> *mut u8 {
    let layout = Layout::from_size_align(size, 8).unwrap();
    unsafe { alloc(layout) }
}

// Memory deallocation
#[no_mangle]
pub extern "C" fn plugin_free(ptr: *mut u8, size: usize) {
    let layout = Layout::from_size_align(size, 8).unwrap();
    unsafe { dealloc(ptr, layout) }
}

// Helper function: Calculate simple similarity
fn calculate_similarity(a: &str, b: &str) -> f64 {
    let a_words: Vec<&str> = a.split_whitespace().collect();
    let b_words: Vec<&str> = b.split_whitespace().collect();

    let mut common = 0;
    for word in &a_words {
        if b_words.contains(word) {
            common += 1;
        }
    }

    if a_words.is_empty() {
        0.0
    } else {
        common as f64 / a_words.len() as f64
    }
}
```

## Building the Plugin

```bash
# Install wasm target
rustup target add wasm32-unknown-unknown

# Build plugin
cargo build --release --target wasm32-unknown-unknown

# The plugin will be at:
# target/wasm32-unknown-unknown/release/my_evaluator_plugin.wasm
```

## Using the Plugin

```rust
use llm_test_bench_core::plugins::{PluginSystem, PluginInput};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create plugin system
    let plugin_system = PluginSystem::new()?;

    // Load the plugin
    let plugin_id = plugin_system
        .load_plugin("target/wasm32-unknown-unknown/release/my_evaluator_plugin.wasm")
        .await?;

    // Create input
    let input = PluginInput {
        data: serde_json::json!({
            "input": "What is 2+2?",
            "output": "The answer is 4",
            "expected": "4"
        }),
        context: Default::default(),
    };

    // Execute plugin
    let output = plugin_system.execute_plugin(&plugin_id, input).await?;

    println!("Result: {:?}", output);

    // Unload plugin
    plugin_system.unload_plugin(&plugin_id).await?;

    Ok(())
}
```

## Best Practices

1. **Keep plugins small**: Optimize for size with `opt-level = "z"`
2. **Handle errors gracefully**: Return appropriate error codes
3. **Validate input**: Always check input before processing
4. **Limit memory usage**: Be mindful of allocations
5. **Use sandboxing**: Run with restricted permissions
6. **Test thoroughly**: Test with various inputs
7. **Version properly**: Use semver for plugin versions
8. **Document capabilities**: Clearly specify what your plugin does

## Security Considerations

- Plugins run in a sandboxed WebAssembly environment
- Memory limits are enforced by the host
- CPU time limits prevent infinite loops
- No direct filesystem or network access unless explicitly granted
- All communication goes through the host API

## Debugging

```bash
# Enable debug logging
RUST_LOG=debug cargo run --example plugin_example

# Use wasm-opt to optimize
wasm-opt -Oz input.wasm -o output.wasm

# Inspect WASM module
wasm-objdump -x my_plugin.wasm
```

# Phase 3, Milestone 3.1 - Quick Start Guide

## What Was Built

A comprehensive dataset management system for LLM benchmarking with:
- ✅ Dataset schema with validation
- ✅ JSON and YAML loader
- ✅ Template engine for variable substitution
- ✅ 5 built-in benchmark datasets (28 test cases)
- ✅ 46+ comprehensive tests

## Quick Examples

### 1. Load a Dataset

```rust
use llm_test_bench_datasets::loader::DatasetLoader;
use std::path::Path;

let loader = DatasetLoader::new();
let dataset = loader.load(Path::new("datasets/data/coding-tasks.json"))?;

println!("Loaded: {} with {} tests", dataset.name, dataset.len());
// Output: Loaded: coding-tasks with 7 tests
```

### 2. Use a Built-in Dataset

```rust
use llm_test_bench_datasets::builtin;

let dataset = builtin::coding_tasks();
for test in &dataset.test_cases {
    println!("Test: {}", test.id);
}
// Output:
// Test: fizzbuzz-python
// Test: reverse-string-rust
// ...
```

### 3. Render a Template

```rust
use llm_test_bench_datasets::template::TemplateEngine;
use std::collections::HashMap;

let mut vars = HashMap::new();
vars.insert("lang".to_string(), "Rust".to_string());

let rendered = TemplateEngine::render("Explain {{lang}} ownership", &vars)?;
// Result: "Explain Rust ownership"
```

### 4. Create a Custom Dataset

```rust
use llm_test_bench_datasets::schema::{Dataset, TestCase, DefaultConfig};

let defaults = DefaultConfig::new()
    .with_temperature(0.7)
    .with_max_tokens(500);

let mut dataset = Dataset::new("my-dataset", "1.0.0")
    .with_description("My custom tests")
    .with_defaults(defaults);

dataset.add_test_case(
    TestCase::new("test-1", "What is {{topic}}?")
        .with_category("qa")
        .add_variable("topic", "Rust")
        .with_expected("A systems programming language")
);
```

### 5. Save and Load

```rust
use llm_test_bench_datasets::loader::DatasetLoader;

let loader = DatasetLoader::new();

// Save as JSON
loader.save_json(&dataset, Path::new("my-dataset.json"))?;

// Save as YAML
loader.save_yaml(&dataset, Path::new("my-dataset.yaml"))?;

// Load (auto-detects format)
let loaded = loader.load(Path::new("my-dataset.json"))?;
```

## Built-in Datasets

| Dataset | Test Cases | Temperature | Description |
|---------|-----------|-------------|-------------|
| coding-tasks | 7 | 0.0 | Programming challenges |
| reasoning-tasks | 5 | 0.7 | Logic puzzles |
| summarization-tasks | 4 | 0.5 | Text summarization |
| instruction-following | 6 | 0.3 | Format compliance |
| creative-writing | 6 | 0.9 | Creative generation |

## File Locations

```
datasets/
├── src/
│   ├── schema.rs       # Dataset schema
│   ├── loader.rs       # JSON/YAML loader
│   ├── template.rs     # Template engine
│   ├── builtin.rs      # Built-in datasets
│   ├── tests.rs        # Integration tests
│   └── lib.rs          # Module exports
├── data/
│   ├── coding-tasks.json
│   ├── reasoning-tasks.yaml
│   ├── summarization-tasks.json
│   ├── instruction-following.yaml
│   └── creative-writing.json
├── Cargo.toml          # Dependencies
└── README.md           # Full documentation
```

## Template Syntax

```rust
// Variables use {{name}} syntax
let template = "Write a {{lang}} function to {{task}}";

// Render with HashMap
let mut vars = HashMap::new();
vars.insert("lang".to_string(), "Python".to_string());
vars.insert("task".to_string(), "reverse a string".to_string());

let result = TemplateEngine::render(template, &vars)?;
// Result: "Write a Python function to reverse a string"
```

## Dataset JSON Format

```json
{
  "name": "my-dataset",
  "description": "Example dataset",
  "version": "1.0.0",
  "defaults": {
    "temperature": 0.7,
    "max_tokens": 500
  },
  "test_cases": [
    {
      "id": "test-1",
      "category": "coding",
      "prompt": "Write a {{lang}} function",
      "variables": {
        "lang": "Python"
      },
      "expected": "def",
      "references": ["def", "return"]
    }
  ]
}
```

## Integration with Milestone 3.2 (Benchmark Runner)

```rust
// Load dataset
let dataset = DatasetLoader::new().load(path)?;

// For each test case
for test_case in &dataset.test_cases {
    // Render template
    let prompt = TemplateEngine::render_optional(
        &test_case.prompt,
        &test_case.variables
    )?;

    // Execute with provider
    let request = CompletionRequest {
        prompt,
        temperature: test_case.config
            .as_ref()
            .and_then(|c| c.temperature)
            .or(dataset.defaults.as_ref().and_then(|d| d.temperature)),
        // ... other config
    };

    let response = provider.complete(request).await?;

    // Store result
    results.push(TestResult {
        test_id: test_case.id.clone(),
        expected: test_case.expected.clone(),
        actual: response.text,
        // ...
    });
}
```

## Running Tests

```bash
# Run all tests
cargo test --package llm-test-bench-datasets

# Run specific module tests
cargo test --package llm-test-bench-datasets template::tests

# Run with output
cargo test --package llm-test-bench-datasets -- --nocapture
```

## Key Features

### Schema Validation
- Automatic validation with `serde_valid`
- Clear error messages for invalid datasets
- Can be disabled if needed

### Template Engine
- Regex-based parsing (`\{\{(\w+)\}\}`)
- Variable extraction
- Validation before rendering
- Clear error messages for missing variables

### Multi-Format Support
- JSON and YAML
- Auto-detection by extension
- Round-trip serialization

### Built-in Datasets
- Production-ready test cases
- Template variables for flexibility
- Expected outputs for evaluation
- Reference strings for scoring

## Next Steps

1. **Milestone 3.2**: Implement Benchmark Runner
2. **Milestone 3.3**: Result Storage and Aggregation
3. **Milestone 3.4**: CLI Bench Command

## Documentation

- **Full Documentation**: `/workspaces/llm-test-bench/datasets/README.md`
- **Implementation Report**: `/workspaces/llm-test-bench/PHASE3_MILESTONE3.1_COMPLETE.md`
- **File Manifest**: `/workspaces/llm-test-bench/DATASET_FILES_SUMMARY.txt`

## Status

✅ **MILESTONE 3.1 COMPLETE**

- 46+ tests passing
- 5 built-in datasets
- 28 test cases
- Complete documentation
- Ready for integration

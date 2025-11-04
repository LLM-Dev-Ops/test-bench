# LLM Test Bench Datasets

Comprehensive dataset management system for the LLM Test Bench framework.

## Features

- **Schema Validation**: Comprehensive dataset schema with `serde_valid` for data integrity
- **Multiple Formats**: Load datasets from JSON and YAML files with automatic format detection
- **Template Engine**: Variable substitution in prompts using `{{variable}}` syntax
- **Built-in Datasets**: 5 ready-to-use benchmark datasets covering common LLM evaluation scenarios
- **Type Safety**: Strongly-typed Rust structures with compile-time guarantees

## Dataset Schema

```rust
pub struct Dataset {
    name: String,              // Required, min_length=1
    description: Option<String>,
    version: String,
    test_cases: Vec<TestCase>, // Required, min_items=1
    defaults: Option<DefaultConfig>,
    metadata: Option<HashMap<String, serde_json::Value>>,
}

pub struct TestCase {
    id: String,                // Required, unique
    category: Option<String>,
    prompt: String,            // Required, supports {{variables}}
    variables: Option<HashMap<String, String>>,
    expected: Option<String>,  // For evaluation
    references: Option<Vec<String>>,
    config: Option<TestConfig>,
    metadata: Option<HashMap<String, serde_json::Value>>,
}
```

## Template Engine

The template engine uses `{{variable_name}}` syntax for variable substitution:

```rust
use llm_test_bench_datasets::template::TemplateEngine;
use std::collections::HashMap;

let mut vars = HashMap::new();
vars.insert("lang".to_string(), "Rust".to_string());
vars.insert("topic".to_string(), "ownership".to_string());

let rendered = TemplateEngine::render(
    "Explain {{lang}} {{topic}}",
    &vars
).unwrap();

assert_eq!(rendered, "Explain Rust ownership");
```

### Template Features

- **Variable Extraction**: `extract_variables()` returns all variable names
- **Validation**: `validate()` checks all required variables are provided
- **Optional Rendering**: `render_optional()` handles templates with or without variables
- **Error Reporting**: Clear error messages for missing variables

## Built-in Datasets

The crate includes 5 production-ready benchmark datasets:

### 1. Coding Tasks (`coding-tasks`)

Programming challenges including FizzBuzz, string manipulation, and algorithms.

- **Test Cases**: 7
- **Temperature**: 0.0 (deterministic)
- **Categories**: coding
- **Features**: Template variables for language selection

### 2. Reasoning Tasks (`reasoning-tasks`)

Logic puzzles and math word problems.

- **Test Cases**: 5
- **Temperature**: 0.7 (natural reasoning)
- **Categories**: reasoning
- **Features**: Pattern recognition, age problems, river crossing puzzle

### 3. Summarization Tasks (`summarization-tasks`)

Text summarization and compression challenges.

- **Test Cases**: 4
- **Temperature**: 0.5 (balanced)
- **Categories**: summarization
- **Features**: Article summaries, bullet points, TL;DR generation

### 4. Instruction Following (`instruction-following`)

Tests for instruction adherence and format compliance.

- **Test Cases**: 6
- **Temperature**: 0.3 (consistent)
- **Categories**: instruction-following
- **Features**: Format constraints (JSON, CSV, lists), word limits

### 5. Creative Writing (`creative-writing`)

Creative generation and storytelling tasks.

- **Test Cases**: 6
- **Temperature**: 0.9 (creative)
- **Categories**: creative-writing
- **Features**: Story openings, haikus, product descriptions, metaphors

## Usage Examples

### Loading a Dataset

```rust
use llm_test_bench_datasets::loader::DatasetLoader;
use std::path::Path;

let loader = DatasetLoader::new();

// Load from file (auto-detects JSON or YAML)
let dataset = loader.load(Path::new("datasets/coding-tasks.json"))?;

// Load all datasets from a directory
let datasets = loader.load_dir(Path::new("./datasets/data"))?;

// Load from string
let json = r#"{"name": "test", "version": "1.0.0", "test_cases": [...]}"#;
let dataset = loader.load_from_json_str(json)?;
```

### Using Built-in Datasets

```rust
use llm_test_bench_datasets::builtin;

// Get all built-in datasets
let datasets = builtin::get_builtin_datasets();

// Get a specific dataset
let coding = builtin::coding_tasks();
let reasoning = builtin::reasoning_tasks();
```

### Rendering Templates

```rust
use llm_test_bench_datasets::template::TemplateEngine;

let test_case = &dataset.test_cases[0];

if let Some(ref vars) = test_case.variables {
    let prompt = TemplateEngine::render(&test_case.prompt, vars)?;
    println!("Rendered: {}", prompt);
}
```

### Creating Custom Datasets

```rust
use llm_test_bench_datasets::schema::{Dataset, TestCase, DefaultConfig};

let defaults = DefaultConfig::new()
    .with_temperature(0.7)
    .with_max_tokens(500);

let mut dataset = Dataset::new("my-dataset", "1.0.0")
    .with_description("Custom benchmark dataset")
    .with_defaults(defaults);

dataset.add_test_case(
    TestCase::new("test-1", "What is {{topic}}?")
        .with_category("qa")
        .add_variable("topic", "Rust")
        .with_expected("A systems programming language")
);

// Save to file
let loader = DatasetLoader::new();
loader.save_json(&dataset, Path::new("my-dataset.json"))?;
```

## Dataset File Formats

### JSON Format

```json
{
  "name": "example-dataset",
  "description": "Example benchmark dataset",
  "version": "1.0.0",
  "defaults": {
    "temperature": 0.7,
    "max_tokens": 500
  },
  "test_cases": [
    {
      "id": "test-1",
      "category": "coding",
      "prompt": "Write a {{lang}} function to {{task}}",
      "variables": {
        "lang": "Python",
        "task": "reverse a string"
      },
      "expected": "def reverse",
      "references": ["[::-1]", "reversed()"]
    }
  ]
}
```

### YAML Format

```yaml
name: example-dataset
description: Example benchmark dataset
version: "1.0.0"

defaults:
  temperature: 0.7
  max_tokens: 500

test_cases:
  - id: test-1
    category: coding
    prompt: Write a {{lang}} function to {{task}}
    variables:
      lang: Python
      task: reverse a string
    expected: "def reverse"
    references:
      - "[::-1]"
      - "reversed()"
```

## Validation

All datasets are validated against the schema:

```rust
use serde_valid::Validate;

let dataset = loader.load(path)?;
dataset.validate()?; // Validates schema constraints
```

### Validation Rules

- **Dataset name**: Minimum length 1
- **Test cases**: Minimum 1 test case required
- **Test IDs**: Minimum length 1
- **Prompts**: Minimum length 1

## Directory Structure

```
datasets/
├── src/
│   ├── lib.rs           # Main library exports
│   ├── schema.rs        # Dataset schema definitions
│   ├── loader.rs        # Dataset loader (JSON/YAML)
│   ├── template.rs      # Template engine
│   ├── builtin.rs       # Built-in dataset functions
│   └── tests.rs         # Integration tests
├── data/
│   ├── coding-tasks.json
│   ├── reasoning-tasks.yaml
│   ├── summarization-tasks.json
│   ├── instruction-following.yaml
│   └── creative-writing.json
├── Cargo.toml
└── README.md
```

## Testing

The crate includes 46+ comprehensive tests covering:

- Dataset loading (JSON and YAML)
- Schema validation
- Template rendering
- Variable extraction and validation
- Built-in dataset integrity
- Round-trip serialization
- Error handling

Run tests with:

```bash
cargo test --package llm-test-bench-datasets
```

## Dependencies

- `serde` & `serde_json` - Serialization
- `serde_yaml` - YAML support
- `serde_valid` - Schema validation
- `regex` - Template engine
- `anyhow` & `thiserror` - Error handling
- `tracing` - Logging

## Integration with Benchmark Runner

Datasets from this crate integrate seamlessly with the benchmark runner (Milestone 3.2):

```rust
use llm_test_bench_datasets::loader::DatasetLoader;
use llm_test_bench_core::benchmarks::BenchmarkRunner;

let dataset = DatasetLoader::new().load(path)?;
let runner = BenchmarkRunner::new(config);
let results = runner.run(&dataset, provider).await?;
```

## License

Dual-licensed under MIT OR Apache-2.0.

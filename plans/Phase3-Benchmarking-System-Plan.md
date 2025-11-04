# LLM Test Bench - Phase 3: Benchmarking System
## Detailed Implementation Plan

**Phase:** Phase 3 - Benchmarking System (Weeks 9-12)
**Planning Date:** November 4, 2025
**Document Version:** 1.0
**Status:** Ready for Implementation
**Previous Phases:** Phase 1 Complete ✅ | Phase 2 Complete ✅

---

## Executive Summary

### Phase 3 Objectives

Phase 3 focuses on building the benchmarking system that enables systematic, large-scale testing of LLM providers across multiple prompts and models. This phase transforms the test bench from a single-query tool into a comprehensive benchmarking platform capable of processing datasets, running concurrent tests, aggregating results, and generating reports.

### Key Deliverables

1. **Dataset Management System** - Load, validate, and manage test datasets (JSON/YAML)
2. **Benchmark Runner** - Async batch processing with configurable concurrency
3. **Result Storage** - Structured result format with aggregation and incremental updates
4. **CLI Bench Command** - Functional `llm-test-bench bench` command with progress reporting
5. **Export Formats** - CSV and JSON result export with customizable fields
6. **Built-in Datasets** - 3-5 standard benchmark datasets for common use cases

### Success Criteria

- ✅ Load datasets from JSON/YAML files
- ✅ Run benchmarks across multiple providers in parallel
- ✅ Process 100+ prompts concurrently without degradation
- ✅ Save results in structured JSON format
- ✅ Export results to CSV for analysis
- ✅ Progress reporting with ETA
- ✅ 80%+ code coverage on benchmark modules
- ✅ Complete documentation with examples

---

## Table of Contents

1. [Phase 3 Overview](#phase-3-overview)
2. [Milestone Breakdown](#milestone-breakdown)
3. [Technical Architecture](#technical-architecture)
4. [Implementation Details](#implementation-details)
5. [Testing Strategy](#testing-strategy)
6. [Risk Assessment](#risk-assessment)
7. [Timeline and Resources](#timeline-and-resources)
8. [Success Metrics](#success-metrics)
9. [Appendices](#appendices)

---

## 1. Phase 3 Overview

### 1.1 Phase Scope

**In Scope:**
- Dataset loading from JSON/YAML files
- Dataset schema validation with serde_valid
- Built-in benchmark datasets (coding, reasoning, summarization)
- Prompt templating with variable substitution
- Async batch processing with Tokio
- Configurable concurrency limits
- Progress reporting with indicatif
- Result storage in JSON format
- Result aggregation and statistics
- Incremental result updates
- CSV export for analysis
- CLI bench command implementation

**Out of Scope (Deferred to Later Phases):**
- Evaluation metrics (Phase 4)
- Advanced analytics (Phase 5)
- Distributed benchmarking (Future)
- Real-time dashboards (Future)
- Database storage (Future)
- A/B testing framework (Future)

### 1.2 Dependencies

**From Previous Phases:**
- ✅ Phase 1: Configuration system, CLI framework
- ✅ Phase 2: Provider implementations (OpenAI, Anthropic)
- ✅ Phase 2: Streaming support and error handling
- ✅ Phase 2: Async runtime (Tokio)

**External Dependencies:**
- Multiple LLM provider API keys (OpenAI, Anthropic)
- Internet connectivity for API calls
- Sufficient API rate limits for concurrent testing
- Disk space for result storage

### 1.3 Architecture Context

```
Phase 1 (Foundation) + Phase 2 (Providers)
                ↓
┌───────────────────────────────────┐
│  Phase 3: Benchmarking System     │
├───────────────────────────────────┤
│                                   │
│  ┌─────────────────────────────┐ │
│  │   Dataset Loader            │ │
│  │   (JSON/YAML + Validation)  │ │
│  └─────────────────────────────┘ │
│              ↓                    │
│  ┌─────────────────────────────┐ │
│  │   Benchmark Runner          │ │
│  │   (Async + Concurrency)     │ │
│  └─────────────────────────────┘ │
│              ↓                    │
│  ┌─────────────────────────────┐ │
│  │   Result Storage            │ │
│  │   (JSON + CSV Export)       │ │
│  └─────────────────────────────┘ │
│              ↓                    │
│  ┌─────────────────────────────┐ │
│  │   CLI Bench Command         │ │
│  └─────────────────────────────┘ │
└───────────────────────────────────┘
                ↓
Phase 4: Evaluation Metrics
```

---

## 2. Milestone Breakdown

### Milestone 3.1: Dataset Management (Week 9, Days 1-5)

**Status:** Not Started
**Duration:** 5 days
**Priority:** CRITICAL (blocks all other work)

#### Objectives
- Implement dataset loading from JSON and YAML files
- Define and validate dataset schema
- Support prompt templating with variable substitution
- Create 3-5 built-in benchmark datasets
- Ground truth answer storage for evaluation

#### Tasks

**Task 3.1.1: Dataset Schema Definition** (6 hours)

Define comprehensive dataset schema with serde:

```rust
// Location: datasets/src/schema.rs

use serde::{Deserialize, Serialize};
use serde_valid::Validate;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct Dataset {
    /// Dataset name
    #[validate(min_length = 1)]
    pub name: String,

    /// Dataset description
    pub description: Option<String>,

    /// Dataset version
    pub version: String,

    /// Test cases
    #[validate(min_items = 1)]
    pub test_cases: Vec<TestCase>,

    /// Default model configuration
    pub defaults: Option<DefaultConfig>,

    /// Metadata
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct TestCase {
    /// Unique test ID
    #[validate(min_length = 1)]
    pub id: String,

    /// Test category/tag
    pub category: Option<String>,

    /// Prompt template (supports {{variables}})
    #[validate(min_length = 1)]
    pub prompt: String,

    /// Variable values for templating
    pub variables: Option<HashMap<String, String>>,

    /// Expected output (for evaluation)
    pub expected: Option<String>,

    /// Reference answers (for comparison)
    pub references: Option<Vec<String>>,

    /// Per-test model configuration overrides
    pub config: Option<TestConfig>,

    /// Test metadata
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefaultConfig {
    pub temperature: Option<f32>,
    pub max_tokens: Option<usize>,
    pub top_p: Option<f32>,
    pub stop: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestConfig {
    pub model: Option<String>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<usize>,
    pub top_p: Option<f32>,
    pub stop: Option<Vec<String>>,
}
```

**Task 3.1.2: Dataset Loader Implementation** (8 hours)

Implement loader with JSON and YAML support:

```rust
// Location: datasets/src/loader.rs

use crate::schema::Dataset;
use anyhow::{Context, Result};
use std::path::Path;
use serde_valid::Validate;

pub struct DatasetLoader;

impl DatasetLoader {
    /// Load dataset from file (auto-detect JSON or YAML)
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Dataset> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path)
            .context("Failed to read dataset file")?;

        // Auto-detect format by extension
        let dataset = if path.extension().and_then(|s| s.to_str()) == Some("yaml")
            || path.extension().and_then(|s| s.to_str()) == Some("yml") {
            serde_yaml::from_str::<Dataset>(&content)
                .context("Failed to parse YAML dataset")?
        } else {
            serde_json::from_str::<Dataset>(&content)
                .context("Failed to parse JSON dataset")?
        };

        // Validate schema
        dataset.validate()
            .context("Dataset validation failed")?;

        Ok(dataset)
    }

    /// Load multiple datasets from a directory
    pub fn load_dir<P: AsRef<Path>>(dir: P) -> Result<Vec<Dataset>> {
        let dir = dir.as_ref();
        let mut datasets = Vec::new();

        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() && Self::is_dataset_file(&path) {
                match Self::load(&path) {
                    Ok(dataset) => datasets.push(dataset),
                    Err(e) => eprintln!("Warning: Failed to load {}: {}", path.display(), e),
                }
            }
        }

        Ok(datasets)
    }

    fn is_dataset_file(path: &Path) -> bool {
        matches!(
            path.extension().and_then(|s| s.to_str()),
            Some("json") | Some("yaml") | Some("yml")
        )
    }
}
```

**Task 3.1.3: Prompt Templating Engine** (6 hours)

Implement variable substitution in prompts:

```rust
// Location: datasets/src/template.rs

use std::collections::HashMap;
use anyhow::{anyhow, Result};
use regex::Regex;

pub struct TemplateEngine;

impl TemplateEngine {
    /// Render a prompt template with variables
    pub fn render(template: &str, variables: &HashMap<String, String>) -> Result<String> {
        let re = Regex::new(r"\{\{(\w+)\}\}").unwrap();
        let mut result = template.to_string();

        for caps in re.captures_iter(template) {
            let var_name = &caps[1];
            let value = variables.get(var_name)
                .ok_or_else(|| anyhow!("Missing variable: {}", var_name))?;

            result = result.replace(&format!("{{{{{}}}}}", var_name), value);
        }

        // Check for any remaining unsubstituted variables
        if re.is_match(&result) {
            return Err(anyhow!("Unsubstituted variables remain in template"));
        }

        Ok(result)
    }

    /// Extract variable names from a template
    pub fn extract_variables(template: &str) -> Vec<String> {
        let re = Regex::new(r"\{\{(\w+)\}\}").unwrap();
        re.captures_iter(template)
            .map(|caps| caps[1].to_string())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_simple() {
        let mut vars = HashMap::new();
        vars.insert("name".to_string(), "Alice".to_string());

        let result = TemplateEngine::render("Hello, {{name}}!", &vars).unwrap();
        assert_eq!(result, "Hello, Alice!");
    }

    #[test]
    fn test_render_multiple() {
        let mut vars = HashMap::new();
        vars.insert("lang".to_string(), "Rust".to_string());
        vars.insert("feature".to_string(), "ownership".to_string());

        let result = TemplateEngine::render(
            "Explain {{lang}} {{feature}}",
            &vars
        ).unwrap();
        assert_eq!(result, "Explain Rust ownership");
    }

    #[test]
    fn test_missing_variable() {
        let vars = HashMap::new();
        let result = TemplateEngine::render("Hello, {{name}}!", &vars);
        assert!(result.is_err());
    }
}
```

**Task 3.1.4: Built-in Datasets** (10 hours)

Create 3-5 standard benchmark datasets:

```json
// datasets/data/coding-tasks.json
{
  "name": "coding-tasks",
  "description": "Basic coding challenges in multiple languages",
  "version": "1.0.0",
  "defaults": {
    "temperature": 0.0,
    "max_tokens": 500
  },
  "test_cases": [
    {
      "id": "fizzbuzz-python",
      "category": "coding",
      "prompt": "Write a Python function that implements FizzBuzz for numbers 1 to {{n}}.",
      "variables": {"n": "100"},
      "expected": "def fizzbuzz",
      "references": ["for i in range", "if i % 15", "FizzBuzz"]
    },
    {
      "id": "reverse-string-rust",
      "category": "coding",
      "prompt": "Write a Rust function to reverse a string in-place.",
      "expected": "fn reverse",
      "references": ["chars()", "rev()", "collect()"]
    },
    {
      "id": "fibonacci-{{lang}}",
      "category": "coding",
      "prompt": "Implement a {{lang}} function to calculate the nth Fibonacci number.",
      "variables": {"lang": "JavaScript"},
      "references": ["function", "fibonacci"]
    }
  ]
}
```

```yaml
# datasets/data/reasoning-tasks.yaml
name: reasoning-tasks
description: Logical reasoning and problem-solving tasks
version: 1.0.0
defaults:
  temperature: 0.7
  max_tokens: 300

test_cases:
  - id: logic-puzzle-1
    category: reasoning
    prompt: |
      Three people are in a room: Alice, Bob, and Carol.
      - Alice always tells the truth
      - Bob always lies
      - Carol sometimes tells the truth and sometimes lies

      Alice says: "Bob is lying."
      Bob says: "Carol is telling the truth."
      Carol says: "I am lying."

      Who is telling the truth?
    expected: "Alice"
    references:
      - "Alice tells the truth"
      - "logical contradiction"

  - id: math-word-problem
    category: reasoning
    prompt: |
      If a train travels {{distance}} km at {{speed}} km/h,
      how long does the journey take in hours?
    variables:
      distance: "240"
      speed: "80"
    expected: "3 hours"
```

**Additional Datasets:**
- `summarization-tasks.json` - Text summarization
- `instruction-following.yaml` - Instruction adherence
- `creative-writing.json` - Creative tasks

**Task 3.1.5: Unit Tests** (6 hours)

- Test dataset loading (JSON and YAML)
- Test schema validation
- Test template rendering
- Test variable extraction
- Test built-in datasets validity
- Test error handling (missing files, invalid JSON, etc.)

**Deliverables:**
- ✅ Complete dataset schema definition
- ✅ Dataset loader (JSON + YAML)
- ✅ Prompt templating engine
- ✅ 3-5 built-in datasets
- ✅ 20+ unit tests
- ✅ Complete documentation

---

### Milestone 3.2: Benchmark Runner (Week 10, Days 6-10)

**Status:** Not Started
**Duration:** 5 days
**Priority:** CRITICAL

#### Objectives
- Implement async batch processing with Tokio
- Add configurable concurrency limits
- Progress reporting with indicatif
- Save raw responses to disk
- Handle failures gracefully

#### Tasks

**Task 3.2.1: Benchmark Configuration** (4 hours)

```rust
// Location: core/src/benchmarks/config.rs

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkConfig {
    /// Maximum concurrent requests
    pub concurrency: usize,

    /// Whether to save raw responses
    pub save_responses: bool,

    /// Output directory for results
    pub output_dir: PathBuf,

    /// Continue on failure or stop
    pub continue_on_failure: bool,

    /// Random seed for reproducibility
    pub random_seed: Option<u64>,

    /// Delay between requests (ms)
    pub request_delay_ms: Option<u64>,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            concurrency: 5,
            save_responses: true,
            output_dir: PathBuf::from("./bench-results"),
            continue_on_failure: true,
            random_seed: None,
            request_delay_ms: None,
        }
    }
}
```

**Task 3.2.2: Benchmark Runner Implementation** (12 hours)

```rust
// Location: core/src/benchmarks/runner.rs

use crate::providers::{Provider, CompletionRequest, CompletionResponse};
use crate::benchmarks::config::BenchmarkConfig;
use llm_test_bench_datasets::Dataset;
use futures::stream::{self, StreamExt};
use indicatif::{ProgressBar, ProgressStyle};
use tokio::sync::Semaphore;
use std::sync::Arc;
use std::time::Instant;

pub struct BenchmarkRunner {
    config: BenchmarkConfig,
}

impl BenchmarkRunner {
    pub fn new(config: BenchmarkConfig) -> Self {
        Self { config }
    }

    /// Run benchmark across all test cases in dataset
    pub async fn run(
        &self,
        dataset: &Dataset,
        provider: Arc<dyn Provider>,
    ) -> Result<BenchmarkResults> {
        let total = dataset.test_cases.len();
        let pb = Self::create_progress_bar(total);

        // Semaphore for concurrency control
        let semaphore = Arc::new(Semaphore::new(self.config.concurrency));

        // Process test cases concurrently
        let results: Vec<TestResult> = stream::iter(&dataset.test_cases)
            .map(|test_case| {
                let provider = Arc::clone(&provider);
                let semaphore = Arc::clone(&semaphore);
                let pb = pb.clone();
                let config = self.config.clone();

                async move {
                    // Acquire semaphore permit
                    let _permit = semaphore.acquire().await.unwrap();

                    // Optional delay between requests
                    if let Some(delay) = config.request_delay_ms {
                        tokio::time::sleep(Duration::from_millis(delay)).await;
                    }

                    // Run test case
                    let result = Self::run_test_case(
                        test_case,
                        &provider,
                        &config,
                    ).await;

                    pb.inc(1);
                    result
                }
            })
            .buffer_unordered(self.config.concurrency)
            .collect()
            .await;

        pb.finish_with_message("Benchmark complete");

        Ok(BenchmarkResults {
            dataset_name: dataset.name.clone(),
            total_tests: total,
            results,
            timestamp: chrono::Utc::now(),
        })
    }

    async fn run_test_case(
        test_case: &TestCase,
        provider: &Arc<dyn Provider>,
        config: &BenchmarkConfig,
    ) -> TestResult {
        let start = Instant::now();

        // Render prompt template
        let prompt = match test_case.variables {
            Some(ref vars) => TemplateEngine::render(&test_case.prompt, vars),
            None => Ok(test_case.prompt.clone()),
        };

        let prompt = match prompt {
            Ok(p) => p,
            Err(e) => return TestResult::error(test_case.id.clone(), e),
        };

        // Build request
        let request = CompletionRequest {
            model: test_case.config.as_ref()
                .and_then(|c| c.model.clone())
                .unwrap_or_else(|| provider.default_model()),
            prompt,
            temperature: test_case.config.as_ref()
                .and_then(|c| c.temperature),
            max_tokens: test_case.config.as_ref()
                .and_then(|c| c.max_tokens),
            top_p: test_case.config.as_ref()
                .and_then(|c| c.top_p),
            stop: test_case.config.as_ref()
                .and_then(|c| c.stop.clone()),
            stream: false,
        };

        // Execute request
        let response = provider.complete(request).await;
        let duration = start.elapsed();

        match response {
            Ok(resp) => {
                // Save raw response if configured
                if config.save_responses {
                    Self::save_response(&test_case.id, &resp, config);
                }

                TestResult::success(
                    test_case.id.clone(),
                    test_case.category.clone(),
                    resp,
                    duration,
                )
            }
            Err(e) => TestResult::error(test_case.id.clone(), e),
        }
    }

    fn create_progress_bar(total: usize) -> ProgressBar {
        let pb = ProgressBar::new(total as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} {msg}")
                .unwrap()
                .progress_chars("=>-")
        );
        pb
    }

    fn save_response(
        test_id: &str,
        response: &CompletionResponse,
        config: &BenchmarkConfig,
    ) {
        let output_dir = &config.output_dir;
        std::fs::create_dir_all(output_dir).ok();

        let filename = format!("{}/{}.json", output_dir.display(), test_id);
        let json = serde_json::to_string_pretty(response).unwrap();
        std::fs::write(filename, json).ok();
    }
}
```

**Task 3.2.3: Progress Reporting** (4 hours)

Implement detailed progress reporting with:
- Progress bar with ETA
- Current test ID display
- Success/failure counts
- Throughput statistics (tests/sec)
- Estimated completion time

**Task 3.2.4: Error Handling** (4 hours)

Handle failures gracefully:
- Retry failed tests (configurable)
- Continue on failure (configurable)
- Log errors to separate file
- Summary of failures at end

**Task 3.2.5: Unit Tests** (6 hours)

- Test concurrent execution
- Test progress reporting
- Test error handling
- Test result saving
- Test semaphore limiting
- Mock provider for testing

**Deliverables:**
- ✅ Complete benchmark runner
- ✅ Concurrent execution with semaphore
- ✅ Progress reporting with indicatif
- ✅ Raw response saving
- ✅ 15+ unit tests
- ✅ Documentation

---

### Milestone 3.3: Result Storage (Week 11, Days 11-15)

**Status:** Not Started
**Duration:** 5 days
**Priority:** HIGH

#### Objectives
- Design result schema (JSON format)
- Implement result serialization
- Add result aggregation logic
- Support incremental result updates
- Generate summary statistics

#### Tasks

**Task 3.3.1: Result Schema Definition** (4 hours)

```rust
// Location: core/src/benchmarks/results.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResults {
    pub dataset_name: String,
    pub provider_name: String,
    pub total_tests: usize,
    pub results: Vec<TestResult>,
    pub timestamp: DateTime<Utc>,
    pub summary: ResultSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub test_id: String,
    pub category: Option<String>,
    pub status: TestStatus,
    pub response: Option<CompletionResponse>,
    pub error: Option<String>,
    pub duration_ms: u64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TestStatus {
    Success,
    Failure,
    Timeout,
    Skipped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultSummary {
    pub total: usize,
    pub succeeded: usize,
    pub failed: usize,
    pub timeout: usize,
    pub skipped: usize,
    pub success_rate: f64,
    pub avg_duration_ms: f64,
    pub total_tokens: usize,
    pub total_cost: f64,
}
```

**Task 3.3.2: Result Aggregation** (6 hours)

Implement statistics and aggregation:
- Success rate calculation
- Average duration
- Token usage statistics
- Cost estimation
- Latency percentiles (P50, P95, P99)

**Task 3.3.3: Incremental Updates** (4 hours)

Support saving results incrementally:
- Append-only result file
- Resume interrupted benchmarks
- Merge result files

**Task 3.3.4: CSV Export** (6 hours)

```rust
// Location: core/src/benchmarks/export.rs

pub struct CsvExporter;

impl CsvExporter {
    pub fn export(results: &BenchmarkResults, path: &Path) -> Result<()> {
        let mut wtr = csv::Writer::from_path(path)?;

        // Write headers
        wtr.write_record(&[
            "test_id",
            "category",
            "status",
            "duration_ms",
            "tokens",
            "model",
            "prompt_length",
            "response_length",
            "error",
        ])?;

        // Write rows
        for result in &results.results {
            wtr.write_record(&[
                &result.test_id,
                result.category.as_deref().unwrap_or(""),
                &format!("{:?}", result.status),
                &result.duration_ms.to_string(),
                &result.response.as_ref()
                    .map(|r| r.usage.total_tokens.to_string())
                    .unwrap_or_default(),
                &result.response.as_ref()
                    .map(|r| r.model.clone())
                    .unwrap_or_default(),
                // ... more fields
            ])?;
        }

        wtr.flush()?;
        Ok(())
    }
}
```

**Task 3.3.5: Unit Tests** (6 hours)

- Test result serialization
- Test aggregation calculations
- Test CSV export
- Test incremental updates

**Deliverables:**
- ✅ Complete result schema
- ✅ Aggregation and statistics
- ✅ CSV export functionality
- ✅ Incremental update support
- ✅ 15+ unit tests
- ✅ Documentation

---

### Milestone 3.4: CLI Bench Command (Week 12, Days 16-20)

**Status:** Not Started
**Duration:** 5 days
**Priority:** CRITICAL

#### Objectives
- Implement `llm-test-bench bench` command
- Support multi-provider benchmarking
- Add CSV/JSON output formats
- Integration tests with sample datasets

#### Tasks

**Task 3.4.1: Command Implementation** (8 hours)

```rust
// Location: cli/src/commands/bench.rs

#[derive(Debug, Args)]
pub struct BenchArgs {
    /// Path to dataset file
    #[arg(short, long)]
    dataset: PathBuf,

    /// Providers to test (comma-separated)
    #[arg(short, long, value_delimiter = ',')]
    providers: Vec<String>,

    /// Concurrency level
    #[arg(short, long, default_value = "5")]
    concurrency: usize,

    /// Output directory
    #[arg(short, long, default_value = "./bench-results")]
    output: PathBuf,

    /// Export format (json, csv, both)
    #[arg(short, long, default_value = "both")]
    export: ExportFormat,

    /// Continue on failure
    #[arg(long)]
    continue_on_failure: bool,

    /// Save raw responses
    #[arg(long, default_value = "true")]
    save_responses: bool,
}

pub async fn execute(args: BenchArgs) -> Result<()> {
    // 1. Load dataset
    let dataset = DatasetLoader::load(&args.dataset)?;
    println!("Loaded dataset: {} ({} tests)",
        dataset.name, dataset.test_cases.len());

    // 2. Load configuration
    let config = ConfigLoader::new().load()?;

    // 3. Run benchmarks for each provider
    for provider_name in &args.providers {
        println!("\nBenchmarking provider: {}", provider_name);

        let provider = create_provider(provider_name, &config)?;

        let bench_config = BenchmarkConfig {
            concurrency: args.concurrency,
            save_responses: args.save_responses,
            output_dir: args.output.clone(),
            continue_on_failure: args.continue_on_failure,
            ..Default::default()
        };

        let runner = BenchmarkRunner::new(bench_config);
        let results = runner.run(&dataset, provider).await?;

        // 4. Save results
        save_results(&results, &args)?;

        // 5. Print summary
        print_summary(&results);
    }

    Ok(())
}
```

**Task 3.4.2: Multi-Provider Support** (6 hours)

Enable running benchmarks across multiple providers:
- Sequential execution (one provider at a time)
- Comparative results
- Provider performance comparison

**Task 3.4.3: Output Formatting** (4 hours)

Implement result output:
- JSON export
- CSV export
- Summary table to console
- Comparison chart (optional)

**Task 3.4.4: Integration Tests** (8 hours)

- Test with sample datasets
- Test multi-provider execution
- Test CSV/JSON export
- Test error handling
- Use mocked providers

**Deliverables:**
- ✅ Functional bench command
- ✅ Multi-provider support
- ✅ CSV/JSON export
- ✅ 20+ integration tests
- ✅ Complete documentation

---

## 3. Technical Architecture

### 3.1 Benchmarking System Architecture

```
┌─────────────────────────────────────────┐
│         CLI Layer (bench command)        │
│  - Dataset path parsing                  │
│  - Provider selection                    │
│  - Output configuration                  │
└─────────────────────────────────────────┘
                ↓
┌─────────────────────────────────────────┐
│       Dataset Loader                     │
│  - JSON/YAML parsing                     │
│  - Schema validation                     │
│  - Template rendering                    │
└─────────────────────────────────────────┘
                ↓
┌─────────────────────────────────────────┐
│       Benchmark Runner                   │
│  - Concurrent execution (Tokio)          │
│  - Semaphore (concurrency limit)         │
│  - Progress reporting                    │
│  - Error handling                        │
└─────────────────────────────────────────┘
                ↓
┌─────────────────────────────────────────┐
│       Provider Layer                     │
│  (from Phase 2)                          │
└─────────────────────────────────────────┘
                ↓
┌─────────────────────────────────────────┐
│       Result Storage                     │
│  - JSON serialization                    │
│  - CSV export                            │
│  - Aggregation                           │
│  - Statistics                            │
└─────────────────────────────────────────┘
```

### 3.2 Concurrent Execution Flow

```
Load Dataset
     ↓
Parse Test Cases (N tests)
     ↓
Create Semaphore (limit = concurrency)
     ↓
┌────────────────────────────┐
│  Spawn N Async Tasks       │
│  (tokio::spawn)            │
└────────────────────────────┘
     ↓         ↓         ↓
  Task 1    Task 2  ... Task N
     ↓         ↓         ↓
  Acquire   Acquire   Acquire
  Permit    Permit    Permit
     ↓         ↓         ↓
  Execute   Execute   Execute
  Provider  Provider  Provider
     ↓         ↓         ↓
  Release   Release   Release
  Permit    Permit    Permit
     ↓         ↓         ↓
  Return    Return    Return
  Result    Result    Result
     ↓         ↓         ↓
┌────────────────────────────┐
│  Collect Results           │
└────────────────────────────┘
     ↓
Aggregate & Export
```

### 3.3 Dataset Processing

```
Dataset File (JSON/YAML)
     ↓
Parse with serde
     ↓
Validate with serde_valid
     ↓
Extract Test Cases
     ↓
For Each Test Case:
     ↓
┌────────────────────────────┐
│  Has Variables?            │
├─────────┬──────────────────┤
│   Yes   │       No         │
↓         ↓
Render    Use
Template  As-is
↓         ↓
└─────────┴──────────────────┘
     ↓
Build CompletionRequest
     ↓
Execute Provider Call
     ↓
Store Result
```

---

## 4. Implementation Details

### 4.1 Concurrency Control

**Semaphore-based Limiting:**

```rust
use tokio::sync::Semaphore;
use std::sync::Arc;

let semaphore = Arc::new(Semaphore::new(concurrency_limit));

// In each task:
let _permit = semaphore.acquire().await?;
// Execute provider call
// Permit automatically released when dropped
```

**Benefits:**
- Prevents overwhelming APIs
- Respects rate limits
- Controls resource usage
- Graceful degradation

### 4.2 Progress Reporting

**Using indicatif:**

```rust
use indicatif::{ProgressBar, ProgressStyle, MultiProgress};

let multi = MultiProgress::new();
let main_pb = multi.add(ProgressBar::new(total));

main_pb.set_style(
    ProgressStyle::default_bar()
        .template("[{elapsed_precise}] {bar:40} {pos}/{len} ETA: {eta}")
        .unwrap()
);

// Update in each task
main_pb.inc(1);
main_pb.set_message(format!("Testing: {}", test_id));

main_pb.finish_with_message("Complete");
```

### 4.3 Result Aggregation

**Statistics Calculation:**

```rust
impl BenchmarkResults {
    pub fn calculate_summary(&mut self) {
        let total = self.results.len();
        let succeeded = self.results.iter()
            .filter(|r| matches!(r.status, TestStatus::Success))
            .count();
        let failed = total - succeeded;

        let avg_duration = self.results.iter()
            .map(|r| r.duration_ms)
            .sum::<u64>() as f64 / total as f64;

        let total_tokens: usize = self.results.iter()
            .filter_map(|r| r.response.as_ref())
            .map(|resp| resp.usage.total_tokens)
            .sum();

        self.summary = ResultSummary {
            total,
            succeeded,
            failed,
            success_rate: succeeded as f64 / total as f64,
            avg_duration_ms: avg_duration,
            total_tokens,
            // ... more stats
        };
    }
}
```

### 4.4 CSV Export Format

**Example CSV Output:**

```csv
test_id,category,status,duration_ms,tokens,cost,model,error
fizzbuzz-python,coding,Success,1234,450,0.0023,gpt-4,
reverse-string,coding,Success,987,320,0.0016,gpt-4,
logic-puzzle,reasoning,Success,2100,580,0.0029,gpt-4,
invalid-test,coding,Failure,0,0,0.0,,Template error: missing variable
```

### 4.5 Incremental Result Saving

**Append-Only Strategy:**

```rust
// Save each result as it completes
async fn save_incremental(result: &TestResult, output_dir: &Path) -> Result<()> {
    let filename = output_dir.join("results.jsonl"); // JSON Lines format
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(filename)?;

    let json = serde_json::to_string(result)?;
    writeln!(file, "{}", json)?;

    Ok(())
}

// Load all results
fn load_incremental(output_dir: &Path) -> Result<Vec<TestResult>> {
    let filename = output_dir.join("results.jsonl");
    let file = BufReader::new(File::open(filename)?);

    file.lines()
        .map(|line| serde_json::from_str(&line?))
        .collect()
}
```

---

## 5. Testing Strategy

### 5.1 Unit Testing

**Coverage Target:** 80%+

**Test Categories:**

1. **Dataset Loading Tests**
```rust
#[test]
fn test_load_json_dataset() {
    let dataset = DatasetLoader::load("tests/fixtures/sample.json").unwrap();
    assert_eq!(dataset.name, "sample");
    assert_eq!(dataset.test_cases.len(), 5);
}

#[test]
fn test_load_yaml_dataset() {
    let dataset = DatasetLoader::load("tests/fixtures/sample.yaml").unwrap();
    assert!(dataset.validate().is_ok());
}

#[test]
fn test_invalid_dataset() {
    let result = DatasetLoader::load("tests/fixtures/invalid.json");
    assert!(result.is_err());
}
```

2. **Template Rendering Tests**
```rust
#[test]
fn test_template_with_variables() {
    let mut vars = HashMap::new();
    vars.insert("lang".to_string(), "Rust".to_string());

    let result = TemplateEngine::render("Explain {{lang}}", &vars).unwrap();
    assert_eq!(result, "Explain Rust");
}

#[test]
fn test_template_missing_variable() {
    let vars = HashMap::new();
    let result = TemplateEngine::render("Explain {{lang}}", &vars);
    assert!(result.is_err());
}
```

3. **Benchmark Runner Tests**
```rust
#[tokio::test]
async fn test_concurrent_execution() {
    let config = BenchmarkConfig {
        concurrency: 3,
        ..Default::default()
    };

    let runner = BenchmarkRunner::new(config);
    let dataset = create_test_dataset(10);
    let provider = Arc::new(MockProvider::new());

    let results = runner.run(&dataset, provider).await.unwrap();
    assert_eq!(results.results.len(), 10);
}
```

4. **Result Aggregation Tests**
```rust
#[test]
fn test_calculate_summary() {
    let mut results = BenchmarkResults::new();
    results.results = vec![
        TestResult::success(...),
        TestResult::failure(...),
        TestResult::success(...),
    ];

    results.calculate_summary();
    assert_eq!(results.summary.success_rate, 0.666...);
}
```

### 5.2 Integration Testing

**Test Scenarios:**

1. **End-to-End Benchmark**
```rust
#[tokio::test]
#[ignore] // Requires API key
async fn test_e2e_benchmark_openai() {
    let dataset_path = "tests/fixtures/small-dataset.json";
    let config = ConfigLoader::new().load().unwrap();

    let args = BenchArgs {
        dataset: dataset_path.into(),
        providers: vec!["openai".to_string()],
        concurrency: 2,
        // ... other args
    };

    execute(args).await.unwrap();

    // Verify output files exist
    assert!(Path::new("./bench-results/results.json").exists());
    assert!(Path::new("./bench-results/results.csv").exists());
}
```

2. **Multi-Provider Test**
```rust
#[tokio::test]
async fn test_multi_provider_bench() {
    let dataset = create_small_dataset();

    for provider in &["openai", "anthropic"] {
        let results = run_benchmark(dataset, provider).await.unwrap();
        assert!(results.summary.total > 0);
    }
}
```

3. **Failure Handling Test**
```rust
#[tokio::test]
async fn test_continue_on_failure() {
    let config = BenchmarkConfig {
        continue_on_failure: true,
        ..Default::default()
    };

    // Dataset with one invalid test
    let dataset = create_mixed_dataset();

    let results = run_with_config(dataset, config).await.unwrap();
    assert!(results.summary.failed > 0);
    assert!(results.summary.succeeded > 0);
}
```

---

## 6. Risk Assessment

### 6.1 Technical Risks

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| **Rate limiting at scale** | High | High | Configurable delays, semaphore limits, retry logic |
| **Memory usage with large datasets** | Medium | Medium | Streaming results, incremental saving |
| **Concurrent execution bugs** | High | Low | Extensive async testing, Tokio best practices |
| **Dataset validation failures** | Medium | Medium | Comprehensive schema validation, clear error messages |
| **Result file corruption** | Medium | Low | Append-only writes, atomic operations |

### 6.2 Performance Risks

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| **Slow progress with large datasets** | Medium | High | Progress reporting, ETA display |
| **API timeout on large contexts** | Medium | Medium | Configurable timeouts, retry logic |
| **Disk space for saved responses** | Low | Medium | Optional response saving, compression |

### 6.3 Operational Risks

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| **Invalid dataset format** | High | Medium | Clear validation errors, example datasets |
| **Missing API keys** | High | Medium | Early validation, helpful error messages |
| **Interrupted benchmarks** | Medium | Medium | Incremental saving, resume capability |

---

## 7. Timeline and Resources

### 7.1 Detailed Timeline

```
Week 9: Dataset Management
├─ Day 1-2: Dataset schema + loader
├─ Day 3: Template engine
├─ Day 4-5: Built-in datasets + tests

Week 10: Benchmark Runner
├─ Day 6-7: Runner implementation
├─ Day 8: Progress reporting
├─ Day 9: Error handling
├─ Day 10: Testing

Week 11: Result Storage
├─ Day 11-12: Result schema + aggregation
├─ Day 13: CSV export
├─ Day 14: Incremental updates
├─ Day 15: Testing

Week 12: CLI Integration
├─ Day 16-17: Bench command implementation
├─ Day 18: Multi-provider support
├─ Day 19: Integration testing
├─ Day 20: Documentation + polish
```

### 7.2 Resource Allocation

| Agent | Milestones | Time Allocation |
|-------|-----------|-----------------|
| **Dataset Agent** | 3.1 | 5 days (25%) |
| **Runner Agent** | 3.2 | 5 days (25%) |
| **Storage Agent** | 3.3 | 5 days (25%) |
| **CLI Agent** | 3.4 | 5 days (25%) |
| **Testing Agent** | All | Parallel (ongoing) |

### 7.3 Dependencies

**Critical Path:**
```
3.1 (Dataset) → 3.2 (Runner) → 3.3 (Storage) → 3.4 (CLI)
```

All milestones are sequential due to dependencies.

---

## 8. Success Metrics

### 8.1 Functional Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| **Dataset Loading** | JSON + YAML | Integration tests |
| **Concurrent Execution** | 100+ requests | Performance tests |
| **Progress Accuracy** | Within 5% | Manual verification |
| **Result Accuracy** | 100% | Unit tests |

### 8.2 Performance Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| **Throughput** | 10+ tests/sec | Benchmark runs |
| **Memory Usage** | <500MB for 1000 tests | Profiling |
| **Result File Size** | <10KB per test | File analysis |
| **CSV Export Time** | <1s for 1000 tests | Timing |

### 8.3 Quality Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| **Code Coverage** | 80%+ | cargo-tarpaulin |
| **Unit Tests** | 60+ | Test suite |
| **Integration Tests** | 15+ | Test suite |
| **Documentation** | 100% public API | cargo doc |

---

## 9. Appendices

### Appendix A: Example Dataset Format

**JSON Format:**
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
      "prompt": "Write a {{language}} function to {{task}}",
      "variables": {
        "language": "Python",
        "task": "reverse a string"
      },
      "expected": "def reverse",
      "references": ["[::-1]", "reversed()"]
    }
  ]
}
```

**YAML Format:**
```yaml
name: example-dataset
description: Example benchmark dataset
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
```

### Appendix B: CLI Usage Examples

```bash
# Run benchmark with single provider
llm-test-bench bench \
  --dataset ./datasets/coding-tasks.json \
  --providers openai

# Run with multiple providers
llm-test-bench bench \
  --dataset ./datasets/reasoning-tasks.yaml \
  --providers openai,anthropic \
  --concurrency 10

# Custom output directory
llm-test-bench bench \
  --dataset ./my-dataset.json \
  --providers anthropic \
  --output ./my-results \
  --export json

# High concurrency
llm-test-bench bench \
  --dataset ./large-dataset.json \
  --providers openai \
  --concurrency 20 \
  --continue-on-failure
```

### Appendix C: Result File Formats

**JSON Results:**
```json
{
  "dataset_name": "coding-tasks",
  "provider_name": "openai",
  "total_tests": 10,
  "timestamp": "2025-11-04T12:00:00Z",
  "summary": {
    "total": 10,
    "succeeded": 9,
    "failed": 1,
    "success_rate": 0.9,
    "avg_duration_ms": 1234.5,
    "total_tokens": 4500,
    "total_cost": 0.023
  },
  "results": [...]
}
```

**CSV Results:**
```csv
test_id,category,status,duration_ms,tokens,cost,model
test-1,coding,Success,1234,450,0.0023,gpt-4
test-2,coding,Success,987,320,0.0016,gpt-4
```

### Appendix D: Dependencies

```toml
[workspace.dependencies]
# Existing dependencies from Phase 1 & 2
# ...

# Phase 3 additions
indicatif = "0.17"          # Progress bars
csv = "1.3"                 # CSV export
serde_yaml = "0.9"          # YAML parsing
regex = "1.10"              # Template engine

[dev-dependencies]
tempfile = "3.10"           # Temp files for tests
criterion = "0.5"           # Benchmarking
```

---

## Conclusion

Phase 3 represents the transformation of LLM Test Bench from a single-query testing tool to a comprehensive benchmarking platform. By the end of this phase, users will be able to:

✅ Load test datasets from JSON/YAML files
✅ Run benchmarks across multiple providers concurrently
✅ Process hundreds of tests efficiently
✅ Track progress with detailed reporting
✅ Export results for analysis in CSV/JSON format
✅ Compare provider performance systematically

The implementation plan is detailed, realistic, and builds on the solid foundation of Phases 1 and 2. With proper execution, Phase 3 will deliver a production-ready benchmarking system for LLM testing.

---

**Next Review:** End of Week 10 (Milestone 3.2 complete)
**Phase 3 Completion Target:** End of Week 12
**Status:** Ready to Begin Implementation

**Prepared by:** Phase 3 Coordinator
**Date:** November 4, 2025
**Version:** 1.0

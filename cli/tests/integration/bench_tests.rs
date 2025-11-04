// Integration tests for bench command
use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_bench_missing_dataset() {
    let mut cmd = Command::cargo_bin("llm-test-bench").unwrap();
    cmd.arg("bench")
        .arg("--dataset")
        .arg("nonexistent.json")
        .arg("--providers")
        .arg("openai");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Dataset file not found"));
}

#[test]
fn test_bench_missing_providers() {
    // Create a temporary dataset
    let temp_dir = TempDir::new().unwrap();
    let dataset_path = temp_dir.path().join("test.json");
    fs::write(
        &dataset_path,
        r#"{
            "name": "test",
            "version": "1.0.0",
            "test_cases": [
                {
                    "id": "test-1",
                    "prompt": "Hello"
                }
            ]
        }"#,
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("llm-test-bench").unwrap();
    cmd.arg("bench").arg("--dataset").arg(&dataset_path);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
fn test_bench_help() {
    let mut cmd = Command::cargo_bin("llm-test-bench").unwrap();
    cmd.arg("bench").arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Path to dataset file"))
        .stdout(predicate::str::contains("Providers to benchmark"));
}

#[test]
fn test_bench_invalid_export_format() {
    let temp_dir = TempDir::new().unwrap();
    let dataset_path = temp_dir.path().join("test.json");
    fs::write(
        &dataset_path,
        r#"{
            "name": "test",
            "version": "1.0.0",
            "test_cases": [
                {
                    "id": "test-1",
                    "prompt": "Hello"
                }
            ]
        }"#,
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("llm-test-bench").unwrap();
    cmd.arg("bench")
        .arg("--dataset")
        .arg(&dataset_path)
        .arg("--providers")
        .arg("openai")
        .arg("--export")
        .arg("invalid");

    cmd.assert().failure();
}

#[test]
fn test_bench_validates_concurrency() {
    let temp_dir = TempDir::new().unwrap();
    let dataset_path = temp_dir.path().join("test.json");
    fs::write(
        &dataset_path,
        r#"{
            "name": "test",
            "version": "1.0.0",
            "test_cases": [
                {
                    "id": "test-1",
                    "prompt": "Hello"
                }
            ]
        }"#,
    )
    .unwrap();

    // Test with concurrency 0 (should fail during benchmark config validation)
    let mut cmd = Command::cargo_bin("llm-test-bench").unwrap();
    cmd.arg("bench")
        .arg("--dataset")
        .arg(&dataset_path)
        .arg("--providers")
        .arg("fake-provider") // Use fake provider to avoid API calls
        .arg("--concurrency")
        .arg("0");

    // This will fail due to either clap validation or benchmark config validation
    cmd.assert().failure();
}

#[test]
fn test_dataset_loader_json() {
    use llm_test_bench_datasets::loader::DatasetLoader;
    use std::path::Path;

    let temp_dir = TempDir::new().unwrap();
    let dataset_path = temp_dir.path().join("test.json");

    fs::write(
        &dataset_path,
        r#"{
            "name": "test-dataset",
            "version": "1.0.0",
            "test_cases": [
                {
                    "id": "test-1",
                    "prompt": "Hello, world!"
                }
            ]
        }"#,
    )
    .unwrap();

    let loader = DatasetLoader::new();
    let dataset = loader.load(Path::new(&dataset_path)).unwrap();

    assert_eq!(dataset.name, "test-dataset");
    assert_eq!(dataset.test_cases.len(), 1);
    assert_eq!(dataset.test_cases[0].id, "test-1");
}

#[test]
fn test_dataset_loader_yaml() {
    use llm_test_bench_datasets::loader::DatasetLoader;
    use std::path::Path;

    let temp_dir = TempDir::new().unwrap();
    let dataset_path = temp_dir.path().join("test.yaml");

    fs::write(
        &dataset_path,
        r#"
name: test-dataset
version: 1.0.0
test_cases:
  - id: test-1
    prompt: Hello, world!
"#,
    )
    .unwrap();

    let loader = DatasetLoader::new();
    let dataset = loader.load(Path::new(&dataset_path)).unwrap();

    assert_eq!(dataset.name, "test-dataset");
    assert_eq!(dataset.test_cases.len(), 1);
}

#[test]
fn test_template_rendering() {
    use llm_test_bench_datasets::template::TemplateEngine;
    use std::collections::HashMap;

    let mut vars = HashMap::new();
    vars.insert("name".to_string(), "Alice".to_string());
    vars.insert("language".to_string(), "Rust".to_string());

    let template = "Hello, {{name}}! Welcome to {{language}} programming.";
    let rendered = TemplateEngine::render(template, &vars).unwrap();

    assert_eq!(
        rendered,
        "Hello, Alice! Welcome to Rust programming."
    );
}

#[test]
fn test_template_missing_variable() {
    use llm_test_bench_datasets::template::TemplateEngine;
    use std::collections::HashMap;

    let vars = HashMap::new();
    let template = "Hello, {{name}}!";
    let result = TemplateEngine::render(template, &vars);

    assert!(result.is_err());
}

#[test]
fn test_export_format_enum() {
    // Test that the export format enum variants are defined
    use llm_test_bench::commands::bench::ExportFormat;

    let _json = ExportFormat::Json;
    let _csv = ExportFormat::Csv;
    let _both = ExportFormat::Both;
}

#[cfg(test)]
mod benchmark_config_tests {
    use llm_test_bench_core::benchmarks::BenchmarkConfig;
    use std::path::PathBuf;

    #[test]
    fn test_config_default() {
        let config = BenchmarkConfig::default();
        assert_eq!(config.concurrency, 5);
        assert!(config.save_responses);
        assert!(config.continue_on_failure);
    }

    #[test]
    fn test_config_builder() {
        let config = BenchmarkConfig::new()
            .with_concurrency(10)
            .with_save_responses(false)
            .with_output_dir(PathBuf::from("./test"));

        assert_eq!(config.concurrency, 10);
        assert!(!config.save_responses);
        assert_eq!(config.output_dir, PathBuf::from("./test"));
    }

    #[test]
    fn test_config_validation_zero_concurrency() {
        let config = BenchmarkConfig::new().with_concurrency(0);
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_validation_valid() {
        let config = BenchmarkConfig::new().with_concurrency(5);
        assert!(config.validate().is_ok());
    }
}

#[cfg(test)]
mod result_tests {
    use llm_test_bench_core::benchmarks::runner::{
        BenchmarkResults, TestResult, TestStatus,
    };
    use llm_test_bench_core::providers::{CompletionResponse, FinishReason, TokenUsage};
    use chrono::Utc;
    use std::time::Duration;

    fn create_test_response() -> CompletionResponse {
        CompletionResponse {
            id: "test-123".to_string(),
            model: "test-model".to_string(),
            content: "Test content".to_string(),
            usage: TokenUsage::new(10, 5),
            finish_reason: FinishReason::Stop,
            created_at: Utc::now(),
        }
    }

    #[test]
    fn test_result_creation() {
        let result = TestResult::success(
            "test-1".to_string(),
            Some("category".to_string()),
            create_test_response(),
            Duration::from_millis(1000),
        );

        assert_eq!(result.test_id, "test-1");
        assert_eq!(result.status, TestStatus::Success);
        assert!(result.response.is_some());
        assert_eq!(result.duration_ms, 1000);
    }

    #[test]
    fn test_benchmark_results_summary() {
        let results = vec![
            TestResult::success(
                "test-1".to_string(),
                None,
                create_test_response(),
                Duration::from_millis(1000),
            ),
            TestResult::success(
                "test-2".to_string(),
                None,
                create_test_response(),
                Duration::from_millis(2000),
            ),
        ];

        let mut benchmark = BenchmarkResults::new(
            "dataset".to_string(),
            "provider".to_string(),
            results,
        );

        benchmark.calculate_summary();

        assert_eq!(benchmark.summary.total, 2);
        assert_eq!(benchmark.summary.succeeded, 2);
        assert_eq!(benchmark.summary.failed, 0);
        assert_eq!(benchmark.summary.success_rate, 1.0);
        assert_eq!(benchmark.summary.avg_duration_ms, 1500.0);
    }
}

#[test]
fn test_csv_exporter() {
    use llm_test_bench_core::benchmarks::runner::{BenchmarkResults, TestResult};
    use llm_test_bench_core::benchmarks::CsvExporter;
    use llm_test_bench_core::providers::{CompletionResponse, FinishReason, TokenUsage};
    use chrono::Utc;
    use std::time::Duration;

    let temp_dir = TempDir::new().unwrap();
    let csv_path = temp_dir.path().join("results.csv");

    let response = CompletionResponse {
        id: "test-123".to_string(),
        model: "gpt-4".to_string(),
        content: "Test".to_string(),
        usage: TokenUsage::new(10, 5),
        finish_reason: FinishReason::Stop,
        created_at: Utc::now(),
    };

    let results = vec![TestResult::success(
        "test-1".to_string(),
        Some("coding".to_string()),
        response,
        Duration::from_millis(1234),
    )];

    let mut benchmark =
        BenchmarkResults::new("dataset".to_string(), "provider".to_string(), results);
    benchmark.calculate_summary();

    CsvExporter::export_default(&benchmark, &csv_path).unwrap();

    assert!(csv_path.exists());

    let content = fs::read_to_string(&csv_path).unwrap();
    assert!(content.contains("test_id"));
    assert!(content.contains("test-1"));
    assert!(content.contains("coding"));
}

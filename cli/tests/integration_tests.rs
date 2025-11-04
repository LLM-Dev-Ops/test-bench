use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// Helper to get the CLI binary
fn cli() -> Command {
    Command::cargo_bin("llm-test-bench").unwrap()
}

/// Helper to create a temporary test dataset
fn create_test_dataset(dir: &TempDir) -> PathBuf {
    let dataset_path = dir.path().join("test_dataset.json");
    let dataset = r#"{
        "name": "Test Dataset",
        "description": "Test dataset for integration tests",
        "test_cases": [
            {
                "name": "test1",
                "prompt": "Hello, world!",
                "expected_output": "Hi there!",
                "metadata": {}
            }
        ]
    }"#;
    fs::write(&dataset_path, dataset).unwrap();
    dataset_path
}

/// Helper to create a temporary results file
fn create_test_results(dir: &TempDir) -> PathBuf {
    let results_path = dir.path().join("results.json");
    let results = r#"{
        "summary": {
            "total": 10,
            "succeeded": 9,
            "failed": 1,
            "avg_duration_ms": 250.5,
            "total_cost": 0.05
        },
        "results": [
            {
                "duration_ms": 250.0,
                "tokens_used": 100,
                "estimated_cost": 0.005
            }
        ]
    }"#;
    fs::write(&results_path, results).unwrap();
    results_path
}

// ============================================================================
// Basic CLI Tests
// ============================================================================

#[test]
fn test_cli_help() {
    cli()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("LLM Test Bench"))
        .stdout(predicate::str::contains("Commands:"));
}

#[test]
fn test_cli_version() {
    cli()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn test_cli_no_args() {
    cli()
        .assert()
        .failure()
        .stderr(predicate::str::contains("Usage:"));
}

// ============================================================================
// Compare Command Tests
// ============================================================================

#[test]
fn test_compare_help() {
    cli()
        .arg("compare")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Compare multiple models"));
}

#[test]
fn test_compare_missing_models() {
    cli()
        .arg("compare")
        .arg("--prompt")
        .arg("test")
        .assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
fn test_compare_missing_input() {
    cli()
        .arg("compare")
        .arg("--models")
        .arg("openai:gpt-4,anthropic:claude-3")
        .assert()
        .failure()
        .stderr(predicate::str::contains("prompt").or(predicate::str::contains("dataset")));
}

#[test]
fn test_compare_invalid_model_spec() {
    let temp_dir = TempDir::new().unwrap();
    let dataset = create_test_dataset(&temp_dir);

    cli()
        .arg("compare")
        .arg("--dataset")
        .arg(dataset.to_str().unwrap())
        .arg("--models")
        .arg("invalid-spec")
        .assert()
        .failure();
}

#[test]
fn test_compare_valid_args_structure() {
    // Test that the command accepts valid arguments structure (will fail on API call)
    cli()
        .arg("compare")
        .arg("--prompt")
        .arg("test prompt")
        .arg("--models")
        .arg("openai:gpt-4,anthropic:claude-3-opus")
        .arg("--output")
        .arg("table")
        .assert()
        .failure(); // Expected to fail without API keys, but args are valid
}

// ============================================================================
// Dashboard Command Tests
// ============================================================================

#[test]
fn test_dashboard_help() {
    cli()
        .arg("dashboard")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Generate interactive HTML dashboards"));
}

#[test]
fn test_dashboard_missing_results() {
    cli()
        .arg("dashboard")
        .arg("--output")
        .arg("test.html")
        .assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
fn test_dashboard_nonexistent_file() {
    cli()
        .arg("dashboard")
        .arg("--results")
        .arg("nonexistent.json")
        .arg("--output")
        .arg("test.html")
        .assert()
        .failure();
}

#[test]
fn test_dashboard_with_valid_results() {
    let temp_dir = TempDir::new().unwrap();
    let results = create_test_results(&temp_dir);
    let output = temp_dir.path().join("dashboard.html");

    cli()
        .arg("dashboard")
        .arg("--results")
        .arg(results.to_str().unwrap())
        .arg("--output")
        .arg(output.to_str().unwrap())
        .assert()
        .success();

    assert!(output.exists());
}

#[test]
fn test_dashboard_theme_option() {
    let temp_dir = TempDir::new().unwrap();
    let results = create_test_results(&temp_dir);
    let output = temp_dir.path().join("dashboard.html");

    cli()
        .arg("dashboard")
        .arg("--results")
        .arg(results.to_str().unwrap())
        .arg("--output")
        .arg(output.to_str().unwrap())
        .arg("--theme")
        .arg("dark")
        .assert()
        .success();
}

// ============================================================================
// Analyze Command Tests
// ============================================================================

#[test]
fn test_analyze_help() {
    cli()
        .arg("analyze")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("statistical analysis"));
}

#[test]
fn test_analyze_missing_baseline() {
    let temp_dir = TempDir::new().unwrap();
    let comparison = create_test_results(&temp_dir);

    cli()
        .arg("analyze")
        .arg("--comparison")
        .arg(comparison.to_str().unwrap())
        .assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
fn test_analyze_missing_comparison() {
    let temp_dir = TempDir::new().unwrap();
    let baseline = create_test_results(&temp_dir);

    cli()
        .arg("analyze")
        .arg("--baseline")
        .arg(baseline.to_str().unwrap())
        .assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
fn test_analyze_with_valid_files() {
    let temp_dir = TempDir::new().unwrap();
    let baseline = create_test_results(&temp_dir);
    let comparison = create_test_results(&temp_dir);

    cli()
        .arg("analyze")
        .arg("--baseline")
        .arg(baseline.to_str().unwrap())
        .arg("--comparison")
        .arg(comparison.to_str().unwrap())
        .assert()
        .success();
}

#[test]
fn test_analyze_invalid_confidence_level() {
    let temp_dir = TempDir::new().unwrap();
    let baseline = create_test_results(&temp_dir);
    let comparison = create_test_results(&temp_dir);

    cli()
        .arg("analyze")
        .arg("--baseline")
        .arg(baseline.to_str().unwrap())
        .arg("--comparison")
        .arg(comparison.to_str().unwrap())
        .arg("--confidence-level")
        .arg("1.5")
        .assert()
        .failure();
}

#[test]
fn test_analyze_output_format_json() {
    let temp_dir = TempDir::new().unwrap();
    let baseline = create_test_results(&temp_dir);
    let comparison = create_test_results(&temp_dir);

    cli()
        .arg("analyze")
        .arg("--baseline")
        .arg(baseline.to_str().unwrap())
        .arg("--comparison")
        .arg(comparison.to_str().unwrap())
        .arg("--output")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("{"));
}

// ============================================================================
// Optimize Command Tests
// ============================================================================

#[test]
fn test_optimize_help() {
    cli()
        .arg("optimize")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("cost-optimized"));
}

#[test]
fn test_optimize_missing_model() {
    cli()
        .arg("optimize")
        .arg("--monthly-requests")
        .arg("1000")
        .assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
fn test_optimize_missing_requests() {
    cli()
        .arg("optimize")
        .arg("--current-model")
        .arg("gpt-4")
        .assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
fn test_optimize_with_valid_args() {
    cli()
        .arg("optimize")
        .arg("--current-model")
        .arg("gpt-4")
        .arg("--monthly-requests")
        .arg("10000")
        .assert()
        .success();
}

#[test]
fn test_optimize_quality_threshold() {
    cli()
        .arg("optimize")
        .arg("--current-model")
        .arg("gpt-4")
        .arg("--monthly-requests")
        .arg("10000")
        .arg("--quality-threshold")
        .arg("0.8")
        .assert()
        .success();
}

#[test]
fn test_optimize_invalid_quality_threshold() {
    cli()
        .arg("optimize")
        .arg("--current-model")
        .arg("gpt-4")
        .arg("--monthly-requests")
        .arg("10000")
        .arg("--quality-threshold")
        .arg("2.0")
        .assert()
        .failure();
}

#[test]
fn test_optimize_output_format_json() {
    cli()
        .arg("optimize")
        .arg("--current-model")
        .arg("gpt-4")
        .arg("--monthly-requests")
        .arg("10000")
        .arg("--output")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("{"));
}

// ============================================================================
// Config Command Tests
// ============================================================================

#[test]
fn test_config_help() {
    cli()
        .arg("config")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Configuration management"));
}

#[test]
fn test_config_show() {
    cli()
        .arg("config")
        .arg("show")
        .assert()
        .success();
}

#[test]
fn test_config_validate() {
    cli()
        .arg("config")
        .arg("validate")
        .assert()
        .success();
}

// ============================================================================
// Global Options Tests
// ============================================================================

#[test]
fn test_global_verbose_flag() {
    cli()
        .arg("--verbose")
        .arg("config")
        .arg("show")
        .assert()
        .success();
}

#[test]
fn test_global_no_color_flag() {
    cli()
        .arg("--no-color")
        .arg("config")
        .arg("show")
        .assert()
        .success();
}

// ============================================================================
// Completions Tests
// ============================================================================

#[test]
fn test_completions_bash() {
    cli()
        .arg("completions")
        .arg("bash")
        .assert()
        .success()
        .stdout(predicate::str::contains("llm-test-bench"));
}

#[test]
fn test_completions_zsh() {
    cli()
        .arg("completions")
        .arg("zsh")
        .assert()
        .success()
        .stdout(predicate::str::contains("llm-test-bench"));
}

#[test]
fn test_completions_fish() {
    cli()
        .arg("completions")
        .arg("fish")
        .assert()
        .success()
        .stdout(predicate::str::contains("llm-test-bench"));
}

// ============================================================================
// Command Aliases Tests
// ============================================================================

#[test]
fn test_compare_alias() {
    cli()
        .arg("c")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Compare"));
}

#[test]
fn test_dashboard_alias() {
    cli()
        .arg("d")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("dashboard"));
}

#[test]
fn test_analyze_alias() {
    cli()
        .arg("a")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("analyze"));
}

#[test]
fn test_optimize_alias() {
    cli()
        .arg("o")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("optimize"));
}

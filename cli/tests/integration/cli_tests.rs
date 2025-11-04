use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

/// Helper function to create a test command
fn cli() -> Command {
    Command::cargo_bin("llm-test-bench").unwrap()
}

#[test]
fn test_help_command() {
    cli()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("production-grade CLI"))
        .stdout(predicate::str::contains("Usage:"))
        .stdout(predicate::str::contains("Commands:"));
}

#[test]
fn test_version_command() {
    cli()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("llm-test-bench"));
}

#[test]
fn test_no_args_shows_help() {
    cli()
        .assert()
        .failure()
        .stderr(predicate::str::contains("Usage:"));
}

#[test]
fn test_test_command_help() {
    cli()
        .arg("test")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Run a single test"))
        .stdout(predicate::str::contains("--prompt"))
        .stdout(predicate::str::contains("--model"));
}

#[test]
fn test_test_command_missing_args() {
    cli()
        .arg("test")
        .assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
fn test_test_command_basic_execution() {
    cli()
        .arg("test")
        .arg("openai")
        .arg("--prompt")
        .arg("Hello, world!")
        .assert()
        .success()
        .stdout(predicate::str::contains("Coming in Phase 2"));
}

#[test]
fn test_test_command_with_model() {
    cli()
        .arg("test")
        .arg("openai")
        .arg("--prompt")
        .arg("Test prompt")
        .arg("--model")
        .arg("gpt-4")
        .assert()
        .success()
        .stdout(predicate::str::contains("Model: gpt-4"));
}

#[test]
fn test_test_command_invalid_temperature() {
    cli()
        .arg("test")
        .arg("openai")
        .arg("--prompt")
        .arg("Test")
        .arg("--temperature")
        .arg("5.0")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Temperature must be between"));
}

#[test]
fn test_bench_command_help() {
    cli()
        .arg("bench")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("benchmark"))
        .stdout(predicate::str::contains("--dataset"))
        .stdout(predicate::str::contains("--providers"));
}

#[test]
fn test_bench_command_missing_dataset() {
    cli()
        .arg("bench")
        .arg("--dataset")
        .arg("nonexistent.json")
        .arg("--providers")
        .arg("openai")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Dataset file not found"));
}

#[test]
fn test_bench_command_no_providers() {
    // Create a temporary file
    let temp_dir = TempDir::new().unwrap();
    let dataset_path = temp_dir.path().join("dataset.json");
    fs::write(&dataset_path, "[]").unwrap();

    // Test missing the required --providers argument entirely
    cli()
        .arg("bench")
        .arg("--dataset")
        .arg(dataset_path.to_str().unwrap())
        .assert()
        .failure()
        .stderr(predicate::str::contains("At least one provider"));
}

#[test]
fn test_bench_command_basic_execution() {
    // Create a temporary dataset file
    let temp_dir = TempDir::new().unwrap();
    let dataset_path = temp_dir.path().join("dataset.json");
    fs::write(&dataset_path, r#"[{"name": "test", "prompt": "Hello"}]"#).unwrap();

    cli()
        .arg("bench")
        .arg("--dataset")
        .arg(dataset_path.to_str().unwrap())
        .arg("--providers")
        .arg("openai,anthropic")
        .assert()
        .success()
        .stdout(predicate::str::contains("Coming in Phase 3"))
        .stdout(predicate::str::contains("Providers: openai, anthropic"));
}

#[test]
fn test_eval_command_help() {
    cli()
        .arg("eval")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Evaluate"))
        .stdout(predicate::str::contains("--results"))
        .stdout(predicate::str::contains("--metrics"));
}

#[test]
fn test_eval_command_missing_results() {
    cli()
        .arg("eval")
        .arg("--results")
        .arg("nonexistent.json")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Results file not found"));
}

#[test]
fn test_eval_command_basic_execution() {
    // Create a temporary results file
    let temp_dir = TempDir::new().unwrap();
    let results_path = temp_dir.path().join("results.json");
    fs::write(&results_path, r#"{"tests": []}"#).unwrap();

    cli()
        .arg("eval")
        .arg("--results")
        .arg(results_path.to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("Coming in Phase 4"));
}

#[test]
fn test_eval_command_with_baseline() {
    let temp_dir = TempDir::new().unwrap();
    let results_path = temp_dir.path().join("results.json");
    let baseline_path = temp_dir.path().join("baseline.json");

    fs::write(&results_path, r#"{"tests": []}"#).unwrap();
    fs::write(&baseline_path, r#"{"tests": []}"#).unwrap();

    cli()
        .arg("eval")
        .arg("--results")
        .arg(results_path.to_str().unwrap())
        .arg("--baseline")
        .arg(baseline_path.to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("Baseline comparison"));
}

#[test]
fn test_eval_command_invalid_threshold() {
    let temp_dir = TempDir::new().unwrap();
    let results_path = temp_dir.path().join("results.json");
    fs::write(&results_path, r#"{"tests": []}"#).unwrap();

    cli()
        .arg("eval")
        .arg("--results")
        .arg(results_path.to_str().unwrap())
        .arg("--threshold")
        .arg("1.5")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Threshold must be between"));
}

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
fn test_config_init_help() {
    cli()
        .arg("config")
        .arg("init")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Initialize configuration"))
        .stdout(predicate::str::contains("--non-interactive"));
}

#[test]
fn test_config_show_no_config() {
    // This test assumes no config exists in the test environment
    // In a real scenario, we'd mock the config directory
    cli()
        .arg("config")
        .arg("show")
        .assert()
        .success()
        .stdout(predicate::str::contains("configuration").or(predicate::str::contains("Configuration")));
}

#[test]
fn test_verbose_flag() {
    cli()
        .arg("--verbose")
        .arg("test")
        .arg("openai")
        .arg("--prompt")
        .arg("Test")
        .assert()
        .success()
        .stdout(predicate::str::contains("Arguments received:"));
}

#[test]
fn test_command_aliases() {
    // Test that aliases work
    cli()
        .arg("t")
        .arg("openai")
        .arg("--prompt")
        .arg("Test")
        .assert()
        .success();

    cli()
        .arg("b")
        .arg("--help")
        .assert()
        .success();

    cli()
        .arg("e")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_completions_command() {
    cli()
        .arg("completions")
        .arg("bash")
        .assert()
        .success()
        .stdout(predicate::str::contains("llm-test-bench"));
}

#[test]
fn test_global_no_color_flag() {
    cli()
        .arg("--no-color")
        .arg("test")
        .arg("openai")
        .arg("--prompt")
        .arg("Test")
        .assert()
        .success();
}

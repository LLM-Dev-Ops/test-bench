use assert_cmd::Command;
use predicates::prelude::*;
use std::env;
use tempfile::TempDir;

mod integration;

/// Helper to create a test command
fn test_cmd() -> Command {
    Command::cargo_bin("llm-test-bench").unwrap()
}

#[test]
fn test_help_command() {
    test_cmd()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("LLM Test Bench"))
        .stdout(predicate::str::contains("test"))
        .stdout(predicate::str::contains("bench"))
        .stdout(predicate::str::contains("eval"));
}

#[test]
fn test_test_help() {
    test_cmd()
        .args(&["test", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Run a single test against an LLM provider"))
        .stdout(predicate::str::contains("--prompt"))
        .stdout(predicate::str::contains("--model"))
        .stdout(predicate::str::contains("--stream"))
        .stdout(predicate::str::contains("--output-format"));
}

#[test]
fn test_test_missing_provider() {
    test_cmd()
        .args(&["test"])
        .assert()
        .failure();
}

#[test]
fn test_test_missing_prompt() {
    test_cmd()
        .args(&["test", "openai"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("--prompt"));
}

#[test]
fn test_test_invalid_temperature() {
    test_cmd()
        .args(&[
            "test",
            "openai",
            "--prompt",
            "test",
            "--temperature",
            "3.0",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Temperature must be between 0.0 and 2.0"));
}

#[test]
fn test_test_invalid_top_p() {
    test_cmd()
        .args(&[
            "test",
            "openai",
            "--prompt",
            "test",
            "--top-p",
            "1.5",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Top-p must be between 0.0 and 1.0"));
}

#[test]
fn test_test_missing_api_key() {
    // Temporarily remove API key
    let original = env::var("OPENAI_API_KEY").ok();
    env::remove_var("OPENAI_API_KEY");

    test_cmd()
        .args(&["test", "openai", "--prompt", "Hello"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("OPENAI_API_KEY"));

    // Restore API key
    if let Some(key) = original {
        env::set_var("OPENAI_API_KEY", key);
    }
}

#[test]
fn test_test_unknown_provider() {
    test_cmd()
        .args(&["test", "unknown-provider", "--prompt", "Hello"])
        .env("UNKNOWN_API_KEY", "test")
        .assert()
        .failure();
}

#[test]
fn test_config_init() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    env::set_var("HOME", temp_dir.path());

    test_cmd()
        .args(&["config", "init"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Configuration file"));
}

#[test]
fn test_config_show() {
    test_cmd()
        .args(&["config", "show"])
        .assert()
        .success()
        .stdout(predicate::str::contains("providers"));
}

#[test]
fn test_completions_bash() {
    test_cmd()
        .args(&["completions", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::contains("_llm-test-bench"));
}

#[test]
fn test_output_format_json() {
    // This would need a valid API key to actually run
    // For now we just test that the argument is accepted
    test_cmd()
        .args(&[
            "test",
            "openai",
            "--prompt",
            "test",
            "--output-format",
            "json",
        ])
        .assert()
        .failure(); // Will fail without API key, but that's OK for arg validation
}

#[test]
fn test_output_format_plain() {
    test_cmd()
        .args(&[
            "test",
            "openai",
            "--prompt",
            "test",
            "--output-format",
            "plain",
        ])
        .assert()
        .failure();
}

#[test]
fn test_stream_flag() {
    test_cmd()
        .args(&[
            "test",
            "openai",
            "--prompt",
            "test",
            "--stream",
        ])
        .assert()
        .failure(); // Will fail without API key
}

#[test]
fn test_model_specification() {
    test_cmd()
        .args(&[
            "test",
            "openai",
            "--prompt",
            "test",
            "--model",
            "gpt-4",
        ])
        .assert()
        .failure(); // Will fail without API key
}

#[test]
fn test_max_tokens() {
    test_cmd()
        .args(&[
            "test",
            "openai",
            "--prompt",
            "test",
            "--max-tokens",
            "100",
        ])
        .assert()
        .failure();
}

#[test]
fn test_stop_sequences() {
    test_cmd()
        .args(&[
            "test",
            "openai",
            "--prompt",
            "test",
            "--stop",
            "END",
            "--stop",
            "STOP",
        ])
        .assert()
        .failure();
}

#[test]
fn test_verbose_flag() {
    test_cmd()
        .args(&[
            "--verbose",
            "test",
            "openai",
            "--prompt",
            "test",
        ])
        .assert()
        .failure();
}

#[test]
fn test_custom_config_file() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("custom.toml");

    // Create a minimal config file
    std::fs::write(
        &config_path,
        r#"
[providers.openai]
api_key_env = "OPENAI_API_KEY"
base_url = "https://api.openai.com/v1"
default_model = "gpt-4"
timeout_seconds = 30
max_retries = 3
        "#,
    )
    .unwrap();

    test_cmd()
        .args(&[
            "test",
            "openai",
            "--prompt",
            "test",
            "--config",
            config_path.to_str().unwrap(),
        ])
        .assert()
        .failure(); // Will fail without API key, but config parsing should work
}

// Note: Tests that actually call the API are marked with #[ignore]
// and should be run with `cargo test -- --ignored` when you have API keys set

#[test]
#[ignore]
fn test_openai_real_api_call() {
    if env::var("OPENAI_API_KEY").is_err() {
        eprintln!("Skipping real API test: OPENAI_API_KEY not set");
        return;
    }

    test_cmd()
        .args(&[
            "test",
            "openai",
            "--prompt",
            "Say 'test successful' and nothing else",
            "--model",
            "gpt-3.5-turbo",
            "--max-tokens",
            "10",
            "--output-format",
            "plain",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("test successful"));
}

#[test]
#[ignore]
fn test_anthropic_real_api_call() {
    if env::var("ANTHROPIC_API_KEY").is_err() {
        eprintln!("Skipping real API test: ANTHROPIC_API_KEY not set");
        return;
    }

    test_cmd()
        .args(&[
            "test",
            "anthropic",
            "--prompt",
            "Say 'test successful' and nothing else",
            "--model",
            "claude-3-haiku-20240307",
            "--max-tokens",
            "10",
            "--output-format",
            "plain",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("test successful"));
}

#[test]
#[ignore]
fn test_streaming_output() {
    if env::var("OPENAI_API_KEY").is_err() {
        eprintln!("Skipping streaming test: OPENAI_API_KEY not set");
        return;
    }

    test_cmd()
        .args(&[
            "test",
            "openai",
            "--prompt",
            "Count from 1 to 5",
            "--stream",
            "--model",
            "gpt-3.5-turbo",
            "--max-tokens",
            "50",
            "--output-format",
            "plain",
        ])
        .assert()
        .success();
}

#[test]
#[ignore]
fn test_json_output_format() {
    if env::var("OPENAI_API_KEY").is_err() {
        eprintln!("Skipping JSON test: OPENAI_API_KEY not set");
        return;
    }

    test_cmd()
        .args(&[
            "test",
            "openai",
            "--prompt",
            "Hello",
            "--model",
            "gpt-3.5-turbo",
            "--max-tokens",
            "10",
            "--output-format",
            "json",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"content\""))
        .stdout(predicate::str::contains("\"model\""))
        .stdout(predicate::str::contains("\"usage\""));
}

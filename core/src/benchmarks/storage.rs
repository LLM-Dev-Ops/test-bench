// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Incremental result storage using JSON Lines (JSONL) format.
//!
//! This module provides functionality to save benchmark results incrementally
//! as tests complete, allowing for:
//! - Resume capability if benchmark is interrupted
//! - Memory efficiency for large benchmark runs
//! - Real-time result tracking
//! - Easy merging of multiple result files
//!
//! # Format
//!
//! Results are stored in JSON Lines format, where each line is a complete
//! JSON object representing a single test result:
//!
//! ```jsonl
//! {"test_id":"test-1","status":"success","duration_ms":1234,...}
//! {"test_id":"test-2","status":"success","duration_ms":987,...}
//! {"test_id":"test-3","status":"failure","error":"Rate limit",...}
//! ```
//!
//! # Examples
//!
//! ```no_run
//! use llm_test_bench_core::benchmarks::{ResultStorage, TestResult};
//! use std::path::Path;
//!
//! # fn example(result: TestResult) -> Result<(), Box<dyn std::error::Error>> {
//! // Save result incrementally (appends to file)
//! ResultStorage::save_incremental(&result, Path::new("results.jsonl"))?;
//!
//! // Load all results
//! let results = ResultStorage::load_incremental(Path::new("results.jsonl"))?;
//! println!("Loaded {} results", results.len());
//! # Ok(())
//! # }
//! ```

use super::runner::{BenchmarkResults, TestResult};
use anyhow::{Context, Result};
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

/// Storage handler for incremental result saving.
///
/// Provides methods to save and load benchmark results in JSON Lines format.
pub struct ResultStorage;

impl ResultStorage {
    /// Saves a single test result to a JSONL file (append-only).
    ///
    /// This method appends the result to the specified file. If the file
    /// doesn't exist, it will be created.
    ///
    /// # Arguments
    ///
    /// * `result` - The test result to save
    /// * `path` - Path to the JSONL file
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be opened or written to.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use llm_test_bench_core::benchmarks::{ResultStorage, TestResult};
    /// # use std::path::Path;
    /// # fn example(result: TestResult) -> Result<(), Box<dyn std::error::Error>> {
    /// ResultStorage::save_incremental(&result, Path::new("results.jsonl"))?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn save_incremental(result: &TestResult, path: &Path) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .context("Failed to create output directory")?;
        }

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .context("Failed to open result file for writing")?;

        let json = serde_json::to_string(result)
            .context("Failed to serialize test result")?;

        writeln!(file, "{}", json)
            .context("Failed to write result to file")?;

        Ok(())
    }

    /// Loads all test results from a JSONL file.
    ///
    /// Reads the file line by line, parsing each line as a JSON object.
    /// Invalid lines are skipped with a warning.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the JSONL file
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be opened or read.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use llm_test_bench_core::benchmarks::ResultStorage;
    /// # use std::path::Path;
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let results = ResultStorage::load_incremental(Path::new("results.jsonl"))?;
    /// println!("Loaded {} test results", results.len());
    /// # Ok(())
    /// # }
    /// ```
    pub fn load_incremental(path: &Path) -> Result<Vec<TestResult>> {
        let file = File::open(path)
            .context("Failed to open result file for reading")?;

        let reader = BufReader::new(file);
        let mut results = Vec::new();

        for (line_num, line) in reader.lines().enumerate() {
            let line = line.context("Failed to read line from file")?;

            // Skip empty lines
            if line.trim().is_empty() {
                continue;
            }

            match serde_json::from_str::<TestResult>(&line) {
                Ok(result) => results.push(result),
                Err(e) => {
                    tracing::warn!(
                        "Failed to parse result on line {}: {}. Skipping.",
                        line_num + 1,
                        e
                    );
                }
            }
        }

        Ok(results)
    }

    /// Merges multiple JSONL result files into a vector of test results.
    ///
    /// This is useful for combining results from multiple benchmark runs or
    /// parallel executions.
    ///
    /// # Arguments
    ///
    /// * `paths` - Paths to JSONL files to merge
    ///
    /// # Errors
    ///
    /// Returns an error if any file cannot be read.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use llm_test_bench_core::benchmarks::ResultStorage;
    /// # use std::path::PathBuf;
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let paths = vec![
    ///     PathBuf::from("run1.jsonl"),
    ///     PathBuf::from("run2.jsonl"),
    /// ];
    ///
    /// let merged = ResultStorage::merge_results(&paths)?;
    ///
    /// println!("Merged {} total results", merged.len());
    /// # Ok(())
    /// # }
    /// ```
    pub fn merge_results(paths: &[PathBuf]) -> Result<Vec<TestResult>> {
        let mut all_results = Vec::new();

        for path in paths {
            let results = Self::load_incremental(path)
                .with_context(|| format!("Failed to load results from {}", path.display()))?;
            all_results.extend(results);
        }

        Ok(all_results)
    }

    /// Saves complete benchmark results to a JSON file (not JSONL).
    ///
    /// This saves the entire `BenchmarkResults` structure including metadata
    /// and summary statistics to a single JSON file.
    ///
    /// # Arguments
    ///
    /// * `results` - The benchmark results to save
    /// * `path` - Path where the JSON file should be written
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be created or written to.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use llm_test_bench_core::benchmarks::{ResultStorage, BenchmarkResults};
    /// # use std::path::Path;
    /// # fn example(results: BenchmarkResults) -> Result<(), Box<dyn std::error::Error>> {
    /// ResultStorage::save_json(&results, Path::new("results.json"))?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn save_json(results: &BenchmarkResults, path: &Path) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .context("Failed to create output directory")?;
        }

        let json = serde_json::to_string_pretty(results)
            .context("Failed to serialize benchmark results")?;

        std::fs::write(path, json)
            .context("Failed to write JSON file")?;

        Ok(())
    }

    /// Loads complete benchmark results from a JSON file.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the JSON file
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or parsed.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use llm_test_bench_core::benchmarks::ResultStorage;
    /// # use std::path::Path;
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let results = ResultStorage::load_json(Path::new("results.json"))?;
    /// println!("Loaded benchmark: {}", results.dataset_name);
    /// # Ok(())
    /// # }
    /// ```
    pub fn load_json(path: &Path) -> Result<BenchmarkResults> {
        let content = std::fs::read_to_string(path)
            .context("Failed to read JSON file")?;

        let results = serde_json::from_str(&content)
            .context("Failed to parse benchmark results")?;

        Ok(results)
    }

    /// Loads results from a JSONL file and appends them to an existing vector.
    ///
    /// This is useful for resuming interrupted benchmarks.
    ///
    /// # Arguments
    ///
    /// * `results` - Existing vector of results to append to
    /// * `path` - Path to the JSONL file containing additional results
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read.
    pub fn append_from_file(results: &mut Vec<TestResult>, path: &Path) -> Result<()> {
        let new_results = Self::load_incremental(path)?;
        results.extend(new_results);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::benchmarks::runner::TestStatus;
    use crate::providers::{CompletionResponse, FinishReason, TokenUsage};
    use chrono::Utc;
    use std::time::Duration;
    use tempfile::TempDir;

    fn create_test_response() -> CompletionResponse {
        CompletionResponse {
            id: "test-resp".to_string(),
            model: "gpt-4".to_string(),
            content: "Test content".to_string(),
            usage: TokenUsage::new(50, 25),
            finish_reason: FinishReason::Stop,
            created_at: Utc::now(),
        }
    }

    fn create_test_result(id: &str) -> TestResult {
        TestResult::success(
            id.to_string(),
            Some("test-category".to_string()),
            create_test_response(),
            Duration::from_millis(1000),
        )
    }

    #[test]
    fn test_save_incremental() {
        let temp_dir = TempDir::new().unwrap();
        let jsonl_path = temp_dir.path().join("results.jsonl");

        let result = create_test_result("test-1");

        ResultStorage::save_incremental(&result, &jsonl_path).unwrap();

        assert!(jsonl_path.exists());
    }

    #[test]
    fn test_save_incremental_appends() {
        let temp_dir = TempDir::new().unwrap();
        let jsonl_path = temp_dir.path().join("results.jsonl");

        // Save three results
        ResultStorage::save_incremental(&create_test_result("test-1"), &jsonl_path).unwrap();
        ResultStorage::save_incremental(&create_test_result("test-2"), &jsonl_path).unwrap();
        ResultStorage::save_incremental(&create_test_result("test-3"), &jsonl_path).unwrap();

        // Check file has 3 lines
        let content = std::fs::read_to_string(&jsonl_path).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 3);
    }

    #[test]
    fn test_load_incremental() {
        let temp_dir = TempDir::new().unwrap();
        let jsonl_path = temp_dir.path().join("results.jsonl");

        // Save multiple results
        ResultStorage::save_incremental(&create_test_result("test-1"), &jsonl_path).unwrap();
        ResultStorage::save_incremental(&create_test_result("test-2"), &jsonl_path).unwrap();

        // Load them back
        let results = ResultStorage::load_incremental(&jsonl_path).unwrap();

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].test_id, "test-1");
        assert_eq!(results[1].test_id, "test-2");
    }

    #[test]
    fn test_load_incremental_empty_file() {
        let temp_dir = TempDir::new().unwrap();
        let jsonl_path = temp_dir.path().join("empty.jsonl");

        // Create empty file
        File::create(&jsonl_path).unwrap();

        let results = ResultStorage::load_incremental(&jsonl_path).unwrap();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_load_incremental_skips_invalid_lines() {
        let temp_dir = TempDir::new().unwrap();
        let jsonl_path = temp_dir.path().join("results.jsonl");

        // Write mixed valid and invalid lines
        let mut file = File::create(&jsonl_path).unwrap();
        writeln!(
            file,
            "{}",
            serde_json::to_string(&create_test_result("test-1")).unwrap()
        )
        .unwrap();
        writeln!(file, "{{invalid json}}").unwrap();
        writeln!(
            file,
            "{}",
            serde_json::to_string(&create_test_result("test-2")).unwrap()
        )
        .unwrap();

        let results = ResultStorage::load_incremental(&jsonl_path).unwrap();

        // Should load only valid results
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].test_id, "test-1");
        assert_eq!(results[1].test_id, "test-2");
    }

    #[test]
    fn test_save_and_load_roundtrip() {
        let temp_dir = TempDir::new().unwrap();
        let jsonl_path = temp_dir.path().join("results.jsonl");

        let original = create_test_result("test-roundtrip");
        ResultStorage::save_incremental(&original, &jsonl_path).unwrap();

        let loaded = ResultStorage::load_incremental(&jsonl_path).unwrap();

        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].test_id, original.test_id);
        assert_eq!(loaded[0].status, original.status);
        assert_eq!(loaded[0].duration_ms, original.duration_ms);
    }

    #[test]
    fn test_merge_results() {
        let temp_dir = TempDir::new().unwrap();
        let file1 = temp_dir.path().join("results1.jsonl");
        let file2 = temp_dir.path().join("results2.jsonl");

        // Save to first file
        ResultStorage::save_incremental(&create_test_result("test-1"), &file1).unwrap();
        ResultStorage::save_incremental(&create_test_result("test-2"), &file1).unwrap();

        // Save to second file
        ResultStorage::save_incremental(&create_test_result("test-3"), &file2).unwrap();

        // Merge
        let merged = ResultStorage::merge_results(&[file1, file2]).unwrap();

        assert_eq!(merged.len(), 3);
    }

    #[test]
    fn test_save_json() {
        let temp_dir = TempDir::new().unwrap();
        let json_path = temp_dir.path().join("results.json");

        let results = vec![create_test_result("test-1"), create_test_result("test-2")];
        let mut benchmark =
            BenchmarkResults::new("dataset".to_string(), "provider".to_string(), results);
        benchmark.calculate_summary();

        ResultStorage::save_json(&benchmark, &json_path).unwrap();

        assert!(json_path.exists());
    }

    #[test]
    fn test_load_json() {
        let temp_dir = TempDir::new().unwrap();
        let json_path = temp_dir.path().join("results.json");

        let results = vec![create_test_result("test-1")];
        let mut original =
            BenchmarkResults::new("my-dataset".to_string(), "openai".to_string(), results);
        original.calculate_summary();

        ResultStorage::save_json(&original, &json_path).unwrap();

        let loaded = ResultStorage::load_json(&json_path).unwrap();

        assert_eq!(loaded.dataset_name, "my-dataset");
        assert_eq!(loaded.provider_name, "openai");
        assert_eq!(loaded.total_tests, 1);
    }

    #[test]
    fn test_json_roundtrip() {
        let temp_dir = TempDir::new().unwrap();
        let json_path = temp_dir.path().join("results.json");

        let results = vec![
            create_test_result("test-1"),
            TestResult::failure(
                "test-2".to_string(),
                None,
                "Error".to_string(),
                Duration::from_millis(100),
            ),
        ];
        let mut original =
            BenchmarkResults::new("dataset".to_string(), "provider".to_string(), results);
        original.calculate_summary();

        ResultStorage::save_json(&original, &json_path).unwrap();
        let loaded = ResultStorage::load_json(&json_path).unwrap();

        assert_eq!(loaded.dataset_name, original.dataset_name);
        assert_eq!(loaded.summary.total, original.summary.total);
        assert_eq!(loaded.summary.succeeded, original.summary.succeeded);
        assert_eq!(loaded.summary.failed, original.summary.failed);
    }

    #[test]
    fn test_append_from_file() {
        let temp_dir = TempDir::new().unwrap();
        let jsonl_path = temp_dir.path().join("additional.jsonl");

        // Create initial results
        let mut results = vec![create_test_result("test-1")];

        // Save additional results to file
        ResultStorage::save_incremental(&create_test_result("test-2"), &jsonl_path).unwrap();
        ResultStorage::save_incremental(&create_test_result("test-3"), &jsonl_path).unwrap();

        // Append from file
        ResultStorage::append_from_file(&mut results, &jsonl_path).unwrap();

        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_creates_parent_directory() {
        let temp_dir = TempDir::new().unwrap();
        let nested_path = temp_dir.path().join("subdir").join("nested").join("results.jsonl");

        ResultStorage::save_incremental(&create_test_result("test-1"), &nested_path).unwrap();

        assert!(nested_path.exists());
    }

    #[test]
    fn test_different_status_types() {
        let temp_dir = TempDir::new().unwrap();
        let jsonl_path = temp_dir.path().join("results.jsonl");

        let results = vec![
            TestResult::success(
                "test-success".to_string(),
                None,
                create_test_response(),
                Duration::from_millis(100),
            ),
            TestResult::failure(
                "test-failure".to_string(),
                None,
                "Failed".to_string(),
                Duration::from_millis(50),
            ),
            TestResult::timeout("test-timeout".to_string(), None, Duration::from_secs(30)),
            TestResult::skipped("test-skipped".to_string(), None),
        ];

        for result in &results {
            ResultStorage::save_incremental(result, &jsonl_path).unwrap();
        }

        let loaded = ResultStorage::load_incremental(&jsonl_path).unwrap();

        assert_eq!(loaded.len(), 4);
        assert_eq!(loaded[0].status, TestStatus::Success);
        assert_eq!(loaded[1].status, TestStatus::Failure);
        assert_eq!(loaded[2].status, TestStatus::Timeout);
        assert_eq!(loaded[3].status, TestStatus::Skipped);
    }

    #[test]
    fn test_merge_empty_files() {
        let temp_dir = TempDir::new().unwrap();
        let file1 = temp_dir.path().join("empty1.jsonl");
        let file2 = temp_dir.path().join("empty2.jsonl");

        File::create(&file1).unwrap();
        File::create(&file2).unwrap();

        let merged = ResultStorage::merge_results(&[file1, file2]).unwrap();

        assert_eq!(merged.len(), 0);
    }
}

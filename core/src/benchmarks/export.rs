// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! CSV export functionality for benchmark results.
//!
//! This module provides utilities to export benchmark results to CSV format
//! for analysis in spreadsheet applications, data visualization tools, or
//! statistical software.
//!
//! # Examples
//!
//! ```no_run
//! use llm_test_bench_core::benchmarks::{BenchmarkResults, CsvExporter};
//! use std::path::Path;
//!
//! # fn example(results: BenchmarkResults) -> Result<(), Box<dyn std::error::Error>> {
//! // Export results to CSV
//! CsvExporter::export(&results, Path::new("results.csv"))?;
//!
//! // Export with custom delimiter
//! CsvExporter::new()
//!     .with_delimiter(b'\t')
//!     .export(&results, Path::new("results.tsv"))?;
//! # Ok(())
//! # }
//! ```

use super::runner::{BenchmarkResults, TestStatus};
use anyhow::{Context, Result};
use std::path::Path;

/// CSV exporter for benchmark results.
///
/// Exports benchmark results to CSV format with configurable options.
pub struct CsvExporter {
    delimiter: u8,
    include_headers: bool,
}

impl CsvExporter {
    /// Creates a new CSV exporter with default settings.
    ///
    /// Default settings:
    /// - Delimiter: comma (`,`)
    /// - Headers: included
    pub fn new() -> Self {
        Self {
            delimiter: b',',
            include_headers: true,
        }
    }

    /// Sets the delimiter character for the CSV file.
    ///
    /// # Examples
    ///
    /// ```
    /// use llm_test_bench_core::benchmarks::CsvExporter;
    ///
    /// let exporter = CsvExporter::new()
    ///     .with_delimiter(b'\t'); // Tab-separated values
    /// ```
    pub fn with_delimiter(mut self, delimiter: u8) -> Self {
        self.delimiter = delimiter;
        self
    }

    /// Sets whether to include column headers in the output.
    pub fn with_headers(mut self, include: bool) -> Self {
        self.include_headers = include;
        self
    }

    /// Exports benchmark results to a CSV file.
    ///
    /// # Arguments
    ///
    /// * `results` - The benchmark results to export
    /// * `path` - Path where the CSV file should be written
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be created or written to.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use llm_test_bench_core::benchmarks::{BenchmarkResults, CsvExporter};
    /// # use std::path::Path;
    /// # fn example(results: BenchmarkResults) -> Result<(), Box<dyn std::error::Error>> {
    /// let exporter = CsvExporter::new();
    /// exporter.export(&results, Path::new("benchmark_results.csv"))?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn export(&self, results: &BenchmarkResults, path: &Path) -> Result<()> {
        let mut wtr = csv::WriterBuilder::new()
            .delimiter(self.delimiter)
            .has_headers(self.include_headers)
            .from_path(path)
            .context("Failed to create CSV file")?;

        // Write header if enabled
        if self.include_headers {
            wtr.write_record(&[
                "test_id",
                "category",
                "status",
                "duration_ms",
                "tokens",
                "cost",
                "model",
                "prompt_length",
                "response_length",
                "prompt_tokens",
                "completion_tokens",
                "finish_reason",
                "error",
                "timestamp",
            ])
            .context("Failed to write CSV header")?;
        }

        // Write data rows
        for result in &results.results {
            let (
                tokens,
                cost,
                model,
                prompt_length,
                response_length,
                prompt_tokens,
                completion_tokens,
                finish_reason,
            ) = if let Some(ref response) = result.response {
                let cost = response.usage.calculate_cost(0.03, 0.06);
                (
                    response.usage.total_tokens.to_string(),
                    format!("{:.6}", cost),
                    response.model.clone(),
                    response.content.len().to_string(),
                    response.content.len().to_string(),
                    response.usage.prompt_tokens.to_string(),
                    response.usage.completion_tokens.to_string(),
                    response.finish_reason.to_string(),
                )
            } else {
                (
                    String::new(),
                    String::new(),
                    String::new(),
                    String::new(),
                    String::new(),
                    String::new(),
                    String::new(),
                    String::new(),
                )
            };

            wtr.write_record(&[
                &result.test_id,
                result.category.as_deref().unwrap_or(""),
                &result.status.to_string(),
                &result.duration_ms.to_string(),
                &tokens,
                &cost,
                &model,
                &prompt_length,
                &response_length,
                &prompt_tokens,
                &completion_tokens,
                &finish_reason,
                result.error.as_deref().unwrap_or(""),
                &result.timestamp.to_rfc3339(),
            ])
            .context("Failed to write CSV record")?;
        }

        wtr.flush().context("Failed to flush CSV writer")?;
        Ok(())
    }

    /// Convenience method to export results to CSV with default settings.
    ///
    /// This is equivalent to `CsvExporter::new().export(results, path)`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use llm_test_bench_core::benchmarks::{BenchmarkResults, CsvExporter};
    /// # use std::path::Path;
    /// # fn example(results: BenchmarkResults) -> Result<(), Box<dyn std::error::Error>> {
    /// CsvExporter::export_default(&results, Path::new("results.csv"))?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn export_default(results: &BenchmarkResults, path: &Path) -> Result<()> {
        Self::new().export(results, path)
    }
}

impl Default for CsvExporter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::benchmarks::runner::{BenchmarkResults, TestResult};
    use crate::providers::{CompletionResponse, FinishReason, TokenUsage};
    use chrono::Utc;
    use std::fs;
    use std::time::Duration;
    use tempfile::TempDir;

    fn create_test_response() -> CompletionResponse {
        CompletionResponse {
            id: "test-resp-123".to_string(),
            model: "gpt-4".to_string(),
            content: "Test response content".to_string(),
            usage: TokenUsage::new(100, 50),
            finish_reason: FinishReason::Stop,
            created_at: Utc::now(),
        }
    }

    fn create_test_results() -> BenchmarkResults {
        let results = vec![
            TestResult::success(
                "test-1".to_string(),
                Some("coding".to_string()),
                create_test_response(),
                Duration::from_millis(1234),
            ),
            TestResult::failure(
                "test-2".to_string(),
                Some("reasoning".to_string()),
                "API rate limit exceeded".to_string(),
                Duration::from_millis(500),
            ),
            TestResult::success(
                "test-3".to_string(),
                None,
                create_test_response(),
                Duration::from_millis(987),
            ),
        ];

        let mut benchmark =
            BenchmarkResults::new("test-dataset".to_string(), "openai".to_string(), results);
        benchmark.calculate_summary();
        benchmark
    }

    #[test]
    fn test_csv_export_creates_file() {
        let results = create_test_results();
        let temp_dir = TempDir::new().unwrap();
        let csv_path = temp_dir.path().join("results.csv");

        CsvExporter::export_default(&results, &csv_path).unwrap();

        assert!(csv_path.exists());
    }

    #[test]
    fn test_csv_export_has_headers() {
        let results = create_test_results();
        let temp_dir = TempDir::new().unwrap();
        let csv_path = temp_dir.path().join("results.csv");

        CsvExporter::new().export(&results, &csv_path).unwrap();

        let content = fs::read_to_string(&csv_path).unwrap();
        let lines: Vec<&str> = content.lines().collect();

        // Check header
        assert!(lines[0].contains("test_id"));
        assert!(lines[0].contains("category"));
        assert!(lines[0].contains("status"));
        assert!(lines[0].contains("duration_ms"));
        assert!(lines[0].contains("tokens"));
        assert!(lines[0].contains("cost"));
    }

    #[test]
    fn test_csv_export_correct_row_count() {
        let results = create_test_results();
        let temp_dir = TempDir::new().unwrap();
        let csv_path = temp_dir.path().join("results.csv");

        CsvExporter::new().export(&results, &csv_path).unwrap();

        let content = fs::read_to_string(&csv_path).unwrap();
        let lines: Vec<&str> = content.lines().collect();

        // Header + 3 data rows
        assert_eq!(lines.len(), 4);
    }

    #[test]
    fn test_csv_export_data_integrity() {
        let results = create_test_results();
        let temp_dir = TempDir::new().unwrap();
        let csv_path = temp_dir.path().join("results.csv");

        CsvExporter::new().export(&results, &csv_path).unwrap();

        let content = fs::read_to_string(&csv_path).unwrap();
        let lines: Vec<&str> = content.lines().collect();

        // Check first data row (test-1, success)
        assert!(lines[1].contains("test-1"));
        assert!(lines[1].contains("coding"));
        assert!(lines[1].contains("success"));
        assert!(lines[1].contains("1234"));
        assert!(lines[1].contains("150")); // Total tokens

        // Check second data row (test-2, failure)
        assert!(lines[2].contains("test-2"));
        assert!(lines[2].contains("reasoning"));
        assert!(lines[2].contains("failure"));
        assert!(lines[2].contains("API rate limit exceeded"));
    }

    #[test]
    fn test_csv_export_without_headers() {
        let results = create_test_results();
        let temp_dir = TempDir::new().unwrap();
        let csv_path = temp_dir.path().join("results.csv");

        CsvExporter::new()
            .with_headers(false)
            .export(&results, &csv_path)
            .unwrap();

        let content = fs::read_to_string(&csv_path).unwrap();
        let lines: Vec<&str> = content.lines().collect();

        // Only data rows, no header
        assert_eq!(lines.len(), 3);
        assert!(lines[0].contains("test-1"));
    }

    #[test]
    fn test_csv_export_custom_delimiter() {
        let results = create_test_results();
        let temp_dir = TempDir::new().unwrap();
        let csv_path = temp_dir.path().join("results.tsv");

        CsvExporter::new()
            .with_delimiter(b'\t')
            .export(&results, &csv_path)
            .unwrap();

        let content = fs::read_to_string(&csv_path).unwrap();
        assert!(content.contains('\t'));
    }

    #[test]
    fn test_csv_export_empty_results() {
        let results = BenchmarkResults::new("dataset".to_string(), "provider".to_string(), vec![]);
        let temp_dir = TempDir::new().unwrap();
        let csv_path = temp_dir.path().join("results.csv");

        CsvExporter::export_default(&results, &csv_path).unwrap();

        let content = fs::read_to_string(&csv_path).unwrap();
        let lines: Vec<&str> = content.lines().collect();

        // Only header, no data
        assert_eq!(lines.len(), 1);
    }

    #[test]
    fn test_csv_export_missing_fields() {
        // Test with a failure result that has no response
        let results = vec![TestResult::failure(
            "test-fail".to_string(),
            None,
            "Error occurred".to_string(),
            Duration::from_millis(100),
        )];

        let benchmark =
            BenchmarkResults::new("dataset".to_string(), "provider".to_string(), results);
        let temp_dir = TempDir::new().unwrap();
        let csv_path = temp_dir.path().join("results.csv");

        CsvExporter::export_default(&benchmark, &csv_path).unwrap();

        let content = fs::read_to_string(&csv_path).unwrap();
        let lines: Vec<&str> = content.lines().collect();

        // Should have header + 1 row
        assert_eq!(lines.len(), 2);
        assert!(lines[1].contains("test-fail"));
        assert!(lines[1].contains("Error occurred"));
    }

    #[test]
    fn test_csv_export_cost_calculation() {
        let results = create_test_results();
        let temp_dir = TempDir::new().unwrap();
        let csv_path = temp_dir.path().join("results.csv");

        CsvExporter::new().export(&results, &csv_path).unwrap();

        let content = fs::read_to_string(&csv_path).unwrap();
        let lines: Vec<&str> = content.lines().collect();

        // Check that cost is calculated (100 prompt * 0.03/1k + 50 completion * 0.06/1k)
        // = 0.003 + 0.003 = 0.006
        assert!(lines[1].contains("0.006"));
    }

    #[test]
    fn test_csv_export_all_status_types() {
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
            TestResult::timeout("test-timeout".to_string(), None, Duration::from_millis(30000)),
            TestResult::skipped("test-skipped".to_string(), None),
        ];

        let benchmark =
            BenchmarkResults::new("dataset".to_string(), "provider".to_string(), results);
        let temp_dir = TempDir::new().unwrap();
        let csv_path = temp_dir.path().join("results.csv");

        CsvExporter::export_default(&benchmark, &csv_path).unwrap();

        let content = fs::read_to_string(&csv_path).unwrap();

        assert!(content.contains("success"));
        assert!(content.contains("failure"));
        assert!(content.contains("timeout"));
        assert!(content.contains("skipped"));
    }

    #[test]
    fn test_csv_can_be_parsed() {
        let results = create_test_results();
        let temp_dir = TempDir::new().unwrap();
        let csv_path = temp_dir.path().join("results.csv");

        CsvExporter::export_default(&results, &csv_path).unwrap();

        // Try to read it back with csv reader
        let mut rdr = csv::Reader::from_path(&csv_path).unwrap();
        let headers = rdr.headers().unwrap();

        assert_eq!(headers.get(0).unwrap(), "test_id");
        assert_eq!(headers.get(1).unwrap(), "category");

        let records: Vec<_> = rdr.records().collect();
        assert_eq!(records.len(), 3);
    }
}

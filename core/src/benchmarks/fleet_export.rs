// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Fleet-level export functionality for benchmark results.
//!
//! This module provides utilities to export fleet benchmark results to various formats:
//! - CSV summaries (repository-level and fleet-level)
//! - Executive reports (HTML/Markdown)
//! - Deterministic JSON output
//!
//! # Examples
//!
//! ```no_run
//! use llm_test_bench_core::benchmarks::fleet::{FleetBenchmarkResults, FleetCsvExporter};
//! use std::path::Path;
//!
//! # fn example(fleet_results: FleetBenchmarkResults) -> Result<(), Box<dyn std::error::Error>> {
//! // Export fleet summary to CSV
//! FleetCsvExporter::export_summary(&fleet_results, Path::new("fleet_summary.csv"))?;
//!
//! // Export repository details to CSV
//! FleetCsvExporter::export_repositories(&fleet_results, Path::new("repositories.csv"))?;
//!
//! // Export executive report
//! FleetCsvExporter::export_executive_report(&fleet_results, Path::new("executive_report.html"))?;
//! # Ok(())
//! # }
//! ```

use super::fleet::{FleetBenchmarkResults, ProviderFleetStats, CategoryFleetStats};
use anyhow::{Context, Result};
use serde_json::json;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::Write;
use std::path::Path;

/// CSV exporter for fleet-level benchmark results.
///
/// Provides multiple export formats optimized for different use cases:
/// - Summary: High-level fleet statistics
/// - Repositories: Per-repository details
/// - Providers: Per-provider breakdown
/// - Categories: Per-category breakdown
pub struct FleetCsvExporter {
    delimiter: u8,
    include_headers: bool,
}

impl FleetCsvExporter {
    /// Creates a new fleet CSV exporter with default settings.
    pub fn new() -> Self {
        Self {
            delimiter: b',',
            include_headers: true,
        }
    }

    /// Sets the delimiter character for CSV files.
    pub fn with_delimiter(mut self, delimiter: u8) -> Self {
        self.delimiter = delimiter;
        self
    }

    /// Sets whether to include headers in CSV output.
    pub fn with_headers(mut self, include: bool) -> Self {
        self.include_headers = include;
        self
    }

    /// Exports fleet summary statistics to CSV.
    ///
    /// Produces a single-row CSV with fleet-wide metrics.
    pub fn export_summary(results: &FleetBenchmarkResults, path: &Path) -> Result<()> {
        let mut wtr = csv::WriterBuilder::new()
            .delimiter(b',')
            .has_headers(true)
            .from_path(path)
            .context("Failed to create fleet summary CSV file")?;

        // Write header
        wtr.write_record(&[
            "fleet_id",
            "timestamp",
            "total_repositories",
            "total_tests",
            "total_succeeded",
            "total_failed",
            "total_timeout",
            "total_skipped",
            "success_rate",
            "avg_duration_ms",
            "p50_duration_ms",
            "p95_duration_ms",
            "p99_duration_ms",
            "min_duration_ms",
            "max_duration_ms",
            "total_tokens",
            "avg_tokens_per_request",
            "total_cost",
            "avg_cost_per_repository",
            "avg_tests_per_repository",
        ])
        .context("Failed to write fleet summary header")?;

        // Write data
        wtr.write_record(&[
            &results.fleet_id,
            &results.timestamp.to_rfc3339(),
            &results.fleet_summary.total_repositories.to_string(),
            &results.fleet_summary.total_tests.to_string(),
            &results.fleet_summary.total_succeeded.to_string(),
            &results.fleet_summary.total_failed.to_string(),
            &results.fleet_summary.total_timeout.to_string(),
            &results.fleet_summary.total_skipped.to_string(),
            &format!("{:.4}", results.fleet_summary.success_rate),
            &format!("{:.2}", results.fleet_summary.avg_duration_ms),
            &format!("{:.2}", results.fleet_summary.p50_duration_ms),
            &format!("{:.2}", results.fleet_summary.p95_duration_ms),
            &format!("{:.2}", results.fleet_summary.p99_duration_ms),
            &results.fleet_summary.min_duration_ms.to_string(),
            &results.fleet_summary.max_duration_ms.to_string(),
            &results.fleet_summary.total_tokens.to_string(),
            &format!("{:.2}", results.fleet_summary.avg_tokens_per_request),
            &format!("{:.6}", results.fleet_summary.total_cost),
            &format!("{:.6}", results.fleet_summary.avg_cost_per_repository),
            &format!("{:.2}", results.fleet_summary.avg_tests_per_repository),
        ])
        .context("Failed to write fleet summary record")?;

        wtr.flush().context("Failed to flush CSV writer")?;
        Ok(())
    }

    /// Exports per-repository statistics to CSV.
    ///
    /// Produces one row per repository with detailed metrics.
    pub fn export_repositories(results: &FleetBenchmarkResults, path: &Path) -> Result<()> {
        let mut wtr = csv::WriterBuilder::new()
            .delimiter(b',')
            .has_headers(true)
            .from_path(path)
            .context("Failed to create repositories CSV file")?;

        // Write header
        wtr.write_record(&[
            "repository_id",
            "repository_name",
            "provider_name",
            "total_tests",
            "succeeded",
            "failed",
            "timeout",
            "skipped",
            "success_rate",
            "avg_duration_ms",
            "p50_duration_ms",
            "p95_duration_ms",
            "p99_duration_ms",
            "total_tokens",
            "total_cost",
        ])
        .context("Failed to write repositories header")?;

        // Write data rows
        for repo in &results.repository_results {
            wtr.write_record(&[
                &repo.repository_id,
                &repo.repository_name,
                &repo.provider_name,
                &repo.results.summary.total.to_string(),
                &repo.results.summary.succeeded.to_string(),
                &repo.results.summary.failed.to_string(),
                &repo.results.summary.timeout.to_string(),
                &repo.results.summary.skipped.to_string(),
                &format!("{:.4}", repo.results.summary.success_rate),
                &format!("{:.2}", repo.results.summary.avg_duration_ms),
                &format!("{:.2}", repo.results.summary.p50_duration_ms),
                &format!("{:.2}", repo.results.summary.p95_duration_ms),
                &format!("{:.2}", repo.results.summary.p99_duration_ms),
                &repo.results.summary.total_tokens.to_string(),
                &format!("{:.6}", repo.results.summary.total_cost),
            ])
            .context("Failed to write repository record")?;
        }

        wtr.flush().context("Failed to flush CSV writer")?;
        Ok(())
    }

    /// Exports per-provider statistics to CSV.
    pub fn export_providers(results: &FleetBenchmarkResults, path: &Path) -> Result<()> {
        let mut wtr = csv::WriterBuilder::new()
            .delimiter(b',')
            .has_headers(true)
            .from_path(path)
            .context("Failed to create providers CSV file")?;

        // Write header
        wtr.write_record(&[
            "provider_name",
            "repository_count",
            "total_tests",
            "total_succeeded",
            "total_failed",
            "success_rate",
            "total_tokens",
            "total_cost",
        ])
        .context("Failed to write providers header")?;

        // Sort providers by name for deterministic output
        let mut providers: Vec<_> = results.provider_breakdown.iter().collect();
        providers.sort_by_key(|(name, _)| *name);

        // Write data rows
        for (_, stats) in providers {
            wtr.write_record(&[
                &stats.provider_name,
                &stats.repository_count.to_string(),
                &stats.total_tests.to_string(),
                &stats.total_succeeded.to_string(),
                &stats.total_failed.to_string(),
                &format!("{:.4}", stats.success_rate),
                &stats.total_tokens.to_string(),
                &format!("{:.6}", stats.total_cost),
            ])
            .context("Failed to write provider record")?;
        }

        wtr.flush().context("Failed to flush CSV writer")?;
        Ok(())
    }

    /// Exports per-category statistics to CSV.
    pub fn export_categories(results: &FleetBenchmarkResults, path: &Path) -> Result<()> {
        let mut wtr = csv::WriterBuilder::new()
            .delimiter(b',')
            .has_headers(true)
            .from_path(path)
            .context("Failed to create categories CSV file")?;

        // Write header
        wtr.write_record(&[
            "category_name",
            "total_tests",
            "total_succeeded",
            "total_failed",
            "success_rate",
        ])
        .context("Failed to write categories header")?;

        // Sort categories by name for deterministic output
        let mut categories: Vec<_> = results.category_breakdown.iter().collect();
        categories.sort_by_key(|(name, _)| *name);

        // Write data rows
        for (_, stats) in categories {
            wtr.write_record(&[
                &stats.category_name,
                &stats.total_tests.to_string(),
                &stats.total_succeeded.to_string(),
                &stats.total_failed.to_string(),
                &format!("{:.4}", stats.success_rate),
            ])
            .context("Failed to write category record")?;
        }

        wtr.flush().context("Failed to flush CSV writer")?;
        Ok(())
    }

    /// Exports an executive summary report in HTML format.
    ///
    /// Generates a comprehensive, human-readable report with visualizations.
    pub fn export_executive_report(results: &FleetBenchmarkResults, path: &Path) -> Result<()> {
        let html = Self::generate_executive_html(results);
        let mut file = File::create(path).context("Failed to create executive report file")?;
        file.write_all(html.as_bytes())
            .context("Failed to write executive report")?;
        Ok(())
    }

    /// Generates HTML for executive report.
    fn generate_executive_html(results: &FleetBenchmarkResults) -> String {
        let best_repo = results.best_repository();
        let worst_repo = results.worst_repository();
        let failing_repos = results.failing_repositories(0.9);

        // Format large numbers with commas for readability
        let total_tokens_formatted = Self::format_number_with_commas(results.fleet_summary.total_tokens);

        format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>Fleet Benchmark Executive Report - {fleet_id}</title>
    <style>
        body {{
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
            max-width: 1200px;
            margin: 0 auto;
            padding: 20px;
            background-color: #f5f5f5;
        }}
        h1, h2 {{ color: #333; }}
        .header {{
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
            padding: 30px;
            border-radius: 8px;
            margin-bottom: 30px;
        }}
        .header h1 {{ margin: 0 0 10px 0; }}
        .header .meta {{ opacity: 0.9; font-size: 14px; }}
        .metric-grid {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
            gap: 20px;
            margin-bottom: 30px;
        }}
        .metric-card {{
            background: white;
            padding: 20px;
            border-radius: 8px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }}
        .metric-card .label {{
            font-size: 12px;
            text-transform: uppercase;
            color: #888;
            margin-bottom: 8px;
        }}
        .metric-card .value {{
            font-size: 32px;
            font-weight: bold;
            color: #333;
        }}
        .metric-card .subvalue {{
            font-size: 14px;
            color: #666;
            margin-top: 5px;
        }}
        .success {{ color: #10b981; }}
        .warning {{ color: #f59e0b; }}
        .error {{ color: #ef4444; }}
        .section {{
            background: white;
            padding: 25px;
            border-radius: 8px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
            margin-bottom: 20px;
        }}
        table {{
            width: 100%;
            border-collapse: collapse;
            margin-top: 15px;
        }}
        th, td {{
            text-align: left;
            padding: 12px;
            border-bottom: 1px solid #e5e5e5;
        }}
        th {{
            background-color: #f9f9f9;
            font-weight: 600;
            color: #666;
        }}
        .progress-bar {{
            width: 100%;
            height: 8px;
            background-color: #e5e5e5;
            border-radius: 4px;
            overflow: hidden;
        }}
        .progress-fill {{
            height: 100%;
            transition: width 0.3s ease;
        }}
    </style>
</head>
<body>
    <div class="header">
        <h1>Fleet Benchmark Report</h1>
        <div class="meta">
            Fleet ID: {fleet_id} | Generated: {timestamp}
        </div>
    </div>

    <div class="metric-grid">
        <div class="metric-card">
            <div class="label">Total Repositories</div>
            <div class="value">{total_repos}</div>
        </div>
        <div class="metric-card">
            <div class="label">Total Tests</div>
            <div class="value">{total_tests}</div>
            <div class="subvalue">{avg_tests:.1} per repository</div>
        </div>
        <div class="metric-card">
            <div class="label">Success Rate</div>
            <div class="value {success_class}">{success_rate:.1}%</div>
            <div class="subvalue">{succeeded} / {total_tests} passed</div>
        </div>
        <div class="metric-card">
            <div class="label">Total Cost</div>
            <div class="value">${total_cost:.4}</div>
            <div class="subvalue">${avg_cost:.4} per repository</div>
        </div>
    </div>

    <div class="section">
        <h2>Performance Metrics</h2>
        <table>
            <tr>
                <th>Metric</th>
                <th>Value</th>
            </tr>
            <tr>
                <td>Average Duration</td>
                <td>{avg_duration:.2} ms</td>
            </tr>
            <tr>
                <td>P50 (Median)</td>
                <td>{p50:.2} ms</td>
            </tr>
            <tr>
                <td>P95</td>
                <td>{p95:.2} ms</td>
            </tr>
            <tr>
                <td>P99</td>
                <td>{p99:.2} ms</td>
            </tr>
            <tr>
                <td>Min / Max</td>
                <td>{min} ms / {max} ms</td>
            </tr>
            <tr>
                <td>Total Tokens</td>
                <td>{total_tokens_formatted}</td>
            </tr>
            <tr>
                <td>Avg Tokens/Request</td>
                <td>{avg_tokens:.2}</td>
            </tr>
        </table>
    </div>

    <div class="section">
        <h2>Provider Breakdown</h2>
        <table>
            <tr>
                <th>Provider</th>
                <th>Repositories</th>
                <th>Tests</th>
                <th>Success Rate</th>
                <th>Cost</th>
            </tr>
            {provider_rows}
        </table>
    </div>

    {best_worst_section}

    {failing_section}

</body>
</html>"#,
            fleet_id = results.fleet_id,
            timestamp = results.timestamp.format("%Y-%m-%d %H:%M:%S UTC"),
            total_repos = results.fleet_summary.total_repositories,
            total_tests = results.fleet_summary.total_tests,
            avg_tests = results.fleet_summary.avg_tests_per_repository,
            success_rate = results.fleet_summary.success_rate * 100.0,
            success_class = if results.fleet_summary.success_rate >= 0.95 {
                "success"
            } else if results.fleet_summary.success_rate >= 0.8 {
                "warning"
            } else {
                "error"
            },
            succeeded = results.fleet_summary.total_succeeded,
            total_cost = results.fleet_summary.total_cost,
            avg_cost = results.fleet_summary.avg_cost_per_repository,
            avg_duration = results.fleet_summary.avg_duration_ms,
            p50 = results.fleet_summary.p50_duration_ms,
            p95 = results.fleet_summary.p95_duration_ms,
            p99 = results.fleet_summary.p99_duration_ms,
            min = results.fleet_summary.min_duration_ms,
            max = results.fleet_summary.max_duration_ms,
            total_tokens_formatted = total_tokens_formatted,
            avg_tokens = results.fleet_summary.avg_tokens_per_request,
            provider_rows = Self::generate_provider_rows(results),
            best_worst_section = Self::generate_best_worst_section(best_repo, worst_repo),
            failing_section = Self::generate_failing_section(&failing_repos),
        )
    }

    fn generate_provider_rows(results: &FleetBenchmarkResults) -> String {
        let mut providers: Vec<_> = results.provider_breakdown.iter().collect();
        providers.sort_by_key(|(name, _)| *name);

        providers
            .iter()
            .map(|(_, stats)| {
                format!(
                    r#"<tr>
                <td>{}</td>
                <td>{}</td>
                <td>{}</td>
                <td>{:.1}%</td>
                <td>${:.4}</td>
            </tr>"#,
                    stats.provider_name,
                    stats.repository_count,
                    stats.total_tests,
                    stats.success_rate * 100.0,
                    stats.total_cost
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn generate_best_worst_section(
        best: Option<&super::fleet::RepositoryResults>,
        worst: Option<&super::fleet::RepositoryResults>,
    ) -> String {
        if let (Some(best), Some(worst)) = (best, worst) {
            format!(
                r#"<div class="section">
        <h2>Repository Comparison</h2>
        <table>
            <tr>
                <th></th>
                <th>Repository</th>
                <th>Provider</th>
                <th>Success Rate</th>
                <th>Tests</th>
            </tr>
            <tr>
                <td style="color: #10b981; font-weight: bold;">Best</td>
                <td>{}</td>
                <td>{}</td>
                <td class="success">{:.1}%</td>
                <td>{}</td>
            </tr>
            <tr>
                <td style="color: #ef4444; font-weight: bold;">Worst</td>
                <td>{}</td>
                <td>{}</td>
                <td class="error">{:.1}%</td>
                <td>{}</td>
            </tr>
        </table>
    </div>"#,
                best.repository_name,
                best.provider_name,
                best.results.summary.success_rate * 100.0,
                best.results.summary.total,
                worst.repository_name,
                worst.provider_name,
                worst.results.summary.success_rate * 100.0,
                worst.results.summary.total,
            )
        } else {
            String::new()
        }
    }

    fn generate_failing_section(
        failing: &[&super::fleet::RepositoryResults],
    ) -> String {
        if failing.is_empty() {
            return String::new();
        }

        let rows = failing
            .iter()
            .map(|repo| {
                format!(
                    r#"<tr>
                <td>{}</td>
                <td>{}</td>
                <td class="error">{:.1}%</td>
                <td>{}</td>
            </tr>"#,
                    repo.repository_name,
                    repo.provider_name,
                    repo.results.summary.success_rate * 100.0,
                    repo.results.summary.total,
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        format!(
            r#"<div class="section">
        <h2 class="error">Repositories Below 90% Success Rate ({} found)</h2>
        <table>
            <tr>
                <th>Repository</th>
                <th>Provider</th>
                <th>Success Rate</th>
                <th>Tests</th>
            </tr>
            {}
        </table>
    </div>"#,
            failing.len(),
            rows
        )
    }

    /// Formats a number with thousand separators (commas).
    fn format_number_with_commas(n: usize) -> String {
        let s = n.to_string();
        let mut result = String::new();
        let chars: Vec<char> = s.chars().collect();

        for (i, c) in chars.iter().enumerate() {
            if i > 0 && (chars.len() - i) % 3 == 0 {
                result.push(',');
            }
            result.push(*c);
        }

        result
    }

    /// Exports deterministic JSON output.
    ///
    /// Ensures consistent field ordering and formatting for reproducibility.
    pub fn export_deterministic_json(results: &FleetBenchmarkResults, path: &Path) -> Result<()> {
        // Use BTreeMap for deterministic ordering
        let mut ordered_json = BTreeMap::new();

        ordered_json.insert("fleet_id", json!(results.fleet_id));
        ordered_json.insert("timestamp", json!(results.timestamp.to_rfc3339()));
        ordered_json.insert("total_repositories", json!(results.total_repositories));
        ordered_json.insert("fleet_summary", json!(results.fleet_summary));
        ordered_json.insert("repository_results", json!(results.repository_results));
        ordered_json.insert("provider_breakdown", json!(results.provider_breakdown));
        ordered_json.insert("category_breakdown", json!(results.category_breakdown));
        ordered_json.insert("metadata", json!(results.metadata));

        let json_str = serde_json::to_string_pretty(&ordered_json)
            .context("Failed to serialize fleet results to JSON")?;

        let mut file = File::create(path).context("Failed to create JSON file")?;
        file.write_all(json_str.as_bytes())
            .context("Failed to write JSON file")?;

        Ok(())
    }
}

impl Default for FleetCsvExporter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::benchmarks::fleet::FleetBenchmarkResults;
    use crate::benchmarks::runner::{BenchmarkResults, TestResult};
    use crate::providers::{CompletionResponse, FinishReason, TokenUsage};
    use chrono::Utc;
    use std::fs;
    use std::time::Duration;
    use tempfile::TempDir;

    fn create_test_response() -> CompletionResponse {
        CompletionResponse {
            id: "test-resp".to_string(),
            model: "test-model".to_string(),
            content: "test content".to_string(),
            usage: TokenUsage::new(100, 50),
            finish_reason: FinishReason::Stop,
            created_at: Utc::now(),
        }
    }

    fn create_fleet_results() -> FleetBenchmarkResults {
        let mut repo1_results = vec![];
        for i in 0..10 {
            repo1_results.push(TestResult::success(
                format!("test-{}", i),
                Some("category".to_string()),
                create_test_response(),
                Duration::from_millis(1000),
            ));
        }

        let mut repo2_results = vec![];
        for i in 0..8 {
            repo2_results.push(TestResult::success(
                format!("test-{}", i),
                Some("category".to_string()),
                create_test_response(),
                Duration::from_millis(1000),
            ));
        }
        for i in 0..2 {
            repo2_results.push(TestResult::failure(
                format!("test-fail-{}", i),
                Some("category".to_string()),
                "Error".to_string(),
                Duration::from_millis(500),
            ));
        }

        let mut bench1 =
            BenchmarkResults::new("repo1".to_string(), "openai".to_string(), repo1_results);
        bench1.calculate_summary();

        let mut bench2 =
            BenchmarkResults::new("repo2".to_string(), "anthropic".to_string(), repo2_results);
        bench2.calculate_summary();

        FleetBenchmarkResults::from_repositories("test-fleet".to_string(), vec![bench1, bench2])
    }

    #[test]
    fn test_export_summary() {
        let fleet = create_fleet_results();
        let temp_dir = TempDir::new().unwrap();
        let csv_path = temp_dir.path().join("fleet_summary.csv");

        FleetCsvExporter::export_summary(&fleet, &csv_path).unwrap();

        assert!(csv_path.exists());
        let content = fs::read_to_string(&csv_path).unwrap();
        assert!(content.contains("fleet_id"));
        assert!(content.contains("test-fleet"));
    }

    #[test]
    fn test_export_repositories() {
        let fleet = create_fleet_results();
        let temp_dir = TempDir::new().unwrap();
        let csv_path = temp_dir.path().join("repositories.csv");

        FleetCsvExporter::export_repositories(&fleet, &csv_path).unwrap();

        assert!(csv_path.exists());
        let content = fs::read_to_string(&csv_path).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 3); // header + 2 repos
    }

    #[test]
    fn test_export_providers() {
        let fleet = create_fleet_results();
        let temp_dir = TempDir::new().unwrap();
        let csv_path = temp_dir.path().join("providers.csv");

        FleetCsvExporter::export_providers(&fleet, &csv_path).unwrap();

        assert!(csv_path.exists());
        let content = fs::read_to_string(&csv_path).unwrap();
        assert!(content.contains("openai"));
        assert!(content.contains("anthropic"));
    }

    #[test]
    fn test_export_executive_report() {
        let fleet = create_fleet_results();
        let temp_dir = TempDir::new().unwrap();
        let html_path = temp_dir.path().join("executive_report.html");

        FleetCsvExporter::export_executive_report(&fleet, &html_path).unwrap();

        assert!(html_path.exists());
        let content = fs::read_to_string(&html_path).unwrap();
        assert!(content.contains("<!DOCTYPE html>"));
        assert!(content.contains("Fleet Benchmark Report"));
        assert!(content.contains("test-fleet"));
    }

    #[test]
    fn test_export_deterministic_json() {
        let fleet = create_fleet_results();
        let temp_dir = TempDir::new().unwrap();
        let json_path = temp_dir.path().join("fleet_results.json");

        FleetCsvExporter::export_deterministic_json(&fleet, &json_path).unwrap();

        assert!(json_path.exists());
        let content = fs::read_to_string(&json_path).unwrap();

        // Verify JSON is valid
        let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert_eq!(parsed["fleet_id"], "test-fleet");
    }
}

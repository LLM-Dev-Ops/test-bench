// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! HTML dashboard generation with interactive charts.
//!
//! This module provides a complete dashboard generation system that creates
//! self-contained HTML files with embedded Chart.js visualizations.

use crate::benchmarks::results::BenchmarkResults;
use crate::visualization::charts::ChartDataFormatter;
use anyhow::{Context, Result};
use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use tera::{Context as TeraContext, Tera};

/// Main dashboard generator using Tera templates.
pub struct DashboardGenerator {
    tera: Tera,
}

/// Type of dashboard to generate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DashboardType {
    /// Single benchmark run results
    BenchmarkResults,
    /// Compare multiple models side-by-side
    ModelComparison,
    /// Show performance trends over time
    TrendAnalysis,
    /// Analyze cost efficiency
    CostAnalysis,
}

/// Configuration for dashboard generation.
#[derive(Debug, Clone)]
pub struct DashboardConfig {
    /// Dashboard title
    pub title: String,
    /// Color theme
    pub theme: Theme,
    /// Maximum data points to display (prevents overcrowding)
    pub max_data_points: usize,
    /// Custom color palette for charts
    pub chart_colors: Vec<String>,
}

impl Default for DashboardConfig {
    fn default() -> Self {
        Self {
            title: "LLM Test Bench Dashboard".to_string(),
            theme: Theme::Auto,
            max_data_points: 1000,
            chart_colors: vec![
                "rgb(59, 130, 246)".to_string(),   // Blue
                "rgb(16, 185, 129)".to_string(),   // Green
                "rgb(245, 158, 11)".to_string(),   // Orange
                "rgb(239, 68, 68)".to_string(),    // Red
                "rgb(139, 92, 246)".to_string(),   // Purple
                "rgb(236, 72, 153)".to_string(),   // Pink
            ],
        }
    }
}

/// Dashboard color theme.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Theme {
    /// Light theme
    Light,
    /// Dark theme
    Dark,
    /// Auto-detect based on system preference
    Auto,
}

/// Data container for dashboard rendering.
#[derive(Debug, Serialize)]
pub struct DashboardData {
    pub title: String,
    pub dataset_name: String,
    pub provider_name: Option<String>,
    pub timestamp: String,
    pub summary_cards: Vec<SummaryCard>,
    pub results: Vec<ResultRow>,
    pub latency_data_json: String,
    pub metrics_data_json: String,
    pub status_data_json: String,
    pub chartjs_code: String,
}

/// Summary card data for the dashboard header.
#[derive(Debug, Serialize)]
pub struct SummaryCard {
    pub title: String,
    pub value: String,
    pub change: Option<String>,
    pub card_class: String,
}

/// Individual result row for detailed table.
#[derive(Debug, Serialize)]
pub struct ResultRow {
    pub test_name: String,
    pub model: String,
    pub status: String,
    pub status_class: String,
    pub faithfulness: f64,
    pub relevance: f64,
    pub coherence: f64,
    pub latency_ms: u64,
    pub cost: f64,
}

impl DashboardGenerator {
    /// Creates a new dashboard generator with templates loaded from embedded strings.
    pub fn new() -> Result<Self> {
        let mut tera = Tera::default();

        // Load templates from embedded strings
        tera.add_raw_template("base.html", include_str!("templates/base.html"))
            .context("Failed to load base template")?;

        tera.add_raw_template(
            "benchmark_results.html",
            include_str!("templates/benchmark_results.html"),
        )
        .context("Failed to load benchmark_results template")?;

        tera.add_raw_template(
            "comparison.html",
            include_str!("templates/comparison.html"),
        )
        .context("Failed to load comparison template")?;

        tera.add_raw_template(
            "trend_analysis.html",
            include_str!("templates/trend_analysis.html"),
        )
        .context("Failed to load trend_analysis template")?;

        tera.add_raw_template(
            "cost_analysis.html",
            include_str!("templates/cost_analysis.html"),
        )
        .context("Failed to load cost_analysis template")?;

        Ok(Self { tera })
    }

    /// Generates a dashboard HTML string.
    pub fn generate_dashboard(
        &self,
        data: &DashboardData,
        dashboard_type: DashboardType,
        config: &DashboardConfig,
    ) -> Result<String> {
        let mut context = TeraContext::new();
        context.insert("title", &data.title);
        context.insert("dataset_name", &data.dataset_name);
        context.insert("provider_name", &data.provider_name);
        context.insert("timestamp", &data.timestamp);
        context.insert("summary_cards", &data.summary_cards);
        context.insert("results", &data.results);
        context.insert("latency_data_json", &data.latency_data_json);
        context.insert("metrics_data_json", &data.metrics_data_json);
        context.insert("status_data_json", &data.status_data_json);
        context.insert("chartjs_code", &data.chartjs_code);
        context.insert("theme", &theme_to_string(config.theme));

        let template_name = match dashboard_type {
            DashboardType::BenchmarkResults => "benchmark_results.html",
            DashboardType::ModelComparison => "comparison.html",
            DashboardType::TrendAnalysis => "trend_analysis.html",
            DashboardType::CostAnalysis => "cost_analysis.html",
        };

        self.tera
            .render(template_name, &context)
            .context("Failed to render template")
    }

    /// Generates a benchmark results dashboard from benchmark data.
    pub fn generate_benchmark_dashboard(
        &self,
        results: &BenchmarkResults,
        config: &DashboardConfig,
    ) -> Result<String> {
        let data = self.prepare_benchmark_data(results)?;
        self.generate_dashboard(&data, DashboardType::BenchmarkResults, config)
    }

    /// Generates a comparison dashboard from multiple benchmark results.
    pub fn generate_comparison_dashboard(
        &self,
        all_results: &[BenchmarkResults],
        config: &DashboardConfig,
    ) -> Result<String> {
        let data = self.prepare_comparison_data(all_results)?;
        self.generate_dashboard(&data, DashboardType::ModelComparison, config)
    }

    /// Generates a trend analysis dashboard from historical results.
    pub fn generate_trend_dashboard(
        &self,
        historical_results: &[BenchmarkResults],
        config: &DashboardConfig,
    ) -> Result<String> {
        let data = self.prepare_trend_data(historical_results)?;
        self.generate_dashboard(&data, DashboardType::TrendAnalysis, config)
    }

    /// Exports HTML to a file.
    pub fn export_to_file(&self, html: &str, output_path: &Path) -> Result<()> {
        // Create parent directory if it doesn't exist
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)
                .context("Failed to create output directory")?;
        }

        fs::write(output_path, html)
            .context("Failed to write dashboard file")?;

        Ok(())
    }

    /// Prepares data for benchmark results dashboard.
    fn prepare_benchmark_data(&self, results: &BenchmarkResults) -> Result<DashboardData> {
        let summary_cards = vec![
            SummaryCard {
                title: "Total Tests".to_string(),
                value: results.summary.total.to_string(),
                change: None,
                card_class: "card-primary".to_string(),
            },
            SummaryCard {
                title: "Success Rate".to_string(),
                value: format!("{:.1}%", results.summary.success_rate * 100.0),
                change: None,
                card_class: if results.summary.success_rate > 0.9 {
                    "card-success"
                } else {
                    "card-warning"
                }
                .to_string(),
            },
            SummaryCard {
                title: "Avg Latency".to_string(),
                value: format!("{:.0}ms", results.summary.avg_duration_ms),
                change: None,
                card_class: "card-info".to_string(),
            },
            SummaryCard {
                title: "Total Cost".to_string(),
                value: format!("${:.4}", results.summary.total_cost),
                change: None,
                card_class: "card-warning".to_string(),
            },
        ];

        let result_rows: Vec<ResultRow> = results
            .results
            .iter()
            .map(|r| {
                let cost = r.response.as_ref().map_or(0.0, |resp| {
                    (resp.usage.prompt_tokens as f64 / 1000.0) * 0.03
                        + (resp.usage.completion_tokens as f64 / 1000.0) * 0.06
                });

                let (status_class, status_str) = match r.status {
                    crate::benchmarks::results::TestStatus::Success => ("success", "Success"),
                    crate::benchmarks::results::TestStatus::Failure => ("danger", "Failed"),
                    crate::benchmarks::results::TestStatus::Timeout => ("warning", "Timeout"),
                    crate::benchmarks::results::TestStatus::Skipped => ("secondary", "Skipped"),
                };

                ResultRow {
                    test_name: r.test_id.clone(),
                    model: r
                        .response
                        .as_ref()
                        .map(|resp| resp.model.clone())
                        .unwrap_or_else(|| "N/A".to_string()),
                    status: status_str.to_string(),
                    status_class: status_class.to_string(),
                    faithfulness: 0.85, // Placeholder
                    relevance: 0.82,    // Placeholder
                    coherence: 0.88,    // Placeholder
                    latency_ms: r.duration_ms,
                    cost,
                }
            })
            .collect();

        let latency_data = ChartDataFormatter::format_latency_histogram(results, 10);
        let metrics_data = ChartDataFormatter::format_metrics_radar(results);
        let status_data = ChartDataFormatter::format_status_distribution(results);

        Ok(DashboardData {
            title: format!("{} - Benchmark Results", results.dataset_name),
            dataset_name: results.dataset_name.clone(),
            provider_name: Some(results.provider_name.clone()),
            timestamp: results.timestamp.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
            summary_cards,
            results: result_rows,
            latency_data_json: serde_json::to_string(&latency_data)?,
            metrics_data_json: serde_json::to_string(&metrics_data)?,
            status_data_json: serde_json::to_string(&status_data)?,
            chartjs_code: include_str!("assets/chartjs.min.js").to_string(),
        })
    }

    /// Prepares data for comparison dashboard.
    fn prepare_comparison_data(&self, all_results: &[BenchmarkResults]) -> Result<DashboardData> {
        let comparison_data = ChartDataFormatter::format_comparison_bar(all_results);
        let scatter_data = ChartDataFormatter::format_cost_quality_scatter(all_results);

        // Create aggregate summary cards
        let total_tests: usize = all_results.iter().map(|r| r.summary.total).sum();
        let avg_success_rate: f64 = all_results
            .iter()
            .map(|r| r.summary.success_rate)
            .sum::<f64>()
            / all_results.len() as f64;

        let summary_cards = vec![
            SummaryCard {
                title: "Models Compared".to_string(),
                value: all_results.len().to_string(),
                change: None,
                card_class: "card-primary".to_string(),
            },
            SummaryCard {
                title: "Total Tests".to_string(),
                value: total_tests.to_string(),
                change: None,
                card_class: "card-info".to_string(),
            },
            SummaryCard {
                title: "Avg Success Rate".to_string(),
                value: format!("{:.1}%", avg_success_rate * 100.0),
                change: None,
                card_class: "card-success".to_string(),
            },
        ];

        Ok(DashboardData {
            title: "Model Comparison Dashboard".to_string(),
            dataset_name: "Multiple Datasets".to_string(),
            provider_name: None,
            timestamp: chrono::Utc::now()
                .format("%Y-%m-%d %H:%M:%S UTC")
                .to_string(),
            summary_cards,
            results: vec![],
            latency_data_json: serde_json::to_string(&comparison_data)?,
            metrics_data_json: serde_json::to_string(&scatter_data)?,
            status_data_json: "{}".to_string(),
            chartjs_code: include_str!("assets/chartjs.min.js").to_string(),
        })
    }

    /// Prepares data for trend analysis dashboard.
    fn prepare_trend_data(
        &self,
        historical_results: &[BenchmarkResults],
    ) -> Result<DashboardData> {
        let trend_data = ChartDataFormatter::format_trend_analysis(historical_results);

        let summary_cards = vec![
            SummaryCard {
                title: "Data Points".to_string(),
                value: historical_results.len().to_string(),
                change: None,
                card_class: "card-primary".to_string(),
            },
        ];

        Ok(DashboardData {
            title: "Performance Trend Analysis".to_string(),
            dataset_name: "Historical Data".to_string(),
            provider_name: None,
            timestamp: chrono::Utc::now()
                .format("%Y-%m-%d %H:%M:%S UTC")
                .to_string(),
            summary_cards,
            results: vec![],
            latency_data_json: serde_json::to_string(&trend_data)?,
            metrics_data_json: "{}".to_string(),
            status_data_json: "{}".to_string(),
            chartjs_code: include_str!("assets/chartjs.min.js").to_string(),
        })
    }
}

fn theme_to_string(theme: Theme) -> &'static str {
    match theme {
        Theme::Light => "light",
        Theme::Dark => "dark",
        Theme::Auto => "auto",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::benchmarks::results::TestResult;
    use crate::providers::{CompletionResponse, FinishReason, TokenUsage};
    use chrono::Utc;
    use std::time::Duration;

    fn create_test_results() -> BenchmarkResults {
        let response = CompletionResponse {
            id: "test-resp".to_string(),
            model: "gpt-4".to_string(),
            content: "test content".to_string(),
            usage: TokenUsage::new(100, 50),
            finish_reason: FinishReason::Stop,
            created_at: Utc::now(),
        };

        let result = TestResult::success(
            "test-1".to_string(),
            Some("category".to_string()),
            response,
            Duration::from_millis(1234),
        );

        let mut results = BenchmarkResults::new(
            "test-dataset".to_string(),
            "openai".to_string(),
            vec![result],
        );
        results.calculate_summary();
        results
    }

    #[test]
    fn test_dashboard_generator_new() {
        let generator = DashboardGenerator::new();
        assert!(generator.is_ok());
    }

    #[test]
    fn test_prepare_benchmark_data() {
        let generator = DashboardGenerator::new().unwrap();
        let results = create_test_results();

        let data = generator.prepare_benchmark_data(&results);
        assert!(data.is_ok());

        let data = data.unwrap();
        assert_eq!(data.dataset_name, "test-dataset");
        assert_eq!(data.provider_name, Some("openai".to_string()));
        assert_eq!(data.summary_cards.len(), 4);
        assert_eq!(data.results.len(), 1);
    }

    #[test]
    fn test_generate_benchmark_dashboard() {
        let generator = DashboardGenerator::new().unwrap();
        let results = create_test_results();
        let config = DashboardConfig::default();

        let html = generator.generate_benchmark_dashboard(&results, &config);
        assert!(html.is_ok());

        let html = html.unwrap();
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("test-dataset"));
        assert!(html.contains("Chart"));
    }

    #[test]
    fn test_generate_comparison_dashboard() {
        let generator = DashboardGenerator::new().unwrap();
        let results1 = create_test_results();
        let results2 = create_test_results();
        let config = DashboardConfig::default();

        let html = generator.generate_comparison_dashboard(&[results1, results2], &config);
        assert!(html.is_ok());

        let html = html.unwrap();
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("Comparison"));
    }

    #[test]
    fn test_export_to_file() {
        let generator = DashboardGenerator::new().unwrap();
        let html = "<html><body>Test</body></html>";

        let temp_dir = std::env::temp_dir();
        let output_path = temp_dir.join("test_dashboard.html");

        let result = generator.export_to_file(html, &output_path);
        assert!(result.is_ok());

        // Verify file was created
        assert!(output_path.exists());

        // Clean up
        let _ = fs::remove_file(output_path);
    }

    #[test]
    fn test_dashboard_config_default() {
        let config = DashboardConfig::default();
        assert_eq!(config.theme, Theme::Auto);
        assert_eq!(config.max_data_points, 1000);
        assert!(!config.chart_colors.is_empty());
    }

    #[test]
    fn test_theme_to_string() {
        assert_eq!(theme_to_string(Theme::Light), "light");
        assert_eq!(theme_to_string(Theme::Dark), "dark");
        assert_eq!(theme_to_string(Theme::Auto), "auto");
    }
}

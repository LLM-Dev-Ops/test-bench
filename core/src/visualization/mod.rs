// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! HTML dashboard generation and visualization.
//!
//! This module provides comprehensive dashboard generation capabilities with
//! interactive Chart.js visualizations for benchmark results, model comparisons,
//! trend analysis, and cost analysis.
//!
//! # Features
//!
//! - **Self-contained HTML**: All dashboards are single-file HTML documents with
//!   embedded CSS, JavaScript, and Chart.js library
//! - **Responsive Design**: Mobile-friendly layouts that adapt to different screen sizes
//! - **Dark Mode Support**: Automatic dark mode based on system preferences
//! - **Interactive Charts**: Multiple chart types (bar, line, radar, scatter, pie/doughnut)
//! - **Fast Generation**: Efficient template rendering using Tera
//!
//! # Dashboard Types
//!
//! - [`BenchmarkResults`](DashboardType::BenchmarkResults) - Single benchmark run analysis
//! - [`ModelComparison`](DashboardType::ModelComparison) - Side-by-side model comparison
//! - [`TrendAnalysis`](DashboardType::TrendAnalysis) - Performance trends over time
//! - [`CostAnalysis`](DashboardType::CostAnalysis) - Cost efficiency analysis
//!
//! # Examples
//!
//! ## Generate a Benchmark Results Dashboard
//!
//! ```no_run
//! use llm_test_bench_core::visualization::{DashboardGenerator, DashboardConfig};
//! use llm_test_bench_core::benchmarks::results::BenchmarkResults;
//! use std::path::Path;
//!
//! # fn example() -> anyhow::Result<()> {
//! // Create generator
//! let generator = DashboardGenerator::new()?;
//!
//! // Use default config or customize
//! let config = DashboardConfig::default();
//!
//! // Generate dashboard from benchmark results
//! # let results = create_dummy_results();
//! let html = generator.generate_benchmark_dashboard(&results, &config)?;
//!
//! // Export to file
//! generator.export_to_file(&html, Path::new("dashboard.html"))?;
//! # Ok(())
//! # }
//! # fn create_dummy_results() -> BenchmarkResults {
//! #     unimplemented!()
//! # }
//! ```
//!
//! ## Generate a Model Comparison Dashboard
//!
//! ```no_run
//! use llm_test_bench_core::visualization::{DashboardGenerator, DashboardConfig, Theme};
//!
//! # fn example() -> anyhow::Result<()> {
//! let generator = DashboardGenerator::new()?;
//!
//! let mut config = DashboardConfig::default();
//! config.title = "GPT-4 vs Claude Comparison".to_string();
//! config.theme = Theme::Dark;
//!
//! # let all_results = vec![];
//! let html = generator.generate_comparison_dashboard(&all_results, &config)?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Custom Chart Data
//!
//! ```
//! use llm_test_bench_core::visualization::charts::ChartDataFormatter;
//! use llm_test_bench_core::benchmarks::results::BenchmarkResults;
//!
//! # fn example(results: &BenchmarkResults) {
//! // Create custom chart data
//! let latency_histogram = ChartDataFormatter::format_latency_histogram(results, 15);
//! let metrics_radar = ChartDataFormatter::format_metrics_radar(results);
//! let status_pie = ChartDataFormatter::format_status_distribution(results);
//!
//! // These return serde_json::Value that can be serialized for templates
//! let json = serde_json::to_string(&latency_histogram).unwrap();
//! # }
//! ```

pub mod charts;
pub mod dashboard;

// Re-export main types
pub use charts::ChartDataFormatter;
pub use dashboard::{
    DashboardConfig, DashboardData, DashboardGenerator, DashboardType, ResultRow, SummaryCard,
    Theme,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_exports() {
        // Verify all main types are accessible
        let _: Option<DashboardGenerator> = None;
        let _: Option<DashboardConfig> = None;
        let _: Option<DashboardType> = None;
        let _: Option<Theme> = None;
        let _: Option<ChartDataFormatter> = None;
    }
}

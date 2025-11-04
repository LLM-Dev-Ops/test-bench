// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Integration tests for the visualization module.

use llm_test_bench_core::benchmarks::results::{BenchmarkResults, TestResult};
use llm_test_bench_core::providers::{CompletionResponse, FinishReason, TokenUsage};
use llm_test_bench_core::visualization::{
    ChartDataFormatter, DashboardConfig, DashboardGenerator, DashboardType, Theme,
};
use std::time::Duration;

// Helper to create test results
fn create_test_result(id: &str, latency_ms: u64) -> TestResult {
    TestResult::success(
        id.to_string(),
        Some("test-category".to_string()),
        CompletionResponse {
            id: format!("resp-{}", id),
            model: "test-model".to_string(),
            content: "test content".to_string(),
            usage: TokenUsage::new(100, 50),
            finish_reason: FinishReason::Stop,
            created_at: chrono::Utc::now(),
        },
        Duration::from_millis(latency_ms),
    )
}

fn create_benchmark_results(num_tests: usize) -> BenchmarkResults {
    let results: Vec<TestResult> = (0..num_tests)
        .map(|i| create_test_result(&format!("test-{}", i), 100 + (i as u64 * 10)))
        .collect();

    let mut benchmark = BenchmarkResults::new(
        "integration-test-dataset".to_string(),
        "test-provider".to_string(),
        results,
    );
    benchmark.calculate_summary();
    benchmark
}

#[test]
fn test_dashboard_generator_initialization() {
    let generator = DashboardGenerator::new();
    assert!(generator.is_ok(), "Dashboard generator should initialize");
}

#[test]
fn test_benchmark_dashboard_generation() {
    let generator = DashboardGenerator::new().unwrap();
    let results = create_benchmark_results(10);
    let config = DashboardConfig::default();

    let html = generator.generate_benchmark_dashboard(&results, &config);
    assert!(html.is_ok(), "Should generate benchmark dashboard");

    let html = html.unwrap();
    assert!(html.contains("<!DOCTYPE html>"));
    assert!(html.contains("integration-test-dataset"));
    assert!(html.contains("Chart"));
    assert!(html.len() > 10000, "Dashboard should contain substantial content");
}

#[test]
fn test_comparison_dashboard_generation() {
    let generator = DashboardGenerator::new().unwrap();
    let results1 = create_benchmark_results(5);
    let results2 = create_benchmark_results(5);
    let config = DashboardConfig::default();

    let html = generator.generate_comparison_dashboard(&[results1, results2], &config);
    assert!(html.is_ok(), "Should generate comparison dashboard");

    let html = html.unwrap();
    assert!(html.contains("<!DOCTYPE html>"));
    assert!(html.contains("Comparison"));
}

#[test]
fn test_trend_dashboard_generation() {
    let generator = DashboardGenerator::new().unwrap();
    let results = vec![
        create_benchmark_results(5),
        create_benchmark_results(5),
        create_benchmark_results(5),
    ];
    let config = DashboardConfig::default();

    let html = generator.generate_trend_dashboard(&results, &config);
    assert!(html.is_ok(), "Should generate trend dashboard");

    let html = html.unwrap();
    assert!(html.contains("<!DOCTYPE html>"));
    assert!(html.contains("Trend"));
}

#[test]
fn test_dashboard_with_custom_config() {
    let generator = DashboardGenerator::new().unwrap();
    let results = create_benchmark_results(5);

    let config = DashboardConfig {
        title: "Custom Title".to_string(),
        theme: Theme::Dark,
        max_data_points: 500,
        chart_colors: vec!["rgb(255, 0, 0)".to_string()],
    };

    let html = generator.generate_benchmark_dashboard(&results, &config);
    assert!(html.is_ok());

    let html = html.unwrap();
    assert!(html.contains("Custom Title"));
}

#[test]
fn test_dashboard_themes() {
    let generator = DashboardGenerator::new().unwrap();
    let results = create_benchmark_results(3);

    for theme in [Theme::Light, Theme::Dark, Theme::Auto] {
        let config = DashboardConfig {
            theme,
            ..Default::default()
        };

        let html = generator.generate_benchmark_dashboard(&results, &config);
        assert!(html.is_ok(), "Should generate dashboard with {:?} theme", theme);
    }
}

#[test]
fn test_dashboard_file_export() {
    let generator = DashboardGenerator::new().unwrap();
    let html = "<html><body>Test</body></html>";

    let temp_dir = std::env::temp_dir();
    let output_path = temp_dir.join("test_dashboard_export.html");

    let result = generator.export_to_file(html, &output_path);
    assert!(result.is_ok(), "Should export to file");
    assert!(output_path.exists(), "File should exist");

    // Clean up
    let _ = std::fs::remove_file(output_path);
}

#[test]
fn test_dashboard_with_empty_results() {
    let generator = DashboardGenerator::new().unwrap();
    let results = create_benchmark_results(0);
    let config = DashboardConfig::default();

    let html = generator.generate_benchmark_dashboard(&results, &config);
    assert!(html.is_ok(), "Should handle empty results gracefully");
}

#[test]
fn test_dashboard_with_large_dataset() {
    let generator = DashboardGenerator::new().unwrap();
    let results = create_benchmark_results(100);
    let config = DashboardConfig::default();

    let start = std::time::Instant::now();
    let html = generator.generate_benchmark_dashboard(&results, &config);
    let duration = start.elapsed();

    assert!(html.is_ok(), "Should generate dashboard with large dataset");
    assert!(duration.as_secs() < 5, "Should complete within 5 seconds");
}

#[test]
fn test_dashboard_html_validity() {
    let generator = DashboardGenerator::new().unwrap();
    let results = create_benchmark_results(5);
    let config = DashboardConfig::default();

    let html = generator.generate_benchmark_dashboard(&results, &config).unwrap();

    // Check for required HTML elements
    assert!(html.contains("<!DOCTYPE html>"));
    assert!(html.contains("<html"));
    assert!(html.contains("<head>"));
    assert!(html.contains("<body>"));
    assert!(html.contains("</html>"));

    // Check for Chart.js
    assert!(html.contains("Chart"));

    // Check for responsive meta tag
    assert!(html.contains("viewport"));
}

#[test]
fn test_chart_data_latency_histogram() {
    let results = create_benchmark_results(20);
    let chart_data = ChartDataFormatter::format_latency_histogram(&results, 10);

    assert!(chart_data["labels"].is_array());
    assert!(chart_data["datasets"].is_array());
    assert_eq!(chart_data["labels"].as_array().unwrap().len(), 10);
}

#[test]
fn test_chart_data_metrics_radar() {
    let results = create_benchmark_results(10);
    let chart_data = ChartDataFormatter::format_metrics_radar(&results);

    assert!(chart_data["labels"].is_array());
    assert!(chart_data["datasets"].is_array());
    assert_eq!(chart_data["labels"].as_array().unwrap().len(), 5);
}

#[test]
fn test_chart_data_comparison_bar() {
    let results1 = create_benchmark_results(5);
    let results2 = create_benchmark_results(5);

    let chart_data = ChartDataFormatter::format_comparison_bar(&[results1, results2]);

    assert!(chart_data["labels"].is_array());
    assert!(chart_data["datasets"].is_array());
    assert_eq!(chart_data["labels"].as_array().unwrap().len(), 2);
}

#[test]
fn test_chart_data_scatter_plot() {
    let results1 = create_benchmark_results(3);
    let results2 = create_benchmark_results(3);

    let chart_data = ChartDataFormatter::format_cost_quality_scatter(&[results1, results2]);

    assert!(chart_data["datasets"].is_array());
    let data_points = chart_data["datasets"][0]["data"].as_array().unwrap();
    assert_eq!(data_points.len(), 2);
}

#[test]
fn test_chart_data_trend_analysis() {
    let results = vec![
        create_benchmark_results(5),
        create_benchmark_results(5),
        create_benchmark_results(5),
    ];

    let chart_data = ChartDataFormatter::format_trend_analysis(&results);

    assert!(chart_data["labels"].is_array());
    assert!(chart_data["datasets"].is_array());
    assert_eq!(chart_data["labels"].as_array().unwrap().len(), 3);
}

#[test]
fn test_chart_data_status_distribution() {
    let results = create_benchmark_results(10);
    let chart_data = ChartDataFormatter::format_status_distribution(&results);

    assert!(chart_data["labels"].is_array());
    assert!(chart_data["datasets"].is_array());
    assert_eq!(chart_data["labels"].as_array().unwrap().len(), 4);
}

#[test]
fn test_dashboard_size_constraint() {
    let generator = DashboardGenerator::new().unwrap();
    let results = create_benchmark_results(100);
    let config = DashboardConfig::default();

    let html = generator.generate_benchmark_dashboard(&results, &config).unwrap();

    // Check that dashboard is under 500KB
    let size_bytes = html.len();
    let size_kb = size_bytes / 1024;

    assert!(
        size_kb < 500,
        "Dashboard size should be under 500KB, got {}KB",
        size_kb
    );
}

#[test]
fn test_chart_data_serialization() {
    let results = create_benchmark_results(5);
    let chart_data = ChartDataFormatter::format_latency_histogram(&results, 5);

    let json_str = serde_json::to_string(&chart_data);
    assert!(json_str.is_ok(), "Chart data should serialize to JSON");

    let json = json_str.unwrap();
    assert!(!json.is_empty());
    assert!(json.contains("labels"));
    assert!(json.contains("datasets"));
}

#[test]
fn test_dashboard_config_defaults() {
    let config = DashboardConfig::default();

    assert_eq!(config.title, "LLM Test Bench Dashboard");
    assert_eq!(config.theme, Theme::Auto);
    assert_eq!(config.max_data_points, 1000);
    assert!(!config.chart_colors.is_empty());
}

#[test]
fn test_multiple_dashboards_generation() {
    let generator = DashboardGenerator::new().unwrap();
    let results = create_benchmark_results(10);
    let config = DashboardConfig::default();

    // Generate multiple dashboards to test stability
    for _ in 0..5 {
        let html = generator.generate_benchmark_dashboard(&results, &config);
        assert!(html.is_ok());
    }
}

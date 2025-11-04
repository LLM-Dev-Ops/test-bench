// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Visualization module demonstration.
//!
//! This example demonstrates how to generate various types of dashboards
//! using the visualization module.
//!
//! Run with:
//! ```bash
//! cargo run --example visualization_demo
//! ```

use llm_test_bench_core::benchmarks::results::{BenchmarkResults, TestResult};
use llm_test_bench_core::providers::{CompletionResponse, FinishReason, TokenUsage};
use llm_test_bench_core::visualization::{DashboardConfig, DashboardGenerator, Theme};
use std::path::Path;
use std::time::Duration;

fn main() -> anyhow::Result<()> {
    println!("🎨 LLM Test Bench - Visualization Demo\n");

    // Create demo data
    let benchmark_results = create_demo_benchmark_results();
    let comparison_results = create_comparison_results();

    // Initialize generator
    let generator = DashboardGenerator::new()?;

    // 1. Generate Benchmark Results Dashboard
    println!("📊 Generating Benchmark Results Dashboard...");
    let config = DashboardConfig {
        title: "Demo Benchmark Results".to_string(),
        theme: Theme::Auto,
        max_data_points: 1000,
        chart_colors: vec![
            "rgb(59, 130, 246)".to_string(),
            "rgb(16, 185, 129)".to_string(),
            "rgb(245, 158, 11)".to_string(),
        ],
    };

    let html = generator.generate_benchmark_dashboard(&benchmark_results, &config)?;
    generator.export_to_file(&html, Path::new("demo_benchmark.html"))?;
    println!("✅ Saved to: demo_benchmark.html");

    // 2. Generate Model Comparison Dashboard
    println!("\n⚖️  Generating Model Comparison Dashboard...");
    let comparison_html = generator.generate_comparison_dashboard(&comparison_results, &config)?;
    generator.export_to_file(&comparison_html, Path::new("demo_comparison.html"))?;
    println!("✅ Saved to: demo_comparison.html");

    // 3. Generate Trend Analysis Dashboard
    println!("\n📈 Generating Trend Analysis Dashboard...");
    let trend_html = generator.generate_trend_dashboard(&comparison_results, &config)?;
    generator.export_to_file(&trend_html, Path::new("demo_trends.html"))?;
    println!("✅ Saved to: demo_trends.html");

    // 4. Dark Mode Dashboard
    println!("\n🌙 Generating Dark Mode Dashboard...");
    let mut dark_config = config.clone();
    dark_config.title = "Dark Mode Dashboard".to_string();
    dark_config.theme = Theme::Dark;

    let dark_html = generator.generate_benchmark_dashboard(&benchmark_results, &dark_config)?;
    generator.export_to_file(&dark_html, Path::new("demo_dark.html"))?;
    println!("✅ Saved to: demo_dark.html");

    println!("\n🎉 All dashboards generated successfully!");
    println!("\nOpen the HTML files in your browser to view the interactive dashboards.");

    // Print summary
    println!("\n📋 Dashboard Summary:");
    println!("  • Benchmark Results: {} tests", benchmark_results.total_tests);
    println!("  • Success Rate: {:.1}%", benchmark_results.summary.success_rate * 100.0);
    println!("  • Avg Latency: {:.0}ms", benchmark_results.summary.avg_duration_ms);
    println!("  • Models Compared: {}", comparison_results.len());

    Ok(())
}

/// Creates demo benchmark results with realistic data
fn create_demo_benchmark_results() -> BenchmarkResults {
    let mut results = Vec::new();

    // Create successful tests with varying latencies
    for i in 0..50 {
        let latency = 100 + (i * 30) + (i % 10) * 20;
        let prompt_tokens = 50 + (i * 5);
        let completion_tokens = 100 + (i * 3);

        let response = CompletionResponse {
            id: format!("resp-{}", i),
            model: "gpt-4".to_string(),
            content: format!("Response content for test {}", i),
            usage: TokenUsage::new(prompt_tokens, completion_tokens),
            finish_reason: FinishReason::Stop,
            created_at: chrono::Utc::now(),
        };

        results.push(TestResult::success(
            format!("test-{}", i),
            Some(format!("category-{}", i % 5)),
            response,
            Duration::from_millis(latency),
        ));
    }

    // Add some failed tests
    for i in 50..53 {
        results.push(TestResult::failure(
            format!("test-{}", i),
            Some("error-category".to_string()),
            "API rate limit exceeded".to_string(),
            Duration::from_millis(500),
        ));
    }

    // Add some timeouts
    for i in 53..55 {
        results.push(TestResult::timeout(
            format!("test-{}", i),
            Some("timeout-category".to_string()),
            Duration::from_millis(30000),
        ));
    }

    let mut benchmark = BenchmarkResults::new(
        "demo-dataset".to_string(),
        "openai".to_string(),
        results,
    );

    benchmark.calculate_summary();
    benchmark
}

/// Creates multiple benchmark results for comparison
fn create_comparison_results() -> Vec<BenchmarkResults> {
    vec![
        create_model_results("gpt-4", 50, 200, 0.95),
        create_model_results("gpt-3.5-turbo", 30, 120, 0.88),
        create_model_results("claude-3-opus", 55, 250, 0.97),
        create_model_results("claude-3-sonnet", 40, 180, 0.92),
    ]
}

/// Creates benchmark results for a specific model
fn create_model_results(
    model: &str,
    base_latency: u64,
    prompt_tokens: usize,
    success_rate: f64,
) -> BenchmarkResults {
    let mut results = Vec::new();
    let num_tests = 30;
    let num_successes = (num_tests as f64 * success_rate) as usize;

    // Successful tests
    for i in 0..num_successes {
        let latency = base_latency + (i as u64 * 10);
        let response = CompletionResponse {
            id: format!("{}-resp-{}", model, i),
            model: model.to_string(),
            content: format!("Response from {}", model),
            usage: TokenUsage::new(prompt_tokens, prompt_tokens / 2),
            finish_reason: FinishReason::Stop,
            created_at: chrono::Utc::now(),
        };

        results.push(TestResult::success(
            format!("{}-test-{}", model, i),
            Some("comparison".to_string()),
            response,
            Duration::from_millis(latency),
        ));
    }

    // Failed tests
    for i in num_successes..num_tests {
        results.push(TestResult::failure(
            format!("{}-test-{}", model, i),
            Some("comparison".to_string()),
            "Request failed".to_string(),
            Duration::from_millis(100),
        ));
    }

    let mut benchmark = BenchmarkResults::new(
        "comparison-dataset".to_string(),
        model.to_string(),
        results,
    );

    benchmark.calculate_summary();
    benchmark
}

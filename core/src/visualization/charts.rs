// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Chart data formatting for Chart.js visualizations.
//!
//! This module provides functions to convert benchmark results into Chart.js
//! compatible JSON data structures for various chart types.

use crate::benchmarks::results::{BenchmarkResults, TestResult, TestStatus};
use serde_json::{json, Value};
use std::collections::HashMap;

/// Formats chart data for Chart.js consumption.
pub struct ChartDataFormatter;

impl ChartDataFormatter {
    /// Creates a latency distribution histogram showing the distribution of response times.
    ///
    /// # Arguments
    ///
    /// * `results` - Benchmark results to analyze
    /// * `bin_count` - Number of histogram bins (default: 10)
    ///
    /// # Returns
    ///
    /// Chart.js bar chart data structure
    pub fn format_latency_histogram(results: &BenchmarkResults, bin_count: usize) -> Value {
        let durations: Vec<u64> = results
            .results
            .iter()
            .filter(|r| r.status == TestStatus::Success)
            .map(|r| r.duration_ms)
            .collect();

        if durations.is_empty() {
            return json!({
                "labels": [],
                "datasets": [{
                    "label": "Latency Distribution",
                    "data": [],
                    "backgroundColor": "rgba(59, 130, 246, 0.5)",
                    "borderColor": "rgb(59, 130, 246)",
                    "borderWidth": 1
                }]
            });
        }

        let min_duration = *durations.iter().min().unwrap();
        let max_duration = *durations.iter().max().unwrap();
        let bin_width = (max_duration - min_duration + 1) / bin_count as u64;

        let mut bins = vec![0; bin_count];
        for duration in &durations {
            let bin_idx = if bin_width > 0 {
                ((duration - min_duration) / bin_width).min((bin_count - 1) as u64) as usize
            } else {
                0
            };
            bins[bin_idx] += 1;
        }

        let labels: Vec<String> = (0..bin_count)
            .map(|i| {
                let start = min_duration + i as u64 * bin_width;
                let end = start + bin_width;
                format!("{}-{}ms", start, end)
            })
            .collect();

        json!({
            "labels": labels,
            "datasets": [{
                "label": "Request Count",
                "data": bins,
                "backgroundColor": "rgba(59, 130, 246, 0.5)",
                "borderColor": "rgb(59, 130, 246)",
                "borderWidth": 1
            }]
        })
    }

    /// Creates a radar chart showing evaluation metrics (faithfulness, relevance, coherence).
    ///
    /// Note: This assumes evaluation metrics are stored in the response metadata.
    /// If evaluation data is available separately, pass it as an argument.
    pub fn format_metrics_radar(results: &BenchmarkResults) -> Value {
        // For now, use dummy data. In a real implementation, this would
        // aggregate actual evaluation metrics from the results.
        let avg_metrics = Self::calculate_average_metrics(results);

        json!({
            "labels": ["Faithfulness", "Relevance", "Coherence", "Fluency", "Correctness"],
            "datasets": [{
                "label": &results.provider_name,
                "data": [
                    avg_metrics.get("faithfulness").unwrap_or(&0.0),
                    avg_metrics.get("relevance").unwrap_or(&0.0),
                    avg_metrics.get("coherence").unwrap_or(&0.0),
                    avg_metrics.get("fluency").unwrap_or(&0.0),
                    avg_metrics.get("correctness").unwrap_or(&0.0)
                ],
                "backgroundColor": "rgba(59, 130, 246, 0.2)",
                "borderColor": "rgb(59, 130, 246)",
                "borderWidth": 2,
                "pointBackgroundColor": "rgb(59, 130, 246)"
            }]
        })
    }

    /// Creates a comparison bar chart comparing multiple models across metrics.
    pub fn format_comparison_bar(all_results: &[BenchmarkResults]) -> Value {
        let metrics = vec!["Success Rate", "Avg Latency (ms)", "P95 Latency (ms)"];
        let labels: Vec<&str> = all_results
            .iter()
            .map(|r| r.provider_name.as_str())
            .collect();

        // Prepare datasets for each metric
        let success_rates: Vec<f64> = all_results
            .iter()
            .map(|r| r.summary.success_rate * 100.0)
            .collect();

        let avg_latencies: Vec<f64> = all_results
            .iter()
            .map(|r| r.summary.avg_duration_ms)
            .collect();

        let p95_latencies: Vec<f64> = all_results
            .iter()
            .map(|r| r.summary.p95_duration_ms)
            .collect();

        json!({
            "labels": labels,
            "datasets": [
                {
                    "label": "Success Rate (%)",
                    "data": success_rates,
                    "backgroundColor": "rgba(16, 185, 129, 0.5)",
                    "borderColor": "rgb(16, 185, 129)",
                    "borderWidth": 1,
                    "yAxisID": "y"
                },
                {
                    "label": "Avg Latency (ms)",
                    "data": avg_latencies,
                    "backgroundColor": "rgba(59, 130, 246, 0.5)",
                    "borderColor": "rgb(59, 130, 246)",
                    "borderWidth": 1,
                    "yAxisID": "y1"
                },
                {
                    "label": "P95 Latency (ms)",
                    "data": p95_latencies,
                    "backgroundColor": "rgba(245, 158, 11, 0.5)",
                    "borderColor": "rgb(245, 158, 11)",
                    "borderWidth": 1,
                    "yAxisID": "y1"
                }
            ]
        })
    }

    /// Creates a scatter plot showing cost vs quality tradeoffs.
    pub fn format_cost_quality_scatter(all_results: &[BenchmarkResults]) -> Value {
        let data_points: Vec<Value> = all_results
            .iter()
            .map(|r| {
                json!({
                    "x": r.summary.total_cost,
                    "y": r.summary.success_rate * 100.0,
                    "label": &r.provider_name
                })
            })
            .collect();

        json!({
            "datasets": [{
                "label": "Cost vs Quality",
                "data": data_points,
                "backgroundColor": "rgba(59, 130, 246, 0.5)",
                "borderColor": "rgb(59, 130, 246)",
                "pointRadius": 8,
                "pointHoverRadius": 10
            }]
        })
    }

    /// Creates a time series chart showing performance trends over time.
    pub fn format_trend_analysis(historical_results: &[BenchmarkResults]) -> Value {
        // Sort by timestamp
        let mut sorted = historical_results.to_vec();
        sorted.sort_by_key(|r| r.timestamp);

        let labels: Vec<String> = sorted
            .iter()
            .map(|r| r.timestamp.format("%Y-%m-%d %H:%M").to_string())
            .collect();

        let success_rates: Vec<f64> = sorted
            .iter()
            .map(|r| r.summary.success_rate * 100.0)
            .collect();

        let avg_latencies: Vec<f64> = sorted
            .iter()
            .map(|r| r.summary.avg_duration_ms)
            .collect();

        json!({
            "labels": labels,
            "datasets": [
                {
                    "label": "Success Rate (%)",
                    "data": success_rates,
                    "borderColor": "rgb(16, 185, 129)",
                    "backgroundColor": "rgba(16, 185, 129, 0.1)",
                    "tension": 0.4,
                    "fill": true,
                    "yAxisID": "y"
                },
                {
                    "label": "Avg Latency (ms)",
                    "data": avg_latencies,
                    "borderColor": "rgb(59, 130, 246)",
                    "backgroundColor": "rgba(59, 130, 246, 0.1)",
                    "tension": 0.4,
                    "fill": true,
                    "yAxisID": "y1"
                }
            ]
        })
    }

    /// Creates a pie chart showing the distribution of test statuses.
    pub fn format_status_distribution(results: &BenchmarkResults) -> Value {
        json!({
            "labels": ["Success", "Failed", "Timeout", "Skipped"],
            "datasets": [{
                "data": [
                    results.summary.succeeded,
                    results.summary.failed,
                    results.summary.timeout,
                    results.summary.skipped
                ],
                "backgroundColor": [
                    "rgba(16, 185, 129, 0.5)",
                    "rgba(239, 68, 68, 0.5)",
                    "rgba(245, 158, 11, 0.5)",
                    "rgba(156, 163, 175, 0.5)"
                ],
                "borderColor": [
                    "rgb(16, 185, 129)",
                    "rgb(239, 68, 68)",
                    "rgb(245, 158, 11)",
                    "rgb(156, 163, 175)"
                ],
                "borderWidth": 1
            }]
        })
    }

    /// Helper to calculate average metrics from results.
    /// This is a placeholder - real implementation would extract from actual evaluation data.
    fn calculate_average_metrics(results: &BenchmarkResults) -> HashMap<String, f64> {
        // In a real implementation, this would aggregate actual evaluation metrics
        // For now, return placeholder data based on success rate
        let base_score = results.summary.success_rate * 0.9;

        let mut metrics = HashMap::new();
        metrics.insert("faithfulness".to_string(), base_score + 0.05);
        metrics.insert("relevance".to_string(), base_score);
        metrics.insert("coherence".to_string(), base_score + 0.03);
        metrics.insert("fluency".to_string(), base_score + 0.02);
        metrics.insert("correctness".to_string(), base_score - 0.02);

        metrics
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::benchmarks::results::{ResultSummary, TestResult};
    use crate::providers::{CompletionResponse, FinishReason, TokenUsage};
    use chrono::Utc;
    use std::time::Duration;

    fn create_test_result(duration_ms: u64) -> TestResult {
        TestResult::success(
            format!("test-{}", duration_ms),
            None,
            CompletionResponse {
                id: "resp".to_string(),
                model: "test-model".to_string(),
                content: "test".to_string(),
                usage: TokenUsage::new(10, 5),
                finish_reason: FinishReason::Stop,
                created_at: Utc::now(),
            },
            Duration::from_millis(duration_ms),
        )
    }

    fn create_benchmark_results(durations: Vec<u64>) -> BenchmarkResults {
        let results: Vec<TestResult> = durations
            .into_iter()
            .map(create_test_result)
            .collect();

        let mut benchmark = BenchmarkResults::new(
            "test-dataset".to_string(),
            "test-provider".to_string(),
            results,
        );
        benchmark.calculate_summary();
        benchmark
    }

    #[test]
    fn test_format_latency_histogram_empty() {
        let results = create_benchmark_results(vec![]);
        let chart_data = ChartDataFormatter::format_latency_histogram(&results, 10);

        assert!(chart_data["labels"].as_array().unwrap().is_empty());
        assert!(chart_data["datasets"][0]["data"]
            .as_array()
            .unwrap()
            .is_empty());
    }

    #[test]
    fn test_format_latency_histogram() {
        let results = create_benchmark_results(vec![100, 150, 200, 250, 300]);
        let chart_data = ChartDataFormatter::format_latency_histogram(&results, 5);

        let labels = chart_data["labels"].as_array().unwrap();
        assert_eq!(labels.len(), 5);

        let data = chart_data["datasets"][0]["data"].as_array().unwrap();
        assert_eq!(data.len(), 5);

        // Verify structure
        assert!(chart_data["datasets"][0]["label"].is_string());
        assert!(chart_data["datasets"][0]["backgroundColor"].is_string());
    }

    #[test]
    fn test_format_metrics_radar() {
        let results = create_benchmark_results(vec![100, 200, 300]);
        let chart_data = ChartDataFormatter::format_metrics_radar(&results);

        let labels = chart_data["labels"].as_array().unwrap();
        assert_eq!(labels.len(), 5);
        assert_eq!(labels[0], "Faithfulness");
        assert_eq!(labels[1], "Relevance");

        let data = chart_data["datasets"][0]["data"].as_array().unwrap();
        assert_eq!(data.len(), 5);
        assert!(chart_data["datasets"][0]["label"].is_string());
    }

    #[test]
    fn test_format_comparison_bar() {
        let results1 = create_benchmark_results(vec![100, 200, 300]);
        let results2 = create_benchmark_results(vec![150, 250, 350]);

        let chart_data = ChartDataFormatter::format_comparison_bar(&[results1, results2]);

        let labels = chart_data["labels"].as_array().unwrap();
        assert_eq!(labels.len(), 2);

        let datasets = chart_data["datasets"].as_array().unwrap();
        assert_eq!(datasets.len(), 3); // Success Rate, Avg Latency, P95 Latency
    }

    #[test]
    fn test_format_cost_quality_scatter() {
        let results1 = create_benchmark_results(vec![100, 200]);
        let results2 = create_benchmark_results(vec![150, 250]);

        let chart_data = ChartDataFormatter::format_cost_quality_scatter(&[results1, results2]);

        let data_points = chart_data["datasets"][0]["data"].as_array().unwrap();
        assert_eq!(data_points.len(), 2);

        // Verify structure
        assert!(data_points[0]["x"].is_number());
        assert!(data_points[0]["y"].is_number());
        assert!(data_points[0]["label"].is_string());
    }

    #[test]
    fn test_format_trend_analysis() {
        let results1 = create_benchmark_results(vec![100, 200]);
        let results2 = create_benchmark_results(vec![150, 250]);

        let chart_data = ChartDataFormatter::format_trend_analysis(&[results1, results2]);

        let labels = chart_data["labels"].as_array().unwrap();
        assert_eq!(labels.len(), 2);

        let datasets = chart_data["datasets"].as_array().unwrap();
        assert_eq!(datasets.len(), 2); // Success Rate and Avg Latency
    }

    #[test]
    fn test_format_status_distribution() {
        let results = create_benchmark_results(vec![100, 200, 300]);
        let chart_data = ChartDataFormatter::format_status_distribution(&results);

        let labels = chart_data["labels"].as_array().unwrap();
        assert_eq!(labels.len(), 4);

        let data = chart_data["datasets"][0]["data"].as_array().unwrap();
        assert_eq!(data.len(), 4);

        // All our test results are successful
        assert_eq!(data[0].as_u64().unwrap(), 3);
        assert_eq!(data[1].as_u64().unwrap(), 0);
    }
}

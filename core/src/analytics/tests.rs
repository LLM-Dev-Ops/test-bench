// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Integration tests for analytics module.
//!
//! These tests verify that statistical analysis and cost optimization
//! work correctly together on realistic benchmark data.

#[cfg(test)]
mod tests {
    use crate::analytics::{CostOptimizer, StatisticalAnalyzer};
    use crate::benchmarks::results::{BenchmarkResults, TestResult};
    use crate::providers::{CompletionResponse, FinishReason, TokenUsage};
    use chrono::Utc;
    use std::time::Duration;

    fn create_benchmark_with_config(
        provider: &str,
        num_tests: usize,
        prompt_tokens: usize,
        completion_tokens: usize,
        duration_ms: u64,
    ) -> BenchmarkResults {
        let results: Vec<TestResult> = (0..num_tests)
            .map(|i| {
                let response = CompletionResponse {
                    id: format!("resp-{}", i),
                    model: provider.to_string(),
                    content: "test response".to_string(),
                    usage: TokenUsage::new(prompt_tokens, completion_tokens),
                    finish_reason: FinishReason::Stop,
                    created_at: Utc::now(),
                };

                TestResult::success(
                    format!("test-{}", i),
                    None,
                    response,
                    Duration::from_millis(duration_ms),
                )
            })
            .collect();

        let mut benchmark = BenchmarkResults::new(
            "integration-test".to_string(),
            provider.to_string(),
            results,
        );
        benchmark.calculate_summary();
        benchmark
    }

    #[test]
    fn test_end_to_end_statistical_comparison() {
        let analyzer = StatisticalAnalyzer::new(0.95);

        // Create baseline and optimized benchmarks
        let baseline = create_benchmark_with_config("gpt-4", 20, 500, 200, 1000);
        let optimized = create_benchmark_with_config("gpt-3.5-turbo", 20, 500, 200, 800);

        // Perform statistical comparison
        let result = analyzer
            .is_significant_improvement(&baseline, &optimized, "latency")
            .unwrap();

        assert!(result.is_significant);
        assert!(result.effect_size > 0.0); // Positive effect = improvement
    }

    #[test]
    fn test_cost_and_quality_tradeoff() {
        let optimizer = CostOptimizer::new(0.90);

        // Create benchmarks with different cost/quality profiles
        let expensive_high_quality = create_benchmark_with_config("gpt-4", 100, 500, 200, 1000);
        let cheap_good_quality = create_benchmark_with_config("gpt-3.5-turbo", 100, 500, 200, 800);

        let benchmarks = vec![expensive_high_quality, cheap_good_quality];
        let recommendation = optimizer.recommend_model(&benchmarks).unwrap();

        assert_eq!(recommendation.recommended_model, "gpt-3.5-turbo");
        assert!(recommendation.monthly_savings > 0.0);
    }

    #[test]
    fn test_pattern_detection_integration() {
        let optimizer = CostOptimizer::new(0.95);

        // Create benchmarks with various problematic patterns
        let long_prompts = create_benchmark_with_config("gpt-4", 50, 1500, 100, 1000);
        let verbose_responses = create_benchmark_with_config("gpt-4", 50, 100, 800, 1000);
        let normal_usage = create_benchmark_with_config("gpt-3.5-turbo", 50, 300, 150, 800);

        let history = vec![long_prompts, verbose_responses, normal_usage];
        let patterns = optimizer.identify_expensive_patterns(&history);

        // Should detect both long prompts and verbose responses
        assert!(patterns.len() >= 2);
    }

    #[test]
    fn test_optimization_suggestions_integration() {
        let optimizer = CostOptimizer::new(0.95);

        // Create benchmark with optimization opportunities
        let benchmark = create_benchmark_with_config("gpt-4", 150, 1200, 600, 1000);

        let suggestions = optimizer.suggest_prompt_optimizations(&benchmark);

        // Should suggest multiple optimizations
        assert!(suggestions.len() >= 3);

        // Verify suggestions cover key areas
        let has_prompt_compression = suggestions.iter().any(|s| s.title.contains("Compress"));
        let has_length_limit = suggestions.iter().any(|s| s.title.contains("Length"));
        let has_batch_processing = suggestions.iter().any(|s| s.title.contains("Batch"));

        assert!(has_prompt_compression);
        assert!(has_length_limit);
        assert!(has_batch_processing);
    }

    #[test]
    fn test_statistical_significance_with_small_difference() {
        let analyzer = StatisticalAnalyzer::new(0.95);

        // Create benchmarks with very small difference
        let baseline = create_benchmark_with_config("gpt-4", 30, 500, 200, 1000);
        let slightly_different = create_benchmark_with_config("gpt-4", 30, 500, 200, 1005);

        let result = analyzer
            .is_significant_improvement(&baseline, &slightly_different, "latency")
            .unwrap();

        // Small difference should not be significant
        assert!(!result.is_significant);
        assert!(result.effect_size.abs() < 0.2); // Negligible effect
    }

    #[test]
    fn test_cost_savings_realistic_volume() {
        let optimizer = CostOptimizer::new(0.95);

        // Test with realistic monthly volumes
        let monthly_requests = vec![10_000, 100_000, 1_000_000];

        for requests in monthly_requests {
            let savings = optimizer.calculate_savings("gpt-4", "gpt-3.5-turbo", requests);

            // Savings should scale linearly with volume
            assert!(savings > 0.0);
            assert!(savings < requests as f64 * 0.1); // Sanity check on upper bound
        }
    }

    #[test]
    fn test_confidence_intervals_coverage() {
        let analyzer = StatisticalAnalyzer::new(0.95);

        // Create multiple benchmarks and check CI coverage
        let benchmark = create_benchmark_with_config("gpt-4", 50, 500, 200, 1000);

        let latencies: Vec<f64> = benchmark
            .results
            .iter()
            .map(|r| r.duration_ms as f64)
            .collect();

        let (lower, upper) = analyzer.confidence_interval(&latencies).unwrap();
        let mean: f64 = latencies.iter().sum::<f64>() / latencies.len() as f64;

        // Mean should be within CI
        assert!(mean >= lower && mean <= upper);

        // CI should be reasonable width (not too narrow or wide)
        let width = upper - lower;
        assert!(width > 0.0);
        assert!(width < mean * 2.0); // CI shouldn't be wider than 2x the mean
    }

    #[test]
    fn test_cohens_d_effect_size_categories() {
        let analyzer = StatisticalAnalyzer::new(0.95);

        // Test small effect (d ~ 0.2)
        let baseline_small = vec![100.0, 102.0, 98.0, 101.0, 99.0];
        let comparison_small = vec![98.0, 100.0, 96.0, 99.0, 97.0];
        let d_small = analyzer.cohens_d(&baseline_small, &comparison_small);
        assert!(d_small.abs() < 0.5); // Should be small

        // Test medium effect (d ~ 0.5)
        let baseline_medium = vec![100.0, 110.0, 105.0, 95.0, 100.0];
        let comparison_medium = vec![95.0, 100.0, 92.0, 88.0, 95.0];
        let d_medium = analyzer.cohens_d(&baseline_medium, &comparison_medium);
        assert!(d_medium.abs() >= 0.3 && d_medium.abs() < 1.0); // Medium range

        // Test large effect (d ~ 0.8+)
        let baseline_large = vec![100.0, 110.0, 105.0, 95.0, 100.0];
        let comparison_large = vec![80.0, 85.0, 78.0, 82.0, 75.0];
        let d_large = analyzer.cohens_d(&baseline_large, &comparison_large);
        assert!(d_large.abs() >= 0.8); // Large effect
    }

    #[test]
    fn test_mann_whitney_robustness_to_outliers() {
        let analyzer = StatisticalAnalyzer::new(0.95);

        // Create samples with outliers
        let sample_a = vec![100.0, 101.0, 99.0, 102.0, 1000.0]; // One extreme outlier
        let sample_b = vec![50.0, 51.0, 49.0, 52.0, 48.0];

        // Mann-Whitney should still detect difference despite outlier
        let result = analyzer.mann_whitney_u(&sample_a, &sample_b).unwrap();
        assert!(result.is_significant);
    }

    #[test]
    fn test_model_recommendation_with_quality_constraint() {
        let optimizer = CostOptimizer::new(0.98); // Very high quality requirement

        // Create benchmarks where only one meets quality threshold
        let mut high_quality = create_benchmark_with_config("gpt-4", 100, 500, 200, 1000);
        high_quality.summary.success_rate = 0.99;

        let mut medium_quality = create_benchmark_with_config("gpt-3.5-turbo", 100, 500, 200, 800);
        medium_quality.summary.success_rate = 0.95;

        let benchmarks = vec![high_quality.clone(), medium_quality];
        let recommendation = optimizer.recommend_model(&benchmarks).unwrap();

        // Should recommend high quality model since medium doesn't meet threshold
        assert_eq!(recommendation.recommended_model, "gpt-4");
    }
}

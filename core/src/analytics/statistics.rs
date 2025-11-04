// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Statistical analysis for benchmark results.
//!
//! This module provides comprehensive statistical testing tools to determine
//! if performance differences between benchmarks are statistically significant.
//!
//! # Statistical Tests
//!
//! - **T-test**: Parametric test for comparing means (assumes normality)
//! - **Mann-Whitney U**: Non-parametric alternative (no normality assumption)
//! - **Effect Size (Cohen's d)**: Measures magnitude of difference
//!
//! # When to Use Each Test
//!
//! - Use t-test when data is approximately normally distributed
//! - Use Mann-Whitney U when data is skewed or has outliers
//! - Always report effect size alongside p-values
//!
//! # Examples
//!
//! ```
//! use llm_test_bench_core::analytics::StatisticalAnalyzer;
//!
//! let analyzer = StatisticalAnalyzer::new(0.95);
//!
//! // Compare two sets of latency measurements
//! let baseline = vec![100.0, 110.0, 105.0, 95.0, 102.0];
//! let optimized = vec![90.0, 88.0, 92.0, 85.0, 87.0];
//!
//! let t_result = analyzer.t_test(&baseline, &optimized).unwrap();
//! println!("P-value: {:.4}", t_result.p_value);
//! println!("Significant: {}", t_result.is_significant);
//!
//! let effect = analyzer.cohens_d(&baseline, &optimized);
//! println!("Effect size: {:.2}", effect);
//! ```

use crate::benchmarks::results::BenchmarkResults;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use statrs::distribution::{ContinuousCDF, StudentsT};

/// Statistical analyzer for benchmark results.
///
/// Provides methods for statistical hypothesis testing and effect size calculations.
#[derive(Debug, Clone)]
pub struct StatisticalAnalyzer {
    /// Confidence level for statistical tests (e.g., 0.95 for 95% confidence)
    pub confidence_level: f64,
}

impl StatisticalAnalyzer {
    /// Creates a new statistical analyzer with the specified confidence level.
    ///
    /// # Arguments
    ///
    /// * `confidence_level` - Confidence level between 0.0 and 1.0 (default: 0.95)
    ///
    /// # Examples
    ///
    /// ```
    /// use llm_test_bench_core::analytics::StatisticalAnalyzer;
    ///
    /// let analyzer = StatisticalAnalyzer::new(0.95); // 95% confidence
    /// ```
    pub fn new(confidence_level: f64) -> Self {
        Self { confidence_level }
    }

    /// Creates a default analyzer with 95% confidence level.
    pub fn default() -> Self {
        Self::new(0.95)
    }

    /// Performs a two-sample t-test (Welch's t-test).
    ///
    /// This test determines if two samples have significantly different means.
    /// Uses Welch's t-test which doesn't assume equal variances.
    ///
    /// # Arguments
    ///
    /// * `sample_a` - First sample (e.g., baseline measurements)
    /// * `sample_b` - Second sample (e.g., optimized measurements)
    ///
    /// # Returns
    ///
    /// A `TTestResult` containing the t-statistic, p-value, and significance.
    ///
    /// # Examples
    ///
    /// ```
    /// use llm_test_bench_core::analytics::StatisticalAnalyzer;
    ///
    /// let analyzer = StatisticalAnalyzer::new(0.95);
    /// let baseline = vec![100.0, 110.0, 105.0];
    /// let optimized = vec![90.0, 88.0, 92.0];
    ///
    /// let result = analyzer.t_test(&baseline, &optimized).unwrap();
    /// assert!(result.t_statistic > 0.0); // Baseline is higher
    /// ```
    pub fn t_test(&self, sample_a: &[f64], sample_b: &[f64]) -> Result<TTestResult> {
        if sample_a.is_empty() || sample_b.is_empty() {
            return Err(anyhow!("Samples cannot be empty"));
        }

        if sample_a.len() < 2 || sample_b.len() < 2 {
            return Err(anyhow!("Samples must have at least 2 observations each"));
        }

        // Calculate means
        let mean_a = mean(sample_a);
        let mean_b = mean(sample_b);

        // Calculate variances
        let var_a = variance(sample_a, mean_a);
        let var_b = variance(sample_b, mean_b);

        let n_a = sample_a.len() as f64;
        let n_b = sample_b.len() as f64;

        // Welch's t-test (doesn't assume equal variances)
        let standard_error = ((var_a / n_a) + (var_b / n_b)).sqrt();
        let t_statistic = (mean_a - mean_b) / standard_error;

        // Welch-Satterthwaite degrees of freedom
        let numerator = (var_a / n_a + var_b / n_b).powi(2);
        let denominator = (var_a / n_a).powi(2) / (n_a - 1.0) + (var_b / n_b).powi(2) / (n_b - 1.0);
        let df = numerator / denominator;

        // Calculate p-value (two-tailed)
        let dist = StudentsT::new(0.0, 1.0, df)
            .map_err(|e| anyhow!("Failed to create t-distribution: {}", e))?;
        let p_value = 2.0 * (1.0 - dist.cdf(t_statistic.abs()));

        // Determine significance
        let alpha = 1.0 - self.confidence_level;
        let is_significant = p_value < alpha;

        // Calculate confidence interval for difference in means
        let t_critical = quantile_t(self.confidence_level + (1.0 - self.confidence_level) / 2.0, df);
        let margin_of_error = t_critical * standard_error;
        let mean_diff = mean_a - mean_b;
        let confidence_interval = (mean_diff - margin_of_error, mean_diff + margin_of_error);

        Ok(TTestResult {
            t_statistic,
            p_value,
            degrees_of_freedom: df as usize,
            is_significant,
            confidence_interval,
            mean_a,
            mean_b,
        })
    }

    /// Performs a Mann-Whitney U test (non-parametric alternative to t-test).
    ///
    /// This test compares the distributions of two samples without assuming normality.
    /// It's more robust to outliers and skewed distributions.
    ///
    /// # Arguments
    ///
    /// * `sample_a` - First sample
    /// * `sample_b` - Second sample
    ///
    /// # Returns
    ///
    /// A `MannWhitneyResult` containing the U-statistic and p-value.
    ///
    /// # Examples
    ///
    /// ```
    /// use llm_test_bench_core::analytics::StatisticalAnalyzer;
    ///
    /// let analyzer = StatisticalAnalyzer::new(0.95);
    /// let sample_a = vec![1.0, 2.0, 3.0, 100.0]; // Has outlier
    /// let sample_b = vec![4.0, 5.0, 6.0, 7.0];
    ///
    /// let result = analyzer.mann_whitney_u(&sample_a, &sample_b).unwrap();
    /// ```
    pub fn mann_whitney_u(&self, sample_a: &[f64], sample_b: &[f64]) -> Result<MannWhitneyResult> {
        if sample_a.is_empty() || sample_b.is_empty() {
            return Err(anyhow!("Samples cannot be empty"));
        }

        let n1 = sample_a.len();
        let n2 = sample_b.len();

        // Combine samples with labels
        let mut combined: Vec<(f64, bool)> = Vec::with_capacity(n1 + n2);
        for &val in sample_a {
            combined.push((val, true)); // true = from sample A
        }
        for &val in sample_b {
            combined.push((val, false)); // false = from sample B
        }

        // Sort by value
        combined.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

        // Assign ranks (handling ties by averaging)
        let mut ranks_a = Vec::new();
        let mut i = 0;
        while i < combined.len() {
            let val = combined[i].0;
            let mut j = i;

            // Find all elements with the same value
            while j < combined.len() && (combined[j].0 - val).abs() < 1e-10 {
                j += 1;
            }

            // Average rank for tied values
            let rank = (i + 1 + j) as f64 / 2.0;

            // Assign average rank to all tied elements
            for k in i..j {
                if combined[k].1 {
                    ranks_a.push(rank);
                }
            }

            i = j;
        }

        // Calculate U statistic
        let r1: f64 = ranks_a.iter().sum();
        let u1 = r1 - (n1 * (n1 + 1)) as f64 / 2.0;
        let u2 = (n1 * n2) as f64 - u1;
        let u_statistic = u1.min(u2);

        // Calculate z-score for normal approximation (valid for large samples)
        let mean_u = (n1 * n2) as f64 / 2.0;
        let std_u = ((n1 * n2 * (n1 + n2 + 1)) as f64 / 12.0).sqrt();
        let z_score = (u_statistic - mean_u) / std_u;

        // Calculate p-value (two-tailed) using normal approximation
        let p_value = 2.0 * (1.0 - standard_normal_cdf(z_score.abs()));

        let alpha = 1.0 - self.confidence_level;
        let is_significant = p_value < alpha;

        Ok(MannWhitneyResult {
            u_statistic,
            p_value,
            is_significant,
            z_score,
        })
    }

    /// Calculates Cohen's d effect size.
    ///
    /// Effect size measures the magnitude of the difference between two groups,
    /// independent of sample size.
    ///
    /// # Interpretation
    ///
    /// - Small effect: d = 0.2
    /// - Medium effect: d = 0.5
    /// - Large effect: d = 0.8
    ///
    /// # Arguments
    ///
    /// * `sample_a` - First sample
    /// * `sample_b` - Second sample
    ///
    /// # Returns
    ///
    /// Cohen's d value. Positive values indicate sample_a > sample_b.
    ///
    /// # Examples
    ///
    /// ```
    /// use llm_test_bench_core::analytics::StatisticalAnalyzer;
    ///
    /// let analyzer = StatisticalAnalyzer::new(0.95);
    /// let baseline = vec![100.0, 110.0, 105.0];
    /// let optimized = vec![90.0, 88.0, 92.0];
    ///
    /// let effect = analyzer.cohens_d(&baseline, &optimized);
    /// println!("Effect size: {:.2}", effect); // Large positive effect
    /// ```
    pub fn cohens_d(&self, sample_a: &[f64], sample_b: &[f64]) -> f64 {
        if sample_a.is_empty() || sample_b.is_empty() {
            return 0.0;
        }

        let mean_a = mean(sample_a);
        let mean_b = mean(sample_b);

        let var_a = variance(sample_a, mean_a);
        let var_b = variance(sample_b, mean_b);

        let n_a = sample_a.len() as f64;
        let n_b = sample_b.len() as f64;

        // Pooled standard deviation
        let pooled_std = (((n_a - 1.0) * var_a + (n_b - 1.0) * var_b) / (n_a + n_b - 2.0)).sqrt();

        if pooled_std == 0.0 {
            return 0.0;
        }

        (mean_a - mean_b) / pooled_std
    }

    /// Calculates a confidence interval for a sample.
    ///
    /// # Arguments
    ///
    /// * `data` - Sample data
    ///
    /// # Returns
    ///
    /// A tuple (lower_bound, upper_bound) for the confidence interval.
    ///
    /// # Examples
    ///
    /// ```
    /// use llm_test_bench_core::analytics::StatisticalAnalyzer;
    ///
    /// let analyzer = StatisticalAnalyzer::new(0.95);
    /// let data = vec![100.0, 110.0, 105.0, 95.0, 102.0];
    ///
    /// let (lower, upper) = analyzer.confidence_interval(&data).unwrap();
    /// println!("95% CI: [{:.2}, {:.2}]", lower, upper);
    /// ```
    pub fn confidence_interval(&self, data: &[f64]) -> Result<(f64, f64)> {
        if data.is_empty() {
            return Err(anyhow!("Data cannot be empty"));
        }

        if data.len() == 1 {
            let val = data[0];
            return Ok((val, val));
        }

        let mean_val = mean(data);
        let std_err = standard_error(data);
        let df = (data.len() - 1) as f64;

        let t_critical = quantile_t(self.confidence_level + (1.0 - self.confidence_level) / 2.0, df);
        let margin_of_error = t_critical * std_err;

        Ok((mean_val - margin_of_error, mean_val + margin_of_error))
    }

    /// Determines if an improvement is statistically significant.
    ///
    /// This is a high-level method that analyzes two benchmark results and
    /// provides a comprehensive significance test with interpretation.
    ///
    /// # Arguments
    ///
    /// * `baseline` - Baseline benchmark results
    /// * `comparison` - Comparison benchmark results
    /// * `metric` - Metric to compare (e.g., "latency", "cost")
    ///
    /// # Returns
    ///
    /// A `SignificanceTest` with detailed results and interpretation.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use llm_test_bench_core::analytics::StatisticalAnalyzer;
    /// use llm_test_bench_core::benchmarks::results::BenchmarkResults;
    ///
    /// let analyzer = StatisticalAnalyzer::new(0.95);
    /// # let baseline = BenchmarkResults::new("test".into(), "provider".into(), vec![]);
    /// # let comparison = BenchmarkResults::new("test".into(), "provider".into(), vec![]);
    ///
    /// let test = analyzer.is_significant_improvement(&baseline, &comparison, "latency").unwrap();
    /// if test.is_significant {
    ///     println!("{}", test.interpretation);
    /// }
    /// ```
    pub fn is_significant_improvement(
        &self,
        baseline: &BenchmarkResults,
        comparison: &BenchmarkResults,
        metric: &str,
    ) -> Result<SignificanceTest> {
        // Extract metric values
        let baseline_values = extract_metric(baseline, metric)?;
        let comparison_values = extract_metric(comparison, metric)?;

        // Perform t-test
        let t_result = self.t_test(&baseline_values, &comparison_values)?;

        // Calculate effect size
        let effect_size = self.cohens_d(&baseline_values, &comparison_values);

        // Generate interpretation
        let interpretation = interpret_results(
            &t_result,
            effect_size,
            metric,
            mean(&baseline_values),
            mean(&comparison_values),
        );

        Ok(SignificanceTest {
            is_significant: t_result.is_significant,
            p_value: t_result.p_value,
            effect_size,
            interpretation,
        })
    }
}

/// Result of a t-test.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TTestResult {
    /// The t-statistic
    pub t_statistic: f64,
    /// P-value (probability of observing this difference by chance)
    pub p_value: f64,
    /// Degrees of freedom
    pub degrees_of_freedom: usize,
    /// Whether the difference is statistically significant
    pub is_significant: bool,
    /// Confidence interval for the difference in means (lower, upper)
    pub confidence_interval: (f64, f64),
    /// Mean of sample A
    pub mean_a: f64,
    /// Mean of sample B
    pub mean_b: f64,
}

/// Result of a Mann-Whitney U test.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MannWhitneyResult {
    /// The U-statistic
    pub u_statistic: f64,
    /// P-value
    pub p_value: f64,
    /// Whether the difference is statistically significant
    pub is_significant: bool,
    /// Z-score (for normal approximation)
    pub z_score: f64,
}

/// Result of a significance test with interpretation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignificanceTest {
    /// Whether the improvement is statistically significant
    pub is_significant: bool,
    /// P-value from the test
    pub p_value: f64,
    /// Effect size (Cohen's d)
    pub effect_size: f64,
    /// Plain-language interpretation of the results
    pub interpretation: String,
}

// Helper functions

fn mean(data: &[f64]) -> f64 {
    if data.is_empty() {
        return 0.0;
    }
    data.iter().sum::<f64>() / data.len() as f64
}

fn variance(data: &[f64], mean_val: f64) -> f64 {
    if data.len() < 2 {
        return 0.0;
    }
    let sum_squared_diff: f64 = data.iter().map(|&x| (x - mean_val).powi(2)).sum();
    sum_squared_diff / (data.len() - 1) as f64
}

fn standard_error(data: &[f64]) -> f64 {
    if data.is_empty() {
        return 0.0;
    }
    let mean_val = mean(data);
    let var = variance(data, mean_val);
    (var / data.len() as f64).sqrt()
}

/// Approximation of the t-distribution quantile function.
/// For more accuracy, this uses a simple approximation suitable for df > 30.
fn quantile_t(p: f64, df: f64) -> f64 {
    if df > 30.0 {
        // Use normal approximation for large df
        quantile_normal(p)
    } else {
        // For smaller df, use a better approximation
        // This is a simplified version; for production, use statrs
        let dist = StudentsT::new(0.0, 1.0, df).unwrap();
        inverse_cdf_t(&dist, p)
    }
}

fn inverse_cdf_t(dist: &StudentsT, p: f64) -> f64 {
    // Binary search for inverse CDF
    let mut low = -10.0;
    let mut high = 10.0;
    let tolerance = 1e-6;

    for _ in 0..100 {
        let mid = (low + high) / 2.0;
        let cdf_val = dist.cdf(mid);

        if (cdf_val - p).abs() < tolerance {
            return mid;
        }

        if cdf_val < p {
            low = mid;
        } else {
            high = mid;
        }
    }

    (low + high) / 2.0
}

fn quantile_normal(p: f64) -> f64 {
    // Approximation of the inverse normal CDF (quantile function)
    // Using Beasley-Springer-Moro algorithm (simplified)
    if p <= 0.0 {
        return f64::NEG_INFINITY;
    }
    if p >= 1.0 {
        return f64::INFINITY;
    }

    let a = [
        -3.969683028665376e+01,
        2.209460984245205e+02,
        -2.759285104469687e+02,
        1.383577518672690e+02,
        -3.066479806614716e+01,
        2.506628277459239e+00,
    ];

    let b = [
        -5.447609879822406e+01,
        1.615858368580409e+02,
        -1.556989798598866e+02,
        6.680131188771972e+01,
        -1.328068155288572e+01,
    ];

    let c = [
        -7.784894002430293e-03,
        -3.223964580411365e-01,
        -2.400758277161838e+00,
        -2.549732539343734e+00,
        4.374664141464968e+00,
        2.938163982698783e+00,
    ];

    let d = [
        7.784695709041462e-03,
        3.224671290700398e-01,
        2.445134137142996e+00,
        3.754408661907416e+00,
    ];

    let p_low = 0.02425;
    let p_high = 1.0 - p_low;

    if p < p_low {
        let q = (-2.0 * p.ln()).sqrt();
        return (((((c[0] * q + c[1]) * q + c[2]) * q + c[3]) * q + c[4]) * q + c[5])
            / ((((d[0] * q + d[1]) * q + d[2]) * q + d[3]) * q + 1.0);
    } else if p <= p_high {
        let q = p - 0.5;
        let r = q * q;
        return (((((a[0] * r + a[1]) * r + a[2]) * r + a[3]) * r + a[4]) * r + a[5]) * q
            / (((((b[0] * r + b[1]) * r + b[2]) * r + b[3]) * r + b[4]) * r + 1.0);
    } else {
        let q = (-2.0 * (1.0 - p).ln()).sqrt();
        return -(((((c[0] * q + c[1]) * q + c[2]) * q + c[3]) * q + c[4]) * q + c[5])
            / ((((d[0] * q + d[1]) * q + d[2]) * q + d[3]) * q + 1.0);
    }
}

fn standard_normal_cdf(x: f64) -> f64 {
    // Approximation of the standard normal CDF
    // Using the error function approximation
    0.5 * (1.0 + erf(x / std::f64::consts::SQRT_2))
}

fn erf(x: f64) -> f64 {
    // Approximation of the error function using Abramowitz and Stegun formula
    let a1 = 0.254829592;
    let a2 = -0.284496736;
    let a3 = 1.421413741;
    let a4 = -1.453152027;
    let a5 = 1.061405429;
    let p = 0.3275911;

    let sign = if x < 0.0 { -1.0 } else { 1.0 };
    let x = x.abs();

    let t = 1.0 / (1.0 + p * x);
    let y = 1.0 - (((((a5 * t + a4) * t) + a3) * t + a2) * t + a1) * t * (-x * x).exp();

    sign * y
}

fn extract_metric(results: &BenchmarkResults, metric: &str) -> Result<Vec<f64>> {
    match metric.to_lowercase().as_str() {
        "latency" | "duration" => {
            Ok(results.results.iter().map(|r| r.duration_ms as f64).collect())
        }
        "cost" => {
            let values: Vec<f64> = results
                .results
                .iter()
                .filter_map(|r| {
                    r.response.as_ref().map(|resp| {
                        let prompt_cost = (resp.usage.prompt_tokens as f64 / 1000.0) * 0.03;
                        let completion_cost = (resp.usage.completion_tokens as f64 / 1000.0) * 0.06;
                        prompt_cost + completion_cost
                    })
                })
                .collect();
            if values.is_empty() {
                Err(anyhow!("No cost data available"))
            } else {
                Ok(values)
            }
        }
        "tokens" => {
            let values: Vec<f64> = results
                .results
                .iter()
                .filter_map(|r| r.response.as_ref().map(|resp| resp.usage.total_tokens as f64))
                .collect();
            if values.is_empty() {
                Err(anyhow!("No token data available"))
            } else {
                Ok(values)
            }
        }
        _ => Err(anyhow!("Unknown metric: {}", metric)),
    }
}

fn interpret_results(
    t_result: &TTestResult,
    effect_size: f64,
    metric: &str,
    baseline_mean: f64,
    comparison_mean: f64,
) -> String {
    let effect_magnitude = if effect_size.abs() < 0.2 {
        "negligible"
    } else if effect_size.abs() < 0.5 {
        "small"
    } else if effect_size.abs() < 0.8 {
        "medium"
    } else {
        "large"
    };

    let direction = if comparison_mean < baseline_mean {
        "improvement"
    } else {
        "regression"
    };

    let percent_change = ((comparison_mean - baseline_mean) / baseline_mean * 100.0).abs();

    if t_result.is_significant {
        format!(
            "Statistically significant {} in {} (p={:.4}). \
             Effect size is {} (d={:.2}). \
             {} changed by {:.1}% (from {:.2} to {:.2}).",
            direction,
            metric,
            t_result.p_value,
            effect_magnitude,
            metric,
            percent_change,
            baseline_mean,
            comparison_mean
        )
    } else {
        format!(
            "No statistically significant difference in {} (p={:.4}). \
             Effect size is {} (d={:.2}). \
             Observed change of {:.1}% could be due to random variation.",
            metric, t_result.p_value, effect_magnitude, percent_change
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mean() {
        assert_eq!(mean(&[1.0, 2.0, 3.0, 4.0, 5.0]), 3.0);
        assert_eq!(mean(&[10.0]), 10.0);
        assert_eq!(mean(&[]), 0.0);
    }

    #[test]
    fn test_variance() {
        let data = vec![2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
        let mean_val = mean(&data);
        let var = variance(&data, mean_val);
        assert!((var - 4.571).abs() < 0.01); // Sample variance
    }

    #[test]
    fn test_cohens_d() {
        let analyzer = StatisticalAnalyzer::new(0.95);

        // Same samples should have d=0
        let sample = vec![1.0, 2.0, 3.0];
        assert_eq!(analyzer.cohens_d(&sample, &sample), 0.0);

        // Known example: one SD difference
        let a = vec![100.0, 110.0, 120.0, 130.0, 140.0];
        let b = vec![80.0, 90.0, 100.0, 110.0, 120.0];
        let d = analyzer.cohens_d(&a, &b);
        assert!((d - 1.0).abs() < 0.1); // Approximately 1 SD difference
    }

    #[test]
    fn test_t_test_same_samples() {
        let analyzer = StatisticalAnalyzer::new(0.95);
        let sample = vec![100.0, 110.0, 105.0, 95.0, 102.0];

        let result = analyzer.t_test(&sample, &sample).unwrap();
        assert!((result.t_statistic).abs() < 1e-10);
        assert!(result.p_value > 0.9); // Very high p-value
        assert!(!result.is_significant);
    }

    #[test]
    fn test_t_test_different_samples() {
        let analyzer = StatisticalAnalyzer::new(0.95);
        let baseline = vec![100.0, 110.0, 105.0, 95.0, 102.0];
        let improved = vec![80.0, 88.0, 82.0, 75.0, 77.0];

        let result = analyzer.t_test(&baseline, &improved).unwrap();
        assert!(result.t_statistic > 0.0); // Baseline is higher
        assert!(result.is_significant); // Clear difference
        assert!(result.p_value < 0.05);
    }

    #[test]
    fn test_t_test_edge_cases() {
        let analyzer = StatisticalAnalyzer::new(0.95);

        // Empty samples
        assert!(analyzer.t_test(&[], &[1.0, 2.0]).is_err());
        assert!(analyzer.t_test(&[1.0, 2.0], &[]).is_err());

        // Too small samples
        assert!(analyzer.t_test(&[1.0], &[2.0]).is_err());
    }

    #[test]
    fn test_mann_whitney_u() {
        let analyzer = StatisticalAnalyzer::new(0.95);

        let sample_a = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let sample_b = vec![6.0, 7.0, 8.0, 9.0, 10.0];

        let result = analyzer.mann_whitney_u(&sample_a, &sample_b).unwrap();
        assert!(result.is_significant); // Clear separation
        assert!(result.p_value < 0.05);
    }

    #[test]
    fn test_mann_whitney_u_with_ties() {
        let analyzer = StatisticalAnalyzer::new(0.95);

        let sample_a = vec![1.0, 2.0, 3.0, 3.0, 3.0];
        let sample_b = vec![3.0, 4.0, 5.0, 6.0, 7.0];

        let result = analyzer.mann_whitney_u(&sample_a, &sample_b).unwrap();
        // Should handle ties correctly
        assert!(result.u_statistic >= 0.0);
    }

    #[test]
    fn test_confidence_interval() {
        let analyzer = StatisticalAnalyzer::new(0.95);
        let data = vec![100.0, 110.0, 105.0, 95.0, 102.0];

        let (lower, upper) = analyzer.confidence_interval(&data).unwrap();
        let mean_val = mean(&data);

        assert!(lower < mean_val);
        assert!(upper > mean_val);
        assert!(lower < upper);
    }

    #[test]
    fn test_confidence_interval_single_value() {
        let analyzer = StatisticalAnalyzer::new(0.95);
        let data = vec![100.0];

        let (lower, upper) = analyzer.confidence_interval(&data).unwrap();
        assert_eq!(lower, 100.0);
        assert_eq!(upper, 100.0);
    }

    #[test]
    fn test_confidence_interval_empty() {
        let analyzer = StatisticalAnalyzer::new(0.95);
        assert!(analyzer.confidence_interval(&[]).is_err());
    }

    #[test]
    fn test_standard_normal_cdf() {
        // Test known values
        assert!((standard_normal_cdf(0.0) - 0.5).abs() < 0.001);
        assert!((standard_normal_cdf(1.96) - 0.975).abs() < 0.01);
        assert!((standard_normal_cdf(-1.96) - 0.025).abs() < 0.01);
    }

    #[test]
    fn test_extract_metric_latency() {
        use crate::benchmarks::results::{BenchmarkResults, TestResult};
        use crate::providers::{CompletionResponse, FinishReason, TokenUsage};
        use chrono::Utc;
        use std::time::Duration;

        let response = CompletionResponse {
            id: "test".to_string(),
            model: "gpt-4".to_string(),
            content: "test".to_string(),
            usage: TokenUsage::new(10, 5),
            finish_reason: FinishReason::Stop,
            created_at: Utc::now(),
        };

        let result = TestResult::success(
            "test-1".to_string(),
            None,
            response,
            Duration::from_millis(1000),
        );

        let results = BenchmarkResults::new("test".to_string(), "provider".to_string(), vec![result]);

        let latencies = extract_metric(&results, "latency").unwrap();
        assert_eq!(latencies, vec![1000.0]);
    }

    #[test]
    fn test_statistical_analyzer_default() {
        let analyzer = StatisticalAnalyzer::default();
        assert_eq!(analyzer.confidence_level, 0.95);
    }
}

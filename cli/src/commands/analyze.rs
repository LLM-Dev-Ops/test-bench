use anyhow::{Context, Result};
use clap::Args;
use colored::Colorize;
use llm_test_bench_core::config::{AnalyticsConfig, ConfigLoader};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Args, Debug)]
pub struct AnalyzeArgs {
    /// Baseline results file
    #[arg(short, long, required = true)]
    pub baseline: PathBuf,

    /// Comparison results file
    #[arg(short, long, required = true)]
    pub comparison: PathBuf,

    /// Metric to analyze
    #[arg(short, long, default_value = "overall")]
    pub metric: String,

    /// Confidence level (0.90, 0.95, 0.99)
    #[arg(long, default_value = "0.95")]
    pub confidence_level: f64,

    /// Fail if regression detected
    #[arg(long)]
    pub fail_on_regression: bool,

    /// Effect size threshold for practical significance
    #[arg(long, default_value = "0.2")]
    pub effect_size_threshold: f64,

    /// Output format
    #[arg(short, long, default_value = "detailed")]
    pub output: OutputFormat,

    /// Save report to file
    #[arg(long)]
    pub report_file: Option<PathBuf>,

    /// Path to custom configuration file
    #[arg(long)]
    pub config: Option<PathBuf>,
}

#[derive(Debug, Clone, clap::ValueEnum)]
pub enum OutputFormat {
    Detailed,
    Summary,
    Json,
}

#[derive(Debug, Serialize, Deserialize)]
struct AnalysisReport {
    baseline: ResultsSummary,
    comparison: ResultsSummary,
    metric: String,
    statistical_tests: StatisticalTestResults,
    interpretation: Interpretation,
    recommendations: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ResultsSummary {
    file: String,
    total_tests: usize,
    mean: f64,
    std_dev: f64,
    min: f64,
    max: f64,
    median: f64,
    p95: f64,
}

#[derive(Debug, Serialize, Deserialize)]
struct StatisticalTestResults {
    confidence_level: f64,
    t_statistic: f64,
    p_value: f64,
    degrees_of_freedom: usize,
    effect_size: f64,
    effect_size_interpretation: String,
    is_significant: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct Interpretation {
    regression_detected: bool,
    improvement_detected: bool,
    practically_significant: bool,
    summary: String,
    details: Vec<String>,
}

pub async fn execute(args: AnalyzeArgs, verbose: bool) -> Result<()> {
    println!("{}", "LLM Test Bench - Analyze Command".bold().cyan());
    println!();

    // Validate input files
    if !args.baseline.exists() {
        anyhow::bail!("Baseline file not found: {}", args.baseline.display());
    }
    if !args.comparison.exists() {
        anyhow::bail!("Comparison file not found: {}", args.comparison.display());
    }

    // Validate confidence level
    if args.confidence_level < 0.8 || args.confidence_level > 0.999 {
        anyhow::bail!(
            "Confidence level must be between 0.8 and 0.999, got: {}",
            args.confidence_level
        );
    }

    if verbose {
        println!("{}", "Configuration:".bold());
        println!("  Baseline: {}", args.baseline.display());
        println!("  Comparison: {}", args.comparison.display());
        println!("  Metric: {}", args.metric);
        println!("  Confidence level: {:.0}%", args.confidence_level * 100.0);
        println!("  Effect size threshold: {}", args.effect_size_threshold);
        println!();
    }

    // Load configuration
    let config_loader = if let Some(ref config_path) = args.config {
        ConfigLoader::new().with_file(config_path)
    } else {
        ConfigLoader::new()
    };
    let config = config_loader.load().context("Failed to load configuration")?;
    let analytics_config = config.analytics.unwrap_or_default();

    // Load results
    println!("{} Loading results...", "▶".green());
    let baseline_data = load_results(&args.baseline)?;
    let comparison_data = load_results(&args.comparison)?;
    println!("  {} Baseline: {} tests", "✓".green(), baseline_data.total_tests);
    println!("  {} Comparison: {} tests", "✓".green(), comparison_data.total_tests);
    println!();

    // Extract metric values
    println!("{} Extracting metric values...", "▶".green());
    let baseline_values = extract_metric_values(&baseline_data, &args.metric)?;
    let comparison_values = extract_metric_values(&comparison_data, &args.metric)?;
    println!("  {} Baseline: {} values", "✓".green(), baseline_values.len());
    println!("  {} Comparison: {} values", "✓".green(), comparison_values.len());
    println!();

    // Perform statistical analysis
    println!("{} Running statistical tests...", "▶".green());
    let baseline_summary = calculate_summary(&args.baseline.display().to_string(), &baseline_values);
    let comparison_summary = calculate_summary(&args.comparison.display().to_string(), &comparison_values);
    let test_results = run_t_test(&baseline_values, &comparison_values, args.confidence_level)?;
    println!("  {} Statistical analysis complete", "✓".green());
    println!();

    // Interpret results
    let interpretation = interpret_results(
        &baseline_summary,
        &comparison_summary,
        &test_results,
        args.effect_size_threshold,
        &analytics_config,
    )?;

    // Generate recommendations
    let recommendations = generate_recommendations(&interpretation, &test_results, &analytics_config)?;

    // Create analysis report
    let report = AnalysisReport {
        baseline: baseline_summary,
        comparison: comparison_summary,
        metric: args.metric.clone(),
        statistical_tests: test_results,
        interpretation,
        recommendations,
    };

    // Display results
    display_analysis(&report, &args, verbose)?;

    // Save report if requested
    if let Some(ref report_path) = args.report_file {
        save_report(&report, report_path)?;
        println!();
        println!("{} Report saved to: {}", "✓".green(), report_path.display().to_string().cyan());
    }

    println!();

    // Exit with appropriate code
    if args.fail_on_regression && report.interpretation.regression_detected {
        println!("{} Regression detected! Exiting with error code.", "✗".red().bold());
        std::process::exit(2);
    }

    println!("{} Analysis complete!", "✓".green().bold());
    Ok(())
}

fn load_results(path: &PathBuf) -> Result<serde_json::Value> {
    let content = std::fs::read_to_string(path)
        .context(format!("Failed to read file: {}", path.display()))?;

    let data: serde_json::Value = serde_json::from_str(&content)
        .context(format!("Failed to parse JSON from: {}", path.display()))?;

    Ok(data)
}

fn extract_metric_values(data: &serde_json::Value, metric: &str) -> Result<Vec<f64>> {
    let mut values = Vec::new();

    // Try to extract from benchmark results format
    if let Some(results) = data.get("results").and_then(|v| v.as_array()) {
        for result in results {
            if let Some(value) = extract_single_metric(result, metric) {
                values.push(value);
            }
        }
    }

    // Try to extract from summary format
    if values.is_empty() {
        if let Some(summary) = data.get("summary") {
            if let Some(value) = extract_single_metric(summary, metric) {
                values.push(value);
            }
        }
    }

    // Try to extract from comparison results format
    if values.is_empty() {
        if let Some(results_array) = data.as_array() {
            for item in results_array {
                if let Some(results) = item.get("results").and_then(|v| v.as_array()) {
                    for result in results {
                        if let Some(value) = extract_single_metric(result, metric) {
                            values.push(value);
                        }
                    }
                }
            }
        }
    }

    if values.is_empty() {
        anyhow::bail!(
            "No values found for metric '{}'. Available data structure: {}",
            metric,
            serde_json::to_string_pretty(data)?
        );
    }

    Ok(values)
}

fn extract_single_metric(data: &serde_json::Value, metric: &str) -> Option<f64> {
    match metric {
        "overall" | "duration" => data.get("duration_ms").and_then(|v| v.as_f64()),
        "latency" => data.get("duration_ms").and_then(|v| v.as_f64()),
        "tokens" => data.get("tokens_used").and_then(|v| v.as_f64()),
        "cost" => data.get("estimated_cost").and_then(|v| v.as_f64()),
        "faithfulness" | "relevance" | "coherence" => {
            data.get("metrics")
                .and_then(|m| m.get(metric))
                .and_then(|v| v.as_f64())
        }
        _ => data.get(metric).and_then(|v| v.as_f64()),
    }
}

fn calculate_summary(file: &str, values: &[f64]) -> ResultsSummary {
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let mean = values.iter().sum::<f64>() / values.len() as f64;
    let variance = values.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / values.len() as f64;
    let std_dev = variance.sqrt();

    let median = if sorted.len() % 2 == 0 {
        (sorted[sorted.len() / 2 - 1] + sorted[sorted.len() / 2]) / 2.0
    } else {
        sorted[sorted.len() / 2]
    };

    let p95_index = ((sorted.len() as f64 * 0.95) as usize).min(sorted.len() - 1);

    ResultsSummary {
        file: file.to_string(),
        total_tests: values.len(),
        mean,
        std_dev,
        min: sorted[0],
        max: sorted[sorted.len() - 1],
        median,
        p95: sorted[p95_index],
    }
}

fn run_t_test(
    baseline: &[f64],
    comparison: &[f64],
    confidence_level: f64,
) -> Result<StatisticalTestResults> {
    // Welch's t-test implementation (unequal variances assumed)

    let n1 = baseline.len() as f64;
    let n2 = comparison.len() as f64;

    let mean1 = baseline.iter().sum::<f64>() / n1;
    let mean2 = comparison.iter().sum::<f64>() / n2;

    let var1 = baseline.iter().map(|x| (x - mean1).powi(2)).sum::<f64>() / (n1 - 1.0);
    let var2 = comparison.iter().map(|x| (x - mean2).powi(2)).sum::<f64>() / (n2 - 1.0);

    // Welch-Satterthwaite degrees of freedom
    let df_numerator = (var1 / n1 + var2 / n2).powi(2);
    let df_denominator = (var1 / n1).powi(2) / (n1 - 1.0) + (var2 / n2).powi(2) / (n2 - 1.0);
    let df = (df_numerator / df_denominator) as usize;

    // t-statistic
    let t_statistic = (mean1 - mean2) / (var1 / n1 + var2 / n2).sqrt();

    // Approximate p-value using two-tailed test
    // For production, use a proper t-distribution library
    let p_value = approximate_p_value(t_statistic.abs(), df);

    // Cohen's d effect size
    let pooled_std = ((var1 + var2) / 2.0).sqrt();
    let effect_size = (mean2 - mean1).abs() / pooled_std;

    let effect_size_interpretation = interpret_effect_size(effect_size);

    let is_significant = p_value < (1.0 - confidence_level);

    Ok(StatisticalTestResults {
        confidence_level,
        t_statistic,
        p_value,
        degrees_of_freedom: df,
        effect_size,
        effect_size_interpretation,
        is_significant,
    })
}

fn approximate_p_value(t: f64, _df: usize) -> f64 {
    // Simplified p-value approximation
    // For production, use proper statistical library (statrs, etc.)
    if t > 3.0 {
        0.001
    } else if t > 2.576 {
        0.01
    } else if t > 1.96 {
        0.05
    } else if t > 1.645 {
        0.10
    } else {
        0.20
    }
}

fn interpret_effect_size(d: f64) -> String {
    if d < 0.2 {
        "negligible".to_string()
    } else if d < 0.5 {
        "small".to_string()
    } else if d < 0.8 {
        "medium".to_string()
    } else {
        "large".to_string()
    }
}

fn interpret_results(
    baseline: &ResultsSummary,
    comparison: &ResultsSummary,
    tests: &StatisticalTestResults,
    effect_threshold: f64,
    _config: &AnalyticsConfig,
) -> Result<Interpretation> {
    let mean_change = comparison.mean - baseline.mean;
    let percent_change = (mean_change / baseline.mean) * 100.0;

    let regression_detected = tests.is_significant
        && mean_change > 0.0 // For latency/cost, increase is regression
        && tests.effect_size >= effect_threshold;

    let improvement_detected = tests.is_significant
        && mean_change < 0.0 // For latency/cost, decrease is improvement
        && tests.effect_size >= effect_threshold;

    let practically_significant = tests.effect_size >= effect_threshold;

    let summary = if regression_detected {
        format!(
            "Regression detected: {} increase ({:.1}% change) with {} effect size",
            if mean_change > 0.0 { "significant" } else { "significant" },
            percent_change.abs(),
            tests.effect_size_interpretation
        )
    } else if improvement_detected {
        format!(
            "Improvement detected: {} decrease ({:.1}% change) with {} effect size",
            if tests.is_significant { "significant" } else { "non-significant" },
            percent_change.abs(),
            tests.effect_size_interpretation
        )
    } else if tests.is_significant {
        format!(
            "Statistically significant change ({:.1}% change) but effect size is {}",
            percent_change.abs(),
            tests.effect_size_interpretation
        )
    } else {
        format!(
            "No significant difference detected ({:.1}% change, p={:.3})",
            percent_change.abs(),
            tests.p_value
        )
    };

    let mut details = Vec::new();
    details.push(format!("Baseline mean: {:.2}", baseline.mean));
    details.push(format!("Comparison mean: {:.2}", comparison.mean));
    details.push(format!("Change: {:.2} ({:.1}%)", mean_change, percent_change));
    details.push(format!("P-value: {:.4}", tests.p_value));
    details.push(format!("Effect size (Cohen's d): {:.3}", tests.effect_size));
    details.push(format!("Statistical significance: {}", if tests.is_significant { "Yes" } else { "No" }));
    details.push(format!("Practical significance: {}", if practically_significant { "Yes" } else { "No" }));

    Ok(Interpretation {
        regression_detected,
        improvement_detected,
        practically_significant,
        summary,
        details,
    })
}

fn generate_recommendations(
    interpretation: &Interpretation,
    tests: &StatisticalTestResults,
    _config: &AnalyticsConfig,
) -> Result<Vec<String>> {
    let mut recommendations = Vec::new();

    if interpretation.regression_detected {
        recommendations.push("Consider reverting recent changes or investigating the cause of regression".to_string());
        recommendations.push("Review system performance metrics and resource utilization".to_string());
        recommendations.push("Run additional tests to confirm the regression is consistent".to_string());
    } else if interpretation.improvement_detected {
        recommendations.push("Document the changes that led to this improvement".to_string());
        recommendations.push("Consider deploying these changes to production".to_string());
    } else if tests.is_significant && !interpretation.practically_significant {
        recommendations.push("The change is statistically significant but may not be practically meaningful".to_string());
        recommendations.push("Consider collecting more data to better assess the real-world impact".to_string());
    } else {
        recommendations.push("No significant changes detected - continue monitoring".to_string());
        recommendations.push("Consider increasing sample size for more robust statistical power".to_string());
    }

    Ok(recommendations)
}

fn display_analysis(report: &AnalysisReport, args: &AnalyzeArgs, verbose: bool) -> Result<()> {
    match args.output {
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(report)?;
            println!("{}", json);
        }
        OutputFormat::Summary | OutputFormat::Detailed => {
            println!("{}", "Analysis Results".bold().cyan());
            println!("{}", "═".repeat(80).dimmed());
            println!();

            // Summary statistics
            println!("{}", "Baseline Statistics".bold());
            print_summary(&report.baseline);
            println!();

            println!("{}", "Comparison Statistics".bold());
            print_summary(&report.comparison);
            println!();

            // Statistical test results
            println!("{}", "Statistical Test Results".bold().yellow());
            println!("  Confidence level: {:.0}%", report.statistical_tests.confidence_level * 100.0);
            println!("  T-statistic: {:.3}", report.statistical_tests.t_statistic);
            println!("  P-value: {:.4}", report.statistical_tests.p_value);
            println!("  Degrees of freedom: {}", report.statistical_tests.degrees_of_freedom);
            println!("  Effect size (Cohen's d): {:.3} ({})",
                report.statistical_tests.effect_size,
                report.statistical_tests.effect_size_interpretation
            );
            println!("  Statistically significant: {}",
                if report.statistical_tests.is_significant { "Yes".green() } else { "No".yellow() }
            );
            println!();

            // Interpretation
            let status_color = if report.interpretation.regression_detected {
                "red"
            } else if report.interpretation.improvement_detected {
                "green"
            } else {
                "yellow"
            };

            println!("{}", "Interpretation".bold());
            match status_color {
                "red" => println!("  {}", report.interpretation.summary.red()),
                "green" => println!("  {}", report.interpretation.summary.green()),
                _ => println!("  {}", report.interpretation.summary.yellow()),
            }
            println!();

            if verbose || args.output == OutputFormat::Detailed {
                println!("{}", "Details:".dimmed());
                for detail in &report.interpretation.details {
                    println!("  • {}", detail.dimmed());
                }
                println!();
            }

            // Recommendations
            println!("{}", "Recommendations".bold().cyan());
            for (idx, rec) in report.recommendations.iter().enumerate() {
                println!("  {}. {}", idx + 1, rec);
            }
        }
    }

    Ok(())
}

fn print_summary(summary: &ResultsSummary) {
    println!("  Total tests: {}", summary.total_tests);
    println!("  Mean: {:.2}", summary.mean);
    println!("  Std Dev: {:.2}", summary.std_dev);
    println!("  Min: {:.2}", summary.min);
    println!("  Max: {:.2}", summary.max);
    println!("  Median: {:.2}", summary.median);
    println!("  P95: {:.2}", summary.p95);
}

fn save_report(report: &AnalysisReport, path: &PathBuf) -> Result<()> {
    let json = serde_json::to_string_pretty(report)?;
    std::fs::write(path, json)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_summary() {
        let values = vec![100.0, 200.0, 150.0, 180.0, 120.0];
        let summary = calculate_summary("test", &values);

        assert_eq!(summary.total_tests, 5);
        assert_eq!(summary.mean, 150.0);
        assert_eq!(summary.min, 100.0);
        assert_eq!(summary.max, 200.0);
    }

    #[test]
    fn test_interpret_effect_size() {
        assert_eq!(interpret_effect_size(0.1), "negligible");
        assert_eq!(interpret_effect_size(0.3), "small");
        assert_eq!(interpret_effect_size(0.6), "medium");
        assert_eq!(interpret_effect_size(1.0), "large");
    }

    #[test]
    fn test_run_t_test() {
        let baseline = vec![100.0, 110.0, 105.0, 115.0, 108.0];
        let comparison = vec![150.0, 160.0, 155.0, 165.0, 158.0];

        let result = run_t_test(&baseline, &comparison, 0.95).unwrap();

        assert!(result.is_significant);
        assert!(result.effect_size > 0.8); // Should be large effect
    }
}

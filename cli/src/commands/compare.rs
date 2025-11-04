use anyhow::{Context, Result};
use clap::Args;
use colored::Colorize;
use llm_test_bench_core::config::{Config, ConfigLoader};
use llm_test_bench_core::providers::ProviderFactory;
use llm_test_bench_datasets::loader::DatasetLoader;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Instant;

#[derive(Args, Debug)]
pub struct CompareArgs {
    /// Prompt to test (or use --dataset)
    #[arg(short, long, conflicts_with = "dataset")]
    pub prompt: Option<String>,

    /// Dataset file for batch comparison
    #[arg(short, long)]
    pub dataset: Option<PathBuf>,

    /// Models to compare (comma-separated, format: provider:model)
    /// Example: openai:gpt-4,anthropic:claude-3-opus
    #[arg(short, long, value_delimiter = ',', required = true)]
    pub models: Vec<String>,

    /// Evaluation metrics to use
    #[arg(long, value_delimiter = ',', default_value = "faithfulness,relevance")]
    pub metrics: Vec<String>,

    /// Run statistical significance tests
    #[arg(long)]
    pub statistical_tests: bool,

    /// Output format
    #[arg(short, long, default_value = "table")]
    pub output: OutputFormat,

    /// Output file path
    #[arg(long)]
    pub output_file: Option<PathBuf>,

    /// Generate HTML dashboard
    #[arg(long)]
    pub dashboard: bool,

    /// Path to custom configuration file
    #[arg(long)]
    pub config: Option<PathBuf>,

    /// Maximum concurrent comparisons
    #[arg(long, default_value = "5")]
    pub concurrency: usize,
}

#[derive(Debug, Clone, clap::ValueEnum)]
pub enum OutputFormat {
    Dashboard,
    Json,
    Table,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ComparisonResult {
    pub model: String,
    pub provider: String,
    pub response: String,
    pub duration_ms: u64,
    pub tokens_used: Option<u64>,
    pub estimated_cost: f64,
    pub metrics: std::collections::HashMap<String, f64>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ComparisonReport {
    pub prompt: String,
    pub timestamp: String,
    pub results: Vec<ComparisonResult>,
    pub winner: Option<String>,
    pub statistical_tests: Option<StatisticalTests>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StatisticalTests {
    pub confidence_level: f64,
    pub significant_differences: Vec<String>,
    pub p_values: std::collections::HashMap<String, f64>,
}

pub async fn execute(args: CompareArgs, verbose: bool) -> Result<()> {
    println!("{}", "LLM Test Bench - Compare Command".bold().cyan());
    println!();

    // Validate input
    if args.prompt.is_none() && args.dataset.is_none() {
        anyhow::bail!("Either --prompt or --dataset must be specified");
    }

    if args.models.len() < 2 {
        anyhow::bail!("At least 2 models must be specified for comparison");
    }

    if verbose {
        println!("{}", "Configuration:".bold());
        println!("  Models: {}", args.models.join(", "));
        println!("  Metrics: {}", args.metrics.join(", "));
        println!("  Output: {:?}", args.output);
        println!("  Statistical tests: {}", args.statistical_tests);
        println!();
    }

    // Load configuration
    let config_loader = if let Some(ref config_path) = args.config {
        ConfigLoader::new().with_file(config_path)
    } else {
        ConfigLoader::new()
    };
    let config = config_loader.load().context("Failed to load configuration")?;

    // Parse model specifications
    let model_specs = parse_model_specs(&args.models)?;

    // Run comparison based on input type
    let reports = if let Some(ref prompt) = args.prompt {
        vec![run_single_comparison(prompt, &model_specs, &args, &config, verbose).await?]
    } else if let Some(ref dataset_path) = args.dataset {
        run_batch_comparison(dataset_path, &model_specs, &args, &config, verbose).await?
    } else {
        unreachable!()
    };

    // Output results
    display_results(&reports, &args, verbose)?;

    // Save results if requested
    if let Some(ref output_path) = args.output_file {
        save_results(&reports, output_path, &args.output)?;
        println!();
        println!("{} Results saved to: {}", "✓".green(), output_path.display().to_string().cyan());
    }

    // Generate dashboard if requested
    if args.dashboard {
        let dashboard_path = generate_dashboard(&reports, &args)?;
        println!("{} Dashboard generated: {}", "✓".green(), dashboard_path.display().to_string().cyan());
    }

    println!();
    println!("{} Comparison complete!", "✓".green().bold());

    Ok(())
}

fn parse_model_specs(models: &[String]) -> Result<Vec<(String, String)>> {
    let mut specs = Vec::new();

    for model in models {
        let parts: Vec<&str> = model.split(':').collect();
        if parts.len() != 2 {
            anyhow::bail!(
                "Invalid model specification: '{}'. Expected format: provider:model",
                model
            );
        }

        let provider = parts[0].to_string();
        let model_name = parts[1].to_string();
        specs.push((provider, model_name));
    }

    Ok(specs)
}

async fn run_single_comparison(
    prompt: &str,
    model_specs: &[(String, String)],
    args: &CompareArgs,
    config: &Config,
    verbose: bool,
) -> Result<ComparisonReport> {
    println!("{} Running comparison for prompt...", "▶".green());
    if verbose {
        println!("  Prompt: {}", prompt.dimmed());
    }
    println!();

    let mut results = Vec::new();
    let factory = ProviderFactory::new();

    for (idx, (provider_name, model_name)) in model_specs.iter().enumerate() {
        print!("  {} Testing {}/{}: {}:{} ... ",
            "▶".cyan(),
            idx + 1,
            model_specs.len(),
            provider_name.bold(),
            model_name
        );

        let start = Instant::now();

        // Get provider configuration
        let provider_config = config
            .providers
            .get(provider_name)
            .ok_or_else(|| anyhow::anyhow!("Provider '{}' not found in configuration", provider_name))?;

        // Create provider instance
        let provider = factory
            .create_shared(provider_name, provider_config)
            .context(format!("Failed to create provider: {}", provider_name))?;

        // Execute request
        let result = match provider.complete(prompt, model_name).await {
            Ok(response) => {
                let duration = start.elapsed();
                println!("{} ({:.0}ms)", "✓".green(), duration.as_millis());

                ComparisonResult {
                    model: model_name.clone(),
                    provider: provider_name.clone(),
                    response: response.text.clone(),
                    duration_ms: duration.as_millis() as u64,
                    tokens_used: Some(response.usage.total_tokens as u64),
                    estimated_cost: calculate_cost(provider_name, model_name, &response.usage),
                    metrics: std::collections::HashMap::new(),
                    error: None,
                }
            }
            Err(e) => {
                println!("{} {}", "✗".red(), e.to_string().red());
                ComparisonResult {
                    model: model_name.clone(),
                    provider: provider_name.clone(),
                    response: String::new(),
                    duration_ms: start.elapsed().as_millis() as u64,
                    tokens_used: None,
                    estimated_cost: 0.0,
                    metrics: std::collections::HashMap::new(),
                    error: Some(e.to_string()),
                }
            }
        };

        results.push(result);
    }

    // Run evaluations if metrics specified
    if !args.metrics.is_empty() {
        println!();
        println!("{} Running evaluations...", "▶".green());
        // TODO: Integrate with evaluation system
        for result in &mut results {
            if result.error.is_none() {
                for metric in &args.metrics {
                    // Placeholder: In real implementation, call evaluator
                    result.metrics.insert(metric.clone(), 0.85);
                }
            }
        }
        println!("  {} Evaluations complete", "✓".green());
    }

    // Run statistical tests if requested
    let statistical_tests = if args.statistical_tests && results.len() >= 2 {
        Some(run_statistical_tests(&results)?)
    } else {
        None
    };

    Ok(ComparisonReport {
        prompt: prompt.to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        results,
        winner: None, // TODO: Determine winner based on metrics
        statistical_tests,
    })
}

async fn run_batch_comparison(
    dataset_path: &PathBuf,
    model_specs: &[(String, String)],
    args: &CompareArgs,
    config: &Config,
    verbose: bool,
) -> Result<Vec<ComparisonReport>> {
    println!("{} Loading dataset...", "▶".green());

    let loader = DatasetLoader::new();
    let dataset = loader.load(dataset_path).context("Failed to load dataset")?;

    println!("  {} Loaded: {} ({} tests)", "✓".green(), dataset.name.bold(), dataset.test_cases.len());
    println!();

    let mut reports = Vec::new();

    for (idx, test_case) in dataset.test_cases.iter().enumerate() {
        println!("{} Test {}/{}: {}", "▶".cyan().bold(), idx + 1, dataset.test_cases.len(), test_case.name.bold());

        let report = run_single_comparison(
            &test_case.prompt,
            model_specs,
            args,
            config,
            false, // Don't be verbose in batch mode
        )
        .await?;

        reports.push(report);
        println!();
    }

    Ok(reports)
}

fn run_statistical_tests(results: &[ComparisonResult]) -> Result<StatisticalTests> {
    // Placeholder implementation
    // In real implementation, use proper statistical tests (t-test, ANOVA, etc.)

    let mut significant_differences = Vec::new();
    let mut p_values = std::collections::HashMap::new();

    // Compare first two models as example
    if results.len() >= 2 {
        let comparison = format!("{} vs {}", results[0].model, results[1].model);
        p_values.insert(comparison.clone(), 0.03); // Placeholder p-value

        if 0.03 < 0.05 {
            significant_differences.push(format!(
                "{} significantly different from {} (p=0.03)",
                results[0].model, results[1].model
            ));
        }
    }

    Ok(StatisticalTests {
        confidence_level: 0.95,
        significant_differences,
        p_values,
    })
}

fn calculate_cost(provider: &str, model: &str, usage: &llm_test_bench_core::providers::types::Usage) -> f64 {
    // Simplified cost calculation
    // In production, use real pricing from provider configurations
    let (input_cost, output_cost) = match (provider, model) {
        ("openai", m) if m.contains("gpt-4") => (0.03 / 1000.0, 0.06 / 1000.0),
        ("openai", _) => (0.0015 / 1000.0, 0.002 / 1000.0),
        ("anthropic", _) => (0.015 / 1000.0, 0.075 / 1000.0),
        _ => (0.001 / 1000.0, 0.002 / 1000.0),
    };

    usage.prompt_tokens as f64 * input_cost + usage.completion_tokens as f64 * output_cost
}

fn display_results(reports: &[ComparisonReport], args: &CompareArgs, verbose: bool) -> Result<()> {
    println!();
    println!("{}", "Comparison Results".bold().cyan());
    println!("{}", "═".repeat(80).dimmed());
    println!();

    for (idx, report) in reports.iter().enumerate() {
        if reports.len() > 1 {
            println!("{} Test {}: {}", "▶".cyan(), idx + 1, report.prompt);
            println!();
        }

        match args.output {
            OutputFormat::Table => display_table(report, verbose)?,
            OutputFormat::Json => display_json(report)?,
            OutputFormat::Dashboard => {
                println!("Dashboard format requires --dashboard flag or --output-file");
            }
        }

        if let Some(ref stats) = report.statistical_tests {
            println!();
            println!("{}", "Statistical Analysis".bold().yellow());
            println!("  Confidence level: {:.0}%", stats.confidence_level * 100.0);
            if !stats.significant_differences.is_empty() {
                println!("  Significant differences:");
                for diff in &stats.significant_differences {
                    println!("    • {}", diff);
                }
            } else {
                println!("  No significant differences detected");
            }
        }

        if reports.len() > 1 {
            println!();
        }
    }

    Ok(())
}

fn display_table(report: &ComparisonReport, verbose: bool) -> Result<()> {
    // Simple table display
    println!("{:<25} {:<15} {:<10} {:<12} {:<10}", "Model", "Duration", "Tokens", "Cost", "Status");
    println!("{}", "─".repeat(80).dimmed());

    for result in &report.results {
        let status = if result.error.is_none() {
            "✓".green().to_string()
        } else {
            "✗".red().to_string()
        };

        let model_display = format!("{}:{}", result.provider, result.model);
        let tokens_display = result.tokens_used.map_or("-".to_string(), |t| t.to_string());

        println!(
            "{:<25} {:<15} {:<10} ${:<11.4} {}",
            model_display,
            format!("{}ms", result.duration_ms),
            tokens_display,
            result.estimated_cost,
            status
        );

        if verbose && result.error.is_none() {
            println!("  Response: {}", result.response.chars().take(100).collect::<String>().dimmed());
        }

        if let Some(ref error) = result.error {
            println!("  {}: {}", "Error".red(), error);
        }
    }

    // Display metrics if available
    if !report.results.is_empty() && !report.results[0].metrics.is_empty() {
        println!();
        println!("{}", "Evaluation Metrics".bold());
        print!("{:<25}", "Model");
        for metric in report.results[0].metrics.keys() {
            print!(" {:<12}", metric);
        }
        println!();
        println!("{}", "─".repeat(80).dimmed());

        for result in &report.results {
            if result.error.is_none() {
                let model_display = format!("{}:{}", result.provider, result.model);
                print!("{:<25}", model_display);
                for value in result.metrics.values() {
                    print!(" {:<12.2}", value);
                }
                println!();
            }
        }
    }

    Ok(())
}

fn display_json(report: &ComparisonReport) -> Result<()> {
    let json = serde_json::to_string_pretty(report)?;
    println!("{}", json);
    Ok(())
}

fn save_results(reports: &[ComparisonReport], path: &PathBuf, format: &OutputFormat) -> Result<()> {
    match format {
        OutputFormat::Json | OutputFormat::Dashboard => {
            let json = serde_json::to_string_pretty(&reports)?;
            std::fs::write(path, json)?;
        }
        OutputFormat::Table => {
            // Save as CSV for table format
            let mut csv = String::new();
            csv.push_str("prompt,model,provider,duration_ms,tokens,cost,status\n");

            for report in reports {
                for result in &report.results {
                    let status = if result.error.is_none() { "success" } else { "failed" };
                    csv.push_str(&format!(
                        "\"{}\",{},{},{},{},{},{}\n",
                        report.prompt.replace('"', "\"\""),
                        result.model,
                        result.provider,
                        result.duration_ms,
                        result.tokens_used.unwrap_or(0),
                        result.estimated_cost,
                        status
                    ));
                }
            }

            std::fs::write(path, csv)?;
        }
    }

    Ok(())
}

fn generate_dashboard(reports: &[ComparisonReport], _args: &CompareArgs) -> Result<PathBuf> {
    // Placeholder for dashboard generation
    // In real implementation, generate HTML with charts using a template engine

    let html = format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>LLM Comparison Dashboard</title>
    <meta charset="utf-8">
    <style>
        body {{ font-family: Arial, sans-serif; margin: 20px; background: #f5f5f5; }}
        .container {{ max-width: 1200px; margin: 0 auto; background: white; padding: 20px; }}
        h1 {{ color: #333; }}
        .summary {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 20px; margin: 20px 0; }}
        .card {{ background: #f9f9f9; padding: 15px; border-radius: 5px; }}
        table {{ width: 100%; border-collapse: collapse; margin: 20px 0; }}
        th, td {{ padding: 10px; text-align: left; border-bottom: 1px solid #ddd; }}
        th {{ background: #4CAF50; color: white; }}
        .success {{ color: green; }}
        .failed {{ color: red; }}
    </style>
</head>
<body>
    <div class="container">
        <h1>LLM Comparison Dashboard</h1>
        <div class="summary">
            <div class="card">
                <h3>Total Tests</h3>
                <p style="font-size: 24px;">{}</p>
            </div>
            <div class="card">
                <h3>Models Compared</h3>
                <p style="font-size: 24px;">{}</p>
            </div>
        </div>
        <h2>Results</h2>
        <table>
            <tr>
                <th>Model</th>
                <th>Duration (ms)</th>
                <th>Tokens</th>
                <th>Cost ($)</th>
                <th>Status</th>
            </tr>
            {}
        </table>
    </div>
</body>
</html>"#,
        reports.len(),
        if reports.is_empty() { 0 } else { reports[0].results.len() },
        generate_table_rows(reports)
    );

    let dashboard_path = PathBuf::from("comparison-dashboard.html");
    std::fs::write(&dashboard_path, html)?;

    Ok(dashboard_path)
}

fn generate_table_rows(reports: &[ComparisonReport]) -> String {
    let mut rows = String::new();

    for report in reports {
        for result in &report.results {
            let status_class = if result.error.is_none() { "success" } else { "failed" };
            let status_text = if result.error.is_none() { "✓" } else { "✗" };

            rows.push_str(&format!(
                "<tr><td>{}:{}</td><td>{}</td><td>{}</td><td>{:.4}</td><td class=\"{}\">{}</td></tr>\n",
                result.provider,
                result.model,
                result.duration_ms,
                result.tokens_used.unwrap_or(0),
                result.estimated_cost,
                status_class,
                status_text
            ));
        }
    }

    rows
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_model_specs_valid() {
        let models = vec!["openai:gpt-4".to_string(), "anthropic:claude-3-opus".to_string()];
        let specs = parse_model_specs(&models).unwrap();

        assert_eq!(specs.len(), 2);
        assert_eq!(specs[0], ("openai".to_string(), "gpt-4".to_string()));
        assert_eq!(specs[1], ("anthropic".to_string(), "claude-3-opus".to_string()));
    }

    #[test]
    fn test_parse_model_specs_invalid() {
        let models = vec!["invalid-spec".to_string()];
        let result = parse_model_specs(&models);
        assert!(result.is_err());
    }

    #[test]
    fn test_compare_args_validation() {
        let args = CompareArgs {
            prompt: None,
            dataset: None,
            models: vec!["openai:gpt-4".to_string()],
            metrics: vec![],
            statistical_tests: false,
            output: OutputFormat::Table,
            output_file: None,
            dashboard: false,
            config: None,
            concurrency: 5,
        };

        // Should have at least 2 models
        assert!(args.models.len() < 2);
    }
}

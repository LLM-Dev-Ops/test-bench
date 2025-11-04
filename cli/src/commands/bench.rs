use anyhow::{Context, Result};
use clap::Args;
use colored::Colorize;
use llm_test_bench_core::benchmarks::{BenchmarkConfig, BenchmarkRunner, CsvExporter};
use llm_test_bench_core::config::ConfigLoader;
use llm_test_bench_core::providers::ProviderFactory;
use llm_test_bench_datasets::loader::DatasetLoader;
use std::path::PathBuf;

#[derive(Args, Debug)]
pub struct BenchArgs {
    /// Path to dataset file (JSON or YAML)
    #[arg(short, long)]
    pub dataset: PathBuf,

    /// Providers to benchmark (comma-separated, e.g., openai,anthropic)
    #[arg(short, long, value_delimiter = ',')]
    pub providers: Vec<String>,

    /// Number of concurrent requests
    #[arg(short, long, default_value = "5")]
    pub concurrency: usize,

    /// Output directory for benchmark results
    #[arg(short, long, default_value = "./bench-results")]
    pub output: PathBuf,

    /// Export format (json, csv, both)
    #[arg(short, long, default_value = "both")]
    pub export: ExportFormat,

    /// Continue on failure instead of stopping
    #[arg(long, default_value = "true")]
    pub continue_on_failure: bool,

    /// Save raw responses to disk
    #[arg(long, default_value = "true")]
    pub save_responses: bool,

    /// Request delay in milliseconds (to avoid rate limiting)
    #[arg(long)]
    pub delay: Option<u64>,

    /// Path to custom configuration file
    #[arg(long)]
    pub config: Option<PathBuf>,

    /// Evaluation metrics to run (comma-separated)
    #[arg(long, value_delimiter = ',')]
    pub metrics: Option<Vec<String>>,

    /// Judge model for evaluations (overrides config)
    #[arg(long)]
    pub judge_model: Option<String>,

    /// Judge provider (openai, anthropic)
    #[arg(long)]
    pub judge_provider: Option<String>,

    /// Generate HTML dashboard after benchmark
    #[arg(long)]
    pub dashboard: bool,
}

#[derive(Debug, Clone, clap::ValueEnum)]
pub enum ExportFormat {
    Json,
    Csv,
    Both,
}

pub async fn execute(args: BenchArgs, verbose: bool) -> Result<()> {
    println!("{}", "LLM Test Bench - Benchmark Command".bold().cyan());
    println!();

    // Validate dataset path
    if !args.dataset.exists() {
        anyhow::bail!("Dataset file not found: {}", args.dataset.display());
    }

    // Validate providers
    if args.providers.is_empty() {
        anyhow::bail!("At least one provider must be specified");
    }

    if verbose {
        println!("{}", "Configuration:".bold());
        println!("  Dataset: {}", args.dataset.display());
        println!("  Providers: {}", args.providers.join(", "));
        println!("  Concurrency: {}", args.concurrency);
        println!("  Output: {}", args.output.display());
        println!("  Export format: {:?}", args.export);
        if let Some(ref metrics) = args.metrics {
            println!("  Metrics: {}", metrics.join(", "));
        }
        if args.dashboard {
            println!("  Generate dashboard: Yes");
        }
        println!();
    }

    // Step 1: Load dataset
    println!("{} Loading dataset...", "▶".green());
    let loader = DatasetLoader::new();
    let dataset = loader.load(&args.dataset)
        .context("Failed to load dataset")?;

    println!("  {} Loaded: {} ({} tests)",
        "✓".green(),
        dataset.name.bold(),
        dataset.test_cases.len()
    );
    if let Some(ref desc) = dataset.description {
        println!("  Description: {}", desc.dimmed());
    }
    println!();

    // Step 2: Load configuration
    let config_loader = if let Some(ref config_path) = args.config {
        ConfigLoader::new().with_file(config_path)
    } else {
        ConfigLoader::new()
    };
    let config = config_loader.load()
        .context("Failed to load configuration")?;

    // Step 3: Create output directory
    std::fs::create_dir_all(&args.output)
        .context("Failed to create output directory")?;

    // Step 4: Run benchmark for each provider
    for (idx, provider_name) in args.providers.iter().enumerate() {
        println!("{} Benchmarking provider {} ({}/{})...",
            "▶".green().bold(),
            provider_name.bold(),
            idx + 1,
            args.providers.len()
        );

        // Get provider configuration
        let provider_config = config.providers.get(provider_name)
            .ok_or_else(|| anyhow::anyhow!("Provider '{}' not found in configuration", provider_name))?;

        // Create provider instance
        let factory = ProviderFactory::new();
        let provider = factory.create_shared(provider_name, provider_config)
            .context(format!("Failed to create provider: {}", provider_name))?;

        if verbose {
            println!("  Provider: {}", provider.name());
            println!("  Default model: {}", provider_config.default_model);
        }

        // Configure benchmark
        let bench_config = BenchmarkConfig {
            concurrency: args.concurrency,
            save_responses: args.save_responses,
            output_dir: args.output.join(provider_name),
            continue_on_failure: args.continue_on_failure,
            random_seed: None,
            request_delay_ms: args.delay,
        };

        // Validate benchmark configuration
        if let Err(e) = bench_config.validate() {
            anyhow::bail!("Invalid benchmark configuration: {}", e);
        }

        // Create output directory for this provider
        std::fs::create_dir_all(&bench_config.output_dir)
            .context("Failed to create provider output directory")?;

        // Run benchmark
        let runner = BenchmarkRunner::new(bench_config);
        let results = runner.run(&dataset, provider).await
            .context(format!("Benchmark failed for provider: {}", provider_name))?;

        // Export results
        export_results(&results, &args.output, provider_name, &args.export)?;

        // Print summary
        print_summary(&results, provider_name);
        println!();
    }

    // Run evaluations if metrics specified
    if let Some(ref metrics) = args.metrics {
        println!();
        println!("{} Running evaluations...", "▶".green().bold());
        println!("  Metrics: {}", metrics.join(", "));
        println!("  {} Note: Full evaluation integration pending Phase 4 completion", "ℹ".blue());
        // TODO: Integrate with evaluation system when available
        println!();
    }

    // Generate dashboard if requested
    if args.dashboard {
        println!();
        println!("{} Generating dashboard...", "▶".green());
        let dashboard_path = args.output.join("benchmark-dashboard.html");
        // TODO: Call dashboard generation
        println!("  {} Dashboard would be generated at: {}", "ℹ".blue(), dashboard_path.display());
        println!();
    }

    // Final summary
    println!("{} Benchmark complete!", "✓".green().bold());
    println!("Results saved to: {}", args.output.display().to_string().cyan());
    println!();

    Ok(())
}

/// Export benchmark results based on the selected format
fn export_results(
    results: &llm_test_bench_core::benchmarks::runner::BenchmarkResults,
    output_dir: &PathBuf,
    provider_name: &str,
    format: &ExportFormat,
) -> Result<()> {
    match format {
        ExportFormat::Json => {
            let json_path = output_dir.join(format!("{}-results.json", provider_name));
            let json = serde_json::to_string_pretty(results)?;
            std::fs::write(&json_path, json)?;
            println!("  {} Saved JSON: {}", "✓".green(), json_path.display());
        }
        ExportFormat::Csv => {
            let csv_path = output_dir.join(format!("{}-results.csv", provider_name));
            CsvExporter::export_default(results, &csv_path)?;
            println!("  {} Saved CSV: {}", "✓".green(), csv_path.display());
        }
        ExportFormat::Both => {
            let json_path = output_dir.join(format!("{}-results.json", provider_name));
            let json = serde_json::to_string_pretty(results)?;
            std::fs::write(&json_path, json)?;
            println!("  {} Saved JSON: {}", "✓".green(), json_path.display());

            let csv_path = output_dir.join(format!("{}-results.csv", provider_name));
            CsvExporter::export_default(results, &csv_path)?;
            println!("  {} Saved CSV: {}", "✓".green(), csv_path.display());
        }
    }

    Ok(())
}

/// Print a formatted summary of the benchmark results
fn print_summary(
    results: &llm_test_bench_core::benchmarks::runner::BenchmarkResults,
    provider_name: &str,
) {
    let summary = &results.summary;

    println!();
    println!("{}", format!("Results for {}:", provider_name).bold());
    println!("{}", "─".repeat(60).dimmed());

    // Test counts
    println!("  {} Tests:        {}", "ℹ".blue(), summary.total.to_string().bold());
    println!("  {} Success:      {} ({:.1}%)",
        "✓".green(),
        summary.succeeded.to_string().green(),
        summary.success_rate * 100.0
    );

    if summary.failed > 0 {
        println!("  {} Failed:       {}",
            "✗".red(),
            summary.failed.to_string().red()
        );
    }

    if summary.timeout > 0 {
        println!("  {} Timeout:      {}",
            "⏱".yellow(),
            summary.timeout.to_string().yellow()
        );
    }

    if summary.skipped > 0 {
        println!("  {} Skipped:      {}",
            "⊘".yellow(),
            summary.skipped.to_string().dimmed()
        );
    }

    println!();

    // Performance metrics
    println!("  {} Avg Duration: {:.0}ms",
        "⏱".cyan(),
        summary.avg_duration_ms
    );
    println!("  {} P50 Latency:  {:.0}ms",
        "ℹ".blue(),
        summary.p50_duration_ms
    );
    println!("  {} P95 Latency:  {:.0}ms",
        "ℹ".blue(),
        summary.p95_duration_ms
    );
    println!("  {} P99 Latency:  {:.0}ms",
        "ℹ".blue(),
        summary.p99_duration_ms
    );

    println!();

    // Token usage and cost
    println!("  {} Total Tokens: {}",
        "💰".to_string(),
        summary.total_tokens.to_string().yellow()
    );
    println!("  {} Est. Cost:    ${:.4}",
        "💰".to_string(),
        summary.total_cost.to_string().green()
    );

    println!("{}", "─".repeat(60).dimmed());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bench_args_creation() {
        let args = BenchArgs {
            dataset: PathBuf::from("./test.json"),
            providers: vec!["openai".to_string()],
            concurrency: 5,
            output: PathBuf::from("./results"),
            export: ExportFormat::Both,
            continue_on_failure: true,
            save_responses: true,
            delay: None,
            config: None,
        };

        assert_eq!(args.concurrency, 5);
        assert_eq!(args.providers.len(), 1);
    }

    #[test]
    fn test_export_format_variants() {
        // Just ensure the enum variants exist and can be created
        let _json = ExportFormat::Json;
        let _csv = ExportFormat::Csv;
        let _both = ExportFormat::Both;
    }
}

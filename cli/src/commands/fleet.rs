use anyhow::{Context, Result};
use clap::Args;
use colored::Colorize;
use llm_test_bench_core::benchmarks::fleet_api::{
    FleetBenchmarkAPI, FleetConfig, FleetExecutionHandle,
};
use std::path::PathBuf;

#[derive(Args, Debug)]
pub struct FleetArgs {
    /// Path to fleet manifest file (JSON)
    #[arg(short, long)]
    pub manifest: PathBuf,

    /// Output directory for fleet results
    #[arg(short, long, default_value = "./fleet-results")]
    pub output: PathBuf,

    /// Number of concurrent requests per repository
    #[arg(short, long, default_value = "5")]
    pub concurrency: usize,

    /// Save raw responses to disk
    #[arg(long, default_value = "true")]
    pub save_responses: bool,

    /// Request delay in milliseconds (to avoid rate limiting)
    #[arg(long)]
    pub delay: Option<u64>,

    /// Path to custom configuration file
    #[arg(long)]
    pub config: Option<PathBuf>,

    /// Wait for fleet execution to complete before returning
    #[arg(long)]
    pub wait: bool,

    /// Output format (json, summary, or quiet)
    #[arg(long, default_value = "summary")]
    pub format: OutputFormat,
}

#[derive(Debug, Clone, clap::ValueEnum)]
pub enum OutputFormat {
    /// Full JSON output
    Json,
    /// Human-readable summary
    Summary,
    /// Minimal output (just run ID)
    Quiet,
}

pub async fn execute(args: FleetArgs, verbose: bool) -> Result<()> {
    // Print header
    if !matches!(args.format, OutputFormat::Quiet) {
        println!("{}", "LLM Test Bench - Fleet Command".bold().cyan());
        println!();
    }

    // Validate manifest path
    if !args.manifest.exists() {
        anyhow::bail!("Fleet manifest file not found: {}", args.manifest.display());
    }

    if verbose && !matches!(args.format, OutputFormat::Quiet) {
        println!("{}", "Configuration:".bold());
        println!("  Manifest: {}", args.manifest.display());
        println!("  Output: {}", args.output.display());
        println!("  Concurrency: {}", args.concurrency);
        if let Some(delay) = args.delay {
            println!("  Request delay: {}ms", delay);
        }
        if args.wait {
            println!("  Mode: Synchronous (wait for completion)");
        } else {
            println!("  Mode: Asynchronous (return immediately)");
        }
        println!();
    }

    // Configure fleet API
    let mut config = FleetConfig::new(args.output.clone())
        .with_concurrency(args.concurrency)
        .with_save_responses(args.save_responses);

    if let Some(delay) = args.delay {
        config = config.with_request_delay_ms(delay);
    }

    if let Some(config_path) = args.config {
        config = config.with_config(config_path);
    }

    // Create API instance
    let api = FleetBenchmarkAPI::new(config);

    // Execute fleet benchmark
    if !matches!(args.format, OutputFormat::Quiet) {
        println!("{} Executing fleet benchmark...", "▶".green());
    }

    let handle = api
        .execute_fleet_benchmark(&args.manifest)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to execute fleet benchmark: {}", e))?;

    // Display execution information
    print_execution_info(&handle, &args.format);

    // Wait for completion if requested
    if args.wait {
        if !matches!(args.format, OutputFormat::Quiet) {
            println!();
            println!("{} Waiting for execution to complete...", "⏳".yellow());
        }

        let results = handle
            .execution_future
            .await
            .context("Fleet execution task panicked")??;

        if !matches!(args.format, OutputFormat::Quiet) {
            println!("{} Fleet benchmark completed!", "✓".green().bold());
            println!();
        }

        // Print results based on format
        match args.format {
            OutputFormat::Json => {
                let json = serde_json::to_string_pretty(&results)?;
                println!("{}", json);
            }
            OutputFormat::Summary => {
                print_fleet_summary(&results);
            }
            OutputFormat::Quiet => {
                // Already printed run_id earlier
            }
        }
    } else if !matches!(args.format, OutputFormat::Quiet) {
        println!();
        println!(
            "{} Fleet benchmark executing in background",
            "ℹ".blue()
        );
        println!("  Use 'llm-test-bench fleet status {}' to check progress", handle.run_id);
    }

    Ok(())
}

/// Print execution information based on output format
fn print_execution_info(handle: &FleetExecutionHandle, format: &OutputFormat) {
    match format {
        OutputFormat::Quiet => {
            println!("{}", handle.run_id);
        }
        OutputFormat::Json => {
            let info = serde_json::json!({
                "run_id": handle.run_id,
                "artifact_base_dir": handle.artifact_base_dir.display().to_string(),
                "metadata": handle.metadata,
            });
            println!("{}", serde_json::to_string_pretty(&info).unwrap());
        }
        OutputFormat::Summary => {
            println!("{}", "Execution Started".bold());
            println!("{}", "─".repeat(60).dimmed());
            println!("  {} Run ID:      {}", "🆔".to_string(), handle.run_id.bold());
            println!(
                "  {} Artifacts:   {}",
                "📁".to_string(),
                handle.artifact_base_dir.display().to_string().cyan()
            );
            println!(
                "  {} Fleet:       {}",
                "🚢".to_string(),
                handle.metadata.fleet_id.bold()
            );
            println!(
                "  {} Repos:       {}",
                "📦".to_string(),
                handle.metadata.repository_count
            );
            println!(
                "  {} Providers:   {}",
                "🔌".to_string(),
                handle.metadata.providers.join(", ")
            );
            println!(
                "  {} Started:     {}",
                "⏱".to_string(),
                handle.metadata.started_at.format("%Y-%m-%d %H:%M:%S UTC")
            );
            println!("{}", "─".repeat(60).dimmed());
        }
    }
}

/// Print fleet benchmark summary
fn print_fleet_summary(results: &llm_test_bench_core::benchmarks::FleetBenchmarkResults) {
    let summary = &results.fleet_summary;

    println!("{}", "Fleet Benchmark Results".bold().green());
    println!("{}", "═".repeat(60).dimmed());
    println!();

    // Fleet Overview
    println!("{}", "Fleet Overview".bold());
    println!("{}", "─".repeat(60).dimmed());
    println!("  Fleet ID:        {}", results.fleet_id.bold());
    println!("  Repositories:    {}", summary.total_repositories);
    println!("  Total Tests:     {}", summary.total_tests);
    println!("  Duration:        {:.2}s", summary.total_duration_ms as f64 / 1000.0);
    println!();

    // Test Results
    println!("{}", "Test Results".bold());
    println!("{}", "─".repeat(60).dimmed());
    println!(
        "  {} Success:      {} ({:.1}%)",
        "✓".green(),
        summary.total_succeeded.to_string().green(),
        summary.success_rate * 100.0
    );

    if summary.total_failed > 0 {
        println!(
            "  {} Failed:       {}",
            "✗".red(),
            summary.total_failed.to_string().red()
        );
    }

    if summary.total_timeout > 0 {
        println!(
            "  {} Timeout:      {}",
            "⏱".yellow(),
            summary.total_timeout.to_string().yellow()
        );
    }

    if summary.total_skipped > 0 {
        println!(
            "  {} Skipped:      {}",
            "⊘".yellow(),
            summary.total_skipped.to_string().dimmed()
        );
    }
    println!();

    // Performance Metrics
    println!("{}", "Performance Metrics".bold());
    println!("{}", "─".repeat(60).dimmed());
    println!("  Avg Duration:    {:.0}ms", summary.avg_duration_ms);
    println!("  P50 Latency:     {:.0}ms", summary.p50_duration_ms);
    println!("  P95 Latency:     {:.0}ms", summary.p95_duration_ms);
    println!("  P99 Latency:     {:.0}ms", summary.p99_duration_ms);
    println!("  Min Duration:    {}ms", summary.min_duration_ms);
    println!("  Max Duration:    {}ms", summary.max_duration_ms);
    println!();

    // Cost & Token Usage
    println!("{}", "Cost & Token Usage".bold());
    println!("{}", "─".repeat(60).dimmed());
    println!("  Total Tokens:    {}", summary.total_tokens.to_string().yellow());
    println!(
        "  Avg Tokens/Req:  {:.0}",
        summary.avg_tokens_per_request
    );
    println!(
        "  Total Cost:      ${}",
        format!("{:.4}", summary.total_cost).green()
    );
    println!(
        "  Avg Cost/Repo:   ${}",
        format!("{:.4}", summary.avg_cost_per_repository).green()
    );
    println!();

    // Per-Provider Breakdown
    if !results.provider_breakdown.is_empty() {
        println!("{}", "Provider Breakdown".bold());
        println!("{}", "─".repeat(60).dimmed());
        for (provider, stats) in &results.provider_breakdown {
            println!(
                "  {} {} ({} repos, {:.1}% success)",
                "•".cyan(),
                provider.bold(),
                stats.repository_count,
                stats.success_rate * 100.0
            );
        }
        println!();
    }

    // Top/Bottom Repositories
    if let Some(best) = results.best_repository() {
        println!("{}", "Top Performing Repository".bold());
        println!("{}", "─".repeat(60).dimmed());
        println!(
            "  {} {} ({:.1}% success)",
            "🏆".to_string(),
            best.repository_name.bold(),
            best.results.summary.success_rate * 100.0
        );
        println!();
    }

    if let Some(worst) = results.worst_repository() {
        println!("{}", "Needs Attention".bold());
        println!("{}", "─".repeat(60).dimmed());
        println!(
            "  {} {} ({:.1}% success)",
            "⚠".yellow(),
            worst.repository_name.bold(),
            worst.results.summary.success_rate * 100.0
        );
        println!();
    }

    println!("{}", "═".repeat(60).dimmed());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fleet_args_creation() {
        let args = FleetArgs {
            manifest: PathBuf::from("./fleet.json"),
            output: PathBuf::from("./results"),
            concurrency: 5,
            save_responses: true,
            delay: None,
            config: None,
            wait: false,
            format: OutputFormat::Summary,
        };

        assert_eq!(args.concurrency, 5);
        assert!(!args.wait);
    }

    #[test]
    fn test_output_format_variants() {
        // Ensure the enum variants exist
        let _json = OutputFormat::Json;
        let _summary = OutputFormat::Summary;
        let _quiet = OutputFormat::Quiet;
    }
}

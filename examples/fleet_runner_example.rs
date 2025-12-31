// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Fleet Runner Example
//!
//! This example demonstrates how to use the FleetRunner to orchestrate
//! benchmarks across multiple repositories using a fleet manifest.
//!
//! # Usage
//!
//! ```bash
//! cargo run --example fleet_runner_example -- --manifest ./examples/fleet-manifest-example.json
//! ```

use llm_test_bench_core::benchmarks::{FleetRunner, FleetManifest};
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Load the fleet manifest
    let manifest_path = Path::new("./examples/fleet-manifest-example.json");
    println!("Loading fleet manifest from: {}", manifest_path.display());

    let manifest = FleetManifest::load_from_file(manifest_path)?;

    println!("\n=== Fleet Configuration ===");
    println!("Fleet ID: {}", manifest.fleet_id);
    println!("Description: {}", manifest.description);
    println!("Repositories: {}", manifest.repositories.len());
    println!("Providers: {:?}", manifest.providers);
    println!("Scenarios: {}", manifest.scenario_profiles.len());

    // Create the fleet runner
    let runner = FleetRunner::new();

    println!("\n=== Starting Fleet Benchmark ===");

    // Execute the fleet benchmark
    let results = runner.run(&manifest).await?;

    println!("\n=== Fleet Results ===");
    println!("Fleet ID: {}", results.fleet_id);
    println!("Total Repositories: {}", results.total_repositories);
    println!("Total Tests: {}", results.fleet_summary.total_tests);
    println!("Success Rate: {:.2}%", results.fleet_summary.success_rate * 100.0);
    println!("Total Duration: {} ms", results.fleet_summary.total_duration_ms);
    println!("Total Cost: ${:.4}", results.fleet_summary.total_cost);

    println!("\n=== Per-Repository Results ===");
    for repo in &results.repository_results {
        println!("\nRepository: {} ({})", repo.repository_id, repo.provider_name);
        println!("  Tests: {}", repo.results.summary.total);
        println!("  Success: {}", repo.results.summary.succeeded);
        println!("  Failed: {}", repo.results.summary.failed);
        println!("  Success Rate: {:.2}%", repo.results.summary.success_rate * 100.0);
        println!("  Avg Duration: {:.2} ms", repo.results.summary.avg_duration_ms);
        println!("  P95 Duration: {:.2} ms", repo.results.summary.p95_duration_ms);
    }

    println!("\n=== Per-Provider Breakdown ===");
    for (provider, stats) in &results.provider_breakdown {
        println!("\nProvider: {}", provider);
        println!("  Repositories: {}", stats.repository_count);
        println!("  Total Tests: {}", stats.total_tests);
        println!("  Success Rate: {:.2}%", stats.success_rate * 100.0);
        println!("  Total Cost: ${:.4}", stats.total_cost);
    }

    println!("\n=== Per-Category Breakdown ===");
    for (category, stats) in &results.category_breakdown {
        println!("\nCategory: {}", category);
        println!("  Total Tests: {}", stats.total_tests);
        println!("  Success Rate: {:.2}%", stats.success_rate * 100.0);
    }

    // Find best and worst performing repositories
    if let Some(best) = results.best_repository() {
        println!("\n=== Best Repository ===");
        println!("Repository: {}", best.repository_id);
        println!("Success Rate: {:.2}%", best.results.summary.success_rate * 100.0);
    }

    if let Some(worst) = results.worst_repository() {
        println!("\n=== Worst Repository ===");
        println!("Repository: {}", worst.repository_id);
        println!("Success Rate: {:.2}%", worst.results.summary.success_rate * 100.0);
    }

    // Find failing repositories (< 80% success rate)
    let failing = results.failing_repositories(0.8);
    if !failing.is_empty() {
        println!("\n=== Failing Repositories (< 80%) ===");
        for repo in failing {
            println!(
                "  - {}: {:.2}%",
                repo.repository_id,
                repo.results.summary.success_rate * 100.0
            );
        }
    }

    println!("\n=== Fleet benchmark completed successfully! ===");
    println!("Results saved to: {}", manifest.output.base_dir.display());

    Ok(())
}

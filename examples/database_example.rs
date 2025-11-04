// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! # Database Example
//!
//! This example demonstrates using the PostgreSQL database backend.

use llm_test_bench_core::database::{Database, DatabaseConfig, models::*};
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🗄️  LLM Test Bench Database Example");
    println!("===================================\n");

    // Get mode from command line
    let args: Vec<String> = std::env::args().collect();
    let mode = args.get(1).map(|s| s.as_str()).unwrap_or("help");

    match mode {
        "setup" => setup_database().await?,
        "benchmark" => create_benchmark().await?,
        "job" => create_job().await?,
        "worker" => register_worker().await?,
        "stats" => show_stats().await?,
        _ => print_help(),
    }

    Ok(())
}

/// Setup database (run migrations)
async fn setup_database() -> Result<()> {
    println!("Setting up database...\n");

    let config = DatabaseConfig::from_env().unwrap_or_default();

    println!("Database Configuration:");
    println!("  Host: {}", config.host);
    println!("  Port: {}", config.port);
    println!("  Database: {}", config.database);
    println!("  Pool Size: {}", config.pool_size);
    println!();

    println!("Connecting to database...");
    let db = Database::connect(config).await?;

    println!("✓ Connected\n");

    println!("Running migrations...");
    db.migrate().await?;

    println!("✓ Migrations complete\n");

    println!("Database setup complete!");

    Ok(())
}

/// Create a benchmark
async fn create_benchmark() -> Result<()> {
    println!("Creating benchmark...\n");

    let config = DatabaseConfig::from_env().unwrap_or_default();
    let db = Database::connect(config).await?;

    let benchmark = NewBenchmark {
        name: "GPT-4 Performance Test".to_string(),
        description: Some("Testing GPT-4 on MMLU dataset".to_string()),
        provider: "openai".to_string(),
        model: "gpt-4".to_string(),
        dataset: "mmlu".to_string(),
        total_iterations: 100,
        created_by: None,
    };

    let record = db.benchmarks().create(benchmark).await?;

    println!("✓ Benchmark created!");
    println!("  ID: {}", record.id);
    println!("  Name: {}", record.name);
    println!("  Provider: {}", record.provider);
    println!("  Model: {}", record.model);
    println!("  Status: {}", record.status);
    println!();

    // Create some evaluations
    println!("Creating evaluations...");
    for i in 1..=5 {
        let evaluation = NewEvaluation {
            benchmark_id: Some(record.id),
            provider: "openai".to_string(),
            model: "gpt-4".to_string(),
            input: format!("Question {}", i),
            output: format!("Answer {}", i),
            expected: Some(format!("Expected {}", i)),
            metrics: serde_json::json!({
                "accuracy": 0.95,
                "latency_ms": 150,
            }),
            score: 0.95,
            metadata: None,
            created_by: None,
        };

        db.evaluations().create(evaluation).await?;
    }

    println!("✓ Created 5 evaluations\n");

    // Update benchmark status
    let update = UpdateBenchmark {
        status: Some("completed".to_string()),
        completed_iterations: Some(5),
        ..Default::default()
    };

    let updated = db.benchmarks().update(record.id, update).await?;
    println!("✓ Benchmark updated to status: {}", updated.status);

    Ok(())
}

/// Create a job
async fn create_job() -> Result<()> {
    println!("Creating job...\n");

    let config = DatabaseConfig::from_env().unwrap_or_default();
    let db = Database::connect(config).await?;

    let job = NewJob {
        job_type: "benchmark".to_string(),
        payload: serde_json::json!({
            "provider": "openai",
            "model": "gpt-4",
            "iterations": 100
        }),
        priority: 10,
        max_retries: 3,
        timeout_seconds: 600,
    };

    let record = db.jobs().create(job).await?;

    println!("✓ Job created!");
    println!("  ID: {}", record.id);
    println!("  Type: {}", record.job_type);
    println!("  Priority: {}", record.priority);
    println!("  Status: {}", record.status);
    println!();

    // Get job statistics
    let stats = db.jobs().get_stats().await?;
    println!("Job Statistics:");
    println!("  Pending: {}", stats.pending);
    println!("  Running: {}", stats.running);
    println!("  Completed: {}", stats.completed);
    println!("  Failed: {}", stats.failed);
    println!("  Total: {}", stats.total());

    Ok(())
}

/// Register a worker
async fn register_worker() -> Result<()> {
    println!("Registering worker...\n");

    let config = DatabaseConfig::from_env().unwrap_or_default();
    let db = Database::connect(config).await?;

    let worker = NewWorker {
        worker_id: "worker-example-1".to_string(),
        address: "localhost:50052".to_string(),
        capacity: 4,
        tags: vec!["benchmark".to_string(), "evaluation".to_string()],
        metadata: Some(serde_json::json!({
            "version": "1.0.0",
            "os": "linux"
        })),
    };

    let record = db.workers().create(worker).await?;

    println!("✓ Worker registered!");
    println!("  ID: {}", record.id);
    println!("  Worker ID: {}", record.worker_id);
    println!("  Address: {}", record.address);
    println!("  Capacity: {}", record.capacity);
    println!("  Status: {}", record.status);
    println!("  Tags: {:?}", record.tags);
    println!();

    // Update heartbeat
    db.workers().update_heartbeat(&record.worker_id).await?;
    println!("✓ Heartbeat updated");

    // List active workers
    let active = db.workers().list_active(30).await?;
    println!("\nActive Workers: {}", active.len());
    for w in active {
        println!("  - {} ({}) - {} tasks", w.worker_id, w.status, w.current_tasks);
    }

    Ok(())
}

/// Show database statistics
async fn show_stats() -> Result<()> {
    println!("Database Statistics\n");
    println!("==================\n");

    let config = DatabaseConfig::from_env().unwrap_or_default();
    let db = Database::connect(config).await?;

    let stats = db.stats().await?;

    println!("Record Counts:");
    println!("  Benchmarks: {}", stats.benchmark_count);
    println!("  Evaluations: {}", stats.evaluation_count);
    println!("  Jobs: {}", stats.job_count);
    println!("  Workers: {}", stats.worker_count);
    println!("  Users: {}", stats.user_count);
    println!();

    println!("Database Size:");
    println!("  {:.2} MB", stats.database_size_mb());
    println!();

    // Job statistics
    let job_stats = db.jobs().get_stats().await?;
    println!("Job Status:");
    println!("  Pending: {}", job_stats.pending);
    println!("  Running: {}", job_stats.running);
    println!("  Completed: {}", job_stats.completed);
    println!("  Failed: {}", job_stats.failed);
    println!("  Cancelled: {}", job_stats.cancelled);
    println!("  Total: {}", job_stats.total());
    println!();

    // Recent benchmarks
    println!("Recent Benchmarks:");
    let benchmarks = db.benchmarks().list_recent(5).await?;
    for b in benchmarks {
        println!("  - {} ({}) - {}", b.name, b.status, b.provider);
    }

    Ok(())
}

/// Print help message
fn print_help() {
    println!("LLM Test Bench Database Example");
    println!("===============================\n");

    println!("Usage:");
    println!("  cargo run --example database_example <mode>\n");

    println!("Modes:");
    println!("  setup      - Setup database and run migrations");
    println!("  benchmark  - Create a benchmark with evaluations");
    println!("  job        - Create a job");
    println!("  worker     - Register a worker");
    println!("  stats      - Show database statistics\n");

    println!("Environment Variables:");
    println!("  DATABASE_HOST      - Database host (default: localhost)");
    println!("  DATABASE_PORT      - Database port (default: 5432)");
    println!("  DATABASE_NAME      - Database name (default: llm_test_bench)");
    println!("  DATABASE_USER      - Database user (default: postgres)");
    println!("  DATABASE_PASSWORD  - Database password (default: postgres)");
    println!("  DATABASE_POOL_SIZE - Connection pool size (default: 20)\n");

    println!("Examples:");
    println!("  # Setup database");
    println!("  cargo run --example database_example setup\n");

    println!("  # Create a benchmark");
    println!("  cargo run --example database_example benchmark\n");

    println!("  # Show statistics");
    println!("  cargo run --example database_example stats\n");

    println!("Database Schema:");
    println!("  ┌────────────┐   ┌─────────────┐   ┌────────┐");
    println!("  │ Benchmarks │───│ Evaluations │   │  Jobs  │");
    println!("  └────────────┘   └─────────────┘   └────────┘");
    println!("       │                                  │");
    println!("       │           ┌─────────┐            │");
    println!("       └───────────│  Users  │────────────┘");
    println!("                   └─────────┘");
    println!("                        │");
    println!("                   ┌────────┐");
    println!("                   │Workers │");
    println!("                   └────────┘");
    println!();
}

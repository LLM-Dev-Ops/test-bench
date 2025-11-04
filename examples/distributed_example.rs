// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! # Distributed System Example
//!
//! This example demonstrates running a coordinator and workers in a distributed setup.

use llm_test_bench_core::distributed::{
    Coordinator, CoordinatorConfig, Worker, WorkerConfig, JobRequest,
};
use anyhow::Result;
use std::sync::Arc;
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_target(false)
        .with_thread_ids(true)
        .with_level(true)
        .init();

    println!("🔗 LLM Test Bench Distributed System Example");
    println!("============================================\n");

    // Get mode from command line
    let args: Vec<String> = std::env::args().collect();
    let mode = args.get(1).map(|s| s.as_str()).unwrap_or("help");

    match mode {
        "coordinator" => run_coordinator().await?,
        "worker" => run_worker(&args).await?,
        "submit" => submit_job(&args).await?,
        _ => print_help(),
    }

    Ok(())
}

/// Run coordinator node
async fn run_coordinator() -> Result<()> {
    println!("Starting Coordinator Node");
    println!("=========================\n");

    let config = CoordinatorConfig::builder()
        .heartbeat_interval(5)
        .health_check_timeout(10)
        .unhealthy_threshold(30)
        .max_retries(3)
        .build();

    println!("Configuration:");
    println!("  Bind Address: {}", config.bind_address);
    println!("  Heartbeat Interval: {}s", config.heartbeat_interval);
    println!("  Health Check Timeout: {}s", config.health_check_timeout);
    println!("  Max Retries: {}", config.max_retries);
    println!();

    let coordinator = Arc::new(Coordinator::new(config));

    println!("📡 Coordinator API:");
    println!("  - Register workers");
    println!("  - Distribute tasks");
    println!("  - Monitor cluster health");
    println!("  - Collect results");
    println!();

    // Print initial stats
    let stats = coordinator.get_cluster_stats();
    println!("Cluster Statistics:");
    println!("  Workers: {}", stats.total_workers);
    println!("  Jobs: {} total, {} pending, {} running",
        stats.total_jobs, stats.pending_jobs, stats.running_jobs);
    println!();

    println!("⚙️  Starting coordinator...\n");

    // Start coordinator
    coordinator.start().await?;

    Ok(())
}

/// Run worker node
async fn run_worker(args: &[String]) -> Result<()> {
    let worker_id = args.get(2).map(|s| s.as_str()).unwrap_or("worker-1");

    println!("Starting Worker Node: {}", worker_id);
    println!("=========================\n");

    let config = WorkerConfig::builder()
        .worker_id(worker_id)
        .coordinator_address("http://localhost:50051")
        .capacity(4)
        .tags(vec!["benchmark".to_string(), "evaluation".to_string()])
        .build();

    println!("Configuration:");
    println!("  Worker ID: {}", config.worker_id);
    println!("  Coordinator: {}", config.coordinator_address);
    println!("  Capacity: {} concurrent tasks", config.capacity);
    println!("  Tags: {:?}", config.tags);
    println!();

    let worker = Arc::new(Worker::new(config));

    println!("⚙️  Starting worker...\n");

    println!("Worker will:");
    println!("  1. Register with coordinator");
    println!("  2. Send heartbeat every 5s");
    println!("  3. Pull and execute tasks");
    println!("  4. Report results");
    println!();

    // Start worker
    worker.start().await?;

    Ok(())
}

/// Submit a job to the coordinator
async fn submit_job(args: &[String]) -> Result<()> {
    let job_type = args.get(2).map(|s| s.as_str()).unwrap_or("benchmark");

    println!("Submitting Job");
    println!("=============\n");

    // Create coordinator client
    println!("Connecting to coordinator at localhost:50051");

    // Create job request
    let job = JobRequest::builder()
        .job_type(job_type)
        .payload(serde_json::json!({
            "provider": "openai",
            "model": "gpt-4",
            "iterations": 100,
            "dataset": "mmlu"
        }))
        .priority(10)
        .timeout_seconds(600)
        .max_retries(3)
        .build();

    println!("\nJob Details:");
    println!("  Type: {}", job.job_type);
    println!("  Priority: {}", job.priority);
    println!("  Timeout: {}s", job.timeout_seconds);
    println!("  Max Retries: {}", job.max_retries);
    println!();

    // In a real implementation, this would submit to coordinator via gRPC
    println!("Job would be submitted to coordinator");
    println!("Job ID would be returned for tracking");

    Ok(())
}

/// Print help message
fn print_help() {
    println!("LLM Test Bench Distributed System");
    println!("=================================\n");

    println!("Usage:");
    println!("  cargo run --example distributed_example <mode> [options]\n");

    println!("Modes:");
    println!("  coordinator              Start a coordinator node");
    println!("  worker [id]              Start a worker node");
    println!("  submit [job_type]        Submit a job to the coordinator\n");

    println!("Examples:");
    println!("  # Start coordinator");
    println!("  cargo run --example distributed_example coordinator\n");

    println!("  # Start worker");
    println!("  cargo run --example distributed_example worker worker-1\n");

    println!("  # Submit benchmark job");
    println!("  cargo run --example distributed_example submit benchmark\n");

    println!("Architecture:");
    println!("  ┌────────────────────┐");
    println!("  │   Coordinator      │");
    println!("  │   - Job Queue      │");
    println!("  │   - Worker Pool    │");
    println!("  │   - Health Monitor │");
    println!("  └──────────┬─────────┘");
    println!("             │");
    println!("      ┌──────┼──────┐");
    println!("      │      │      │");
    println!("      ▼      ▼      ▼");
    println!("  ┌────┐  ┌────┐  ┌────┐");
    println!("  │ W1 │  │ W2 │  │ WN │");
    println!("  └────┘  └────┘  └────┘");
    println!();

    println!("Features:");
    println!("  ✓ Horizontal scaling");
    println!("  ✓ Fault tolerance");
    println!("  ✓ Load balancing");
    println!("  ✓ Health monitoring");
    println!("  ✓ Job scheduling");
    println!("  ✓ Result caching");
    println!();
}

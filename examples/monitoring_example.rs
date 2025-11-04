// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Example: Real-time Monitoring System
//!
//! This example demonstrates the complete monitoring system with:
//! - Prometheus metrics export
//! - WebSocket real-time dashboards
//! - Automatic provider instrumentation
//! - Live benchmark tracking

use llm_test_bench_core::{
    monitoring::{MonitoringSystem, MonitoringConfig, MonitoredProvider},
    providers::{Provider, CompletionRequest},
};
use std::sync::Arc;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    println!("🚀 LLM Test Bench - Real-time Monitoring Example\n");

    // Step 1: Initialize monitoring system
    println!("📊 Initializing monitoring system...");
    let config = MonitoringConfig::default()
        .with_prometheus_port(9090)
        .with_websocket_port(8080)
        .with_dashboard_port(3000);

    let monitoring = Arc::new(MonitoringSystem::new(config).await?);

    // Step 2: Start monitoring services
    println!("🔧 Starting monitoring services...");
    monitoring.start().await?;

    println!("\n✅ Monitoring system started!");
    println!("   📊 Prometheus: http://localhost:9090/metrics");
    println!("   🔌 WebSocket:  ws://localhost:8080/ws");
    println!("   📱 Dashboard:  http://localhost:3000\n");

    // Step 3: Create and monitor a provider
    println!("🔧 Setting up monitored provider...");

    // For this example, we'll simulate provider calls
    // In real usage, you would use actual providers like OpenAI, Anthropic, etc.
    println!("   (In production, wrap actual providers like OpenAI, Anthropic, etc.)");

    // Step 4: Record some example metrics
    println!("\n📈 Simulating LLM operations...\n");

    for i in 1..=10 {
        println!("Request #{}", i);

        // Simulate a request
        monitoring.record_request("openai", "gpt-4");

        // Simulate processing time
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Record latency
        let latency = 0.5 + (i as f64 * 0.1);
        monitoring.record_latency("openai", latency);

        // Record tokens
        let input_tokens = 100 + (i * 10);
        let output_tokens = 50 + (i * 5);
        monitoring.record_tokens("openai", input_tokens, output_tokens);

        // Record cost
        let cost = 0.001 + (i as f64 * 0.0001);
        monitoring.record_cost("openai", cost);

        println!("  ✓ Latency: {:.2}s", latency);
        println!("  ✓ Tokens: {} in, {} out", input_tokens, output_tokens);
        println!("  ✓ Cost: ${:.4}", cost);
        println!();

        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }

    // Step 5: Demonstrate benchmark tracking
    println!("📊 Simulating benchmark execution...\n");

    let benchmark_id = "example_benchmark";
    let benchmark_name = "MMLU Test";
    let total_examples = 100;

    // Start benchmark
    let event = llm_test_bench_core::monitoring::MonitoringEvent::benchmark_started(
        benchmark_id,
        benchmark_name,
        total_examples,
    );
    monitoring.event_bus().publish(event);

    // Simulate progress
    for progress in (0..=100).step_by(10) {
        let event = llm_test_bench_core::monitoring::MonitoringEvent::benchmark_progress(
            benchmark_id,
            benchmark_name,
            progress,
            total_examples,
        );
        monitoring.event_bus().publish(event);

        println!("  Benchmark progress: {}%", progress);
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    }

    // Complete benchmark
    let event = llm_test_bench_core::monitoring::MonitoringEvent::benchmark_completed(
        benchmark_id,
        benchmark_name,
        total_examples,
    );
    monitoring.event_bus().publish(event);

    println!("\n✅ Benchmark completed!");

    // Step 6: Display provider statistics
    println!("\n📊 Provider Statistics:\n");

    let stats = monitoring.get_all_provider_stats();
    for stat in stats {
        println!("Provider: {}", stat.provider);
        println!("  Total Requests: {}", stat.total_requests);
        println!("  Successful: {}", stat.successful_requests);
        println!("  Failed: {}", stat.failed_requests);
        if let Some(latency) = stat.avg_latency {
            println!("  Avg Latency: {:.2}s", latency);
        }
        println!("  Total Tokens: {}", stat.total_input_tokens + stat.total_output_tokens);
        println!("  Total Cost: ${:.4}", stat.total_cost);
        println!();
    }

    // Step 7: Keep running for dashboard viewing
    println!("🌐 Monitoring system is running!");
    println!("   Open http://localhost:3000 to view the dashboard");
    println!("   Press Ctrl+C to stop\n");

    // Keep the program running
    tokio::signal::ctrl_c().await?;

    println!("\n🛑 Shutting down monitoring system...");
    monitoring.stop().await?;

    println!("✅ Monitoring system stopped\n");

    Ok(())
}

/* Example Output:

🚀 LLM Test Bench - Real-time Monitoring Example

📊 Initializing monitoring system...
🔧 Starting monitoring services...

✅ Monitoring system started!
   📊 Prometheus: http://localhost:9090/metrics
   🔌 WebSocket:  ws://localhost:8080/ws
   📱 Dashboard:  http://localhost:3000

🔧 Setting up monitored provider...
   (In production, wrap actual providers like OpenAI, Anthropic, etc.)

📈 Simulating LLM operations...

Request #1
  ✓ Latency: 0.60s
  ✓ Tokens: 110 in, 55 out
  ✓ Cost: $0.0011

Request #2
  ✓ Latency: 0.70s
  ✓ Tokens: 120 in, 60 out
  ✓ Cost: $0.0012

...

📊 Simulating benchmark execution...

  Benchmark progress: 0%
  Benchmark progress: 10%
  Benchmark progress: 20%
  ...
  Benchmark progress: 100%

✅ Benchmark completed!

📊 Provider Statistics:

Provider: openai
  Total Requests: 10
  Successful: 10
  Failed: 0
  Avg Latency: 1.00s
  Total Tokens: 1650
  Total Cost: $0.0055

🌐 Monitoring system is running!
   Open http://localhost:3000 to view the dashboard
   Press Ctrl+C to stop

*/

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! # API Server Example
//!
//! This example demonstrates how to run the LLM Test Bench API server
//! with REST, GraphQL, and WebSocket support.

use llm_test_bench_core::api::{ApiServer, ApiConfig, CorsConfig};
use anyhow::Result;
use std::net::SocketAddr;
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_target(false)
        .with_thread_ids(true)
        .with_level(true)
        .init();

    println!("🚀 LLM Test Bench API Server Example");
    println!("=====================================\n");

    // Create API configuration
    let config = ApiConfig::builder()
        .bind_address("0.0.0.0:3000".parse::<SocketAddr>()?)
        .enable_rest(true)
        .enable_graphql(true)
        .enable_websocket(true)
        .enable_swagger(true)
        .jwt_secret(std::env::var("JWT_SECRET").unwrap_or_else(|_| {
            "demo_secret_change_in_production".to_string()
        }))
        .jwt_expiration(3600) // 1 hour
        .rate_limit(100, 50) // 100 rps, burst of 50
        .cors(CorsConfig {
            allowed_origins: vec!["http://localhost:3000".to_string()],
            ..Default::default()
        })
        .build();

    println!("Configuration:");
    println!("  Address: {}", config.bind_address);
    println!("  REST API: {}", config.enable_rest);
    println!("  GraphQL: {}", config.enable_graphql);
    println!("  WebSocket: {}", config.enable_websocket);
    println!("  Swagger UI: {}", config.enable_swagger);
    println!();

    // Create and start server
    let server = ApiServer::new(config);

    println!("📡 API Endpoints:");
    println!("  REST API:     http://localhost:3000/v1");
    println!("  GraphQL:      http://localhost:3000/graphql");
    println!("  GraphiQL:     http://localhost:3000/graphql (browser)");
    println!("  WebSocket:    ws://localhost:3000/ws");
    println!("  Swagger UI:   http://localhost:3000/swagger-ui");
    println!("  Health Check: http://localhost:3000/health");
    println!();

    println!("📚 Example Requests:");
    println!();
    println!("REST API - Create Completion:");
    println!(r#"  curl -X POST http://localhost:3000/v1/completions \"#);
    println!(r#"    -H "Content-Type: application/json" \"#);
    println!(r#"    -d '{{
      "provider": "openai",
      "model": "gpt-4",
      "prompt": "Hello, world!",
      "max_tokens": 100
    }}'
"#);

    println!("GraphQL - Query:");
    println!(r#"  curl -X POST http://localhost:3000/graphql \"#);
    println!(r#"    -H "Content-Type: application/json" \"#);
    println!(r#"    -d '{{
      "query": "{{ version health }}"
    }}'
"#);

    println!("WebSocket - Connect:");
    println!(r#"  wscat -c ws://localhost:3000/ws
  > {{"type": "subscribe", "topics": ["benchmark.progress"]}}
"#);

    println!("🔐 Authentication:");
    println!("  The API supports JWT tokens and API keys.");
    println!("  Set the Authorization header: 'Bearer <token>'");
    println!("  Or use X-API-Key header: 'ltb_<key>'");
    println!();

    println!("⚙️  Starting server...\n");

    // Start the server (this will block)
    server.start().await?;

    Ok(())
}

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! # Enterprise API Server
//!
//! This module provides a comprehensive API server with REST, GraphQL, and WebSocket support.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    API Server (Axum)                        │
//! │                                                             │
//! │  ┌────────────────┐  ┌────────────────┐  ┌────────────────┐ │
//! │  │   REST API     │  │  GraphQL API   │  │   WebSocket    │ │
//! │  │  (OpenAPI)     │  │  (async-gql)   │  │  (real-time)   │ │
//! │  └────────┬───────┘  └────────┬───────┘  └────────┬───────┘ │
//! │           │                   │                   │          │
//! │           └───────────────────┼───────────────────┘          │
//! │                               ▼                              │
//! │  ┌───────────────────────────────────────────────────────┐  │
//! │  │              Middleware Layer                         │  │
//! │  │  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐   │  │
//! │  │  │  Auth   │ │  CORS   │ │  Rate   │ │ Logging │   │  │
//! │  │  │  (JWT)  │ │         │ │ Limit   │ │         │   │  │
//! │  │  └─────────┘ └─────────┘ └─────────┘ └─────────┘   │  │
//! │  └───────────────────────────────────────────────────────┘  │
//! │                               ▼                              │
//! │  ┌───────────────────────────────────────────────────────┐  │
//! │  │              Core Business Logic                      │  │
//! │  │  Providers | Benchmarks | Evaluators | Plugins       │  │
//! │  └───────────────────────────────────────────────────────┘  │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Features
//!
//! - **REST API**: OpenAPI-documented endpoints
//! - **GraphQL API**: Flexible querying and mutations
//! - **WebSocket**: Real-time updates and streaming
//! - **Authentication**: JWT-based auth with role-based access
//! - **Rate Limiting**: Per-user and per-endpoint limits
//! - **CORS**: Configurable cross-origin support
//! - **Swagger UI**: Interactive API documentation
//! - **API Versioning**: v1, v2, etc.
//!
//! ## Usage
//!
//! ```rust,no_run
//! use llm_test_bench_core::api::{ApiServer, ApiConfig};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let config = ApiConfig::default();
//!     let server = ApiServer::new(config);
//!
//!     server.start().await?;
//!
//!     Ok(())
//! }
//! ```

pub mod server;
pub mod rest;
pub mod graphql;
pub mod websocket;
pub mod auth;
pub mod middleware;
pub mod models;
pub mod error;

pub use server::{ApiServer, ApiConfig, ApiConfigBuilder, AppState};
pub use rest::RestApi;
pub use graphql::{GraphQLApi, GraphQLSchema};
pub use websocket::{WsState, WsMessage, WsTopic};
pub use auth::{AuthService, JwtClaims, ApiKey, UserRole};
pub use middleware::{RateLimiter, CorsConfig};
pub use models::*;
pub use error::{ApiError, ApiResult};

/// API server version
pub const API_VERSION: &str = "1.0.0";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_version() {
        assert!(!API_VERSION.is_empty());
    }
}

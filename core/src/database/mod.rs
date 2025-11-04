// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! # Database Backend (PostgreSQL)
//!
//! This module provides persistent storage for all LLM Test Bench components
//! using PostgreSQL.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                   Application Layer                         │
//! │  ┌────────────┐  ┌────────────┐  ┌────────────┐           │
//! │  │    API     │  │ Coordinator│  │  Workers   │           │
//! │  └─────┬──────┘  └─────┬──────┘  └─────┬──────┘           │
//! │        │               │               │                   │
//! └────────┼───────────────┼───────────────┼───────────────────┘
//!          │               │               │
//! ┌────────┼───────────────┼───────────────┼───────────────────┐
//! │        │       Repository Layer        │                   │
//! │        ▼               ▼               ▼                   │
//! │  ┌─────────────────────────────────────────────────────┐  │
//! │  │  BenchmarkRepo │ JobRepo │ WorkerRepo │ UserRepo   │  │
//! │  └─────────────────────────────────────────────────────┘  │
//! │                          │                                 │
//! │                          ▼                                 │
//! │  ┌───────────────────────────────────────────────────┐    │
//! │  │              Connection Pool (sqlx)               │    │
//! │  └───────────────────────────────────────────────────┘    │
//! │                          │                                 │
//! └──────────────────────────┼─────────────────────────────────┘
//!                            │
//! ┌──────────────────────────┼─────────────────────────────────┐
//! │                          ▼                                 │
//! │  ┌───────────────────────────────────────────────────┐    │
//! │  │              PostgreSQL Database                   │    │
//! │  │  ┌──────────┐ ┌──────────┐ ┌──────────┐          │    │
//! │  │  │Benchmarks│ │   Jobs   │ │ Workers  │          │    │
//! │  │  └──────────┘ └──────────┘ └──────────┘          │    │
//! │  └───────────────────────────────────────────────────┘    │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Features
//!
//! - **Connection Pooling**: Efficient connection management with sqlx
//! - **Migrations**: Automatic schema migrations
//! - **Repository Pattern**: Clean data access layer
//! - **Transaction Support**: ACID compliance
//! - **Audit Logging**: Track all data changes
//! - **Query Optimization**: Indexes and query planning
//! - **Type Safety**: Compile-time query validation
//!
//! ## Usage
//!
//! ```rust,no_run
//! use llm_test_bench_core::database::{Database, DatabaseConfig};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let config = DatabaseConfig::from_env()?;
//!     let db = Database::connect(config).await?;
//!
//!     // Run migrations
//!     db.migrate().await?;
//!
//!     // Use repositories
//!     let benchmarks = db.benchmarks();
//!     let results = benchmarks.list_recent(10).await?;
//!
//!     Ok(())
//! }
//! ```

pub mod config;
pub mod connection;
pub mod models;
pub mod error;
pub mod repositories;

pub use config::{DatabaseConfig, DatabaseConfigBuilder};
pub use connection::Database;
pub use error::{DatabaseError, DatabaseResult};
pub use models::*;
pub use repositories::{
    BenchmarkRepository, EvaluationRepository, JobRepository,
    WorkerRepository, UserRepository, AuditRepository,
};

/// Database version for migrations
pub const DATABASE_VERSION: &str = "1.0.0";

/// Default database name
pub const DEFAULT_DATABASE_NAME: &str = "llm_test_bench";

/// Default connection pool size
pub const DEFAULT_POOL_SIZE: u32 = 20;

/// Default connection timeout (seconds)
pub const DEFAULT_CONNECT_TIMEOUT: u64 = 10;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert_eq!(DATABASE_VERSION, "1.0.0");
        assert_eq!(DEFAULT_DATABASE_NAME, "llm_test_bench");
        assert_eq!(DEFAULT_POOL_SIZE, 20);
    }
}

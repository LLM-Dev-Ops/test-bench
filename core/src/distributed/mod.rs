// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! # Distributed Architecture
//!
//! This module implements a coordinator-worker architecture for distributed
//! execution of benchmarks and evaluations across multiple nodes.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                      Coordinator                            │
//! │  ┌────────────────┐  ┌────────────────┐  ┌────────────────┐ │
//! │  │  Job Queue     │  │  Worker Pool   │  │ Health Monitor │ │
//! │  │  - Pending     │  │  - Active      │  │  - Heartbeat   │ │
//! │  │  - Running     │  │  - Idle        │  │  - Status      │ │
//! │  │  - Completed   │  │  - Failed      │  │  - Metrics     │ │
//! │  └────────────────┘  └────────────────┘  └────────────────┘ │
//! │           │                   │                   │          │
//! │           └───────────────────┼───────────────────┘          │
//! │                               │                              │
//! │                          gRPC API                            │
//! └───────────────────────────────┼──────────────────────────────┘
//!                                 │
//!          ┌──────────────────────┼──────────────────────┐
//!          │                      │                      │
//!          ▼                      ▼                      ▼
//!  ┌───────────────┐      ┌───────────────┐     ┌───────────────┐
//!  │   Worker 1    │      │   Worker 2    │     │   Worker N    │
//!  │               │      │               │     │               │
//!  │  - Executor   │      │  - Executor   │     │  - Executor   │
//!  │  - Cache      │      │  - Cache      │     │  - Cache      │
//!  │  - Metrics    │      │  - Metrics    │     │  - Metrics    │
//!  └───────────────┘      └───────────────┘     └───────────────┘
//! ```
//!
//! ## Features
//!
//! - **Horizontal Scaling**: Add workers to increase capacity
//! - **Fault Tolerance**: Automatic task retry on worker failure
//! - **Load Balancing**: Work stealing and fair distribution
//! - **Health Monitoring**: Continuous health checks and metrics
//! - **Job Scheduling**: Priority queues and dependencies
//! - **Result Caching**: Distributed result cache
//! - **Graceful Shutdown**: Clean worker deregistration
//!
//! ## Usage
//!
//! ### Starting a Coordinator
//!
//! ```rust,no_run
//! use llm_test_bench_core::distributed::{Coordinator, CoordinatorConfig};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let config = CoordinatorConfig::default();
//!     let coordinator = Coordinator::new(config).await?;
//!
//!     coordinator.start().await?;
//!     Ok(())
//! }
//! ```
//!
//! ### Starting a Worker
//!
//! ```rust,no_run
//! use llm_test_bench_core::distributed::{Worker, WorkerConfig};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let config = WorkerConfig::builder()
//!         .coordinator_address("http://localhost:50051")
//!         .worker_id("worker-1")
//!         .build();
//!
//!     let worker = Worker::new(config).await?;
//!     worker.start().await?;
//!     Ok(())
//! }
//! ```
//!
//! ### Submitting Jobs
//!
//! ```rust,no_run
//! use llm_test_bench_core::distributed::{Coordinator, JobRequest};
//!
//! async fn submit_benchmark(coordinator: &Coordinator) -> anyhow::Result<String> {
//!     let job = JobRequest::new()
//!         .job_type("benchmark")
//!         .payload(serde_json::json!({
//!             "provider": "openai",
//!             "model": "gpt-4",
//!             "iterations": 100
//!         }))
//!         .priority(10)
//!         .build();
//!
//!     let job_id = coordinator.submit_job(job).await?;
//!     Ok(job_id)
//! }
//! ```

pub mod types;
pub mod protocol;
pub mod coordinator;
pub mod worker;
pub mod jobs;
pub mod cluster;
pub mod health;

pub use types::{
    DistributedError, DistributedResult, NodeId, JobId, WorkerInfo, WorkerStatus,
};
pub use protocol::{
    JobRequest, JobResponse, TaskRequest, TaskResponse, HealthCheckRequest,
    HealthCheckResponse, RegisterRequest, RegisterResponse,
};
pub use coordinator::{Coordinator, CoordinatorConfig};
pub use worker::{Worker, WorkerConfig};
pub use jobs::{Job, JobStatus, JobQueue, JobPriority};
pub use cluster::{ClusterState, ClusterMetrics};
pub use health::{HealthMonitor, HealthStatus};

/// Distributed system version
pub const DISTRIBUTED_VERSION: &str = "1.0.0";

/// Default gRPC port for coordinator
pub const DEFAULT_COORDINATOR_PORT: u16 = 50051;

/// Default heartbeat interval (seconds)
pub const DEFAULT_HEARTBEAT_INTERVAL: u64 = 5;

/// Default health check timeout (seconds)
pub const DEFAULT_HEALTH_CHECK_TIMEOUT: u64 = 10;

/// Default max retries for failed tasks
pub const DEFAULT_MAX_RETRIES: u32 = 3;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert_eq!(DEFAULT_COORDINATOR_PORT, 50051);
        assert_eq!(DEFAULT_HEARTBEAT_INTERVAL, 5);
        assert_eq!(DEFAULT_HEALTH_CHECK_TIMEOUT, 10);
        assert_eq!(DEFAULT_MAX_RETRIES, 3);
    }
}

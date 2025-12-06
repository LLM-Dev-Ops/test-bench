// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! # LLM Test Bench Core
//!
//! This crate provides the core business logic and provider integrations
//! for the LLM Test Bench framework.
//!
//! ## Modules
//!
//! - `config`: Configuration management and validation
//! - `providers`: LLM provider implementations (OpenAI, Anthropic, etc.)
//! - `evaluators`: Evaluation metrics (perplexity, faithfulness, relevance, coherence)
//! - `benchmarks`: Benchmarking logic and reporting
//! - `orchestration`: Multi-model comparison, ranking, and routing
//! - `analytics`: Statistical analysis and cost optimization
//! - `visualization`: HTML dashboard generation with interactive charts
//! - `multimodal`: Multi-modal support for vision, audio, and video
//! - `monitoring`: Real-time monitoring with Prometheus and WebSocket dashboards
//! - `plugins`: WASM-based plugin system for extensibility
//! - `api`: REST, GraphQL, and WebSocket API server
//! - `distributed`: Coordinator-worker distributed architecture
//! - `database`: PostgreSQL database backend
//! - `infra`: LLM Dev Ops Infrastructure integration (Phase 2B)
//!
//! ## Infrastructure Integration
//!
//! Enable the `infra-core` feature for unified error handling, configuration,
//! and tracing from the LLM-Dev-Ops/infra repository:
//!
//! ```toml
//! [dependencies]
//! llm-test-bench-core = { version = "0.1", features = ["infra-core"] }
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]
#![deny(clippy::correctness)]

pub mod config;
pub mod providers;
pub mod evaluators;
pub mod benchmarks;
pub mod orchestration;
pub mod analytics;
pub mod visualization;
pub mod multimodal;
pub mod monitoring;
pub mod plugins;
pub mod api;
pub mod distributed;
#[cfg(feature = "database")]
pub mod database;

// LLM Dev Ops Infrastructure Integration (Phase 2B)
// Enable via feature flags: infra-core, infra-testing, or infra-full
#[cfg(any(
    feature = "infra-errors-feature",
    feature = "infra-config-feature",
    feature = "infra-otel-feature",
    feature = "infra-sim-feature",
    feature = "infra-vector-feature"
))]
pub mod infra;

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Re-export commonly used types
pub mod prelude {
    pub use crate::config::Config;
    pub use crate::providers::Provider;
    pub use crate::evaluators::Evaluator;
    pub use crate::multimodal::{
        MultiModalRequest, MultiModalResponse, ImageInput, AudioInput,
    };
    pub use crate::monitoring::{
        MonitoringSystem, MonitoringConfig, MonitoringEvent, EventBus,
    };
    pub use crate::plugins::{
        PluginSystem, PluginManager, PluginType, PluginInput, PluginOutput,
    };
    pub use crate::api::{
        ApiServer, ApiConfig, AppState, ApiError, ApiResult,
    };
    pub use crate::distributed::{
        Coordinator, CoordinatorConfig, Worker, WorkerConfig,
        JobRequest, JobStatus, ClusterMetrics,
    };
    #[cfg(feature = "database")]
    pub use crate::database::{
        Database, DatabaseConfig, DatabaseError, DatabaseResult,
    };

    // LLM Dev Ops Infra re-exports (Phase 2B)
    #[cfg(feature = "infra-errors-feature")]
    pub use infra_errors::{InfraError, InfraResult};

    #[cfg(feature = "infra-config-feature")]
    pub use infra_config::{ConfigLoader as InfraConfigLoader, ConfigFormat};

    #[cfg(feature = "infra-otel-feature")]
    pub use infra_otel::{OtelConfig, init as init_otel};

    #[cfg(feature = "infra-sim-feature")]
    pub use infra_sim::{MockBuilder, MockResponse, SimulatedClock};

    #[cfg(feature = "infra-vector-feature")]
    pub use infra_vector::{Vector, VectorIndex, cosine_similarity};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }
}

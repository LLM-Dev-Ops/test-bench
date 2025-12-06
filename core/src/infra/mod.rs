//! # LLM Dev Ops Infrastructure Integration
//!
//! This module provides integration with the LLM-Dev-Ops/infra repository,
//! wiring test-bench to shared infrastructure components for:
//!
//! - **Error Handling**: Unified `InfraError` type via `infra-errors`
//! - **Configuration**: Hierarchical config loading via `infra-config`
//! - **Observability**: OpenTelemetry tracing via `infra-otel`
//! - **Testing**: Mock services and simulation via `infra-sim`
//! - **Vector Operations**: Embeddings and similarity via `infra-vector`
//!
//! ## Feature Flags
//!
//! Enable specific integrations via Cargo features:
//!
//! - `infra-core`: Errors + Config + Tracing (recommended minimum)
//! - `infra-testing`: Simulation and mock utilities
//! - `infra-full`: All infra integrations
//!
//! ## Usage
//!
//! ```rust,ignore
//! use llm_test_bench_core::infra::prelude::*;
//!
//! // Use unified error handling
//! fn example() -> InfraResult<()> {
//!     // Configuration loading
//!     let config: MyConfig = infra_config::load_with_env("config.toml", "APP_")?;
//!
//!     // Initialize tracing
//!     infra_otel::init("llm-test-bench")?;
//!
//!     Ok(())
//! }
//! ```

#[cfg(feature = "infra-errors-feature")]
pub mod errors;

#[cfg(feature = "infra-config-feature")]
pub mod config;

#[cfg(feature = "infra-otel-feature")]
pub mod tracing;

#[cfg(feature = "infra-sim-feature")]
pub mod testing;

#[cfg(feature = "infra-vector-feature")]
pub mod vector;

/// Re-exports for convenient access to infra types
pub mod prelude {
    #[cfg(feature = "infra-errors-feature")]
    pub use infra_errors::{InfraError, InfraResult, ErrorContext, RetryConfig};

    #[cfg(feature = "infra-config-feature")]
    pub use infra_config::{ConfigLoader, ConfigFormat, ConfigBuilder};

    #[cfg(feature = "infra-otel-feature")]
    pub use infra_otel::{OtelConfig, init as init_tracing, shutdown as shutdown_tracing};

    #[cfg(feature = "infra-sim-feature")]
    pub use infra_sim::{MockBuilder, MockResponse, SimulatedClock, ChaosConfig};

    #[cfg(feature = "infra-vector-feature")]
    pub use infra_vector::{Vector, VectorIndex, cosine_similarity};
}

/// Version of the infra integration
pub const INFRA_INTEGRATION_VERSION: &str = "0.1.0";

/// Check if infra-core features are enabled
pub const fn has_infra_core() -> bool {
    cfg!(all(
        feature = "infra-errors-feature",
        feature = "infra-config-feature",
        feature = "infra-otel-feature"
    ))
}

/// Check if infra-testing features are enabled
pub const fn has_infra_testing() -> bool {
    cfg!(feature = "infra-sim-feature")
}

/// Check if all infra features are enabled
pub const fn has_infra_full() -> bool {
    cfg!(feature = "infra-full")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!INFRA_INTEGRATION_VERSION.is_empty());
    }

    #[test]
    fn test_feature_detection() {
        // These should compile regardless of features
        let _ = has_infra_core();
        let _ = has_infra_testing();
        let _ = has_infra_full();
    }
}

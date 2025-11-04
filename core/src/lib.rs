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

#![warn(missing_docs)]
#![warn(clippy::all)]
#![deny(clippy::correctness)]

pub mod config;
pub mod providers;
pub mod evaluators;
pub mod benchmarks;

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Re-export commonly used types
pub mod prelude {
    pub use crate::config::Config;
    pub use crate::providers::Provider;
    pub use crate::evaluators::Evaluator;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }
}

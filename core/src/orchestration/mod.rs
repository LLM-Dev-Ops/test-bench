// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Multi-model orchestration with comparison, ranking, and routing.
//!
//! This module provides enterprise-grade functionality for:
//! - Parallel multi-model execution and comparison
//! - Intelligent ranking based on quality, performance, and cost
//! - Automated model selection and routing
//!
//! # Examples
//!
//! ## Comparing Multiple Models
//!
//! ```no_run
//! use llm_test_bench_core::orchestration::{ComparisonEngine, ComparisonConfig, ModelConfig};
//! use std::sync::Arc;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let mut engine = ComparisonEngine::new();
//!
//! // Register providers and evaluators
//! // engine.register_provider("openai", Arc::new(openai_provider));
//! // engine.register_evaluator("quality", Arc::new(quality_evaluator));
//!
//! let config = ComparisonConfig::new(vec![
//!     ModelConfig::new("openai", "gpt-4"),
//!     ModelConfig::new("anthropic", "claude-3-opus-20240229"),
//! ]);
//!
//! let result = engine.compare("Explain quantum computing", config).await?;
//! println!("Winner: {:?}", result.winner);
//! # Ok(())
//! # }
//! ```
//!
//! ## Intelligent Model Routing
//!
//! ```no_run
//! use llm_test_bench_core::orchestration::{ModelRouter, ModelConfig, ModelConstraints, RoutingStrategy};
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let mut router = ModelRouter::new(RoutingStrategy::Balanced);
//!
//! // Load historical profiles
//! // router.load_profiles()?;
//!
//! let available_models = vec![
//!     ModelConfig::new("openai", "gpt-4"),
//!     ModelConfig::new("openai", "gpt-3.5-turbo"),
//! ];
//!
//! let constraints = ModelConstraints::new()
//!     .with_max_cost(0.01)
//!     .with_min_quality(0.8);
//!
//! let selection = router.select_model(
//!     "Write a function to sort an array",
//!     &available_models,
//!     &constraints,
//! )?;
//!
//! println!("Selected: {}", selection.model_config.identifier());
//! println!("Reasoning: {}", selection.reasoning);
//! # Ok(())
//! # }
//! ```

pub mod comparison;
pub mod ranking;
pub mod router;
pub mod types;

// Re-export main types
pub use comparison::{ComparisonEngine, ComparisonError};
pub use ranking::{RankingEngine, RankingError, RankingWeights};
pub use router::{ModelRouter, RoutingError};
pub use types::{
    ComparativeAnalysis, ComparisonConfig, ComparisonResult, ComponentScores, Finding,
    FindingCategory, ModelAlternative, ModelConfig, ModelConstraints, ModelProfile, ModelRanking,
    ModelResult, ModelSelection, Recommendation, RecommendationType, RoutingStrategy,
    SignificantDifference, SignificanceTest, TaskType,
};

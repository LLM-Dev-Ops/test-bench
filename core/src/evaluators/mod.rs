// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Evaluation metrics for LLM responses
//!
//! This module provides various metrics for evaluating LLM outputs:
//! - Perplexity: Language model prediction quality
//! - Faithfulness: Factual accuracy and hallucination detection
//! - Relevance: Task/prompt alignment scoring
//! - Coherence: Output fluency and logical consistency

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Evaluator errors
#[derive(Error, Debug)]
pub enum EvaluatorError {
    /// Invalid input data
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Evaluation failed
    #[error("Evaluation failed: {0}")]
    EvaluationFailed(String),
}

/// Evaluation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationResult {
    /// Name of the metric
    pub metric: String,

    /// Score (0.0 - 1.0, where higher is better unless otherwise noted)
    pub score: f64,

    /// Additional details about the evaluation
    #[serde(default)]
    pub details: serde_json::Value,
}

/// Evaluator trait
pub trait Evaluator: Send + Sync {
    /// Evaluate a response
    fn evaluate(&self, prompt: &str, response: &str) -> Result<EvaluationResult, EvaluatorError>;

    /// Get the evaluator name
    fn name(&self) -> &str;
}

// Metric implementations
pub mod perplexity;
pub mod faithfulness;
pub mod relevance;
pub mod coherence;

pub use perplexity::PerplexityEvaluator;
pub use faithfulness::FaithfulnessEvaluator;
pub use relevance::RelevanceEvaluator;
pub use coherence::CoherenceEvaluator;

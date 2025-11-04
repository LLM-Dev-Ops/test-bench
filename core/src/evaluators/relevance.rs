// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Relevance metric implementation

use super::{EvaluationResult, Evaluator, EvaluatorError};

/// Relevance evaluator - measures task/prompt alignment
pub struct RelevanceEvaluator;

impl RelevanceEvaluator {
    pub fn new() -> Self {
        Self
    }
}

impl Default for RelevanceEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

impl Evaluator for RelevanceEvaluator {
    fn evaluate(&self, _prompt: &str, _response: &str) -> Result<EvaluationResult, EvaluatorError> {
        Ok(EvaluationResult {
            metric: "relevance".to_string(),
            score: 0.0,
            details: serde_json::json!({"status": "not_implemented"}),
        })
    }

    fn name(&self) -> &str {
        "Relevance"
    }
}

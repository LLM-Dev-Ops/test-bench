// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Faithfulness metric implementation

use super::{EvaluationResult, Evaluator, EvaluatorError};

/// Faithfulness evaluator - measures factual accuracy
pub struct FaithfulnessEvaluator;

impl FaithfulnessEvaluator {
    pub fn new() -> Self {
        Self
    }
}

impl Default for FaithfulnessEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

impl Evaluator for FaithfulnessEvaluator {
    fn evaluate(&self, _prompt: &str, _response: &str) -> Result<EvaluationResult, EvaluatorError> {
        Ok(EvaluationResult {
            metric: "faithfulness".to_string(),
            score: 0.0,
            details: serde_json::json!({"status": "not_implemented"}),
        })
    }

    fn name(&self) -> &str {
        "Faithfulness"
    }
}

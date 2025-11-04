// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Coherence metric implementation

use super::{EvaluationResult, Evaluator, EvaluatorError};

/// Coherence evaluator - measures output fluency and logical consistency
pub struct CoherenceEvaluator;

impl CoherenceEvaluator {
    pub fn new() -> Self {
        Self
    }
}

impl Default for CoherenceEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

impl Evaluator for CoherenceEvaluator {
    fn evaluate(&self, _prompt: &str, _response: &str) -> Result<EvaluationResult, EvaluatorError> {
        Ok(EvaluationResult {
            metric: "coherence".to_string(),
            score: 0.0,
            details: serde_json::json!({"status": "not_implemented"}),
        })
    }

    fn name(&self) -> &str {
        "Coherence"
    }
}

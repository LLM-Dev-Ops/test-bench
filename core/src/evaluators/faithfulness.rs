// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Faithfulness metric implementation
//!
//! This module implements hallucination detection using LLM-as-judge.
//! It extracts claims from responses and verifies each claim against
//! the original context (prompt).

use super::{EvaluationResult, Evaluator, EvaluatorError};
use super::llm_judge::{LLMJudge, JudgeConfig};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// A specific hallucination detected in the response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hallucination {
    /// The hallucinated claim
    pub claim: String,

    /// Explanation of why this is considered a hallucination
    pub explanation: String,

    /// Severity (0.0 - 1.0, where 1.0 is most severe)
    pub severity: f64,
}

/// Detailed faithfulness score breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaithfulnessScore {
    /// Overall faithfulness score (0.0-1.0, where 1.0 = fully faithful)
    pub overall_score: f64,

    /// Number of verified claims
    pub verified_claims: usize,

    /// Total number of claims extracted
    pub total_claims: usize,

    /// List of detected hallucinations
    pub hallucinations: Vec<Hallucination>,

    /// Confidence in the evaluation (0.0 - 1.0)
    pub confidence: f64,

    /// Detailed reasoning
    pub reasoning: String,

    /// Cost of this evaluation
    pub cost: f64,
}

/// Faithfulness evaluator - measures factual accuracy and detects hallucinations
///
/// This evaluator uses LLM-as-judge to:
/// 1. Extract claims from the response
/// 2. Verify each claim against the context
/// 3. Calculate faithfulness score as verified_claims / total_claims
/// 4. Identify specific hallucinations with explanations
///
/// # Examples
///
/// ```no_run
/// use llm_test_bench_core::evaluators::faithfulness::FaithfulnessEvaluator;
/// use llm_test_bench_core::evaluators::llm_judge::JudgeConfig;
/// use llm_test_bench_core::providers::OpenAIProvider;
/// use std::sync::Arc;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let provider = Arc::new(OpenAIProvider::new("key".to_string()));
/// let config = JudgeConfig::new("gpt-4");
/// let evaluator = FaithfulnessEvaluator::new(provider, config);
///
/// let prompt = "The capital of France is Paris.";
/// let response = "The capital of France is Paris, which has a population of about 2 million.";
///
/// let result = evaluator.evaluate(prompt, response).await?;
/// println!("Faithfulness score: {}", result.score);
/// # Ok(())
/// # }
/// ```
pub struct FaithfulnessEvaluator {
    judge: Arc<LLMJudge>,
}

impl FaithfulnessEvaluator {
    /// Create a new faithfulness evaluator with the given judge
    pub fn new(provider: Arc<dyn crate::providers::Provider>, config: JudgeConfig) -> Self {
        Self {
            judge: Arc::new(LLMJudge::new(provider, config)),
        }
    }

    /// Create a new faithfulness evaluator with an existing judge
    pub fn with_judge(judge: Arc<LLMJudge>) -> Self {
        Self { judge }
    }

    /// Get the underlying LLM judge
    pub fn judge(&self) -> &Arc<LLMJudge> {
        &self.judge
    }

    /// Perform faithfulness evaluation
    async fn evaluate_faithfulness(
        &self,
        prompt: &str,
        response: &str,
    ) -> Result<FaithfulnessScore, EvaluatorError> {
        // Build the faithfulness rubric
        let rubric = Self::faithfulness_rubric();

        // Evaluate using the judge
        let eval_result = self
            .judge
            .evaluate(prompt, response, "faithfulness", &rubric)
            .await
            .map_err(|e| EvaluatorError::EvaluationFailed(e.to_string()))?;

        // Parse the detailed response
        self.parse_faithfulness_result(&eval_result.reasoning, eval_result.confidence, eval_result.cost)
    }

    /// Get the faithfulness evaluation rubric
    fn faithfulness_rubric() -> String {
        r#"You are evaluating the FAITHFULNESS of a response. Faithfulness measures whether the response contains only information that can be verified from the given context (the original prompt).

Your task:
1. Extract all factual claims from the response
2. For each claim, determine if it can be verified from the context
3. Identify any hallucinations (claims that cannot be verified or contradict the context)
4. Calculate the faithfulness score as: verified_claims / total_claims

Scoring guidelines:
- 1.0: All claims can be verified from the context
- 0.8-0.99: Mostly faithful with minor unverifiable details
- 0.5-0.79: Some hallucinations present
- 0.2-0.49: Significant hallucinations
- 0.0-0.19: Mostly hallucinated content

Provide your assessment in JSON format:
{
    "score": <float 0.0-1.0>,
    "reasoning": "<overall assessment>",
    "confidence": <float 0.0-1.0>,
    "verified_claims": <int>,
    "total_claims": <int>,
    "hallucinations": [
        {
            "claim": "<hallucinated claim>",
            "explanation": "<why this is a hallucination>",
            "severity": <float 0.0-1.0>
        }
    ]
}"#.to_string()
    }

    /// Parse the faithfulness result from the judge's reasoning
    fn parse_faithfulness_result(
        &self,
        reasoning: &str,
        confidence: f64,
        cost: f64,
    ) -> Result<FaithfulnessScore, EvaluatorError> {
        // Try to extract JSON from reasoning
        let json_str = self.extract_json(reasoning)?;

        // Parse the JSON
        let parsed: serde_json::Value = serde_json::from_str(&json_str)
            .map_err(|e| EvaluatorError::EvaluationFailed(format!("JSON parse error: {}", e)))?;

        let overall_score = parsed["score"]
            .as_f64()
            .ok_or_else(|| EvaluatorError::EvaluationFailed("Missing score".to_string()))?;

        let verified_claims = parsed["verified_claims"]
            .as_u64()
            .unwrap_or(0) as usize;

        let total_claims = parsed["total_claims"]
            .as_u64()
            .unwrap_or(1) as usize;

        let reasoning_text = parsed["reasoning"]
            .as_str()
            .unwrap_or("No reasoning provided")
            .to_string();

        // Parse hallucinations array
        let hallucinations = if let Some(hall_array) = parsed["hallucinations"].as_array() {
            hall_array
                .iter()
                .filter_map(|h| {
                    Some(Hallucination {
                        claim: h["claim"].as_str()?.to_string(),
                        explanation: h["explanation"].as_str()?.to_string(),
                        severity: h["severity"].as_f64().unwrap_or(0.5),
                    })
                })
                .collect()
        } else {
            Vec::new()
        };

        Ok(FaithfulnessScore {
            overall_score,
            verified_claims,
            total_claims,
            hallucinations,
            confidence,
            reasoning: reasoning_text,
            cost,
        })
    }

    /// Extract JSON from text (handles cases where LLM adds extra text)
    fn extract_json(&self, text: &str) -> Result<String, EvaluatorError> {
        let trimmed = text.trim();

        // Try to find JSON object boundaries
        if let Some(start) = trimmed.find('{') {
            if let Some(end) = trimmed.rfind('}') {
                if start < end {
                    return Ok(trimmed[start..=end].to_string());
                }
            }
        }

        // If no braces found, return error
        Err(EvaluatorError::EvaluationFailed(
            "No JSON object found in response".to_string(),
        ))
    }
}

#[async_trait]
impl Evaluator for FaithfulnessEvaluator {
    async fn evaluate(&self, prompt: &str, response: &str) -> Result<EvaluationResult, EvaluatorError> {
        // Validate inputs
        if prompt.trim().is_empty() {
            return Err(EvaluatorError::InvalidInput("Prompt cannot be empty".to_string()));
        }
        if response.trim().is_empty() {
            return Err(EvaluatorError::InvalidInput("Response cannot be empty".to_string()));
        }

        // Perform faithfulness evaluation
        let score = self.evaluate_faithfulness(prompt, response).await?;

        // Convert to standard EvaluationResult
        Ok(EvaluationResult {
            metric: "faithfulness".to_string(),
            score: score.overall_score,
            details: serde_json::to_value(score)
                .map_err(|e| EvaluatorError::EvaluationFailed(e.to_string()))?,
        })
    }

    fn name(&self) -> &str {
        "Faithfulness"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::{CompletionResponse, FinishReason, TokenUsage};
    use mockall::mock;
    use mockall::predicate::*;
    use chrono::Utc;

    mock! {
        Provider {}

        #[async_trait::async_trait]
        impl crate::providers::Provider for Provider {
            async fn complete(&self, request: crate::providers::CompletionRequest) -> Result<CompletionResponse, crate::providers::ProviderError>;
            async fn stream(&self, request: crate::providers::CompletionRequest) -> Result<crate::providers::ResponseStream, crate::providers::ProviderError>;
            fn supported_models(&self) -> Vec<crate::providers::ModelInfo>;
            fn max_context_length(&self, model: &str) -> Option<usize>;
            fn name(&self) -> &str;
            async fn validate_config(&self) -> Result<(), crate::providers::ProviderError>;
            fn estimate_tokens(&self, text: &str, model: &str) -> Result<usize, crate::providers::ProviderError>;
        }
    }

    fn create_mock_response(content: String) -> CompletionResponse {
        CompletionResponse {
            id: "test-123".to_string(),
            model: "gpt-4".to_string(),
            content,
            usage: TokenUsage::new(100, 50),
            finish_reason: FinishReason::Stop,
            created_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_faithfulness_perfect_score() {
        let mut mock = MockProvider::new();

        let response_json = r#"{
            "score": 1.0,
            "reasoning": "All claims are verified from the context.",
            "confidence": 0.95,
            "verified_claims": 3,
            "total_claims": 3,
            "hallucinations": []
        }"#;

        mock.expect_complete()
            .times(1)
            .returning(move |_| Ok(create_mock_response(response_json.to_string())));

        let config = JudgeConfig::new("gpt-4");
        let evaluator = FaithfulnessEvaluator::new(Arc::new(mock), config);

        let result = evaluator
            .evaluate(
                "Paris is the capital of France.",
                "Paris is the capital of France and a major European city.",
            )
            .await
            .unwrap();

        assert_eq!(result.score, 1.0);
        assert_eq!(result.metric, "faithfulness");

        let details: FaithfulnessScore = serde_json::from_value(result.details).unwrap();
        assert_eq!(details.verified_claims, 3);
        assert_eq!(details.total_claims, 3);
        assert!(details.hallucinations.is_empty());
    }

    #[tokio::test]
    async fn test_faithfulness_with_hallucinations() {
        let mut mock = MockProvider::new();

        let response_json = r#"{
            "score": 0.67,
            "reasoning": "Response contains some hallucinated information.",
            "confidence": 0.9,
            "verified_claims": 2,
            "total_claims": 3,
            "hallucinations": [
                {
                    "claim": "Paris has 10 million residents",
                    "explanation": "The population number is not mentioned in the context",
                    "severity": 0.6
                }
            ]
        }"#;

        mock.expect_complete()
            .times(1)
            .returning(move |_| Ok(create_mock_response(response_json.to_string())));

        let config = JudgeConfig::new("gpt-4");
        let evaluator = FaithfulnessEvaluator::new(Arc::new(mock), config);

        let result = evaluator
            .evaluate(
                "Paris is the capital of France.",
                "Paris is the capital of France and has 10 million residents.",
            )
            .await
            .unwrap();

        assert!((result.score - 0.67).abs() < 0.01);

        let details: FaithfulnessScore = serde_json::from_value(result.details).unwrap();
        assert_eq!(details.verified_claims, 2);
        assert_eq!(details.total_claims, 3);
        assert_eq!(details.hallucinations.len(), 1);
        assert_eq!(
            details.hallucinations[0].claim,
            "Paris has 10 million residents"
        );
    }

    #[tokio::test]
    async fn test_faithfulness_empty_prompt() {
        let mock = MockProvider::new();
        let config = JudgeConfig::new("gpt-4");
        let evaluator = FaithfulnessEvaluator::new(Arc::new(mock), config);

        let result = evaluator.evaluate("", "Some response").await;

        assert!(result.is_err());
        match result {
            Err(EvaluatorError::InvalidInput(msg)) => {
                assert!(msg.contains("Prompt cannot be empty"));
            }
            _ => panic!("Expected InvalidInput error"),
        }
    }

    #[tokio::test]
    async fn test_faithfulness_empty_response() {
        let mock = MockProvider::new();
        let config = JudgeConfig::new("gpt-4");
        let evaluator = FaithfulnessEvaluator::new(Arc::new(mock), config);

        let result = evaluator.evaluate("Some prompt", "").await;

        assert!(result.is_err());
        match result {
            Err(EvaluatorError::InvalidInput(msg)) => {
                assert!(msg.contains("Response cannot be empty"));
            }
            _ => panic!("Expected InvalidInput error"),
        }
    }

    #[tokio::test]
    async fn test_faithfulness_severe_hallucinations() {
        let mut mock = MockProvider::new();

        let response_json = r#"{
            "score": 0.0,
            "reasoning": "Response is completely fabricated.",
            "confidence": 1.0,
            "verified_claims": 0,
            "total_claims": 5,
            "hallucinations": [
                {
                    "claim": "Berlin is the capital of France",
                    "explanation": "This directly contradicts the context",
                    "severity": 1.0
                }
            ]
        }"#;

        mock.expect_complete()
            .times(1)
            .returning(move |_| Ok(create_mock_response(response_json.to_string())));

        let config = JudgeConfig::new("gpt-4");
        let evaluator = FaithfulnessEvaluator::new(Arc::new(mock), config);

        let result = evaluator
            .evaluate(
                "Paris is the capital of France.",
                "Berlin is the capital of France.",
            )
            .await
            .unwrap();

        assert_eq!(result.score, 0.0);

        let details: FaithfulnessScore = serde_json::from_value(result.details).unwrap();
        assert_eq!(details.verified_claims, 0);
        assert!(details.hallucinations[0].severity >= 0.9);
    }

    #[tokio::test]
    async fn test_faithfulness_json_extraction() {
        let mut mock = MockProvider::new();

        let response_with_extra = r#"Here's my analysis:

        {
            "score": 0.85,
            "reasoning": "Good response",
            "confidence": 0.9,
            "verified_claims": 4,
            "total_claims": 5,
            "hallucinations": []
        }

        That's my evaluation."#;

        mock.expect_complete()
            .times(1)
            .returning(move |_| Ok(create_mock_response(response_with_extra.to_string())));

        let config = JudgeConfig::new("gpt-4");
        let evaluator = FaithfulnessEvaluator::new(Arc::new(mock), config);

        let result = evaluator
            .evaluate("Context", "Response")
            .await
            .unwrap();

        assert_eq!(result.score, 0.85);
    }

    #[tokio::test]
    async fn test_faithfulness_caching() {
        let mut mock = MockProvider::new();

        let response_json = r#"{
            "score": 0.9,
            "reasoning": "Test",
            "confidence": 0.9,
            "verified_claims": 9,
            "total_claims": 10,
            "hallucinations": []
        }"#;

        mock.expect_complete()
            .times(1) // Should only be called once due to caching
            .returning(move |_| Ok(create_mock_response(response_json.to_string())));

        let config = JudgeConfig::new("gpt-4");
        let evaluator = FaithfulnessEvaluator::new(Arc::new(mock), config);

        // First call - cache miss
        let result1 = evaluator
            .evaluate("prompt", "response")
            .await
            .unwrap();

        // Second call - cache hit
        let result2 = evaluator
            .evaluate("prompt", "response")
            .await
            .unwrap();

        assert_eq!(result1.score, result2.score);

        let stats = evaluator.judge().cache_stats().unwrap();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
    }

    #[test]
    fn test_hallucination_serialization() {
        let hallucination = Hallucination {
            claim: "Test claim".to_string(),
            explanation: "Test explanation".to_string(),
            severity: 0.8,
        };

        let json = serde_json::to_string(&hallucination).unwrap();
        let deserialized: Hallucination = serde_json::from_str(&json).unwrap();

        assert_eq!(hallucination.claim, deserialized.claim);
        assert_eq!(hallucination.explanation, deserialized.explanation);
        assert_eq!(hallucination.severity, deserialized.severity);
    }

    #[test]
    fn test_faithfulness_score_serialization() {
        let score = FaithfulnessScore {
            overall_score: 0.75,
            verified_claims: 3,
            total_claims: 4,
            hallucinations: vec![Hallucination {
                claim: "Test".to_string(),
                explanation: "Explanation".to_string(),
                severity: 0.5,
            }],
            confidence: 0.9,
            reasoning: "Test reasoning".to_string(),
            cost: 0.001,
        };

        let json = serde_json::to_string(&score).unwrap();
        let deserialized: FaithfulnessScore = serde_json::from_str(&json).unwrap();

        assert_eq!(score.overall_score, deserialized.overall_score);
        assert_eq!(score.verified_claims, deserialized.verified_claims);
        assert_eq!(score.hallucinations.len(), deserialized.hallucinations.len());
    }
}

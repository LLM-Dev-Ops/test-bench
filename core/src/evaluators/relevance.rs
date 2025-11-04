// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Relevance metric implementation
//!
//! This module implements multi-dimensional relevance scoring using LLM-as-judge.
//! It evaluates topic alignment, instruction following, and completeness.

use super::{EvaluationResult, Evaluator, EvaluatorError};
use super::llm_judge::{LLMJudge, JudgeConfig};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Detailed relevance score breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelevanceScore {
    /// Overall relevance score (0.0-1.0, where 1.0 = highly relevant)
    pub overall_score: f64,

    /// Topic alignment score (0.0-1.0)
    /// Measures how well the response addresses the main topic
    pub topic_alignment: f64,

    /// Instruction following score (0.0-1.0)
    /// Measures how well the response follows specific instructions
    pub instruction_following: f64,

    /// Completeness score (0.0-1.0)
    /// Measures whether the response fully addresses all aspects of the prompt
    pub completeness: f64,

    /// Detailed reasoning for the scores
    pub reasoning: String,

    /// Confidence in the evaluation (0.0 - 1.0)
    pub confidence: f64,

    /// Cost of this evaluation
    pub cost: f64,
}

/// Relevance evaluator - measures task/prompt alignment
///
/// This evaluator uses LLM-as-judge with multi-dimensional scoring to evaluate:
/// 1. Topic alignment - How well the response addresses the main topic
/// 2. Instruction following - How well specific instructions are followed
/// 3. Completeness - Whether all aspects of the prompt are addressed
/// 4. Overall relevance - Weighted combination of the above
///
/// # Examples
///
/// ```no_run
/// use llm_test_bench_core::evaluators::relevance::RelevanceEvaluator;
/// use llm_test_bench_core::evaluators::llm_judge::JudgeConfig;
/// use llm_test_bench_core::providers::OpenAIProvider;
/// use std::sync::Arc;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let provider = Arc::new(OpenAIProvider::new("key".to_string()));
/// let config = JudgeConfig::new("gpt-4");
/// let evaluator = RelevanceEvaluator::new(provider, config);
///
/// let prompt = "Explain photosynthesis in simple terms for a 10-year-old.";
/// let response = "Photosynthesis is how plants make food using sunlight, water, and air.";
///
/// let result = evaluator.evaluate(prompt, response).await?;
/// println!("Relevance score: {}", result.score);
/// # Ok(())
/// # }
/// ```
pub struct RelevanceEvaluator {
    judge: Arc<LLMJudge>,
}

impl RelevanceEvaluator {
    /// Create a new relevance evaluator with the given judge
    pub fn new(provider: Arc<dyn crate::providers::Provider>, config: JudgeConfig) -> Self {
        Self {
            judge: Arc::new(LLMJudge::new(provider, config)),
        }
    }

    /// Create a new relevance evaluator with an existing judge
    pub fn with_judge(judge: Arc<LLMJudge>) -> Self {
        Self { judge }
    }

    /// Get the underlying LLM judge
    pub fn judge(&self) -> &Arc<LLMJudge> {
        &self.judge
    }

    /// Perform relevance evaluation
    async fn evaluate_relevance(
        &self,
        prompt: &str,
        response: &str,
    ) -> Result<RelevanceScore, EvaluatorError> {
        // Build the relevance rubric
        let rubric = Self::relevance_rubric();

        // Evaluate using the judge
        let eval_result = self
            .judge
            .evaluate(prompt, response, "relevance", &rubric)
            .await
            .map_err(|e| EvaluatorError::EvaluationFailed(e.to_string()))?;

        // Parse the detailed response
        self.parse_relevance_result(&eval_result.reasoning, eval_result.confidence, eval_result.cost)
    }

    /// Get the relevance evaluation rubric
    fn relevance_rubric() -> String {
        r#"You are evaluating the RELEVANCE of a response to a given prompt. Relevance measures how well the response addresses the prompt's requirements across multiple dimensions.

Your task:
1. Evaluate TOPIC ALIGNMENT (0.0-1.0):
   - Does the response stay on topic?
   - Does it address the main subject of the prompt?
   - Is there any off-topic content?

2. Evaluate INSTRUCTION FOLLOWING (0.0-1.0):
   - Does the response follow any specific instructions in the prompt?
   - Are format requirements met?
   - Is the appropriate level of detail provided?
   - Is the target audience considered?

3. Evaluate COMPLETENESS (0.0-1.0):
   - Are all parts of the prompt addressed?
   - Is any important aspect missing?
   - Is the response thorough?

4. Calculate OVERALL RELEVANCE:
   - Weighted average of the above scores
   - Consider the relative importance of each dimension

Scoring guidelines:
- 1.0: Perfect alignment, follows all instructions, completely addresses prompt
- 0.8-0.99: Excellent relevance with minor issues
- 0.6-0.79: Good relevance but some aspects missed
- 0.4-0.59: Partially relevant, significant gaps
- 0.2-0.39: Mostly irrelevant or off-topic
- 0.0-0.19: Completely irrelevant

Provide your assessment in JSON format:
{
    "score": <float 0.0-1.0>,
    "reasoning": "<detailed explanation of overall relevance>",
    "confidence": <float 0.0-1.0>,
    "topic_alignment": <float 0.0-1.0>,
    "instruction_following": <float 0.0-1.0>,
    "completeness": <float 0.0-1.0>
}"#.to_string()
    }

    /// Parse the relevance result from the judge's reasoning
    fn parse_relevance_result(
        &self,
        reasoning: &str,
        confidence: f64,
        cost: f64,
    ) -> Result<RelevanceScore, EvaluatorError> {
        // Try to extract JSON from reasoning
        let json_str = self.extract_json(reasoning)?;

        // Parse the JSON
        let parsed: serde_json::Value = serde_json::from_str(&json_str)
            .map_err(|e| EvaluatorError::EvaluationFailed(format!("JSON parse error: {}", e)))?;

        let overall_score = parsed["score"]
            .as_f64()
            .ok_or_else(|| EvaluatorError::EvaluationFailed("Missing overall score".to_string()))?;

        let topic_alignment = parsed["topic_alignment"]
            .as_f64()
            .ok_or_else(|| EvaluatorError::EvaluationFailed("Missing topic_alignment".to_string()))?;

        let instruction_following = parsed["instruction_following"]
            .as_f64()
            .ok_or_else(|| EvaluatorError::EvaluationFailed("Missing instruction_following".to_string()))?;

        let completeness = parsed["completeness"]
            .as_f64()
            .ok_or_else(|| EvaluatorError::EvaluationFailed("Missing completeness".to_string()))?;

        let reasoning_text = parsed["reasoning"]
            .as_str()
            .unwrap_or("No reasoning provided")
            .to_string();

        // Validate score ranges
        if !(0.0..=1.0).contains(&overall_score) {
            return Err(EvaluatorError::EvaluationFailed(
                format!("Overall score {} is outside valid range [0.0, 1.0]", overall_score)
            ));
        }

        if !(0.0..=1.0).contains(&topic_alignment) {
            return Err(EvaluatorError::EvaluationFailed(
                format!("Topic alignment {} is outside valid range [0.0, 1.0]", topic_alignment)
            ));
        }

        if !(0.0..=1.0).contains(&instruction_following) {
            return Err(EvaluatorError::EvaluationFailed(
                format!("Instruction following {} is outside valid range [0.0, 1.0]", instruction_following)
            ));
        }

        if !(0.0..=1.0).contains(&completeness) {
            return Err(EvaluatorError::EvaluationFailed(
                format!("Completeness {} is outside valid range [0.0, 1.0]", completeness)
            ));
        }

        Ok(RelevanceScore {
            overall_score,
            topic_alignment,
            instruction_following,
            completeness,
            reasoning: reasoning_text,
            confidence,
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
impl Evaluator for RelevanceEvaluator {
    async fn evaluate(&self, prompt: &str, response: &str) -> Result<EvaluationResult, EvaluatorError> {
        // Validate inputs
        if prompt.trim().is_empty() {
            return Err(EvaluatorError::InvalidInput("Prompt cannot be empty".to_string()));
        }
        if response.trim().is_empty() {
            return Err(EvaluatorError::InvalidInput("Response cannot be empty".to_string()));
        }

        // Perform relevance evaluation
        let score = self.evaluate_relevance(prompt, response).await?;

        // Convert to standard EvaluationResult
        Ok(EvaluationResult {
            metric: "relevance".to_string(),
            score: score.overall_score,
            details: serde_json::to_value(score)
                .map_err(|e| EvaluatorError::EvaluationFailed(e.to_string()))?,
        })
    }

    fn name(&self) -> &str {
        "Relevance"
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
    async fn test_relevance_perfect_score() {
        let mut mock = MockProvider::new();

        let response_json = r#"{
            "score": 1.0,
            "reasoning": "Response perfectly addresses all aspects of the prompt.",
            "confidence": 0.95,
            "topic_alignment": 1.0,
            "instruction_following": 1.0,
            "completeness": 1.0
        }"#;

        mock.expect_complete()
            .times(1)
            .returning(move |_| Ok(create_mock_response(response_json.to_string())));

        let config = JudgeConfig::new("gpt-4");
        let evaluator = RelevanceEvaluator::new(Arc::new(mock), config);

        let result = evaluator
            .evaluate(
                "Explain photosynthesis.",
                "Photosynthesis is the process by which plants convert light energy into chemical energy.",
            )
            .await
            .unwrap();

        assert_eq!(result.score, 1.0);
        assert_eq!(result.metric, "relevance");

        let details: RelevanceScore = serde_json::from_value(result.details).unwrap();
        assert_eq!(details.topic_alignment, 1.0);
        assert_eq!(details.instruction_following, 1.0);
        assert_eq!(details.completeness, 1.0);
    }

    #[tokio::test]
    async fn test_relevance_partial_score() {
        let mut mock = MockProvider::new();

        let response_json = r#"{
            "score": 0.70,
            "reasoning": "Response addresses the topic but lacks detail and misses some aspects.",
            "confidence": 0.85,
            "topic_alignment": 0.9,
            "instruction_following": 0.6,
            "completeness": 0.6
        }"#;

        mock.expect_complete()
            .times(1)
            .returning(move |_| Ok(create_mock_response(response_json.to_string())));

        let config = JudgeConfig::new("gpt-4");
        let evaluator = RelevanceEvaluator::new(Arc::new(mock), config);

        let result = evaluator
            .evaluate(
                "Explain photosynthesis in detail for a college student.",
                "Plants make food from sunlight.",
            )
            .await
            .unwrap();

        assert_eq!(result.score, 0.70);

        let details: RelevanceScore = serde_json::from_value(result.details).unwrap();
        assert_eq!(details.topic_alignment, 0.9);
        assert_eq!(details.instruction_following, 0.6);
        assert_eq!(details.completeness, 0.6);
    }

    #[tokio::test]
    async fn test_relevance_off_topic() {
        let mut mock = MockProvider::new();

        let response_json = r#"{
            "score": 0.1,
            "reasoning": "Response is completely off-topic.",
            "confidence": 1.0,
            "topic_alignment": 0.0,
            "instruction_following": 0.0,
            "completeness": 0.3
        }"#;

        mock.expect_complete()
            .times(1)
            .returning(move |_| Ok(create_mock_response(response_json.to_string())));

        let config = JudgeConfig::new("gpt-4");
        let evaluator = RelevanceEvaluator::new(Arc::new(mock), config);

        let result = evaluator
            .evaluate(
                "Explain photosynthesis.",
                "The weather today is sunny and warm.",
            )
            .await
            .unwrap();

        assert!((result.score - 0.1).abs() < 0.01);

        let details: RelevanceScore = serde_json::from_value(result.details).unwrap();
        assert_eq!(details.topic_alignment, 0.0);
    }

    #[tokio::test]
    async fn test_relevance_empty_prompt() {
        let mock = MockProvider::new();
        let config = JudgeConfig::new("gpt-4");
        let evaluator = RelevanceEvaluator::new(Arc::new(mock), config);

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
    async fn test_relevance_empty_response() {
        let mock = MockProvider::new();
        let config = JudgeConfig::new("gpt-4");
        let evaluator = RelevanceEvaluator::new(Arc::new(mock), config);

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
    async fn test_relevance_json_extraction() {
        let mut mock = MockProvider::new();

        let response_with_extra = r#"Here's my evaluation:

        {
            "score": 0.85,
            "reasoning": "Good response",
            "confidence": 0.9,
            "topic_alignment": 0.9,
            "instruction_following": 0.8,
            "completeness": 0.85
        }

        That's my assessment."#;

        mock.expect_complete()
            .times(1)
            .returning(move |_| Ok(create_mock_response(response_with_extra.to_string())));

        let config = JudgeConfig::new("gpt-4");
        let evaluator = RelevanceEvaluator::new(Arc::new(mock), config);

        let result = evaluator
            .evaluate("Prompt", "Response")
            .await
            .unwrap();

        assert_eq!(result.score, 0.85);
    }

    #[tokio::test]
    async fn test_relevance_caching() {
        let mut mock = MockProvider::new();

        let response_json = r#"{
            "score": 0.8,
            "reasoning": "Test",
            "confidence": 0.85,
            "topic_alignment": 0.85,
            "instruction_following": 0.75,
            "completeness": 0.8
        }"#;

        mock.expect_complete()
            .times(1) // Should only be called once due to caching
            .returning(move |_| Ok(create_mock_response(response_json.to_string())));

        let config = JudgeConfig::new("gpt-4");
        let evaluator = RelevanceEvaluator::new(Arc::new(mock), config);

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

    #[tokio::test]
    async fn test_relevance_invalid_score_range() {
        let mut mock = MockProvider::new();

        let response_json = r#"{
            "score": 1.5,
            "reasoning": "Test",
            "confidence": 0.9,
            "topic_alignment": 0.9,
            "instruction_following": 0.8,
            "completeness": 0.85
        }"#;

        mock.expect_complete()
            .times(1)
            .returning(move |_| Ok(create_mock_response(response_json.to_string())));

        let config = JudgeConfig::new("gpt-4");
        let evaluator = RelevanceEvaluator::new(Arc::new(mock), config);

        let result = evaluator.evaluate("prompt", "response").await;

        assert!(result.is_err());
        match result {
            Err(EvaluatorError::EvaluationFailed(msg)) => {
                assert!(msg.contains("outside valid range"));
            }
            _ => panic!("Expected EvaluationFailed error"),
        }
    }

    #[test]
    fn test_relevance_score_serialization() {
        let score = RelevanceScore {
            overall_score: 0.85,
            topic_alignment: 0.9,
            instruction_following: 0.8,
            completeness: 0.85,
            reasoning: "Test reasoning".to_string(),
            confidence: 0.9,
            cost: 0.001,
        };

        let json = serde_json::to_string(&score).unwrap();
        let deserialized: RelevanceScore = serde_json::from_str(&json).unwrap();

        assert_eq!(score.overall_score, deserialized.overall_score);
        assert_eq!(score.topic_alignment, deserialized.topic_alignment);
        assert_eq!(score.instruction_following, deserialized.instruction_following);
        assert_eq!(score.completeness, deserialized.completeness);
    }

    #[tokio::test]
    async fn test_relevance_instruction_emphasis() {
        let mut mock = MockProvider::new();

        let response_json = r#"{
            "score": 0.65,
            "reasoning": "Response is on-topic but doesn't follow specific format instructions.",
            "confidence": 0.9,
            "topic_alignment": 0.95,
            "instruction_following": 0.3,
            "completeness": 0.7
        }"#;

        mock.expect_complete()
            .times(1)
            .returning(move |_| Ok(create_mock_response(response_json.to_string())));

        let config = JudgeConfig::new("gpt-4");
        let evaluator = RelevanceEvaluator::new(Arc::new(mock), config);

        let result = evaluator
            .evaluate(
                "List three benefits of exercise in bullet points.",
                "Exercise is good for your health. It helps with fitness and wellbeing.",
            )
            .await
            .unwrap();

        let details: RelevanceScore = serde_json::from_value(result.details).unwrap();
        assert!(details.topic_alignment > 0.9);
        assert!(details.instruction_following < 0.5);
    }
}

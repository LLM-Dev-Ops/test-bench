// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Model ranking and comparative analysis.
//!
//! This module provides functionality for ranking models based on multiple criteria
//! and generating comparative analysis with insights and recommendations.

use std::collections::HashMap;
use thiserror::Error;

use super::types::{
    ComparativeAnalysis, ComponentScores, Finding, FindingCategory, ModelRanking, ModelResult,
    Recommendation, RecommendationType, SignificantDifference, SignificanceTest,
};

/// Errors that can occur during ranking.
#[derive(Error, Debug)]
pub enum RankingError {
    /// No successful results to rank
    #[error("No successful results to rank")]
    NoSuccessfulResults,

    /// Invalid weights
    #[error("Invalid weights: {0}")]
    InvalidWeights(String),

    /// Calculation error
    #[error("Calculation error: {0}")]
    CalculationError(String),
}

/// Weights for ranking components.
#[derive(Debug, Clone)]
pub struct RankingWeights {
    /// Quality weight (0.0 - 1.0)
    pub quality: f64,

    /// Performance weight (0.0 - 1.0)
    pub performance: f64,

    /// Cost efficiency weight (0.0 - 1.0)
    pub cost_efficiency: f64,
}

impl Default for RankingWeights {
    fn default() -> Self {
        Self {
            quality: 0.6,
            performance: 0.2,
            cost_efficiency: 0.2,
        }
    }
}

impl RankingWeights {
    /// Validate that weights sum to 1.0.
    pub fn validate(&self) -> Result<(), RankingError> {
        let sum = self.quality + self.performance + self.cost_efficiency;
        if (sum - 1.0).abs() > 0.01 {
            return Err(RankingError::InvalidWeights(format!(
                "Weights must sum to 1.0, got {}",
                sum
            )));
        }
        Ok(())
    }
}

/// Engine for calculating model rankings and generating analysis.
pub struct RankingEngine {
    /// Weights for ranking components
    weights: RankingWeights,
}

impl RankingEngine {
    /// Create a new ranking engine with default weights.
    pub fn new() -> Self {
        Self {
            weights: RankingWeights::default(),
        }
    }

    /// Create a new ranking engine with custom weights.
    pub fn with_weights(weights: RankingWeights) -> Result<Self, RankingError> {
        weights.validate()?;
        Ok(Self { weights })
    }

    /// Calculate rankings for a set of model results.
    pub fn calculate_rankings(
        &self,
        results: &[ModelResult],
    ) -> Result<Vec<ModelRanking>, RankingError> {
        // Filter to only successful results
        let successful_results: Vec<_> = results.iter().filter(|r| r.success).collect();

        if successful_results.is_empty() {
            return Err(RankingError::NoSuccessfulResults);
        }

        // Calculate component scores for each model
        let mut rankings: Vec<ModelRanking> = successful_results
            .iter()
            .map(|result| {
                let component_scores = self.calculate_component_scores(result, &successful_results);
                let overall_score = self.calculate_overall_score(&component_scores);

                ModelRanking {
                    model_config: result.model_config.clone(),
                    rank: 0, // Will be set after sorting
                    overall_score,
                    component_scores,
                    strengths: self.identify_strengths(result, &successful_results),
                    weaknesses: self.identify_weaknesses(result, &successful_results),
                }
            })
            .collect();

        // Sort by overall score (descending)
        rankings.sort_by(|a, b| {
            b.overall_score
                .partial_cmp(&a.overall_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Assign ranks
        for (i, ranking) in rankings.iter_mut().enumerate() {
            ranking.rank = i + 1;
        }

        Ok(rankings)
    }

    /// Calculate component scores for a single model.
    fn calculate_component_scores(
        &self,
        result: &ModelResult,
        all_results: &[&ModelResult],
    ) -> ComponentScores {
        let quality = self.calculate_quality_score(result);
        let performance = self.calculate_performance_score(result, all_results);
        let cost_efficiency = self.calculate_cost_efficiency_score(result, all_results);

        ComponentScores {
            quality,
            performance,
            cost_efficiency,
        }
    }

    /// Calculate quality score from evaluation metrics.
    fn calculate_quality_score(&self, result: &ModelResult) -> f64 {
        if result.evaluation_scores.is_empty() {
            return 0.0;
        }

        // Weighted average of evaluation metrics
        // Faithfulness gets higher weight (40%), relevance (30%), coherence (30%)
        let mut weighted_sum = 0.0;
        let mut total_weight = 0.0;

        if let Some(&faithfulness) = result.evaluation_scores.get("faithfulness") {
            weighted_sum += faithfulness * 0.4;
            total_weight += 0.4;
        }

        if let Some(&relevance) = result.evaluation_scores.get("relevance") {
            weighted_sum += relevance * 0.3;
            total_weight += 0.3;
        }

        if let Some(&coherence) = result.evaluation_scores.get("coherence") {
            weighted_sum += coherence * 0.3;
            total_weight += 0.3;
        }

        // If standard metrics aren't available, use average of all metrics
        if total_weight == 0.0 {
            return result.average_quality_score();
        }

        weighted_sum / total_weight
    }

    /// Calculate performance score based on latency.
    fn calculate_performance_score(&self, result: &ModelResult, all_results: &[&ModelResult]) -> f64 {
        if all_results.is_empty() {
            return 0.0;
        }

        // Find min and max latencies for normalization
        let latencies: Vec<u64> = all_results.iter().map(|r| r.latency_ms).collect();
        let min_latency = *latencies.iter().min().unwrap_or(&0);
        let max_latency = *latencies.iter().max().unwrap_or(&1);

        if max_latency == min_latency {
            return 1.0; // All models have same latency
        }

        // Normalize: faster = higher score
        // Score = 1 - (latency - min) / (max - min)
        1.0 - ((result.latency_ms - min_latency) as f64 / (max_latency - min_latency) as f64)
    }

    /// Calculate cost efficiency score.
    fn calculate_cost_efficiency_score(&self, result: &ModelResult, all_results: &[&ModelResult]) -> f64 {
        let quality = self.calculate_quality_score(result);

        if result.cost <= 0.0 {
            return quality; // If cost is 0, just return quality
        }

        // Cost efficiency = quality / cost
        let efficiency = quality / result.cost;

        // Normalize across all models
        let efficiencies: Vec<f64> = all_results
            .iter()
            .filter(|r| r.cost > 0.0)
            .map(|r| {
                let q = self.calculate_quality_score(r);
                q / r.cost
            })
            .collect();

        if efficiencies.is_empty() {
            return quality;
        }

        let max_efficiency = efficiencies
            .iter()
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();

        if *max_efficiency <= 0.0 {
            return 0.0;
        }

        efficiency / max_efficiency
    }

    /// Calculate overall score from component scores.
    fn calculate_overall_score(&self, components: &ComponentScores) -> f64 {
        components.quality * self.weights.quality
            + components.performance * self.weights.performance
            + components.cost_efficiency * self.weights.cost_efficiency
    }

    /// Identify strengths of a model compared to others.
    fn identify_strengths(&self, result: &ModelResult, all_results: &[&ModelResult]) -> Vec<String> {
        let mut strengths = Vec::new();

        let quality = self.calculate_quality_score(result);
        let avg_quality: f64 = all_results
            .iter()
            .map(|r| self.calculate_quality_score(r))
            .sum::<f64>()
            / all_results.len() as f64;

        if quality > avg_quality + 0.1 {
            strengths.push("High quality responses".to_string());
        }

        let avg_latency: f64 = all_results.iter().map(|r| r.latency_ms as f64).sum::<f64>()
            / all_results.len() as f64;

        if (result.latency_ms as f64) < avg_latency * 0.8 {
            strengths.push("Fast response time".to_string());
        }

        let avg_cost: f64 = all_results.iter().map(|r| r.cost).sum::<f64>() / all_results.len() as f64;

        if result.cost < avg_cost * 0.8 {
            strengths.push("Cost-effective".to_string());
        }

        if strengths.is_empty() {
            strengths.push("Balanced performance".to_string());
        }

        strengths
    }

    /// Identify weaknesses of a model compared to others.
    fn identify_weaknesses(&self, result: &ModelResult, all_results: &[&ModelResult]) -> Vec<String> {
        let mut weaknesses = Vec::new();

        let quality = self.calculate_quality_score(result);
        let avg_quality: f64 = all_results
            .iter()
            .map(|r| self.calculate_quality_score(r))
            .sum::<f64>()
            / all_results.len() as f64;

        if quality < avg_quality - 0.1 {
            weaknesses.push("Lower quality scores".to_string());
        }

        let avg_latency: f64 = all_results.iter().map(|r| r.latency_ms as f64).sum::<f64>()
            / all_results.len() as f64;

        if (result.latency_ms as f64) > avg_latency * 1.2 {
            weaknesses.push("Slower response time".to_string());
        }

        let avg_cost: f64 = all_results.iter().map(|r| r.cost).sum::<f64>() / all_results.len() as f64;

        if result.cost > avg_cost * 1.2 {
            weaknesses.push("Higher cost".to_string());
        }

        weaknesses
    }

    /// Generate comparative analysis across all models.
    pub fn generate_comparative_analysis(
        &self,
        results: &[ModelResult],
        rankings: &[ModelRanking],
    ) -> ComparativeAnalysis {
        let summary = self.generate_summary(rankings);
        let key_findings = self.generate_findings(results, rankings);
        let model_comparison_matrix = self.generate_comparison_matrix(results);
        let consensus_areas = self.identify_consensus_areas(results);
        let divergence_areas = self.identify_divergence_areas(results);
        let recommendations = self.generate_recommendations(rankings);

        ComparativeAnalysis {
            summary,
            key_findings,
            model_comparison_matrix,
            consensus_areas,
            divergence_areas,
            recommendations,
        }
    }

    /// Generate a summary of the comparison.
    fn generate_summary(&self, rankings: &[ModelRanking]) -> String {
        if rankings.is_empty() {
            return "No models successfully completed the comparison.".to_string();
        }

        let winner = &rankings[0];
        let winner_id = winner.model_config.identifier();

        format!(
            "Compared {} model(s). {} achieved the highest overall score ({:.2}), excelling in: {}",
            rankings.len(),
            winner_id,
            winner.overall_score,
            winner.strengths.join(", ")
        )
    }

    /// Generate key findings from the comparison.
    fn generate_findings(&self, results: &[ModelResult], rankings: &[ModelRanking]) -> Vec<Finding> {
        let mut findings = Vec::new();

        // Quality finding
        if let Some(best_quality) = rankings
            .iter()
            .max_by(|a, b| {
                a.component_scores
                    .quality
                    .partial_cmp(&b.component_scores.quality)
                    .unwrap()
            })
        {
            findings.push(Finding {
                category: FindingCategory::Quality,
                description: format!(
                    "{} achieved the highest quality score",
                    best_quality.model_config.identifier()
                ),
                evidence: format!(
                    "Quality score: {:.2}",
                    best_quality.component_scores.quality
                ),
            });
        }

        // Performance finding
        if let Some(fastest) = results.iter().filter(|r| r.success).min_by_key(|r| r.latency_ms) {
            findings.push(Finding {
                category: FindingCategory::Performance,
                description: format!(
                    "{} was the fastest model",
                    fastest.model_config.identifier()
                ),
                evidence: format!("Latency: {}ms", fastest.latency_ms),
            });
        }

        // Cost finding
        if let Some(cheapest) = results
            .iter()
            .filter(|r| r.success && r.cost > 0.0)
            .min_by(|a, b| a.cost.partial_cmp(&b.cost).unwrap())
        {
            findings.push(Finding {
                category: FindingCategory::Cost,
                description: format!(
                    "{} was the most cost-effective",
                    cheapest.model_config.identifier()
                ),
                evidence: format!("Cost: ${:.4}", cheapest.cost),
            });
        }

        // Reliability finding
        let success_rate = results.iter().filter(|r| r.success).count() as f64 / results.len() as f64;
        findings.push(Finding {
            category: FindingCategory::Reliability,
            description: format!("Overall success rate: {:.1}%", success_rate * 100.0),
            evidence: format!(
                "{} of {} models completed successfully",
                results.iter().filter(|r| r.success).count(),
                results.len()
            ),
        });

        findings
    }

    /// Generate a pairwise comparison matrix.
    fn generate_comparison_matrix(&self, results: &[ModelResult]) -> Vec<Vec<f64>> {
        let n = results.len();
        let mut matrix = vec![vec![0.0; n]; n];

        for i in 0..n {
            for j in 0..n {
                if i == j {
                    matrix[i][j] = 1.0;
                } else {
                    let quality_i = self.calculate_quality_score(&results[i]);
                    let quality_j = self.calculate_quality_score(&results[j]);

                    if quality_j > 0.0 {
                        matrix[i][j] = quality_i / quality_j;
                    } else {
                        matrix[i][j] = if quality_i > 0.0 { 2.0 } else { 1.0 };
                    }
                }
            }
        }

        matrix
    }

    /// Identify areas where models agree.
    fn identify_consensus_areas(&self, results: &[ModelResult]) -> Vec<String> {
        // For now, return a placeholder
        // In a full implementation, this would analyze response similarity
        vec!["Models generally agree on key points".to_string()]
    }

    /// Identify areas where models diverge.
    fn identify_divergence_areas(&self, results: &[ModelResult]) -> Vec<String> {
        // For now, return a placeholder
        // In a full implementation, this would analyze response differences
        let quality_variance = self.calculate_quality_variance(results);

        if quality_variance > 0.1 {
            vec!["Significant quality variation across models".to_string()]
        } else {
            vec![]
        }
    }

    /// Calculate variance in quality scores.
    fn calculate_quality_variance(&self, results: &[ModelResult]) -> f64 {
        let qualities: Vec<f64> = results
            .iter()
            .filter(|r| r.success)
            .map(|r| self.calculate_quality_score(r))
            .collect();

        if qualities.len() < 2 {
            return 0.0;
        }

        let mean = qualities.iter().sum::<f64>() / qualities.len() as f64;
        let variance = qualities.iter().map(|q| (q - mean).powi(2)).sum::<f64>() / qualities.len() as f64;

        variance
    }

    /// Generate recommendations based on rankings.
    fn generate_recommendations(&self, rankings: &[ModelRanking]) -> Vec<Recommendation> {
        let mut recommendations = Vec::new();

        if rankings.is_empty() {
            return recommendations;
        }

        // Best overall
        let best_overall = &rankings[0];
        recommendations.push(Recommendation {
            recommendation_type: RecommendationType::BestOverall,
            model: best_overall.model_config.identifier(),
            reasoning: format!(
                "Highest overall score ({:.2}) with strengths in: {}",
                best_overall.overall_score,
                best_overall.strengths.join(", ")
            ),
            confidence: 0.9,
        });

        // Best for quality
        if let Some(best_quality) = rankings
            .iter()
            .max_by(|a, b| {
                a.component_scores
                    .quality
                    .partial_cmp(&b.component_scores.quality)
                    .unwrap()
            })
        {
            recommendations.push(Recommendation {
                recommendation_type: RecommendationType::BestForQuality,
                model: best_quality.model_config.identifier(),
                reasoning: format!("Highest quality score ({:.2})", best_quality.component_scores.quality),
                confidence: 0.85,
            });
        }

        // Best for speed
        if let Some(best_performance) = rankings
            .iter()
            .max_by(|a, b| {
                a.component_scores
                    .performance
                    .partial_cmp(&b.component_scores.performance)
                    .unwrap()
            })
        {
            recommendations.push(Recommendation {
                recommendation_type: RecommendationType::BestForSpeed,
                model: best_performance.model_config.identifier(),
                reasoning: format!(
                    "Fastest response time (score: {:.2})",
                    best_performance.component_scores.performance
                ),
                confidence: 0.85,
            });
        }

        // Best for cost
        if let Some(best_cost) = rankings
            .iter()
            .max_by(|a, b| {
                a.component_scores
                    .cost_efficiency
                    .partial_cmp(&b.component_scores.cost_efficiency)
                    .unwrap()
            })
        {
            recommendations.push(Recommendation {
                recommendation_type: RecommendationType::BestForCost,
                model: best_cost.model_config.identifier(),
                reasoning: format!(
                    "Best cost efficiency (score: {:.2})",
                    best_cost.component_scores.cost_efficiency
                ),
                confidence: 0.85,
            });
        }

        recommendations
    }

    /// Run statistical significance tests on results.
    pub fn run_significance_tests(&self, results: &[ModelResult]) -> SignificanceTest {
        // Simple placeholder implementation
        // In production, this would use proper statistical tests (t-test, wilcoxon, etc.)

        let mut p_values = HashMap::new();
        let mut significant_differences = Vec::new();

        let successful_results: Vec<_> = results.iter().filter(|r| r.success).collect();

        // Compare all pairs
        for i in 0..successful_results.len() {
            for j in (i + 1)..successful_results.len() {
                let model_a = successful_results[i];
                let model_b = successful_results[j];

                let quality_a = self.calculate_quality_score(model_a);
                let quality_b = self.calculate_quality_score(model_b);

                let diff = (quality_a - quality_b).abs();
                let effect_size = diff;

                // Placeholder p-value calculation
                // In reality, would need multiple samples to calculate properly
                let p_value = if diff > 0.1 {
                    0.05 // Significant
                } else {
                    0.5 // Not significant
                };

                let pair_key = format!(
                    "{} vs {}",
                    model_a.model_config.identifier(),
                    model_b.model_config.identifier()
                );
                p_values.insert(pair_key, p_value);

                if p_value < 0.05 {
                    significant_differences.push(SignificantDifference {
                        model_a: model_a.model_config.identifier(),
                        model_b: model_b.model_config.identifier(),
                        p_value,
                        effect_size,
                        better_model: if quality_a > quality_b {
                            model_a.model_config.identifier()
                        } else {
                            model_b.model_config.identifier()
                        },
                    });
                }
            }
        }

        SignificanceTest {
            test_method: "placeholder".to_string(),
            p_values,
            significant_differences,
        }
    }
}

impl Default for RankingEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::orchestration::types::ModelConfig;
    use std::collections::HashMap;

    fn create_test_result(
        provider: &str,
        model: &str,
        quality: f64,
        latency_ms: u64,
        cost: f64,
    ) -> ModelResult {
        let mut scores = HashMap::new();
        scores.insert("faithfulness".to_string(), quality);
        scores.insert("relevance".to_string(), quality);
        scores.insert("coherence".to_string(), quality);

        ModelResult {
            model_config: ModelConfig::new(provider, model),
            success: true,
            response: None,
            evaluation_scores: scores,
            latency_ms,
            cost,
            error: None,
        }
    }

    #[test]
    fn test_ranking_basic() {
        let engine = RankingEngine::new();

        let results = vec![
            create_test_result("provider1", "model1", 0.9, 1000, 0.01),
            create_test_result("provider2", "model2", 0.8, 500, 0.02),
            create_test_result("provider3", "model3", 0.7, 2000, 0.005),
        ];

        let rankings = engine.calculate_rankings(&results).unwrap();

        assert_eq!(rankings.len(), 3);
        assert_eq!(rankings[0].rank, 1);
        assert_eq!(rankings[1].rank, 2);
        assert_eq!(rankings[2].rank, 3);

        // Model 1 should rank first due to highest quality (weighted 60%)
        assert_eq!(rankings[0].model_config.model, "model1");
    }

    #[test]
    fn test_ranking_weights() {
        let weights = RankingWeights {
            quality: 0.5,
            performance: 0.3,
            cost_efficiency: 0.2,
        };

        assert!(weights.validate().is_ok());

        let invalid_weights = RankingWeights {
            quality: 0.5,
            performance: 0.3,
            cost_efficiency: 0.3, // Sum > 1.0
        };

        assert!(invalid_weights.validate().is_err());
    }

    #[test]
    fn test_component_scores() {
        let engine = RankingEngine::new();

        let result = create_test_result("provider", "model", 0.85, 1000, 0.01);
        let all_results = vec![&result];

        let scores = engine.calculate_component_scores(&result, &all_results);

        assert_eq!(scores.quality, 0.85);
        assert_eq!(scores.performance, 1.0); // Only one model, so normalized to 1.0
        assert!(scores.cost_efficiency > 0.0);
    }

    #[test]
    fn test_identify_strengths() {
        let engine = RankingEngine::new();

        let high_quality = create_test_result("provider", "model", 0.95, 1000, 0.01);
        let average = create_test_result("provider", "model", 0.75, 1000, 0.01);

        let all_results = vec![&high_quality, &average];

        let strengths = engine.identify_strengths(&high_quality, &all_results);

        assert!(!strengths.is_empty());
        assert!(strengths.iter().any(|s| s.contains("quality")));
    }

    #[test]
    fn test_comparative_analysis() {
        let engine = RankingEngine::new();

        let results = vec![
            create_test_result("provider1", "model1", 0.9, 1000, 0.01),
            create_test_result("provider2", "model2", 0.8, 500, 0.02),
        ];

        let rankings = engine.calculate_rankings(&results).unwrap();
        let analysis = engine.generate_comparative_analysis(&results, &rankings);

        assert!(!analysis.summary.is_empty());
        assert!(!analysis.key_findings.is_empty());
        assert!(!analysis.recommendations.is_empty());
        assert_eq!(analysis.model_comparison_matrix.len(), 2);
    }

    #[test]
    fn test_no_successful_results() {
        let engine = RankingEngine::new();

        let failed_result = ModelResult {
            model_config: ModelConfig::new("provider", "model"),
            success: false,
            response: None,
            evaluation_scores: HashMap::new(),
            latency_ms: 0,
            cost: 0.0,
            error: Some("Failed".to_string()),
        };

        let result = engine.calculate_rankings(&[failed_result]);

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), RankingError::NoSuccessfulResults));
    }

    #[test]
    fn test_recommendations() {
        let engine = RankingEngine::new();

        let results = vec![
            create_test_result("provider1", "model1", 0.9, 1000, 0.01),
            create_test_result("provider2", "model2", 0.8, 500, 0.02),
            create_test_result("provider3", "model3", 0.7, 2000, 0.005),
        ];

        let rankings = engine.calculate_rankings(&results).unwrap();
        let recommendations = engine.generate_recommendations(&rankings);

        assert!(!recommendations.is_empty());

        // Should have recommendations for different types
        let rec_types: Vec<_> = recommendations
            .iter()
            .map(|r| r.recommendation_type)
            .collect();

        assert!(rec_types.contains(&RecommendationType::BestOverall));
        assert!(rec_types.contains(&RecommendationType::BestForQuality));
        assert!(rec_types.contains(&RecommendationType::BestForSpeed));
        assert!(rec_types.contains(&RecommendationType::BestForCost));
    }

    #[test]
    fn test_significance_tests() {
        let engine = RankingEngine::new();

        let results = vec![
            create_test_result("provider1", "model1", 0.9, 1000, 0.01),
            create_test_result("provider2", "model2", 0.7, 500, 0.02),
        ];

        let significance = engine.run_significance_tests(&results);

        assert!(!significance.p_values.is_empty());
        assert!(!significance.significant_differences.is_empty());
    }
}

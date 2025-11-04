// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Intelligent model routing and selection.
//!
//! This module provides functionality for selecting the most appropriate model
//! for a given task based on historical performance, constraints, and routing strategy.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

use serde::{Deserialize, Serialize};

use super::types::{
    ModelAlternative, ModelConfig, ModelConstraints, ModelProfile, ModelResult, ModelSelection,
    RoutingStrategy, TaskType,
};

/// Errors that can occur during routing.
#[derive(Error, Debug)]
pub enum RoutingError {
    /// No models available
    #[error("No models available for routing")]
    NoModelsAvailable,

    /// No models meet constraints
    #[error("No models meet the specified constraints")]
    NoModelsMeetConstraints,

    /// Invalid routing strategy
    #[error("Invalid routing strategy: {0}")]
    InvalidStrategy(String),

    /// Profile load/save error
    #[error("Profile I/O error: {0}")]
    ProfileIOError(String),

    /// Invalid profile
    #[error("Invalid profile: {0}")]
    InvalidProfile(String),
}

/// Model router for intelligent selection.
pub struct ModelRouter {
    /// Routing strategy to use
    routing_strategy: RoutingStrategy,

    /// Model profiles by model identifier (provider/model)
    model_profiles: HashMap<String, ModelProfile>,

    /// Path to profiles storage
    profiles_path: Option<PathBuf>,

    /// Auto-update profiles from results
    auto_update: bool,

    /// Weight for exponential moving average updates (0.0 - 1.0)
    update_weight: f64,
}

impl ModelRouter {
    /// Create a new model router with the specified strategy.
    pub fn new(routing_strategy: RoutingStrategy) -> Self {
        Self {
            routing_strategy,
            model_profiles: HashMap::new(),
            profiles_path: None,
            auto_update: true,
            update_weight: 0.1, // 10% weight for new results
        }
    }

    /// Set the path for storing model profiles.
    pub fn with_profiles_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.profiles_path = Some(path.into());
        self
    }

    /// Enable or disable auto-update of profiles.
    pub fn with_auto_update(mut self, auto_update: bool) -> Self {
        self.auto_update = auto_update;
        self
    }

    /// Set the update weight for exponential moving average.
    pub fn with_update_weight(mut self, weight: f64) -> Self {
        self.update_weight = weight.clamp(0.0, 1.0);
        self
    }

    /// Register a model profile.
    pub fn register_profile(&mut self, profile: ModelProfile) {
        self.model_profiles.insert(profile.name.clone(), profile);
    }

    /// Get a model profile by identifier.
    pub fn get_profile(&self, model_id: &str) -> Option<&ModelProfile> {
        self.model_profiles.get(model_id)
    }

    /// Load profiles from disk.
    pub fn load_profiles(&mut self) -> Result<(), RoutingError> {
        let path = self
            .profiles_path
            .as_ref()
            .ok_or_else(|| RoutingError::ProfileIOError("No profiles path set".to_string()))?;

        if !path.exists() {
            return Ok(()); // No profiles to load yet
        }

        let content = fs::read_to_string(path)
            .map_err(|e| RoutingError::ProfileIOError(format!("Failed to read profiles: {}", e)))?;

        let profiles: HashMap<String, ModelProfile> = serde_json::from_str(&content)
            .map_err(|e| RoutingError::ProfileIOError(format!("Failed to parse profiles: {}", e)))?;

        self.model_profiles = profiles;

        Ok(())
    }

    /// Save profiles to disk.
    pub fn save_profiles(&self) -> Result<(), RoutingError> {
        let path = self
            .profiles_path
            .as_ref()
            .ok_or_else(|| RoutingError::ProfileIOError("No profiles path set".to_string()))?;

        // Create parent directory if it doesn't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                RoutingError::ProfileIOError(format!("Failed to create directory: {}", e))
            })?;
        }

        let content = serde_json::to_string_pretty(&self.model_profiles)
            .map_err(|e| RoutingError::ProfileIOError(format!("Failed to serialize profiles: {}", e)))?;

        fs::write(path, content)
            .map_err(|e| RoutingError::ProfileIOError(format!("Failed to write profiles: {}", e)))?;

        Ok(())
    }

    /// Update profiles from benchmark results.
    pub fn update_from_results(&mut self, results: &[ModelResult]) {
        for result in results {
            if !result.success {
                continue;
            }

            let model_id = result.model_config.identifier();

            let profile = self
                .model_profiles
                .entry(model_id.clone())
                .or_insert_with(|| ModelProfile::new(model_id));

            profile.update_from_result(result, self.update_weight);
        }

        // Auto-save if enabled and path is set
        if self.auto_update && self.profiles_path.is_some() {
            if let Err(e) = self.save_profiles() {
                tracing::warn!("Failed to save profiles: {}", e);
            }
        }
    }

    /// Select the best model for a given prompt and constraints.
    pub fn select_model(
        &self,
        prompt: &str,
        available_models: &[ModelConfig],
        constraints: &ModelConstraints,
    ) -> Result<ModelSelection, RoutingError> {
        if available_models.is_empty() {
            return Err(RoutingError::NoModelsAvailable);
        }

        // Classify the task
        let task_type = TaskType::classify_prompt(prompt);

        // Filter models that meet constraints
        let viable_models: Vec<_> = available_models
            .iter()
            .filter(|model| self.meets_constraints(model, constraints))
            .collect();

        if viable_models.is_empty() {
            return Err(RoutingError::NoModelsMeetConstraints);
        }

        // Score each model according to strategy
        let mut scored_models: Vec<(ModelConfig, f64, String)> = viable_models
            .iter()
            .map(|model| {
                let (score, reasoning) = self.score_model(model, task_type, constraints);
                ((*model).clone(), score, reasoning)
            })
            .collect();

        // Sort by score (descending)
        scored_models.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Select the best model
        let (selected, score, reasoning) = scored_models.first().unwrap();
        let profile = self.get_profile(&selected.identifier());

        // Build alternatives list
        let alternatives: Vec<ModelAlternative> = scored_models
            .iter()
            .skip(1)
            .take(3) // Top 3 alternatives
            .map(|(model, alt_score, _)| ModelAlternative {
                model_config: model.clone(),
                reason: format!("Lower score: {:.2} vs {:.2}", alt_score, score),
                score: *alt_score,
            })
            .collect();

        Ok(ModelSelection {
            model_config: selected.clone(),
            reasoning: reasoning.clone(),
            expected_quality: profile.map(|p| p.typical_quality).unwrap_or(0.7),
            expected_latency_ms: profile.map(|p| p.avg_latency_ms).unwrap_or(1000),
            expected_cost: profile.map(|p| p.cost_per_1k_tokens * 0.1).unwrap_or(0.001),
            alternatives,
        })
    }

    /// Check if a model meets the specified constraints.
    fn meets_constraints(&self, model: &ModelConfig, constraints: &ModelConstraints) -> bool {
        let profile = self.get_profile(&model.identifier());

        // Check quality constraint
        if let Some(profile) = profile {
            if profile.typical_quality < constraints.min_quality {
                return false;
            }

            // Check latency constraint
            if let Some(max_latency) = constraints.max_latency_ms {
                if profile.avg_latency_ms > max_latency {
                    return false;
                }
            }

            // Check context length constraint
            if let Some(min_context) = constraints.min_context_length {
                if profile.context_limit < min_context {
                    return false;
                }
            }

            // Check cost constraint (approximate)
            if let Some(max_cost) = constraints.max_cost {
                let estimated_cost = profile.cost_per_1k_tokens * 0.1; // Assume ~100 tokens
                if estimated_cost > max_cost {
                    return false;
                }
            }
        } else {
            // No profile available - be permissive and allow it
            // In production, might want to be more conservative
        }

        true
    }

    /// Score a model according to the routing strategy.
    fn score_model(
        &self,
        model: &ModelConfig,
        task_type: TaskType,
        constraints: &ModelConstraints,
    ) -> (f64, String) {
        let profile = self.get_profile(&model.identifier());

        match self.routing_strategy {
            RoutingStrategy::Quality => self.score_for_quality(model, profile),
            RoutingStrategy::CostOptimized => self.score_for_cost(model, profile, constraints),
            RoutingStrategy::Latency => self.score_for_latency(model, profile),
            RoutingStrategy::Balanced => self.score_balanced(model, profile, task_type),
        }
    }

    /// Score model for quality-focused routing.
    fn score_for_quality(&self, model: &ModelConfig, profile: Option<&ModelProfile>) -> (f64, String) {
        let quality = profile.map(|p| p.typical_quality).unwrap_or(0.7);

        (
            quality,
            format!("Selected for quality (score: {:.2})", quality),
        )
    }

    /// Score model for cost-optimized routing.
    fn score_for_cost(
        &self,
        model: &ModelConfig,
        profile: Option<&ModelProfile>,
        constraints: &ModelConstraints,
    ) -> (f64, String) {
        if let Some(profile) = profile {
            let quality = profile.typical_quality;

            // Cost efficiency = quality / cost
            let cost = profile.cost_per_1k_tokens.max(0.0001); // Avoid division by zero
            let efficiency = quality / cost;

            // Ensure minimum quality threshold is met
            if quality < constraints.min_quality {
                return (0.0, "Does not meet quality threshold".to_string());
            }

            (
                efficiency,
                format!(
                    "Selected for cost efficiency (quality: {:.2}, cost: ${:.4}/1k)",
                    quality, cost
                ),
            )
        } else {
            (0.5, "No profile available".to_string())
        }
    }

    /// Score model for latency-focused routing.
    fn score_for_latency(&self, model: &ModelConfig, profile: Option<&ModelProfile>) -> (f64, String) {
        if let Some(profile) = profile {
            let latency = profile.avg_latency_ms.max(1) as f64;

            // Score = 1 / (latency_seconds)
            let score = 1000.0 / latency;

            (
                score,
                format!("Selected for speed (latency: {}ms)", profile.avg_latency_ms),
            )
        } else {
            (0.5, "No profile available".to_string())
        }
    }

    /// Score model for balanced routing.
    fn score_balanced(
        &self,
        model: &ModelConfig,
        profile: Option<&ModelProfile>,
        task_type: TaskType,
    ) -> (f64, String) {
        if let Some(profile) = profile {
            let quality = profile.typical_quality;
            let latency_score = 1000.0 / profile.avg_latency_ms.max(1) as f64;
            let cost_efficiency = quality / profile.cost_per_1k_tokens.max(0.0001);

            // Task-specific boost
            let task_boost = if profile.strengths.contains(&task_type) {
                1.2
            } else {
                1.0
            };

            // Balanced score: 50% quality, 25% speed, 25% cost
            let score = (quality * 0.5 + latency_score * 0.25 + cost_efficiency * 0.25) * task_boost;

            (
                score,
                format!(
                    "Selected for balanced performance (quality: {:.2}, speed: {}ms, cost: ${:.4}/1k)",
                    quality, profile.avg_latency_ms, profile.cost_per_1k_tokens
                ),
            )
        } else {
            (0.5, "No profile available".to_string())
        }
    }

    /// Get the number of registered profiles.
    pub fn profile_count(&self) -> usize {
        self.model_profiles.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use tempfile::TempDir;

    fn create_test_profile(name: &str, quality: f64, latency_ms: u64, cost: f64) -> ModelProfile {
        ModelProfile {
            name: name.to_string(),
            typical_quality: quality,
            avg_latency_ms: latency_ms,
            cost_per_1k_tokens: cost,
            context_limit: 4096,
            strengths: vec![TaskType::General],
            sample_count: 10,
        }
    }

    #[test]
    fn test_router_creation() {
        let router = ModelRouter::new(RoutingStrategy::Quality);
        assert_eq!(router.profile_count(), 0);
    }

    #[test]
    fn test_register_profile() {
        let mut router = ModelRouter::new(RoutingStrategy::Quality);
        let profile = create_test_profile("openai/gpt-4", 0.9, 1000, 0.03);

        router.register_profile(profile);

        assert_eq!(router.profile_count(), 1);
        assert!(router.get_profile("openai/gpt-4").is_some());
    }

    #[test]
    fn test_select_model_quality() {
        let mut router = ModelRouter::new(RoutingStrategy::Quality);

        router.register_profile(create_test_profile("provider1/model1", 0.9, 1000, 0.03));
        router.register_profile(create_test_profile("provider2/model2", 0.8, 500, 0.01));

        let available_models = vec![
            ModelConfig::new("provider1", "model1"),
            ModelConfig::new("provider2", "model2"),
        ];

        let constraints = ModelConstraints::default();
        let selection = router
            .select_model("Test prompt", &available_models, &constraints)
            .unwrap();

        // Should select model1 due to higher quality
        assert_eq!(selection.model_config.model, "model1");
    }

    #[test]
    fn test_select_model_latency() {
        let mut router = ModelRouter::new(RoutingStrategy::Latency);

        router.register_profile(create_test_profile("provider1/model1", 0.9, 1000, 0.03));
        router.register_profile(create_test_profile("provider2/model2", 0.8, 500, 0.01));

        let available_models = vec![
            ModelConfig::new("provider1", "model1"),
            ModelConfig::new("provider2", "model2"),
        ];

        let constraints = ModelConstraints::default();
        let selection = router
            .select_model("Test prompt", &available_models, &constraints)
            .unwrap();

        // Should select model2 due to lower latency
        assert_eq!(selection.model_config.model, "model2");
    }

    #[test]
    fn test_select_model_cost() {
        let mut router = ModelRouter::new(RoutingStrategy::CostOptimized);

        router.register_profile(create_test_profile("provider1/model1", 0.9, 1000, 0.03));
        router.register_profile(create_test_profile("provider2/model2", 0.8, 500, 0.01));

        let available_models = vec![
            ModelConfig::new("provider1", "model1"),
            ModelConfig::new("provider2", "model2"),
        ];

        let constraints = ModelConstraints::default();
        let selection = router
            .select_model("Test prompt", &available_models, &constraints)
            .unwrap();

        // Should select model2 due to better cost efficiency (0.8/0.01 > 0.9/0.03)
        assert_eq!(selection.model_config.model, "model2");
    }

    #[test]
    fn test_constraints_filtering() {
        let mut router = ModelRouter::new(RoutingStrategy::Quality);

        router.register_profile(create_test_profile("provider1/model1", 0.9, 1000, 0.03));
        router.register_profile(create_test_profile("provider2/model2", 0.6, 500, 0.01));

        let available_models = vec![
            ModelConfig::new("provider1", "model1"),
            ModelConfig::new("provider2", "model2"),
        ];

        let constraints = ModelConstraints::new().with_min_quality(0.75);

        let selection = router
            .select_model("Test prompt", &available_models, &constraints)
            .unwrap();

        // Should select model1 as model2 doesn't meet quality threshold
        assert_eq!(selection.model_config.model, "model1");
    }

    #[test]
    fn test_no_models_meet_constraints() {
        let mut router = ModelRouter::new(RoutingStrategy::Quality);

        router.register_profile(create_test_profile("provider1/model1", 0.7, 1000, 0.03));

        let available_models = vec![ModelConfig::new("provider1", "model1")];

        let constraints = ModelConstraints::new().with_min_quality(0.95);

        let result = router.select_model("Test prompt", &available_models, &constraints);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            RoutingError::NoModelsMeetConstraints
        ));
    }

    #[test]
    fn test_no_models_available() {
        let router = ModelRouter::new(RoutingStrategy::Quality);
        let available_models = vec![];
        let constraints = ModelConstraints::default();

        let result = router.select_model("Test prompt", &available_models, &constraints);

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), RoutingError::NoModelsAvailable));
    }

    #[test]
    fn test_update_from_results() {
        let mut router = ModelRouter::new(RoutingStrategy::Quality);

        let mut scores = HashMap::new();
        scores.insert("quality".to_string(), 0.85);

        let result = ModelResult {
            model_config: ModelConfig::new("provider", "model"),
            success: true,
            response: None,
            evaluation_scores: scores,
            latency_ms: 1200,
            cost: 0.02,
            error: None,
        };

        router.update_from_results(&[result]);

        assert_eq!(router.profile_count(), 1);

        let profile = router.get_profile("provider/model").unwrap();
        assert_eq!(profile.sample_count, 1);
    }

    #[test]
    fn test_save_and_load_profiles() {
        let temp_dir = TempDir::new().unwrap();
        let profiles_path = temp_dir.path().join("profiles.json");

        let mut router = ModelRouter::new(RoutingStrategy::Quality)
            .with_profiles_path(profiles_path.clone());

        router.register_profile(create_test_profile("provider/model", 0.9, 1000, 0.03));

        // Save profiles
        router.save_profiles().unwrap();

        // Create new router and load profiles
        let mut router2 = ModelRouter::new(RoutingStrategy::Quality)
            .with_profiles_path(profiles_path);

        router2.load_profiles().unwrap();

        assert_eq!(router2.profile_count(), 1);
        assert!(router2.get_profile("provider/model").is_some());
    }

    #[test]
    fn test_alternatives() {
        let mut router = ModelRouter::new(RoutingStrategy::Quality);

        router.register_profile(create_test_profile("provider1/model1", 0.9, 1000, 0.03));
        router.register_profile(create_test_profile("provider2/model2", 0.8, 500, 0.01));
        router.register_profile(create_test_profile("provider3/model3", 0.7, 2000, 0.005));

        let available_models = vec![
            ModelConfig::new("provider1", "model1"),
            ModelConfig::new("provider2", "model2"),
            ModelConfig::new("provider3", "model3"),
        ];

        let constraints = ModelConstraints::default();
        let selection = router
            .select_model("Test prompt", &available_models, &constraints)
            .unwrap();

        assert!(!selection.alternatives.is_empty());
        assert!(selection.alternatives.len() <= 3);
    }

    #[test]
    fn test_balanced_strategy() {
        let mut router = ModelRouter::new(RoutingStrategy::Balanced);

        router.register_profile(create_test_profile("provider1/model1", 0.95, 2000, 0.05));
        router.register_profile(create_test_profile("provider2/model2", 0.85, 800, 0.02));

        let available_models = vec![
            ModelConfig::new("provider1", "model1"),
            ModelConfig::new("provider2", "model2"),
        ];

        let constraints = ModelConstraints::default();
        let selection = router
            .select_model("Test prompt", &available_models, &constraints)
            .unwrap();

        // model2 should win in balanced mode due to better overall balance
        assert_eq!(selection.model_config.model, "model2");
    }
}

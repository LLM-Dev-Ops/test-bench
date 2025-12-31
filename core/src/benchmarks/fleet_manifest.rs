// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Fleet manifest schema and parser.
//!
//! This module provides the schema and parsing logic for fleet manifests,
//! which define how to orchestrate benchmarks across multiple repositories.
//!
//! # Fleet Manifest Format
//!
//! A fleet manifest is a JSON or YAML file that specifies:
//! - Which repositories to benchmark
//! - Which providers/models to use
//! - Scenario configurations per repository
//! - Output format and location
//!
//! # Example Manifest
//!
//! ```json
//! {
//!   "fleet_id": "agentics-fleet-2025",
//!   "version": "1.0",
//!   "description": "Full Agentics system benchmark",
//!   "repositories": [
//!     {
//!       "repo_id": "test-bench",
//!       "path": ".",
//!       "adapter": "native",
//!       "scenarios": ["coding", "reasoning"]
//!     }
//!   ],
//!   "providers": ["openai:gpt-4", "anthropic:claude-3-opus"],
//!   "scenario_profiles": {
//!     "coding": {
//!       "dataset": "coding-tasks.json",
//!       "concurrency": 5,
//!       "num_examples": 100
//!     }
//!   },
//!   "output": {
//!     "base_dir": "./fleet-results",
//!     "formats": ["json", "csv", "html"]
//!   }
//! }
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Fleet manifest errors
#[derive(Error, Debug)]
pub enum FleetManifestError {
    /// Failed to read manifest file
    #[error("Failed to read manifest: {0}")]
    IoError(#[from] std::io::Error),

    /// Failed to parse JSON
    #[error("Failed to parse JSON manifest: {0}")]
    JsonParseError(#[from] serde_json::Error),

    /// Failed to parse YAML
    #[error("Failed to parse YAML manifest: {0}")]
    YamlParseError(#[from] serde_yaml::Error),

    /// Validation error
    #[error("Manifest validation failed: {0}")]
    ValidationError(String),

    /// Invalid manifest version
    #[error("Unsupported manifest version: {0}")]
    UnsupportedVersion(String),
}

/// Fleet manifest defining multi-repository benchmark orchestration.
///
/// This is the top-level structure that coordinates benchmarking across
/// multiple repositories with different adapters and configurations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FleetManifest {
    /// Unique identifier for this fleet
    pub fleet_id: String,

    /// Manifest schema version (currently "1.0")
    pub version: String,

    /// Human-readable description of this fleet
    #[serde(default)]
    pub description: String,

    /// List of repositories to benchmark
    pub repositories: Vec<RepositoryConfig>,

    /// Provider specifications (format: "provider:model")
    pub providers: Vec<String>,

    /// Scenario configurations
    #[serde(default)]
    pub scenario_profiles: HashMap<String, ScenarioProfile>,

    /// Output configuration
    pub output: OutputConfig,

    /// Optional global settings
    #[serde(default)]
    pub global_settings: GlobalSettings,
}

impl FleetManifest {
    /// Loads a fleet manifest from a file.
    ///
    /// Automatically detects JSON or YAML format based on file extension.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the manifest file (.json, .yaml, or .yml)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use llm_test_bench_core::benchmarks::fleet_manifest::FleetManifest;
    /// use std::path::Path;
    ///
    /// let manifest = FleetManifest::load_from_file(Path::new("fleet.json")).unwrap();
    /// println!("Fleet: {}", manifest.fleet_id);
    /// ```
    pub fn load_from_file(path: &Path) -> Result<Self, FleetManifestError> {
        let content = std::fs::read_to_string(path)?;

        // Auto-detect format by extension
        let manifest = match path.extension().and_then(|s| s.to_str()) {
            Some("yaml") | Some("yml") => serde_yaml::from_str::<FleetManifest>(&content)?,
            Some("json") | _ => serde_json::from_str::<FleetManifest>(&content)?,
        };

        // Validate the manifest
        manifest.validate()?;

        Ok(manifest)
    }

    /// Loads a fleet manifest from a JSON string.
    pub fn from_json(json: &str) -> Result<Self, FleetManifestError> {
        let manifest: FleetManifest = serde_json::from_str(json)?;
        manifest.validate()?;
        Ok(manifest)
    }

    /// Loads a fleet manifest from a YAML string.
    pub fn from_yaml(yaml: &str) -> Result<Self, FleetManifestError> {
        let manifest: FleetManifest = serde_yaml::from_str(yaml)?;
        manifest.validate()?;
        Ok(manifest)
    }

    /// Validates the fleet manifest.
    ///
    /// Checks for:
    /// - Supported version
    /// - Non-empty fleet_id
    /// - At least one repository
    /// - At least one provider
    /// - Valid scenario references
    pub fn validate(&self) -> Result<(), FleetManifestError> {
        // Check version
        if self.version != "1.0" {
            return Err(FleetManifestError::UnsupportedVersion(
                self.version.clone(),
            ));
        }

        // Check fleet_id
        if self.fleet_id.trim().is_empty() {
            return Err(FleetManifestError::ValidationError(
                "fleet_id cannot be empty".to_string(),
            ));
        }

        // Check repositories
        if self.repositories.is_empty() {
            return Err(FleetManifestError::ValidationError(
                "At least one repository is required".to_string(),
            ));
        }

        // Check providers
        if self.providers.is_empty() {
            return Err(FleetManifestError::ValidationError(
                "At least one provider is required".to_string(),
            ));
        }

        // Validate each repository
        for repo in &self.repositories {
            repo.validate(&self.scenario_profiles)?;
        }

        // Validate provider format (provider:model)
        for provider in &self.providers {
            if !provider.contains(':') {
                return Err(FleetManifestError::ValidationError(format!(
                    "Provider must be in 'provider:model' format, got: {}",
                    provider
                )));
            }
        }

        Ok(())
    }

    /// Saves the manifest to a JSON file.
    pub fn save_to_json(&self, path: &Path) -> Result<(), FleetManifestError> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Saves the manifest to a YAML file.
    pub fn save_to_yaml(&self, path: &Path) -> Result<(), FleetManifestError> {
        let content = serde_yaml::to_string(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Parses provider string into (provider_name, model_name).
    ///
    /// # Examples
    ///
    /// ```
    /// use llm_test_bench_core::benchmarks::fleet_manifest::FleetManifest;
    ///
    /// let (provider, model) = FleetManifest::parse_provider("openai:gpt-4");
    /// assert_eq!(provider, "openai");
    /// assert_eq!(model, "gpt-4");
    /// ```
    pub fn parse_provider(provider_spec: &str) -> (&str, &str) {
        let parts: Vec<&str> = provider_spec.splitn(2, ':').collect();
        if parts.len() == 2 {
            (parts[0], parts[1])
        } else {
            (provider_spec, "")
        }
    }
}

/// Configuration for a single repository in the fleet.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryConfig {
    /// Unique identifier for this repository
    pub repo_id: String,

    /// Path to the repository (relative or absolute)
    #[serde(default)]
    pub path: PathBuf,

    /// Optional git URL for external repositories
    #[serde(default)]
    pub git_url: Option<String>,

    /// Adapter type to use for this repository
    pub adapter: String,

    /// List of scenarios to run for this repository
    pub scenarios: Vec<String>,

    /// Repository-specific metadata
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

impl RepositoryConfig {
    /// Validates the repository configuration.
    pub fn validate(
        &self,
        scenario_profiles: &HashMap<String, ScenarioProfile>,
    ) -> Result<(), FleetManifestError> {
        // Check repo_id
        if self.repo_id.trim().is_empty() {
            return Err(FleetManifestError::ValidationError(
                "repo_id cannot be empty".to_string(),
            ));
        }

        // Check scenarios
        if self.scenarios.is_empty() {
            return Err(FleetManifestError::ValidationError(format!(
                "Repository '{}' must have at least one scenario",
                self.repo_id
            )));
        }

        // Check that all scenarios exist in profiles
        for scenario in &self.scenarios {
            if !scenario_profiles.contains_key(scenario) {
                return Err(FleetManifestError::ValidationError(format!(
                    "Scenario '{}' referenced by repository '{}' not found in scenario_profiles",
                    scenario, self.repo_id
                )));
            }
        }

        Ok(())
    }

    /// Returns true if this is an external repository (has git_url).
    pub fn is_external(&self) -> bool {
        self.git_url.is_some()
    }

    /// Returns the effective path for this repository.
    pub fn effective_path(&self) -> &Path {
        &self.path
    }
}

/// Scenario profile defining execution parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioProfile {
    /// Dataset file path (relative to repository)
    pub dataset: String,

    /// Concurrency level for this scenario
    #[serde(default = "default_concurrency")]
    pub concurrency: usize,

    /// Number of examples to run (None = all)
    #[serde(default)]
    pub num_examples: Option<usize>,

    /// Request delay in milliseconds
    #[serde(default)]
    pub request_delay_ms: Option<u64>,

    /// Additional scenario-specific settings
    #[serde(default)]
    pub settings: HashMap<String, serde_json::Value>,
}

fn default_concurrency() -> usize {
    5
}

/// Output configuration for fleet results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    /// Base directory for all output
    pub base_dir: PathBuf,

    /// Output formats to generate (json, csv, html)
    #[serde(default = "default_formats")]
    pub formats: Vec<String>,

    /// Whether to save individual responses
    #[serde(default = "default_true")]
    pub save_responses: bool,

    /// Whether to generate aggregate reports
    #[serde(default = "default_true")]
    pub generate_reports: bool,
}

fn default_formats() -> Vec<String> {
    vec!["json".to_string(), "csv".to_string()]
}

fn default_true() -> bool {
    true
}

/// Global settings for fleet execution.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GlobalSettings {
    /// Continue on failure across repositories
    #[serde(default = "default_true")]
    pub continue_on_failure: bool,

    /// Random seed for reproducibility
    #[serde(default)]
    pub random_seed: Option<u64>,

    /// Timeout for individual tests (seconds)
    #[serde(default)]
    pub test_timeout_seconds: Option<u64>,

    /// Maximum retries for failed tests
    #[serde(default)]
    pub max_retries: Option<usize>,

    /// Custom settings
    #[serde(default)]
    pub custom: HashMap<String, serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn create_test_manifest() -> FleetManifest {
        let mut scenario_profiles = HashMap::new();
        scenario_profiles.insert(
            "coding".to_string(),
            ScenarioProfile {
                dataset: "coding-tasks.json".to_string(),
                concurrency: 5,
                num_examples: Some(100),
                request_delay_ms: None,
                settings: HashMap::new(),
            },
        );

        FleetManifest {
            fleet_id: "test-fleet".to_string(),
            version: "1.0".to_string(),
            description: "Test fleet".to_string(),
            repositories: vec![RepositoryConfig {
                repo_id: "test-bench".to_string(),
                path: PathBuf::from("."),
                git_url: None,
                adapter: "native".to_string(),
                scenarios: vec!["coding".to_string()],
                metadata: HashMap::new(),
            }],
            providers: vec!["openai:gpt-4".to_string()],
            scenario_profiles,
            output: OutputConfig {
                base_dir: PathBuf::from("./fleet-results"),
                formats: vec!["json".to_string()],
                save_responses: true,
                generate_reports: true,
            },
            global_settings: GlobalSettings::default(),
        }
    }

    #[test]
    fn test_manifest_validation_success() {
        let manifest = create_test_manifest();
        assert!(manifest.validate().is_ok());
    }

    #[test]
    fn test_manifest_validation_empty_fleet_id() {
        let mut manifest = create_test_manifest();
        manifest.fleet_id = "".to_string();
        assert!(manifest.validate().is_err());
    }

    #[test]
    fn test_manifest_validation_no_repositories() {
        let mut manifest = create_test_manifest();
        manifest.repositories.clear();
        assert!(manifest.validate().is_err());
    }

    #[test]
    fn test_manifest_validation_no_providers() {
        let mut manifest = create_test_manifest();
        manifest.providers.clear();
        assert!(manifest.validate().is_err());
    }

    #[test]
    fn test_manifest_validation_invalid_version() {
        let mut manifest = create_test_manifest();
        manifest.version = "2.0".to_string();
        assert!(manifest.validate().is_err());
    }

    #[test]
    fn test_manifest_validation_invalid_provider_format() {
        let mut manifest = create_test_manifest();
        manifest.providers = vec!["openai".to_string()]; // Missing model
        assert!(manifest.validate().is_err());
    }

    #[test]
    fn test_manifest_validation_missing_scenario() {
        let mut manifest = create_test_manifest();
        manifest.repositories[0].scenarios = vec!["nonexistent".to_string()];
        let result = manifest.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_provider() {
        let (provider, model) = FleetManifest::parse_provider("openai:gpt-4");
        assert_eq!(provider, "openai");
        assert_eq!(model, "gpt-4");

        let (provider, model) = FleetManifest::parse_provider("anthropic:claude-3-opus");
        assert_eq!(provider, "anthropic");
        assert_eq!(model, "claude-3-opus");
    }

    #[test]
    fn test_repository_config_validation() {
        let mut scenario_profiles = HashMap::new();
        scenario_profiles.insert(
            "coding".to_string(),
            ScenarioProfile {
                dataset: "test.json".to_string(),
                concurrency: 5,
                num_examples: None,
                request_delay_ms: None,
                settings: HashMap::new(),
            },
        );

        let config = RepositoryConfig {
            repo_id: "test".to_string(),
            path: PathBuf::from("."),
            git_url: None,
            adapter: "native".to_string(),
            scenarios: vec!["coding".to_string()],
            metadata: HashMap::new(),
        };

        assert!(config.validate(&scenario_profiles).is_ok());
    }

    #[test]
    fn test_repository_is_external() {
        let mut config = RepositoryConfig {
            repo_id: "test".to_string(),
            path: PathBuf::from("."),
            git_url: None,
            adapter: "native".to_string(),
            scenarios: vec!["coding".to_string()],
            metadata: HashMap::new(),
        };

        assert!(!config.is_external());

        config.git_url = Some("https://github.com/test/repo.git".to_string());
        assert!(config.is_external());
    }

    #[test]
    fn test_manifest_serialization_json() {
        let manifest = create_test_manifest();
        let json = serde_json::to_string(&manifest).unwrap();
        let deserialized: FleetManifest = serde_json::from_str(&json).unwrap();
        assert_eq!(manifest.fleet_id, deserialized.fleet_id);
    }

    #[test]
    fn test_manifest_serialization_yaml() {
        let manifest = create_test_manifest();
        let yaml = serde_yaml::to_string(&manifest).unwrap();
        let deserialized: FleetManifest = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(manifest.fleet_id, deserialized.fleet_id);
    }

    #[test]
    fn test_manifest_from_json() {
        let json = r#"{
            "fleet_id": "test",
            "version": "1.0",
            "repositories": [
                {
                    "repo_id": "r1",
                    "adapter": "native",
                    "scenarios": ["s1"]
                }
            ],
            "providers": ["openai:gpt-4"],
            "scenario_profiles": {
                "s1": {
                    "dataset": "test.json"
                }
            },
            "output": {
                "base_dir": "./results"
            }
        }"#;

        let manifest = FleetManifest::from_json(json).unwrap();
        assert_eq!(manifest.fleet_id, "test");
    }

    #[test]
    fn test_save_and_load_manifest() {
        let manifest = create_test_manifest();
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("fleet.json");

        // Save
        manifest.save_to_json(&file_path).unwrap();
        assert!(file_path.exists());

        // Load
        let loaded = FleetManifest::load_from_file(&file_path).unwrap();
        assert_eq!(manifest.fleet_id, loaded.fleet_id);
        assert_eq!(manifest.repositories.len(), loaded.repositories.len());
    }
}

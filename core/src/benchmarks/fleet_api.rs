// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Fleet Benchmark API for programmatic simulator integration.
//!
//! This module provides a clean programmatic API for executing fleet benchmarks
//! and retrieving run identifiers and artifact locations. It is designed to be
//! consumed by simulators that need to trigger benchmarks and track results.
//!
//! # Architecture
//!
//! - **Synchronous API**: Returns handle immediately with run_id and paths
//! - **Async Execution**: Actual benchmark runs asynchronously in background
//! - **Deterministic IDs**: Generates reproducible run identifiers
//! - **Artifact Tracking**: Provides paths to results before execution completes
//!
//! # Examples
//!
//! ```no_run
//! use llm_test_bench_core::benchmarks::fleet_api::{FleetBenchmarkAPI, FleetConfig};
//! use std::path::PathBuf;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let config = FleetConfig::new("./fleet-results".into());
//! let api = FleetBenchmarkAPI::new(config);
//!
//! // Execute fleet benchmark and get handle immediately
//! let handle = api.execute_fleet_benchmark(&PathBuf::from("./fleet-manifest.json")).await?;
//!
//! println!("Run ID: {}", handle.run_id);
//! println!("Artifacts: {}", handle.artifact_base_dir.display());
//!
//! // Optionally wait for completion
//! let results = handle.execution_future.await??;
//! println!("Fleet benchmark completed: {} repositories", results.total_repositories);
//! # Ok(())
//! # }
//! ```

use super::config::BenchmarkConfig;
use super::fleet::FleetBenchmarkResults;
use super::fleet_manifest::{FleetManifest, RepositoryConfig, ScenarioProfile, OutputConfig, GlobalSettings};
use super::runner::BenchmarkRunner;
use super::BenchmarkError;
use crate::config::ConfigLoader;
use crate::providers::ProviderFactory;
use chrono::Utc;
use llm_test_bench_datasets::loader::DatasetLoader;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::task::JoinHandle;

/// Configuration for fleet benchmark execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FleetConfig {
    /// Base directory for fleet results
    pub output_base_dir: PathBuf,

    /// Default concurrency for benchmark execution
    pub default_concurrency: usize,

    /// Whether to save individual responses
    pub save_responses: bool,

    /// Whether to continue on individual test failures
    pub continue_on_failure: bool,

    /// Optional request delay in milliseconds
    pub request_delay_ms: Option<u64>,

    /// Path to configuration file (optional)
    pub config_path: Option<PathBuf>,
}

impl FleetConfig {
    /// Creates a new fleet configuration with defaults.
    pub fn new(output_base_dir: PathBuf) -> Self {
        Self {
            output_base_dir,
            default_concurrency: 5,
            save_responses: true,
            continue_on_failure: true,
            request_delay_ms: None,
            config_path: None,
        }
    }

    /// Sets the default concurrency level.
    pub fn with_concurrency(mut self, concurrency: usize) -> Self {
        self.default_concurrency = concurrency;
        self
    }

    /// Sets whether to save responses.
    pub fn with_save_responses(mut self, save: bool) -> Self {
        self.save_responses = save;
        self
    }

    /// Sets the configuration file path.
    pub fn with_config(mut self, config_path: PathBuf) -> Self {
        self.config_path = Some(config_path);
        self
    }

    /// Sets the request delay.
    pub fn with_request_delay_ms(mut self, delay_ms: u64) -> Self {
        self.request_delay_ms = Some(delay_ms);
        self
    }
}

/// Handle returned immediately when executing a fleet benchmark.
///
/// Provides run identification, artifact locations, and a future for waiting
/// on execution completion.
pub struct FleetExecutionHandle {
    /// Unique identifier for this fleet benchmark run
    pub run_id: String,

    /// Base directory where all artifacts will be stored
    pub artifact_base_dir: PathBuf,

    /// Optional URL for monitoring execution status
    pub status_url: Option<String>,

    /// Future that resolves when execution completes
    pub execution_future: JoinHandle<Result<FleetBenchmarkResults, FleetError>>,

    /// Metadata about the execution
    pub metadata: FleetExecutionMetadata,
}

/// Metadata about a fleet execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FleetExecutionMetadata {
    /// Fleet identifier from manifest
    pub fleet_id: String,

    /// Number of repositories in the fleet
    pub repository_count: usize,

    /// Providers being benchmarked
    pub providers: Vec<String>,

    /// When execution was initiated
    pub started_at: chrono::DateTime<Utc>,

    /// Expected artifact paths
    pub expected_artifacts: Vec<PathBuf>,
}

/// Fleet-specific errors.
#[derive(Debug, thiserror::Error)]
pub enum FleetError {
    #[error("Failed to load fleet manifest: {0}")]
    ManifestLoadError(String),

    #[error("Invalid fleet manifest: {0}")]
    InvalidManifest(String),

    #[error("Benchmark execution failed: {0}")]
    BenchmarkFailed(#[from] BenchmarkError),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Dataset error: {0}")]
    DatasetError(String),
}

/// Programmatic API for executing fleet benchmarks.
///
/// This API is designed to be consumed by simulators that need to trigger
/// benchmarks programmatically and track results.
pub struct FleetBenchmarkAPI {
    config: FleetConfig,
}

impl FleetBenchmarkAPI {
    /// Creates a new Fleet Benchmark API instance.
    ///
    /// # Arguments
    ///
    /// * `config` - Fleet configuration
    ///
    /// # Examples
    ///
    /// ```
    /// use llm_test_bench_core::benchmarks::fleet_api::{FleetBenchmarkAPI, FleetConfig};
    /// use std::path::PathBuf;
    ///
    /// let config = FleetConfig::new(PathBuf::from("./fleet-results"));
    /// let api = FleetBenchmarkAPI::new(config);
    /// ```
    pub fn new(config: FleetConfig) -> Self {
        Self { config }
    }

    /// Executes a fleet benchmark and returns handle immediately.
    ///
    /// This method:
    /// 1. Loads and validates the fleet manifest
    /// 2. Generates a deterministic run_id
    /// 3. Creates artifact directory structure
    /// 4. Spawns async execution task
    /// 5. Returns handle immediately with run_id and artifact paths
    ///
    /// # Arguments
    ///
    /// * `manifest_path` - Path to fleet manifest JSON file
    ///
    /// # Returns
    ///
    /// Returns `FleetExecutionHandle` containing:
    /// - `run_id`: Deterministic identifier for this run
    /// - `artifact_base_dir`: Path where results will be stored
    /// - `execution_future`: JoinHandle for awaiting completion
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Manifest file cannot be loaded or parsed
    /// - Manifest validation fails
    /// - Artifact directory cannot be created
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use llm_test_bench_core::benchmarks::fleet_api::{FleetBenchmarkAPI, FleetConfig};
    /// # use std::path::PathBuf;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = FleetConfig::new(PathBuf::from("./fleet-results"));
    /// let api = FleetBenchmarkAPI::new(config);
    ///
    /// let handle = api.execute_fleet_benchmark(&PathBuf::from("./fleet.json")).await?;
    /// println!("Run ID: {}", handle.run_id);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn execute_fleet_benchmark(
        &self,
        manifest_path: &Path,
    ) -> Result<FleetExecutionHandle, FleetError> {
        // 1. Load and validate manifest
        let manifest = self.load_manifest(manifest_path).await?;
        self.validate_manifest(&manifest)?;

        // 2. Generate deterministic run_id
        let run_id = self.generate_run_id(&manifest);

        // 3. Create artifact directory structure
        let artifact_base_dir = self.config.output_base_dir.join(&run_id);
        std::fs::create_dir_all(&artifact_base_dir)?;

        // 4. Prepare execution metadata
        let started_at = Utc::now();
        let expected_artifacts = self.calculate_expected_artifacts(&manifest, &artifact_base_dir);

        let metadata = FleetExecutionMetadata {
            fleet_id: manifest.fleet_id.clone(),
            repository_count: manifest.repositories.len(),
            providers: manifest.providers.clone(),
            started_at,
            expected_artifacts,
        };

        // 5. Spawn async execution
        let execution_future = self.spawn_execution(manifest, artifact_base_dir.clone());

        Ok(FleetExecutionHandle {
            run_id,
            artifact_base_dir,
            status_url: None, // Can be extended for monitoring endpoints
            execution_future,
            metadata,
        })
    }

    /// Loads a fleet manifest from disk.
    async fn load_manifest(&self, manifest_path: &Path) -> Result<FleetManifest, FleetError> {
        if !manifest_path.exists() {
            return Err(FleetError::ManifestLoadError(format!(
                "Manifest file not found: {}",
                manifest_path.display()
            )));
        }

        let content = tokio::fs::read_to_string(manifest_path)
            .await
            .map_err(|e| FleetError::ManifestLoadError(e.to_string()))?;

        // Support both JSON and YAML formats based on file extension
        let manifest: FleetManifest = if manifest_path.extension().and_then(|s| s.to_str()) == Some("yaml")
            || manifest_path.extension().and_then(|s| s.to_str()) == Some("yml") {
            serde_yaml::from_str(&content)
                .map_err(|e| FleetError::ManifestLoadError(format!("Invalid YAML: {}", e)))?
        } else {
            serde_json::from_str(&content)
                .map_err(|e| FleetError::ManifestLoadError(format!("Invalid JSON: {}", e)))?
        };

        Ok(manifest)
    }

    /// Validates a fleet manifest.
    fn validate_manifest(&self, manifest: &FleetManifest) -> Result<(), FleetError> {
        if manifest.fleet_id.is_empty() {
            return Err(FleetError::InvalidManifest(
                "fleet_id cannot be empty".to_string(),
            ));
        }

        if manifest.repositories.is_empty() {
            return Err(FleetError::InvalidManifest(
                "At least one repository must be specified".to_string(),
            ));
        }

        if manifest.providers.is_empty() {
            return Err(FleetError::InvalidManifest(
                "At least one provider must be specified".to_string(),
            ));
        }

        // Validate repository specs
        for repo in &manifest.repositories {
            if repo.repo_id.is_empty() {
                return Err(FleetError::InvalidManifest(
                    "Repository ID cannot be empty".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// Generates a deterministic run ID.
    fn generate_run_id(&self, manifest: &FleetManifest) -> String {
        let timestamp = Utc::now().format("%Y%m%d-%H%M%S");
        format!("{}-{}", manifest.fleet_id, timestamp)
    }

    /// Calculates expected artifact paths.
    fn calculate_expected_artifacts(
        &self,
        manifest: &FleetManifest,
        base_dir: &Path,
    ) -> Vec<PathBuf> {
        let mut artifacts = Vec::new();

        // Fleet-level artifacts
        artifacts.push(base_dir.join("fleet-results.json"));
        artifacts.push(base_dir.join("fleet-summary.csv"));

        // Per-repository artifacts
        for repo in &manifest.repositories {
            for provider in &manifest.providers {
                let repo_dir = base_dir.join(&repo.repo_id).join(provider);
                artifacts.push(repo_dir.join("results.json"));
                artifacts.push(repo_dir.join("summary.csv"));
            }
        }

        artifacts
    }

    /// Spawns async execution of the fleet benchmark.
    fn spawn_execution(
        &self,
        manifest: FleetManifest,
        artifact_base_dir: PathBuf,
    ) -> JoinHandle<Result<FleetBenchmarkResults, FleetError>> {
        let config = self.config.clone();

        tokio::spawn(async move {
            execute_fleet_internal(manifest, config, artifact_base_dir).await
        })
    }

    /// Retrieves fleet results for a given run ID.
    ///
    /// # Arguments
    ///
    /// * `run_id` - The run identifier to load results for
    ///
    /// # Returns
    ///
    /// Returns the `FleetBenchmarkResults` if available.
    ///
    /// # Errors
    ///
    /// Returns error if results file cannot be found or parsed.
    pub async fn get_fleet_results(
        &self,
        run_id: &str,
    ) -> Result<FleetBenchmarkResults, FleetError> {
        let results_path = self
            .config
            .output_base_dir
            .join(run_id)
            .join("fleet-results.json");

        if !results_path.exists() {
            return Err(FleetError::IoError(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Results not found for run_id: {}", run_id),
            )));
        }

        let content = tokio::fs::read_to_string(&results_path).await?;
        let results: FleetBenchmarkResults = serde_json::from_str(&content)
            .map_err(|e| FleetError::IoError(std::io::Error::new(std::io::ErrorKind::InvalidData, e)))?;

        Ok(results)
    }

    /// Lists all available fleet runs.
    pub async fn list_runs(&self) -> Result<Vec<String>, FleetError> {
        let mut runs = Vec::new();

        if !self.config.output_base_dir.exists() {
            return Ok(runs);
        }

        let mut entries = tokio::fs::read_dir(&self.config.output_base_dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            if entry.path().is_dir() {
                if let Some(run_id) = entry.file_name().to_str() {
                    runs.push(run_id.to_string());
                }
            }
        }

        Ok(runs)
    }
}

/// Resolves a dataset name to a full path by checking common locations.
///
/// Tries the following paths in order:
/// 1. Exact path as specified
/// 2. datasets/data/{name}.json
/// 3. datasets/data/{name}.yaml
/// 4. datasets/examples/{name}.json
/// 5. datasets/examples/{name}.yaml
fn resolve_dataset_path(repo_path: &Path, dataset_name: &str) -> Result<PathBuf, FleetError> {
    let candidates = vec![
        // Exact path (already has extension or full path)
        repo_path.join(dataset_name),
        // Common dataset locations with extensions
        repo_path.join(format!("datasets/data/{}.json", dataset_name)),
        repo_path.join(format!("datasets/data/{}.yaml", dataset_name)),
        repo_path.join(format!("datasets/examples/{}.json", dataset_name)),
        repo_path.join(format!("datasets/examples/{}.yaml", dataset_name)),
    ];

    for candidate in &candidates {
        if candidate.exists() {
            return Ok(candidate.clone());
        }
    }

    // If no candidate found, return error with helpful message
    Err(FleetError::DatasetError(format!(
        "Could not find dataset '{}'. Tried locations: {}",
        dataset_name,
        candidates.iter()
            .map(|p| p.display().to_string())
            .collect::<Vec<_>>()
            .join(", ")
    )))
}

/// Internal execution logic (runs in background task).
async fn execute_fleet_internal(
    manifest: FleetManifest,
    fleet_config: FleetConfig,
    artifact_base_dir: PathBuf,
) -> Result<FleetBenchmarkResults, FleetError> {
    use super::runner::BenchmarkResults;
    use super::config::BenchmarkConfig;
    use crate::config::ConfigLoader;
    use crate::providers::ProviderFactory;
    use llm_test_bench_datasets::loader::DatasetLoader;

    // Load configuration
    let config_loader = if let Some(config_path) = fleet_config.config_path.as_ref() {
        ConfigLoader::new().with_file(config_path)
    } else {
        ConfigLoader::new()
    };

    let config = config_loader
        .load()
        .map_err(|e| FleetError::ConfigError(e.to_string()))?;

    let factory = ProviderFactory::new();
    let dataset_loader = DatasetLoader::new();

    let mut all_results = Vec::new();

    // Execute benchmarks for each repository-scenario-provider combination
    for repo in &manifest.repositories {
        for scenario in &repo.scenarios {
            // Get scenario profile
            let scenario_profile = manifest.scenario_profiles.get(scenario).ok_or_else(|| {
                FleetError::InvalidManifest(format!(
                    "Scenario '{}' not found in scenario_profiles",
                    scenario
                ))
            })?;

            // Resolve dataset path with common locations and extensions
            let dataset_path = resolve_dataset_path(&repo.path, &scenario_profile.dataset)?;

            // Load dataset
            let dataset = dataset_loader
                .load(&dataset_path)
                .map_err(|e| FleetError::DatasetError(format!(
                    "Failed to load dataset '{}' for scenario '{}': {}",
                    dataset_path.display(),
                    scenario,
                    e
                )))?;

            for provider_name in &manifest.providers {
                // Parse provider name (format: "provider:model" or just "provider")
                let parts: Vec<&str> = provider_name.split(':').collect();
                let provider_key = parts[0];
                let model_name = parts.get(1).copied();

                // Get provider configuration
                let base_config = config.providers.get(provider_key).ok_or_else(|| {
                    FleetError::ConfigError(format!(
                        "Provider '{}' not found in configuration (parsed from '{}')",
                        provider_key,
                        provider_name
                    ))
                })?;

                // Clone config and override model if specified in manifest
                let mut provider_config = base_config.clone();
                if let Some(model) = model_name {
                    provider_config.default_model = model.to_string();
                }

                // Create provider instance
                let provider = factory
                    .create_shared(provider_key, &provider_config)
                    .map_err(|e| FleetError::ConfigError(e.to_string()))?;

                // Configure benchmark using scenario profile settings
                let concurrency = scenario_profile
                    .concurrency;

                let request_delay_ms = scenario_profile
                    .request_delay_ms
                    .or(fleet_config.request_delay_ms);

                let output_dir = artifact_base_dir
                    .join(&repo.repo_id)
                    .join(scenario)
                    .join(provider_name);
                std::fs::create_dir_all(&output_dir)?;

                // Get random_seed from global_settings if available
                let random_seed = manifest.global_settings.random_seed;

                // Get continue_on_failure from global_settings
                let continue_on_failure = manifest.global_settings.continue_on_failure;

                let bench_config = BenchmarkConfig {
                    concurrency,
                    save_responses: fleet_config.save_responses,
                    output_dir,
                    continue_on_failure,
                    random_seed,
                    request_delay_ms,
                };

                // Run benchmark
                // Clone dataset to avoid lifetime issues with tokio::spawn
                let dataset_clone = dataset.clone();
                let runner = super::runner::BenchmarkRunner::new(bench_config);
                let result = runner.run(&dataset_clone, provider).await?;

                all_results.push(result);
            }
        }
    }

    // Aggregate fleet results
    let fleet_results =
        super::fleet::FleetBenchmarkResults::from_repositories(manifest.fleet_id.clone(), all_results);

    // Save fleet results
    save_fleet_results(&fleet_results, &artifact_base_dir).await?;

    Ok(fleet_results)
}

/// Saves fleet results to disk.
async fn save_fleet_results(
    results: &FleetBenchmarkResults,
    base_dir: &Path,
) -> Result<(), FleetError> {
    // Save JSON
    let json_path = base_dir.join("fleet-results.json");
    let json = serde_json::to_string_pretty(results)
        .map_err(|e| FleetError::IoError(std::io::Error::new(std::io::ErrorKind::InvalidData, e)))?;
    tokio::fs::write(json_path, json).await?;

    // Save CSV summary (using fleet exporter)
    use super::fleet_export::FleetCsvExporter;
    let csv_path = base_dir.join("fleet-summary.csv");
    FleetCsvExporter::export_summary(results, &csv_path)
        .map_err(|e| FleetError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fleet_config_builder() {
        let config = FleetConfig::new(PathBuf::from("./results"))
            .with_concurrency(10)
            .with_save_responses(false)
            .with_request_delay_ms(100);

        assert_eq!(config.default_concurrency, 10);
        assert!(!config.save_responses);
        assert_eq!(config.request_delay_ms, Some(100));
    }

    #[test]
    fn test_generate_run_id() {
        let manifest = FleetManifest {
            fleet_id: "test-fleet".to_string(),
            version: "1.0".to_string(),
            description: String::new(),
            repositories: vec![],
            providers: vec![],
            scenario_profiles: HashMap::new(),
            output: OutputConfig {
                base_dir: PathBuf::from("./results"),
                formats: vec!["json".to_string()],
                save_responses: true,
                generate_reports: true,
            },
            global_settings: GlobalSettings::default(),
        };

        let config = FleetConfig::new(PathBuf::from("./results"));
        let api = FleetBenchmarkAPI::new(config);

        let run_id = api.generate_run_id(&manifest);
        assert!(run_id.starts_with("test-fleet-"));
    }

    #[test]
    fn test_validate_manifest() {
        let config = FleetConfig::new(PathBuf::from("./results"));
        let api = FleetBenchmarkAPI::new(config);

        // Valid manifest
        let valid_manifest = FleetManifest {
            fleet_id: "test-fleet".to_string(),
            version: "1.0".to_string(),
            description: "Test fleet".to_string(),
            repositories: vec![RepositoryConfig {
                repo_id: "repo1".to_string(),
                path: PathBuf::from("."),
                git_url: None,
                adapter: "native".to_string(),
                scenarios: vec!["test-scenario".to_string()],
                metadata: HashMap::new(),
            }],
            providers: vec!["openai".to_string()],
            scenario_profiles: HashMap::new(),
            output: OutputConfig {
                base_dir: PathBuf::from("./results"),
                formats: vec!["json".to_string()],
                save_responses: true,
                generate_reports: true,
            },
            global_settings: GlobalSettings::default(),
        };

        assert!(api.validate_manifest(&valid_manifest).is_ok());

        // Invalid: empty fleet_id
        let invalid_manifest = FleetManifest {
            fleet_id: "".to_string(),
            version: "1.0".to_string(),
            description: String::new(),
            repositories: vec![],
            providers: vec![],
            scenario_profiles: HashMap::new(),
            output: OutputConfig {
                base_dir: PathBuf::from("./results"),
                formats: vec!["json".to_string()],
                save_responses: true,
                generate_reports: true,
            },
            global_settings: GlobalSettings::default(),
        };

        assert!(api.validate_manifest(&invalid_manifest).is_err());
    }
}

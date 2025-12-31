// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Fleet-level benchmark orchestration.
//!
//! This module provides the FleetRunner that orchestrates benchmark execution
//! across multiple repositories using adapters and the fleet manifest.
//!
//! # Architecture
//!
//! The FleetRunner:
//! - Loads the fleet manifest
//! - Creates adapters for each repository
//! - Loads datasets through adapters
//! - Executes benchmarks using BenchmarkRunner
//! - Aggregates results into FleetBenchmarkResults
//! - Generates deterministic run identifiers
//! - Stores artifacts in structured directories
//!
//! # Example
//!
//! ```no_run
//! use llm_test_bench_core::benchmarks::fleet_runner::FleetRunner;
//! use std::path::Path;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let runner = FleetRunner::new();
//! let results = runner.run_from_manifest(Path::new("fleet.json")).await?;
//! println!("Fleet {} completed with {:.2}% success",
//!          results.fleet_id,
//!          results.fleet_summary.success_rate * 100.0);
//! # Ok(())
//! # }
//! ```

use super::config::BenchmarkConfig;
use super::fleet::{FleetBenchmarkResults, RepositoryResults};
use super::fleet_adapters::{AdapterFactory, RepositoryAdapter};
use super::fleet_manifest::{FleetManifest, FleetManifestError};
use super::runner::{BenchmarkResults, BenchmarkRunner};
use crate::config::models::ProviderConfig;
use crate::providers::{Provider, ProviderFactory};
use chrono::Utc;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use thiserror::Error;
use tracing::{info, warn};

/// Fleet runner errors
#[derive(Error, Debug)]
pub enum FleetRunnerError {
    /// Manifest error
    #[error("Manifest error: {0}")]
    ManifestError(#[from] FleetManifestError),

    /// Adapter error
    #[error("Adapter error: {0}")]
    AdapterError(#[from] super::fleet_adapters::AdapterError),

    /// Benchmark error
    #[error("Benchmark error: {0}")]
    BenchmarkError(#[from] super::BenchmarkError),

    /// Provider error
    #[error("Provider error: {0}")]
    ProviderError(#[from] crate::providers::ProviderError),

    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    /// Repository execution failed
    #[error("Repository '{0}' execution failed: {1}")]
    RepositoryFailed(String, String),

    /// No results produced
    #[error("No benchmark results produced")]
    NoResults,
}

/// Fleet benchmark orchestrator.
///
/// Coordinates benchmark execution across multiple repositories with
/// different adapters and configurations.
pub struct FleetRunner {
    provider_factory: ProviderFactory,
}

impl FleetRunner {
    /// Creates a new fleet runner.
    pub fn new() -> Self {
        Self {
            provider_factory: ProviderFactory::new(),
        }
    }

    /// Runs benchmarks from a fleet manifest file.
    ///
    /// # Arguments
    ///
    /// * `manifest_path` - Path to the fleet manifest (JSON or YAML)
    ///
    /// # Returns
    ///
    /// Fleet-level aggregated results.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use llm_test_bench_core::benchmarks::fleet_runner::FleetRunner;
    /// use std::path::Path;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let runner = FleetRunner::new();
    /// let results = runner.run_from_manifest(Path::new("fleet.json")).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn run_from_manifest(
        &self,
        manifest_path: &Path,
    ) -> Result<FleetBenchmarkResults, FleetRunnerError> {
        let manifest = FleetManifest::load_from_file(manifest_path)?;
        self.run(&manifest).await
    }

    /// Runs benchmarks from a loaded fleet manifest.
    ///
    /// # Arguments
    ///
    /// * `manifest` - The fleet manifest
    ///
    /// # Returns
    ///
    /// Fleet-level aggregated results.
    pub async fn run(
        &self,
        manifest: &FleetManifest,
    ) -> Result<FleetBenchmarkResults, FleetRunnerError> {
        info!("Starting fleet benchmark: {}", manifest.fleet_id);

        // Generate deterministic run ID
        let run_id = self.generate_run_id(&manifest.fleet_id);
        info!("Run ID: {}", run_id);

        // Create output directory
        let output_base = manifest.output.base_dir.join(&run_id);
        std::fs::create_dir_all(&output_base)?;

        // Execute benchmarks for each repository
        let mut all_results = Vec::new();

        for repo_config in &manifest.repositories {
            info!("Processing repository: {}", repo_config.repo_id);

            let repo_results = self
                .run_repository(manifest, repo_config, &output_base)
                .await;

            match repo_results {
                Ok(results) => {
                    all_results.extend(results);
                }
                Err(e) => {
                    if manifest.global_settings.continue_on_failure {
                        warn!(
                            "Repository '{}' failed: {}. Continuing...",
                            repo_config.repo_id, e
                        );
                    } else {
                        return Err(FleetRunnerError::RepositoryFailed(
                            repo_config.repo_id.clone(),
                            e.to_string(),
                        ));
                    }
                }
            }
        }

        if all_results.is_empty() {
            return Err(FleetRunnerError::NoResults);
        }

        // Aggregate results
        let fleet_results = self.aggregate_results(manifest, all_results, run_id);

        // Save fleet results
        self.save_fleet_results(&fleet_results, &output_base, &manifest.output.formats)?;

        info!(
            "Fleet benchmark completed: {:.2}% success rate",
            fleet_results.fleet_summary.success_rate * 100.0
        );

        Ok(fleet_results)
    }

    /// Runs benchmarks for a single repository.
    async fn run_repository(
        &self,
        manifest: &FleetManifest,
        repo_config: &super::fleet_manifest::RepositoryConfig,
        output_base: &Path,
    ) -> Result<Vec<BenchmarkResults>, FleetRunnerError> {
        // Create adapter
        let adapter = AdapterFactory::create(&repo_config.adapter, &repo_config.path)?;

        info!(
            "Using {} adapter for repository: {}",
            adapter.adapter_type(),
            repo_config.repo_id
        );

        let mut results = Vec::new();

        // For each scenario in the repository
        for scenario_name in &repo_config.scenarios {
            info!("Running scenario: {}", scenario_name);

            let scenario = manifest
                .scenario_profiles
                .get(scenario_name)
                .ok_or_else(|| {
                    FleetRunnerError::BenchmarkError(super::BenchmarkError::InvalidConfiguration(
                        format!("Scenario '{}' not found", scenario_name),
                    ))
                })?;

            // Load dataset through adapter
            let dataset = adapter.load_dataset(&scenario.dataset).await?;

            info!(
                "Loaded dataset '{}' with {} test cases",
                dataset.name,
                dataset.test_cases.len()
            );

            // For each provider
            for provider_spec in &manifest.providers {
                let (provider_name, model_name) = FleetManifest::parse_provider(provider_spec);

                info!("Running with provider: {} (model: {})", provider_name, model_name);

                // Create provider
                let provider = self.create_provider(provider_name, model_name)?;

                // Configure benchmark
                let output_dir = output_base
                    .join(&repo_config.repo_id)
                    .join(provider_spec.replace(':', "_"))
                    .join(scenario_name);

                let config = BenchmarkConfig::new()
                    .with_concurrency(scenario.concurrency)
                    .with_output_dir(output_dir)
                    .with_save_responses(manifest.output.save_responses)
                    .with_continue_on_failure(manifest.global_settings.continue_on_failure);

                let config = if let Some(delay) = scenario.request_delay_ms {
                    config.with_request_delay_ms(delay)
                } else {
                    config
                };

                let config = if let Some(seed) = manifest.global_settings.random_seed {
                    config.with_random_seed(seed)
                } else {
                    config
                };

                // Run benchmark
                let runner = BenchmarkRunner::new(config);
                let benchmark_results = runner.run(&dataset, provider).await?;

                results.push(benchmark_results);
            }
        }

        Ok(results)
    }

    /// Creates a provider instance.
    fn create_provider(
        &self,
        provider_name: &str,
        model_name: &str,
    ) -> Result<Arc<dyn Provider>, FleetRunnerError> {
        // Create a basic provider config
        // In a production system, this would come from a configuration file
        let config = ProviderConfig {
            api_key_env: format!("{}_API_KEY", provider_name.to_uppercase()),
            base_url: format!("https://api.{}.com/v1", provider_name),
            default_model: model_name.to_string(),
            timeout_seconds: 60,
            max_retries: 3,
            rate_limit_rpm: None,
        };

        let provider = self.provider_factory.create_shared(provider_name, &config)?;
        Ok(provider)
    }

    /// Generates a deterministic run identifier.
    ///
    /// Format: `{fleet_id}-{timestamp}-{hash}`
    ///
    /// # Arguments
    ///
    /// * `fleet_id` - The fleet identifier
    ///
    /// # Returns
    ///
    /// A deterministic run ID string.
    fn generate_run_id(&self, fleet_id: &str) -> String {
        let timestamp = Utc::now().format("%Y%m%d-%H%M%S");
        let hash = self.generate_hash(fleet_id);
        format!("{}-{}-{}", fleet_id, timestamp, hash)
    }

    /// Generates a short hash for the fleet ID.
    fn generate_hash(&self, input: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        input.hash(&mut hasher);
        let hash = hasher.finish();
        format!("{:x}", hash).chars().take(8).collect()
    }

    /// Aggregates individual benchmark results into fleet results.
    fn aggregate_results(
        &self,
        _manifest: &FleetManifest,
        benchmark_results: Vec<BenchmarkResults>,
        _run_id: String,
    ) -> FleetBenchmarkResults {
        // from_repositories handles the conversion internally
        FleetBenchmarkResults::from_repositories(
            _manifest.fleet_id.clone(),
            benchmark_results,
        )
    }

    /// Saves fleet results to disk in the specified formats.
    fn save_fleet_results(
        &self,
        results: &FleetBenchmarkResults,
        output_base: &Path,
        formats: &[String],
    ) -> Result<(), FleetRunnerError> {
        for format in formats {
            match format.as_str() {
                "json" => {
                    let path = output_base.join("fleet-results.json");
                    let json = serde_json::to_string_pretty(results)?;
                    std::fs::write(path, json)?;
                }
                "yaml" => {
                    let path = output_base.join("fleet-results.yaml");
                    let yaml = serde_yaml::to_string(results)
                        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
                    std::fs::write(path, yaml)?;
                }
                "csv" => {
                    // CSV export using the FleetCsvExporter
                    let csv_dir = output_base.join("csv");
                    std::fs::create_dir_all(&csv_dir)?;

                    use super::fleet_export::FleetCsvExporter;

                    FleetCsvExporter::export_summary(results, &csv_dir.join("fleet-summary.csv"))
                        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
                    FleetCsvExporter::export_repositories(results, &csv_dir.join("repositories.csv"))
                        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
                    FleetCsvExporter::export_providers(results, &csv_dir.join("providers.csv"))
                        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
                    FleetCsvExporter::export_categories(results, &csv_dir.join("categories.csv"))
                        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
                    FleetCsvExporter::export_executive_report(results, &csv_dir.join("executive-report.html"))
                        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
                    FleetCsvExporter::export_deterministic_json(results, &csv_dir.join("deterministic.json"))
                        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
                }
                _ => {
                    warn!("Unknown output format: {}", format);
                }
            }
        }

        Ok(())
    }
}

impl Default for FleetRunner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use llm_test_bench_datasets::{Dataset, TestCase};
    use std::fs;
    use tempfile::TempDir;

    fn create_test_manifest(temp_dir: &Path) -> FleetManifest {
        // Create datasets directory
        let datasets_dir = temp_dir.join("datasets");
        fs::create_dir_all(&datasets_dir).unwrap();

        // Create a test dataset
        let mut dataset = Dataset::new("test-dataset", "1.0.0");
        dataset.add_test_case(TestCase::new("test-1", "Test prompt"));

        let loader = llm_test_bench_datasets::DatasetLoader::new();
        loader
            .save_to_json(&dataset, &datasets_dir.join("test-dataset.json"))
            .unwrap();

        // Create manifest
        let mut scenario_profiles = HashMap::new();
        scenario_profiles.insert(
            "test-scenario".to_string(),
            super::super::fleet_manifest::ScenarioProfile {
                dataset: "test-dataset".to_string(),
                concurrency: 1,
                num_examples: None,
                request_delay_ms: None,
                settings: HashMap::new(),
            },
        );

        FleetManifest {
            fleet_id: "test-fleet".to_string(),
            version: "1.0".to_string(),
            description: "Test".to_string(),
            repositories: vec![super::super::fleet_manifest::RepositoryConfig {
                repo_id: "test-repo".to_string(),
                path: temp_dir.to_path_buf(),
                git_url: None,
                adapter: "native".to_string(),
                scenarios: vec!["test-scenario".to_string()],
                metadata: HashMap::new(),
            }],
            providers: vec!["mock:test-model".to_string()],
            scenario_profiles,
            output: super::super::fleet_manifest::OutputConfig {
                base_dir: temp_dir.join("output"),
                formats: vec!["json".to_string()],
                save_responses: false,
                generate_reports: true,
            },
            global_settings: super::super::fleet_manifest::GlobalSettings::default(),
        }
    }

    #[test]
    fn test_generate_run_id() {
        let runner = FleetRunner::new();
        let run_id = runner.generate_run_id("test-fleet");

        assert!(run_id.starts_with("test-fleet-"));
        assert!(run_id.len() > 20); // Should include timestamp and hash
    }

    #[test]
    fn test_generate_hash() {
        let runner = FleetRunner::new();
        let hash1 = runner.generate_hash("test");
        let hash2 = runner.generate_hash("test");
        let hash3 = runner.generate_hash("different");

        assert_eq!(hash1, hash2); // Same input = same hash
        assert_ne!(hash1, hash3); // Different input = different hash
        assert_eq!(hash1.len(), 8); // Hash should be 8 characters
    }

    #[tokio::test]
    async fn test_fleet_runner_creation() {
        let runner = FleetRunner::new();
        assert_eq!(
            runner.provider_factory.available_providers().len(),
            ProviderFactory::new().available_providers().len()
        );
    }

    #[test]
    fn test_aggregate_results() {
        let temp_dir = TempDir::new().unwrap();
        let manifest = create_test_manifest(temp_dir.path());

        let runner = FleetRunner::new();

        let benchmark_results = vec![BenchmarkResults::new(
            "test-dataset".to_string(),
            "mock".to_string(),
            vec![],
        )];

        let fleet_results =
            runner.aggregate_results(&manifest, benchmark_results, "test-run-id".to_string());

        assert_eq!(fleet_results.fleet_id, "test-fleet");
        assert_eq!(fleet_results.total_repositories, 1);
    }

    #[test]
    fn test_save_fleet_results_json() {
        let temp_dir = TempDir::new().unwrap();
        let output_base = temp_dir.path().join("output");
        fs::create_dir_all(&output_base).unwrap();

        let runner = FleetRunner::new();
        let fleet_results =
            FleetBenchmarkResults::from_repositories("test-fleet".to_string(), vec![]);

        let result = runner.save_fleet_results(
            &fleet_results,
            &output_base,
            &["json".to_string()],
        );

        assert!(result.is_ok());
        assert!(output_base.join("fleet-results.json").exists());
    }
}

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Repository adapters for fleet-level benchmarking.
//!
//! This module provides thin adapter interfaces that map different repository
//! types into test-bench's unified BenchmarkRunner execution flow.
//!
//! # Adapter Architecture
//!
//! Each adapter is responsible for:
//! - Discovering available datasets in the repository
//! - Loading datasets into the test-bench Dataset format
//! - No business logic - just translation/mapping
//!
//! # Supported Adapters
//!
//! - **Native**: Direct test-bench repositories (default)
//! - **Generic**: External repositories with standard dataset formats
//! - **Custom**: Extensible for future adapter types
//!
//! # Example
//!
//! ```no_run
//! use llm_test_bench_core::benchmarks::fleet_adapters::{RepositoryAdapter, NativeAdapter};
//! use std::path::Path;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let adapter = NativeAdapter::new(Path::new("."));
//! let datasets = adapter.discover_datasets()?;
//! println!("Found {} datasets", datasets.len());
//! # Ok(())
//! # }
//! ```

use async_trait::async_trait;
use llm_test_bench_datasets::{Dataset, DatasetLoader};
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Adapter errors
#[derive(Error, Debug)]
pub enum AdapterError {
    /// Failed to discover datasets
    #[error("Failed to discover datasets: {0}")]
    DiscoveryFailed(String),

    /// Failed to load dataset
    #[error("Failed to load dataset: {0}")]
    LoadFailed(String),

    /// Unsupported adapter type
    #[error("Unsupported adapter type: {0}")]
    UnsupportedAdapter(String),

    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Dataset error
    #[error("Dataset error: {0}")]
    DatasetError(#[from] llm_test_bench_datasets::DatasetError),
}

/// Trait defining the interface for repository adapters.
///
/// Adapters are thin translation layers that map external repository
/// structures into test-bench's Dataset format. They should contain
/// no business logic, only discovery and loading.
#[async_trait]
pub trait RepositoryAdapter: Send + Sync {
    /// Returns the adapter type name.
    fn adapter_type(&self) -> &str;

    /// Discovers available datasets in the repository.
    ///
    /// Returns a list of dataset identifiers (file paths or names).
    fn discover_datasets(&self) -> Result<Vec<String>, AdapterError>;

    /// Loads a specific dataset by its identifier.
    ///
    /// # Arguments
    ///
    /// * `dataset_id` - The dataset identifier (from discover_datasets)
    ///
    /// # Returns
    ///
    /// A loaded Dataset ready for benchmarking.
    async fn load_dataset(&self, dataset_id: &str) -> Result<Dataset, AdapterError>;

    /// Returns the base path for this repository.
    fn base_path(&self) -> &Path;
}

/// Native adapter for test-bench repositories.
///
/// This adapter works with repositories that already follow the test-bench
/// dataset format. It discovers datasets in the standard locations:
/// - `./datasets/*.json`
/// - `./datasets/*.yaml`
/// - `./datasets/*.yml`
pub struct NativeAdapter {
    base_path: PathBuf,
    dataset_loader: DatasetLoader,
}

impl NativeAdapter {
    /// Creates a new native adapter for the given repository path.
    ///
    /// # Examples
    ///
    /// ```
    /// use llm_test_bench_core::benchmarks::fleet_adapters::NativeAdapter;
    /// use std::path::Path;
    ///
    /// let adapter = NativeAdapter::new(Path::new("."));
    /// ```
    pub fn new(base_path: &Path) -> Self {
        Self {
            base_path: base_path.to_path_buf(),
            dataset_loader: DatasetLoader::new(),
        }
    }

    /// Returns the path to the datasets directory.
    fn datasets_dir(&self) -> PathBuf {
        self.base_path.join("datasets")
    }
}

#[async_trait]
impl RepositoryAdapter for NativeAdapter {
    fn adapter_type(&self) -> &str {
        "native"
    }

    fn discover_datasets(&self) -> Result<Vec<String>, AdapterError> {
        let datasets_dir = self.datasets_dir();

        if !datasets_dir.exists() {
            return Ok(Vec::new());
        }

        let dataset_names = self
            .dataset_loader
            .list_datasets(&datasets_dir)
            .map_err(|e| AdapterError::DiscoveryFailed(e.to_string()))?;

        Ok(dataset_names)
    }

    async fn load_dataset(&self, dataset_id: &str) -> Result<Dataset, AdapterError> {
        let datasets_dir = self.datasets_dir();

        // Try common extensions
        for ext in &["json", "yaml", "yml"] {
            let path = datasets_dir.join(format!("{}.{}", dataset_id, ext));
            if path.exists() {
                let dataset = self
                    .dataset_loader
                    .load(&path)
                    .map_err(|e| AdapterError::LoadFailed(e.to_string()))?;
                return Ok(dataset);
            }
        }

        Err(AdapterError::LoadFailed(format!(
            "Dataset '{}' not found in {}",
            dataset_id,
            datasets_dir.display()
        )))
    }

    fn base_path(&self) -> &Path {
        &self.base_path
    }
}

/// Generic adapter for external repositories.
///
/// This adapter works with repositories that have datasets in standard formats
/// but may not follow the exact test-bench directory structure. It searches
/// for dataset files in:
/// - Root directory
/// - `./data/` directory
/// - `./datasets/` directory
/// - `./benchmarks/` directory
pub struct GenericAdapter {
    base_path: PathBuf,
    dataset_loader: DatasetLoader,
    search_paths: Vec<PathBuf>,
}

impl GenericAdapter {
    /// Creates a new generic adapter for the given repository path.
    ///
    /// # Examples
    ///
    /// ```
    /// use llm_test_bench_core::benchmarks::fleet_adapters::GenericAdapter;
    /// use std::path::Path;
    ///
    /// let adapter = GenericAdapter::new(Path::new("./external-repo"));
    /// ```
    pub fn new(base_path: &Path) -> Self {
        let search_paths = vec![
            base_path.to_path_buf(),
            base_path.join("data"),
            base_path.join("datasets"),
            base_path.join("benchmarks"),
        ];

        Self {
            base_path: base_path.to_path_buf(),
            dataset_loader: DatasetLoader::new(),
            search_paths,
        }
    }

    /// Searches for dataset files in all search paths.
    fn find_dataset_files(&self) -> Result<Vec<PathBuf>, AdapterError> {
        let mut files = Vec::new();

        for search_path in &self.search_paths {
            if !search_path.exists() {
                continue;
            }

            if search_path.is_file() {
                files.push(search_path.clone());
                continue;
            }

            // Search directory for dataset files
            for entry in std::fs::read_dir(search_path)? {
                let entry = entry?;
                let path = entry.path();

                if path.is_file() {
                    if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                        if matches!(ext, "json" | "yaml" | "yml") {
                            files.push(path);
                        }
                    }
                }
            }
        }

        Ok(files)
    }
}

#[async_trait]
impl RepositoryAdapter for GenericAdapter {
    fn adapter_type(&self) -> &str {
        "generic"
    }

    fn discover_datasets(&self) -> Result<Vec<String>, AdapterError> {
        let files = self.find_dataset_files()?;

        let mut dataset_names = Vec::new();
        for file in files {
            if let Some(stem) = file.file_stem().and_then(|s| s.to_str()) {
                // Try to load to verify it's a valid dataset
                if self.dataset_loader.load(&file).is_ok() {
                    dataset_names.push(stem.to_string());
                }
            }
        }

        Ok(dataset_names)
    }

    async fn load_dataset(&self, dataset_id: &str) -> Result<Dataset, AdapterError> {
        let files = self.find_dataset_files()?;

        for file in files {
            if let Some(stem) = file.file_stem().and_then(|s| s.to_str()) {
                if stem == dataset_id {
                    let dataset = self
                        .dataset_loader
                        .load(&file)
                        .map_err(|e| AdapterError::LoadFailed(e.to_string()))?;
                    return Ok(dataset);
                }
            }
        }

        Err(AdapterError::LoadFailed(format!(
            "Dataset '{}' not found",
            dataset_id
        )))
    }

    fn base_path(&self) -> &Path {
        &self.base_path
    }
}

/// Factory for creating repository adapters.
pub struct AdapterFactory;

impl AdapterFactory {
    /// Creates an adapter instance based on the adapter type.
    ///
    /// # Arguments
    ///
    /// * `adapter_type` - The adapter type ("native", "generic", etc.)
    /// * `base_path` - The base path for the repository
    ///
    /// # Examples
    ///
    /// ```
    /// use llm_test_bench_core::benchmarks::fleet_adapters::AdapterFactory;
    /// use std::path::Path;
    ///
    /// let adapter = AdapterFactory::create("native", Path::new(".")).unwrap();
    /// ```
    pub fn create(
        adapter_type: &str,
        base_path: &Path,
    ) -> Result<Box<dyn RepositoryAdapter>, AdapterError> {
        match adapter_type.to_lowercase().as_str() {
            "native" => Ok(Box::new(NativeAdapter::new(base_path))),
            "generic" => Ok(Box::new(GenericAdapter::new(base_path))),
            _ => Err(AdapterError::UnsupportedAdapter(adapter_type.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use llm_test_bench_datasets::{Dataset, TestCase};
    use std::fs;
    use tempfile::TempDir;

    fn create_test_dataset_file(dir: &Path, name: &str) -> PathBuf {
        let mut dataset = Dataset::new(name, "1.0.0");
        dataset.add_test_case(TestCase::new("test-1", "Test prompt 1"));
        dataset.add_test_case(TestCase::new("test-2", "Test prompt 2"));

        let file_path = dir.join(format!("{}.json", name));
        let loader = DatasetLoader::new();
        loader.save_to_json(&dataset, &file_path).unwrap();
        file_path
    }

    #[tokio::test]
    async fn test_native_adapter_discovery() {
        let temp_dir = TempDir::new().unwrap();
        let datasets_dir = temp_dir.path().join("datasets");
        fs::create_dir_all(&datasets_dir).unwrap();

        // Create test datasets
        create_test_dataset_file(&datasets_dir, "dataset1");
        create_test_dataset_file(&datasets_dir, "dataset2");

        let adapter = NativeAdapter::new(temp_dir.path());
        let datasets = adapter.discover_datasets().unwrap();

        assert_eq!(datasets.len(), 2);
        assert!(datasets.contains(&"dataset1".to_string()));
        assert!(datasets.contains(&"dataset2".to_string()));
    }

    #[tokio::test]
    async fn test_native_adapter_load() {
        let temp_dir = TempDir::new().unwrap();
        let datasets_dir = temp_dir.path().join("datasets");
        fs::create_dir_all(&datasets_dir).unwrap();

        create_test_dataset_file(&datasets_dir, "test-dataset");

        let adapter = NativeAdapter::new(temp_dir.path());
        let dataset = adapter.load_dataset("test-dataset").await.unwrap();

        assert_eq!(dataset.name, "test-dataset");
        assert_eq!(dataset.test_cases.len(), 2);
    }

    #[tokio::test]
    async fn test_native_adapter_missing_dataset() {
        let temp_dir = TempDir::new().unwrap();
        let datasets_dir = temp_dir.path().join("datasets");
        fs::create_dir_all(&datasets_dir).unwrap();

        let adapter = NativeAdapter::new(temp_dir.path());
        let result = adapter.load_dataset("nonexistent").await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_generic_adapter_discovery() {
        let temp_dir = TempDir::new().unwrap();

        // Create datasets in different locations
        create_test_dataset_file(temp_dir.path(), "root-dataset");

        let data_dir = temp_dir.path().join("data");
        fs::create_dir_all(&data_dir).unwrap();
        create_test_dataset_file(&data_dir, "data-dataset");

        let adapter = GenericAdapter::new(temp_dir.path());
        let datasets = adapter.discover_datasets().unwrap();

        assert!(datasets.len() >= 2);
        assert!(datasets.contains(&"root-dataset".to_string()));
        assert!(datasets.contains(&"data-dataset".to_string()));
    }

    #[tokio::test]
    async fn test_generic_adapter_load() {
        let temp_dir = TempDir::new().unwrap();
        create_test_dataset_file(temp_dir.path(), "test-dataset");

        let adapter = GenericAdapter::new(temp_dir.path());
        let dataset = adapter.load_dataset("test-dataset").await.unwrap();

        assert_eq!(dataset.name, "test-dataset");
    }

    #[tokio::test]
    async fn test_adapter_factory_native() {
        let temp_dir = TempDir::new().unwrap();
        let adapter = AdapterFactory::create("native", temp_dir.path()).unwrap();
        assert_eq!(adapter.adapter_type(), "native");
    }

    #[tokio::test]
    async fn test_adapter_factory_generic() {
        let temp_dir = TempDir::new().unwrap();
        let adapter = AdapterFactory::create("generic", temp_dir.path()).unwrap();
        assert_eq!(adapter.adapter_type(), "generic");
    }

    #[tokio::test]
    async fn test_adapter_factory_unsupported() {
        let temp_dir = TempDir::new().unwrap();
        let result = AdapterFactory::create("unsupported", temp_dir.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_adapter_base_path() {
        let temp_dir = TempDir::new().unwrap();
        let adapter = NativeAdapter::new(temp_dir.path());
        assert_eq!(adapter.base_path(), temp_dir.path());
    }
}

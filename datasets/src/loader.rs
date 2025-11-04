// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Dataset loader implementation

use crate::schema::Dataset;
use crate::DatasetError;
use serde_valid::Validate;
use std::path::Path;

/// Dataset loader
pub struct DatasetLoader;

impl DatasetLoader {
    /// Create a new dataset loader
    pub fn new() -> Self {
        Self
    }

    /// Load a dataset from a file (auto-detects JSON or YAML)
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the dataset file (.json, .yaml, or .yml)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use llm_test_bench_datasets::loader::DatasetLoader;
    /// use std::path::Path;
    ///
    /// let loader = DatasetLoader::new();
    /// let dataset = loader.load(Path::new("dataset.json")).unwrap();
    /// println!("Loaded: {} with {} tests", dataset.name, dataset.test_cases.len());
    /// ```
    pub fn load(&self, path: &Path) -> Result<Dataset, DatasetError> {
        let content = std::fs::read_to_string(path)?;

        // Auto-detect format by extension
        let dataset = match path.extension().and_then(|s| s.to_str()) {
            Some("yaml") | Some("yml") => {
                serde_yaml::from_str::<Dataset>(&content)?
            }
            Some("json") | _ => {
                serde_json::from_str::<Dataset>(&content)?
            }
        };

        // Validate schema
        dataset.validate().map_err(|e| {
            DatasetError::ValidationError(format!("Dataset validation failed: {}", e))
        })?;

        Ok(dataset)
    }

    /// Load a dataset from a JSON file
    pub fn load_from_json(&self, path: &Path) -> Result<Dataset, DatasetError> {
        let content = std::fs::read_to_string(path)?;
        let dataset: Dataset = serde_json::from_str(&content)?;

        dataset.validate().map_err(|e| {
            DatasetError::ValidationError(format!("Dataset validation failed: {}", e))
        })?;

        Ok(dataset)
    }

    /// Load a dataset from a YAML file
    pub fn load_from_yaml(&self, path: &Path) -> Result<Dataset, DatasetError> {
        let content = std::fs::read_to_string(path)?;
        let dataset: Dataset = serde_yaml::from_str(&content)?;

        dataset.validate().map_err(|e| {
            DatasetError::ValidationError(format!("Dataset validation failed: {}", e))
        })?;

        Ok(dataset)
    }

    /// Save a dataset to a JSON file
    pub fn save_to_json(&self, dataset: &Dataset, path: &Path) -> Result<(), DatasetError> {
        let content = serde_json::to_string_pretty(dataset)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Save a dataset to a YAML file
    pub fn save_to_yaml(&self, dataset: &Dataset, path: &Path) -> Result<(), DatasetError> {
        let content = serde_yaml::to_string(dataset)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Load multiple datasets from a directory
    pub fn load_dir(&self, dir: &Path) -> Result<Vec<Dataset>, DatasetError> {
        let mut datasets = Vec::new();

        if !dir.exists() {
            return Ok(datasets);
        }

        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() && Self::is_dataset_file(&path) {
                match self.load(&path) {
                    Ok(dataset) => datasets.push(dataset),
                    Err(e) => {
                        tracing::warn!("Failed to load {}: {}", path.display(), e);
                    }
                }
            }
        }

        Ok(datasets)
    }

    /// List available datasets in a directory
    pub fn list_datasets(&self, dir: &Path) -> Result<Vec<String>, DatasetError> {
        let mut datasets = Vec::new();

        if !dir.exists() {
            return Ok(datasets);
        }

        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() && Self::is_dataset_file(&path) {
                if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                    datasets.push(name.to_string());
                }
            }
        }

        Ok(datasets)
    }

    fn is_dataset_file(path: &Path) -> bool {
        matches!(
            path.extension().and_then(|s| s.to_str()),
            Some("json") | Some("yaml") | Some("yml")
        )
    }
}

impl Default for DatasetLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::TestCase;

    #[test]
    fn test_dataset_loader_creation() {
        let _loader = DatasetLoader::new();
    }

    #[test]
    fn test_save_and_load_dataset() {
        let loader = DatasetLoader::new();
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("test_dataset.json");

        let mut dataset = Dataset::new("test", "1.0.0");
        dataset.add_test_case(TestCase::new("tc1", "Test prompt"));

        // Save
        loader.save_to_json(&dataset, &file_path).unwrap();
        assert!(file_path.exists());

        // Load
        let loaded_dataset = loader.load_from_json(&file_path).unwrap();
        assert_eq!(loaded_dataset.name, dataset.name);
        assert_eq!(loaded_dataset.test_cases.len(), 1);
    }
}

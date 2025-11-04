// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Dataset schema definitions with validation.
//!
//! This module provides comprehensive dataset structures with built-in validation
//! using serde_valid. Datasets can be loaded from JSON or YAML files and validated
//! against the schema requirements.

use serde::{Deserialize, Serialize};
use serde_valid::Validate;
use std::collections::HashMap;

/// Main dataset structure containing test cases and configuration.
///
/// # Example
///
/// ```
/// use llm_test_bench_datasets::schema::{Dataset, TestCase};
/// use serde_valid::Validate;
///
/// let mut dataset = Dataset {
///     name: "my-dataset".to_string(),
///     description: Some("Test dataset".to_string()),
///     version: "1.0.0".to_string(),
///     test_cases: vec![
///         TestCase {
///             id: "test-1".to_string(),
///             category: Some("coding".to_string()),
///             prompt: "Write a function".to_string(),
///             variables: None,
///             expected: None,
///             references: None,
///             config: None,
///             metadata: None,
///         }
///     ],
///     defaults: None,
///     metadata: None,
/// };
///
/// assert!(dataset.validate().is_ok());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct Dataset {
    /// Dataset name (required, minimum length 1)
    #[validate(min_length = 1)]
    pub name: String,

    /// Dataset description (optional)
    pub description: Option<String>,

    /// Dataset version (semantic versioning recommended)
    pub version: String,

    /// Test cases (required, minimum 1 test case)
    #[validate(min_items = 1)]
    pub test_cases: Vec<TestCase>,

    /// Default model configuration for all test cases
    pub defaults: Option<DefaultConfig>,

    /// Additional metadata for the dataset
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Individual test case with prompt and optional configuration.
///
/// Test cases support templating via the `{{variable}}` syntax. Variables
/// are substituted at runtime using the values in the `variables` map.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct TestCase {
    /// Unique test case identifier (required, minimum length 1)
    #[validate(min_length = 1)]
    pub id: String,

    /// Category or tag for grouping tests
    pub category: Option<String>,

    /// Prompt template (required, supports {{variable}} substitution)
    #[validate(min_length = 1)]
    pub prompt: String,

    /// Variable values for template substitution
    pub variables: Option<HashMap<String, String>>,

    /// Expected output for evaluation (optional)
    pub expected: Option<String>,

    /// Reference answers for comparison (optional)
    pub references: Option<Vec<String>>,

    /// Per-test configuration overrides (optional)
    pub config: Option<TestConfig>,

    /// Test-specific metadata (optional)
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Default configuration applied to all test cases unless overridden.
///
/// These settings provide dataset-wide defaults that can be overridden
/// on a per-test basis using `TestConfig`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefaultConfig {
    /// Default temperature for sampling
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,

    /// Default maximum tokens to generate
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<usize>,

    /// Default top-p (nucleus sampling) parameter
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,

    /// Default stop sequences
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<String>>,
}

/// Per-test configuration that overrides dataset defaults.
///
/// Allows fine-grained control over model parameters for individual tests.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestConfig {
    /// Model to use for this test (overrides provider default)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,

    /// Temperature for this test
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,

    /// Maximum tokens for this test
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<usize>,

    /// Top-p parameter for this test
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,

    /// Stop sequences for this test
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<String>>,
}

impl Dataset {
    /// Create a new dataset with the given name and version.
    ///
    /// # Example
    ///
    /// ```
    /// use llm_test_bench_datasets::schema::Dataset;
    ///
    /// let dataset = Dataset::new("my-dataset", "1.0.0");
    /// assert_eq!(dataset.name, "my-dataset");
    /// assert_eq!(dataset.version, "1.0.0");
    /// ```
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            version: version.into(),
            test_cases: Vec::new(),
            defaults: None,
            metadata: None,
        }
    }

    /// Set the dataset description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Add a test case to the dataset.
    pub fn add_test_case(&mut self, test_case: TestCase) {
        self.test_cases.push(test_case);
    }

    /// Set default configuration for all test cases.
    pub fn with_defaults(mut self, defaults: DefaultConfig) -> Self {
        self.defaults = Some(defaults);
        self
    }

    /// Get test cases by category.
    pub fn filter_by_category(&self, category: &str) -> Vec<&TestCase> {
        self.test_cases
            .iter()
            .filter(|tc| tc.category.as_deref() == Some(category))
            .collect()
    }

    /// Get total number of test cases.
    pub fn len(&self) -> usize {
        self.test_cases.len()
    }

    /// Check if dataset is empty.
    pub fn is_empty(&self) -> bool {
        self.test_cases.is_empty()
    }
}

impl TestCase {
    /// Create a new test case with the given ID and prompt.
    ///
    /// # Example
    ///
    /// ```
    /// use llm_test_bench_datasets::schema::TestCase;
    ///
    /// let test = TestCase::new("test-1", "Explain Rust ownership");
    /// assert_eq!(test.id, "test-1");
    /// assert_eq!(test.prompt, "Explain Rust ownership");
    /// ```
    pub fn new(id: impl Into<String>, prompt: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            category: None,
            prompt: prompt.into(),
            variables: None,
            expected: None,
            references: None,
            config: None,
            metadata: None,
        }
    }

    /// Set the category for this test case.
    pub fn with_category(mut self, category: impl Into<String>) -> Self {
        self.category = Some(category.into());
        self
    }

    /// Set variables for template substitution.
    pub fn with_variables(mut self, variables: HashMap<String, String>) -> Self {
        self.variables = Some(variables);
        self
    }

    /// Add a single variable for template substitution.
    pub fn add_variable(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.variables
            .get_or_insert_with(HashMap::new)
            .insert(key.into(), value.into());
        self
    }

    /// Set the expected output for evaluation.
    pub fn with_expected(mut self, expected: impl Into<String>) -> Self {
        self.expected = Some(expected.into());
        self
    }

    /// Set reference answers for comparison.
    pub fn with_references(mut self, references: Vec<String>) -> Self {
        self.references = Some(references);
        self
    }

    /// Set test-specific configuration.
    pub fn with_config(mut self, config: TestConfig) -> Self {
        self.config = Some(config);
        self
    }
}

impl DefaultConfig {
    /// Create a new default configuration.
    pub fn new() -> Self {
        Self {
            temperature: None,
            max_tokens: None,
            top_p: None,
            stop: None,
        }
    }

    /// Set the default temperature.
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }

    /// Set the default max tokens.
    pub fn with_max_tokens(mut self, max_tokens: usize) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    /// Set the default top-p.
    pub fn with_top_p(mut self, top_p: f32) -> Self {
        self.top_p = Some(top_p);
        self
    }

    /// Set the default stop sequences.
    pub fn with_stop(mut self, stop: Vec<String>) -> Self {
        self.stop = Some(stop);
        self
    }
}

impl Default for DefaultConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl TestConfig {
    /// Create a new test configuration.
    pub fn new() -> Self {
        Self {
            model: None,
            temperature: None,
            max_tokens: None,
            top_p: None,
            stop: None,
        }
    }

    /// Set the model for this test.
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    /// Set the temperature for this test.
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }

    /// Set the max tokens for this test.
    pub fn with_max_tokens(mut self, max_tokens: usize) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    /// Set the top-p for this test.
    pub fn with_top_p(mut self, top_p: f32) -> Self {
        self.top_p = Some(top_p);
        self
    }

    /// Set the stop sequences for this test.
    pub fn with_stop(mut self, stop: Vec<String>) -> Self {
        self.stop = Some(stop);
        self
    }
}

impl Default for TestConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dataset_creation() {
        let dataset = Dataset::new("test-dataset", "1.0.0")
            .with_description("Test description");

        assert_eq!(dataset.name, "test-dataset");
        assert_eq!(dataset.version, "1.0.0");
        assert_eq!(dataset.description, Some("Test description".to_string()));
        assert_eq!(dataset.test_cases.len(), 0);
    }

    #[test]
    fn test_dataset_validation_empty_name() {
        let dataset = Dataset {
            name: "".to_string(), // Invalid: empty name
            description: None,
            version: "1.0.0".to_string(),
            test_cases: vec![TestCase::new("test-1", "prompt")],
            defaults: None,
            metadata: None,
        };

        assert!(dataset.validate().is_err());
    }

    #[test]
    fn test_dataset_validation_no_test_cases() {
        let dataset = Dataset {
            name: "test".to_string(),
            description: None,
            version: "1.0.0".to_string(),
            test_cases: vec![], // Invalid: no test cases
            defaults: None,
            metadata: None,
        };

        assert!(dataset.validate().is_err());
    }

    #[test]
    fn test_dataset_validation_valid() {
        let dataset = Dataset {
            name: "test".to_string(),
            description: None,
            version: "1.0.0".to_string(),
            test_cases: vec![TestCase::new("test-1", "prompt")],
            defaults: None,
            metadata: None,
        };

        assert!(dataset.validate().is_ok());
    }

    #[test]
    fn test_test_case_creation() {
        let test = TestCase::new("test-1", "What is Rust?")
            .with_category("qa")
            .with_expected("Rust is a systems programming language");

        assert_eq!(test.id, "test-1");
        assert_eq!(test.prompt, "What is Rust?");
        assert_eq!(test.category, Some("qa".to_string()));
        assert!(test.expected.is_some());
    }

    #[test]
    fn test_test_case_with_variables() {
        let test = TestCase::new("test-1", "Explain {{topic}}")
            .add_variable("topic", "ownership");

        assert_eq!(test.variables.as_ref().unwrap().get("topic").unwrap(), "ownership");
    }

    #[test]
    fn test_filter_by_category() {
        let mut dataset = Dataset::new("test", "1.0.0");
        dataset.add_test_case(TestCase::new("t1", "prompt1").with_category("coding"));
        dataset.add_test_case(TestCase::new("t2", "prompt2").with_category("qa"));
        dataset.add_test_case(TestCase::new("t3", "prompt3").with_category("coding"));

        let coding_tests = dataset.filter_by_category("coding");
        assert_eq!(coding_tests.len(), 2);
    }

    #[test]
    fn test_default_config() {
        let config = DefaultConfig::new()
            .with_temperature(0.7)
            .with_max_tokens(500);

        assert_eq!(config.temperature, Some(0.7));
        assert_eq!(config.max_tokens, Some(500));
    }

    #[test]
    fn test_test_config() {
        let config = TestConfig::new()
            .with_model("gpt-4")
            .with_temperature(0.0);

        assert_eq!(config.model, Some("gpt-4".to_string()));
        assert_eq!(config.temperature, Some(0.0));
    }
}

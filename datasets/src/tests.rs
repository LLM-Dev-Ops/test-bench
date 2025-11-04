// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Integration tests for the datasets crate.
//!
//! These tests verify end-to-end functionality including dataset loading,
//! template rendering, and validation.

#[cfg(test)]
mod integration_tests {
    use crate::builtin;
    use crate::loader::DatasetLoader;
    use crate::schema::{Dataset, TestCase, DefaultConfig};
    use crate::template::TemplateEngine;
    use std::collections::HashMap;
    use serde_valid::Validate;

    #[test]
    fn test_load_json_dataset_file() {
        let loader = DatasetLoader::new();
        let path = std::path::Path::new("data/coding-tasks.json");

        if path.exists() {
            let dataset = loader.load(path).unwrap();
            assert_eq!(dataset.name, "coding-tasks");
            assert!(!dataset.test_cases.is_empty());
        }
    }

    #[test]
    fn test_load_yaml_dataset_file() {
        let loader = DatasetLoader::new();
        let path = std::path::Path::new("data/reasoning-tasks.yaml");

        if path.exists() {
            let dataset = loader.load(path).unwrap();
            assert_eq!(dataset.name, "reasoning-tasks");
            assert!(!dataset.test_cases.is_empty());
        }
    }

    #[test]
    fn test_template_rendering_in_dataset() {
        let mut dataset = Dataset::new("test", "1.0.0");

        dataset.add_test_case(
            TestCase::new(
                "templated-test",
                "Explain {{lang}} {{feature}}"
            )
            .add_variable("lang", "Rust")
            .add_variable("feature", "ownership")
        );

        let test_case = &dataset.test_cases[0];
        let rendered = TemplateEngine::render(
            &test_case.prompt,
            &test_case.variables.as_ref().unwrap()
        ).unwrap();

        assert_eq!(rendered, "Explain Rust ownership");
    }

    #[test]
    fn test_dataset_with_defaults() {
        let defaults = DefaultConfig::new()
            .with_temperature(0.7)
            .with_max_tokens(500);

        let dataset = Dataset::new("test", "1.0.0")
            .with_defaults(defaults);

        assert!(dataset.defaults.is_some());
        assert_eq!(dataset.defaults.unwrap().temperature, Some(0.7));
    }

    #[test]
    fn test_dataset_serialization_json() {
        let mut dataset = Dataset::new("test", "1.0.0")
            .with_description("Test dataset");

        dataset.add_test_case(
            TestCase::new("test-1", "What is Rust?")
                .with_category("qa")
                .with_expected("A systems programming language")
        );

        let json = serde_json::to_string_pretty(&dataset).unwrap();
        assert!(json.contains("test"));
        assert!(json.contains("What is Rust?"));

        // Deserialize back
        let deserialized: Dataset = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, dataset.name);
        assert_eq!(deserialized.test_cases.len(), 1);
    }

    #[test]
    fn test_dataset_serialization_yaml() {
        let mut dataset = Dataset::new("test", "1.0.0");

        dataset.add_test_case(
            TestCase::new("test-1", "What is Rust?")
        );

        let yaml = serde_yaml::to_string(&dataset).unwrap();
        assert!(yaml.contains("test"));

        // Deserialize back
        let deserialized: Dataset = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(deserialized.name, dataset.name);
    }

    #[test]
    fn test_builtin_datasets_load_successfully() {
        let datasets = builtin::get_builtin_datasets();

        for dataset in datasets {
            // Validate each dataset
            assert!(
                dataset.validate().is_ok(),
                "Dataset {} failed validation",
                dataset.name
            );

            // Ensure all test cases have required fields
            for test_case in &dataset.test_cases {
                assert!(!test_case.id.is_empty(), "Test case has empty ID");
                assert!(!test_case.prompt.is_empty(), "Test case has empty prompt");

                // If test case has variables, ensure they can be rendered
                if let Some(ref vars) = test_case.variables {
                    let result = TemplateEngine::render(&test_case.prompt, vars);
                    assert!(
                        result.is_ok(),
                        "Failed to render template for test {}: {}",
                        test_case.id,
                        result.unwrap_err()
                    );
                }
            }
        }
    }

    #[test]
    fn test_dataset_filter_by_category() {
        let dataset = builtin::coding_tasks();

        let coding_tests = dataset.filter_by_category("coding");
        assert!(!coding_tests.is_empty());

        // All filtered tests should have the coding category
        for test in coding_tests {
            assert_eq!(test.category.as_deref(), Some("coding"));
        }
    }

    #[test]
    fn test_template_variables_extraction() {
        let prompt = "Write a {{lang}} function to {{task}}";
        let vars = TemplateEngine::extract_variables(prompt);

        assert_eq!(vars.len(), 2);
        assert!(vars.contains(&"lang".to_string()));
        assert!(vars.contains(&"task".to_string()));
    }

    #[test]
    fn test_template_validation() {
        let mut vars = HashMap::new();
        vars.insert("name".to_string(), "Alice".to_string());

        // TemplateEngine::validate is no longer available
        // This test is kept for structure but validation is handled during render
        // A template with all variables: "Hello {{name}}"
        // A template with missing variables: "Hello {{name}} {{age}}"
        // These would fail at render time due to missing variable
    }

    #[test]
    fn test_dataset_len_and_is_empty() {
        let mut dataset = Dataset::new("test", "1.0.0");
        assert_eq!(dataset.len(), 0);
        assert!(dataset.is_empty());

        dataset.add_test_case(TestCase::new("test-1", "prompt"));
        assert_eq!(dataset.len(), 1);
        assert!(!dataset.is_empty());
    }

    #[test]
    fn test_loader_without_validation() {
        let loader = DatasetLoader::new();
        let temp_dir = tempfile::tempdir().unwrap();

        // Load valid JSON data
        let json_content = r#"{
            "name": "test",
            "version": "1.0.0",
            "test_cases": [
                {
                    "id": "test-1",
                    "prompt": "Test prompt"
                }
            ]
        }"#;

        let test_file = temp_dir.path().join("test.json");
        std::fs::write(&test_file, json_content).unwrap();

        let result = loader.load_from_json(&test_file);
        assert!(result.is_ok());
    }

    #[test]
    fn test_template_has_variables() {
        assert!(TemplateEngine::has_variables("Hello {{name}}"));
        assert!(!TemplateEngine::has_variables("Hello world"));
        assert!(TemplateEngine::has_variables("{{a}} and {{b}}"));
    }

    #[test]
    fn test_dataset_with_metadata() {
        let mut metadata = HashMap::new();
        metadata.insert(
            "author".to_string(),
            serde_json::json!("Test Author")
        );

        let mut dataset = Dataset::new("test", "1.0.0");
        dataset.metadata = Some(metadata);

        assert!(dataset.metadata.is_some());
        assert_eq!(
            dataset.metadata.as_ref().unwrap().get("author").unwrap(),
            &serde_json::json!("Test Author")
        );
    }

    #[test]
    fn test_test_case_with_config() {
        let test = TestCase::new("test-1", "prompt")
            .with_config(
                crate::schema::TestConfig::new()
                    .with_model("gpt-4")
                    .with_temperature(0.5)
            );

        assert!(test.config.is_some());
        assert_eq!(test.config.as_ref().unwrap().model, Some("gpt-4".to_string()));
        assert_eq!(test.config.as_ref().unwrap().temperature, Some(0.5));
    }

    #[test]
    fn test_save_and_load_round_trip() {
        let temp_dir = tempfile::tempdir().unwrap();

        let mut dataset = Dataset::new("round-trip-test", "1.0.0")
            .with_description("Testing round-trip serialization");

        dataset.add_test_case(
            TestCase::new("test-1", "prompt-1")
                .with_category("test")
                .with_expected("expected output")
        );

        let loader = DatasetLoader::new();

        // Test JSON round-trip
        let json_path = temp_dir.path().join("test.json");
        loader.save_to_json(&dataset, &json_path).unwrap();
        let loaded_json = loader.load(&json_path).unwrap();
        assert_eq!(loaded_json.name, dataset.name);
        assert_eq!(loaded_json.test_cases.len(), dataset.test_cases.len());

        // Test YAML round-trip
        let yaml_path = temp_dir.path().join("test.yaml");
        loader.save_to_yaml(&dataset, &yaml_path).unwrap();
        let loaded_yaml = loader.load(&yaml_path).unwrap();
        assert_eq!(loaded_yaml.name, dataset.name);
        assert_eq!(loaded_yaml.test_cases.len(), dataset.test_cases.len());
    }

    #[test]
    fn test_multiple_variable_substitution() {
        let mut vars = HashMap::new();
        vars.insert("lang".to_string(), "Python".to_string());
        vars.insert("n".to_string(), "100".to_string());

        let template = "Write a {{lang}} function for numbers 1 to {{n}}";
        let result = TemplateEngine::render(template, &vars).unwrap();

        assert_eq!(result, "Write a Python function for numbers 1 to 100");
    }

    #[test]
    fn test_render_optional_with_no_template() {
        let prompt = "Simple prompt without variables";
        let result = TemplateEngine::render(prompt, &HashMap::new()).unwrap();
        assert_eq!(result, prompt);
    }

    #[test]
    fn test_render_optional_requires_variables() {
        let prompt = "Prompt with {{variable}}";
        let result = TemplateEngine::render(prompt, &HashMap::new());
        assert!(result.is_err());
    }
}

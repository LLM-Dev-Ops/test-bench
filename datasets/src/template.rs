// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Template engine for prompt variable substitution

use crate::DatasetError;
use regex::Regex;
use std::collections::HashMap;

/// Template engine for rendering prompts with variable substitution
pub struct TemplateEngine;

impl TemplateEngine {
    /// Render a prompt template with variables
    ///
    /// Variables are specified using `{{variable_name}}` syntax.
    ///
    /// # Arguments
    ///
    /// * `template` - The template string containing {{variable}} placeholders
    /// * `variables` - HashMap of variable names to their values
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::HashMap;
    /// use llm_test_bench_datasets::template::TemplateEngine;
    ///
    /// let mut vars = HashMap::new();
    /// vars.insert("name".to_string(), "Alice".to_string());
    /// vars.insert("lang".to_string(), "Rust".to_string());
    ///
    /// let result = TemplateEngine::render(
    ///     "Hello {{name}}, welcome to {{lang}}!",
    ///     &vars
    /// ).unwrap();
    ///
    /// assert_eq!(result, "Hello Alice, welcome to Rust!");
    /// ```
    pub fn render(template: &str, variables: &HashMap<String, String>) -> Result<String, DatasetError> {
        let re = Regex::new(r"\{\{(\w+)\}\}").unwrap();
        let mut result = template.to_string();
        let mut missing_vars = Vec::new();

        // Find all variables in the template
        for caps in re.captures_iter(template) {
            let var_name = &caps[1];

            if let Some(value) = variables.get(var_name) {
                result = result.replace(&format!("{{{{{}}}}}", var_name), value);
            } else {
                missing_vars.push(var_name.to_string());
            }
        }

        // Check for missing variables
        if !missing_vars.is_empty() {
            return Err(DatasetError::TemplateError(format!(
                "Missing variables: {}",
                missing_vars.join(", ")
            )));
        }

        Ok(result)
    }

    /// Extract variable names from a template
    ///
    /// # Examples
    ///
    /// ```
    /// use llm_test_bench_datasets::template::TemplateEngine;
    ///
    /// let vars = TemplateEngine::extract_variables("Hello {{name}}, use {{lang}}!");
    /// assert_eq!(vars, vec!["name".to_string(), "lang".to_string()]);
    /// ```
    pub fn extract_variables(template: &str) -> Vec<String> {
        let re = Regex::new(r"\{\{(\w+)\}\}").unwrap();
        re.captures_iter(template)
            .map(|caps| caps[1].to_string())
            .collect()
    }

    /// Check if a template has any variables
    pub fn has_variables(template: &str) -> bool {
        template.contains("{{") && template.contains("}}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_simple() {
        let mut vars = HashMap::new();
        vars.insert("name".to_string(), "Alice".to_string());

        let result = TemplateEngine::render("Hello, {{name}}!", &vars).unwrap();
        assert_eq!(result, "Hello, Alice!");
    }

    #[test]
    fn test_render_multiple_variables() {
        let mut vars = HashMap::new();
        vars.insert("lang".to_string(), "Rust".to_string());
        vars.insert("feature".to_string(), "ownership".to_string());

        let result = TemplateEngine::render(
            "Explain {{lang}} {{feature}} in detail.",
            &vars
        ).unwrap();
        assert_eq!(result, "Explain Rust ownership in detail.");
    }

    #[test]
    fn test_render_same_variable_multiple_times() {
        let mut vars = HashMap::new();
        vars.insert("word".to_string(), "test".to_string());

        let result = TemplateEngine::render(
            "{{word}} {{word}} {{word}}",
            &vars
        ).unwrap();
        assert_eq!(result, "test test test");
    }

    #[test]
    fn test_render_missing_variable() {
        let vars = HashMap::new();
        let result = TemplateEngine::render("Hello, {{name}}!", &vars);
        assert!(result.is_err());

        if let Err(DatasetError::TemplateError(msg)) = result {
            assert!(msg.contains("Missing variables"));
            assert!(msg.contains("name"));
        } else {
            panic!("Expected TemplateError");
        }
    }

    #[test]
    fn test_render_no_variables() {
        let vars = HashMap::new();
        let result = TemplateEngine::render("Hello, world!", &vars).unwrap();
        assert_eq!(result, "Hello, world!");
    }

    #[test]
    fn test_extract_variables() {
        let vars = TemplateEngine::extract_variables(
            "Write a {{lang}} function to {{task}}"
        );
        assert_eq!(vars.len(), 2);
        assert!(vars.contains(&"lang".to_string()));
        assert!(vars.contains(&"task".to_string()));
    }

    #[test]
    fn test_extract_variables_none() {
        let vars = TemplateEngine::extract_variables("No variables here");
        assert_eq!(vars.len(), 0);
    }

    #[test]
    fn test_extract_variables_duplicate() {
        let vars = TemplateEngine::extract_variables(
            "{{word}} and {{word}} again"
        );
        assert_eq!(vars.len(), 2); // Duplicates included
    }

    #[test]
    fn test_has_variables() {
        assert!(TemplateEngine::has_variables("Hello {{name}}"));
        assert!(!TemplateEngine::has_variables("Hello world"));
        assert!(!TemplateEngine::has_variables("{{incomplete"));
        assert!(!TemplateEngine::has_variables("incomplete}}"));
    }
}

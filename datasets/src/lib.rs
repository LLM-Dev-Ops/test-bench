// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! # LLM Test Bench Datasets
//!
//! This crate provides comprehensive dataset management and built-in benchmark datasets
//! for the LLM Test Bench framework.
//!
//! ## Features
//!
//! - **Schema validation**: Comprehensive dataset schema with serde_valid
//! - **Multiple formats**: Load datasets from JSON and YAML files
//! - **Template engine**: Variable substitution in prompts using {{variable}} syntax
//! - **Built-in datasets**: 5+ ready-to-use benchmark datasets
//! - **Type safety**: Strongly-typed Rust structures with validation
//!
//! ## Modules
//!
//! - `schema`: Dataset schema definitions with validation
//! - `loader`: Dataset loading and saving (JSON/YAML)
//! - `template`: Template engine for variable substitution
//! - `builtin`: Built-in benchmark datasets
//!
//! ## Example
//!
//! ```no_run
//! use llm_test_bench_datasets::loader::DatasetLoader;
//! use llm_test_bench_datasets::template::TemplateEngine;
//! use std::path::Path;
//!
//! // Load a dataset
//! let loader = DatasetLoader::new();
//! let dataset = loader.load(Path::new("datasets/coding-tasks.json")).unwrap();
//!
//! // Render a templated prompt
//! let test_case = &dataset.test_cases[0];
//! if let Some(ref vars) = test_case.variables {
//!     let prompt = TemplateEngine::render(&test_case.prompt, vars).unwrap();
//!     println!("Rendered prompt: {}", prompt);
//! }
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]
#![deny(clippy::correctness)]

pub mod schema;
pub mod loader;
pub mod template;
pub mod builtin;

#[cfg(test)]
mod tests;

// Re-export main types for convenience
pub use schema::{Dataset, TestCase, DefaultConfig, TestConfig};

use thiserror::Error;

/// Dataset errors
#[derive(Error, Debug)]
pub enum DatasetError {
    /// Dataset not found
    #[error("Dataset not found: {0}")]
    NotFound(String),

    /// Invalid dataset format
    #[error("Invalid dataset format: {0}")]
    InvalidFormat(String),

    /// I/O error
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    /// YAML serialization error
    #[error("YAML error: {0}")]
    YamlError(#[from] serde_yaml::Error),

    /// Template error
    #[error("Template error: {0}")]
    TemplateError(String),

    /// Validation error
    #[error("Validation error: {0}")]
    ValidationError(String),
}

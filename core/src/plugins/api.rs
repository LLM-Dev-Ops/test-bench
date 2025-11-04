// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Plugin API definitions and traits.

use serde::{Deserialize, Serialize};
use async_trait::async_trait;
use anyhow::Result;

/// Plugin input data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInput {
    /// Input data as JSON
    pub data: serde_json::Value,

    /// Additional context
    #[serde(default)]
    pub context: PluginContext,
}

/// Plugin execution context
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PluginContext {
    /// Request ID for tracking
    pub request_id: Option<String>,

    /// Additional metadata
    #[serde(default)]
    pub metadata: std::collections::HashMap<String, String>,
}

/// Plugin output data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginOutput {
    /// Output data as JSON
    pub data: serde_json::Value,

    /// Execution metadata
    #[serde(default)]
    pub metadata: OutputMetadata,
}

/// Output metadata
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OutputMetadata {
    /// Execution time in milliseconds
    pub execution_time_ms: u64,

    /// Memory used in bytes
    pub memory_used_bytes: Option<usize>,

    /// Additional metrics
    #[serde(default)]
    pub metrics: std::collections::HashMap<String, f64>,
}

/// Plugin result type
pub type PluginResult<T> = Result<T, PluginError>;

/// Plugin error (distinct from PluginError in types.rs for API-level errors)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginError {
    pub message: String,
    pub code: String,
}

impl std::fmt::Display for PluginError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for PluginError {}

/// Base plugin API trait
#[async_trait]
pub trait PluginApi: Send + Sync {
    /// Get plugin name
    fn name(&self) -> &str;

    /// Get plugin version
    fn version(&self) -> &str;

    /// Initialize the plugin
    async fn initialize(&mut self, config: serde_json::Value) -> Result<()>;

    /// Execute the plugin
    async fn execute(&self, input: PluginInput) -> Result<PluginOutput>;

    /// Shutdown the plugin
    async fn shutdown(&mut self) -> Result<()>;
}

/// Evaluator plugin trait
#[async_trait]
pub trait EvaluatorPlugin: PluginApi {
    /// Evaluate an LLM output
    async fn evaluate(
        &self,
        input: &str,
        output: &str,
        expected: Option<&str>,
    ) -> Result<EvaluationResult>;
}

/// Evaluation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationResult {
    /// Evaluation score (0.0 to 1.0)
    pub score: f64,

    /// Detailed scores by metric
    #[serde(default)]
    pub metrics: std::collections::HashMap<String, f64>,

    /// Explanation of the evaluation
    pub explanation: Option<String>,

    /// Confidence in the evaluation (0.0 to 1.0)
    pub confidence: Option<f64>,
}

/// Provider plugin trait
#[async_trait]
pub trait ProviderPlugin: PluginApi {
    /// Complete a prompt
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse>;

    /// Get supported models
    fn supported_models(&self) -> Vec<String>;
}

/// Completion request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionRequest {
    /// Model identifier
    pub model: String,

    /// Input prompt or messages
    pub prompt: String,

    /// Maximum tokens to generate
    pub max_tokens: Option<usize>,

    /// Temperature (0.0 to 2.0)
    pub temperature: Option<f32>,

    /// Top-p sampling
    pub top_p: Option<f32>,

    /// Stop sequences
    #[serde(default)]
    pub stop: Vec<String>,
}

/// Completion response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionResponse {
    /// Generated text
    pub text: String,

    /// Token usage
    pub usage: Option<TokenUsage>,

    /// Model used
    pub model: String,

    /// Finish reason
    pub finish_reason: Option<String>,
}

/// Token usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub total_tokens: u64,
}

/// Transform plugin trait
#[async_trait]
pub trait TransformPlugin: PluginApi {
    /// Transform input data
    async fn transform(&self, data: serde_json::Value) -> Result<serde_json::Value>;
}

/// Filter plugin trait
#[async_trait]
pub trait FilterPlugin: PluginApi {
    /// Filter results based on criteria
    async fn filter(&self, items: Vec<serde_json::Value>) -> Result<Vec<serde_json::Value>>;
}

/// WASM plugin interface (exported from WASM module)
pub mod wasm_interface {
    use super::*;

    /// Plugin initialization function signature
    /// extern "C" fn plugin_init(config_ptr: *const u8, config_len: usize) -> i32
    pub const PLUGIN_INIT: &str = "plugin_init";

    /// Plugin execution function signature
    /// extern "C" fn plugin_execute(input_ptr: *const u8, input_len: usize, output_ptr: *mut u8, output_len: *mut usize) -> i32
    pub const PLUGIN_EXECUTE: &str = "plugin_execute";

    /// Plugin shutdown function signature
    /// extern "C" fn plugin_shutdown() -> i32
    pub const PLUGIN_SHUTDOWN: &str = "plugin_shutdown";

    /// Get plugin metadata function signature
    /// extern "C" fn plugin_metadata(output_ptr: *mut u8, output_len: *mut usize) -> i32
    pub const PLUGIN_METADATA: &str = "plugin_metadata";

    /// Memory allocation function (for host to allocate in plugin)
    /// extern "C" fn plugin_alloc(size: usize) -> *mut u8
    pub const PLUGIN_ALLOC: &str = "plugin_alloc";

    /// Memory deallocation function
    /// extern "C" fn plugin_free(ptr: *mut u8, size: usize)
    pub const PLUGIN_FREE: &str = "plugin_free";

    /// Result codes
    pub const RESULT_OK: i32 = 0;
    pub const RESULT_ERROR: i32 = -1;
    pub const RESULT_INVALID_INPUT: i32 = -2;
    pub const RESULT_TIMEOUT: i32 = -3;
    pub const RESULT_OUT_OF_MEMORY: i32 = -4;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_input() {
        let input = PluginInput {
            data: serde_json::json!({"key": "value"}),
            context: PluginContext::default(),
        };

        assert!(input.data.is_object());
    }

    #[test]
    fn test_evaluation_result() {
        let result = EvaluationResult {
            score: 0.85,
            metrics: [("accuracy".to_string(), 0.9)].iter().cloned().collect(),
            explanation: Some("Good result".to_string()),
            confidence: Some(0.95),
        };

        assert_eq!(result.score, 0.85);
        assert_eq!(result.metrics.get("accuracy"), Some(&0.9));
    }

    #[test]
    fn test_completion_request() {
        let request = CompletionRequest {
            model: "gpt-4".to_string(),
            prompt: "Hello, world!".to_string(),
            max_tokens: Some(100),
            temperature: Some(0.7),
            top_p: Some(0.9),
            stop: vec![],
        };

        assert_eq!(request.model, "gpt-4");
        assert_eq!(request.max_tokens, Some(100));
    }
}

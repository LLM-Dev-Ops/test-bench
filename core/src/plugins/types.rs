// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Core plugin types and structures.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

/// Plugin type identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PluginType {
    /// Evaluation metric plugin
    Evaluator,
    /// LLM provider plugin
    Provider,
    /// Data transformation plugin
    Transform,
    /// Result filtering plugin
    Filter,
    /// Custom plugin type
    Custom,
}

/// Plugin metadata embedded in the WASM module
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    /// Plugin name
    pub name: String,

    /// Plugin version (semver)
    pub version: String,

    /// Plugin description
    pub description: String,

    /// Plugin author
    pub author: String,

    /// Plugin type
    pub plugin_type: PluginType,

    /// Required capabilities
    pub capabilities: Vec<PluginCapability>,

    /// Plugin API version
    pub api_version: String,

    /// Minimum host version required
    pub min_host_version: Option<String>,

    /// Plugin homepage/repository
    pub homepage: Option<String>,

    /// Plugin license
    pub license: Option<String>,

    /// Custom metadata
    #[serde(default)]
    pub custom: HashMap<String, String>,
}

/// Plugin configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    /// Plugin ID
    pub id: String,

    /// Plugin metadata
    pub metadata: PluginMetadata,

    /// Plugin permissions
    pub permissions: PluginPermissions,

    /// Resource limits
    pub limits: ResourceLimits,

    /// Plugin-specific configuration
    #[serde(default)]
    pub config: HashMap<String, serde_json::Value>,
}

/// Plugin permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginPermissions {
    /// Allow filesystem access
    pub filesystem: bool,

    /// Allow network access
    pub network: bool,

    /// Allowed directories for filesystem access
    #[serde(default)]
    pub allowed_dirs: Vec<String>,

    /// Allowed hosts for network access
    #[serde(default)]
    pub allowed_hosts: Vec<String>,

    /// Allow environment variable access
    pub env_vars: bool,
}

impl Default for PluginPermissions {
    fn default() -> Self {
        Self {
            filesystem: false,
            network: false,
            allowed_dirs: Vec::new(),
            allowed_hosts: Vec::new(),
            env_vars: false,
        }
    }
}

/// Resource limits for plugin execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum memory in bytes
    pub max_memory_bytes: usize,

    /// Maximum execution time in milliseconds
    pub max_execution_time_ms: u64,

    /// Maximum CPU instructions
    pub max_instructions: Option<u64>,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory_bytes: 64 * 1024 * 1024, // 64 MB
            max_execution_time_ms: 30_000,       // 30 seconds
            max_instructions: Some(1_000_000_000), // 1 billion instructions
        }
    }
}

/// Plugin information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    /// Plugin ID
    pub id: String,

    /// Plugin metadata
    pub metadata: PluginMetadata,

    /// Plugin status
    pub status: PluginStatus,

    /// Load timestamp
    pub loaded_at: DateTime<Utc>,

    /// Last execution timestamp
    pub last_executed: Option<DateTime<Utc>>,

    /// Execution count
    pub execution_count: u64,

    /// Total execution time (ms)
    pub total_execution_time_ms: u64,

    /// Error count
    pub error_count: u64,
}

/// Plugin status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PluginStatus {
    /// Plugin is loaded and ready
    Ready,
    /// Plugin is currently executing
    Executing,
    /// Plugin has an error
    Error,
    /// Plugin is being unloaded
    Unloading,
}

/// Plugin capabilities
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PluginCapability {
    /// Can evaluate text outputs
    TextEvaluation,
    /// Can evaluate multi-modal outputs
    MultiModalEvaluation,
    /// Can make API requests
    ApiRequests,
    /// Can transform data
    DataTransform,
    /// Can filter results
    ResultFilter,
    /// Can stream responses
    Streaming,
    /// Can access filesystem
    FileSystem,
    /// Can access network
    Network,
}

/// Plugin ABI version
pub const PLUGIN_ABI_VERSION: &str = "1.0.0";

/// Plugin error types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PluginError {
    /// Plugin not found
    NotFound { plugin_id: String },
    /// Plugin already loaded
    AlreadyLoaded { plugin_id: String },
    /// Invalid plugin format
    InvalidFormat { reason: String },
    /// Incompatible API version
    IncompatibleApi {
        plugin_version: String,
        host_version: String,
    },
    /// Permission denied
    PermissionDenied { operation: String },
    /// Resource limit exceeded
    ResourceLimitExceeded { resource: String, limit: String },
    /// Execution error
    ExecutionError { message: String },
    /// Timeout
    Timeout { duration_ms: u64 },
    /// Invalid input
    InvalidInput { message: String },
    /// Invalid output
    InvalidOutput { message: String },
}

impl std::fmt::Display for PluginError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound { plugin_id } => write!(f, "Plugin not found: {}", plugin_id),
            Self::AlreadyLoaded { plugin_id } => write!(f, "Plugin already loaded: {}", plugin_id),
            Self::InvalidFormat { reason } => write!(f, "Invalid plugin format: {}", reason),
            Self::IncompatibleApi {
                plugin_version,
                host_version,
            } => write!(
                f,
                "Incompatible API version: plugin={}, host={}",
                plugin_version, host_version
            ),
            Self::PermissionDenied { operation } => {
                write!(f, "Permission denied: {}", operation)
            }
            Self::ResourceLimitExceeded { resource, limit } => {
                write!(f, "Resource limit exceeded: {} (limit: {})", resource, limit)
            }
            Self::ExecutionError { message } => write!(f, "Execution error: {}", message),
            Self::Timeout { duration_ms } => write!(f, "Plugin timeout after {}ms", duration_ms),
            Self::InvalidInput { message } => write!(f, "Invalid input: {}", message),
            Self::InvalidOutput { message } => write!(f, "Invalid output: {}", message),
        }
    }
}

impl std::error::Error for PluginError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_metadata() {
        let metadata = PluginMetadata {
            name: "test-plugin".to_string(),
            version: "1.0.0".to_string(),
            description: "Test plugin".to_string(),
            author: "Test Author".to_string(),
            plugin_type: PluginType::Evaluator,
            capabilities: vec![PluginCapability::TextEvaluation],
            api_version: "1.0.0".to_string(),
            min_host_version: None,
            homepage: None,
            license: Some("MIT".to_string()),
            custom: HashMap::new(),
        };

        assert_eq!(metadata.name, "test-plugin");
        assert_eq!(metadata.plugin_type, PluginType::Evaluator);
    }

    #[test]
    fn test_default_permissions() {
        let perms = PluginPermissions::default();
        assert!(!perms.filesystem);
        assert!(!perms.network);
        assert!(!perms.env_vars);
    }

    #[test]
    fn test_default_limits() {
        let limits = ResourceLimits::default();
        assert_eq!(limits.max_memory_bytes, 64 * 1024 * 1024);
        assert_eq!(limits.max_execution_time_ms, 30_000);
    }

    #[test]
    fn test_plugin_error_display() {
        let error = PluginError::NotFound {
            plugin_id: "test-plugin".to_string(),
        };
        assert_eq!(error.to_string(), "Plugin not found: test-plugin");
    }
}

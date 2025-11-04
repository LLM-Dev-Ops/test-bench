// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! # WASM-based Plugin System
//!
//! This module provides an enterprise-grade, secure plugin system using WebAssembly.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    LLM Test Bench Host                      │
//! │                                                             │
//! │  ┌───────────────────────────────────────────────────────┐ │
//! │  │              Plugin Manager                           │ │
//! │  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  │ │
//! │  │  │   Plugin    │  │   Plugin    │  │   Plugin    │  │ │
//! │  │  │  Registry   │  │   Loader    │  │   Sandbox   │  │ │
//! │  │  └─────────────┘  └─────────────┘  └─────────────┘  │ │
//! │  └───────────────────────────────────────────────────────┘ │
//! │                            │                                │
//! │                            ▼                                │
//! │  ┌───────────────────────────────────────────────────────┐ │
//! │  │              WASM Runtime (wasmtime)                  │ │
//! │  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  │ │
//! │  │  │   Plugin A  │  │   Plugin B  │  │   Plugin C  │  │ │
//! │  │  │  (sandboxed)│  │  (sandboxed)│  │  (sandboxed)│  │ │
//! │  │  └─────────────┘  └─────────────┘  └─────────────┘  │ │
//! │  └───────────────────────────────────────────────────────┘ │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Features
//!
//! - **Security**: Sandboxed execution with resource limits
//! - **Performance**: Native-speed execution with WASM
//! - **Portability**: Write once, run anywhere
//! - **Type Safety**: Strongly-typed plugin API
//! - **Hot Reloading**: Load/unload plugins at runtime
//!
//! ## Plugin Types
//!
//! ### Evaluator Plugins
//! Custom evaluation metrics for LLM outputs.
//!
//! ### Provider Plugins
//! Custom LLM provider integrations.
//!
//! ### Transform Plugins
//! Data transformation and preprocessing.
//!
//! ### Filter Plugins
//! Result filtering and post-processing.
//!
//! ## Usage
//!
//! ```rust,no_run
//! use llm_test_bench_core::plugins::{PluginManager, PluginConfig};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Initialize plugin manager
//!     let mut manager = PluginManager::new()?;
//!
//!     // Load a plugin
//!     let plugin_id = manager.load_plugin("path/to/plugin.wasm").await?;
//!
//!     // Execute plugin
//!     let result = manager.execute_plugin(&plugin_id, input).await?;
//!
//!     // Unload plugin
//!     manager.unload_plugin(&plugin_id).await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Security
//!
//! All plugins run in a sandboxed WebAssembly environment with:
//! - Memory limits
//! - CPU time limits
//! - No direct filesystem access (WASI controlled)
//! - No network access by default
//! - Capability-based security

pub mod types;
pub mod api;
pub mod runtime;
pub mod manager;
pub mod loader;
pub mod sandbox;
pub mod registry;
pub mod host;

pub use types::{
    PluginType, PluginMetadata, PluginConfig, PluginPermissions,
    PluginInfo, PluginStatus, PluginCapability,
};

pub use api::{
    PluginApi, EvaluatorPlugin, ProviderPlugin, TransformPlugin, FilterPlugin,
    PluginInput, PluginOutput, PluginResult,
};

pub use runtime::{WasmRuntime, RuntimeConfig, RuntimeLimits};
pub use manager::{PluginManager, ManagerConfig};
pub use loader::{PluginLoader, LoaderConfig};
pub use sandbox::{PluginSandbox, SandboxConfig};
pub use registry::{PluginRegistry, RegistryConfig};
pub use host::{HostFunctions, HostContext};

use anyhow::Result;
use std::sync::Arc;
use parking_lot::RwLock;

/// Main plugin system coordinator
pub struct PluginSystem {
    manager: Arc<PluginManager>,
    registry: Arc<RwLock<PluginRegistry>>,
}

impl PluginSystem {
    /// Create a new plugin system
    pub fn new() -> Result<Self> {
        let manager = Arc::new(PluginManager::new(ManagerConfig::default())?);
        let registry = Arc::new(RwLock::new(PluginRegistry::new(RegistryConfig::default())));

        Ok(Self { manager, registry })
    }

    /// Create a plugin system with custom configuration
    pub fn with_config(manager_config: ManagerConfig, registry_config: RegistryConfig) -> Result<Self> {
        let manager = Arc::new(PluginManager::new(manager_config)?);
        let registry = Arc::new(RwLock::new(PluginRegistry::new(registry_config)));

        Ok(Self { manager, registry })
    }

    /// Load a plugin from a file
    pub async fn load_plugin(&self, path: impl AsRef<std::path::Path>) -> Result<String> {
        let plugin_id = self.manager.load_plugin(path).await?;

        // Get plugin info and register it
        if let Some(info) = self.manager.get_plugin_info(&plugin_id).await {
            let mut registry = self.registry.write();
            registry.register_plugin(info)?;
        }

        Ok(plugin_id)
    }

    /// Load a plugin from bytes
    pub async fn load_plugin_from_bytes(&self, name: String, bytes: Vec<u8>) -> Result<String> {
        let plugin_id = self.manager.load_plugin_from_bytes(name, bytes).await?;

        // Get plugin info and register it
        if let Some(info) = self.manager.get_plugin_info(&plugin_id).await {
            let mut registry = self.registry.write();
            registry.register_plugin(info)?;
        }

        Ok(plugin_id)
    }

    /// Unload a plugin
    pub async fn unload_plugin(&self, plugin_id: &str) -> Result<()> {
        self.manager.unload_plugin(plugin_id).await?;

        let mut registry = self.registry.write();
        registry.unregister_plugin(plugin_id);

        Ok(())
    }

    /// Execute a plugin
    pub async fn execute_plugin(
        &self,
        plugin_id: &str,
        input: PluginInput,
    ) -> Result<PluginOutput> {
        self.manager.execute_plugin(plugin_id, input).await
    }

    /// List all loaded plugins
    pub async fn list_plugins(&self) -> Vec<PluginInfo> {
        self.manager.list_plugins().await
    }

    /// Get plugin info
    pub async fn get_plugin_info(&self, plugin_id: &str) -> Option<PluginInfo> {
        self.manager.get_plugin_info(plugin_id).await
    }

    /// Find plugins by type
    pub fn find_plugins_by_type(&self, plugin_type: PluginType) -> Vec<PluginInfo> {
        self.registry.read().find_by_type(plugin_type)
    }

    /// Find plugins by capability
    pub fn find_plugins_by_capability(&self, capability: PluginCapability) -> Vec<PluginInfo> {
        self.registry.read().find_by_capability(capability)
    }

    /// Get the plugin manager
    pub fn manager(&self) -> Arc<PluginManager> {
        self.manager.clone()
    }

    /// Get the plugin registry
    pub fn registry(&self) -> Arc<RwLock<PluginRegistry>> {
        self.registry.clone()
    }
}

impl Default for PluginSystem {
    fn default() -> Self {
        Self::new().expect("Failed to create default plugin system")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_system_creation() {
        let system = PluginSystem::new();
        assert!(system.is_ok());
    }

    #[tokio::test]
    async fn test_plugin_system_list_empty() {
        let system = PluginSystem::new().unwrap();
        let plugins = system.list_plugins().await;
        assert_eq!(plugins.len(), 0);
    }
}

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Plugin manager for loading and executing plugins.

use anyhow::{Result, Context, bail};
use std::collections::HashMap;
use std::sync::Arc;
use std::path::Path;
use parking_lot::RwLock;
use chrono::Utc;

use crate::plugins::{
    types::{
        PluginError, PluginInfo, PluginMetadata, PluginConfig, PluginPermissions, PluginStatus,
    },
    api::{PluginApi, PluginInput, PluginOutput, wasm_interface},
    runtime::{WasmRuntime, RuntimeConfig, WasmInstance},
    loader::PluginLoader,
};

/// Plugin manager configuration
#[derive(Debug, Clone)]
pub struct ManagerConfig {
    /// Runtime configuration
    pub runtime_config: RuntimeConfig,

    /// Maximum number of concurrent plugins
    pub max_concurrent_plugins: usize,

    /// Plugin cache directory
    pub cache_dir: Option<std::path::PathBuf>,
}

impl Default for ManagerConfig {
    fn default() -> Self {
        Self {
            runtime_config: RuntimeConfig::default(),
            max_concurrent_plugins: 100,
            cache_dir: None,
        }
    }
}

/// Loaded plugin state
struct LoadedPlugin {
    info: PluginInfo,
    instance: Arc<WasmInstance>,
    config: PluginConfig,
}

/// Plugin manager
pub struct PluginManager {
    config: ManagerConfig,
    runtime: Arc<WasmRuntime>,
    loader: PluginLoader,
    plugins: Arc<RwLock<HashMap<String, LoadedPlugin>>>,
}

impl PluginManager {
    /// Create a new plugin manager
    pub fn new(config: ManagerConfig) -> Result<Self> {
        let runtime = Arc::new(WasmRuntime::new(config.runtime_config.clone())?);
        let loader = PluginLoader::new();

        Ok(Self {
            config,
            runtime,
            loader,
            plugins: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Load a plugin from a file
    pub async fn load_plugin(&self, path: impl AsRef<Path>) -> Result<String> {
        let path = path.as_ref();
        let wasm_bytes = tokio::fs::read(path)
            .await
            .context("Failed to read plugin file")?;

        let plugin_name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        self.load_plugin_from_bytes(plugin_name, wasm_bytes).await
    }

    /// Load a plugin from bytes
    pub async fn load_plugin_from_bytes(&self, name: String, wasm_bytes: Vec<u8>) -> Result<String> {
        // Check concurrent plugin limit
        {
            let plugins = self.plugins.read();
            if plugins.len() >= self.config.max_concurrent_plugins {
                bail!("Maximum concurrent plugins limit reached");
            }
        }

        // Load the WASM module
        let module = self.runtime.load_module(&wasm_bytes)
            .context("Failed to load WASM module")?;

        // Instantiate the module
        let instance = self.runtime.instantiate(&module).await
            .context("Failed to instantiate WASM module")?;

        // Get plugin metadata
        let metadata = self.get_plugin_metadata(&instance).await
            .context("Failed to get plugin metadata")?;

        // Generate plugin ID
        let plugin_id = format!("{}_{}", metadata.name, uuid::Uuid::new_v4());

        // Create plugin config
        let plugin_config = PluginConfig {
            id: plugin_id.clone(),
            metadata: metadata.clone(),
            permissions: PluginPermissions::default(),
            limits: self.config.runtime_config.limits.clone(),
            config: HashMap::new(),
        };

        // Create plugin info
        let plugin_info = PluginInfo {
            id: plugin_id.clone(),
            metadata,
            status: PluginStatus::Ready,
            loaded_at: Utc::now(),
            last_executed: None,
            execution_count: 0,
            total_execution_time_ms: 0,
            error_count: 0,
        };

        // Initialize the plugin
        self.initialize_plugin(&instance).await?;

        // Store the loaded plugin
        let loaded_plugin = LoadedPlugin {
            info: plugin_info,
            instance: Arc::new(instance),
            config: plugin_config,
        };

        let mut plugins = self.plugins.write();
        plugins.insert(plugin_id.clone(), loaded_plugin);

        tracing::info!("Plugin loaded: {}", plugin_id);

        Ok(plugin_id)
    }

    /// Unload a plugin
    pub async fn unload_plugin(&self, plugin_id: &str) -> Result<()> {
        let mut plugins = self.plugins.write();

        let plugin = plugins.get_mut(plugin_id)
            .ok_or_else(|| PluginError::NotFound {
                plugin_id: plugin_id.to_string(),
            })?;

        // Update status
        plugin.info.status = PluginStatus::Unloading;

        // Shutdown the plugin
        self.shutdown_plugin(&plugin.instance).await?;

        // Remove from loaded plugins
        plugins.remove(plugin_id);

        tracing::info!("Plugin unloaded: {}", plugin_id);

        Ok(())
    }

    /// Execute a plugin
    pub async fn execute_plugin(
        &self,
        plugin_id: &str,
        input: PluginInput,
    ) -> Result<PluginOutput> {
        let start = std::time::Instant::now();

        // Get the plugin
        let (instance, plugin_info) = {
            let plugins = self.plugins.read();
            let plugin = plugins.get(plugin_id)
                .ok_or_else(|| PluginError::NotFound {
                    plugin_id: plugin_id.to_string(),
                })?;

            (plugin.instance.clone(), plugin.info.clone())
        };

        // Serialize input
        let input_bytes = serde_json::to_vec(&input)
            .context("Failed to serialize plugin input")?;

        // Execute with timeout
        let timeout_duration = std::time::Duration::from_millis(
            self.config.runtime_config.limits.max_execution_time_ms
        );

        let result = tokio::time::timeout(
            timeout_duration,
            self.execute_plugin_internal(&instance, &input_bytes)
        ).await;

        let output_bytes = match result {
            Ok(Ok(bytes)) => bytes,
            Ok(Err(e)) => {
                self.update_plugin_error(plugin_id).await;
                return Err(e);
            }
            Err(_) => {
                self.update_plugin_error(plugin_id).await;
                bail!(PluginError::Timeout {
                    duration_ms: self.config.runtime_config.limits.max_execution_time_ms
                });
            }
        };

        // Deserialize output
        let mut output: PluginOutput = serde_json::from_slice(&output_bytes)
            .context("Failed to deserialize plugin output")?;

        // Update execution metadata
        let elapsed = start.elapsed();
        output.metadata.execution_time_ms = elapsed.as_millis() as u64;

        // Update plugin statistics
        self.update_plugin_stats(plugin_id, elapsed.as_millis() as u64).await;

        Ok(output)
    }

    /// Internal plugin execution
    async fn execute_plugin_internal(
        &self,
        instance: &WasmInstance,
        input_bytes: &[u8],
    ) -> Result<Vec<u8>> {
        // Get memory
        let memory = instance.get_memory("memory")
            .context("Plugin memory not found")?;

        // Allocate input memory in plugin
        let input_ptr = instance.allocate(input_bytes.len()).await
            .context("Failed to allocate input memory")?;

        // Write input to plugin memory
        instance.write_memory(&memory, input_ptr as usize, input_bytes)
            .context("Failed to write input to plugin memory")?;

        // Allocate output buffer (max 10MB)
        let max_output_size = 10 * 1024 * 1024;
        let output_ptr = instance.allocate(max_output_size).await
            .context("Failed to allocate output memory")?;

        // Call plugin execute function
        let args = [
            wasmtime::Val::I32(input_ptr as i32),
            wasmtime::Val::I32(input_bytes.len() as i32),
            wasmtime::Val::I32(output_ptr as i32),
            wasmtime::Val::I32(max_output_size as i32),
        ];

        let results = instance.call_function(wasm_interface::PLUGIN_EXECUTE, &args).await
            .context("Failed to call plugin execute function")?;

        // Check result code
        if let Some(wasmtime::Val::I32(code)) = results.first() {
            if *code != wasm_interface::RESULT_OK {
                bail!("Plugin execution failed with code: {}", code);
            }
        }

        // Get output length (returned in second result)
        let output_len = if let Some(wasmtime::Val::I32(len)) = results.get(1) {
            *len as usize
        } else {
            bail!("Invalid plugin output length");
        };

        // Read output from plugin memory
        let output_bytes = instance.read_memory(&memory, output_ptr as usize, output_len)
            .context("Failed to read output from plugin memory")?;

        // Free memory
        instance.free(input_ptr, input_bytes.len()).await.ok();
        instance.free(output_ptr, max_output_size).await.ok();

        Ok(output_bytes)
    }

    /// Get plugin metadata from WASM instance
    async fn get_plugin_metadata(&self, instance: &WasmInstance) -> Result<PluginMetadata> {
        // Get memory
        let memory = instance.get_memory("memory")
            .context("Plugin memory not found")?;

        // Allocate output buffer
        let max_metadata_size = 64 * 1024; // 64KB
        let output_ptr = instance.allocate(max_metadata_size).await
            .context("Failed to allocate metadata buffer")?;

        // Call metadata function
        let args = [
            wasmtime::Val::I32(output_ptr as i32),
            wasmtime::Val::I32(max_metadata_size as i32),
        ];

        let results = instance.call_function(wasm_interface::PLUGIN_METADATA, &args).await
            .context("Failed to call plugin metadata function")?;

        // Get metadata length
        let metadata_len = if let Some(wasmtime::Val::I32(len)) = results.get(1) {
            *len as usize
        } else {
            bail!("Invalid metadata length");
        };

        // Read metadata
        let metadata_bytes = instance.read_memory(&memory, output_ptr as usize, metadata_len)
            .context("Failed to read metadata")?;

        // Free memory
        instance.free(output_ptr, max_metadata_size).await.ok();

        // Deserialize metadata
        let metadata: PluginMetadata = serde_json::from_slice(&metadata_bytes)
            .context("Failed to deserialize plugin metadata")?;

        Ok(metadata)
    }

    /// Initialize a plugin
    async fn initialize_plugin(&self, instance: &WasmInstance) -> Result<()> {
        let config = serde_json::json!({});
        let config_bytes = serde_json::to_vec(&config)?;

        let memory = instance.get_memory("memory")?;
        let config_ptr = instance.allocate(config_bytes.len()).await?;

        instance.write_memory(&memory, config_ptr as usize, &config_bytes)?;

        let args = [
            wasmtime::Val::I32(config_ptr as i32),
            wasmtime::Val::I32(config_bytes.len() as i32),
        ];

        instance.call_function(wasm_interface::PLUGIN_INIT, &args).await?;
        instance.free(config_ptr, config_bytes.len()).await.ok();

        Ok(())
    }

    /// Shutdown a plugin
    async fn shutdown_plugin(&self, instance: &WasmInstance) -> Result<()> {
        instance.call_function(wasm_interface::PLUGIN_SHUTDOWN, &[]).await.ok();
        Ok(())
    }

    /// Update plugin statistics
    async fn update_plugin_stats(&self, plugin_id: &str, execution_time_ms: u64) {
        let mut plugins = self.plugins.write();
        if let Some(plugin) = plugins.get_mut(plugin_id) {
            plugin.info.execution_count += 1;
            plugin.info.total_execution_time_ms += execution_time_ms;
            plugin.info.last_executed = Some(Utc::now());
        }
    }

    /// Update plugin error count
    async fn update_plugin_error(&self, plugin_id: &str) {
        let mut plugins = self.plugins.write();
        if let Some(plugin) = plugins.get_mut(plugin_id) {
            plugin.info.error_count += 1;
            plugin.info.status = PluginStatus::Error;
        }
    }

    /// List all loaded plugins
    pub async fn list_plugins(&self) -> Vec<PluginInfo> {
        let plugins = self.plugins.read();
        plugins.values().map(|p| p.info.clone()).collect()
    }

    /// Get plugin info
    pub async fn get_plugin_info(&self, plugin_id: &str) -> Option<PluginInfo> {
        let plugins = self.plugins.read();
        plugins.get(plugin_id).map(|p| p.info.clone())
    }

    /// Get plugin count
    pub fn plugin_count(&self) -> usize {
        self.plugins.read().len()
    }
}

// Helper to generate UUID without the uuid crate
mod uuid {
    pub struct Uuid;
    impl Uuid {
        pub fn new_v4() -> String {
            use std::sync::atomic::{AtomicU64, Ordering};
            static COUNTER: AtomicU64 = AtomicU64::new(0);
            let id = COUNTER.fetch_add(1, Ordering::SeqCst);
            format!("{:016x}", id)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manager_config() {
        let config = ManagerConfig::default();
        assert_eq!(config.max_concurrent_plugins, 100);
    }

    #[tokio::test]
    async fn test_manager_creation() {
        let config = ManagerConfig::default();
        let manager = PluginManager::new(config);
        assert!(manager.is_ok());
    }

    #[tokio::test]
    async fn test_manager_empty_list() {
        let manager = PluginManager::new(ManagerConfig::default()).unwrap();
        let plugins = manager.list_plugins().await;
        assert_eq!(plugins.len(), 0);
    }
}

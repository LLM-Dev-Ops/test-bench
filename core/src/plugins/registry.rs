// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Plugin registry for tracking and discovering plugins.

use anyhow::Result;
use std::collections::HashMap;
use crate::plugins::types::*;

/// Registry configuration
#[derive(Debug, Clone)]
pub struct RegistryConfig {
    /// Enable persistent storage
    pub persistent: bool,

    /// Storage path
    pub storage_path: Option<std::path::PathBuf>,
}

impl Default for RegistryConfig {
    fn default() -> Self {
        Self {
            persistent: false,
            storage_path: None,
        }
    }
}

/// Plugin registry
pub struct PluginRegistry {
    config: RegistryConfig,
    plugins: HashMap<String, PluginInfo>,
    by_type: HashMap<PluginType, Vec<String>>,
    by_capability: HashMap<PluginCapability, Vec<String>>,
}

impl PluginRegistry {
    /// Create a new plugin registry
    pub fn new(config: RegistryConfig) -> Self {
        Self {
            config,
            plugins: HashMap::new(),
            by_type: HashMap::new(),
            by_capability: HashMap::new(),
        }
    }

    /// Register a plugin
    pub fn register_plugin(&mut self, info: PluginInfo) -> Result<()> {
        let plugin_id = info.id.clone();

        // Add to main registry
        self.plugins.insert(plugin_id.clone(), info.clone());

        // Index by type
        self.by_type
            .entry(info.metadata.plugin_type)
            .or_insert_with(Vec::new)
            .push(plugin_id.clone());

        // Index by capabilities
        for capability in &info.metadata.capabilities {
            self.by_capability
                .entry(*capability)
                .or_insert_with(Vec::new)
                .push(plugin_id.clone());
        }

        Ok(())
    }

    /// Unregister a plugin
    pub fn unregister_plugin(&mut self, plugin_id: &str) {
        if let Some(info) = self.plugins.remove(plugin_id) {
            // Remove from type index
            if let Some(plugins) = self.by_type.get_mut(&info.metadata.plugin_type) {
                plugins.retain(|id| id != plugin_id);
            }

            // Remove from capability index
            for capability in &info.metadata.capabilities {
                if let Some(plugins) = self.by_capability.get_mut(capability) {
                    plugins.retain(|id| id != plugin_id);
                }
            }
        }
    }

    /// Get plugin info
    pub fn get_plugin(&self, plugin_id: &str) -> Option<&PluginInfo> {
        self.plugins.get(plugin_id)
    }

    /// Find plugins by type
    pub fn find_by_type(&self, plugin_type: PluginType) -> Vec<PluginInfo> {
        self.by_type
            .get(&plugin_type)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.plugins.get(id).cloned())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Find plugins by capability
    pub fn find_by_capability(&self, capability: PluginCapability) -> Vec<PluginInfo> {
        self.by_capability
            .get(&capability)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.plugins.get(id).cloned())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// List all plugins
    pub fn list_all(&self) -> Vec<PluginInfo> {
        self.plugins.values().cloned().collect()
    }

    /// Get plugin count
    pub fn count(&self) -> usize {
        self.plugins.len()
    }

    /// Check if a plugin exists
    pub fn has_plugin(&self, plugin_id: &str) -> bool {
        self.plugins.contains_key(plugin_id)
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new(RegistryConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn create_test_plugin_info(id: &str, plugin_type: PluginType) -> PluginInfo {
        PluginInfo {
            id: id.to_string(),
            metadata: PluginMetadata {
                name: id.to_string(),
                version: "1.0.0".to_string(),
                description: "Test plugin".to_string(),
                author: "Test".to_string(),
                plugin_type,
                capabilities: vec![PluginCapability::TextEvaluation],
                api_version: "1.0.0".to_string(),
                min_host_version: None,
                homepage: None,
                license: None,
                custom: HashMap::new(),
            },
            status: PluginStatus::Ready,
            loaded_at: Utc::now(),
            last_executed: None,
            execution_count: 0,
            total_execution_time_ms: 0,
            error_count: 0,
        }
    }

    #[test]
    fn test_registry_creation() {
        let registry = PluginRegistry::new(RegistryConfig::default());
        assert_eq!(registry.count(), 0);
    }

    #[test]
    fn test_register_plugin() {
        let mut registry = PluginRegistry::default();
        let info = create_test_plugin_info("test-plugin", PluginType::Evaluator);

        assert!(registry.register_plugin(info).is_ok());
        assert_eq!(registry.count(), 1);
        assert!(registry.has_plugin("test-plugin"));
    }

    #[test]
    fn test_unregister_plugin() {
        let mut registry = PluginRegistry::default();
        let info = create_test_plugin_info("test-plugin", PluginType::Evaluator);

        registry.register_plugin(info).unwrap();
        assert_eq!(registry.count(), 1);

        registry.unregister_plugin("test-plugin");
        assert_eq!(registry.count(), 0);
        assert!(!registry.has_plugin("test-plugin"));
    }

    #[test]
    fn test_find_by_type() {
        let mut registry = PluginRegistry::default();

        let info1 = create_test_plugin_info("eval-1", PluginType::Evaluator);
        let info2 = create_test_plugin_info("eval-2", PluginType::Evaluator);
        let info3 = create_test_plugin_info("provider-1", PluginType::Provider);

        registry.register_plugin(info1).unwrap();
        registry.register_plugin(info2).unwrap();
        registry.register_plugin(info3).unwrap();

        let evaluators = registry.find_by_type(PluginType::Evaluator);
        assert_eq!(evaluators.len(), 2);

        let providers = registry.find_by_type(PluginType::Provider);
        assert_eq!(providers.len(), 1);
    }

    #[test]
    fn test_find_by_capability() {
        let mut registry = PluginRegistry::default();
        let info = create_test_plugin_info("test-plugin", PluginType::Evaluator);

        registry.register_plugin(info).unwrap();

        let plugins = registry.find_by_capability(PluginCapability::TextEvaluation);
        assert_eq!(plugins.len(), 1);
    }
}

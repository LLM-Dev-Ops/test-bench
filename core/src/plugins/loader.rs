// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Plugin loader for discovering and loading plugins.

use anyhow::{Result, Context};
use std::path::{Path, PathBuf};

/// Plugin loader configuration
#[derive(Debug, Clone)]
pub struct LoaderConfig {
    /// Search paths for plugins
    pub search_paths: Vec<PathBuf>,

    /// File extensions to consider
    pub extensions: Vec<String>,
}

impl Default for LoaderConfig {
    fn default() -> Self {
        Self {
            search_paths: vec![
                PathBuf::from("./plugins"),
                PathBuf::from("~/.llm-test-bench/plugins"),
            ],
            extensions: vec!["wasm".to_string()],
        }
    }
}

/// Plugin loader
pub struct PluginLoader {
    config: LoaderConfig,
}

impl PluginLoader {
    /// Create a new plugin loader
    pub fn new() -> Self {
        Self {
            config: LoaderConfig::default(),
        }
    }

    /// Create a plugin loader with custom configuration
    pub fn with_config(config: LoaderConfig) -> Self {
        Self { config }
    }

    /// Discover plugins in search paths
    pub async fn discover_plugins(&self) -> Result<Vec<PathBuf>> {
        let mut plugins = Vec::new();

        for search_path in &self.config.search_paths {
            if !search_path.exists() {
                continue;
            }

            let entries = tokio::fs::read_dir(search_path)
                .await
                .context(format!("Failed to read directory: {:?}", search_path))?;

            let mut entries = entries;
            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();

                if self.is_plugin_file(&path) {
                    plugins.push(path);
                }
            }
        }

        Ok(plugins)
    }

    /// Check if a file is a plugin file
    fn is_plugin_file(&self, path: &Path) -> bool {
        if !path.is_file() {
            return false;
        }

        if let Some(ext) = path.extension() {
            if let Some(ext_str) = ext.to_str() {
                return self.config.extensions.contains(&ext_str.to_lowercase());
            }
        }

        false
    }

    /// Load plugin bytes from file
    pub async fn load_plugin_bytes(&self, path: impl AsRef<Path>) -> Result<Vec<u8>> {
        let path = path.as_ref();

        if !self.is_plugin_file(path) {
            anyhow::bail!("Not a valid plugin file: {:?}", path);
        }

        tokio::fs::read(path)
            .await
            .context(format!("Failed to read plugin file: {:?}", path))
    }

    /// Validate plugin bytes
    pub fn validate_plugin_bytes(&self, bytes: &[u8]) -> Result<()> {
        // Check if it's valid WASM
        if bytes.len() < 8 {
            anyhow::bail!("File too small to be a valid WASM module");
        }

        // Check WASM magic number
        if &bytes[0..4] != b"\0asm" {
            anyhow::bail!("Invalid WASM magic number");
        }

        // Check WASM version
        let version = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
        if version != 1 {
            anyhow::bail!("Unsupported WASM version: {}", version);
        }

        Ok(())
    }
}

impl Default for PluginLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loader_config() {
        let config = LoaderConfig::default();
        assert_eq!(config.extensions, vec!["wasm"]);
    }

    #[test]
    fn test_loader_creation() {
        let loader = PluginLoader::new();
        assert_eq!(loader.config.extensions, vec!["wasm"]);
    }

    #[test]
    fn test_validate_plugin_bytes_invalid() {
        let loader = PluginLoader::new();
        let bytes = vec![0, 1, 2, 3];
        assert!(loader.validate_plugin_bytes(&bytes).is_err());
    }

    #[test]
    fn test_validate_plugin_bytes_valid() {
        let loader = PluginLoader::new();
        // Valid WASM header
        let bytes = vec![0x00, 0x61, 0x73, 0x6D, 0x01, 0x00, 0x00, 0x00];
        assert!(loader.validate_plugin_bytes(&bytes).is_ok());
    }
}

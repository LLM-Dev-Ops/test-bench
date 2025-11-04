// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Plugin sandboxing and security enforcement.

use anyhow::Result;
use crate::plugins::types::PluginPermissions;

/// Sandbox configuration
#[derive(Debug, Clone)]
pub struct SandboxConfig {
    /// Enable strict sandboxing
    pub strict_mode: bool,

    /// Allow logging from plugins
    pub allow_logging: bool,

    /// Maximum log message size
    pub max_log_size_bytes: usize,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            strict_mode: true,
            allow_logging: true,
            max_log_size_bytes: 10 * 1024, // 10 KB
        }
    }
}

/// Plugin sandbox
pub struct PluginSandbox {
    config: SandboxConfig,
}

impl PluginSandbox {
    /// Create a new plugin sandbox
    pub fn new(config: SandboxConfig) -> Self {
        Self { config }
    }

    /// Validate plugin permissions
    pub fn validate_permissions(&self, permissions: &PluginPermissions) -> Result<()> {
        if self.config.strict_mode {
            // In strict mode, no permissions are allowed by default
            if permissions.filesystem && permissions.allowed_dirs.is_empty() {
                anyhow::bail!("Filesystem access requires explicit directory permissions");
            }

            if permissions.network && permissions.allowed_hosts.is_empty() {
                anyhow::bail!("Network access requires explicit host permissions");
            }
        }

        Ok(())
    }

    /// Check if filesystem access is allowed
    pub fn check_filesystem_access(&self, permissions: &PluginPermissions, path: &str) -> bool {
        if !permissions.filesystem {
            return false;
        }

        if permissions.allowed_dirs.is_empty() {
            // No restrictions if no allowed dirs specified (and not in strict mode)
            return !self.config.strict_mode;
        }

        // Check if path is within allowed directories
        permissions.allowed_dirs.iter().any(|allowed_dir| {
            path.starts_with(allowed_dir)
        })
    }

    /// Check if network access is allowed
    pub fn check_network_access(&self, permissions: &PluginPermissions, host: &str) -> bool {
        if !permissions.network {
            return false;
        }

        if permissions.allowed_hosts.is_empty() {
            // No restrictions if no allowed hosts specified (and not in strict mode)
            return !self.config.strict_mode;
        }

        // Check if host is in allowed hosts
        permissions.allowed_hosts.iter().any(|allowed_host| {
            host == allowed_host || host.ends_with(&format!(".{}", allowed_host))
        })
    }

    /// Sanitize log message
    pub fn sanitize_log_message(&self, message: &str) -> String {
        if message.len() > self.config.max_log_size_bytes {
            format!("{}...", &message[..self.config.max_log_size_bytes])
        } else {
            message.to_string()
        }
    }
}

impl Default for PluginSandbox {
    fn default() -> Self {
        Self::new(SandboxConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sandbox_config() {
        let config = SandboxConfig::default();
        assert!(config.strict_mode);
        assert!(config.allow_logging);
    }

    #[test]
    fn test_validate_permissions_strict() {
        let sandbox = PluginSandbox::new(SandboxConfig {
            strict_mode: true,
            ..Default::default()
        });

        let mut permissions = PluginPermissions::default();
        permissions.filesystem = true;

        // Should fail without allowed dirs
        assert!(sandbox.validate_permissions(&permissions).is_err());

        permissions.allowed_dirs = vec!["/tmp".to_string()];
        // Should succeed with allowed dirs
        assert!(sandbox.validate_permissions(&permissions).is_ok());
    }

    #[test]
    fn test_check_filesystem_access() {
        let sandbox = PluginSandbox::default();
        let mut permissions = PluginPermissions::default();
        permissions.filesystem = true;
        permissions.allowed_dirs = vec!["/tmp".to_string(), "/var/data".to_string()];

        assert!(sandbox.check_filesystem_access(&permissions, "/tmp/file.txt"));
        assert!(sandbox.check_filesystem_access(&permissions, "/var/data/test.db"));
        assert!(!sandbox.check_filesystem_access(&permissions, "/etc/passwd"));
    }

    #[test]
    fn test_check_network_access() {
        let sandbox = PluginSandbox::default();
        let mut permissions = PluginPermissions::default();
        permissions.network = true;
        permissions.allowed_hosts = vec!["api.example.com".to_string(), "example.org".to_string()];

        assert!(sandbox.check_network_access(&permissions, "api.example.com"));
        assert!(sandbox.check_network_access(&permissions, "sub.example.org"));
        assert!(!sandbox.check_network_access(&permissions, "evil.com"));
    }

    #[test]
    fn test_sanitize_log_message() {
        let sandbox = PluginSandbox::default();
        let short = "Hello, world!";
        assert_eq!(sandbox.sanitize_log_message(short), short);

        let long = "a".repeat(20000);
        let sanitized = sandbox.sanitize_log_message(&long);
        assert!(sanitized.len() < long.len());
        assert!(sanitized.ends_with("..."));
    }
}

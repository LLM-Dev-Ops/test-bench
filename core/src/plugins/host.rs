// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Host functions exposed to plugins.

use anyhow::Result;
use std::sync::Arc;
use parking_lot::Mutex;
use std::collections::HashMap;

/// Host context for plugin execution
#[derive(Clone)]
pub struct HostContext {
    inner: Arc<Mutex<HostContextInner>>,
}

struct HostContextInner {
    /// Plugin ID
    plugin_id: String,

    /// Logging enabled
    logging_enabled: bool,

    /// Log buffer
    log_buffer: Vec<String>,

    /// Shared state
    state: HashMap<String, Vec<u8>>,
}

impl HostContext {
    /// Create a new host context
    pub fn new(plugin_id: String) -> Self {
        Self {
            inner: Arc::new(Mutex::new(HostContextInner {
                plugin_id,
                logging_enabled: true,
                log_buffer: Vec::new(),
                state: HashMap::new(),
            })),
        }
    }

    /// Log a message from the plugin
    pub fn log(&self, level: LogLevel, message: String) {
        let mut inner = self.inner.lock();
        if inner.logging_enabled {
            let log_entry = format!("[{}] {}: {}", level, inner.plugin_id, message);
            inner.log_buffer.push(log_entry.clone());

            // Also log to tracing
            match level {
                LogLevel::Trace => tracing::trace!("{}", message),
                LogLevel::Debug => tracing::debug!("{}", message),
                LogLevel::Info => tracing::info!("{}", message),
                LogLevel::Warn => tracing::warn!("{}", message),
                LogLevel::Error => tracing::error!("{}", message),
            }
        }
    }

    /// Get log buffer
    pub fn get_logs(&self) -> Vec<String> {
        self.inner.lock().log_buffer.clone()
    }

    /// Clear log buffer
    pub fn clear_logs(&self) {
        self.inner.lock().log_buffer.clear();
    }

    /// Store state
    pub fn set_state(&self, key: String, value: Vec<u8>) {
        self.inner.lock().state.insert(key, value);
    }

    /// Get state
    pub fn get_state(&self, key: &str) -> Option<Vec<u8>> {
        self.inner.lock().state.get(key).cloned()
    }

    /// Remove state
    pub fn remove_state(&self, key: &str) -> Option<Vec<u8>> {
        self.inner.lock().state.remove(key)
    }
}

/// Log level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevel::Trace => write!(f, "TRACE"),
            LogLevel::Debug => write!(f, "DEBUG"),
            LogLevel::Info => write!(f, "INFO"),
            LogLevel::Warn => write!(f, "WARN"),
            LogLevel::Error => write!(f, "ERROR"),
        }
    }
}

/// Host functions exposed to plugins
pub struct HostFunctions {
    context: HostContext,
}

impl HostFunctions {
    /// Create new host functions
    pub fn new(context: HostContext) -> Self {
        Self { context }
    }

    /// Host function: Log message
    pub fn host_log(&self, level: i32, message_ptr: u32, message_len: u32) -> i32 {
        let level = match level {
            0 => LogLevel::Trace,
            1 => LogLevel::Debug,
            2 => LogLevel::Info,
            3 => LogLevel::Warn,
            4 => LogLevel::Error,
            _ => LogLevel::Info,
        };

        // Note: In a real implementation, we would read the message from plugin memory
        // For now, we'll use a placeholder
        self.context.log(level, format!("Plugin log (ptr={}, len={})", message_ptr, message_len));

        0 // Success
    }

    /// Host function: Get current time (Unix timestamp in milliseconds)
    pub fn host_current_time_ms(&self) -> i64 {
        chrono::Utc::now().timestamp_millis()
    }

    /// Host function: Random number (0-max)
    pub fn host_random(&self, max: u32) -> u32 {
        // Simple pseudo-random (in production, use a proper RNG)
        use std::sync::atomic::{AtomicU32, Ordering};
        static SEED: AtomicU32 = AtomicU32::new(12345);

        let seed = SEED.load(Ordering::Relaxed);
        let next = seed.wrapping_mul(1103515245).wrapping_add(12345);
        SEED.store(next, Ordering::Relaxed);

        (next / 65536) % max
    }

    /// Host function: Set plugin state
    pub fn host_set_state(&self, key_ptr: u32, key_len: u32, value_ptr: u32, value_len: u32) -> i32 {
        // Note: In a real implementation, we would read from plugin memory
        // For now, we'll use placeholders
        let key = format!("key_{}", key_ptr);
        let value = vec![0u8; value_len as usize];

        self.context.set_state(key, value);
        0 // Success
    }

    /// Host function: Get plugin state
    pub fn host_get_state(&self, key_ptr: u32, key_len: u32, value_ptr: u32, value_max_len: u32) -> i32 {
        // Note: In a real implementation, we would read/write from plugin memory
        let key = format!("key_{}", key_ptr);

        if let Some(value) = self.context.get_state(&key) {
            let len = value.len().min(value_max_len as usize);
            len as i32
        } else {
            -1 // Not found
        }
    }

    /// Get the context
    pub fn context(&self) -> &HostContext {
        &self.context
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_host_context() {
        let context = HostContext::new("test-plugin".to_string());

        context.log(LogLevel::Info, "Test message".to_string());
        let logs = context.get_logs();
        assert_eq!(logs.len(), 1);

        context.clear_logs();
        assert_eq!(context.get_logs().len(), 0);
    }

    #[test]
    fn test_host_context_state() {
        let context = HostContext::new("test-plugin".to_string());

        context.set_state("key1".to_string(), vec![1, 2, 3]);
        assert_eq!(context.get_state("key1"), Some(vec![1, 2, 3]));

        context.remove_state("key1");
        assert_eq!(context.get_state("key1"), None);
    }

    #[test]
    fn test_host_functions() {
        let context = HostContext::new("test-plugin".to_string());
        let host = HostFunctions::new(context.clone());

        let result = host.host_log(2, 0, 10);
        assert_eq!(result, 0);

        let time = host.host_current_time_ms();
        assert!(time > 0);

        let rand = host.host_random(100);
        assert!(rand < 100);
    }

    #[test]
    fn test_log_level_display() {
        assert_eq!(LogLevel::Info.to_string(), "INFO");
        assert_eq!(LogLevel::Error.to_string(), "ERROR");
    }
}

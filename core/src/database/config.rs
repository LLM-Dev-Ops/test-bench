// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Database configuration.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::database::{DEFAULT_POOL_SIZE, DEFAULT_CONNECT_TIMEOUT};

/// Database configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// Database host
    pub host: String,
    /// Database port
    pub port: u16,
    /// Database name
    pub database: String,
    /// Database user
    pub username: String,
    /// Database password
    pub password: String,
    /// Connection pool size
    pub pool_size: u32,
    /// Connection timeout (seconds)
    pub connect_timeout: u64,
    /// Idle timeout (seconds)
    pub idle_timeout: Option<u64>,
    /// Max lifetime (seconds)
    pub max_lifetime: Option<u64>,
    /// SSL mode
    pub ssl_mode: SslMode,
    /// Application name
    pub application_name: String,
}

/// SSL modes for PostgreSQL
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SslMode {
    /// No SSL
    Disable,
    /// Prefer SSL if available
    Prefer,
    /// Require SSL
    Require,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 5432,
            database: crate::database::DEFAULT_DATABASE_NAME.to_string(),
            username: "postgres".to_string(),
            password: "postgres".to_string(),
            pool_size: DEFAULT_POOL_SIZE,
            connect_timeout: DEFAULT_CONNECT_TIMEOUT,
            idle_timeout: Some(600), // 10 minutes
            max_lifetime: Some(1800), // 30 minutes
            ssl_mode: SslMode::Prefer,
            application_name: "llm-test-bench".to_string(),
        }
    }
}

impl DatabaseConfig {
    /// Create a new configuration builder
    pub fn builder() -> DatabaseConfigBuilder {
        DatabaseConfigBuilder::default()
    }

    /// Create configuration from environment variables
    pub fn from_env() -> Result<Self> {
        let mut config = Self::default();

        if let Ok(host) = std::env::var("DATABASE_HOST") {
            config.host = host;
        }

        if let Ok(port) = std::env::var("DATABASE_PORT") {
            config.port = port.parse()?;
        }

        if let Ok(database) = std::env::var("DATABASE_NAME") {
            config.database = database;
        }

        if let Ok(username) = std::env::var("DATABASE_USER") {
            config.username = username;
        }

        if let Ok(password) = std::env::var("DATABASE_PASSWORD") {
            config.password = password;
        }

        if let Ok(pool_size) = std::env::var("DATABASE_POOL_SIZE") {
            config.pool_size = pool_size.parse()?;
        }

        if let Ok(ssl_mode) = std::env::var("DATABASE_SSL_MODE") {
            config.ssl_mode = match ssl_mode.to_lowercase().as_str() {
                "disable" => SslMode::Disable,
                "prefer" => SslMode::Prefer,
                "require" => SslMode::Require,
                _ => SslMode::Prefer,
            };
        }

        Ok(config)
    }

    /// Get connection URL
    pub fn connection_url(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}?application_name={}",
            self.username,
            self.password,
            self.host,
            self.port,
            self.database,
            self.application_name
        )
    }

    /// Get connection URL (without password for logging)
    pub fn connection_url_safe(&self) -> String {
        format!(
            "postgres://{}:****@{}:{}/{}",
            self.username,
            self.host,
            self.port,
            self.database
        )
    }

    /// Get connection timeout as Duration
    pub fn connect_timeout_duration(&self) -> Duration {
        Duration::from_secs(self.connect_timeout)
    }

    /// Get idle timeout as Duration
    pub fn idle_timeout_duration(&self) -> Option<Duration> {
        self.idle_timeout.map(Duration::from_secs)
    }

    /// Get max lifetime as Duration
    pub fn max_lifetime_duration(&self) -> Option<Duration> {
        self.max_lifetime.map(Duration::from_secs)
    }
}

/// Database configuration builder
#[derive(Default)]
pub struct DatabaseConfigBuilder {
    config: DatabaseConfig,
}

impl DatabaseConfigBuilder {
    pub fn host(mut self, host: impl Into<String>) -> Self {
        self.config.host = host.into();
        self
    }

    pub fn port(mut self, port: u16) -> Self {
        self.config.port = port;
        self
    }

    pub fn database(mut self, database: impl Into<String>) -> Self {
        self.config.database = database.into();
        self
    }

    pub fn username(mut self, username: impl Into<String>) -> Self {
        self.config.username = username.into();
        self
    }

    pub fn password(mut self, password: impl Into<String>) -> Self {
        self.config.password = password.into();
        self
    }

    pub fn pool_size(mut self, size: u32) -> Self {
        self.config.pool_size = size;
        self
    }

    pub fn connect_timeout(mut self, seconds: u64) -> Self {
        self.config.connect_timeout = seconds;
        self
    }

    pub fn idle_timeout(mut self, seconds: u64) -> Self {
        self.config.idle_timeout = Some(seconds);
        self
    }

    pub fn max_lifetime(mut self, seconds: u64) -> Self {
        self.config.max_lifetime = Some(seconds);
        self
    }

    pub fn ssl_mode(mut self, mode: SslMode) -> Self {
        self.config.ssl_mode = mode;
        self
    }

    pub fn application_name(mut self, name: impl Into<String>) -> Self {
        self.config.application_name = name.into();
        self
    }

    pub fn build(self) -> DatabaseConfig {
        self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = DatabaseConfig::default();
        assert_eq!(config.host, "localhost");
        assert_eq!(config.port, 5432);
        assert_eq!(config.pool_size, DEFAULT_POOL_SIZE);
    }

    #[test]
    fn test_builder() {
        let config = DatabaseConfig::builder()
            .host("db.example.com")
            .port(5433)
            .database("test_db")
            .username("test_user")
            .password("test_pass")
            .pool_size(10)
            .ssl_mode(SslMode::Require)
            .build();

        assert_eq!(config.host, "db.example.com");
        assert_eq!(config.port, 5433);
        assert_eq!(config.database, "test_db");
        assert_eq!(config.pool_size, 10);
        assert_eq!(config.ssl_mode, SslMode::Require);
    }

    #[test]
    fn test_connection_url() {
        let config = DatabaseConfig::builder()
            .host("localhost")
            .port(5432)
            .database("testdb")
            .username("user")
            .password("pass")
            .build();

        let url = config.connection_url();
        assert!(url.contains("user:pass@localhost:5432/testdb"));
    }

    #[test]
    fn test_connection_url_safe() {
        let config = DatabaseConfig::builder()
            .password("secret123")
            .build();

        let url = config.connection_url_safe();
        assert!(!url.contains("secret123"));
        assert!(url.contains("****"));
    }
}

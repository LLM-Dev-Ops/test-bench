// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! API middleware components.

use tower_http::cors::{CorsLayer, Any};
use axum::http::{Method, HeaderValue};

/// CORS configuration
#[derive(Debug, Clone)]
pub struct CorsConfig {
    /// Allowed origins
    pub allowed_origins: Vec<String>,
    /// Allowed methods
    pub allowed_methods: Vec<Method>,
    /// Allowed headers
    pub allowed_headers: Vec<String>,
    /// Allow credentials
    pub allow_credentials: bool,
}

impl Default for CorsConfig {
    fn default() -> Self {
        Self {
            allowed_origins: vec!["*".to_string()],
            allowed_methods: vec![
                Method::GET,
                Method::POST,
                Method::PUT,
                Method::DELETE,
                Method::PATCH,
                Method::OPTIONS,
            ],
            allowed_headers: vec![
                "content-type".to_string(),
                "authorization".to_string(),
                "x-api-key".to_string(),
            ],
            allow_credentials: true,
        }
    }
}

impl CorsConfig {
    pub fn to_layer(&self) -> CorsLayer {
        let mut layer = CorsLayer::new()
            .allow_methods(self.allowed_methods.clone())
            .allow_headers(
                self.allowed_headers
                    .iter()
                    .map(|h| h.parse().unwrap())
                    .collect::<Vec<_>>(),
            );

        // Allow credentials if configured
        if self.allow_credentials {
            layer = layer.allow_credentials(true);
        }

        // Configure origins
        if self.allowed_origins.contains(&"*".to_string()) {
            layer = layer.allow_origin(Any);
        } else {
            layer = layer.allow_origin(
                self.allowed_origins
                    .iter()
                    .map(|o| o.parse::<HeaderValue>().unwrap())
                    .collect::<Vec<_>>(),
            );
        }

        layer
    }
}

/// Rate limiter using tower-governor
pub struct RateLimiter;

impl RateLimiter {
    /// Create a rate limiter middleware
    pub fn new(_requests_per_second: u64, _burst_size: u32) -> Self {
        // Note: Actual implementation would use tower-governor
        // Simplified for this example
        Self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cors_config_default() {
        let config = CorsConfig::default();
        assert_eq!(config.allowed_origins, vec!["*"]);
        assert!(config.allow_credentials);
    }

    #[test]
    fn test_cors_layer_creation() {
        let config = CorsConfig::default();
        let _layer = config.to_layer();
        // Layer created successfully
    }
}

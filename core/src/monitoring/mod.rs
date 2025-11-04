// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! # Real-time Monitoring System
//!
//! This module provides enterprise-grade monitoring capabilities including:
//!
//! - **Prometheus Metrics**: Industry-standard metrics export for LLM operations
//! - **WebSocket Dashboards**: Real-time streaming of metrics and events
//! - **Event System**: Publish/subscribe event bus for distributed monitoring
//! - **Live Dashboards**: Interactive HTML dashboards with real-time updates
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    LLM Test Bench Core                      │
//! └─────────────────────────────────────────────────────────────┘
//!                              │
//!                              ▼
//!                    ┌──────────────────┐
//!                    │   Event Bus      │
//!                    │  (pub/sub)       │
//!                    └──────────────────┘
//!                              │
//!                 ┌────────────┼────────────┐
//!                 ▼            ▼            ▼
//!         ┌──────────┐  ┌──────────┐  ┌──────────┐
//!         │Prometheus│  │WebSocket │  │ Metrics  │
//!         │ Exporter │  │  Server  │  │Collector │
//!         └──────────┘  └──────────┘  └──────────┘
//!                │            │            │
//!                ▼            ▼            ▼
//!         [Prometheus]  [Dashboard]  [Aggregation]
//! ```
//!
//! ## Features
//!
//! ### Prometheus Integration
//! - Counter metrics (requests, tokens, errors)
//! - Gauge metrics (active requests, queue depth)
//! - Histogram metrics (latency, cost distribution)
//! - Summary metrics (percentiles)
//!
//! ### WebSocket Streaming
//! - Real-time event streaming
//! - Live metric updates
//! - Benchmark progress tracking
//! - Connection management
//!
//! ### Dashboard
//! - Interactive charts (Chart.js)
//! - Real-time updates via WebSocket
//! - Provider comparison views
//! - Cost tracking
//!
//! ## Usage
//!
//! ```rust,no_run
//! use llm_test_bench_core::monitoring::{MonitoringSystem, MonitoringConfig};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Initialize monitoring system
//!     let config = MonitoringConfig::default()
//!         .with_prometheus_port(9090)
//!         .with_websocket_port(8080);
//!
//!     let monitoring = MonitoringSystem::new(config).await?;
//!
//!     // Start monitoring services
//!     monitoring.start().await?;
//!
//!     // Record metrics
//!     monitoring.record_request("openai", "gpt-4");
//!     monitoring.record_latency("openai", 1.5);
//!     monitoring.record_tokens("openai", 150, 50);
//!
//!     Ok(())
//! }
//! ```

pub mod metrics;
pub mod events;
pub mod prometheus;
pub mod websocket;
pub mod dashboard;
pub mod collector;
pub mod integration;

pub use metrics::{
    Metric, MetricType, MetricValue, MetricLabels,
    RequestMetric, LatencyMetric, TokenMetric, CostMetric, ErrorMetric,
};

pub use events::{
    MonitoringEvent, EventBus, EventSubscriber, EventType,
    BenchmarkEvent, ProviderEvent, EvaluationEvent,
};

pub use prometheus::{PrometheusExporter, PrometheusConfig};
pub use websocket::{WebSocketServer, WebSocketConfig, WebSocketMessage};
pub use dashboard::{Dashboard, DashboardConfig};
pub use collector::{MetricCollector, CollectorConfig};
pub use integration::{MonitoredProvider, monitor_providers};

use anyhow::Result;
use std::sync::Arc;
use parking_lot::RwLock;

/// Main monitoring system that coordinates all monitoring components
pub struct MonitoringSystem {
    config: MonitoringConfig,
    event_bus: Arc<EventBus>,
    prometheus: Arc<PrometheusExporter>,
    websocket: Arc<WebSocketServer>,
    collector: Arc<MetricCollector>,
}

/// Configuration for the monitoring system
#[derive(Debug, Clone)]
pub struct MonitoringConfig {
    /// Enable Prometheus metrics export
    pub prometheus_enabled: bool,
    /// Prometheus metrics port
    pub prometheus_port: u16,
    /// Enable WebSocket server
    pub websocket_enabled: bool,
    /// WebSocket server port
    pub websocket_port: u16,
    /// Enable dashboard
    pub dashboard_enabled: bool,
    /// Dashboard port
    pub dashboard_port: u16,
    /// Metric retention period (seconds)
    pub retention_period: u64,
    /// Enable detailed metrics (may impact performance)
    pub detailed_metrics: bool,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            prometheus_enabled: true,
            prometheus_port: 9090,
            websocket_enabled: true,
            websocket_port: 8080,
            dashboard_enabled: true,
            dashboard_port: 3000,
            retention_period: 3600, // 1 hour
            detailed_metrics: false,
        }
    }
}

impl MonitoringConfig {
    /// Create a new monitoring configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable Prometheus metrics
    pub fn with_prometheus(mut self, enabled: bool) -> Self {
        self.prometheus_enabled = enabled;
        self
    }

    /// Set Prometheus port
    pub fn with_prometheus_port(mut self, port: u16) -> Self {
        self.prometheus_port = port;
        self
    }

    /// Enable WebSocket server
    pub fn with_websocket(mut self, enabled: bool) -> Self {
        self.websocket_enabled = enabled;
        self
    }

    /// Set WebSocket port
    pub fn with_websocket_port(mut self, port: u16) -> Self {
        self.websocket_port = port;
        self
    }

    /// Enable dashboard
    pub fn with_dashboard(mut self, enabled: bool) -> Self {
        self.dashboard_enabled = enabled;
        self
    }

    /// Set dashboard port
    pub fn with_dashboard_port(mut self, port: u16) -> Self {
        self.dashboard_port = port;
        self
    }

    /// Set metric retention period
    pub fn with_retention_period(mut self, seconds: u64) -> Self {
        self.retention_period = seconds;
        self
    }

    /// Enable detailed metrics
    pub fn with_detailed_metrics(mut self, enabled: bool) -> Self {
        self.detailed_metrics = enabled;
        self
    }
}

impl MonitoringSystem {
    /// Create a new monitoring system
    pub async fn new(config: MonitoringConfig) -> Result<Self> {
        let event_bus = Arc::new(EventBus::new());

        let prometheus_config = PrometheusConfig {
            port: config.prometheus_port,
            enabled: config.prometheus_enabled,
        };
        let prometheus = Arc::new(PrometheusExporter::new(prometheus_config)?);

        let websocket_config = WebSocketConfig {
            port: config.websocket_port,
            enabled: config.websocket_enabled,
        };
        let websocket = Arc::new(WebSocketServer::new(websocket_config, event_bus.clone()).await?);

        let collector_config = CollectorConfig {
            retention_period: config.retention_period,
            detailed_metrics: config.detailed_metrics,
        };
        let collector = Arc::new(MetricCollector::new(collector_config, event_bus.clone()));

        Ok(Self {
            config,
            event_bus,
            prometheus,
            websocket,
            collector,
        })
    }

    /// Start all monitoring services
    pub async fn start(&self) -> Result<()> {
        tracing::info!("Starting monitoring system");

        if self.config.prometheus_enabled {
            self.prometheus.start().await?;
            tracing::info!("Prometheus exporter started on port {}", self.config.prometheus_port);
        }

        if self.config.websocket_enabled {
            self.websocket.start().await?;
            tracing::info!("WebSocket server started on port {}", self.config.websocket_port);
        }

        self.collector.start().await?;
        tracing::info!("Metric collector started");

        Ok(())
    }

    /// Stop all monitoring services
    pub async fn stop(&self) -> Result<()> {
        tracing::info!("Stopping monitoring system");

        if self.config.prometheus_enabled {
            self.prometheus.stop().await?;
        }

        if self.config.websocket_enabled {
            self.websocket.stop().await?;
        }

        self.collector.stop().await?;

        Ok(())
    }

    /// Record a request metric
    pub fn record_request(&self, provider: &str, model: &str) {
        let event = MonitoringEvent::request(provider, model);
        self.event_bus.publish(event);
    }

    /// Record a latency metric
    pub fn record_latency(&self, provider: &str, latency: f64) {
        let event = MonitoringEvent::latency(provider, latency);
        self.event_bus.publish(event);
    }

    /// Record token usage
    pub fn record_tokens(&self, provider: &str, input_tokens: u64, output_tokens: u64) {
        let event = MonitoringEvent::tokens(provider, input_tokens, output_tokens);
        self.event_bus.publish(event);
    }

    /// Record cost
    pub fn record_cost(&self, provider: &str, cost: f64) {
        let event = MonitoringEvent::cost(provider, cost);
        self.event_bus.publish(event);
    }

    /// Record an error
    pub fn record_error(&self, provider: &str, error_type: &str) {
        let event = MonitoringEvent::error(provider, error_type);
        self.event_bus.publish(event);
    }

    /// Get the event bus for custom subscriptions
    pub fn event_bus(&self) -> Arc<EventBus> {
        self.event_bus.clone()
    }

    /// Get current metrics snapshot
    pub async fn get_metrics(&self) -> Vec<Metric> {
        self.collector.get_metrics().await
    }

    /// Get metrics for a specific provider
    pub async fn get_provider_metrics(&self, provider: &str) -> Vec<Metric> {
        self.collector.get_provider_metrics(provider).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monitoring_config() {
        let config = MonitoringConfig::new()
            .with_prometheus_port(9091)
            .with_websocket_port(8081)
            .with_detailed_metrics(true);

        assert_eq!(config.prometheus_port, 9091);
        assert_eq!(config.websocket_port, 8081);
        assert!(config.detailed_metrics);
    }

    #[tokio::test]
    async fn test_monitoring_system_creation() {
        let config = MonitoringConfig::new()
            .with_prometheus(false)
            .with_websocket(false);

        let result = MonitoringSystem::new(config).await;
        assert!(result.is_ok());
    }
}

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Prometheus metrics exporter for LLM Test Bench.

use anyhow::Result;
use prometheus::{
    Counter, CounterVec, Gauge, GaugeVec, Histogram, HistogramVec, Opts, Registry,
    TextEncoder, Encoder, HistogramOpts,
};
use std::sync::Arc;
use parking_lot::RwLock;
use axum::{
    routing::get,
    Router,
    response::{Response, IntoResponse},
    http::StatusCode,
};
use tokio::task::JoinHandle;

/// Prometheus exporter configuration
#[derive(Debug, Clone)]
pub struct PrometheusConfig {
    /// Port to expose metrics on
    pub port: u16,
    /// Enable the exporter
    pub enabled: bool,
}

impl Default for PrometheusConfig {
    fn default() -> Self {
        Self {
            port: 9090,
            enabled: true,
        }
    }
}

/// Prometheus metrics exporter
pub struct PrometheusExporter {
    config: PrometheusConfig,
    registry: Arc<Registry>,
    metrics: Arc<PrometheusMetrics>,
    server_handle: Arc<RwLock<Option<JoinHandle<()>>>>,
}

/// Collection of Prometheus metrics
struct PrometheusMetrics {
    // Request metrics
    requests_total: CounterVec,
    requests_duration: HistogramVec,
    requests_active: GaugeVec,

    // Token metrics
    tokens_input_total: CounterVec,
    tokens_output_total: CounterVec,

    // Cost metrics
    cost_usd_total: CounterVec,

    // Error metrics
    errors_total: CounterVec,

    // Evaluation metrics
    evaluation_score: GaugeVec,

    // Benchmark metrics
    benchmark_progress: GaugeVec,
    benchmark_duration: HistogramVec,
}

impl PrometheusMetrics {
    fn new(registry: &Registry) -> Result<Self> {
        // Request metrics
        let requests_total = CounterVec::new(
            Opts::new("llm_requests_total", "Total number of LLM requests"),
            &["provider", "model", "status"],
        )?;
        registry.register(Box::new(requests_total.clone()))?;

        let requests_duration = HistogramVec::new(
            HistogramOpts::new("llm_request_duration_seconds", "Request duration in seconds")
                .buckets(vec![0.1, 0.5, 1.0, 2.0, 5.0, 10.0, 30.0, 60.0]),
            &["provider", "model"],
        )?;
        registry.register(Box::new(requests_duration.clone()))?;

        let requests_active = GaugeVec::new(
            Opts::new("llm_requests_active", "Number of active requests"),
            &["provider"],
        )?;
        registry.register(Box::new(requests_active.clone()))?;

        // Token metrics
        let tokens_input_total = CounterVec::new(
            Opts::new("llm_tokens_input_total", "Total input tokens processed"),
            &["provider", "model"],
        )?;
        registry.register(Box::new(tokens_input_total.clone()))?;

        let tokens_output_total = CounterVec::new(
            Opts::new("llm_tokens_output_total", "Total output tokens generated"),
            &["provider", "model"],
        )?;
        registry.register(Box::new(tokens_output_total.clone()))?;

        // Cost metrics
        let cost_usd_total = CounterVec::new(
            Opts::new("llm_cost_usd_total", "Total cost in USD"),
            &["provider", "model"],
        )?;
        registry.register(Box::new(cost_usd_total.clone()))?;

        // Error metrics
        let errors_total = CounterVec::new(
            Opts::new("llm_errors_total", "Total number of errors"),
            &["provider", "model", "error_type"],
        )?;
        registry.register(Box::new(errors_total.clone()))?;

        // Evaluation metrics
        let evaluation_score = GaugeVec::new(
            Opts::new("llm_evaluation_score", "Evaluation metric score"),
            &["provider", "model", "metric"],
        )?;
        registry.register(Box::new(evaluation_score.clone()))?;

        // Benchmark metrics
        let benchmark_progress = GaugeVec::new(
            Opts::new("llm_benchmark_progress", "Benchmark progress percentage"),
            &["benchmark_id", "name"],
        )?;
        registry.register(Box::new(benchmark_progress.clone()))?;

        let benchmark_duration = HistogramVec::new(
            HistogramOpts::new("llm_benchmark_duration_seconds", "Benchmark duration in seconds")
                .buckets(vec![1.0, 5.0, 10.0, 30.0, 60.0, 300.0, 600.0, 1800.0]),
            &["benchmark_id", "name"],
        )?;
        registry.register(Box::new(benchmark_duration.clone()))?;

        Ok(Self {
            requests_total,
            requests_duration,
            requests_active,
            tokens_input_total,
            tokens_output_total,
            cost_usd_total,
            errors_total,
            evaluation_score,
            benchmark_progress,
            benchmark_duration,
        })
    }

    /// Record a request
    fn record_request(&self, provider: &str, model: &str, status: &str) {
        self.requests_total
            .with_label_values(&[provider, model, status])
            .inc();
    }

    /// Record request duration
    fn record_duration(&self, provider: &str, model: &str, duration: f64) {
        self.requests_duration
            .with_label_values(&[provider, model])
            .observe(duration);
    }

    /// Set active requests
    fn set_active_requests(&self, provider: &str, count: i64) {
        self.requests_active
            .with_label_values(&[provider])
            .set(count as f64);
    }

    /// Record tokens
    fn record_tokens(&self, provider: &str, model: &str, input: u64, output: u64) {
        self.tokens_input_total
            .with_label_values(&[provider, model])
            .inc_by(input as f64);
        self.tokens_output_total
            .with_label_values(&[provider, model])
            .inc_by(output as f64);
    }

    /// Record cost
    fn record_cost(&self, provider: &str, model: &str, cost: f64) {
        self.cost_usd_total
            .with_label_values(&[provider, model])
            .inc_by(cost);
    }

    /// Record error
    fn record_error(&self, provider: &str, model: &str, error_type: &str) {
        self.errors_total
            .with_label_values(&[provider, model, error_type])
            .inc();
    }

    /// Record evaluation score
    fn record_evaluation(&self, provider: &str, model: &str, metric: &str, score: f64) {
        self.evaluation_score
            .with_label_values(&[provider, model, metric])
            .set(score);
    }

    /// Record benchmark progress
    fn record_benchmark_progress(&self, benchmark_id: &str, name: &str, progress: f64) {
        self.benchmark_progress
            .with_label_values(&[benchmark_id, name])
            .set(progress);
    }

    /// Record benchmark duration
    fn record_benchmark_duration(&self, benchmark_id: &str, name: &str, duration: f64) {
        self.benchmark_duration
            .with_label_values(&[benchmark_id, name])
            .observe(duration);
    }
}

impl PrometheusExporter {
    /// Create a new Prometheus exporter
    pub fn new(config: PrometheusConfig) -> Result<Self> {
        let registry = Registry::new();
        let metrics = PrometheusMetrics::new(&registry)?;

        Ok(Self {
            config,
            registry: Arc::new(registry),
            metrics: Arc::new(metrics),
            server_handle: Arc::new(RwLock::new(None)),
        })
    }

    /// Start the Prometheus HTTP server
    pub async fn start(&self) -> Result<()> {
        if !self.config.enabled {
            tracing::debug!("Prometheus exporter is disabled");
            return Ok(());
        }

        let registry = self.registry.clone();
        let port = self.config.port;

        let app = Router::new()
            .route("/metrics", get(move || Self::metrics_handler(registry.clone())));

        let addr = format!("0.0.0.0:{}", port).parse()?;
        tracing::info!("Starting Prometheus exporter on {}", addr);

        let server = tokio::spawn(async move {
            let listener = tokio::net::TcpListener::bind(addr)
                .await
                .expect("Failed to bind Prometheus server");

            axum::serve(listener, app)
                .await
                .expect("Prometheus server error");
        });

        let mut handle = self.server_handle.write();
        *handle = Some(server);

        Ok(())
    }

    /// Stop the Prometheus HTTP server
    pub async fn stop(&self) -> Result<()> {
        let mut handle = self.server_handle.write();
        if let Some(h) = handle.take() {
            h.abort();
        }
        Ok(())
    }

    /// Metrics endpoint handler
    async fn metrics_handler(registry: Arc<Registry>) -> Response {
        let encoder = TextEncoder::new();
        let metric_families = registry.gather();

        let mut buffer = Vec::new();
        match encoder.encode(&metric_families, &mut buffer) {
            Ok(_) => {
                Response::builder()
                    .status(StatusCode::OK)
                    .header("Content-Type", encoder.format_type())
                    .body(buffer.into())
                    .unwrap()
            }
            Err(e) => {
                tracing::error!("Failed to encode metrics: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Failed to encode metrics").into_response()
            }
        }
    }

    /// Record a request
    pub fn record_request(&self, provider: &str, model: &str, status: &str) {
        self.metrics.record_request(provider, model, status);
    }

    /// Record request duration
    pub fn record_duration(&self, provider: &str, model: &str, duration: f64) {
        self.metrics.record_duration(provider, model, duration);
    }

    /// Set active requests
    pub fn set_active_requests(&self, provider: &str, count: i64) {
        self.metrics.set_active_requests(provider, count);
    }

    /// Record tokens
    pub fn record_tokens(&self, provider: &str, model: &str, input: u64, output: u64) {
        self.metrics.record_tokens(provider, model, input, output);
    }

    /// Record cost
    pub fn record_cost(&self, provider: &str, model: &str, cost: f64) {
        self.metrics.record_cost(provider, model, cost);
    }

    /// Record error
    pub fn record_error(&self, provider: &str, model: &str, error_type: &str) {
        self.metrics.record_error(provider, model, error_type);
    }

    /// Record evaluation score
    pub fn record_evaluation(&self, provider: &str, model: &str, metric: &str, score: f64) {
        self.metrics.record_evaluation(provider, model, metric, score);
    }

    /// Record benchmark progress
    pub fn record_benchmark_progress(&self, benchmark_id: &str, name: &str, progress: f64) {
        self.metrics.record_benchmark_progress(benchmark_id, name, progress);
    }

    /// Record benchmark duration
    pub fn record_benchmark_duration(&self, benchmark_id: &str, name: &str, duration: f64) {
        self.metrics.record_benchmark_duration(benchmark_id, name, duration);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prometheus_config() {
        let config = PrometheusConfig {
            port: 9091,
            enabled: true,
        };
        assert_eq!(config.port, 9091);
        assert!(config.enabled);
    }

    #[tokio::test]
    async fn test_prometheus_exporter_creation() {
        let config = PrometheusConfig {
            port: 9092,
            enabled: false,
        };
        let exporter = PrometheusExporter::new(config);
        assert!(exporter.is_ok());
    }

    #[tokio::test]
    async fn test_record_metrics() {
        let config = PrometheusConfig {
            port: 9093,
            enabled: false,
        };
        let exporter = PrometheusExporter::new(config).unwrap();

        exporter.record_request("openai", "gpt-4", "success");
        exporter.record_duration("openai", "gpt-4", 1.5);
        exporter.record_tokens("openai", "gpt-4", 100, 50);
        exporter.record_cost("openai", "gpt-4", 0.05);
        exporter.record_error("openai", "gpt-4", "rate_limit");
    }
}

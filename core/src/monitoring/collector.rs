// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Metric collection and aggregation.

use anyhow::Result;
use std::sync::Arc;
use std::collections::HashMap;
use parking_lot::RwLock;
use tokio::task::JoinHandle;
use chrono::{DateTime, Utc, Duration};

use crate::monitoring::{
    metrics::Metric,
    events::{EventBus, EventSubscriber, MonitoringEvent, EventPayload},
};

/// Metric collector configuration
#[derive(Debug, Clone)]
pub struct CollectorConfig {
    /// Metric retention period in seconds
    pub retention_period: u64,
    /// Enable detailed metrics
    pub detailed_metrics: bool,
}

impl Default for CollectorConfig {
    fn default() -> Self {
        Self {
            retention_period: 3600, // 1 hour
            detailed_metrics: false,
        }
    }
}

/// Time-series metric storage
#[derive(Debug, Clone)]
struct MetricSeries {
    /// Metric name
    name: String,
    /// Data points (timestamp, value)
    points: Vec<(DateTime<Utc>, f64)>,
    /// Labels
    labels: HashMap<String, String>,
}

impl MetricSeries {
    fn new(name: String, labels: HashMap<String, String>) -> Self {
        Self {
            name,
            points: Vec::new(),
            labels,
        }
    }

    fn add_point(&mut self, timestamp: DateTime<Utc>, value: f64) {
        self.points.push((timestamp, value));
    }

    fn retain(&mut self, cutoff: DateTime<Utc>) {
        self.points.retain(|(ts, _)| *ts > cutoff);
    }

    fn latest(&self) -> Option<f64> {
        self.points.last().map(|(_, v)| *v)
    }

    fn average(&self) -> Option<f64> {
        if self.points.is_empty() {
            return None;
        }
        let sum: f64 = self.points.iter().map(|(_, v)| v).sum();
        Some(sum / self.points.len() as f64)
    }

    fn sum(&self) -> f64 {
        self.points.iter().map(|(_, v)| v).sum()
    }
}

/// Provider statistics
#[derive(Debug, Clone)]
pub struct ProviderStats {
    /// Provider name
    pub provider: String,
    /// Total requests
    pub total_requests: u64,
    /// Successful requests
    pub successful_requests: u64,
    /// Failed requests
    pub failed_requests: u64,
    /// Average latency (seconds)
    pub avg_latency: Option<f64>,
    /// Total input tokens
    pub total_input_tokens: u64,
    /// Total output tokens
    pub total_output_tokens: u64,
    /// Total cost (USD)
    pub total_cost: f64,
}

/// Metric collector with time-series storage
pub struct MetricCollector {
    config: CollectorConfig,
    event_bus: Arc<EventBus>,
    metrics: Arc<RwLock<HashMap<String, MetricSeries>>>,
    provider_stats: Arc<RwLock<HashMap<String, ProviderStats>>>,
    cleanup_handle: Arc<RwLock<Option<JoinHandle<()>>>>,
}

impl MetricCollector {
    /// Create a new metric collector
    pub fn new(config: CollectorConfig, event_bus: Arc<EventBus>) -> Self {
        let collector = Self {
            config,
            event_bus: event_bus.clone(),
            metrics: Arc::new(RwLock::new(HashMap::new())),
            provider_stats: Arc::new(RwLock::new(HashMap::new())),
            cleanup_handle: Arc::new(RwLock::new(None)),
        };

        // Subscribe to events
        let collector_ref = Arc::new(CollectorSubscriber {
            metrics: collector.metrics.clone(),
            provider_stats: collector.provider_stats.clone(),
        });
        event_bus.add_subscriber(collector_ref);

        collector
    }

    /// Start the metric collector
    pub async fn start(&self) -> Result<()> {
        tracing::info!("Starting metric collector");

        // Start cleanup task
        let metrics = self.metrics.clone();
        let retention_period = self.config.retention_period;

        let cleanup = tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60));
            loop {
                interval.tick().await;
                Self::cleanup_old_metrics(&metrics, retention_period);
            }
        });

        let mut handle = self.cleanup_handle.write();
        *handle = Some(cleanup);

        Ok(())
    }

    /// Stop the metric collector
    pub async fn stop(&self) -> Result<()> {
        let mut handle = self.cleanup_handle.write();
        if let Some(h) = handle.take() {
            h.abort();
        }
        Ok(())
    }

    /// Cleanup old metrics
    fn cleanup_old_metrics(metrics: &RwLock<HashMap<String, MetricSeries>>, retention_period: u64) {
        let cutoff = Utc::now() - Duration::seconds(retention_period as i64);
        let mut metrics = metrics.write();

        for series in metrics.values_mut() {
            series.retain(cutoff);
        }

        // Remove empty series
        metrics.retain(|_, series| !series.points.is_empty());
    }

    /// Get all metrics
    pub async fn get_metrics(&self) -> Vec<Metric> {
        let metrics = self.metrics.read();
        metrics
            .values()
            .filter_map(|series| {
                series.latest().map(|value| {
                    let mut metric = Metric::new(
                        &series.name,
                        crate::monitoring::metrics::MetricType::Gauge,
                        crate::monitoring::metrics::MetricValue::Gauge(value),
                    );
                    for (k, v) in &series.labels {
                        metric = metric.with_label(k, v);
                    }
                    metric
                })
            })
            .collect()
    }

    /// Get metrics for a specific provider
    pub async fn get_provider_metrics(&self, provider: &str) -> Vec<Metric> {
        let metrics = self.metrics.read();
        metrics
            .values()
            .filter(|series| {
                series.labels.get("provider").map(|p| p.as_str()) == Some(provider)
            })
            .filter_map(|series| {
                series.latest().map(|value| {
                    let mut metric = Metric::new(
                        &series.name,
                        crate::monitoring::metrics::MetricType::Gauge,
                        crate::monitoring::metrics::MetricValue::Gauge(value),
                    );
                    for (k, v) in &series.labels {
                        metric = metric.with_label(k, v);
                    }
                    metric
                })
            })
            .collect()
    }

    /// Get provider statistics
    pub fn get_provider_stats(&self, provider: &str) -> Option<ProviderStats> {
        self.provider_stats.read().get(provider).cloned()
    }

    /// Get all provider statistics
    pub fn get_all_provider_stats(&self) -> Vec<ProviderStats> {
        self.provider_stats.read().values().cloned().collect()
    }
}

/// Event subscriber for metric collection
struct CollectorSubscriber {
    metrics: Arc<RwLock<HashMap<String, MetricSeries>>>,
    provider_stats: Arc<RwLock<HashMap<String, ProviderStats>>>,
}

impl EventSubscriber for CollectorSubscriber {
    fn on_event(&self, event: &MonitoringEvent) {
        match &event.payload {
            EventPayload::Metric(metric_event) => {
                self.record_metric(&metric_event.metric);
            }
            EventPayload::Request(request_event) => {
                self.record_request(request_event);
            }
            _ => {}
        }
    }
}

impl CollectorSubscriber {
    fn record_metric(&self, metric: &Metric) {
        let key = format!("{}:{:?}", metric.name, metric.labels.all());
        let mut metrics = self.metrics.write();

        let series = metrics
            .entry(key)
            .or_insert_with(|| MetricSeries::new(metric.name.clone(), metric.labels.all().clone()));

        let value = match &metric.value {
            crate::monitoring::metrics::MetricValue::Counter(v) => *v as f64,
            crate::monitoring::metrics::MetricValue::Gauge(v) => *v,
            crate::monitoring::metrics::MetricValue::Histogram(h) => h.sum,
            crate::monitoring::metrics::MetricValue::Summary(s) => s.sum,
        };

        series.add_point(metric.timestamp, value);
    }

    fn record_request(&self, request: &crate::monitoring::events::RequestEvent) {
        let mut stats = self.provider_stats.write();
        let provider_stats = stats.entry(request.provider.clone()).or_insert_with(|| {
            ProviderStats {
                provider: request.provider.clone(),
                total_requests: 0,
                successful_requests: 0,
                failed_requests: 0,
                avg_latency: None,
                total_input_tokens: 0,
                total_output_tokens: 0,
                total_cost: 0.0,
            }
        });

        provider_stats.total_requests += 1;

        if request.error.is_some() {
            provider_stats.failed_requests += 1;
        } else {
            provider_stats.successful_requests += 1;
        }

        if let Some(latency) = request.latency {
            let current_avg = provider_stats.avg_latency.unwrap_or(0.0);
            let count = provider_stats.successful_requests as f64;
            provider_stats.avg_latency = Some((current_avg * (count - 1.0) + latency) / count);
        }

        if let Some(ref tokens) = request.tokens {
            provider_stats.total_input_tokens += tokens.input_tokens;
            provider_stats.total_output_tokens += tokens.output_tokens;
        }

        if let Some(cost) = request.cost {
            provider_stats.total_cost += cost;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metric_series() {
        let mut series = MetricSeries::new("test".to_string(), HashMap::new());
        series.add_point(Utc::now(), 10.0);
        series.add_point(Utc::now(), 20.0);
        series.add_point(Utc::now(), 30.0);

        assert_eq!(series.latest(), Some(30.0));
        assert_eq!(series.average(), Some(20.0));
        assert_eq!(series.sum(), 60.0);
    }

    #[test]
    fn test_metric_series_retention() {
        let mut series = MetricSeries::new("test".to_string(), HashMap::new());
        let old_time = Utc::now() - Duration::hours(2);
        let recent_time = Utc::now();

        series.add_point(old_time, 10.0);
        series.add_point(recent_time, 20.0);

        let cutoff = Utc::now() - Duration::hours(1);
        series.retain(cutoff);

        assert_eq!(series.points.len(), 1);
        assert_eq!(series.latest(), Some(20.0));
    }

    #[tokio::test]
    async fn test_collector_creation() {
        let config = CollectorConfig::default();
        let event_bus = Arc::new(EventBus::new());
        let collector = MetricCollector::new(config, event_bus);

        let stats = collector.get_all_provider_stats();
        assert_eq!(stats.len(), 0);
    }

    #[test]
    fn test_provider_stats() {
        let stats = ProviderStats {
            provider: "openai".to_string(),
            total_requests: 100,
            successful_requests: 95,
            failed_requests: 5,
            avg_latency: Some(1.5),
            total_input_tokens: 10000,
            total_output_tokens: 5000,
            total_cost: 2.50,
        };

        assert_eq!(stats.provider, "openai");
        assert_eq!(stats.total_requests, 100);
        assert_eq!(stats.successful_requests, 95);
    }
}

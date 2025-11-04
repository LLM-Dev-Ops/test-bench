// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Event system for real-time monitoring with pub/sub architecture.

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use parking_lot::RwLock;
use tokio::sync::broadcast;
use chrono::{DateTime, Utc};
use crate::monitoring::metrics::*;

/// Event types for monitoring
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    /// Request started
    RequestStarted,
    /// Request completed
    RequestCompleted,
    /// Request failed
    RequestFailed,
    /// Benchmark started
    BenchmarkStarted,
    /// Benchmark progress update
    BenchmarkProgress,
    /// Benchmark completed
    BenchmarkCompleted,
    /// Evaluation started
    EvaluationStarted,
    /// Evaluation completed
    EvaluationCompleted,
    /// Provider status change
    ProviderStatus,
    /// Metric recorded
    MetricRecorded,
    /// System alert
    SystemAlert,
}

/// A monitoring event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringEvent {
    /// Event type
    pub event_type: EventType,

    /// Event timestamp
    pub timestamp: DateTime<Utc>,

    /// Event payload
    pub payload: EventPayload,

    /// Event ID (for tracking)
    pub id: String,
}

/// Event payload variants
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum EventPayload {
    /// Request event
    Request(RequestEvent),
    /// Benchmark event
    Benchmark(BenchmarkEvent),
    /// Evaluation event
    Evaluation(EvaluationEvent),
    /// Provider event
    Provider(ProviderEvent),
    /// Metric event
    Metric(MetricEvent),
    /// Alert event
    Alert(AlertEvent),
}

/// Request-related event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestEvent {
    /// Provider name
    pub provider: String,
    /// Model name
    pub model: String,
    /// Request ID
    pub request_id: String,
    /// Latency (if completed)
    pub latency: Option<f64>,
    /// Token usage
    pub tokens: Option<TokenUsage>,
    /// Cost
    pub cost: Option<f64>,
    /// Error (if failed)
    pub error: Option<String>,
}

/// Token usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub total_tokens: u64,
}

/// Benchmark-related event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkEvent {
    /// Benchmark ID
    pub benchmark_id: String,
    /// Benchmark name
    pub name: String,
    /// Total examples
    pub total_examples: usize,
    /// Completed examples
    pub completed_examples: usize,
    /// Progress percentage (0-100)
    pub progress_percent: f64,
    /// Current provider
    pub current_provider: Option<String>,
    /// Estimated time remaining (seconds)
    pub eta_seconds: Option<f64>,
}

impl BenchmarkEvent {
    pub fn started(benchmark_id: impl Into<String>, name: impl Into<String>, total: usize) -> Self {
        Self {
            benchmark_id: benchmark_id.into(),
            name: name.into(),
            total_examples: total,
            completed_examples: 0,
            progress_percent: 0.0,
            current_provider: None,
            eta_seconds: None,
        }
    }

    pub fn progress(
        benchmark_id: impl Into<String>,
        name: impl Into<String>,
        completed: usize,
        total: usize,
    ) -> Self {
        let progress_percent = if total > 0 {
            (completed as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        Self {
            benchmark_id: benchmark_id.into(),
            name: name.into(),
            total_examples: total,
            completed_examples: completed,
            progress_percent,
            current_provider: None,
            eta_seconds: None,
        }
    }

    pub fn completed(benchmark_id: impl Into<String>, name: impl Into<String>, total: usize) -> Self {
        Self {
            benchmark_id: benchmark_id.into(),
            name: name.into(),
            total_examples: total,
            completed_examples: total,
            progress_percent: 100.0,
            current_provider: None,
            eta_seconds: Some(0.0),
        }
    }
}

/// Evaluation-related event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationEvent {
    /// Evaluation ID
    pub evaluation_id: String,
    /// Metric name
    pub metric_name: String,
    /// Provider name
    pub provider: String,
    /// Model name
    pub model: String,
    /// Score
    pub score: f64,
    /// Additional details
    pub details: Option<serde_json::Value>,
}

/// Provider status event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderEvent {
    /// Provider name
    pub provider: String,
    /// Status
    pub status: ProviderStatus,
    /// Status message
    pub message: Option<String>,
    /// Active requests
    pub active_requests: u64,
    /// Average latency (ms)
    pub avg_latency_ms: Option<f64>,
}

/// Provider status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderStatus {
    /// Provider is healthy
    Healthy,
    /// Provider is degraded
    Degraded,
    /// Provider is unavailable
    Unavailable,
    /// Provider is rate limited
    RateLimited,
}

/// Metric event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricEvent {
    /// Metric
    pub metric: Metric,
}

/// Alert event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertEvent {
    /// Alert severity
    pub severity: AlertSeverity,
    /// Alert title
    pub title: String,
    /// Alert message
    pub message: String,
    /// Related provider (if any)
    pub provider: Option<String>,
}

/// Alert severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AlertSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

impl MonitoringEvent {
    /// Create a new event with generated ID
    pub fn new(event_type: EventType, payload: EventPayload) -> Self {
        Self {
            event_type,
            timestamp: Utc::now(),
            payload,
            id: Self::generate_id(),
        }
    }

    /// Create a request event
    pub fn request(provider: &str, model: &str) -> Self {
        Self::new(
            EventType::RequestStarted,
            EventPayload::Request(RequestEvent {
                provider: provider.to_string(),
                model: model.to_string(),
                request_id: Self::generate_id(),
                latency: None,
                tokens: None,
                cost: None,
                error: None,
            }),
        )
    }

    /// Create a latency event
    pub fn latency(provider: &str, latency: f64) -> Self {
        let metric = LatencyMetric::new(provider, "", latency);
        Self::new(
            EventType::MetricRecorded,
            EventPayload::Metric(MetricEvent {
                metric: metric.to_metric(),
            }),
        )
    }

    /// Create a token event
    pub fn tokens(provider: &str, input_tokens: u64, output_tokens: u64) -> Self {
        let metric = TokenMetric::new(provider, "", input_tokens, output_tokens);
        Self::new(
            EventType::MetricRecorded,
            EventPayload::Metric(MetricEvent {
                metric: metric.to_metrics()[0].clone(),
            }),
        )
    }

    /// Create a cost event
    pub fn cost(provider: &str, cost: f64) -> Self {
        let metric = CostMetric::new(provider, "", cost);
        Self::new(
            EventType::MetricRecorded,
            EventPayload::Metric(MetricEvent {
                metric: metric.to_metric(),
            }),
        )
    }

    /// Create an error event
    pub fn error(provider: &str, error_type: &str) -> Self {
        let metric = ErrorMetric::new(provider, error_type);
        Self::new(
            EventType::RequestFailed,
            EventPayload::Metric(MetricEvent {
                metric: metric.to_metric(),
            }),
        )
    }

    /// Create a benchmark started event
    pub fn benchmark_started(id: impl Into<String>, name: impl Into<String>, total: usize) -> Self {
        Self::new(
            EventType::BenchmarkStarted,
            EventPayload::Benchmark(BenchmarkEvent::started(id, name, total)),
        )
    }

    /// Create a benchmark progress event
    pub fn benchmark_progress(
        id: impl Into<String>,
        name: impl Into<String>,
        completed: usize,
        total: usize,
    ) -> Self {
        Self::new(
            EventType::BenchmarkProgress,
            EventPayload::Benchmark(BenchmarkEvent::progress(id, name, completed, total)),
        )
    }

    /// Create a benchmark completed event
    pub fn benchmark_completed(id: impl Into<String>, name: impl Into<String>, total: usize) -> Self {
        Self::new(
            EventType::BenchmarkCompleted,
            EventPayload::Benchmark(BenchmarkEvent::completed(id, name, total)),
        )
    }

    fn generate_id() -> String {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        format!("evt_{}", id)
    }
}

/// Event subscriber trait
pub trait EventSubscriber: Send + Sync {
    /// Handle an event
    fn on_event(&self, event: &MonitoringEvent);
}

/// Event bus for pub/sub messaging
pub struct EventBus {
    /// Broadcast channel for events
    sender: broadcast::Sender<MonitoringEvent>,
    /// Subscribers
    subscribers: Arc<RwLock<Vec<Arc<dyn EventSubscriber>>>>,
}

impl EventBus {
    /// Create a new event bus
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(1000);
        Self {
            sender,
            subscribers: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Publish an event
    pub fn publish(&self, event: MonitoringEvent) {
        // Send to broadcast channel (for WebSocket)
        let _ = self.sender.send(event.clone());

        // Notify subscribers
        let subscribers = self.subscribers.read();
        for subscriber in subscribers.iter() {
            subscriber.on_event(&event);
        }
    }

    /// Subscribe to events
    pub fn subscribe(&self) -> broadcast::Receiver<MonitoringEvent> {
        self.sender.subscribe()
    }

    /// Add a subscriber
    pub fn add_subscriber(&self, subscriber: Arc<dyn EventSubscriber>) {
        let mut subscribers = self.subscribers.write();
        subscribers.push(subscriber);
    }

    /// Remove all subscribers
    pub fn clear_subscribers(&self) {
        let mut subscribers = self.subscribers.write();
        subscribers.clear();
    }

    /// Get subscriber count
    pub fn subscriber_count(&self) -> usize {
        self.subscribers.read().len()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_creation() {
        let event = MonitoringEvent::request("openai", "gpt-4");
        assert_eq!(event.event_type, EventType::RequestStarted);
    }

    #[test]
    fn test_benchmark_event() {
        let event = BenchmarkEvent::started("bench_1", "Test Benchmark", 100);
        assert_eq!(event.total_examples, 100);
        assert_eq!(event.completed_examples, 0);
        assert_eq!(event.progress_percent, 0.0);

        let progress = BenchmarkEvent::progress("bench_1", "Test Benchmark", 50, 100);
        assert_eq!(progress.progress_percent, 50.0);

        let completed = BenchmarkEvent::completed("bench_1", "Test Benchmark", 100);
        assert_eq!(completed.progress_percent, 100.0);
    }

    #[test]
    fn test_event_bus() {
        let bus = EventBus::new();
        let event = MonitoringEvent::request("openai", "gpt-4");

        bus.publish(event.clone());

        // Test subscriber
        let mut rx = bus.subscribe();
        bus.publish(event.clone());

        let received = rx.try_recv();
        assert!(received.is_ok());
    }

    struct TestSubscriber;
    impl EventSubscriber for TestSubscriber {
        fn on_event(&self, _event: &MonitoringEvent) {
            // Test subscriber
        }
    }

    #[test]
    fn test_event_subscribers() {
        let bus = EventBus::new();
        bus.add_subscriber(Arc::new(TestSubscriber));
        assert_eq!(bus.subscriber_count(), 1);

        bus.clear_subscribers();
        assert_eq!(bus.subscriber_count(), 0);
    }
}

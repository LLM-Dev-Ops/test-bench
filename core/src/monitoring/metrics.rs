// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Core metric types and structures for monitoring.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

/// A metric data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    /// Metric name
    pub name: String,

    /// Metric type
    pub metric_type: MetricType,

    /// Metric value
    pub value: MetricValue,

    /// Labels/tags for the metric
    pub labels: MetricLabels,

    /// Timestamp when the metric was recorded
    pub timestamp: DateTime<Utc>,

    /// Optional help text
    #[serde(skip_serializing_if = "Option::is_none")]
    pub help: Option<String>,
}

impl Metric {
    /// Create a new metric
    pub fn new(
        name: impl Into<String>,
        metric_type: MetricType,
        value: MetricValue,
    ) -> Self {
        Self {
            name: name.into(),
            metric_type,
            value,
            labels: MetricLabels::new(),
            timestamp: Utc::now(),
            help: None,
        }
    }

    /// Add a label to the metric
    pub fn with_label(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.labels.add(key, value);
        self
    }

    /// Set multiple labels
    pub fn with_labels(mut self, labels: MetricLabels) -> Self {
        self.labels = labels;
        self
    }

    /// Set help text
    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }
}

/// Types of metrics
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MetricType {
    /// Counter - monotonically increasing value
    Counter,
    /// Gauge - value that can go up and down
    Gauge,
    /// Histogram - distribution of values
    Histogram,
    /// Summary - similar to histogram but with percentiles
    Summary,
}

/// Metric value variants
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MetricValue {
    /// Integer counter value
    Counter(u64),
    /// Floating-point gauge value
    Gauge(f64),
    /// Histogram buckets with counts
    Histogram(HistogramValue),
    /// Summary with percentiles
    Summary(SummaryValue),
}

/// Histogram value with buckets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistogramValue {
    /// Bucket upper bounds and counts
    pub buckets: Vec<HistogramBucket>,
    /// Total count of observations
    pub count: u64,
    /// Sum of all observed values
    pub sum: f64,
}

/// A single histogram bucket
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistogramBucket {
    /// Upper bound (inclusive)
    pub le: f64,
    /// Cumulative count
    pub count: u64,
}

/// Summary value with quantiles
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummaryValue {
    /// Quantiles (e.g., 0.5 = median, 0.95 = 95th percentile)
    pub quantiles: Vec<Quantile>,
    /// Total count of observations
    pub count: u64,
    /// Sum of all observed values
    pub sum: f64,
}

/// A single quantile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quantile {
    /// Quantile value (0.0 to 1.0)
    pub quantile: f64,
    /// Value at this quantile
    pub value: f64,
}

/// Labels/tags for metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MetricLabels {
    labels: HashMap<String, String>,
}

impl MetricLabels {
    /// Create a new empty label set
    pub fn new() -> Self {
        Self {
            labels: HashMap::new(),
        }
    }

    /// Add a label
    pub fn add(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.labels.insert(key.into(), value.into());
    }

    /// Get a label value
    pub fn get(&self, key: &str) -> Option<&str> {
        self.labels.get(key).map(|s| s.as_str())
    }

    /// Get all labels
    pub fn all(&self) -> &HashMap<String, String> {
        &self.labels
    }

    /// Check if labels are empty
    pub fn is_empty(&self) -> bool {
        self.labels.is_empty()
    }
}

/// Request metric for tracking API calls
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestMetric {
    /// Provider name
    pub provider: String,
    /// Model name
    pub model: String,
    /// Request timestamp
    pub timestamp: DateTime<Utc>,
    /// Request ID (optional)
    pub request_id: Option<String>,
}

impl RequestMetric {
    pub fn new(provider: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            provider: provider.into(),
            model: model.into(),
            timestamp: Utc::now(),
            request_id: None,
        }
    }

    pub fn to_metric(&self) -> Metric {
        Metric::new(
            "llm_requests_total",
            MetricType::Counter,
            MetricValue::Counter(1),
        )
        .with_label("provider", &self.provider)
        .with_label("model", &self.model)
        .with_help("Total number of LLM requests")
    }
}

/// Latency metric for tracking response times
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyMetric {
    /// Provider name
    pub provider: String,
    /// Model name
    pub model: String,
    /// Latency in seconds
    pub latency: f64,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

impl LatencyMetric {
    pub fn new(provider: impl Into<String>, model: impl Into<String>, latency: f64) -> Self {
        Self {
            provider: provider.into(),
            model: model.into(),
            latency,
            timestamp: Utc::now(),
        }
    }

    pub fn to_metric(&self) -> Metric {
        Metric::new(
            "llm_request_duration_seconds",
            MetricType::Histogram,
            MetricValue::Histogram(HistogramValue {
                buckets: Self::default_buckets(),
                count: 1,
                sum: self.latency,
            }),
        )
        .with_label("provider", &self.provider)
        .with_label("model", &self.model)
        .with_help("Request duration in seconds")
    }

    fn default_buckets() -> Vec<HistogramBucket> {
        vec![
            HistogramBucket { le: 0.1, count: 0 },
            HistogramBucket { le: 0.5, count: 0 },
            HistogramBucket { le: 1.0, count: 0 },
            HistogramBucket { le: 2.0, count: 0 },
            HistogramBucket { le: 5.0, count: 0 },
            HistogramBucket { le: 10.0, count: 0 },
            HistogramBucket { le: f64::INFINITY, count: 0 },
        ]
    }
}

/// Token usage metric
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenMetric {
    /// Provider name
    pub provider: String,
    /// Model name
    pub model: String,
    /// Input tokens
    pub input_tokens: u64,
    /// Output tokens
    pub output_tokens: u64,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

impl TokenMetric {
    pub fn new(
        provider: impl Into<String>,
        model: impl Into<String>,
        input_tokens: u64,
        output_tokens: u64,
    ) -> Self {
        Self {
            provider: provider.into(),
            model: model.into(),
            input_tokens,
            output_tokens,
            timestamp: Utc::now(),
        }
    }

    pub fn to_metrics(&self) -> Vec<Metric> {
        vec![
            Metric::new(
                "llm_tokens_input_total",
                MetricType::Counter,
                MetricValue::Counter(self.input_tokens),
            )
            .with_label("provider", &self.provider)
            .with_label("model", &self.model)
            .with_help("Total input tokens processed"),
            Metric::new(
                "llm_tokens_output_total",
                MetricType::Counter,
                MetricValue::Counter(self.output_tokens),
            )
            .with_label("provider", &self.provider)
            .with_label("model", &self.model)
            .with_help("Total output tokens generated"),
        ]
    }
}

/// Cost metric for tracking expenses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostMetric {
    /// Provider name
    pub provider: String,
    /// Model name
    pub model: String,
    /// Cost in USD
    pub cost: f64,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

impl CostMetric {
    pub fn new(provider: impl Into<String>, model: impl Into<String>, cost: f64) -> Self {
        Self {
            provider: provider.into(),
            model: model.into(),
            cost,
            timestamp: Utc::now(),
        }
    }

    pub fn to_metric(&self) -> Metric {
        Metric::new(
            "llm_cost_usd_total",
            MetricType::Counter,
            MetricValue::Counter((self.cost * 1_000_000.0) as u64), // Store as micro-dollars
        )
        .with_label("provider", &self.provider)
        .with_label("model", &self.model)
        .with_help("Total cost in USD (micro-dollars)")
    }
}

/// Error metric for tracking failures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorMetric {
    /// Provider name
    pub provider: String,
    /// Model name (optional)
    pub model: Option<String>,
    /// Error type
    pub error_type: String,
    /// Error message (optional)
    pub error_message: Option<String>,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

impl ErrorMetric {
    pub fn new(provider: impl Into<String>, error_type: impl Into<String>) -> Self {
        Self {
            provider: provider.into(),
            model: None,
            error_type: error_type.into(),
            error_message: None,
            timestamp: Utc::now(),
        }
    }

    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.error_message = Some(message.into());
        self
    }

    pub fn to_metric(&self) -> Metric {
        let mut metric = Metric::new(
            "llm_errors_total",
            MetricType::Counter,
            MetricValue::Counter(1),
        )
        .with_label("provider", &self.provider)
        .with_label("error_type", &self.error_type)
        .with_help("Total number of errors");

        if let Some(ref model) = self.model {
            metric = metric.with_label("model", model);
        }

        metric
    }
}

/// Active requests gauge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveRequestsMetric {
    /// Provider name
    pub provider: String,
    /// Number of active requests
    pub count: u64,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

impl ActiveRequestsMetric {
    pub fn new(provider: impl Into<String>, count: u64) -> Self {
        Self {
            provider: provider.into(),
            count,
            timestamp: Utc::now(),
        }
    }

    pub fn to_metric(&self) -> Metric {
        Metric::new(
            "llm_active_requests",
            MetricType::Gauge,
            MetricValue::Gauge(self.count as f64),
        )
        .with_label("provider", &self.provider)
        .with_help("Number of active requests")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metric_creation() {
        let metric = Metric::new(
            "test_metric",
            MetricType::Counter,
            MetricValue::Counter(42),
        )
        .with_label("key", "value");

        assert_eq!(metric.name, "test_metric");
        assert_eq!(metric.labels.get("key"), Some("value"));
    }

    #[test]
    fn test_request_metric() {
        let request = RequestMetric::new("openai", "gpt-4");
        let metric = request.to_metric();

        assert_eq!(metric.name, "llm_requests_total");
        assert_eq!(metric.labels.get("provider"), Some("openai"));
    }

    #[test]
    fn test_latency_metric() {
        let latency = LatencyMetric::new("openai", "gpt-4", 1.5);
        let metric = latency.to_metric();

        assert_eq!(metric.name, "llm_request_duration_seconds");
    }

    #[test]
    fn test_token_metric() {
        let tokens = TokenMetric::new("openai", "gpt-4", 100, 50);
        let metrics = tokens.to_metrics();

        assert_eq!(metrics.len(), 2);
        assert_eq!(metrics[0].name, "llm_tokens_input_total");
        assert_eq!(metrics[1].name, "llm_tokens_output_total");
    }

    #[test]
    fn test_cost_metric() {
        let cost = CostMetric::new("openai", "gpt-4", 0.05);
        let metric = cost.to_metric();

        assert_eq!(metric.name, "llm_cost_usd_total");
    }

    #[test]
    fn test_error_metric() {
        let error = ErrorMetric::new("openai", "rate_limit")
            .with_model("gpt-4")
            .with_message("Rate limit exceeded");
        let metric = error.to_metric();

        assert_eq!(metric.name, "llm_errors_total");
        assert_eq!(metric.labels.get("error_type"), Some("rate_limit"));
    }

    #[test]
    fn test_metric_labels() {
        let mut labels = MetricLabels::new();
        labels.add("provider", "openai");
        labels.add("model", "gpt-4");

        assert_eq!(labels.get("provider"), Some("openai"));
        assert_eq!(labels.get("model"), Some("gpt-4"));
        assert!(!labels.is_empty());
    }
}

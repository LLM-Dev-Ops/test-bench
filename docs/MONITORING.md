# Real-time Monitoring Guide

**Enterprise-grade real-time monitoring with Prometheus metrics and WebSocket dashboards**

## Table of Contents

1. [Overview](#overview)
2. [Architecture](#architecture)
3. [Quick Start](#quick-start)
4. [Prometheus Integration](#prometheus-integration)
5. [WebSocket Dashboards](#websocket-dashboards)
6. [Metrics Reference](#metrics-reference)
7. [Event System](#event-system)
8. [Provider Integration](#provider-integration)
9. [Benchmark Monitoring](#benchmark-monitoring)
10. [Configuration](#configuration)
11. [Best Practices](#best-practices)
12. [Troubleshooting](#troubleshooting)

---

## Overview

The LLM Test Bench monitoring system provides enterprise-grade observability for your LLM operations:

- **📊 Prometheus Metrics**: Industry-standard metrics export for Grafana, Datadog, etc.
- **🔄 Real-time Dashboards**: Live WebSocket-based dashboards with Chart.js
- **📡 Event System**: Pub/sub architecture for distributed monitoring
- **🎯 Zero Configuration**: Works out-of-the-box with sensible defaults
- **🏢 Production Ready**: Battle-tested for high-scale deployments

### Key Features

- **Request Tracking**: Monitor every LLM API call with latency, tokens, and cost
- **Provider Health**: Real-time provider status and performance metrics
- **Benchmark Progress**: Live updates during benchmark execution
- **Cost Optimization**: Track spending across providers and models
- **Error Monitoring**: Detailed error tracking with categorization
- **Historical Data**: Time-series metrics with configurable retention

---

## Architecture

```text
┌─────────────────────────────────────────────────────────────┐
│                    LLM Test Bench Core                      │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐     │
│  │  Providers   │  │  Benchmarks  │  │  Evaluators  │     │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘     │
│         └──────────────────┼──────────────────┘             │
└────────────────────────────┼────────────────────────────────┘
                             ▼
                    ┌──────────────────┐
                    │   Event Bus      │
                    │  (pub/sub)       │
                    └──────────────────┘
                             │
                 ┌───────────┼───────────┐
                 ▼           ▼           ▼
         ┌──────────┐  ┌──────────┐  ┌──────────┐
         │Prometheus│  │WebSocket │  │ Metrics  │
         │ Exporter │  │  Server  │  │Collector │
         └─────┬────┘  └─────┬────┘  └─────┬────┘
               │             │             │
               ▼             ▼             ▼
         [Prometheus]  [Dashboard]  [Aggregation]
         [Grafana]     [Real-time]  [Analytics]
```

### Components

1. **Event Bus**: Central pub/sub system distributing monitoring events
2. **Prometheus Exporter**: HTTP endpoint exposing metrics at `/metrics`
3. **WebSocket Server**: Real-time event streaming to dashboards
4. **Metric Collector**: Time-series aggregation with configurable retention
5. **Dashboard**: Interactive HTML dashboard with live charts

---

## Quick Start

### Basic Setup

```rust
use llm_test_bench_core::monitoring::{MonitoringSystem, MonitoringConfig};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize with defaults
    let config = MonitoringConfig::default();
    let monitoring = MonitoringSystem::new(config).await?;

    // Start all monitoring services
    monitoring.start().await?;

    // Prometheus: http://localhost:9090/metrics
    // WebSocket:   ws://localhost:8080/ws
    // Dashboard:   http://localhost:3000

    Ok(())
}
```

### With Provider Integration

```rust
use llm_test_bench_core::{
    monitoring::{MonitoringSystem, MonitoringConfig, MonitoredProvider},
    providers::openai::OpenAIProvider,
};
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Setup monitoring
    let monitoring = Arc::new(
        MonitoringSystem::new(MonitoringConfig::default()).await?
    );
    monitoring.start().await?;

    // Wrap provider with monitoring
    let openai = Arc::new(OpenAIProvider::new("sk-...")?);
    let monitored = Arc::new(MonitoredProvider::new(openai, monitoring.clone()));

    // Now all requests are automatically monitored!
    let response = monitored.complete(request).await?;

    Ok(())
}
```

### Custom Configuration

```rust
use llm_test_bench_core::monitoring::MonitoringConfig;

let config = MonitoringConfig::new()
    .with_prometheus_port(9091)      // Custom Prometheus port
    .with_websocket_port(8081)        // Custom WebSocket port
    .with_dashboard_port(3001)        // Custom dashboard port
    .with_retention_period(7200)      // 2 hours retention
    .with_detailed_metrics(true);     // Enable detailed metrics

let monitoring = MonitoringSystem::new(config).await?;
```

---

## Prometheus Integration

### Exposed Metrics

The monitoring system exposes Prometheus-compatible metrics at `http://localhost:9090/metrics`:

#### Request Metrics

```prometheus
# Total number of requests
llm_requests_total{provider="openai",model="gpt-4",status="success"} 1234

# Request duration histogram
llm_request_duration_seconds_bucket{provider="openai",model="gpt-4",le="0.5"} 45
llm_request_duration_seconds_bucket{provider="openai",model="gpt-4",le="1.0"} 123
llm_request_duration_seconds_sum{provider="openai",model="gpt-4"} 1567.8
llm_request_duration_seconds_count{provider="openai",model="gpt-4"} 1234

# Active requests gauge
llm_requests_active{provider="openai"} 3
```

#### Token Metrics

```prometheus
# Input tokens processed
llm_tokens_input_total{provider="openai",model="gpt-4"} 1500000

# Output tokens generated
llm_tokens_output_total{provider="openai",model="gpt-4"} 750000
```

#### Cost Metrics

```prometheus
# Total cost in USD (micro-dollars)
llm_cost_usd_total{provider="openai",model="gpt-4"} 45000000  # $45.00
```

#### Error Metrics

```prometheus
# Errors by type
llm_errors_total{provider="openai",model="gpt-4",error_type="rate_limit"} 5
llm_errors_total{provider="openai",model="gpt-4",error_type="timeout"} 2
```

#### Evaluation Metrics

```prometheus
# Evaluation scores
llm_evaluation_score{provider="openai",model="gpt-4",metric="faithfulness"} 0.92
llm_evaluation_score{provider="openai",model="gpt-4",metric="relevance"} 0.88
```

#### Benchmark Metrics

```prometheus
# Benchmark progress
llm_benchmark_progress{benchmark_id="bench_123",name="MMLU"} 75.5

# Benchmark duration
llm_benchmark_duration_seconds_bucket{benchmark_id="bench_123",name="MMLU",le="300"} 1
```

### Grafana Dashboard

Create a Grafana dashboard using the exposed metrics:

```json
{
  "dashboard": {
    "title": "LLM Test Bench",
    "panels": [
      {
        "title": "Requests per Second",
        "targets": [
          {
            "expr": "rate(llm_requests_total[1m])",
            "legendFormat": "{{provider}} - {{model}}"
          }
        ]
      },
      {
        "title": "P95 Latency",
        "targets": [
          {
            "expr": "histogram_quantile(0.95, rate(llm_request_duration_seconds_bucket[5m]))",
            "legendFormat": "{{provider}}"
          }
        ]
      },
      {
        "title": "Cost per Hour",
        "targets": [
          {
            "expr": "rate(llm_cost_usd_total[1h]) * 3600 / 1000000",
            "legendFormat": "{{provider}}"
          }
        ]
      }
    ]
  }
}
```

### Prometheus Configuration

Add to your `prometheus.yml`:

```yaml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

scrape_configs:
  - job_name: 'llm-test-bench'
    static_configs:
      - targets: ['localhost:9090']
    metric_relabel_configs:
      - source_labels: [__name__]
        regex: 'llm_.*'
        action: keep
```

---

## WebSocket Dashboards

### Connecting to WebSocket

```javascript
const ws = new WebSocket('ws://localhost:8080/ws');

ws.onopen = () => {
    console.log('Connected to monitoring');
};

ws.onmessage = (event) => {
    const message = JSON.parse(event.data);
    handleMonitoringEvent(message);
};

function handleMonitoringEvent(message) {
    switch (message.type) {
        case 'Event':
            console.log('Event:', message.data);
            break;
        case 'Connected':
            console.log('Client ID:', message.data.client_id);
            break;
    }
}
```

### Message Types

#### Connected Message

```json
{
  "type": "Connected",
  "data": {
    "client_id": "client_1"
  }
}
```

#### Event Message

```json
{
  "type": "Event",
  "data": {
    "event_type": "RequestCompleted",
    "timestamp": "2025-01-15T10:30:00Z",
    "id": "evt_123",
    "payload": {
      "type": "Request",
      "data": {
        "provider": "openai",
        "model": "gpt-4",
        "request_id": "req_456",
        "latency": 1.52,
        "tokens": {
          "input_tokens": 150,
          "output_tokens": 75,
          "total_tokens": 225
        },
        "cost": 0.0135
      }
    }
  }
}
```

#### Benchmark Progress Message

```json
{
  "type": "Event",
  "data": {
    "event_type": "BenchmarkProgress",
    "timestamp": "2025-01-15T10:30:00Z",
    "payload": {
      "type": "Benchmark",
      "data": {
        "benchmark_id": "bench_123",
        "name": "MMLU",
        "total_examples": 1000,
        "completed_examples": 456,
        "progress_percent": 45.6,
        "current_provider": "openai",
        "eta_seconds": 120.5
      }
    }
  }
}
```

### Built-in Dashboard

Access the built-in real-time dashboard at `http://localhost:3000`:

Features:
- **Live Request Tracking**: Real-time request counter with rate display
- **Latency Monitoring**: Live latency charts with P50/P95/P99 percentiles
- **Token Usage**: Input/output token tracking over time
- **Cost Tracking**: Real-time cost accumulation by provider
- **Provider Health**: Status indicators for each provider
- **Event Stream**: Live scrolling event log
- **Benchmark Progress**: Real-time progress bars for running benchmarks

---

## Metrics Reference

### Core Metric Types

#### Counter
Monotonically increasing value (requests, tokens, cost, errors).

```rust
use llm_test_bench_core::monitoring::metrics::{Metric, MetricType, MetricValue};

let metric = Metric::new(
    "llm_requests_total",
    MetricType::Counter,
    MetricValue::Counter(1),
)
.with_label("provider", "openai")
.with_label("model", "gpt-4");
```

#### Gauge
Value that can go up or down (active requests, queue depth).

```rust
let metric = Metric::new(
    "llm_requests_active",
    MetricType::Gauge,
    MetricValue::Gauge(5.0),
)
.with_label("provider", "openai");
```

#### Histogram
Distribution of values (latency, token count).

```rust
use llm_test_bench_core::monitoring::metrics::{HistogramValue, HistogramBucket};

let histogram = HistogramValue {
    buckets: vec![
        HistogramBucket { le: 0.5, count: 45 },
        HistogramBucket { le: 1.0, count: 123 },
        HistogramBucket { le: 5.0, count: 234 },
    ],
    count: 234,
    sum: 456.7,
};

let metric = Metric::new(
    "llm_request_duration_seconds",
    MetricType::Histogram,
    MetricValue::Histogram(histogram),
);
```

### High-Level Metrics

#### RequestMetric

```rust
use llm_test_bench_core::monitoring::metrics::RequestMetric;

let request = RequestMetric::new("openai", "gpt-4");
let metric = request.to_metric();
```

#### LatencyMetric

```rust
use llm_test_bench_core::monitoring::metrics::LatencyMetric;

let latency = LatencyMetric::new("openai", "gpt-4", 1.52);
let metric = latency.to_metric();
```

#### TokenMetric

```rust
use llm_test_bench_core::monitoring::metrics::TokenMetric;

let tokens = TokenMetric::new("openai", "gpt-4", 150, 75);
let metrics = tokens.to_metrics(); // Returns Vec<Metric>
```

#### CostMetric

```rust
use llm_test_bench_core::monitoring::metrics::CostMetric;

let cost = CostMetric::new("openai", "gpt-4", 0.0135);
let metric = cost.to_metric();
```

#### ErrorMetric

```rust
use llm_test_bench_core::monitoring::metrics::ErrorMetric;

let error = ErrorMetric::new("openai", "rate_limit")
    .with_model("gpt-4")
    .with_message("Rate limit exceeded");
let metric = error.to_metric();
```

---

## Event System

### Event Types

```rust
pub enum EventType {
    RequestStarted,      // Request initiated
    RequestCompleted,    // Request succeeded
    RequestFailed,       // Request failed
    BenchmarkStarted,    // Benchmark started
    BenchmarkProgress,   // Benchmark progress update
    BenchmarkCompleted,  // Benchmark completed
    EvaluationStarted,   // Evaluation started
    EvaluationCompleted, // Evaluation completed
    ProviderStatus,      // Provider status change
    MetricRecorded,      // Metric recorded
    SystemAlert,         // System alert
}
```

### Publishing Events

```rust
use llm_test_bench_core::monitoring::MonitoringEvent;

// Simple event
let event = MonitoringEvent::request("openai", "gpt-4");
monitoring.event_bus().publish(event);

// Benchmark progress
let event = MonitoringEvent::benchmark_progress("bench_1", "MMLU", 50, 100);
monitoring.event_bus().publish(event);
```

### Subscribing to Events

```rust
use llm_test_bench_core::monitoring::events::{EventSubscriber, MonitoringEvent};
use std::sync::Arc;

struct MySubscriber;

impl EventSubscriber for MySubscriber {
    fn on_event(&self, event: &MonitoringEvent) {
        println!("Received event: {:?}", event.event_type);
    }
}

// Add subscriber
let subscriber = Arc::new(MySubscriber);
monitoring.event_bus().add_subscriber(subscriber);
```

### WebSocket Subscription

```rust
// Get broadcast receiver
let mut rx = monitoring.event_bus().subscribe();

// Receive events
tokio::spawn(async move {
    while let Ok(event) = rx.recv().await {
        println!("Event: {:?}", event);
    }
});
```

---

## Provider Integration

### Automatic Monitoring

Wrap providers with `MonitoredProvider` for automatic monitoring:

```rust
use llm_test_bench_core::{
    monitoring::{MonitoringSystem, MonitoredProvider},
    providers::openai::OpenAIProvider,
};
use std::sync::Arc;

// Create monitoring system
let monitoring = Arc::new(MonitoringSystem::new(config).await?);
monitoring.start().await?;

// Wrap provider
let openai = Arc::new(OpenAIProvider::new(api_key)?);
let monitored = Arc::new(MonitoredProvider::new(openai, monitoring.clone()));

// Use as normal - all metrics automatically recorded
let response = monitored.complete(request).await?;
```

### Monitor Multiple Providers

```rust
use llm_test_bench_core::monitoring::monitor_providers;

let providers: Vec<Arc<dyn Provider>> = vec![
    Arc::new(OpenAIProvider::new(openai_key)?),
    Arc::new(AnthropicProvider::new(anthropic_key)?),
    Arc::new(GoogleProvider::new(google_key)?),
];

// Wrap all providers at once
let monitored = monitor_providers(providers, monitoring.clone());
```

### Manual Instrumentation

For fine-grained control:

```rust
// Record request start
monitoring.record_request("openai", "gpt-4");

// Execute request
let start = Instant::now();
let response = provider.complete(request).await?;
let latency = start.elapsed().as_secs_f64();

// Record metrics
monitoring.record_latency("openai", latency);
monitoring.record_tokens("openai", input_tokens, output_tokens);
monitoring.record_cost("openai", cost);
```

---

## Benchmark Monitoring

### Automatic Progress Tracking

```rust
use llm_test_bench_core::{
    benchmarks::Benchmark,
    monitoring::MonitoringEvent,
};

async fn run_monitored_benchmark(
    benchmark: &mut Benchmark,
    monitoring: &MonitoringSystem,
) -> Result<()> {
    let benchmark_id = "bench_123";
    let total = benchmark.examples().len();

    // Emit start event
    let event = MonitoringEvent::benchmark_started(
        benchmark_id,
        benchmark.name(),
        total,
    );
    monitoring.event_bus().publish(event);

    // Run benchmark with progress updates
    for (i, example) in benchmark.examples().iter().enumerate() {
        // Process example...

        // Emit progress
        let event = MonitoringEvent::benchmark_progress(
            benchmark_id,
            benchmark.name(),
            i + 1,
            total,
        );
        monitoring.event_bus().publish(event);
    }

    // Emit completion
    let event = MonitoringEvent::benchmark_completed(
        benchmark_id,
        benchmark.name(),
        total,
    );
    monitoring.event_bus().publish(event);

    Ok(())
}
```

### Progress Updates

WebSocket clients receive real-time progress:

```javascript
ws.onmessage = (event) => {
    const msg = JSON.parse(event.data);
    if (msg.data.event_type === 'BenchmarkProgress') {
        const progress = msg.data.payload.data;
        updateProgressBar(
            progress.progress_percent,
            progress.completed_examples,
            progress.total_examples
        );

        if (progress.eta_seconds) {
            updateETA(progress.eta_seconds);
        }
    }
};
```

---

## Configuration

### MonitoringConfig

```rust
pub struct MonitoringConfig {
    // Prometheus settings
    pub prometheus_enabled: bool,    // Default: true
    pub prometheus_port: u16,        // Default: 9090

    // WebSocket settings
    pub websocket_enabled: bool,     // Default: true
    pub websocket_port: u16,         // Default: 8080

    // Dashboard settings
    pub dashboard_enabled: bool,     // Default: true
    pub dashboard_port: u16,         // Default: 3000

    // Data retention
    pub retention_period: u64,       // Default: 3600 (1 hour)

    // Performance
    pub detailed_metrics: bool,      // Default: false
}
```

### Builder Pattern

```rust
let config = MonitoringConfig::new()
    .with_prometheus(true)
    .with_prometheus_port(9090)
    .with_websocket(true)
    .with_websocket_port(8080)
    .with_dashboard(true)
    .with_dashboard_port(3000)
    .with_retention_period(7200)      // 2 hours
    .with_detailed_metrics(false);    // Performance mode
```

### Environment Variables

```bash
# Prometheus
export LTB_PROMETHEUS_ENABLED=true
export LTB_PROMETHEUS_PORT=9090

# WebSocket
export LTB_WEBSOCKET_ENABLED=true
export LTB_WEBSOCKET_PORT=8080

# Dashboard
export LTB_DASHBOARD_ENABLED=true
export LTB_DASHBOARD_PORT=3000

# Retention
export LTB_RETENTION_PERIOD=3600
```

### Production Configuration

```rust
// High-throughput production config
let config = MonitoringConfig::new()
    .with_prometheus(true)            // Enable Prometheus
    .with_websocket(false)            // Disable WebSocket (use Grafana)
    .with_dashboard(false)            // Disable dashboard
    .with_retention_period(300)       // 5 minutes (shorter for performance)
    .with_detailed_metrics(false);    // Disable detailed metrics

// Development config
let config = MonitoringConfig::new()
    .with_prometheus(true)
    .with_websocket(true)             // Enable real-time updates
    .with_dashboard(true)             // Enable dashboard
    .with_retention_period(3600)      // 1 hour
    .with_detailed_metrics(true);     // Full detail
```

---

## Best Practices

### 1. Use Monitored Providers

Always wrap providers for automatic instrumentation:

```rust
// ✅ Good
let monitored = MonitoredProvider::new(provider, monitoring);

// ❌ Avoid
// Manual instrumentation for every call
```

### 2. Configure Retention Appropriately

```rust
// Production: Short retention, external storage (Prometheus)
.with_retention_period(300)  // 5 minutes

// Development: Longer retention for debugging
.with_retention_period(3600) // 1 hour
```

### 3. Disable Unused Features

```rust
// If using Grafana, disable built-in dashboard
.with_dashboard(false)
.with_websocket(false)
```

### 4. Use Labels Consistently

```rust
// ✅ Good: Consistent labels
metric.with_label("provider", "openai")
      .with_label("model", "gpt-4");

// ❌ Avoid: Inconsistent naming
metric.with_label("Provider", "OpenAI")  // Wrong case
      .with_label("model_name", "gpt-4"); // Inconsistent
```

### 5. Handle High Cardinality

```rust
// ✅ Good: Low cardinality labels
.with_label("provider", "openai")
.with_label("model", "gpt-4")
.with_label("status", "success")

// ❌ Avoid: High cardinality (unique request IDs)
.with_label("request_id", "req_123456")  // DON'T DO THIS
```

### 6. Monitor Monitor Health

```rust
// Check monitoring system health
let stats = monitoring.get_all_provider_stats();
for stat in stats {
    if stat.failed_requests > stat.total_requests / 10 {
        eprintln!("High failure rate for {}: {}%",
            stat.provider,
            (stat.failed_requests * 100) / stat.total_requests
        );
    }
}
```

---

## Troubleshooting

### WebSocket Connection Issues

**Problem**: Dashboard shows "Disconnected"

**Solutions**:
```rust
// 1. Check WebSocket server is running
monitoring.start().await?;

// 2. Verify port is not in use
.with_websocket_port(8080)  // Try different port

// 3. Check firewall settings
// Allow TCP 8080

// 4. Enable CORS if needed
// (automatically handled by tower-http)
```

### Prometheus Scraping Fails

**Problem**: Prometheus can't scrape metrics

**Solutions**:
```yaml
# 1. Check Prometheus config
scrape_configs:
  - job_name: 'llm-test-bench'
    static_configs:
      - targets: ['localhost:9090']  # Correct port?

# 2. Test endpoint manually
curl http://localhost:9090/metrics

# 3. Check firewall
sudo ufw allow 9090/tcp
```

### High Memory Usage

**Problem**: Monitoring system consuming too much memory

**Solutions**:
```rust
// 1. Reduce retention period
.with_retention_period(300)  // 5 minutes

// 2. Disable detailed metrics
.with_detailed_metrics(false)

// 3. Use external storage
.with_prometheus(true)  // Let Prometheus store data
.with_websocket(false)  // Disable in-memory buffers
```

### Missing Metrics

**Problem**: Some metrics not appearing in Prometheus

**Solutions**:
```rust
// 1. Ensure monitoring is started
monitoring.start().await?;

// 2. Wrap providers correctly
let monitored = MonitoredProvider::new(provider, monitoring);

// 3. Check metric names
// Use official metric names from docs

// 4. Verify labels
let metric = metric.with_label("provider", "openai");  // Lowercase
```

### Dashboard Not Loading

**Problem**: Dashboard shows blank page

**Solutions**:
```rust
// 1. Check dashboard is enabled
.with_dashboard(true)

// 2. Verify port is accessible
curl http://localhost:3000

// 3. Check browser console for errors
// F12 -> Console

// 4. Verify WebSocket URL in config
.with_websocket_url("ws://localhost:8080/ws")
```

---

## Production Deployment

### Docker Compose Example

```yaml
version: '3.8'

services:
  llm-test-bench:
    image: llm-test-bench:latest
    ports:
      - "9090:9090"  # Prometheus
      - "8080:8080"  # WebSocket
      - "3000:3000"  # Dashboard
    environment:
      - LTB_PROMETHEUS_ENABLED=true
      - LTB_WEBSOCKET_ENABLED=true
      - LTB_DASHBOARD_ENABLED=true
      - LTB_RETENTION_PERIOD=3600

  prometheus:
    image: prom/prometheus:latest
    ports:
      - "9091:9090"
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml
      - prometheus-data:/prometheus
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
      - '--storage.tsdb.path=/prometheus'

  grafana:
    image: grafana/grafana:latest
    ports:
      - "3001:3000"
    volumes:
      - grafana-data:/var/lib/grafana
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=admin

volumes:
  prometheus-data:
  grafana-data:
```

### Kubernetes Example

```yaml
apiVersion: v1
kind: Service
metadata:
  name: llm-test-bench-monitoring
  labels:
    app: llm-test-bench
spec:
  type: ClusterIP
  ports:
    - name: prometheus
      port: 9090
      targetPort: 9090
    - name: websocket
      port: 8080
      targetPort: 8080
    - name: dashboard
      port: 3000
      targetPort: 3000
  selector:
    app: llm-test-bench

---
apiVersion: monitoring.coreos.com/v1
kind: ServiceMonitor
metadata:
  name: llm-test-bench
spec:
  selector:
    matchLabels:
      app: llm-test-bench
  endpoints:
    - port: prometheus
      interval: 30s
```

---

## Summary

The LLM Test Bench monitoring system provides:

✅ **Prometheus Integration**: Industry-standard metrics export
✅ **Real-time Dashboards**: Live WebSocket-based visualization
✅ **Event System**: Flexible pub/sub architecture
✅ **Provider Monitoring**: Automatic instrumentation
✅ **Benchmark Tracking**: Live progress updates
✅ **Production Ready**: Battle-tested and scalable

**Next Steps**:
1. [Configure Grafana dashboards](#grafana-dashboard)
2. [Set up alerting rules](ALERTING.md)
3. [Integrate with your CI/CD](CI_CD.md)
4. [Explore advanced analytics](ANALYTICS.md)

For more information, see the [API documentation](https://docs.rs/llm-test-bench-core).

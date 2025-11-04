# Phase 5 Quick Reference Guide
## Technical Decision Reference

**Version:** 1.0
**Date:** November 4, 2025

---

## Module Ownership & Locations

| Module | Location | Purpose | Key Files |
|--------|----------|---------|-----------|
| **Providers** | `core/src/providers/` | LLM provider integrations | `gemini.rs`, `cohere.rs`, `mistral.rs`, `local/ollama.rs`, `local/llamacpp.rs`, `registry.rs` |
| **Multi-Modal** | `core/src/multimodal/` | Vision & audio evaluation | `vision.rs`, `audio.rs`, `dataset.rs`, `metrics.rs` |
| **Monitoring** | `core/src/monitoring/` | Real-time monitoring | `realtime.rs`, `metrics.rs`, `alerts.rs`, `collectors.rs` |
| **Plugins** | `core/src/plugins/` | Plugin system | `loader.rs`, `registry.rs`, `api.rs`, `sandbox.rs` |
| **Integrations** | `core/src/integrations/` | External integrations | `langchain.rs`, `llamaindex.rs`, `mlflow.rs`, `wandb.rs` |
| **Storage** | `core/src/storage/` | Database layer | `database.rs`, `postgres.rs`, `sqlite.rs`, `migrations/` |
| **Server** | `server/` | API server (new crate) | `api/rest.rs`, `api/graphql.rs`, `api/websocket.rs`, `auth/` |

---

## New Dependencies

### Core Dependencies (add to `core/Cargo.toml`)

```toml
# Multi-modal
image = { version = "0.24", features = ["jpeg", "png", "webp"] }
symphonia = { version = "0.5", features = ["all"] }

# Plugins
wasmer = "4.2"
wasmer-compiler-cranelift = "4.2"

# Database
sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "postgres", "sqlite", "chrono", "uuid", "json"] }
refinery = { version = "0.8", features = ["tokio-postgres"] }

# Caching
redis = { version = "0.24", features = ["tokio-comp", "connection-manager"] }

# Observability
opentelemetry = { version = "0.21", features = ["rt-tokio"] }
prometheus = "0.13"
```

### Server Dependencies (new `server/Cargo.toml`)

```toml
[package]
name = "llm-test-bench-server"
version = "0.5.0"
edition = "2021"

[dependencies]
# Core
llm-test-bench-core = { path = "../core" }

# Web framework
axum = { version = "0.7", features = ["ws", "multipart", "macros"] }
tower = { version = "0.4", features = ["limit", "timeout", "retry"] }
tower-http = { version = "0.5", features = ["cors", "compression", "trace"] }
tower-governor = "0.3"

# GraphQL
async-graphql = { version = "7.0", features = ["chrono", "uuid"] }
async-graphql-axum = "7.0"

# WebSocket
tokio-tungstenite = "0.21"

# Database
sqlx = { workspace = true }

# Auth
jsonwebtoken = "9.2"
argon2 = "0.5"

# Async
tokio = { workspace = true }
```

---

## API Endpoints Reference

### REST API (`/api/v1`)

#### Benchmarks
```
POST   /api/v1/benchmarks          - Create benchmark
GET    /api/v1/benchmarks/:id      - Get benchmark status
GET    /api/v1/benchmarks          - List benchmarks
POST   /api/v1/benchmarks/:id/cancel - Cancel benchmark
```

#### Results
```
GET    /api/v1/results             - Query results (with filters)
GET    /api/v1/results/:id         - Get specific result
POST   /api/v1/results/export      - Export results (CSV, JSON)
```

#### Providers
```
GET    /api/v1/providers           - List all providers
GET    /api/v1/providers/:name     - Get provider details
GET    /api/v1/providers/:name/models - List models for provider
POST   /api/v1/providers/:name/health - Health check provider
```

#### Evaluation
```
POST   /api/v1/evaluate            - Evaluate single response
POST   /api/v1/compare             - Compare multiple models
```

#### Monitoring
```
GET    /api/v1/metrics             - Prometheus metrics
GET    /api/v1/health              - Health check
GET    /api/v1/health/live         - Liveness probe (K8s)
GET    /api/v1/health/ready        - Readiness probe (K8s)
```

#### Plugins
```
GET    /api/v1/plugins             - List plugins
POST   /api/v1/plugins             - Upload plugin (multipart)
GET    /api/v1/plugins/:name       - Get plugin info
DELETE /api/v1/plugins/:name       - Uninstall plugin
POST   /api/v1/plugins/:name/reload - Hot reload plugin
```

#### Auth
```
POST   /api/v1/auth/login          - Login (get JWT)
POST   /api/v1/auth/refresh        - Refresh token
POST   /api/v1/auth/logout         - Logout (revoke token)
POST   /api/v1/auth/apikey         - Generate API key
DELETE /api/v1/auth/apikey         - Revoke API key
```

### GraphQL API (`/graphql`)

```graphql
# Queries
query {
  benchmark(id: UUID!)
  benchmarks(filters: BenchmarkFilters, pagination: Pagination)
  results(filters: ResultFilters)
  provider(name: String!)
  providers
  metrics(labels: [Label!])
}

# Mutations
mutation {
  createBenchmark(input: CreateBenchmarkInput!)
  cancelBenchmark(id: UUID!)
  uploadPlugin(file: Upload!)
  updateSettings(input: SettingsInput!)
}

# Subscriptions (real-time)
subscription {
  benchmarkProgress(id: UUID!)
  metricsStream
  alertsStream
}
```

### WebSocket API (`/ws`)

```javascript
// Connect
const ws = new WebSocket('ws://localhost:8080/ws');

// Subscribe
ws.send(JSON.stringify({
  type: 'Subscribe',
  topics: ['benchmark.progress', 'metrics.*', 'alerts.*']
}));

// Receive events
ws.onmessage = (event) => {
  const message = JSON.parse(event.data);
  if (message.type === 'Event') {
    console.log(message.payload);
  }
};
```

---

## Database Schema Quick Reference

### Main Tables

#### `users`
```sql
id, username, email, password_hash, role, api_key_hash, created_at, updated_at
```

#### `providers`
```sql
id, name, type, config (JSONB), capabilities (JSONB), api_version, is_active
```

#### `sessions`
```sql
id, name, description, user_id, status, config (JSONB), tags, created_at
```

#### `results_v5`
```sql
id, session_id, provider_id, dataset_id, test_id, status, model, prompt, response,
duration_ms, total_tokens, evaluation_scores (JSONB), multi_modal_inputs (JSONB),
plugin_metrics (JSONB), trace_id, span_id, tags, metadata (JSONB), cost_details (JSONB)
```

#### `alerts`
```sql
id, rule_id, severity, message, details (JSONB), triggered_at, resolved_at
```

#### `plugins`
```sql
id, name, version, author, plugin_type, capabilities (JSONB), file_path, is_enabled
```

#### `model_profiles`
```sql
id, provider_id, model_name, typical_quality, avg_latency_ms, cost_per_1k_tokens,
context_limit, strengths (JSONB), sample_count
```

### Key Indexes
```sql
-- Performance indexes
CREATE INDEX idx_results_session_created ON results_v5(session_id, created_at DESC);
CREATE INDEX idx_results_trace_id ON results_v5(trace_id);
CREATE INDEX idx_results_tags ON results_v5 USING GIN(tags);
CREATE INDEX idx_results_evaluation_scores ON results_v5 USING GIN(evaluation_scores);

-- Full-text search
CREATE INDEX idx_results_prompt_text ON results_v5 USING GIN(to_tsvector('english', prompt));
```

---

## Configuration Schema

### New Phase 5 Config Sections

```toml
# config.toml (extends Phase 4)

# NEW: Multi-modal evaluation
[multimodal]
enable_vision = true
enable_audio = true
max_image_size_mb = 10
max_audio_duration_seconds = 300
supported_image_formats = ["jpeg", "png", "webp"]
supported_audio_formats = ["wav", "mp3", "flac"]

# NEW: Monitoring
[monitoring]
enable_realtime = true
enable_prometheus = true
enable_tracing = true
websocket_port = 8081
metrics_port = 9090
alert_rules = [
    { name = "high_latency", condition = "p95_latency_ms > 5000", severity = "warning" },
    { name = "high_cost", condition = "hourly_cost > 10.0", severity = "error" },
]

# NEW: Plugins
[plugins]
enable_plugins = true
plugin_directory = "./plugins"
max_plugin_memory_mb = 100
plugin_timeout_seconds = 30
auto_reload = false  # Hot reload on file change

# NEW: Storage
[storage]
backend = "postgresql"  # or "sqlite"
database_url = "postgresql://localhost/llm_test_bench"
max_connections = 20
connection_timeout_seconds = 5
enable_migrations = true

# NEW: Server
[server]
host = "0.0.0.0"
port = 8080
enable_rest = true
enable_graphql = true
enable_websocket = true
cors_allowed_origins = ["*"]
rate_limit_requests_per_minute = 100

# NEW: Distributed (optional)
[distributed]
enable_distributed = false
coordinator_url = "http://localhost:50051"
worker_count = 5
worker_capabilities = { max_concurrent_tasks = 10, has_gpu = false }

# NEW: Integrations (optional)
[integrations.langchain]
enable = false
api_url = "https://api.langchain.com"
project_name = "my-project"

[integrations.mlflow]
enable = false
tracking_uri = "http://localhost:5000"
experiment_name = "llm-benchmarks"

[integrations.wandb]
enable = false
project = "llm-eval"
entity = "my-team"
```

---

## Plugin Development Quick Start

### 1. Create Plugin Manifest (`plugin.toml`)

```toml
[package]
name = "custom-evaluator"
version = "1.0.0"
author = "Your Name"
description = "Custom evaluation metric"

[plugin]
type = "evaluator"  # or "transformer", "exporter", "provider", "notifier"
entry_point = "main.wasm"

[capabilities]
requires_llm = true
supports_batch = true
max_batch_size = 10

[config_schema]
schema = """
{
  "type": "object",
  "properties": {
    "threshold": { "type": "number", "minimum": 0.0, "maximum": 1.0 }
  }
}
"""
```

### 2. Implement Plugin (Rust)

```rust
use llm_test_bench_plugin_sdk::*;

#[derive(Default)]
pub struct CustomEvaluator;

#[async_trait]
impl Plugin for CustomEvaluator {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "custom-evaluator".to_string(),
            version: "1.0.0".to_string(),
            author: "Your Name".to_string(),
            description: "Custom metric".to_string(),
            plugin_type: PluginType::Evaluator,
            capabilities: vec![Capability::RequiresLLM],
        }
    }

    async fn execute(&self, context: PluginContext) -> Result<PluginResult, PluginError> {
        // Your custom logic here
        let score = 0.85;

        Ok(PluginResult {
            success: true,
            output: serde_json::json!({ "score": score }),
            metrics: vec![("custom_metric".to_string(), score)].into_iter().collect(),
            logs: vec!["Evaluation complete".to_string()],
        })
    }

    // Other required methods...
}

declare_plugin!(CustomEvaluator);
```

### 3. Build Plugin

```bash
cargo build --target wasm32-wasi --release
cp target/wasm32-wasi/release/custom_evaluator.wasm ./plugins/
```

### 4. Load Plugin

```bash
llm-test-bench plugin install ./plugins/custom_evaluator.wasm
llm-test-bench plugin list
```

---

## Distributed Architecture Quick Start

### 1. Start Coordinator

```bash
# Using Docker
docker run -p 50051:50051 llm-test-bench/coordinator:latest

# Using Kubernetes
kubectl apply -f k8s/coordinator.yaml
```

### 2. Start Workers

```bash
# Using Docker Compose (auto-scaling)
docker-compose up --scale worker=5

# Using Kubernetes (with HPA)
kubectl apply -f k8s/worker.yaml
kubectl apply -f k8s/hpa.yaml
```

### 3. Submit Distributed Benchmark

```bash
# CLI
llm-test-bench bench \
  --dataset large-dataset.json \
  --distributed \
  --workers 10

# API
curl -X POST http://localhost:8080/api/v1/benchmarks \
  -H "Content-Type: application/json" \
  -d '{
    "dataset": "large-dataset.json",
    "distributed": true,
    "workers": 10
  }'
```

---

## Monitoring & Observability

### Prometheus Metrics

**Available at:** `http://localhost:9090/metrics`

**Key Metrics:**
```prometheus
# Benchmark metrics
llm_benchmark_total                  # Total benchmarks
llm_benchmark_duration_seconds       # Benchmark duration histogram

# Test metrics
llm_test_total{status="success|failure"}
llm_test_duration_seconds

# Evaluation metrics
llm_evaluation_score{metric="faithfulness|relevance|coherence"}
llm_evaluation_cache_hits_total
llm_evaluation_cache_misses_total

# Provider metrics
llm_provider_requests_total{provider="openai|anthropic"}
llm_provider_errors_total{provider,error_type}
llm_provider_latency_seconds{provider,model}
llm_provider_tokens_total{provider,model,type="prompt|completion"}
llm_provider_cost_total{provider,model}

# System metrics
llm_active_sessions
llm_active_benchmarks
llm_memory_usage_bytes
llm_cpu_usage_percent

# API metrics
llm_api_requests_total{method,path,status}
llm_api_duration_seconds{method,path}
llm_websocket_connections
```

### Grafana Dashboards

**Provided dashboards:**
1. **Overview:** System health, active sessions, throughput
2. **Benchmarks:** Success rate, duration, cost over time
3. **Providers:** Latency, errors, cost by provider/model
4. **Evaluation:** Score distributions, cache hit rate
5. **API:** Request rate, latency, errors

**Access:** `http://localhost:3000` (admin/admin)

### OpenTelemetry Traces

**Trace format:**
```json
{
  "trace_id": "550e8400-e29b-41d4-a716-446655440000",
  "span_id": "f47ac10b-58cc-4372-a567-0e02b2c3d479",
  "name": "benchmark_execution",
  "spans": [
    {"name": "provider_call", "duration_ms": 1234},
    {"name": "evaluation", "duration_ms": 567}
  ]
}
```

**View traces:** Export to Jaeger or Zipkin

---

## Security Best Practices

### JWT Configuration

```rust
// Generate secure secret (32+ bytes)
use rand::RngCore;
let mut secret = [0u8; 32];
rand::thread_rng().fill_bytes(&mut secret);

// Environment variable
export JWT_SECRET="your-secure-secret-here"
```

### API Key Generation

```bash
# Generate API key for user
llm-test-bench auth generate-key --user-id <uuid>

# Use API key
curl -H "Authorization: Bearer ltb_xxxxxxxxxxxxx" \
  http://localhost:8080/api/v1/benchmarks
```

### RBAC Configuration

```toml
[security]
enable_rbac = true
default_role = "viewer"  # New users default to viewer

[[security.roles]]
name = "admin"
permissions = ["*"]  # All permissions

[[security.roles]]
name = "user"
permissions = [
    "benchmark.create",
    "benchmark.read",
    "results.read",
    "provider.read"
]

[[security.roles]]
name = "viewer"
permissions = ["results.read"]
```

### Rate Limiting

```toml
[server.rate_limit]
# Global rate limit
requests_per_minute = 100
burst_size = 20

# Per-endpoint rate limits
[[server.rate_limit.endpoints]]
path = "/api/v1/benchmarks"
requests_per_minute = 10

[[server.rate_limit.endpoints]]
path = "/api/v1/evaluate"
requests_per_minute = 50
```

---

## Performance Tuning

### Database Connection Pool

```rust
// Optimal pool size = (number_of_cores * 2) + effective_spindle_count
let pool_size = num_cpus::get() * 2 + 1;

let pool = deadpool_postgres::Config {
    max_size: pool_size,
    timeouts: Timeouts {
        wait: Some(Duration::from_secs(5)),
        create: Some(Duration::from_secs(5)),
        recycle: Some(Duration::from_secs(5)),
    },
}.create_pool(Runtime::Tokio1)?;
```

### Redis Caching

```rust
// Cache TTL by data type
let cache_ttl = match data_type {
    DataType::EvaluationResult => Duration::from_hours(24),
    DataType::ProviderMetadata => Duration::from_minutes(30),
    DataType::SessionData => Duration::from_hours(1),
};

// Cache key design
let cache_key = format!(
    "eval:{}:{}:{}",
    metric_name,
    hash_prompt(&prompt),  // SHA-256 hash
    hash_response(&response)
);
```

### Async Task Limits

```rust
// Limit concurrent LLM API calls
let semaphore = Arc::new(Semaphore::new(10));  // Max 10 concurrent

for request in requests {
    let permit = semaphore.clone().acquire_owned().await?;
    tokio::spawn(async move {
        let _permit = permit;  // Held until task completes
        provider.complete(request).await
    });
}
```

---

## Troubleshooting

### Common Issues

#### 1. WebSocket Disconnects

**Problem:** WebSocket connections dropping frequently

**Solutions:**
```toml
[server.websocket]
ping_interval_seconds = 30
pong_timeout_seconds = 5
max_message_size_mb = 10
```

#### 2. Database Connection Pool Exhausted

**Problem:** "timeout waiting for connection from pool"

**Solutions:**
```toml
[storage]
max_connections = 50  # Increase pool size
connection_timeout_seconds = 10  # Increase timeout
```

#### 3. High Memory Usage

**Problem:** Server using too much memory

**Solutions:**
```toml
[evaluation]
cache_size = 1000  # Reduce cache size (from default 10000)

[plugins]
max_plugin_memory_mb = 50  # Reduce plugin memory
```

#### 4. Slow Evaluation

**Problem:** Evaluations taking too long

**Solutions:**
```toml
[evaluation]
parallel_evaluations = 10  # Increase parallelism (default 5)
enable_caching = true
cache_ttl_hours = 24
```

#### 5. Plugin Won't Load

**Problem:** "Failed to load plugin: invalid WASM"

**Solutions:**
```bash
# Ensure correct build target
cargo build --target wasm32-wasi --release

# Check plugin manifest
llm-test-bench plugin validate ./plugin.toml

# View plugin logs
llm-test-bench plugin logs <plugin-name>
```

---

## Version Compatibility

| Phase 5 Version | Minimum Rust | Compatible Phase 4 | PostgreSQL | Redis |
|----------------|--------------|---------------------|------------|-------|
| 0.5.0          | 1.75.0       | 0.4.0+              | 12+        | 6+    |
| 0.5.1          | 1.75.0       | 0.4.0+              | 12+        | 6+    |

---

## Quick Links

- **Full Architecture:** [PHASE5_TECHNICAL_ARCHITECTURE.md](./PHASE5_TECHNICAL_ARCHITECTURE.md)
- **Executive Summary:** [PHASE5_ARCHITECTURE_SUMMARY.md](./PHASE5_ARCHITECTURE_SUMMARY.md)
- **Phase 4 Baseline:** [PHASE4_COMPLETE.md](../PHASE4_COMPLETE.md)
- **API Documentation:** `/graphql/playground` (when server running)
- **Metrics:** `http://localhost:9090/metrics`
- **Grafana:** `http://localhost:3000`

---

**Document Version:** 1.0
**Last Updated:** November 4, 2025
**Maintained By:** Technical Architecture Team

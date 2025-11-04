# Phase 5 Technical Architecture
## LLM Test Bench - Production-Scale Features

**Version:** 1.0
**Date:** November 4, 2025
**Status:** Architecture Design
**Target:** Production-Ready Implementation

---

## Executive Summary

This document defines the comprehensive technical architecture for Phase 5 of the LLM Test Bench, building upon the solid foundation of Phases 1-4. Phase 5 introduces enterprise-scale features including additional LLM providers, multi-modal evaluation, real-time monitoring, plugin architecture, distributed benchmarking, and full-stack API server capabilities.

### Key Objectives

1. **Provider Expansion**: Add 6+ new LLM providers (Gemini, Cohere, Mistral, Ollama, LlamaCpp, custom)
2. **Multi-Modal Support**: Vision and audio evaluation capabilities
3. **Real-Time Monitoring**: WebSocket-based live dashboards and alerting
4. **Plugin Architecture**: WASM-based extensibility for custom metrics
5. **Integration Layer**: Langchain, LlamaIndex, MLflow, W&B connectors
6. **API Server**: REST/GraphQL/WebSocket APIs with authentication
7. **Distributed Architecture**: Multi-worker benchmarking at scale
8. **Database Layer**: PostgreSQL/SQLite for persistent storage
9. **Advanced Observability**: Prometheus metrics, OpenTelemetry tracing
10. **Enterprise Security**: RBAC, OAuth, audit logging

---

## Table of Contents

1. [Module Architecture](#1-module-architecture)
2. [Data Architecture](#2-data-architecture)
3. [Integration Architecture](#3-integration-architecture)
4. [Real-Time Architecture](#4-real-time-architecture)
5. [Distributed Architecture](#5-distributed-architecture)
6. [API Server Architecture](#6-api-server-architecture)
7. [Deployment Architecture](#7-deployment-architecture)
8. [Security Architecture](#8-security-architecture)
9. [Technology Stack](#9-technology-stack)
10. [Performance Considerations](#10-performance-considerations)
11. [Migration Strategy](#11-migration-strategy)
12. [Development Roadmap](#12-development-roadmap)

---

## 1. Module Architecture

### 1.1 Core Provider Expansion (`core/src/providers/`)

```rust
providers/
├── mod.rs                  # Public module exports
├── traits.rs               # Enhanced Provider trait (existing)
├── types.rs                # Common types (existing)
├── error.rs                # Error handling (existing)
├── factory.rs              # Enhanced factory with registry (existing)
│
├── openai.rs               # Existing - OpenAI provider
├── anthropic.rs            # Existing - Anthropic provider
│
├── gemini.rs               # NEW - Google Gemini provider
├── cohere.rs               # NEW - Cohere provider
├── mistral.rs              # NEW - Mistral AI provider
│
├── local/                  # NEW - Local model providers
│   ├── mod.rs
│   ├── ollama.rs           # Ollama integration
│   └── llamacpp.rs         # llama.cpp integration
│
├── registry.rs             # NEW - Dynamic provider registry
├── capabilities.rs         # NEW - Capability detection
└── versioning.rs           # NEW - Provider API versioning
```

#### Enhanced Provider Trait

```rust
use async_trait::async_trait;

/// Capabilities that a provider may support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderCapabilities {
    pub supports_streaming: bool,
    pub supports_vision: bool,
    pub supports_audio: bool,
    pub supports_function_calling: bool,
    pub supports_embeddings: bool,
    pub supports_fine_tuning: bool,
    pub max_context_length: usize,
    pub supported_modalities: Vec<Modality>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Modality {
    Text,
    Image,
    Audio,
    Video,
}

/// Enhanced provider trait for Phase 5
#[async_trait]
pub trait Provider: Send + Sync {
    // Existing methods from Phase 4
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse, ProviderError>;
    async fn stream(&self, request: CompletionRequest) -> Result<ResponseStream, ProviderError>;
    fn supported_models(&self) -> Vec<ModelInfo>;
    fn max_context_length(&self, model: &str) -> Option<usize>;
    fn name(&self) -> &str;
    async fn validate_config(&self) -> Result<(), ProviderError>;
    fn estimate_tokens(&self, text: &str, model: &str) -> Result<usize, ProviderError>;

    // NEW methods for Phase 5

    /// Returns the capabilities of this provider
    fn capabilities(&self) -> ProviderCapabilities;

    /// Process multi-modal input (images, audio, etc.)
    async fn complete_multimodal(
        &self,
        request: MultiModalRequest,
    ) -> Result<CompletionResponse, ProviderError>;

    /// Generate embeddings for given texts
    async fn embed(&self, texts: Vec<String>, model: &str) -> Result<Vec<Vec<f64>>, ProviderError>;

    /// Check provider health and availability
    async fn health_check(&self) -> Result<HealthStatus, ProviderError>;

    /// Get provider version and API version
    fn version_info(&self) -> VersionInfo;
}

/// Provider registry for dynamic provider management
pub struct ProviderRegistry {
    providers: Arc<RwLock<HashMap<String, Arc<dyn Provider>>>>,
    capabilities_cache: Arc<RwLock<HashMap<String, ProviderCapabilities>>>,
}

impl ProviderRegistry {
    pub fn new() -> Self;

    /// Register a provider
    pub fn register(&mut self, name: String, provider: Arc<dyn Provider>);

    /// Get provider by name
    pub fn get(&self, name: &str) -> Option<Arc<dyn Provider>>;

    /// List all registered providers
    pub fn list(&self) -> Vec<String>;

    /// Find providers by capability
    pub fn find_by_capability(&self, capability: Capability) -> Vec<String>;

    /// Auto-discover providers from configuration
    pub async fn discover_from_config(&mut self, config: &Config) -> Result<()>;
}
```

#### New Provider Implementations

**Gemini Provider** (`gemini.rs`):
```rust
pub struct GeminiProvider {
    api_key: String,
    base_url: String,
    client: reqwest::Client,
    config: ProviderConfig,
}

impl GeminiProvider {
    // Supports: gemini-1.5-pro, gemini-1.5-flash, gemini-1.0-pro
    // Features: Vision, audio, long context (2M tokens)
    // Rate limits: 60 RPM (free), 1000 RPM (paid)
}
```

**Cohere Provider** (`cohere.rs`):
```rust
pub struct CohereProvider {
    api_key: String,
    base_url: String,
    client: reqwest::Client,
    config: ProviderConfig,
}

impl CohereProvider {
    // Supports: command, command-light, command-r, command-r-plus
    // Features: Embeddings, reranking, classification
    // Rate limits: 100 RPM (trial), 10000 RPM (production)
}
```

**Mistral Provider** (`mistral.rs`):
```rust
pub struct MistralProvider {
    api_key: String,
    base_url: String,
    client: reqwest::Client,
    config: ProviderConfig,
}

impl MistralProvider {
    // Supports: mistral-tiny, mistral-small, mistral-medium, mistral-large
    // Features: Function calling, JSON mode, embeddings
    // Rate limits: 60 RPM (free), 500 RPM (paid)
}
```

**Ollama Provider** (`local/ollama.rs`):
```rust
pub struct OllamaProvider {
    base_url: String,  // Default: http://localhost:11434
    client: reqwest::Client,
    config: ProviderConfig,
}

impl OllamaProvider {
    // Supports: llama2, mistral, codellama, vicuna, etc.
    // Features: Local hosting, no API key, unlimited rate
    // Connection: HTTP API to local Ollama server
}
```

**LlamaCpp Provider** (`local/llamacpp.rs`):
```rust
pub struct LlamaCppProvider {
    binary_path: PathBuf,
    model_path: PathBuf,
    config: ProviderConfig,
}

impl LlamaCppProvider {
    // Supports: Any GGUF model
    // Features: Direct binary execution, maximum performance
    // Connection: Process spawning with stdout/stderr capture
}
```

---

### 1.2 Multi-Modal Module (`core/src/multimodal/`)

```rust
multimodal/
├── mod.rs              # Public exports
├── vision.rs           # Vision evaluation
├── audio.rs            # Audio evaluation
├── dataset.rs          # Multi-modal datasets
├── metrics.rs          # Multi-modal metrics
└── types.rs            # Common types
```

#### Vision Evaluation (`vision.rs`)

```rust
use image::{DynamicImage, ImageFormat};

/// Vision evaluation capabilities
pub struct VisionEvaluator {
    provider: Arc<dyn Provider>,
    config: VisionConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisionRequest {
    pub prompt: String,
    pub images: Vec<ImageInput>,
    pub model: String,
    pub evaluation_criteria: VisionCriteria,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImageInput {
    /// Base64-encoded image data
    Base64 { data: String, format: ImageFormat },

    /// Image URL (http/https)
    Url(String),

    /// Local file path
    FilePath(PathBuf),

    /// In-memory image
    Image(DynamicImage),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisionCriteria {
    /// Accuracy of image description
    pub description_accuracy: bool,

    /// Object detection correctness
    pub object_detection: bool,

    /// Text extraction (OCR) accuracy
    pub text_extraction: bool,

    /// Spatial relationship understanding
    pub spatial_reasoning: bool,

    /// Color and visual attribute recognition
    pub visual_attributes: bool,
}

impl VisionEvaluator {
    /// Evaluate vision model response
    pub async fn evaluate(
        &self,
        request: VisionRequest,
    ) -> Result<VisionEvaluationResult, EvaluationError>;

    /// Batch evaluate multiple vision requests
    pub async fn evaluate_batch(
        &self,
        requests: Vec<VisionRequest>,
    ) -> Result<Vec<VisionEvaluationResult>, EvaluationError>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisionEvaluationResult {
    pub overall_score: f64,
    pub description_accuracy: Option<f64>,
    pub object_detection_score: Option<f64>,
    pub text_extraction_score: Option<f64>,
    pub spatial_reasoning_score: Option<f64>,
    pub visual_attributes_score: Option<f64>,
    pub explanation: String,
    pub detected_objects: Vec<DetectedObject>,
    pub extracted_text: Option<String>,
}
```

#### Audio Evaluation (`audio.rs`)

```rust
/// Audio evaluation capabilities
pub struct AudioEvaluator {
    provider: Arc<dyn Provider>,
    config: AudioConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioRequest {
    pub audio: AudioInput,
    pub task: AudioTask,
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AudioInput {
    /// Audio file path (WAV, MP3, FLAC, etc.)
    FilePath(PathBuf),

    /// Audio bytes with format
    Bytes { data: Vec<u8>, format: AudioFormat },

    /// Audio URL
    Url(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AudioFormat {
    Wav,
    Mp3,
    Flac,
    Ogg,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AudioTask {
    /// Speech-to-text transcription
    Transcription { language: Option<String> },

    /// Audio classification
    Classification { categories: Vec<String> },

    /// Speaker identification
    SpeakerIdentification,

    /// Emotion detection
    EmotionDetection,
}

impl AudioEvaluator {
    /// Evaluate audio model response
    pub async fn evaluate(
        &self,
        request: AudioRequest,
        expected_output: String,
    ) -> Result<AudioEvaluationResult, EvaluationError>;
}
```

---

### 1.3 Monitoring Module (`core/src/monitoring/`)

```rust
monitoring/
├── mod.rs              # Public exports
├── realtime.rs         # WebSocket server
├── metrics.rs          # Prometheus metrics
├── alerts.rs           # Alert system
├── collectors.rs       # Metric collectors
└── traces.rs           # Distributed tracing
```

#### Real-Time Monitoring (`realtime.rs`)

```rust
use tokio_tungstenite::tungstenite::protocol::Message;

/// WebSocket server for real-time updates
pub struct RealtimeServer {
    addr: SocketAddr,
    connections: Arc<Mutex<HashMap<Uuid, Connection>>>,
    event_bus: Arc<EventBus>,
    config: RealtimeConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RealtimeEvent {
    /// Benchmark started
    BenchmarkStarted {
        session_id: Uuid,
        dataset: String,
        providers: Vec<String>,
        total_tests: usize,
    },

    /// Benchmark progress update
    BenchmarkProgress {
        session_id: Uuid,
        completed: usize,
        total: usize,
        current_test: String,
        progress_percent: f32,
    },

    /// Test completed
    TestCompleted {
        session_id: Uuid,
        test_id: String,
        status: TestStatus,
        duration_ms: u64,
    },

    /// Benchmark completed
    BenchmarkCompleted {
        session_id: Uuid,
        summary: ResultSummary,
        duration: Duration,
    },

    /// Metric update
    MetricUpdate {
        metric_name: String,
        value: f64,
        timestamp: DateTime<Utc>,
    },

    /// Alert triggered
    Alert {
        severity: AlertSeverity,
        message: String,
        details: HashMap<String, serde_json::Value>,
    },

    /// System health update
    HealthUpdate {
        cpu_usage: f32,
        memory_usage: f64,
        active_tasks: usize,
    },
}

impl RealtimeServer {
    /// Start the WebSocket server
    pub async fn start(&self) -> Result<(), MonitoringError>;

    /// Broadcast event to all connected clients
    pub async fn broadcast(&self, event: RealtimeEvent) -> Result<(), MonitoringError>;

    /// Send event to specific client
    pub async fn send_to(
        &self,
        connection_id: Uuid,
        event: RealtimeEvent,
    ) -> Result<(), MonitoringError>;

    /// Subscribe client to event types
    pub async fn subscribe(
        &self,
        connection_id: Uuid,
        event_types: Vec<String>,
    ) -> Result<(), MonitoringError>;
}

/// Event bus for internal event routing
pub struct EventBus {
    channels: Arc<RwLock<HashMap<String, broadcast::Sender<RealtimeEvent>>>>,
}

impl EventBus {
    pub fn new() -> Self;

    /// Publish event to topic
    pub async fn publish(&self, topic: &str, event: RealtimeEvent) -> Result<()>;

    /// Subscribe to topic
    pub async fn subscribe(&self, topic: &str) -> broadcast::Receiver<RealtimeEvent>;
}
```

#### Prometheus Metrics (`metrics.rs`)

```rust
use prometheus::{
    Registry, Counter, Gauge, Histogram, HistogramVec,
    CounterVec, GaugeVec, IntCounter, IntGauge,
};

/// Prometheus metrics collector
pub struct MetricsCollector {
    registry: Registry,

    // Benchmark metrics
    pub benchmark_total: IntCounter,
    pub benchmark_duration: HistogramVec,
    pub benchmark_failures: CounterVec,

    // Test metrics
    pub test_total: CounterVec,
    pub test_duration: HistogramVec,
    pub test_failures: CounterVec,

    // Evaluation metrics
    pub evaluation_scores: GaugeVec,
    pub evaluation_duration: HistogramVec,
    pub evaluation_cache_hits: Counter,
    pub evaluation_cache_misses: Counter,

    // Provider metrics
    pub provider_requests: CounterVec,
    pub provider_errors: CounterVec,
    pub provider_latency: HistogramVec,
    pub provider_tokens: CounterVec,
    pub provider_cost: CounterVec,

    // System metrics
    pub active_sessions: IntGauge,
    pub active_benchmarks: IntGauge,
    pub memory_usage: Gauge,
    pub cpu_usage: Gauge,

    // API metrics (Phase 5 server)
    pub api_requests: CounterVec,
    pub api_duration: HistogramVec,
    pub api_errors: CounterVec,
    pub websocket_connections: IntGauge,
}

impl MetricsCollector {
    pub fn new() -> Result<Self, MetricsError>;

    /// Record benchmark start
    pub fn record_benchmark_start(&self, dataset: &str, provider: &str);

    /// Record benchmark completion
    pub fn record_benchmark_complete(
        &self,
        dataset: &str,
        provider: &str,
        duration: Duration,
        success: bool,
    );

    /// Record evaluation score
    pub fn record_evaluation_score(
        &self,
        metric: &str,
        score: f64,
    );

    /// Export metrics in Prometheus format
    pub fn export(&self) -> Result<String, MetricsError>;

    /// Get metrics for specific labels
    pub fn get_metrics(&self, labels: HashMap<String, String>) -> Vec<Metric>;
}
```

#### Alert System (`alerts.rs`)

```rust
/// Alert severity levels
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum AlertSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Alert rule definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRule {
    pub id: Uuid,
    pub name: String,
    pub condition: AlertCondition,
    pub severity: AlertSeverity,
    pub message_template: String,
    pub cooldown: Duration,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertCondition {
    /// Metric exceeds threshold
    MetricThreshold {
        metric: String,
        operator: Operator,
        threshold: f64,
        duration: Duration,
    },

    /// Failure rate exceeds threshold
    FailureRate {
        window: Duration,
        threshold: f64,
    },

    /// Cost exceeds budget
    CostBudget {
        window: Duration,
        budget: f64,
    },

    /// Latency exceeds threshold
    LatencyThreshold {
        percentile: f64,
        threshold: Duration,
    },
}

/// Alert manager
pub struct AlertManager {
    rules: Arc<RwLock<Vec<AlertRule>>>,
    triggered_alerts: Arc<RwLock<HashMap<Uuid, Alert>>>,
    notifiers: Vec<Box<dyn AlertNotifier>>,
}

impl AlertManager {
    /// Add alert rule
    pub fn add_rule(&mut self, rule: AlertRule);

    /// Evaluate all rules against current metrics
    pub async fn evaluate(&self, metrics: &MetricsSnapshot) -> Vec<Alert>;

    /// Trigger alert
    pub async fn trigger_alert(&self, alert: Alert) -> Result<()>;

    /// Resolve alert
    pub async fn resolve_alert(&self, alert_id: Uuid) -> Result<()>;
}

/// Alert notifier trait
#[async_trait]
pub trait AlertNotifier: Send + Sync {
    async fn notify(&self, alert: &Alert) -> Result<()>;
}

/// Webhook notifier
pub struct WebhookNotifier {
    url: String,
    client: reqwest::Client,
}

/// Slack notifier
pub struct SlackNotifier {
    webhook_url: String,
    channel: String,
}

/// Email notifier
pub struct EmailNotifier {
    smtp_host: String,
    from: String,
    to: Vec<String>,
}
```

---

### 1.4 Plugin System (`core/src/plugins/`)

```rust
plugins/
├── mod.rs              # Public exports
├── loader.rs           # Dynamic plugin loading
├── registry.rs         # Plugin registry
├── api.rs              # Plugin API definition
├── sandbox.rs          # WASM sandbox
└── builtin/            # Built-in plugins
    ├── example.rs
    └── template.rs
```

#### Plugin API (`api.rs`)

```rust
/// Plugin trait - implemented by all plugins
#[async_trait]
pub trait Plugin: Send + Sync {
    /// Plugin metadata
    fn metadata(&self) -> PluginMetadata;

    /// Initialize plugin with configuration
    async fn initialize(&mut self, config: PluginConfig) -> Result<(), PluginError>;

    /// Execute plugin logic
    async fn execute(&self, context: PluginContext) -> Result<PluginResult, PluginError>;

    /// Cleanup resources
    async fn cleanup(&mut self) -> Result<(), PluginError>;

    /// Validate plugin configuration
    fn validate_config(&self, config: &PluginConfig) -> Result<(), PluginError>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub plugin_type: PluginType,
    pub capabilities: Vec<Capability>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PluginType {
    /// Custom evaluation metric
    Evaluator,

    /// Data transformer
    Transformer,

    /// Result exporter
    Exporter,

    /// Provider extension
    Provider,

    /// Notification handler
    Notifier,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginContext {
    pub session_id: Uuid,
    pub input: serde_json::Value,
    pub metadata: HashMap<String, String>,
    pub providers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginResult {
    pub success: bool,
    pub output: serde_json::Value,
    pub metrics: HashMap<String, f64>,
    pub logs: Vec<String>,
}
```

#### WASM Plugin Loader (`loader.rs`)

```rust
use wasmer::{Store, Module, Instance, imports, Function};

/// WASM plugin loader using Wasmer
pub struct WasmPluginLoader {
    store: Store,
    module_cache: Arc<RwLock<HashMap<PathBuf, Module>>>,
}

impl WasmPluginLoader {
    /// Load plugin from WASM file
    pub async fn load_from_file(&self, path: &Path) -> Result<Box<dyn Plugin>, PluginError>;

    /// Load plugin from bytes
    pub async fn load_from_bytes(&self, bytes: &[u8]) -> Result<Box<dyn Plugin>, PluginError>;

    /// Precompile plugin for faster loading
    pub async fn precompile(&self, path: &Path) -> Result<(), PluginError>;
}

/// WASM plugin wrapper
pub struct WasmPlugin {
    instance: Instance,
    metadata: PluginMetadata,
    exports: PluginExports,
}

struct PluginExports {
    initialize: Function,
    execute: Function,
    cleanup: Function,
}
```

#### Plugin Registry (`registry.rs`)

```rust
/// Central plugin registry
pub struct PluginRegistry {
    plugins: Arc<RwLock<HashMap<String, Arc<dyn Plugin>>>>,
    loader: WasmPluginLoader,
    config: PluginRegistryConfig,
}

impl PluginRegistry {
    /// Register plugin
    pub fn register(&mut self, name: String, plugin: Arc<dyn Plugin>) -> Result<()>;

    /// Unregister plugin
    pub fn unregister(&mut self, name: &str) -> Result<()>;

    /// Get plugin by name
    pub fn get(&self, name: &str) -> Option<Arc<dyn Plugin>>;

    /// List all plugins
    pub fn list(&self) -> Vec<PluginMetadata>;

    /// Load plugins from directory
    pub async fn load_from_directory(&mut self, path: &Path) -> Result<Vec<String>>;

    /// Hot reload plugin
    pub async fn reload(&mut self, name: &str) -> Result<()>;
}
```

---

### 1.5 Integration Layer (`core/src/integrations/`)

```rust
integrations/
├── mod.rs              # Public exports
├── langchain.rs        # Langchain integration
├── llamaindex.rs       # LlamaIndex integration
├── mlflow.rs           # MLflow integration
├── wandb.rs            # Weights & Biases integration
└── adapter.rs          # Common adapter traits
```

#### Langchain Integration (`langchain.rs`)

```rust
/// Langchain integration for LLM Test Bench
pub struct LangchainIntegration {
    config: LangchainConfig,
    client: reqwest::Client,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LangchainConfig {
    pub api_url: Option<String>,
    pub project_name: String,
    pub enable_tracing: bool,
}

impl LangchainIntegration {
    /// Export benchmark results to Langchain format
    pub async fn export_results(
        &self,
        results: &BenchmarkResults,
    ) -> Result<(), IntegrationError>;

    /// Import Langchain dataset for benchmarking
    pub async fn import_dataset(
        &self,
        dataset_name: &str,
    ) -> Result<Dataset, IntegrationError>;

    /// Sync evaluation traces
    pub async fn sync_traces(
        &self,
        session_id: Uuid,
    ) -> Result<(), IntegrationError>;
}
```

#### MLflow Integration (`mlflow.rs`)

```rust
/// MLflow integration for experiment tracking
pub struct MlflowIntegration {
    tracking_uri: String,
    experiment_name: String,
    client: reqwest::Client,
}

impl MlflowIntegration {
    /// Log benchmark run as MLflow experiment
    pub async fn log_run(
        &self,
        run_name: &str,
        results: &BenchmarkResults,
    ) -> Result<String, IntegrationError>;

    /// Log metrics to MLflow
    pub async fn log_metrics(
        &self,
        run_id: &str,
        metrics: HashMap<String, f64>,
        step: Option<i64>,
    ) -> Result<(), IntegrationError>;

    /// Log artifacts (dashboards, reports)
    pub async fn log_artifact(
        &self,
        run_id: &str,
        file_path: &Path,
    ) -> Result<(), IntegrationError>;

    /// Compare runs
    pub async fn compare_runs(
        &self,
        run_ids: Vec<String>,
    ) -> Result<ComparisonReport, IntegrationError>;
}
```

#### Weights & Biases Integration (`wandb.rs`)

```rust
/// W&B integration for model tracking
pub struct WandbIntegration {
    api_key: String,
    project: String,
    entity: String,
    client: reqwest::Client,
}

impl WandbIntegration {
    /// Initialize W&B run
    pub async fn init_run(
        &self,
        run_name: &str,
        config: &BenchmarkConfig,
    ) -> Result<String, IntegrationError>;

    /// Log metrics to W&B
    pub async fn log_metrics(
        &self,
        run_id: &str,
        metrics: HashMap<String, f64>,
    ) -> Result<(), IntegrationError>;

    /// Log comparison table
    pub async fn log_table(
        &self,
        run_id: &str,
        table_name: &str,
        data: Vec<HashMap<String, serde_json::Value>>,
    ) -> Result<(), IntegrationError>;

    /// Finish run
    pub async fn finish_run(&self, run_id: &str) -> Result<(), IntegrationError>;
}
```

---

### 1.6 Storage Layer (`core/src/storage/`)

```rust
storage/
├── mod.rs              # Public exports
├── database.rs         # Database abstraction
├── postgres.rs         # PostgreSQL backend
├── sqlite.rs           # SQLite backend
├── migrations/         # Schema migrations
│   ├── 001_initial.sql
│   ├── 002_providers.sql
│   └── 003_sessions.sql
└── queries.rs          # Query builders
```

#### Database Abstraction (`database.rs`)

```rust
use sqlx::{Pool, Postgres, Sqlite, Row};

/// Database abstraction layer
#[async_trait]
pub trait Database: Send + Sync {
    /// Insert benchmark result
    async fn insert_result(&self, result: &BenchmarkResultV5) -> Result<Uuid, StorageError>;

    /// Get result by ID
    async fn get_result(&self, id: Uuid) -> Result<Option<BenchmarkResultV5>, StorageError>;

    /// Query results with filters
    async fn query_results(
        &self,
        filters: ResultFilters,
    ) -> Result<Vec<BenchmarkResultV5>, StorageError>;

    /// Insert session
    async fn insert_session(&self, session: &Session) -> Result<Uuid, StorageError>;

    /// Get session
    async fn get_session(&self, id: Uuid) -> Result<Option<Session>, StorageError>;

    /// Register provider
    async fn register_provider(&self, provider: &ProviderRecord) -> Result<Uuid, StorageError>;

    /// Insert alert
    async fn insert_alert(&self, alert: &Alert) -> Result<Uuid, StorageError>;

    /// Run migrations
    async fn migrate(&self) -> Result<(), StorageError>;
}

/// PostgreSQL backend
pub struct PostgresDatabase {
    pool: Pool<Postgres>,
}

impl PostgresDatabase {
    pub async fn new(database_url: &str) -> Result<Self, StorageError>;
}

#[async_trait]
impl Database for PostgresDatabase {
    // Implementation using sqlx PostgreSQL driver
}

/// SQLite backend (for development/testing)
pub struct SqliteDatabase {
    pool: Pool<Sqlite>,
}

impl SqliteDatabase {
    pub async fn new(path: &Path) -> Result<Self, StorageError>;
}

#[async_trait]
impl Database for SqliteDatabase {
    // Implementation using sqlx SQLite driver
}
```

---

### 1.7 API Server (`server/` - NEW CRATE)

```
server/
├── Cargo.toml
├── src/
│   ├── main.rs         # Server entry point
│   ├── lib.rs          # Library exports
│   ├── api/
│   │   ├── mod.rs
│   │   ├── rest.rs     # REST API handlers
│   │   ├── graphql.rs  # GraphQL schema and resolvers
│   │   └── websocket.rs# WebSocket handlers
│   ├── auth/
│   │   ├── mod.rs
│   │   ├── jwt.rs      # JWT authentication
│   │   ├── rbac.rs     # Role-based access control
│   │   └── oauth.rs    # OAuth2 integration
│   ├── middleware/
│   │   ├── mod.rs
│   │   ├── auth.rs     # Auth middleware
│   │   ├── rate_limit.rs # Rate limiting
│   │   ├── logging.rs  # Request logging
│   │   └── cors.rs     # CORS handling
│   └── state.rs        # Shared application state
```

#### REST API (`api/rest.rs`)

```rust
use axum::{
    Router, Json, extract::{Path, Query, State},
    http::StatusCode, response::IntoResponse,
};

/// REST API routes
pub fn routes(state: Arc<AppState>) -> Router {
    Router::new()
        // Benchmarks
        .route("/api/v1/benchmarks", post(create_benchmark))
        .route("/api/v1/benchmarks/:id", get(get_benchmark))
        .route("/api/v1/benchmarks", get(list_benchmarks))
        .route("/api/v1/benchmarks/:id/cancel", post(cancel_benchmark))

        // Results
        .route("/api/v1/results", get(query_results))
        .route("/api/v1/results/:id", get(get_result))
        .route("/api/v1/results/export", post(export_results))

        // Providers
        .route("/api/v1/providers", get(list_providers))
        .route("/api/v1/providers/:name", get(get_provider))
        .route("/api/v1/providers/:name/models", get(list_models))

        // Evaluation
        .route("/api/v1/evaluate", post(evaluate_response))
        .route("/api/v1/compare", post(compare_models))

        // Monitoring
        .route("/api/v1/metrics", get(get_metrics))
        .route("/api/v1/health", get(health_check))

        // Plugins
        .route("/api/v1/plugins", get(list_plugins))
        .route("/api/v1/plugins", post(upload_plugin))
        .route("/api/v1/plugins/:name", delete(uninstall_plugin))

        .with_state(state)
}

/// POST /api/v1/benchmarks - Create new benchmark
async fn create_benchmark(
    State(state): State<Arc<AppState>>,
    Json(request): Json<CreateBenchmarkRequest>,
) -> Result<Json<BenchmarkResponse>, ApiError> {
    // Implementation
}

/// GET /api/v1/benchmarks/:id - Get benchmark status
async fn get_benchmark(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<BenchmarkResponse>, ApiError> {
    // Implementation
}
```

#### GraphQL API (`api/graphql.rs`)

```rust
use async_graphql::{Context, Object, Schema, Subscription};

/// GraphQL schema
pub type AppSchema = Schema<Query, Mutation, Subscription>;

/// Query root
pub struct Query;

#[Object]
impl Query {
    /// Get benchmark by ID
    async fn benchmark(&self, ctx: &Context<'_>, id: Uuid) -> Result<Benchmark> {
        // Implementation
    }

    /// List benchmarks with pagination
    async fn benchmarks(
        &self,
        ctx: &Context<'_>,
        first: Option<i32>,
        after: Option<String>,
        filters: Option<BenchmarkFilters>,
    ) -> Result<BenchmarkConnection> {
        // Implementation
    }

    /// Query results
    async fn results(
        &self,
        ctx: &Context<'_>,
        filters: ResultFilters,
    ) -> Result<Vec<BenchmarkResult>> {
        // Implementation
    }

    /// Get provider info
    async fn provider(&self, ctx: &Context<'_>, name: String) -> Result<Provider> {
        // Implementation
    }
}

/// Mutation root
pub struct Mutation;

#[Object]
impl Mutation {
    /// Create benchmark
    async fn create_benchmark(
        &self,
        ctx: &Context<'_>,
        input: CreateBenchmarkInput,
    ) -> Result<Benchmark> {
        // Implementation
    }

    /// Cancel benchmark
    async fn cancel_benchmark(&self, ctx: &Context<'_>, id: Uuid) -> Result<bool> {
        // Implementation
    }

    /// Upload plugin
    async fn upload_plugin(
        &self,
        ctx: &Context<'_>,
        file: Upload,
    ) -> Result<Plugin> {
        // Implementation
    }
}

/// Subscription root for real-time updates
pub struct Subscription;

#[Subscription]
impl Subscription {
    /// Subscribe to benchmark progress
    async fn benchmark_progress(
        &self,
        ctx: &Context<'_>,
        id: Uuid,
    ) -> impl Stream<Item = BenchmarkProgress> {
        // Implementation using tokio broadcast
    }

    /// Subscribe to metrics updates
    async fn metrics_stream(&self) -> impl Stream<Item = MetricUpdate> {
        // Implementation
    }
}
```

#### Authentication (`auth/jwt.rs`)

```rust
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};

/// JWT claims
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,       // User ID
    pub exp: usize,        // Expiration time
    pub iat: usize,        // Issued at
    pub role: UserRole,    // User role
    pub permissions: Vec<Permission>,
}

/// User roles
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum UserRole {
    Admin,
    User,
    Viewer,
}

/// Permissions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Permission {
    RunBenchmark,
    ViewResults,
    ManageProviders,
    ManagePlugins,
    ManageUsers,
    AdminAccess,
}

/// JWT manager
pub struct JwtManager {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    validation: Validation,
}

impl JwtManager {
    /// Generate JWT token
    pub fn generate_token(&self, user_id: &str, role: UserRole) -> Result<String, AuthError>;

    /// Validate and decode token
    pub fn validate_token(&self, token: &str) -> Result<Claims, AuthError>;

    /// Refresh token
    pub fn refresh_token(&self, token: &str) -> Result<String, AuthError>;
}
```

---

## 2. Data Architecture

### 2.1 Enhanced Result Schema

```rust
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

/// Enhanced benchmark result for Phase 5
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResultV5 {
    // Existing fields from Phase 4
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub dataset: String,
    pub provider: String,
    pub model: String,
    pub test_id: String,
    pub status: TestStatus,
    pub response: Option<CompletionResponse>,
    pub error: Option<String>,
    pub duration_ms: u64,

    // Evaluation results
    pub evaluation_scores: HashMap<String, f64>,
    pub faithfulness_score: Option<f64>,
    pub relevance_score: Option<f64>,
    pub coherence_score: Option<f64>,
    pub perplexity_score: Option<f64>,

    // NEW fields for Phase 5

    /// Multi-modal inputs (images, audio, video)
    pub multi_modal_inputs: Option<MultiModalInputs>,

    /// Plugin-generated metrics
    pub plugin_metrics: HashMap<String, serde_json::Value>,

    /// Distributed tracing ID (OpenTelemetry)
    pub trace_id: Option<String>,

    /// Span ID for distributed tracing
    pub span_id: Option<String>,

    /// Session ID (groups related tests)
    pub session_id: Uuid,

    /// User-defined tags for filtering
    pub tags: Vec<String>,

    /// Additional metadata
    pub metadata: HashMap<String, String>,

    /// Provider-specific metadata
    pub provider_metadata: Option<ProviderMetadata>,

    /// Cost breakdown
    pub cost_details: CostDetails,

    /// Cache statistics
    pub cache_stats: Option<CacheStats>,
}

/// Multi-modal inputs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiModalInputs {
    pub images: Vec<ImageInput>,
    pub audio: Vec<AudioInput>,
    pub video: Vec<VideoInput>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageInput {
    pub id: String,
    pub format: String,
    pub size_bytes: usize,
    pub dimensions: Option<(u32, u32)>,
    pub url: Option<String>,
    pub hash: String,  // SHA-256 hash for deduplication
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioInput {
    pub id: String,
    pub format: String,
    pub duration_seconds: f64,
    pub size_bytes: usize,
    pub sample_rate: u32,
    pub channels: u8,
    pub hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoInput {
    pub id: String,
    pub format: String,
    pub duration_seconds: f64,
    pub size_bytes: usize,
    pub dimensions: (u32, u32),
    pub frame_rate: f64,
    pub hash: String,
}

/// Provider-specific metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderMetadata {
    pub api_version: String,
    pub model_version: Option<String>,
    pub region: Option<String>,
    pub endpoint: String,
    pub request_id: String,
}

/// Detailed cost breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostDetails {
    pub total_cost: f64,
    pub prompt_cost: f64,
    pub completion_cost: f64,
    pub image_cost: Option<f64>,
    pub audio_cost: Option<f64>,
    pub embedding_cost: Option<f64>,
    pub cache_savings: f64,
}

/// Cache hit/miss statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub evaluation_cache_hit: bool,
    pub provider_cache_hit: bool,
    pub cache_key: String,
    pub ttl_seconds: Option<u64>,
}
```

### 2.2 Database Schema (PostgreSQL)

```sql
-- ============================================================================
-- Phase 5 Database Schema
-- ============================================================================

-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pg_trgm";  -- For text search

-- ============================================================================
-- Users and Authentication
-- ============================================================================

CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    username VARCHAR(255) UNIQUE NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    role VARCHAR(50) NOT NULL,  -- 'admin', 'user', 'viewer'
    api_key_hash VARCHAR(255),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_login_at TIMESTAMPTZ,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,

    CONSTRAINT users_role_check CHECK (role IN ('admin', 'user', 'viewer'))
);

CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_username ON users(username);
CREATE INDEX idx_users_api_key_hash ON users(api_key_hash);

-- ============================================================================
-- Providers
-- ============================================================================

CREATE TABLE providers (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) UNIQUE NOT NULL,
    type VARCHAR(50) NOT NULL,  -- 'cloud', 'local', 'custom'
    config JSONB NOT NULL,
    capabilities JSONB,
    api_version VARCHAR(50),
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT providers_type_check CHECK (type IN ('cloud', 'local', 'custom'))
);

CREATE INDEX idx_providers_name ON providers(name);
CREATE INDEX idx_providers_type ON providers(type);
CREATE INDEX idx_providers_capabilities ON providers USING GIN(capabilities);

-- ============================================================================
-- Sessions (groups related benchmark runs)
-- ============================================================================

CREATE TABLE sessions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255),
    description TEXT,
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    config JSONB,
    tags TEXT[],
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,

    CONSTRAINT sessions_status_check CHECK (
        status IN ('pending', 'running', 'completed', 'failed', 'cancelled')
    )
);

CREATE INDEX idx_sessions_user_id ON sessions(user_id);
CREATE INDEX idx_sessions_status ON sessions(status);
CREATE INDEX idx_sessions_created_at ON sessions(created_at DESC);
CREATE INDEX idx_sessions_tags ON sessions USING GIN(tags);

-- ============================================================================
-- Benchmark Results (main results table)
-- ============================================================================

CREATE TABLE results_v5 (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    session_id UUID NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    provider_id UUID NOT NULL REFERENCES providers(id) ON DELETE CASCADE,
    dataset_id UUID,
    test_id VARCHAR(255) NOT NULL,

    -- Basic info
    status VARCHAR(50) NOT NULL,
    model VARCHAR(255) NOT NULL,
    prompt TEXT NOT NULL,
    response TEXT,
    error TEXT,

    -- Timing
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    duration_ms BIGINT NOT NULL,

    -- Token usage
    prompt_tokens INTEGER,
    completion_tokens INTEGER,
    total_tokens INTEGER,

    -- Evaluation scores
    evaluation_scores JSONB,
    faithfulness_score DOUBLE PRECISION,
    relevance_score DOUBLE PRECISION,
    coherence_score DOUBLE PRECISION,
    perplexity_score DOUBLE PRECISION,

    -- Multi-modal
    multi_modal_inputs JSONB,

    -- Plugin metrics
    plugin_metrics JSONB,

    -- Distributed tracing
    trace_id VARCHAR(255),
    span_id VARCHAR(255),

    -- Tags and metadata
    tags TEXT[],
    metadata JSONB,

    -- Provider metadata
    provider_metadata JSONB,

    -- Cost tracking
    total_cost DOUBLE PRECISION,
    cost_details JSONB,

    -- Cache stats
    cache_stats JSONB,

    CONSTRAINT results_status_check CHECK (
        status IN ('success', 'failure', 'timeout', 'skipped')
    )
);

-- Indexes for fast queries
CREATE INDEX idx_results_session_id ON results_v5(session_id);
CREATE INDEX idx_results_provider_id ON results_v5(provider_id);
CREATE INDEX idx_results_created_at ON results_v5(created_at DESC);
CREATE INDEX idx_results_status ON results_v5(status);
CREATE INDEX idx_results_model ON results_v5(model);
CREATE INDEX idx_results_tags ON results_v5 USING GIN(tags);
CREATE INDEX idx_results_evaluation_scores ON results_v5 USING GIN(evaluation_scores);
CREATE INDEX idx_results_trace_id ON results_v5(trace_id);

-- Composite indexes for common queries
CREATE INDEX idx_results_session_created ON results_v5(session_id, created_at DESC);
CREATE INDEX idx_results_provider_model ON results_v5(provider_id, model);

-- Full-text search on prompts and responses
CREATE INDEX idx_results_prompt_text ON results_v5 USING GIN(to_tsvector('english', prompt));
CREATE INDEX idx_results_response_text ON results_v5 USING GIN(to_tsvector('english', response));

-- ============================================================================
-- Alerts
-- ============================================================================

CREATE TABLE alerts (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    rule_id UUID,
    severity VARCHAR(50) NOT NULL,
    message TEXT NOT NULL,
    details JSONB,
    triggered_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    resolved_at TIMESTAMPTZ,
    resolved_by UUID REFERENCES users(id),

    CONSTRAINT alerts_severity_check CHECK (
        severity IN ('info', 'warning', 'error', 'critical')
    )
);

CREATE INDEX idx_alerts_severity ON alerts(severity);
CREATE INDEX idx_alerts_triggered_at ON alerts(triggered_at DESC);
CREATE INDEX idx_alerts_resolved ON alerts(resolved_at) WHERE resolved_at IS NULL;

-- ============================================================================
-- Plugins
-- ============================================================================

CREATE TABLE plugins (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) UNIQUE NOT NULL,
    version VARCHAR(50) NOT NULL,
    author VARCHAR(255),
    description TEXT,
    plugin_type VARCHAR(50) NOT NULL,
    capabilities JSONB,
    config_schema JSONB,
    is_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    file_path TEXT NOT NULL,
    file_hash VARCHAR(255) NOT NULL,
    installed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    installed_by UUID REFERENCES users(id),

    CONSTRAINT plugins_type_check CHECK (
        plugin_type IN ('evaluator', 'transformer', 'exporter', 'provider', 'notifier')
    )
);

CREATE INDEX idx_plugins_name ON plugins(name);
CREATE INDEX idx_plugins_type ON plugins(plugin_type);
CREATE INDEX idx_plugins_enabled ON plugins(is_enabled);

-- ============================================================================
-- Audit Log
-- ============================================================================

CREATE TABLE audit_log (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID REFERENCES users(id),
    action VARCHAR(255) NOT NULL,
    resource_type VARCHAR(100) NOT NULL,
    resource_id UUID,
    details JSONB,
    ip_address INET,
    user_agent TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_audit_log_user_id ON audit_log(user_id);
CREATE INDEX idx_audit_log_created_at ON audit_log(created_at DESC);
CREATE INDEX idx_audit_log_action ON audit_log(action);
CREATE INDEX idx_audit_log_resource ON audit_log(resource_type, resource_id);

-- ============================================================================
-- Model Profiles (for routing)
-- ============================================================================

CREATE TABLE model_profiles (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    provider_id UUID NOT NULL REFERENCES providers(id) ON DELETE CASCADE,
    model_name VARCHAR(255) NOT NULL,
    typical_quality DOUBLE PRECISION,
    avg_latency_ms BIGINT,
    cost_per_1k_tokens DOUBLE PRECISION,
    context_limit INTEGER,
    strengths JSONB,
    sample_count INTEGER DEFAULT 0,
    last_updated TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    UNIQUE(provider_id, model_name)
);

CREATE INDEX idx_model_profiles_provider ON model_profiles(provider_id);
CREATE INDEX idx_model_profiles_quality ON model_profiles(typical_quality DESC);
CREATE INDEX idx_model_profiles_cost ON model_profiles(cost_per_1k_tokens ASC);

-- ============================================================================
-- Views for common queries
-- ============================================================================

-- Active sessions with summary
CREATE VIEW v_active_sessions AS
SELECT
    s.id,
    s.name,
    s.status,
    s.created_at,
    COUNT(r.id) as total_tests,
    COUNT(r.id) FILTER (WHERE r.status = 'success') as successful_tests,
    AVG(r.duration_ms) as avg_duration_ms,
    SUM(r.total_cost) as total_cost
FROM sessions s
LEFT JOIN results_v5 r ON s.id = r.session_id
WHERE s.status IN ('pending', 'running')
GROUP BY s.id, s.name, s.status, s.created_at;

-- Provider performance summary
CREATE VIEW v_provider_performance AS
SELECT
    p.name as provider_name,
    r.model,
    COUNT(r.id) as total_requests,
    COUNT(r.id) FILTER (WHERE r.status = 'success') as successful_requests,
    AVG(r.duration_ms) as avg_latency_ms,
    PERCENTILE_CONT(0.95) WITHIN GROUP (ORDER BY r.duration_ms) as p95_latency_ms,
    AVG(r.total_cost) as avg_cost,
    AVG(r.faithfulness_score) as avg_faithfulness,
    AVG(r.relevance_score) as avg_relevance
FROM providers p
JOIN results_v5 r ON p.id = r.provider_id
GROUP BY p.name, r.model;

-- ============================================================================
-- Functions
-- ============================================================================

-- Function to update session status
CREATE OR REPLACE FUNCTION update_session_status()
RETURNS TRIGGER AS $$
BEGIN
    UPDATE sessions
    SET status = CASE
        WHEN (SELECT COUNT(*) FROM results_v5 WHERE session_id = NEW.session_id AND status IN ('running')) > 0
            THEN 'running'
        WHEN (SELECT COUNT(*) FROM results_v5 WHERE session_id = NEW.session_id AND status = 'failure') > 0
            THEN 'failed'
        ELSE 'completed'
    END,
    completed_at = CASE
        WHEN (SELECT COUNT(*) FROM results_v5 WHERE session_id = NEW.session_id AND status = 'running') = 0
            THEN NOW()
        ELSE NULL
    END
    WHERE id = NEW.session_id;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_update_session_status
AFTER INSERT OR UPDATE ON results_v5
FOR EACH ROW
EXECUTE FUNCTION update_session_status();

-- Function to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_users_updated_at
BEFORE UPDATE ON users
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER trigger_providers_updated_at
BEFORE UPDATE ON providers
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();
```

### 2.3 Query Builders (`storage/queries.rs`)

```rust
use sqlx::QueryBuilder;

/// Query builder for results
pub struct ResultQueryBuilder {
    filters: ResultFilters,
    limit: Option<i64>,
    offset: Option<i64>,
    order_by: Option<String>,
}

impl ResultQueryBuilder {
    pub fn new() -> Self;

    /// Add session filter
    pub fn session(mut self, session_id: Uuid) -> Self;

    /// Add provider filter
    pub fn provider(mut self, provider: &str) -> Self;

    /// Add model filter
    pub fn model(mut self, model: &str) -> Self;

    /// Add date range filter
    pub fn date_range(mut self, start: DateTime<Utc>, end: DateTime<Utc>) -> Self;

    /// Add tags filter (ANY)
    pub fn with_tags(mut self, tags: Vec<String>) -> Self;

    /// Add minimum score filter
    pub fn min_score(mut self, metric: &str, min_score: f64) -> Self;

    /// Add pagination
    pub fn paginate(mut self, limit: i64, offset: i64) -> Self;

    /// Add ordering
    pub fn order_by(mut self, field: &str, ascending: bool) -> Self;

    /// Build and execute query
    pub async fn execute(self, db: &dyn Database) -> Result<Vec<BenchmarkResultV5>>;
}
```

---

## 3. Integration Architecture

### 3.1 Plugin System Design

#### Plugin Manifest (`plugin.toml`)

```toml
[package]
name = "custom-faithfulness"
version = "1.0.0"
author = "Your Name"
description = "Custom faithfulness evaluator using domain-specific knowledge"

[plugin]
type = "evaluator"
entry_point = "main.wasm"

[capabilities]
requires_llm = true
supports_batch = true
max_batch_size = 10

[config_schema]
# JSON Schema for plugin configuration
schema = """
{
  "type": "object",
  "properties": {
    "domain": {
      "type": "string",
      "enum": ["medical", "legal", "technical"]
    },
    "strictness": {
      "type": "number",
      "minimum": 0.0,
      "maximum": 1.0
    }
  },
  "required": ["domain"]
}
"""

[dependencies]
llm-test-bench-plugin-sdk = "0.5.0"
```

#### Plugin SDK (Rust)

```rust
// plugin-sdk/src/lib.rs

/// Plugin SDK for writing custom evaluators
#[macro_export]
macro_rules! declare_plugin {
    ($plugin_type:ty) => {
        #[no_mangle]
        pub extern "C" fn _plugin_create() -> *mut dyn Plugin {
            Box::into_raw(Box::new(<$plugin_type>::default()))
        }
    };
}

/// Example plugin implementation
use llm_test_bench_plugin_sdk::*;

#[derive(Default)]
pub struct CustomFaithfulnessPlugin {
    config: Option<PluginConfig>,
}

#[async_trait]
impl Plugin for CustomFaithfulnessPlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "custom-faithfulness".to_string(),
            version: "1.0.0".to_string(),
            author: "Your Name".to_string(),
            description: "Domain-specific faithfulness evaluator".to_string(),
            plugin_type: PluginType::Evaluator,
            capabilities: vec![Capability::RequiresLLM, Capability::SupportsBatch],
        }
    }

    async fn initialize(&mut self, config: PluginConfig) -> Result<(), PluginError> {
        self.config = Some(config);
        Ok(())
    }

    async fn execute(&self, context: PluginContext) -> Result<PluginResult, PluginError> {
        // Custom evaluation logic
        let input: EvaluationInput = serde_json::from_value(context.input)?;

        let score = self.evaluate_faithfulness(&input).await?;

        Ok(PluginResult {
            success: true,
            output: serde_json::json!({
                "score": score,
                "domain": self.config.as_ref().unwrap().get("domain"),
            }),
            metrics: vec![("faithfulness".to_string(), score)].into_iter().collect(),
            logs: vec![format!("Evaluated with score: {}", score)],
        })
    }

    async fn cleanup(&mut self) -> Result<(), PluginError> {
        Ok(())
    }

    fn validate_config(&self, config: &PluginConfig) -> Result<(), PluginError> {
        // Validate against JSON schema
        Ok(())
    }
}

impl CustomFaithfulnessPlugin {
    async fn evaluate_faithfulness(&self, input: &EvaluationInput) -> Result<f64, PluginError> {
        // Custom domain-specific evaluation logic
        // Can use LLM provider from context
        Ok(0.85)
    }
}

declare_plugin!(CustomFaithfulnessPlugin);
```

### 3.2 Integration Adapters

#### Common Adapter Interface

```rust
/// Common adapter trait for external integrations
#[async_trait]
pub trait IntegrationAdapter: Send + Sync {
    /// Export benchmark results
    async fn export_results(
        &self,
        results: &BenchmarkResults,
    ) -> Result<ExportResult, IntegrationError>;

    /// Import dataset
    async fn import_dataset(
        &self,
        dataset_id: &str,
    ) -> Result<Dataset, IntegrationError>;

    /// Sync metadata
    async fn sync_metadata(
        &self,
        metadata: &BenchmarkMetadata,
    ) -> Result<(), IntegrationError>;

    /// Check connection health
    async fn health_check(&self) -> Result<bool, IntegrationError>;
}
```

---

## 4. Real-Time Architecture

### 4.1 Event Flow

```
┌──────────────┐
│  Benchmark   │
│   Executor   │
└───────┬──────┘
        │ emit events
        ▼
┌──────────────────┐
│   Event Bus      │
│ (tokio channels) │
└───────┬──────────┘
        │ broadcast
        ├─────────────────┬──────────────┬───────────────┐
        ▼                 ▼              ▼               ▼
┌─────────────┐  ┌──────────────┐  ┌─────────┐  ┌──────────┐
│  WebSocket  │  │  Prometheus  │  │  Alert  │  │  Storage │
│   Server    │  │  Exporter    │  │ Manager │  │  Writer  │
└─────────────┘  └──────────────┘  └─────────┘  └──────────┘
        │
        │ WebSocket messages
        ▼
┌─────────────┐
│   Browser   │
│  Dashboard  │
└─────────────┘
```

### 4.2 WebSocket Protocol

```rust
/// WebSocket message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WsMessage {
    /// Client → Server: Subscribe to events
    Subscribe {
        topics: Vec<String>,
        session_id: Option<Uuid>,
    },

    /// Client → Server: Unsubscribe
    Unsubscribe {
        topics: Vec<String>,
    },

    /// Server → Client: Event notification
    Event {
        topic: String,
        payload: RealtimeEvent,
        timestamp: DateTime<Utc>,
    },

    /// Server → Client: Acknowledgment
    Ack {
        message_id: Uuid,
    },

    /// Bidirectional: Ping/Pong
    Ping,
    Pong,

    /// Server → Client: Error
    Error {
        code: String,
        message: String,
    },
}

/// Topics for subscription
pub const TOPIC_BENCHMARK_PROGRESS: &str = "benchmark.progress";
pub const TOPIC_TEST_COMPLETED: &str = "test.completed";
pub const TOPIC_METRICS: &str = "metrics.*";
pub const TOPIC_ALERTS: &str = "alerts.*";
```

### 4.3 Browser Dashboard (Vue.js + WebSocket)

```typescript
// dashboard/src/composables/useWebSocket.ts
import { ref, onMounted, onUnmounted } from 'vue';

interface WebSocketHook {
  isConnected: Ref<boolean>;
  events: Ref<RealtimeEvent[]>;
  subscribe: (topics: string[]) => void;
  unsubscribe: (topics: string[]) => void;
}

export function useWebSocket(url: string): WebSocketHook {
  const socket = ref<WebSocket | null>(null);
  const isConnected = ref(false);
  const events = ref<RealtimeEvent[]>([]);

  const connect = () => {
    socket.value = new WebSocket(url);

    socket.value.onopen = () => {
      isConnected.value = true;
      console.log('WebSocket connected');
    };

    socket.value.onmessage = (event) => {
      const message: WsMessage = JSON.parse(event.data);

      if (message.type === 'Event') {
        events.value.push(message.payload);

        // Limit event buffer
        if (events.value.length > 1000) {
          events.value.shift();
        }
      }
    };

    socket.value.onclose = () => {
      isConnected.value = false;
      console.log('WebSocket disconnected');

      // Reconnect after delay
      setTimeout(connect, 5000);
    };
  };

  const subscribe = (topics: string[]) => {
    if (socket.value?.readyState === WebSocket.OPEN) {
      socket.value.send(JSON.stringify({
        type: 'Subscribe',
        topics,
      }));
    }
  };

  const unsubscribe = (topics: string[]) => {
    if (socket.value?.readyState === WebSocket.OPEN) {
      socket.value.send(JSON.stringify({
        type: 'Unsubscribe',
        topics,
      }));
    }
  };

  onMounted(connect);

  onUnmounted(() => {
    socket.value?.close();
  });

  return {
    isConnected,
    events,
    subscribe,
    unsubscribe,
  };
}
```

---

## 5. Distributed Architecture

### 5.1 Coordinator-Worker Pattern

```rust
/// Distributed coordinator
pub struct DistributedCoordinator {
    worker_registry: WorkerRegistry,
    task_scheduler: TaskScheduler,
    result_aggregator: ResultAggregator,
    config: DistributedConfig,
}

#[derive(Debug, Clone)]
pub struct DistributedConfig {
    pub coordinator_addr: SocketAddr,
    pub max_workers: usize,
    pub task_timeout: Duration,
    pub heartbeat_interval: Duration,
    pub enable_work_stealing: bool,
}

impl DistributedCoordinator {
    /// Start coordinator server
    pub async fn start(&self) -> Result<(), DistributedError>;

    /// Submit benchmark for distributed execution
    pub async fn submit_benchmark(
        &self,
        config: BenchmarkConfig,
        dataset: Dataset,
    ) -> Result<Uuid, DistributedError>;

    /// Get benchmark status
    pub async fn get_status(&self, job_id: Uuid) -> Result<JobStatus, DistributedError>;

    /// Cancel benchmark
    pub async fn cancel(&self, job_id: Uuid) -> Result<(), DistributedError>;
}

/// Worker node
pub struct Worker {
    id: Uuid,
    capabilities: WorkerCapabilities,
    executor: BenchmarkExecutor,
    coordinator_client: CoordinatorClient,
    config: WorkerConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerCapabilities {
    pub max_concurrent_tasks: usize,
    pub supported_providers: Vec<String>,
    pub has_gpu: bool,
    pub memory_gb: f64,
    pub cpu_cores: usize,
}

impl Worker {
    /// Register with coordinator
    pub async fn register(&self) -> Result<(), DistributedError>;

    /// Start accepting tasks
    pub async fn start(&mut self) -> Result<(), DistributedError>;

    /// Execute task
    pub async fn execute_task(&self, task: Task) -> Result<TaskResult, DistributedError>;

    /// Send heartbeat to coordinator
    pub async fn heartbeat(&self) -> Result<(), DistributedError>;
}

/// Task scheduler
pub struct TaskScheduler {
    task_queue: Arc<Mutex<VecDeque<Task>>>,
    in_progress: Arc<RwLock<HashMap<Uuid, TaskState>>>,
}

impl TaskScheduler {
    /// Schedule tasks from benchmark
    pub async fn schedule_benchmark(
        &self,
        benchmark: &BenchmarkConfig,
        dataset: &Dataset,
    ) -> Result<Vec<Task>, DistributedError>;

    /// Get next task for worker
    pub async fn get_next_task(
        &self,
        worker_id: Uuid,
    ) -> Option<Task>;

    /// Mark task complete
    pub async fn complete_task(
        &self,
        task_id: Uuid,
        result: TaskResult,
    ) -> Result<(), DistributedError>;

    /// Retry failed task
    pub async fn retry_task(&self, task_id: Uuid) -> Result<(), DistributedError>;
}
```

### 5.2 Communication Protocol (gRPC)

```protobuf
// distributed.proto

syntax = "proto3";

package llm_test_bench.distributed;

// Coordinator service
service Coordinator {
    // Worker registration
    rpc RegisterWorker(RegisterWorkerRequest) returns (RegisterWorkerResponse);

    // Worker heartbeat
    rpc Heartbeat(HeartbeatRequest) returns (HeartbeatResponse);

    // Get next task
    rpc GetTask(GetTaskRequest) returns (Task);

    // Report task result
    rpc ReportResult(TaskResult) returns (ReportResultResponse);

    // Worker deregistration
    rpc UnregisterWorker(UnregisterWorkerRequest) returns (UnregisterWorkerResponse);
}

message RegisterWorkerRequest {
    string worker_id = 1;
    WorkerCapabilities capabilities = 2;
}

message WorkerCapabilities {
    uint32 max_concurrent_tasks = 1;
    repeated string supported_providers = 2;
    bool has_gpu = 3;
    double memory_gb = 4;
    uint32 cpu_cores = 5;
}

message Task {
    string task_id = 1;
    string job_id = 2;
    string provider = 3;
    string model = 4;
    string prompt = 5;
    map<string, string> parameters = 6;
    repeated string evaluation_metrics = 7;
}

message TaskResult {
    string task_id = 1;
    string worker_id = 2;
    bool success = 3;
    bytes result_data = 4;  // Serialized BenchmarkResult
    string error = 5;
    uint64 duration_ms = 6;
}
```

### 5.3 Kubernetes Operator

```rust
// operator/src/lib.rs

use kube::{Api, Client};
use k8s_openapi::api::batch::v1::Job;

/// Kubernetes operator for LLM Test Bench
pub struct BenchmarkOperator {
    client: Client,
    namespace: String,
}

impl BenchmarkOperator {
    /// Create benchmark job in Kubernetes
    pub async fn create_benchmark_job(
        &self,
        config: &BenchmarkConfig,
    ) -> Result<Job, OperatorError>;

    /// Scale workers
    pub async fn scale_workers(&self, replicas: i32) -> Result<(), OperatorError>;

    /// Monitor job status
    pub async fn monitor_job(&self, job_name: &str) -> Result<JobStatus, OperatorError>;
}
```

---

## 6. API Server Architecture

### 6.1 Server Structure (Axum)

```rust
// server/src/main.rs

use axum::{
    Router,
    routing::{get, post},
    Extension,
};
use std::net::SocketAddr;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Initialize application state
    let state = Arc::new(AppState::new().await?);

    // Build router
    let app = Router::new()
        // Health check
        .route("/health", get(health_check))

        // REST API
        .nest("/api/v1", api::rest::routes(Arc::clone(&state)))

        // GraphQL
        .route("/graphql", post(api::graphql::graphql_handler))
        .route("/graphql/playground", get(api::graphql::playground))

        // WebSocket
        .route("/ws", get(api::websocket::ws_handler))

        // Middleware
        .layer(middleware::auth::AuthMiddleware::new())
        .layer(middleware::rate_limit::RateLimitMiddleware::new())
        .layer(middleware::logging::LoggingMiddleware::new())
        .layer(middleware::cors::CorsMiddleware::new())

        // State
        .layer(Extension(state));

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    tracing::info!("Server listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

/// Application state
pub struct AppState {
    pub db: Arc<dyn Database>,
    pub provider_registry: Arc<ProviderRegistry>,
    pub plugin_registry: Arc<PluginRegistry>,
    pub auth_manager: Arc<AuthManager>,
    pub metrics_collector: Arc<MetricsCollector>,
    pub realtime_server: Arc<RealtimeServer>,
}
```

### 6.2 Middleware

#### Rate Limiting (`middleware/rate_limit.rs`)

```rust
use axum::{
    middleware::Next,
    response::Response,
    http::{Request, StatusCode},
};
use tower_governor::{GovernorLayer, GovernorConfig};

pub struct RateLimitMiddleware {
    governor: GovernorConfig,
}

impl RateLimitMiddleware {
    pub fn new() -> Self {
        let governor = GovernorConfig::default()
            .per_second(10)  // 10 requests per second
            .burst_size(20); // Allow burst of 20

        Self { governor }
    }

    pub fn layer(self) -> GovernorLayer {
        GovernorLayer {
            config: Box::leak(Box::new(self.governor)),
        }
    }
}
```

#### Authentication (`middleware/auth.rs`)

```rust
pub struct AuthMiddleware {
    jwt_manager: Arc<JwtManager>,
}

impl AuthMiddleware {
    pub async fn authenticate<B>(
        State(state): State<Arc<AppState>>,
        mut request: Request<B>,
        next: Next<B>,
    ) -> Result<Response, StatusCode> {
        // Extract token from Authorization header
        let auth_header = request
            .headers()
            .get("Authorization")
            .and_then(|h| h.to_str().ok());

        if let Some(auth) = auth_header {
            if auth.starts_with("Bearer ") {
                let token = &auth[7..];

                match state.auth_manager.jwt_manager.validate_token(token) {
                    Ok(claims) => {
                        // Add claims to request extensions
                        request.extensions_mut().insert(claims);
                        return Ok(next.run(request).await);
                    }
                    Err(_) => return Err(StatusCode::UNAUTHORIZED),
                }
            }
        }

        Err(StatusCode::UNAUTHORIZED)
    }
}
```

---

## 7. Deployment Architecture

### 7.1 Docker Compose (Full Stack)

```yaml
version: '3.8'

services:
  # API Server
  api-server:
    build:
      context: .
      dockerfile: Dockerfile.server
    ports:
      - "8080:8080"
    environment:
      - DATABASE_URL=postgresql://postgres:password@postgres:5432/llm_test_bench
      - REDIS_URL=redis://redis:6379
      - RUST_LOG=info
      - JWT_SECRET=${JWT_SECRET}
    depends_on:
      - postgres
      - redis
    volumes:
      - ./config:/config:ro
      - plugins:/plugins
    restart: unless-stopped

  # PostgreSQL Database
  postgres:
    image: postgres:15-alpine
    environment:
      - POSTGRES_USER=postgres
      - POSTGRES_PASSWORD=password
      - POSTGRES_DB=llm_test_bench
    ports:
      - "5432:5432"
    volumes:
      - postgres_data:/var/lib/postgresql/data
      - ./storage/migrations:/docker-entrypoint-initdb.d:ro
    restart: unless-stopped

  # Redis (for caching and pub/sub)
  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"
    volumes:
      - redis_data:/data
    restart: unless-stopped

  # Prometheus (metrics)
  prometheus:
    image: prom/prometheus:latest
    ports:
      - "9090:9090"
    volumes:
      - ./monitoring/prometheus.yml:/etc/prometheus/prometheus.yml:ro
      - prometheus_data:/prometheus
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
      - '--storage.tsdb.path=/prometheus'
    restart: unless-stopped

  # Grafana (dashboards)
  grafana:
    image: grafana/grafana:latest
    ports:
      - "3000:3000"
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=admin
    volumes:
      - grafana_data:/var/lib/grafana
      - ./monitoring/grafana/dashboards:/etc/grafana/provisioning/dashboards:ro
      - ./monitoring/grafana/datasources:/etc/grafana/provisioning/datasources:ro
    depends_on:
      - prometheus
    restart: unless-stopped

  # Coordinator (distributed benchmarking)
  coordinator:
    build:
      context: .
      dockerfile: Dockerfile.coordinator
    ports:
      - "50051:50051"  # gRPC
    environment:
      - DATABASE_URL=postgresql://postgres:password@postgres:5432/llm_test_bench
      - RUST_LOG=info
    depends_on:
      - postgres
    restart: unless-stopped

  # Worker nodes (scale as needed)
  worker:
    build:
      context: .
      dockerfile: Dockerfile.worker
    environment:
      - COORDINATOR_URL=http://coordinator:50051
      - RUST_LOG=info
      - OPENAI_API_KEY=${OPENAI_API_KEY}
      - ANTHROPIC_API_KEY=${ANTHROPIC_API_KEY}
    depends_on:
      - coordinator
    restart: unless-stopped
    deploy:
      replicas: 3

volumes:
  postgres_data:
  redis_data:
  prometheus_data:
  grafana_data:
  plugins:
```

### 7.2 Kubernetes Deployment

```yaml
# k8s/deployment.yaml

apiVersion: v1
kind: Namespace
metadata:
  name: llm-test-bench

---
# API Server Deployment
apiVersion: apps/v1
kind: Deployment
metadata:
  name: api-server
  namespace: llm-test-bench
spec:
  replicas: 3
  selector:
    matchLabels:
      app: api-server
  template:
    metadata:
      labels:
        app: api-server
    spec:
      containers:
      - name: api-server
        image: llm-test-bench/api-server:latest
        ports:
        - containerPort: 8080
          name: http
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: database-credentials
              key: url
        - name: JWT_SECRET
          valueFrom:
            secretKeyRef:
              name: jwt-secret
              key: secret
        resources:
          requests:
            cpu: "500m"
            memory: "512Mi"
          limits:
            cpu: "2000m"
            memory: "2Gi"
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 5

---
# API Server Service
apiVersion: v1
kind: Service
metadata:
  name: api-server
  namespace: llm-test-bench
spec:
  selector:
    app: api-server
  ports:
  - port: 80
    targetPort: 8080
  type: LoadBalancer

---
# Coordinator Deployment
apiVersion: apps/v1
kind: Deployment
metadata:
  name: coordinator
  namespace: llm-test-bench
spec:
  replicas: 1
  selector:
    matchLabels:
      app: coordinator
  template:
    metadata:
      labels:
        app: coordinator
    spec:
      containers:
      - name: coordinator
        image: llm-test-bench/coordinator:latest
        ports:
        - containerPort: 50051
          name: grpc
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: database-credentials
              key: url
        resources:
          requests:
            cpu: "1000m"
            memory: "1Gi"
          limits:
            cpu: "4000m"
            memory: "4Gi"

---
# Worker Deployment
apiVersion: apps/v1
kind: Deployment
metadata:
  name: worker
  namespace: llm-test-bench
spec:
  replicas: 5
  selector:
    matchLabels:
      app: worker
  template:
    metadata:
      labels:
        app: worker
    spec:
      containers:
      - name: worker
        image: llm-test-bench/worker:latest
        env:
        - name: COORDINATOR_URL
          value: "coordinator:50051"
        - name: OPENAI_API_KEY
          valueFrom:
            secretKeyRef:
              name: provider-credentials
              key: openai-key
        - name: ANTHROPIC_API_KEY
          valueFrom:
            secretKeyRef:
              name: provider-credentials
              key: anthropic-key
        resources:
          requests:
            cpu: "2000m"
            memory: "4Gi"
          limits:
            cpu: "4000m"
            memory: "8Gi"

---
# Horizontal Pod Autoscaler for Workers
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: worker-hpa
  namespace: llm-test-bench
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: worker
  minReplicas: 3
  maxReplicas: 20
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: 80
```

### 7.3 Helm Chart Structure

```
helm-chart/
├── Chart.yaml
├── values.yaml
├── values-production.yaml
├── values-staging.yaml
└── templates/
    ├── deployment-api.yaml
    ├── deployment-coordinator.yaml
    ├── deployment-worker.yaml
    ├── service-api.yaml
    ├── service-coordinator.yaml
    ├── configmap.yaml
    ├── secrets.yaml
    ├── hpa.yaml
    ├── ingress.yaml
    └── serviceaccount.yaml
```

---

## 8. Security Architecture

### 8.1 Authentication Flow

```
┌─────────┐                                    ┌────────────┐
│ Client  │                                    │ API Server │
└────┬────┘                                    └─────┬──────┘
     │                                               │
     │ POST /api/v1/auth/login                      │
     │ { username, password }                       │
     │──────────────────────────────────────────────>│
     │                                               │
     │                                               │ Validate credentials
     │                                               │ Generate JWT
     │                                               │
     │ { access_token, refresh_token, expires_in }  │
     │<──────────────────────────────────────────────│
     │                                               │
     │ GET /api/v1/benchmarks                       │
     │ Authorization: Bearer <access_token>         │
     │──────────────────────────────────────────────>│
     │                                               │
     │                                               │ Validate JWT
     │                                               │ Check permissions
     │                                               │
     │ { benchmarks: [...] }                        │
     │<──────────────────────────────────────────────│
     │                                               │
```

### 8.2 RBAC Model

```rust
/// Role-based access control
pub struct RbacEngine {
    policies: Vec<Policy>,
    role_permissions: HashMap<UserRole, Vec<Permission>>,
}

impl RbacEngine {
    /// Check if user has permission
    pub fn check_permission(
        &self,
        user_role: UserRole,
        permission: Permission,
    ) -> bool {
        self.role_permissions
            .get(&user_role)
            .map(|perms| perms.contains(&permission))
            .unwrap_or(false)
    }

    /// Check if user can access resource
    pub fn check_resource_access(
        &self,
        user_id: Uuid,
        resource: &Resource,
        action: Action,
    ) -> bool {
        // Check ownership, team access, public access, etc.
        true  // Simplified
    }
}

/// Default role permissions
impl Default for RbacEngine {
    fn default() -> Self {
        let mut role_permissions = HashMap::new();

        // Admin: Full access
        role_permissions.insert(
            UserRole::Admin,
            vec![
                Permission::RunBenchmark,
                Permission::ViewResults,
                Permission::ManageProviders,
                Permission::ManagePlugins,
                Permission::ManageUsers,
                Permission::AdminAccess,
            ],
        );

        // User: Standard access
        role_permissions.insert(
            UserRole::User,
            vec![
                Permission::RunBenchmark,
                Permission::ViewResults,
                Permission::ManageProviders,
            ],
        );

        // Viewer: Read-only
        role_permissions.insert(
            UserRole::Viewer,
            vec![Permission::ViewResults],
        );

        Self {
            policies: Vec::new(),
            role_permissions,
        }
    }
}
```

### 8.3 API Key Management

```rust
/// API key management
pub struct ApiKeyManager {
    db: Arc<dyn Database>,
    hasher: Argon2,
}

impl ApiKeyManager {
    /// Generate new API key
    pub async fn generate_key(&self, user_id: Uuid) -> Result<String, AuthError> {
        // Generate random key
        let key = Self::generate_random_key();

        // Hash key
        let hash = self.hash_key(&key)?;

        // Store hash in database
        self.db.insert_api_key(user_id, &hash).await?;

        // Return plaintext key (only time it's visible)
        Ok(key)
    }

    /// Validate API key
    pub async fn validate_key(&self, key: &str) -> Result<Uuid, AuthError> {
        let hash = self.hash_key(key)?;

        match self.db.get_user_by_api_key(&hash).await? {
            Some(user_id) => Ok(user_id),
            None => Err(AuthError::InvalidApiKey),
        }
    }

    /// Revoke API key
    pub async fn revoke_key(&self, user_id: Uuid) -> Result<(), AuthError> {
        self.db.delete_api_key(user_id).await
    }

    fn generate_random_key() -> String {
        use rand::Rng;
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
        const KEY_LEN: usize = 32;

        let mut rng = rand::thread_rng();
        let key: String = (0..KEY_LEN)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect();

        format!("ltb_{}", key)  // Prefix for identification
    }

    fn hash_key(&self, key: &str) -> Result<String, AuthError> {
        // Use Argon2 for secure hashing
        let salt = SaltString::generate(&mut OsRng);
        let password_hash = self.hasher
            .hash_password(key.as_bytes(), &salt)?
            .to_string();

        Ok(password_hash)
    }
}
```

### 8.4 Secrets Management

```rust
/// Secrets manager (integrates with Vault, AWS Secrets Manager, etc.)
#[async_trait]
pub trait SecretsManager: Send + Sync {
    /// Get secret by name
    async fn get_secret(&self, name: &str) -> Result<String, SecretsError>;

    /// Set secret
    async fn set_secret(&self, name: &str, value: &str) -> Result<(), SecretsError>;

    /// Delete secret
    async fn delete_secret(&self, name: &str) -> Result<(), SecretsError>;

    /// Rotate secret
    async fn rotate_secret(&self, name: &str) -> Result<String, SecretsError>;
}

/// Kubernetes Secrets backend
pub struct K8sSecretsManager {
    client: Client,
    namespace: String,
}

/// HashiCorp Vault backend
pub struct VaultSecretsManager {
    client: vaultrs::Client,
    mount_path: String,
}

/// AWS Secrets Manager backend
pub struct AwsSecretsManager {
    client: aws_sdk_secretsmanager::Client,
    region: String,
}
```

---

## 9. Technology Stack

### 9.1 Core Dependencies

```toml
[dependencies]
# Existing Phase 4 dependencies
tokio = { version = "1.40", features = ["full"] }
reqwest = { version = "0.12", features = ["json", "stream", "rustls-tls"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
anyhow = "1.0"
thiserror = "1.0"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }

# NEW Phase 5 dependencies

# Web framework
axum = { version = "0.7", features = ["ws", "multipart", "macros"] }
tower = { version = "0.4", features = ["limit", "timeout", "retry"] }
tower-http = { version = "0.5", features = ["cors", "compression", "trace"] }
tower-governor = "0.3"  # Rate limiting

# GraphQL
async-graphql = { version = "7.0", features = ["chrono", "uuid"] }
async-graphql-axum = "7.0"

# WebSocket
tokio-tungstenite = "0.21"

# Database
sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "postgres", "sqlite", "chrono", "uuid", "json"] }
refinery = { version = "0.8", features = ["tokio-postgres"] }

# Connection pooling
deadpool-postgres = "0.13"

# Caching
redis = { version = "0.24", features = ["tokio-comp", "connection-manager"] }

# Authentication
jsonwebtoken = "9.2"
argon2 = "0.5"
rand = "0.8"

# WASM plugins
wasmer = "4.2"
wasmer-compiler-cranelift = "4.2"
wasmtime = { version = "15.0", optional = true }

# gRPC (for distributed)
tonic = { version = "0.10", features = ["tls", "compression"] }
prost = "0.12"

# Observability
opentelemetry = { version = "0.21", features = ["rt-tokio"] }
opentelemetry-otlp = { version = "0.14", features = ["grpc-tonic"] }
opentelemetry-prometheus = "0.14"
prometheus = "0.13"
prometheus-client = "0.22"

# Image processing
image = { version = "0.24", features = ["jpeg", "png", "webp"] }

# Audio processing
symphonia = { version = "0.5", features = ["all"] }

# Kubernetes
kube = { version = "0.87", features = ["client", "runtime", "derive"] }
k8s-openapi = { version = "0.20", features = ["v1_28"] }

# OAuth2
oauth2 = "4.4"

# Distributed consensus (optional)
etcd-client = { version = "0.12", optional = true }

# Message queue (optional)
lapin = { version = "2.3", optional = true }  # RabbitMQ

# Monitoring
sysinfo = "0.30"  # System metrics
```

### 9.2 Technology Decisions

| Feature | Technology | Rationale |
|---------|-----------|-----------|
| **Web Framework** | Axum | Tokio-native, fast, type-safe, excellent async support |
| **Database** | PostgreSQL + SQLx | Production-ready, JSON support, async, compile-time checking |
| **Caching** | Redis | Fast, pub/sub, distributed caching |
| **GraphQL** | async-graphql | Excellent async support, type-safe, subscriptions |
| **WebSocket** | tokio-tungstenite | Tokio-native, efficient, widely used |
| **WASM Runtime** | Wasmer | Better ecosystem, good docs, stable API |
| **Alternative** | Wasmtime | Faster execution, Mozilla-backed (optional) |
| **gRPC** | Tonic | Rust-native, async, efficient, good tooling |
| **Observability** | OpenTelemetry | Industry standard, vendor-neutral, comprehensive |
| **Metrics** | Prometheus | De facto standard, excellent ecosystem |
| **Image** | image crate | Pure Rust, many formats, well-maintained |
| **Audio** | Symphonia | Pure Rust, comprehensive format support |
| **K8s** | kube-rs | Official Rust client, async, well-maintained |
| **Auth** | jsonwebtoken | Standard JWT, secure, well-tested |
| **Hashing** | Argon2 | Secure password hashing, resistant to attacks |

### 9.3 Alternative Technologies Considered

| Category | Primary Choice | Alternative | Reason for Primary |
|----------|----------------|-------------|-------------------|
| Web Framework | Axum | Actix-web | Better Tokio integration, simpler |
| Database ORM | SQLx | Diesel | Async support, compile-time checks |
| WASM Runtime | Wasmer | Wasmtime | Better ecosystem, easier API |
| Message Queue | Redis Pub/Sub | RabbitMQ | Simpler, already using Redis |
| Consensus | Direct HTTP | etcd | Simpler for initial version |
| GraphQL | async-graphql | Juniper | Better async support |

---

## 10. Performance Considerations

### 10.1 Performance Targets

| Metric | Target | Measurement |
|--------|--------|-------------|
| **API Latency (p50)** | <50ms | HTTP request-response |
| **API Latency (p95)** | <200ms | HTTP request-response |
| **API Latency (p99)** | <500ms | HTTP request-response |
| **WebSocket Latency** | <100ms | Event delivery |
| **Database Query (p95)** | <100ms | Simple queries |
| **Evaluation Cache Hit** | >80% | LRU cache effectiveness |
| **Memory Usage (API)** | <500MB | Per server instance |
| **Memory Usage (Worker)** | <2GB | Per worker instance |
| **Throughput** | >1000 req/s | API server, single instance |
| **Concurrent Benchmarks** | >100 | Distributed system |

### 10.2 Optimization Strategies

#### Async Everything
```rust
// Use tokio for all I/O operations
#[tokio::main]
async fn main() {
    // All operations are async
    let results = tokio::join!(
        fetch_provider_data(),
        query_database(),
        call_llm_api(),
    );
}
```

#### Zero-Copy with Bytes
```rust
use bytes::Bytes;

// Avoid copying large payloads
pub struct Response {
    pub body: Bytes,  // Reference-counted, zero-copy
}
```

#### Connection Pooling
```rust
use deadpool_postgres::{Config, Pool, Runtime};

pub async fn create_pool(database_url: &str) -> Pool {
    let mut cfg = Config::new();
    cfg.url = Some(database_url.to_string());
    cfg.pool = Some(deadpool_postgres::PoolConfig {
        max_size: 20,
        timeouts: deadpool_postgres::Timeouts {
            wait: Some(Duration::from_secs(5)),
            create: Some(Duration::from_secs(5)),
            recycle: Some(Duration::from_secs(5)),
        },
    });

    cfg.create_pool(Some(Runtime::Tokio1)).unwrap()
}
```

#### Multi-Level Caching
```rust
pub struct CacheLayer {
    // L1: In-memory LRU cache (fastest)
    memory_cache: Arc<Mutex<LruCache<String, CachedValue>>>,

    // L2: Redis (shared across instances)
    redis: Arc<redis::Client>,

    // L3: Database (persistent)
    db: Arc<dyn Database>,
}

impl CacheLayer {
    pub async fn get(&self, key: &str) -> Option<CachedValue> {
        // Try L1
        if let Some(value) = self.memory_cache.lock().await.get(key) {
            return Some(value.clone());
        }

        // Try L2
        if let Ok(value) = self.redis.get(key).await {
            self.memory_cache.lock().await.put(key.to_string(), value.clone());
            return Some(value);
        }

        // Try L3
        if let Ok(Some(value)) = self.db.get_cached_value(key).await {
            self.redis.set(key, &value).await.ok();
            self.memory_cache.lock().await.put(key.to_string(), value.clone());
            return Some(value);
        }

        None
    }
}
```

#### Batch Processing
```rust
/// Batch multiple operations
pub async fn process_batch<T, R>(
    items: Vec<T>,
    batch_size: usize,
    process_fn: impl Fn(Vec<T>) -> Future<Output = Result<Vec<R>>>,
) -> Result<Vec<R>> {
    let mut results = Vec::new();

    for batch in items.chunks(batch_size) {
        let batch_results = process_fn(batch.to_vec()).await?;
        results.extend(batch_results);
    }

    Ok(results)
}
```

#### Lazy Evaluation
```rust
/// Defer expensive operations until needed
pub struct LazyResult {
    result: OnceCell<BenchmarkResult>,
    compute_fn: Box<dyn Fn() -> Future<Output = BenchmarkResult>>,
}

impl LazyResult {
    pub async fn get(&self) -> &BenchmarkResult {
        self.result.get_or_init(|| (self.compute_fn)()).await
    }
}
```

### 10.3 Load Testing

```rust
// load-test/src/main.rs

use goose::prelude::*;

#[tokio::main]
async fn main() -> Result<(), GooseError> {
    GooseAttack::initialize()?
        .register_scenario(
            scenario!("API Load Test")
                .register_transaction(transaction!(health_check))
                .register_transaction(transaction!(create_benchmark).set_weight(5)?)
                .register_transaction(transaction!(query_results).set_weight(10)?)
        )
        .set_default(GooseDefault::Host, "http://localhost:8080")?
        .set_default(GooseDefault::Users, 100)?
        .set_default(GooseDefault::RunTime, 60)?
        .execute()
        .await?;

    Ok(())
}

async fn health_check(user: &mut GooseUser) -> TransactionResult {
    let _response = user.get("/health").await?;
    Ok(())
}

async fn create_benchmark(user: &mut GooseUser) -> TransactionResult {
    let request = serde_json::json!({
        "dataset": "test",
        "provider": "openai",
        "model": "gpt-4",
    });

    let _response = user.post_json("/api/v1/benchmarks", &request).await?;
    Ok(())
}
```

---

## 11. Migration Strategy

### 11.1 Backward Compatibility

#### Config File Migration
```rust
/// Migrate Phase 4 config to Phase 5
pub fn migrate_config_v4_to_v5(v4_config: ConfigV4) -> ConfigV5 {
    ConfigV5 {
        // Existing Phase 4 fields
        providers: v4_config.providers,
        benchmarks: v4_config.benchmarks,
        evaluation: v4_config.evaluation,
        orchestration: v4_config.orchestration,
        analytics: v4_config.analytics,

        // New Phase 5 fields with defaults
        multimodal: MultiModalConfig::default(),
        monitoring: MonitoringConfig::default(),
        plugins: PluginConfig::default(),
        storage: StorageConfig::default(),
        server: ServerConfig::default(),
        distributed: None,  // Optional
    }
}
```

#### Database Migrations
```sql
-- Migration: 001_add_phase5_fields.sql

-- Add new columns to existing results table
ALTER TABLE results_v5
ADD COLUMN multi_modal_inputs JSONB,
ADD COLUMN plugin_metrics JSONB,
ADD COLUMN trace_id VARCHAR(255),
ADD COLUMN span_id VARCHAR(255),
ADD COLUMN session_id UUID,
ADD COLUMN tags TEXT[],
ADD COLUMN metadata JSONB,
ADD COLUMN provider_metadata JSONB,
ADD COLUMN cost_details JSONB,
ADD COLUMN cache_stats JSONB;

-- Create indexes for new columns
CREATE INDEX idx_results_trace_id ON results_v5(trace_id);
CREATE INDEX idx_results_session_id ON results_v5(session_id);
CREATE INDEX idx_results_tags ON results_v5 USING GIN(tags);

-- Backfill session_id for existing records
UPDATE results_v5
SET session_id = uuid_generate_v4()
WHERE session_id IS NULL;

ALTER TABLE results_v5
ALTER COLUMN session_id SET NOT NULL;
```

### 11.2 Feature Flags

```rust
/// Feature flags for gradual rollout
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureFlags {
    pub enable_multimodal: bool,
    pub enable_plugins: bool,
    pub enable_distributed: bool,
    pub enable_realtime: bool,
    pub enable_graphql: bool,
    pub enable_new_providers: bool,
}

impl Default for FeatureFlags {
    fn default() -> Self {
        Self {
            enable_multimodal: false,
            enable_plugins: false,
            enable_distributed: false,
            enable_realtime: true,  // Least risky
            enable_graphql: true,
            enable_new_providers: true,
        }
    }
}
```

### 11.3 Deprecation Warnings

```rust
/// Deprecation system
#[deprecated(since = "0.5.0", note = "Use `evaluate_v5` instead")]
pub fn evaluate_v4(/* ... */) -> Result<EvaluationResult> {
    tracing::warn!(
        "evaluate_v4 is deprecated and will be removed in version 0.6.0. \
         Please migrate to evaluate_v5."
    );

    // Forward to new implementation
    evaluate_v5(/* ... */)
}
```

### 11.4 Gradual Rollout Plan

**Phase 5.1: Foundation (Weeks 1-2)**
- Database migrations
- New provider framework
- API server infrastructure
- Monitoring basics

**Phase 5.2: Core Features (Weeks 3-4)**
- New providers (Gemini, Cohere, Mistral)
- Local providers (Ollama, LlamaCpp)
- Enhanced storage layer
- Real-time monitoring (WebSocket)

**Phase 5.3: Advanced Features (Weeks 5-6)**
- Multi-modal evaluation (vision, audio)
- Plugin system (WASM)
- Integration layer (Langchain, MLflow, W&B)

**Phase 5.4: Scale Features (Weeks 7-8)**
- Distributed architecture
- GraphQL API
- Advanced observability
- Full authentication/authorization

**Phase 5.5: Production Hardening (Weeks 9-10)**
- Load testing
- Security audits
- Performance optimization
- Documentation and examples

---

## 12. Development Roadmap

### 12.1 Implementation Phases

#### Phase 5.1: Foundation (2 weeks)
**Deliverables:**
- [ ] Database schema and migrations
- [ ] Enhanced provider trait
- [ ] Provider registry
- [ ] Basic API server (Axum)
- [ ] JWT authentication
- [ ] Health checks and basic monitoring

**Success Criteria:**
- API server running and accepting requests
- Database migrations successful
- Provider registry working
- Authentication functional

#### Phase 5.2: Providers & Storage (2 weeks)
**Deliverables:**
- [ ] Gemini provider
- [ ] Cohere provider
- [ ] Mistral provider
- [ ] Ollama provider
- [ ] LlamaCpp provider
- [ ] PostgreSQL backend
- [ ] SQLite backend
- [ ] Query builders

**Success Criteria:**
- All 5 new providers functional
- Database abstraction working
- Queries optimized
- Unit tests passing

#### Phase 5.3: Multi-Modal (2 weeks)
**Deliverables:**
- [ ] Vision evaluator
- [ ] Audio evaluator
- [ ] Multi-modal dataset support
- [ ] Image processing pipeline
- [ ] Audio processing pipeline
- [ ] Multi-modal result types

**Success Criteria:**
- Vision evaluation working with GPT-4V
- Audio transcription with Whisper
- Multi-modal results stored correctly
- Examples and tests

#### Phase 5.4: Real-Time & Monitoring (2 weeks)
**Deliverables:**
- [ ] WebSocket server
- [ ] Event bus
- [ ] Prometheus metrics
- [ ] Alert system
- [ ] Real-time dashboard (frontend)
- [ ] Grafana dashboards

**Success Criteria:**
- WebSocket connections stable
- Events flowing correctly
- Metrics exported to Prometheus
- Alerts triggering appropriately
- Dashboard updating in real-time

#### Phase 5.5: Plugins (2 weeks)
**Deliverables:**
- [ ] Plugin API definition
- [ ] WASM plugin loader
- [ ] Plugin registry
- [ ] Plugin SDK
- [ ] Example plugins
- [ ] Plugin documentation

**Success Criteria:**
- Plugins loading and executing
- Custom metrics working
- Sandbox security effective
- Plugin hot-reload functional
- SDK easy to use

#### Phase 5.6: Integrations (1 week)
**Deliverables:**
- [ ] Langchain integration
- [ ] LlamaIndex integration
- [ ] MLflow integration
- [ ] Weights & Biases integration
- [ ] Integration examples

**Success Criteria:**
- Data export working
- Import functional
- Bidirectional sync
- Examples documented

#### Phase 5.7: Distributed (2 weeks)
**Deliverables:**
- [ ] Coordinator server
- [ ] Worker nodes
- [ ] Task scheduler
- [ ] Result aggregator
- [ ] gRPC protocol
- [ ] Kubernetes operator

**Success Criteria:**
- Coordinator-worker communication stable
- Tasks distributed correctly
- Results aggregated properly
- K8s operator deploying successfully
- Auto-scaling functional

#### Phase 5.8: GraphQL & Advanced API (1 week)
**Deliverables:**
- [ ] GraphQL schema
- [ ] Query resolvers
- [ ] Mutation resolvers
- [ ] Subscriptions
- [ ] GraphQL Playground
- [ ] API documentation

**Success Criteria:**
- All queries working
- Mutations functional
- Subscriptions real-time
- Documentation complete
- Examples provided

#### Phase 5.9: Security & RBAC (1 week)
**Deliverables:**
- [ ] RBAC engine
- [ ] OAuth2 integration
- [ ] API key management
- [ ] Audit logging
- [ ] Rate limiting
- [ ] Security hardening

**Success Criteria:**
- RBAC policies enforced
- OAuth working
- API keys secure
- Audit trail complete
- Rate limits effective

#### Phase 5.10: Testing & Documentation (2 weeks)
**Deliverables:**
- [ ] Comprehensive unit tests
- [ ] Integration tests
- [ ] Load tests
- [ ] Security tests
- [ ] API documentation
- [ ] User guides
- [ ] Architecture docs
- [ ] Video tutorials

**Success Criteria:**
- >90% code coverage
- All tests passing
- Load tests showing targets met
- Security audit passed
- Documentation complete
- Examples working

### 12.2 Resource Requirements

**Team:**
- 1-2 Senior Rust Engineers (full-time)
- 1 Frontend Engineer (part-time, for real-time dashboard)
- 1 DevOps Engineer (part-time, for K8s/deployment)
- 1 Technical Writer (part-time, for documentation)

**Infrastructure:**
- Development environment (local)
- Staging environment (K8s cluster)
- CI/CD pipeline (GitHub Actions)
- Testing infrastructure (load testing)
- API credentials (OpenAI, Anthropic, Gemini, Cohere, Mistral)

**Timeline:**
- **Total: 16 weeks (4 months)**
- Foundation & Providers: 4 weeks
- Multi-Modal & Monitoring: 4 weeks
- Plugins & Integrations: 3 weeks
- Distributed & Advanced: 3 weeks
- Testing & Documentation: 2 weeks

### 12.3 Risk Mitigation

**Technical Risks:**
1. **WASM Plugin Performance** → Mitigation: Benchmark early, use Wasmtime if needed
2. **Distributed Coordination** → Mitigation: Use proven patterns (gRPC), etcd if needed
3. **Database Schema Changes** → Mitigation: Thorough migration testing, rollback plan
4. **WebSocket Scalability** → Mitigation: Load test early, Redis pub/sub for horizontal scaling

**Schedule Risks:**
1. **Multi-Modal Complexity** → Mitigation: MVP first, iterate
2. **Plugin API Design** → Mitigation: Study existing systems (Webpack, VS Code)
3. **Integration Challenges** → Mitigation: Partner with integration teams, start simple

**Mitigation Strategies:**
- Weekly progress reviews
- Bi-weekly demos to stakeholders
- Feature flags for gradual rollout
- Comprehensive testing at each phase
- Documentation as we build

---

## Conclusion

This technical architecture provides a comprehensive blueprint for Phase 5 of the LLM Test Bench, building on the solid foundation of Phases 1-4. The design prioritizes:

1. **Modularity**: Clean separation of concerns, easy to extend
2. **Performance**: Async-first, zero-copy, multi-level caching
3. **Scalability**: Distributed architecture, horizontal scaling
4. **Security**: RBAC, OAuth, audit logging, secrets management
5. **Extensibility**: Plugin system, integration adapters
6. **Production-Ready**: Comprehensive monitoring, alerting, deployment

The phased rollout ensures gradual delivery of value while maintaining stability. Feature flags enable safe deployment and rollback if needed.

**Next Steps:**
1. Review and approve architecture
2. Set up development environment
3. Begin Phase 5.1 implementation
4. Regular sync meetings with stakeholders
5. Continuous documentation updates

---

**Document Version:** 1.0
**Last Updated:** November 4, 2025
**Status:** Ready for Review
**Owner:** Technical Architecture Team

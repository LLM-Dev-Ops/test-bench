# Phase 5 Architecture Diagrams
## Visual Reference

**Version:** 1.0
**Date:** November 4, 2025

---

## Table of Contents

1. [System Overview](#system-overview)
2. [Module Architecture](#module-architecture)
3. [Data Flow](#data-flow)
4. [Distributed Architecture](#distributed-architecture)
5. [API Server Architecture](#api-server-architecture)
6. [Real-Time Event Flow](#real-time-event-flow)
7. [Plugin System](#plugin-system)
8. [Deployment Architectures](#deployment-architectures)

---

## System Overview

```
┌─────────────────────────────────────────────────────────────────────┐
│                      LLM Test Bench Phase 5                          │
│                     Production-Scale Platform                        │
└─────────────────────────────────────────────────────────────────────┘

                                 ┌─────────────┐
                                 │   Users     │
                                 └──────┬──────┘
                                        │
                    ┌───────────────────┼───────────────────┐
                    │                   │                   │
                    ▼                   ▼                   ▼
            ┌──────────────┐    ┌──────────────┐   ┌──────────────┐
            │     CLI      │    │  API Server  │   │  WebSocket   │
            │   Commands   │    │ REST/GraphQL │   │   Real-Time  │
            └──────┬───────┘    └──────┬───────┘   └──────┬───────┘
                   │                   │                   │
                   └───────────────────┼───────────────────┘
                                       │
                        ┌──────────────┴──────────────┐
                        │      Core Framework         │
                        └──────────────┬──────────────┘
                                       │
        ┌──────────────────────────────┼──────────────────────────────┐
        │                              │                              │
        ▼                              ▼                              ▼
┌───────────────┐            ┌─────────────────┐          ┌──────────────┐
│   Providers   │            │   Evaluators    │          │ Orchestration│
│ (10+ LLMs)    │            │ (Multi-Modal)   │          │  (Compare)   │
└───────┬───────┘            └────────┬────────┘          └──────┬───────┘
        │                              │                          │
        │         ┌────────────────────┼──────────────┐          │
        │         │                    │              │          │
        ▼         ▼                    ▼              ▼          ▼
┌──────────────────────────────────────────────────────────────────┐
│                        Storage Layer                              │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │  PostgreSQL  │  │    Redis     │  │   S3/Disk    │          │
│  │  (Results)   │  │  (Cache)     │  │  (Artifacts) │          │
│  └──────────────┘  └──────────────┘  └──────────────┘          │
└──────────────────────────────────────────────────────────────────┘
        │                              │                          │
        └──────────────────────────────┼──────────────────────────┘
                                       │
                        ┌──────────────┴──────────────┐
                        │     Observability           │
                        │  Prometheus + Grafana       │
                        │  OpenTelemetry Traces       │
                        └─────────────────────────────┘
```

---

## Module Architecture

```
llm-test-bench/
│
├── cli/                                  ┌─────────────────┐
│   ├── commands/                         │  CLI Interface  │
│   │   ├── bench.rs                      │  (Existing)     │
│   │   ├── eval.rs                       └────────┬────────┘
│   │   ├── compare.rs                             │
│   │   └── dashboard.rs                           │
│   └── main.rs                                    │
│                                                   │
├── core/                                          ▼
│   ├── providers/               ┌──────────────────────────────────┐
│   │   ├── traits.rs            │      Core Framework              │
│   │   ├── openai.rs            │                                  │
│   │   ├── anthropic.rs         │  ┌────────────────────────────┐ │
│   │   ├── gemini.rs       ←────┼──┤  NEW: Provider Expansion  │ │
│   │   ├── cohere.rs            │  │  • Gemini, Cohere, Mistral│ │
│   │   ├── mistral.rs           │  │  • Ollama, LlamaCpp       │ │
│   │   ├── local/               │  │  • Dynamic registry       │ │
│   │   │   ├── ollama.rs        │  └────────────────────────────┘ │
│   │   │   └── llamacpp.rs      │                                  │
│   │   └── registry.rs          │  ┌────────────────────────────┐ │
│   │                             │  │  NEW: Multi-Modal         │ │
│   ├── multimodal/          ←───┼──┤  • Vision evaluation     │ │
│   │   ├── vision.rs            │  │  • Audio evaluation      │ │
│   │   ├── audio.rs             │  │  • Image/audio processing│ │
│   │   └── metrics.rs           │  └────────────────────────────┘ │
│   │                             │                                  │
│   ├── monitoring/          ←───┼──┤  NEW: Real-Time Monitoring│ │
│   │   ├── realtime.rs          │  │  • WebSocket server      │ │
│   │   ├── metrics.rs           │  │  • Prometheus metrics    │ │
│   │   └── alerts.rs            │  │  • Alert system          │ │
│   │                             │  └────────────────────────────┘ │
│   │                             │                                  │
│   ├── plugins/             ←───┼──┤  NEW: Plugin System      │ │
│   │   ├── loader.rs            │  │  • WASM loader           │ │
│   │   ├── registry.rs          │  │  • Plugin registry       │ │
│   │   └── sandbox.rs           │  │  • Security sandbox      │ │
│   │                             │  └────────────────────────────┘ │
│   │                             │                                  │
│   ├── integrations/        ←───┼──┤  NEW: Integrations       │ │
│   │   ├── langchain.rs         │  │  • Langchain, LlamaIndex │ │
│   │   ├── mlflow.rs            │  │  • MLflow, W&B           │ │
│   │   └── wandb.rs             │  └────────────────────────────┘ │
│   │                             │                                  │
│   ├── storage/             ←───┼──┤  NEW: Storage Layer      │ │
│   │   ├── database.rs          │  │  • PostgreSQL, SQLite    │ │
│   │   ├── postgres.rs          │  │  • Migrations            │ │
│   │   └── migrations/          │  │  • Query builders        │ │
│   │                             │  └────────────────────────────┘ │
│   │                             │                                  │
│   ├── evaluators/              │  Phase 4 (Existing)              │
│   ├── orchestration/           │  • Comparison, Ranking           │
│   ├── analytics/               │  • Statistics, Cost Optimizer    │
│   └── visualization/           │  • Dashboards, Charts            │
│                                 └──────────────────────────────────┘
│
└── server/                     ┌──────────────────────────────────┐
    ├── api/                    │  NEW: API Server (Separate Crate)│
    │   ├── rest.rs             │                                  │
    │   ├── graphql.rs          │  ┌────────────────────────────┐ │
    │   └── websocket.rs        │  │  REST API                  │ │
    ├── auth/                   │  │  • Benchmarks, Results     │ │
    │   ├── jwt.rs              │  │  • Providers, Plugins      │ │
    │   ├── rbac.rs             │  └────────────────────────────┘ │
    │   └── oauth.rs            │                                  │
    └── middleware/             │  ┌────────────────────────────┐ │
        ├── auth.rs             │  │  GraphQL API               │ │
        ├── rate_limit.rs       │  │  • Queries, Mutations      │ │
        └── logging.rs          │  │  • Subscriptions (real-time)│
                                │  └────────────────────────────┘ │
                                │                                  │
                                │  ┌────────────────────────────┐ │
                                │  │  Authentication            │ │
                                │  │  • JWT, OAuth2             │ │
                                │  │  • RBAC, API keys          │ │
                                │  └────────────────────────────┘ │
                                └──────────────────────────────────┘
```

---

## Data Flow

### Benchmark Execution Flow

```
┌──────────┐
│  User    │
└─────┬────┘
      │ 1. Submit benchmark
      ▼
┌──────────────┐
│  CLI / API   │
└──────┬───────┘
       │ 2. Parse config
       │ 3. Load dataset
       ▼
┌──────────────────┐
│ Benchmark Runner │
└────────┬─────────┘
         │ 4. Emit: BenchmarkStarted event
         │
         ├──────────────────────────────────────────┐
         │ 5. For each test in dataset:             │
         │                                           │
         │   ┌─────────────────────────┐            │
         │   │ a. Select provider      │            │
         │   └────────┬────────────────┘            │
         │            │                              │
         │            ▼                              │
         │   ┌─────────────────────────┐            │
         │   │ b. Call LLM provider    │            │
         │   │    (OpenAI, Anthropic,  │            │
         │   │     Gemini, etc.)       │            │
         │   └────────┬────────────────┘            │
         │            │                              │
         │            ▼                              │
         │   ┌─────────────────────────┐            │
         │   │ c. Run evaluations      │            │
         │   │    - Faithfulness       │            │
         │   │    - Relevance          │            │
         │   │    - Coherence          │            │
         │   │    - Custom (plugins)   │            │
         │   └────────┬────────────────┘            │
         │            │                              │
         │            ▼                              │
         │   ┌─────────────────────────┐            │
         │   │ d. Store result         │            │
         │   │    - PostgreSQL         │            │
         │   │    - Cache (Redis)      │            │
         │   └────────┬────────────────┘            │
         │            │                              │
         │            ▼                              │
         │   ┌─────────────────────────┐            │
         │   │ e. Emit: TestCompleted  │            │
         │   │    event (WebSocket)    │            │
         │   └─────────────────────────┘            │
         │                                           │
         └───────────────────────────────────────────┘
         │ 6. Aggregate results
         ▼
┌─────────────────┐
│ Result Summary  │
└────────┬────────┘
         │ 7. Emit: BenchmarkCompleted event
         │ 8. Generate dashboard
         ▼
┌─────────────────┐
│  User (Result)  │
└─────────────────┘
```

### Multi-Modal Evaluation Flow

```
┌──────────────────┐
│  Multi-Modal     │
│  Input Request   │
│  • Prompt        │
│  • Images        │
│  • Audio         │
└────────┬─────────┘
         │
         ▼
┌────────────────────────┐
│  Input Preprocessing   │
│  • Load images         │
│  • Decode audio        │
│  • Validate formats    │
└────────┬───────────────┘
         │
         ├──────────────────┬──────────────────┐
         │                  │                  │
         ▼                  ▼                  ▼
┌─────────────┐   ┌─────────────┐   ┌─────────────┐
│   Vision    │   │    Audio    │   │    Text     │
│  Provider   │   │  Provider   │   │  Provider   │
│             │   │             │   │             │
│ GPT-4V      │   │  Whisper    │   │   GPT-4     │
│ Gemini Pro  │   │  Gemini     │   │  Claude     │
└──────┬──────┘   └──────┬──────┘   └──────┬──────┘
       │                 │                  │
       └─────────────────┼──────────────────┘
                         │ Responses
                         ▼
          ┌──────────────────────────┐
          │  Multi-Modal Evaluation  │
          │  • Vision metrics        │
          │  • Audio metrics         │
          │  • Cross-modal coherence │
          └───────────┬──────────────┘
                      │
                      ▼
          ┌───────────────────────┐
          │  Unified Result       │
          │  • All modalities     │
          │  • Combined scores    │
          └───────────────────────┘
```

---

## Distributed Architecture

### Coordinator-Worker Pattern

```
                    ┌───────────────────────────┐
                    │        Client             │
                    │    (CLI, API, Web UI)     │
                    └─────────────┬─────────────┘
                                  │
                                  │ Submit benchmark
                                  │
                                  ▼
          ┌────────────────────────────────────────────┐
          │             Coordinator                     │
          │                                             │
          │  ┌──────────────────────────────────┐     │
          │  │   Benchmark Manager              │     │
          │  │   • Parse config                 │     │
          │  │   • Load dataset                 │     │
          │  │   • Create tasks                 │     │
          │  └──────────────┬───────────────────┘     │
          │                 │                          │
          │  ┌──────────────▼───────────────────┐     │
          │  │   Task Scheduler                 │     │
          │  │   • Queue tasks                  │     │
          │  │   • Load balancing               │     │
          │  │   • Work stealing                │     │
          │  └──────────────┬───────────────────┘     │
          │                 │                          │
          │  ┌──────────────▼───────────────────┐     │
          │  │   Worker Registry                │     │
          │  │   • Track available workers      │     │
          │  │   • Health monitoring            │     │
          │  │   • Capability matching          │     │
          │  └──────────────┬───────────────────┘     │
          │                 │                          │
          │  ┌──────────────▼───────────────────┐     │
          │  │   Result Aggregator              │     │
          │  │   • Collect results              │     │
          │  │   • Generate summary             │     │
          │  └──────────────────────────────────┘     │
          └─────────────────┬───────────────────────────┘
                            │ gRPC
                            │
        ┌───────────────────┼───────────────────┬───────────────────┐
        │                   │                   │                   │
        ▼                   ▼                   ▼                   ▼
┌───────────────┐   ┌───────────────┐   ┌───────────────┐   ┌───────────────┐
│   Worker 1    │   │   Worker 2    │   │   Worker 3    │   │   Worker N    │
│               │   │               │   │               │   │               │
│ ┌───────────┐ │   │ ┌───────────┐ │   │ ┌───────────┐ │   │ ┌───────────┐ │
│ │  Executor │ │   │ │  Executor │ │   │ │  Executor │ │   │ │  Executor │ │
│ └─────┬─────┘ │   │ └─────┬─────┘ │   │ └─────┬─────┘ │   │ └─────┬─────┘ │
│       │       │   │       │       │   │       │       │   │       │       │
│       ▼       │   │       ▼       │   │       ▼       │   │       ▼       │
│ ┌───────────┐ │   │ ┌───────────┐ │   │ ┌───────────┐ │   │ ┌───────────┐ │
│ │ Providers │ │   │ │ Providers │ │   │ │ Providers │ │   │ │ Providers │ │
│ │ Evaluators│ │   │ │ Evaluators│ │   │ │ Evaluators│ │   │ │ Evaluators│ │
│ └───────────┘ │   │ └───────────┘ │   │ └───────────┘ │   │ └───────────┘ │
│               │   │               │   │               │   │               │
│ Capability:   │   │ Capability:   │   │ Capability:   │   │ Capability:   │
│ • GPU: No     │   │ • GPU: Yes    │   │ • GPU: No     │   │ • GPU: No     │
│ • Max: 5      │   │ • Max: 10     │   │ • Max: 5      │   │ • Max: 5      │
└───────────────┘   └───────────────┘   └───────────────┘   └───────────────┘

                             ┌────────────────┐
                             │   Storage      │
                             │                │
                             │ • PostgreSQL   │
                             │ • Redis        │
                             │ • S3           │
                             └────────────────┘
```

### Task Distribution Algorithm

```
┌────────────────────────────────────────────────────┐
│  Coordinator: Task Distribution                    │
└────────────────────────────────────────────────────┘

1. Receive benchmark request
   └─> Parse config
   └─> Load dataset (1000 tests)

2. Create tasks from dataset
   └─> Split into chunks (100 tasks × 10 tests each)

3. For each task:
   ├─> Check required capabilities
   │   └─> GPU required? Vision model?
   │
   ├─> Find eligible workers
   │   └─> Query worker registry
   │   └─> Filter by capabilities
   │
   ├─> Select best worker
   │   └─> Least loaded
   │   └─> Closest network-wise
   │   └─> Previously successful
   │
   ├─> Assign task to worker
   │   └─> gRPC: SendTask(task)
   │   └─> Track in-progress
   │
   └─> Set timeout (5 minutes)

4. Monitor task execution
   ├─> Receive progress updates
   ├─> Handle failures
   │   └─> Retry on different worker
   │   └─> Max 3 retries
   └─> Collect results

5. Aggregate results when all complete
   └─> Generate summary
   └─> Notify client
```

---

## API Server Architecture

### Request Flow

```
┌─────────────┐
│   Client    │
│ (Browser,   │
│  Mobile,    │
│  CLI, etc.) │
└──────┬──────┘
       │
       │ HTTP/HTTPS Request
       │
       ▼
┌──────────────────────────────────────┐
│        Load Balancer / Ingress       │
│         (Nginx, K8s Ingress)         │
└─────────────────┬────────────────────┘
                  │
                  ▼
┌──────────────────────────────────────────────────────┐
│               API Server (Axum)                       │
│                                                       │
│  ┌────────────────────────────────────────────────┐ │
│  │           Middleware Pipeline                  │ │
│  │                                                 │ │
│  │  1. CORS       → Check origin                  │ │
│  │  2. Rate Limit → Check quota                   │ │
│  │  3. Auth       → Validate JWT/API key          │ │
│  │  4. Logging    → Log request                   │ │
│  └────────────────────┬───────────────────────────┘ │
│                       │                              │
│       ┌───────────────┼───────────────┐             │
│       │               │               │             │
│       ▼               ▼               ▼             │
│  ┌─────────┐   ┌──────────┐   ┌──────────┐        │
│  │  REST   │   │ GraphQL  │   │WebSocket │        │
│  │  API    │   │   API    │   │   API    │        │
│  └────┬────┘   └─────┬────┘   └─────┬────┘        │
│       │              │              │              │
│       └──────────────┼──────────────┘              │
│                      │                              │
│       ┌──────────────┴──────────────┐              │
│       │                              │              │
│       ▼                              ▼              │
│  ┌─────────────────┐        ┌─────────────────┐   │
│  │   Business      │        │   Real-Time     │   │
│  │   Logic         │        │   Events        │   │
│  │   • Benchmark   │        │   • WebSocket   │   │
│  │   • Evaluation  │        │   • Pub/Sub     │   │
│  │   • Providers   │        │   • Broadcast   │   │
│  └────────┬────────┘        └─────────────────┘   │
│           │                                         │
└───────────┼─────────────────────────────────────────┘
            │
            ▼
┌───────────────────────────────────────┐
│         Shared Services               │
│                                        │
│  ┌──────────┐  ┌──────────┐  ┌──────┐│
│  │PostgreSQL│  │  Redis   │  │  S3  ││
│  │(Results) │  │ (Cache)  │  │(Files)││
│  └──────────┘  └──────────┘  └──────┘│
└───────────────────────────────────────┘
```

### API Endpoint Organization

```
/api/v1/
│
├── /auth/
│   ├── POST   /login           → Authenticate user, get JWT
│   ├── POST   /refresh         → Refresh JWT token
│   ├── POST   /logout          → Revoke token
│   ├── POST   /apikey          → Generate API key
│   └── DELETE /apikey          → Revoke API key
│
├── /benchmarks/
│   ├── GET    /                → List benchmarks (paginated)
│   ├── POST   /                → Create new benchmark
│   ├── GET    /:id             → Get benchmark details
│   ├── DELETE /:id             → Cancel/delete benchmark
│   └── POST   /:id/cancel      → Cancel running benchmark
│
├── /results/
│   ├── GET    /                → Query results (filters)
│   ├── GET    /:id             → Get specific result
│   ├── POST   /export          → Export results (CSV, JSON)
│   └── GET    /stats           → Aggregate statistics
│
├── /providers/
│   ├── GET    /                → List all providers
│   ├── GET    /:name           → Get provider details
│   ├── GET    /:name/models    → List provider models
│   ├── POST   /:name/health    → Health check provider
│   └── POST   /                → Register custom provider
│
├── /evaluate/
│   ├── POST   /                → Evaluate single response
│   ├── POST   /batch           → Batch evaluation
│   └── POST   /compare         → Compare multiple models
│
├── /plugins/
│   ├── GET    /                → List installed plugins
│   ├── POST   /                → Upload/install plugin
│   ├── GET    /:name           → Get plugin details
│   ├── DELETE /:name           → Uninstall plugin
│   └── POST   /:name/reload    → Hot reload plugin
│
├── /monitoring/
│   ├── GET    /metrics         → Prometheus metrics
│   ├── GET    /health          → Health check
│   ├── GET    /health/live     → Liveness probe
│   └── GET    /health/ready    → Readiness probe
│
└── /integrations/
    ├── POST   /langchain/export      → Export to Langchain
    ├── POST   /mlflow/log            → Log to MLflow
    └── POST   /wandb/log             → Log to W&B
```

---

## Real-Time Event Flow

### WebSocket Subscription Flow

```
┌─────────────┐
│   Browser   │
│  Dashboard  │
└──────┬──────┘
       │ 1. Connect WebSocket
       │    ws://localhost:8080/ws
       ▼
┌──────────────────────────┐
│  WebSocket Server        │
│                          │
│  ┌────────────────────┐ │
│  │  Connection Manager│ │
│  │  • Assign ID       │ │
│  │  • Track client    │ │
│  └────────┬───────────┘ │
└───────────┼──────────────┘
            │
            │ 2. Client subscribes
            │    { type: "Subscribe",
            │      topics: ["benchmark.progress"] }
            ▼
┌───────────────────────────────────┐
│  Event Bus (tokio channels)       │
│                                    │
│  Topics:                           │
│  • benchmark.started               │
│  • benchmark.progress              │
│  • benchmark.completed             │
│  • test.completed                  │
│  • metrics.*                       │
│  • alerts.*                        │
│                                    │
│  Subscriptions:                    │
│  • Client ABC → [benchmark.*]     │
│  • Client XYZ → [metrics.*]       │
└─────────────┬─────────────────────┘
              │
              │ 3. Benchmark runs
              │    Emits events
              ▼
    ┌─────────────────────────┐
    │  Benchmark Executor     │
    │                         │
    │  event_bus.publish(     │
    │    "benchmark.progress",│
    │    {                    │
    │      progress: 50%,     │
    │      current: "test-5"  │
    │    }                    │
    │  );                     │
    └─────────────────────────┘
              │
              │ 4. Event broadcast
              ▼
┌─────────────────────────────────┐
│  Event Bus                      │
│  • Matches subscriptions        │
│  • Sends to subscribed clients  │
└─────────────┬───────────────────┘
              │
              │ 5. WebSocket message
              │    { type: "Event",
              │      payload: {...} }
              ▼
┌─────────────────────┐
│   Browser           │
│   ws.onmessage      │
│   • Update UI       │
│   • Show progress   │
└─────────────────────┘
```

### Event Types

```
┌────────────────────────────────────────────┐
│         Event Type Hierarchy               │
└────────────────────────────────────────────┘

RealtimeEvent
│
├─ BenchmarkEvents
│  ├─ BenchmarkStarted
│  │  └─ { session_id, dataset, providers, total_tests }
│  │
│  ├─ BenchmarkProgress
│  │  └─ { session_id, completed, total, current_test, progress_percent }
│  │
│  ├─ BenchmarkCompleted
│  │  └─ { session_id, summary, duration }
│  │
│  └─ TestCompleted
│     └─ { session_id, test_id, status, duration_ms }
│
├─ MetricEvents
│  ├─ MetricUpdate
│  │  └─ { metric_name, value, timestamp }
│  │
│  └─ MetricAggregation
│     └─ { metric_name, aggregate, period }
│
├─ AlertEvents
│  ├─ AlertTriggered
│  │  └─ { severity, message, details }
│  │
│  └─ AlertResolved
│     └─ { alert_id, resolved_at }
│
└─ SystemEvents
   ├─ HealthUpdate
   │  └─ { cpu_usage, memory_usage, active_tasks }
   │
   └─ ProviderStatus
      └─ { provider, status, latency_ms }
```

---

## Plugin System

### Plugin Lifecycle

```
┌─────────────────────────────────────────────────────┐
│              Plugin Lifecycle                        │
└─────────────────────────────────────────────────────┘

1. Upload
   ┌──────────────┐
   │  Plugin File │  (custom-evaluator.wasm)
   └──────┬───────┘
          │
          ▼
   ┌──────────────────┐
   │  Validation      │
   │  • Check WASM    │
   │  • Verify manifest│
   │  • Security scan │
   └──────┬───────────┘
          │
          ▼
   ┌──────────────────┐
   │  Storage         │
   │  • Save to disk  │
   │  • Update DB     │
   └──────┬───────────┘

2. Load
          │
          ▼
   ┌──────────────────┐
   │  WASM Loader     │
   │  • Load bytes    │
   │  • Compile       │
   │  • Instantiate   │
   └──────┬───────────┘
          │
          ▼
   ┌──────────────────┐
   │  Initialize      │
   │  • Call init()   │
   │  • Pass config   │
   │  • Validate      │
   └──────┬───────────┘
          │
          ▼
   ┌──────────────────┐
   │  Register        │
   │  • Add to registry│
   │  • Enable plugin │
   └──────┬───────────┘

3. Execute
          │
          ▼
   ┌──────────────────┐
   │  Receive Request │
   │  • Evaluation    │
   │  • Context       │
   └──────┬───────────┘
          │
          ▼
   ┌──────────────────┐
   │  Sandbox         │
   │  • Memory limit  │
   │  • Time limit    │
   │  • No host access│
   └──────┬───────────┘
          │
          ▼
   ┌──────────────────┐
   │  Execute         │
   │  • Run plugin    │
   │  • Collect result│
   └──────┬───────────┘
          │
          ▼
   ┌──────────────────┐
   │  Return Result   │
   │  • Metrics       │
   │  • Logs          │
   └──────────────────┘

4. Unload
          │
          ▼
   ┌──────────────────┐
   │  Cleanup         │
   │  • Call cleanup()│
   │  • Free memory   │
   └──────┬───────────┘
          │
          ▼
   ┌──────────────────┐
   │  Unregister      │
   │  • Remove from   │
   │    registry      │
   └──────────────────┘
```

### Plugin Sandbox

```
┌─────────────────────────────────────────────────────┐
│              WASM Plugin Sandbox                     │
└─────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────┐
│  Host System (LLM Test Bench)                       │
│                                                      │
│  ┌────────────────────────────────────────────┐    │
│  │         Plugin Host                        │    │
│  │                                             │    │
│  │  Allowed:                                   │    │
│  │  • Call LLM providers (via host function)  │    │
│  │  • Log messages (via host function)        │    │
│  │  • Return metrics                          │    │
│  │                                             │    │
│  │  ┌──────────────────────────────────────┐ │    │
│  │  │       WASM Sandbox                   │ │    │
│  │  │                                       │ │    │
│  │  │  ┌─────────────────────────────────┐│ │    │
│  │  │  │    Plugin Code                  ││ │    │
│  │  │  │    (custom-evaluator.wasm)      ││ │    │
│  │  │  │                                  ││ │    │
│  │  │  │  Restrictions:                   ││ │    │
│  │  │  │  • No file system access         ││ │    │
│  │  │  │  • No network access             ││ │    │
│  │  │  │  • No system calls               ││ │    │
│  │  │  │  • Limited memory (100 MB)       ││ │    │
│  │  │  │  • Execution timeout (30s)       ││ │    │
│  │  │  └─────────────────────────────────┘│ │    │
│  │  │                                       │ │    │
│  │  └───────────────────────────────────────┘ │    │
│  │                                             │    │
│  └─────────────────────────────────────────────┘    │
│                                                      │
└──────────────────────────────────────────────────────┘
```

---

## Deployment Architectures

### Single Server (Development/Small Scale)

```
┌──────────────────────────────────────────────────┐
│              Single Server                        │
│              (DigitalOcean Droplet, EC2)         │
│                                                   │
│  ┌─────────────────────────────────────────┐    │
│  │     Docker Compose                      │    │
│  │                                          │    │
│  │  ┌────────────┐    ┌────────────┐      │    │
│  │  │ API Server │    │ PostgreSQL │      │    │
│  │  │   :8080    │───>│   :5432    │      │    │
│  │  └────────────┘    └────────────┘      │    │
│  │        │                                 │    │
│  │        ▼                                 │    │
│  │  ┌────────────┐    ┌────────────┐      │    │
│  │  │   Redis    │    │ Prometheus │      │    │
│  │  │   :6379    │    │   :9090    │      │    │
│  │  └────────────┘    └────────────┘      │    │
│  │                           │              │    │
│  │                           ▼              │    │
│  │                    ┌────────────┐       │    │
│  │                    │  Grafana   │       │    │
│  │                    │   :3000    │       │    │
│  │                    └────────────┘       │    │
│  └──────────────────────────────────────────┘    │
│                                                   │
│  Specs:                                          │
│  • 4 vCPU, 8 GB RAM                              │
│  • 80 GB SSD                                     │
│  • Cost: ~$40-80/month                           │
└──────────────────────────────────────────────────┘
```

### Multi-Server (Production)

```
┌────────────────────────────────────────────────────┐
│                 Load Balancer                       │
│                (Nginx, HAProxy)                     │
│                    :80, :443                        │
└──────────────────┬─────────────────────────────────┘
                   │
       ┌───────────┼───────────┬───────────┐
       │           │           │           │
       ▼           ▼           ▼           ▼
┌─────────────┐ ┌─────────────┐ ┌─────────────┐
│ API Server  │ │ API Server  │ │ API Server  │
│    #1       │ │    #2       │ │    #3       │
│  :8080      │ │  :8080      │ │  :8080      │
└──────┬──────┘ └──────┬──────┘ └──────┬──────┘
       │               │               │
       └───────────────┼───────────────┘
                       │
           ┌───────────┼───────────┐
           │           │           │
           ▼           ▼           ▼
    ┌────────────┐ ┌────────┐ ┌────────────┐
    │PostgreSQL  │ │ Redis  │ │    S3      │
    │   (RDS)    │ │(Elastic│ │  (Object   │
    │  Primary   │ │ Cache) │ │  Storage)  │
    └──────┬─────┘ └────────┘ └────────────┘
           │
           ▼
    ┌────────────┐
    │PostgreSQL  │
    │   (RDS)    │
    │  Replica   │
    └────────────┘

    ┌──────────────────────────────────┐
    │      Monitoring                  │
    │  ┌────────────┐  ┌────────────┐ │
    │  │Prometheus  │  │  Grafana   │ │
    │  └────────────┘  └────────────┘ │
    └──────────────────────────────────┘

Specs:
• API Servers: 3 × (2 vCPU, 4 GB RAM)
• PostgreSQL: db.t3.medium (2 vCPU, 4 GB)
• Redis: cache.t3.small (2 vCPU, 1.5 GB)
• Load Balancer: ALB or similar
• Cost: ~$300-500/month
```

### Kubernetes (Enterprise Scale)

```
┌─────────────────────────────────────────────────────────────┐
│                    Kubernetes Cluster                        │
│                                                              │
│  ┌───────────────────────────────────────────────────────┐  │
│  │                    Ingress Controller                   │  │
│  │                     (Nginx/Traefik)                     │  │
│  └───────────────────────┬─────────────────────────────────┘  │
│                          │                                    │
│          ┌───────────────┼───────────────┐                   │
│          │               │               │                   │
│          ▼               ▼               ▼                   │
│   ┌────────────┐  ┌────────────┐  ┌────────────┐           │
│   │API Server  │  │API Server  │  │API Server  │           │
│   │   Pod 1    │  │   Pod 2    │  │   Pod 3    │           │
│   └────────────┘  └────────────┘  └────────────┘           │
│   Auto-scaling: 3-10 pods                                    │
│                                                              │
│   ┌──────────────────────────────────────────────┐          │
│   │            Coordinator                       │          │
│   │               (StatefulSet)                  │          │
│   │               1 replica                      │          │
│   └───────────────────┬──────────────────────────┘          │
│                       │                                      │
│       ┌───────────────┼───────────────┬──────────┐          │
│       │               │               │          │          │
│       ▼               ▼               ▼          ▼          │
│  ┌─────────┐    ┌─────────┐    ┌─────────┐  ┌─────────┐   │
│  │Worker 1 │    │Worker 2 │    │Worker 3 │..│Worker N │   │
│  │  Pod    │    │  Pod    │    │  Pod    │  │  Pod    │   │
│  └─────────┘    └─────────┘    └─────────┘  └─────────┘   │
│  Auto-scaling: 3-20 pods based on CPU/Memory                │
│                                                              │
│  ┌──────────────────────────────────────────────┐          │
│  │           Stateful Services                  │          │
│  │  ┌─────────────┐  ┌─────────────┐           │          │
│  │  │ PostgreSQL  │  │   Redis     │           │          │
│  │  │ StatefulSet │  │ StatefulSet │           │          │
│  │  │  (or RDS)   │  │(or ElastiC.)│           │          │
│  │  └─────────────┘  └─────────────┘           │          │
│  └──────────────────────────────────────────────┘          │
│                                                              │
│  ┌──────────────────────────────────────────────┐          │
│  │         Monitoring Stack                     │          │
│  │  ┌──────────────┐  ┌──────────────┐         │          │
│  │  │ Prometheus   │  │   Grafana    │         │          │
│  │  │  Operator    │  │  Deployment  │         │          │
│  │  └──────────────┘  └──────────────┘         │          │
│  └──────────────────────────────────────────────┘          │
│                                                              │
└──────────────────────────────────────────────────────────────┘

Resources:
• API Server pods: 0.5-2 CPU, 512Mi-2Gi RAM each
• Worker pods: 2-4 CPU, 4-8Gi RAM each (can have GPU)
• Coordinator: 1-2 CPU, 2-4Gi RAM
• PostgreSQL: 4 CPU, 16Gi RAM (or managed RDS)
• Redis: 2 CPU, 4Gi RAM (or managed ElastiCache)

Cost: $1000-5000+/month depending on scale
```

---

## Component Interaction Matrix

```
┌─────────────────────────────────────────────────────────────────┐
│             Component Interaction Matrix                         │
└─────────────────────────────────────────────────────────────────┘

                 API    Core   Providers  Storage  Cache  Monitor
              ┌────────┬──────┬──────────┬────────┬──────┬────────┐
API Server    │   -    │  ✓   │    ✓     │   ✓    │  ✓   │   ✓    │
Core          │   ✓    │  -   │    ✓     │   ✓    │  ✓   │   ✓    │
Providers     │   ✓    │  ✓   │    -     │   -    │  ✓   │   ✓    │
Storage       │   ✓    │  ✓   │    -     │   -    │  -   │   ✓    │
Cache         │   ✓    │  ✓   │    ✓     │   -    │  -   │   ✓    │
Monitoring    │   ✓    │  ✓   │    ✓     │   ✓    │  ✓   │   -    │
Plugins       │   ✓    │  ✓   │    ✓     │   -    │  ✓   │   ✓    │
Integrations  │   ✓    │  ✓   │    -     │   ✓    │  -   │   -    │
└──────────────────────────────────────────────────────────────────┘

✓ = Direct interaction
- = No direct interaction
```

---

## Summary

This architecture provides:

1. **Scalability**: From single server to K8s cluster
2. **Modularity**: Clear separation of concerns
3. **Extensibility**: Plugin system, integration adapters
4. **Performance**: Async, caching, distributed execution
5. **Observability**: Prometheus, OpenTelemetry, real-time events
6. **Security**: RBAC, JWT, OAuth2, plugin sandboxing
7. **Production-Ready**: Health checks, monitoring, deployment options

---

**Document Version:** 1.0
**Last Updated:** November 4, 2025
**Maintained By:** Technical Architecture Team

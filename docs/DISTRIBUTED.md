# Distributed Architecture Documentation

The LLM Test Bench distributed architecture enables horizontal scaling through a coordinator-worker pattern, allowing you to distribute benchmark and evaluation workloads across multiple machines.

## Table of Contents

1. [Overview](#overview)
2. [Architecture](#architecture)
3. [Components](#components)
4. [Getting Started](#getting-started)
5. [Configuration](#configuration)
6. [Job Management](#job-management)
7. [Fault Tolerance](#fault-tolerance)
8. [Monitoring](#monitoring)
9. [Examples](#examples)
10. [Production Deployment](#production-deployment)

## Overview

The distributed system consists of:
- **Coordinator**: Central node that manages job distribution and worker coordination
- **Workers**: Execution nodes that process tasks
- **gRPC Protocol**: Communication layer between coordinator and workers

### Key Features

- ✅ **Horizontal Scaling**: Add workers to increase capacity
- ✅ **Fault Tolerance**: Automatic task retry on worker failure
- ✅ **Load Balancing**: Work stealing and fair distribution
- ✅ **Health Monitoring**: Continuous worker health checks
- ✅ **Priority Scheduling**: Priority-based job queue
- ✅ **Task Timeout**: Configurable timeouts for tasks
- ✅ **Graceful Shutdown**: Clean worker deregistration

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      Coordinator                            │
│  ┌────────────────┐  ┌────────────────┐  ┌────────────────┐ │
│  │  Job Queue     │  │  Worker Pool   │  │ Health Monitor │ │
│  │  - Pending     │  │  - Active      │  │  - Heartbeat   │ │
│  │  - Running     │  │  - Idle        │  │  - Status      │ │
│  │  - Completed   │  │  - Failed      │  │  - Metrics     │ │
│  └────────────────┘  └────────────────┘  └────────────────┘ │
│           │                   │                   │          │
│           └───────────────────┼───────────────────┘          │
│                               │                              │
│                          gRPC API                            │
└───────────────────────────────┼──────────────────────────────┘
                                │
         ┌──────────────────────┼──────────────────────┐
         │                      │                      │
         ▼                      ▼                      ▼
 ┌───────────────┐      ┌───────────────┐     ┌───────────────┐
 │   Worker 1    │      │   Worker 2    │     │   Worker N    │
 │               │      │               │     │               │
 │  - Executor   │      │  - Executor   │     │  - Executor   │
 │  - Cache      │      │  - Cache      │     │  - Cache      │
 │  - Metrics    │      │  - Metrics    │     │  - Metrics    │
 └───────────────┘      └───────────────┘     └───────────────┘
```

## Components

### Coordinator

The coordinator is the central control node that:
- Accepts job submissions
- Maintains worker registry
- Distributes tasks to workers
- Monitors worker health
- Collects and aggregates results
- Handles task retries

### Worker

Workers are execution nodes that:
- Register with the coordinator
- Pull tasks from the coordinator
- Execute tasks using configured executors
- Report results back to coordinator
- Send regular heartbeats

### Job Queue

Priority-based queue that:
- Stores pending jobs
- Orders by priority (higher first)
- FIFO within same priority
- Supports job cancellation
- Tracks job status

### Health Monitor

Continuous monitoring system that:
- Checks worker heartbeats
- Detects failed workers
- Removes unhealthy workers
- Reports cluster health

## Getting Started

### Starting a Coordinator

```rust
use llm_test_bench_core::distributed::{Coordinator, CoordinatorConfig};
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = CoordinatorConfig::default();
    let coordinator = Arc::new(Coordinator::new(config));

    coordinator.start().await?;
    Ok(())
}
```

### Starting a Worker

```rust
use llm_test_bench_core::distributed::{Worker, WorkerConfig};
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = WorkerConfig::builder()
        .worker_id("worker-1")
        .coordinator_address("http://localhost:50051")
        .capacity(4)
        .build();

    let worker = Arc::new(Worker::new(config));
    worker.start().await?;
    Ok(())
}
```

### Submitting a Job

```rust
use llm_test_bench_core::distributed::{Coordinator, JobRequest};

async fn submit_benchmark(coordinator: &Coordinator) -> anyhow::Result<String> {
    let job = JobRequest::builder()
        .job_type("benchmark")
        .payload(serde_json::json!({
            "provider": "openai",
            "model": "gpt-4",
            "iterations": 100
        }))
        .priority(10)
        .timeout_seconds(600)
        .max_retries(3)
        .build();

    let response = coordinator.submit_job(job);
    Ok(response.job_id)
}
```

## Configuration

### Coordinator Configuration

```rust
use llm_test_bench_core::distributed::CoordinatorConfig;
use std::net::SocketAddr;

let config = CoordinatorConfig::builder()
    .bind_address("0.0.0.0:50051".parse::<SocketAddr>()?)
    .heartbeat_interval(5)           // 5 seconds
    .health_check_timeout(10)        // 10 seconds
    .unhealthy_threshold(30)         // 30 seconds
    .max_retries(3)
    .build();
```

#### Configuration Options

| Option | Default | Description |
|--------|---------|-------------|
| `bind_address` | `0.0.0.0:50051` | Coordinator listening address |
| `heartbeat_interval` | `5` seconds | Expected heartbeat interval |
| `health_check_timeout` | `10` seconds | Health check timeout |
| `unhealthy_threshold` | `30` seconds | Time before worker marked unhealthy |
| `max_retries` | `3` | Max retries for failed tasks |
| `max_completed_jobs` | `1000` | Max completed jobs to keep in memory |

### Worker Configuration

```rust
use llm_test_bench_core::distributed::{WorkerConfig, WorkerCapabilities};

let config = WorkerConfig::builder()
    .worker_id("worker-1")
    .coordinator_address("http://coordinator:50051")
    .capacity(8)
    .tags(vec!["gpu".to_string(), "benchmark".to_string()])
    .build();
```

#### Configuration Options

| Option | Default | Description |
|--------|---------|-------------|
| `worker_id` | Auto-generated UUID | Unique worker identifier |
| `coordinator_address` | `http://localhost:50051` | Coordinator gRPC address |
| `bind_address` | `0.0.0.0:0` | Worker listening address |
| `capacity` | CPU count | Max concurrent tasks |
| `heartbeat_interval` | `5` seconds | Heartbeat frequency |
| `task_pull_interval` | `1` second | Task pulling frequency |
| `tags` | `[]` | Worker tags for filtering |

## Job Management

### Job Lifecycle

```
Pending → Running → Completed
                 ↘ Failed
                 ↘ Cancelled
```

### Job Priority

Jobs are scheduled by priority (higher values first):

```rust
// High priority job
let urgent_job = JobRequest::builder()
    .job_type("benchmark")
    .priority(100)
    .build();

// Normal priority job
let normal_job = JobRequest::builder()
    .job_type("evaluation")
    .priority(50)
    .build();

// Low priority job
let background_job = JobRequest::builder()
    .job_type("analysis")
    .priority(1)
    .build();
```

### Job Status Tracking

```rust
use llm_test_bench_core::distributed::protocol::JobStatusRequest;

let status = coordinator.get_job_status(JobStatusRequest {
    job_id: "job-123".to_string(),
});

if let Some(status) = status {
    println!("Job: {}", status.job_id);
    println!("Status: {}", status.status);
    println!("Progress: {:.1}%", status.progress * 100.0);

    if let Some(result) = status.result {
        println!("Result: {:?}", result);
    }
}
```

### Job Cancellation

```rust
use llm_test_bench_core::distributed::protocol::CancelJobRequest;

let response = coordinator.cancel_job(CancelJobRequest {
    job_id: "job-123".to_string(),
    reason: "User requested cancellation".to_string(),
});

if response.success {
    println!("Job cancelled successfully");
}
```

## Fault Tolerance

### Worker Failure Detection

The coordinator detects worker failures through:
1. **Heartbeat Monitoring**: Workers send heartbeats every 5 seconds
2. **Health Checks**: Coordinator checks worker health every 5 seconds
3. **Timeout Detection**: Workers missing heartbeats are marked unhealthy

### Automatic Task Retry

Failed tasks are automatically retried up to `max_retries` times:

```rust
let job = JobRequest::builder()
    .job_type("benchmark")
    .max_retries(3)  // Retry up to 3 times
    .build();
```

### Graceful Shutdown

Workers gracefully shutdown by:
1. Stopping task pulls
2. Completing running tasks
3. Deregistering from coordinator
4. Exiting cleanly

```rust
// Shutdown triggered by Ctrl+C
// Worker will complete current tasks before exiting
```

## Monitoring

### Cluster Metrics

```rust
let metrics = coordinator.metrics();

println!("Cluster Metrics:");
println!("  Total Workers: {}", metrics.total_workers);
println!("  Active Workers: {}", metrics.active_workers);
println!("  Cluster Load: {:.1}%", metrics.cluster_load * 100.0);
println!("  Total Capacity: {}", metrics.total_capacity);
println!("  Used Capacity: {}", metrics.used_capacity);
println!("  Total Jobs: {}", metrics.total_jobs);
println!("  Completed Jobs: {}", metrics.completed_jobs);
println!("  Failed Jobs: {}", metrics.failed_jobs);
```

### Worker Statistics

```rust
let workers = coordinator.list_workers(ListWorkersRequest {
    status_filter: Some("idle".to_string()),
    tag_filter: vec![],
});

for worker in workers.workers {
    println!("Worker: {}", worker.worker_id);
    println!("  Status: {}", worker.status);
    println!("  Load: {:.1}%", worker.load * 100.0);
    println!("  Tasks: {}/{}", worker.current_tasks, worker.capacity);
}
```

### Health Status

```rust
use llm_test_bench_core::distributed::health::HealthStatus;

let health = coordinator.health_monitor.cluster_health();

println!("Cluster Health: {:?}", health.overall_status);
println!("  Healthy Workers: {}", health.healthy_workers);
println!("  Degraded Workers: {}", health.degraded_workers);
println!("  Unhealthy Workers: {}", health.unhealthy_workers);
```

## Examples

### Example 1: Simple Benchmark Distribution

```rust
use llm_test_bench_core::distributed::*;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Start coordinator
    let coordinator = Arc::new(Coordinator::new(CoordinatorConfig::default()));

    tokio::spawn({
        let coord = coordinator.clone();
        async move {
            coord.start().await
        }
    });

    // Submit 100 benchmark jobs
    for i in 0..100 {
        let job = JobRequest::builder()
            .job_type("benchmark")
            .payload(serde_json::json!({
                "provider": "openai",
                "model": "gpt-4",
                "prompt": format!("Test prompt {}", i)
            }))
            .priority(10)
            .build();

        coordinator.submit_job(job);
    }

    // Wait for completion
    loop {
        let stats = coordinator.get_cluster_stats();
        if stats.running_jobs == 0 && stats.pending_jobs == 0 {
            break;
        }
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }

    println!("All jobs completed!");
    Ok(())
}
```

### Example 2: Tagged Workers

```rust
// GPU worker
let gpu_worker = Worker::new(WorkerConfig::builder()
    .worker_id("gpu-worker-1")
    .coordinator_address("http://coordinator:50051")
    .tags(vec!["gpu".to_string()])
    .build());

// CPU worker
let cpu_worker = Worker::new(WorkerConfig::builder()
    .worker_id("cpu-worker-1")
    .coordinator_address("http://coordinator:50051")
    .tags(vec!["cpu".to_string()])
    .build());

// Submit job requiring GPU
let gpu_job = JobRequest::builder()
    .job_type("multimodal")
    .required_tags(vec!["gpu".to_string()])
    .build();
```

### Example 3: Custom Task Executor

```rust
use llm_test_bench_core::distributed::{Worker, WorkerConfig};
use llm_test_bench_core::distributed::worker::TaskExecutor;
use async_trait::async_trait;

struct BenchmarkExecutor;

#[async_trait]
impl TaskExecutor for BenchmarkExecutor {
    async fn execute(&self, task: TaskRequest) -> anyhow::Result<TaskResponse> {
        // Parse task payload
        let provider = task.payload["provider"].as_str().unwrap();
        let model = task.payload["model"].as_str().unwrap();

        // Run benchmark
        let result = run_benchmark(provider, model).await?;

        Ok(TaskResponse {
            task_id: task.task_id,
            success: true,
            result: Some(serde_json::to_value(result)?),
            error: None,
            execution_time_ms: 1000,
            completed_at: chrono::Utc::now(),
        })
    }
}

// Create worker with custom executor
let worker = Worker::with_executor(
    WorkerConfig::default(),
    Arc::new(BenchmarkExecutor),
);
```

## Production Deployment

### Docker Deployment

#### Coordinator Dockerfile

```dockerfile
FROM rust:1.75 as builder

WORKDIR /app
COPY . .
RUN cargo build --release --bin coordinator

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/coordinator /usr/local/bin/

EXPOSE 50051
CMD ["coordinator"]
```

#### Worker Dockerfile

```dockerfile
FROM rust:1.75 as builder

WORKDIR /app
COPY . .
RUN cargo build --release --bin worker

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/worker /usr/local/bin/

CMD ["worker"]
```

#### Docker Compose

```yaml
version: '3.8'

services:
  coordinator:
    build:
      context: .
      dockerfile: Dockerfile.coordinator
    ports:
      - "50051:50051"
    environment:
      - RUST_LOG=info
    networks:
      - llm-bench

  worker-1:
    build:
      context: .
      dockerfile: Dockerfile.worker
    environment:
      - COORDINATOR_ADDRESS=http://coordinator:50051
      - WORKER_ID=worker-1
      - CAPACITY=8
    depends_on:
      - coordinator
    networks:
      - llm-bench

  worker-2:
    build:
      context: .
      dockerfile: Dockerfile.worker
    environment:
      - COORDINATOR_ADDRESS=http://coordinator:50051
      - WORKER_ID=worker-2
      - CAPACITY=8
    depends_on:
      - coordinator
    networks:
      - llm-bench

networks:
  llm-bench:
    driver: bridge
```

### Kubernetes Deployment

#### Coordinator Deployment

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: coordinator
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
        image: llm-bench-coordinator:latest
        ports:
        - containerPort: 50051
        env:
        - name: RUST_LOG
          value: "info"
---
apiVersion: v1
kind: Service
metadata:
  name: coordinator
spec:
  selector:
    app: coordinator
  ports:
  - port: 50051
    targetPort: 50051
```

#### Worker Deployment

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: workers
spec:
  replicas: 10
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
        image: llm-bench-worker:latest
        env:
        - name: COORDINATOR_ADDRESS
          value: "http://coordinator:50051"
        - name: CAPACITY
          value: "8"
        resources:
          requests:
            cpu: "2"
            memory: "4Gi"
          limits:
            cpu: "4"
            memory: "8Gi"
```

### Scaling Workers

```bash
# Scale to 20 workers
kubectl scale deployment workers --replicas=20

# Auto-scale based on CPU
kubectl autoscale deployment workers \
  --min=5 --max=50 \
  --cpu-percent=80
```

### Monitoring Integration

#### Prometheus Metrics

The coordinator exposes Prometheus metrics:

```yaml
# prometheus.yml
scrape_configs:
  - job_name: 'coordinator'
    static_configs:
      - targets: ['coordinator:9090']
```

#### Grafana Dashboard

Import the provided Grafana dashboard for visualization:
- Cluster health
- Worker status
- Job throughput
- Task latency
- Error rates

## Performance Considerations

### Optimal Worker Capacity

```rust
// Set capacity based on workload type

// CPU-intensive tasks
let cpu_config = WorkerConfig::builder()
    .capacity(num_cpus::get())  // 1 task per core
    .build();

// I/O-intensive tasks
let io_config = WorkerConfig::builder()
    .capacity(num_cpus::get() * 4)  // 4x oversubscription
    .build();

// GPU tasks
let gpu_config = WorkerConfig::builder()
    .capacity(num_gpus::get())  // 1 task per GPU
    .build();
```

### Load Balancing

Workers are selected based on:
1. Availability (has capacity)
2. Load (lowest current load)
3. Tags (matches required tags)

```rust
// Coordinator automatically selects least loaded worker
let worker = cluster.get_least_loaded_worker_with_tags(&required_tags);
```

### Network Optimization

- Use gRPC for efficient binary serialization
- Enable HTTP/2 multiplexing
- Compress large payloads
- Batch heartbeats when possible

## Troubleshooting

### Workers Not Connecting

```bash
# Check coordinator is running
curl http://localhost:50051/health

# Check worker logs
docker logs worker-1

# Verify network connectivity
ping coordinator
```

### Tasks Not Executing

```bash
# Check worker capacity
# Workers at max capacity won't pull new tasks

# Check job requirements
# Jobs with incompatible tags won't be assigned

# Check coordinator logs
docker logs coordinator
```

### High Failure Rate

```bash
# Increase timeouts
let job = JobRequest::builder()
    .timeout_seconds(1200)  # 20 minutes
    .max_retries(5)
    .build();

# Check worker resources
kubectl top pods

# Review error logs
kubectl logs -l app=worker --tail=100
```

## Security

### TLS/mTLS

Enable TLS for coordinator-worker communication:

```rust
// Configure TLS (future enhancement)
let config = CoordinatorConfig::builder()
    .enable_tls(true)
    .cert_path("/path/to/cert.pem")
    .key_path("/path/to/key.pem")
    .build();
```

### Authentication

Authenticate workers with API keys:

```rust
let config = WorkerConfig::builder()
    .api_key(std::env::var("WORKER_API_KEY")?)
    .build();
```

## Best Practices

1. **Resource Limits**: Set appropriate resource limits for workers
2. **Monitoring**: Enable Prometheus metrics and Grafana dashboards
3. **Retries**: Configure reasonable retry limits (3-5)
4. **Timeouts**: Set timeouts based on expected task duration
5. **Tags**: Use tags to route specialized tasks to capable workers
6. **Graceful Shutdown**: Always shut down gracefully to avoid lost work
7. **Health Checks**: Monitor cluster health continuously
8. **Auto-scaling**: Use Kubernetes HPA for dynamic scaling

## Future Enhancements

- [ ] Coordinator high availability (leader election)
- [ ] Result caching with Redis
- [ ] Task dependencies (DAG execution)
- [ ] Resource reservations
- [ ] Advanced scheduling policies
- [ ] Job templates
- [ ] Workflow orchestration

## Support

For issues, questions, or contributions:
- GitHub: https://github.com/yourusername/llm-test-bench
- Documentation: https://docs.llm-test-bench.dev

## License

Licensed under Apache 2.0 or MIT license.

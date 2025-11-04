# Database Backend Documentation

The LLM Test Bench uses PostgreSQL as its database backend for persistent storage of benchmarks, evaluations, jobs, workers, users, and audit logs.

## Table of Contents

1. [Overview](#overview)
2. [Architecture](#architecture)
3. [Schema](#schema)
4. [Configuration](#configuration)
5. [Migrations](#migrations)
6. [Repositories](#repositories)
7. [Usage Examples](#usage-examples)
8. [Production Deployment](#production-deployment)
9. [Performance](#performance)
10. [Backup and Recovery](#backup-and-recovery)

## Overview

The database backend provides:
- **Persistent Storage**: All benchmark results, jobs, and system state
- **ACID Compliance**: Transaction support for data integrity
- **Connection Pooling**: Efficient connection management
- **Type Safety**: Compile-time query validation with sqlx
- **Migrations**: Automatic schema versioning
- **Audit Logging**: Complete change history

### Key Features

- ✅ **PostgreSQL 12+**: Industry-standard RDBMS
- ✅ **Connection Pooling**: sqlx with configurable pool size
- ✅ **Repository Pattern**: Clean data access layer
- ✅ **Automatic Migrations**: Version-controlled schema
- ✅ **Audit Trails**: Complete change history
- ✅ **Query Optimization**: Indexes on all query paths
- ✅ **Type Safety**: Compile-time query validation

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                   Application Layer                         │
│  ┌────────────┐  ┌────────────┐  ┌────────────┐           │
│  │    API     │  │ Coordinator│  │  Workers   │           │
│  └─────┬──────┘  └─────┬──────┘  └─────┬──────┘           │
│        │               │               │                   │
└────────┼───────────────┼───────────────┼───────────────────┘
         │               │               │
┌────────┼───────────────┼───────────────┼───────────────────┐
│        │       Repository Layer        │                   │
│        ▼               ▼               ▼                   │
│  ┌─────────────────────────────────────────────────────┐  │
│  │  BenchmarkRepo │ JobRepo │ WorkerRepo │ UserRepo   │  │
│  └─────────────────────────────────────────────────────┘  │
│                          │                                 │
│                          ▼                                 │
│  ┌───────────────────────────────────────────────────┐    │
│  │              Connection Pool (sqlx)               │    │
│  └───────────────────────────────────────────────────┘    │
│                          │                                 │
└──────────────────────────┼─────────────────────────────────┘
                           │
┌──────────────────────────┼─────────────────────────────────┐
│                          ▼                                 │
│  ┌───────────────────────────────────────────────────┐    │
│  │              PostgreSQL Database                   │    │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐          │    │
│  │  │Benchmarks│ │   Jobs   │ │ Workers  │          │    │
│  │  └──────────┘ └──────────┘ └──────────┘          │    │
│  └───────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────┘
```

## Schema

### Benchmarks Table

Stores benchmark configurations and results.

```sql
CREATE TABLE benchmarks (
    id UUID PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    provider VARCHAR(100) NOT NULL,
    model VARCHAR(255) NOT NULL,
    dataset VARCHAR(255) NOT NULL,
    status VARCHAR(50) NOT NULL,
    total_iterations INTEGER,
    completed_iterations INTEGER,
    failed_iterations INTEGER,
    results JSONB,
    metrics JSONB,
    error TEXT,
    started_at TIMESTAMP WITH TIME ZONE,
    completed_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE,
    updated_at TIMESTAMP WITH TIME ZONE,
    created_by UUID
);
```

**Indexes**:
- `idx_benchmarks_status` - Filter by status
- `idx_benchmarks_provider` - Filter by provider
- `idx_benchmarks_created_at` - Sort by creation date

### Evaluations Table

Stores individual evaluation results.

```sql
CREATE TABLE evaluations (
    id UUID PRIMARY KEY,
    benchmark_id UUID REFERENCES benchmarks(id),
    provider VARCHAR(100) NOT NULL,
    model VARCHAR(255) NOT NULL,
    input TEXT NOT NULL,
    output TEXT NOT NULL,
    expected TEXT,
    metrics JSONB NOT NULL,
    score DOUBLE PRECISION NOT NULL,
    metadata JSONB,
    created_at TIMESTAMP WITH TIME ZONE,
    created_by UUID
);
```

### Jobs Table

Distributed job queue.

```sql
CREATE TABLE jobs (
    id UUID PRIMARY KEY,
    job_type VARCHAR(100) NOT NULL,
    payload JSONB NOT NULL,
    priority INTEGER NOT NULL,
    status VARCHAR(50) NOT NULL,
    result JSONB,
    error TEXT,
    retry_count INTEGER,
    max_retries INTEGER,
    timeout_seconds INTEGER,
    assigned_worker_id UUID,
    started_at TIMESTAMP WITH TIME ZONE,
    completed_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE,
    updated_at TIMESTAMP WITH TIME ZONE
);
```

**Indexes**:
- `idx_jobs_status_priority` - Job queue ordering
- `idx_jobs_created_at` - Created date sorting

### Workers Table

Worker registry.

```sql
CREATE TABLE workers (
    id UUID PRIMARY KEY,
    worker_id VARCHAR(255) UNIQUE NOT NULL,
    address VARCHAR(255) NOT NULL,
    status VARCHAR(50) NOT NULL,
    capacity INTEGER NOT NULL,
    current_tasks INTEGER,
    completed_tasks BIGINT,
    failed_tasks BIGINT,
    tags TEXT[],
    metadata JSONB,
    last_heartbeat TIMESTAMP WITH TIME ZONE,
    registered_at TIMESTAMP WITH TIME ZONE,
    updated_at TIMESTAMP WITH TIME ZONE
);
```

### Users Table

User accounts.

```sql
CREATE TABLE users (
    id UUID PRIMARY KEY,
    email VARCHAR(255) UNIQUE NOT NULL,
    username VARCHAR(100) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    role VARCHAR(50) NOT NULL,
    active BOOLEAN,
    email_verified BOOLEAN,
    metadata JSONB,
    last_login TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE,
    updated_at TIMESTAMP WITH TIME ZONE
);
```

### Audit Logs Table

Complete audit trail.

```sql
CREATE TABLE audit_logs (
    id UUID PRIMARY KEY,
    user_id UUID REFERENCES users(id),
    action VARCHAR(100) NOT NULL,
    entity_type VARCHAR(100) NOT NULL,
    entity_id UUID,
    changes JSONB,
    ip_address VARCHAR(45),
    user_agent TEXT,
    created_at TIMESTAMP WITH TIME ZONE
);
```

## Configuration

### Environment Variables

```bash
DATABASE_HOST=localhost
DATABASE_PORT=5432
DATABASE_NAME=llm_test_bench
DATABASE_USER=postgres
DATABASE_PASSWORD=your_password
DATABASE_POOL_SIZE=20
DATABASE_SSL_MODE=prefer
```

### Programmatic Configuration

```rust
use llm_test_bench_core::database::{Database, DatabaseConfig, SslMode};

let config = DatabaseConfig::builder()
    .host("localhost")
    .port(5432)
    .database("llm_test_bench")
    .username("postgres")
    .password("password")
    .pool_size(20)
    .connect_timeout(10)
    .idle_timeout(600)
    .max_lifetime(1800)
    .ssl_mode(SslMode::Require)
    .build();

let db = Database::connect(config).await?;
```

### Connection String

```
postgres://username:password@localhost:5432/llm_test_bench?application_name=llm-test-bench
```

## Migrations

Migrations are managed with sqlx and stored in `migrations/`.

### Running Migrations

```bash
# Using sqlx CLI
sqlx migrate run --database-url postgres://localhost/llm_test_bench

# Programmatically
let db = Database::connect(config).await?;
db.migrate().await?;
```

### Creating Migrations

```bash
sqlx migrate add <migration_name>
```

### Migration Files

Migrations are SQL files with timestamp prefixes:

```
migrations/
  ├── 20240320000001_initial_schema.sql
  ├── 20240320000002_add_indexes.sql
  └── 20240320000003_add_audit_logs.sql
```

## Repositories

### Benchmark Repository

```rust
// Create benchmark
let benchmark = NewBenchmark {
    name: "GPT-4 Test".to_string(),
    provider: "openai".to_string(),
    model: "gpt-4".to_string(),
    dataset: "mmlu".to_string(),
    total_iterations: 100,
    created_by: None,
    ..Default::default()
};

let record = db.benchmarks().create(benchmark).await?;

// Get benchmark
let benchmark = db.benchmarks().get(id).await?;

// Update benchmark
let update = UpdateBenchmark {
    status: Some("completed".to_string()),
    completed_iterations: Some(100),
    ..Default::default()
};

db.benchmarks().update(id, update).await?;

// List recent benchmarks
let benchmarks = db.benchmarks().list_recent(10).await?;

// List by status
let running = db.benchmarks().list_by_status("running", 20).await?;
```

### Job Repository

```rust
// Create job
let job = NewJob {
    job_type: "benchmark".to_string(),
    payload: serde_json::json!({"model": "gpt-4"}),
    priority: 10,
    max_retries: 3,
    timeout_seconds: 600,
};

let record = db.jobs().create(job).await?;

// List pending jobs (priority order)
let pending = db.jobs().list_pending(50).await?;

// Get job statistics
let stats = db.jobs().get_stats().await?;
println!("Pending: {}, Running: {}", stats.pending, stats.running);

// Cleanup old jobs
let deleted = db.jobs().cleanup_old_jobs(30).await?;
```

### Worker Repository

```rust
// Register worker
let worker = NewWorker {
    worker_id: "worker-1".to_string(),
    address: "localhost:50052".to_string(),
    capacity: 4,
    tags: vec!["benchmark".to_string()],
    metadata: None,
};

let record = db.workers().create(worker).await?;

// Update heartbeat
db.workers().update_heartbeat("worker-1").await?;

// List active workers
let active = db.workers().list_active(30).await?;

// Remove stale workers
let removed = db.workers().remove_stale(60).await?;
```

### User Repository

```rust
// Create user
let user = NewUser {
    email: "user@example.com".to_string(),
    username: "user1".to_string(),
    password_hash: hash_password("password123")?,
    role: "user".to_string(),
};

let record = db.users().create(user).await?;

// Get by email
let user = db.users().get_by_email("user@example.com").await?;

// Create API key
let api_key = NewApiKey {
    user_id: user.id,
    key_hash: hash_key(&key)?,
    name: "Production API Key".to_string(),
    expires_at: Some(Utc::now() + Duration::days(30)),
};

db.users().create_api_key(api_key).await?;
```

### Audit Repository

```rust
// Create audit log
let log = NewAuditLog {
    user_id: Some(user_id),
    action: "create".to_string(),
    entity_type: "benchmark".to_string(),
    entity_id: Some(benchmark_id),
    changes: Some(serde_json::json!({"status": "created"})),
    ip_address: Some("192.168.1.1".to_string()),
    user_agent: Some("Mozilla/5.0".to_string()),
};

db.audit().create(log).await?;

// List user activity
let logs = db.audit().list_by_user(user_id, 50).await?;

// List entity history
let history = db.audit().list_by_entity("benchmark", benchmark_id, 100).await?;
```

## Usage Examples

### Basic Setup

```rust
use llm_test_bench_core::database::{Database, DatabaseConfig};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load config from environment
    let config = DatabaseConfig::from_env()?;

    // Connect
    let db = Database::connect(config).await?;

    // Run migrations
    db.migrate().await?;

    // Health check
    let healthy = db.health_check().await?;
    println!("Database healthy: {}", healthy);

    Ok(())
}
```

### Complete Workflow

```rust
// Create a benchmark
let benchmark = db.benchmarks().create(NewBenchmark {
    name: "Performance Test".to_string(),
    provider: "openai".to_string(),
    model: "gpt-4".to_string(),
    dataset: "mmlu".to_string(),
    total_iterations: 100,
    created_by: None,
    description: None,
}).await?;

// Run evaluations
for i in 0..100 {
    let evaluation = NewEvaluation {
        benchmark_id: Some(benchmark.id),
        provider: "openai".to_string(),
        model: "gpt-4".to_string(),
        input: format!("Question {}", i),
        output: format!("Answer {}", i),
        expected: None,
        metrics: serde_json::json!({"accuracy": 0.95}),
        score: 0.95,
        metadata: None,
        created_by: None,
    };

    db.evaluations().create(evaluation).await?;
}

// Update benchmark
db.benchmarks().update(benchmark.id, UpdateBenchmark {
    status: Some("completed".to_string()),
    completed_iterations: Some(100),
    ..Default::default()
}).await?;
```

## Production Deployment

### Docker Compose

```yaml
version: '3.8'

services:
  postgres:
    image: postgres:16
    environment:
      POSTGRES_DB: llm_test_bench
      POSTGRES_USER: postgres
      POSTGRES_PASSWORD: ${DATABASE_PASSWORD}
    ports:
      - "5432:5432"
    volumes:
      - postgres_data:/var/lib/postgresql/data
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U postgres"]
      interval: 10s
      timeout: 5s
      retries: 5

volumes:
  postgres_data:
```

### Kubernetes

```yaml
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: postgres
spec:
  serviceName: postgres
  replicas: 1
  selector:
    matchLabels:
      app: postgres
  template:
    metadata:
      labels:
        app: postgres
    spec:
      containers:
      - name: postgres
        image: postgres:16
        env:
        - name: POSTGRES_DB
          value: llm_test_bench
        - name: POSTGRES_PASSWORD
          valueFrom:
            secretKeyRef:
              name: postgres-secret
              key: password
        ports:
        - containerPort: 5432
        volumeMounts:
        - name: postgres-storage
          mountPath: /var/lib/postgresql/data
  volumeClaimTemplates:
  - metadata:
      name: postgres-storage
    spec:
      accessModes: ["ReadWriteOnce"]
      resources:
        requests:
          storage: 100Gi
```

## Performance

### Connection Pooling

Configure pool size based on workload:

```rust
let config = DatabaseConfig::builder()
    .pool_size(50)  // Adjust based on concurrent load
    .connect_timeout(10)
    .idle_timeout(600)
    .max_lifetime(1800)
    .build();
```

### Query Optimization

All tables have appropriate indexes:

```sql
-- Benchmark queries
CREATE INDEX idx_benchmarks_status ON benchmarks(status);
CREATE INDEX idx_benchmarks_provider ON benchmarks(provider);

-- Job queries (most important for distributed system)
CREATE INDEX idx_jobs_status_priority ON jobs(status, priority DESC, created_at);

-- Worker queries
CREATE INDEX idx_workers_last_heartbeat ON workers(last_heartbeat DESC);
```

### Monitoring

```rust
// Get database statistics
let stats = db.stats().await?;
println!("Database size: {:.2} MB", stats.database_size_mb());
println!("Total benchmarks: {}", stats.benchmark_count);
println!("Total jobs: {}", stats.job_count);
```

## Backup and Recovery

### Backup

```bash
# Full backup
pg_dump llm_test_bench > backup.sql

# Compressed backup
pg_dump llm_test_bench | gzip > backup.sql.gz

# Custom format (recommended)
pg_dump -Fc llm_test_bench > backup.dump
```

### Restore

```bash
# From SQL
psql llm_test_bench < backup.sql

# From custom format
pg_restore -d llm_test_bench backup.dump
```

### Automated Backups

```bash
#!/bin/bash
# backup.sh
DATE=$(date +%Y%m%d_%H%M%S)
pg_dump -Fc llm_test_bench > /backups/llm_test_bench_$DATE.dump

# Keep only last 7 days
find /backups -name "llm_test_bench_*.dump" -mtime +7 -delete
```

## Best Practices

1. **Always use connection pooling**: Don't create connections per request
2. **Use transactions**: Group related operations
3. **Index appropriately**: Add indexes for query patterns
4. **Monitor query performance**: Use EXPLAIN ANALYZE
5. **Regular backups**: Automated daily backups
6. **Clean up old data**: Periodic cleanup of completed jobs/logs
7. **Use prepared statements**: sqlx provides compile-time checking
8. **Connection limits**: Set appropriate pool size for workload

## License

Licensed under Apache 2.0 or MIT license.


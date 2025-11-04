// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Database models.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Benchmark record
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct BenchmarkRecord {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub provider: String,
    pub model: String,
    pub dataset: String,
    pub status: String,
    pub total_iterations: i32,
    pub completed_iterations: i32,
    pub failed_iterations: i32,
    pub results: Option<serde_json::Value>,
    pub metrics: Option<serde_json::Value>,
    pub error: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
}

/// Evaluation record
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EvaluationRecord {
    pub id: Uuid,
    pub benchmark_id: Option<Uuid>,
    pub provider: String,
    pub model: String,
    pub input: String,
    pub output: String,
    pub expected: Option<String>,
    pub metrics: serde_json::Value,
    pub score: f64,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
}

/// Job record
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct JobRecord {
    pub id: Uuid,
    pub job_type: String,
    pub payload: serde_json::Value,
    pub priority: i32,
    pub status: String,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
    pub retry_count: i32,
    pub max_retries: i32,
    pub timeout_seconds: i32,
    pub assigned_worker_id: Option<Uuid>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Worker record
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct WorkerRecord {
    pub id: Uuid,
    pub worker_id: String,
    pub address: String,
    pub status: String,
    pub capacity: i32,
    pub current_tasks: i32,
    pub completed_tasks: i64,
    pub failed_tasks: i64,
    pub tags: Vec<String>,
    pub metadata: Option<serde_json::Value>,
    pub last_heartbeat: DateTime<Utc>,
    pub registered_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// User record
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserRecord {
    pub id: Uuid,
    pub email: String,
    pub username: String,
    pub password_hash: String,
    pub role: String,
    pub active: bool,
    pub email_verified: bool,
    pub metadata: Option<serde_json::Value>,
    pub last_login: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// API key record
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ApiKeyRecord {
    pub id: Uuid,
    pub user_id: Uuid,
    pub key_hash: String,
    pub name: String,
    pub active: bool,
    pub expires_at: Option<DateTime<Utc>>,
    pub last_used: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// Audit log record
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AuditLogRecord {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub action: String,
    pub entity_type: String,
    pub entity_id: Option<Uuid>,
    pub changes: Option<serde_json::Value>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Monitoring event record
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MonitoringEventRecord {
    pub id: Uuid,
    pub event_type: String,
    pub source: String,
    pub data: serde_json::Value,
    pub timestamp: DateTime<Utc>,
}

/// Benchmark insert data
#[derive(Debug, Clone)]
pub struct NewBenchmark {
    pub name: String,
    pub description: Option<String>,
    pub provider: String,
    pub model: String,
    pub dataset: String,
    pub total_iterations: i32,
    pub created_by: Option<Uuid>,
}

/// Evaluation insert data
#[derive(Debug, Clone)]
pub struct NewEvaluation {
    pub benchmark_id: Option<Uuid>,
    pub provider: String,
    pub model: String,
    pub input: String,
    pub output: String,
    pub expected: Option<String>,
    pub metrics: serde_json::Value,
    pub score: f64,
    pub metadata: Option<serde_json::Value>,
    pub created_by: Option<Uuid>,
}

/// Job insert data
#[derive(Debug, Clone)]
pub struct NewJob {
    pub job_type: String,
    pub payload: serde_json::Value,
    pub priority: i32,
    pub max_retries: i32,
    pub timeout_seconds: i32,
}

/// Worker insert data
#[derive(Debug, Clone)]
pub struct NewWorker {
    pub worker_id: String,
    pub address: String,
    pub capacity: i32,
    pub tags: Vec<String>,
    pub metadata: Option<serde_json::Value>,
}

/// User insert data
#[derive(Debug, Clone)]
pub struct NewUser {
    pub email: String,
    pub username: String,
    pub password_hash: String,
    pub role: String,
}

/// API key insert data
#[derive(Debug, Clone)]
pub struct NewApiKey {
    pub user_id: Uuid,
    pub key_hash: String,
    pub name: String,
    pub expires_at: Option<DateTime<Utc>>,
}

/// Audit log insert data
#[derive(Debug, Clone)]
pub struct NewAuditLog {
    pub user_id: Option<Uuid>,
    pub action: String,
    pub entity_type: String,
    pub entity_id: Option<Uuid>,
    pub changes: Option<serde_json::Value>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

/// Benchmark update data
#[derive(Debug, Clone, Default)]
pub struct UpdateBenchmark {
    pub status: Option<String>,
    pub completed_iterations: Option<i32>,
    pub failed_iterations: Option<i32>,
    pub results: Option<serde_json::Value>,
    pub metrics: Option<serde_json::Value>,
    pub error: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
}

/// Job update data
#[derive(Debug, Clone, Default)]
pub struct UpdateJob {
    pub status: Option<String>,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
    pub retry_count: Option<i32>,
    pub assigned_worker_id: Option<Uuid>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
}

/// Worker update data
#[derive(Debug, Clone, Default)]
pub struct UpdateWorker {
    pub status: Option<String>,
    pub current_tasks: Option<i32>,
    pub completed_tasks: Option<i64>,
    pub failed_tasks: Option<i64>,
    pub last_heartbeat: Option<DateTime<Utc>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_benchmark() {
        let benchmark = NewBenchmark {
            name: "Test Benchmark".to_string(),
            description: Some("Test description".to_string()),
            provider: "openai".to_string(),
            model: "gpt-4".to_string(),
            dataset: "mmlu".to_string(),
            total_iterations: 100,
            created_by: None,
        };

        assert_eq!(benchmark.name, "Test Benchmark");
        assert_eq!(benchmark.total_iterations, 100);
    }

    #[test]
    fn test_update_benchmark() {
        let update = UpdateBenchmark {
            status: Some("running".to_string()),
            completed_iterations: Some(50),
            ..Default::default()
        };

        assert_eq!(update.status.as_ref().unwrap(), "running");
        assert_eq!(update.completed_iterations.unwrap(), 50);
    }
}

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Core types for the distributed system.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

/// Node identifier
pub type NodeId = String;

/// Job identifier
pub type JobId = String;

/// Task identifier
pub type TaskId = String;

/// Result type for distributed operations
pub type DistributedResult<T> = Result<T, DistributedError>;

/// Distributed system errors
#[derive(Debug, Clone, Error, Serialize, Deserialize)]
pub enum DistributedError {
    /// Worker not found
    #[error("Worker not found: {0}")]
    WorkerNotFound(NodeId),

    /// Job not found
    #[error("Job not found: {0}")]
    JobNotFound(JobId),

    /// Task not found
    #[error("Task not found: {0}")]
    TaskNotFound(TaskId),

    /// Worker unavailable
    #[error("Worker unavailable: {0}")]
    WorkerUnavailable(NodeId),

    /// Job execution failed
    #[error("Job execution failed: {0}")]
    JobExecutionFailed(String),

    /// Task execution failed
    #[error("Task execution failed: {0}")]
    TaskExecutionFailed(String),

    /// Communication error
    #[error("Communication error: {0}")]
    CommunicationError(String),

    /// Timeout error
    #[error("Timeout error: {0}")]
    TimeoutError(String),

    /// Invalid job
    #[error("Invalid job: {0}")]
    InvalidJob(String),

    /// Invalid task
    #[error("Invalid task: {0}")]
    InvalidTask(String),

    /// Coordinator error
    #[error("Coordinator error: {0}")]
    CoordinatorError(String),

    /// Worker error
    #[error("Worker error: {0}")]
    WorkerError(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Internal error
    #[error("Internal error: {0}")]
    InternalError(String),
}

/// Worker status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkerStatus {
    /// Worker is idle and available
    Idle,
    /// Worker is busy executing tasks
    Busy,
    /// Worker is starting up
    Starting,
    /// Worker is shutting down
    ShuttingDown,
    /// Worker has failed
    Failed,
    /// Worker is offline
    Offline,
}

impl fmt::Display for WorkerStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Idle => write!(f, "idle"),
            Self::Busy => write!(f, "busy"),
            Self::Starting => write!(f, "starting"),
            Self::ShuttingDown => write!(f, "shutting_down"),
            Self::Failed => write!(f, "failed"),
            Self::Offline => write!(f, "offline"),
        }
    }
}

/// Worker information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerInfo {
    /// Worker ID
    pub id: NodeId,
    /// Worker address (host:port)
    pub address: String,
    /// Worker status
    pub status: WorkerStatus,
    /// Worker capacity (max concurrent tasks)
    pub capacity: usize,
    /// Current task count
    pub current_tasks: usize,
    /// Total tasks completed
    pub completed_tasks: u64,
    /// Total tasks failed
    pub failed_tasks: u64,
    /// Worker tags (for filtering)
    pub tags: Vec<String>,
    /// Worker metadata
    pub metadata: std::collections::HashMap<String, String>,
    /// Last heartbeat time
    pub last_heartbeat: DateTime<Utc>,
    /// Registration time
    pub registered_at: DateTime<Utc>,
}

impl WorkerInfo {
    /// Check if worker is available for tasks
    pub fn is_available(&self) -> bool {
        self.status == WorkerStatus::Idle && self.current_tasks < self.capacity
    }

    /// Get worker load (0.0 to 1.0)
    pub fn load(&self) -> f64 {
        if self.capacity == 0 {
            1.0
        } else {
            self.current_tasks as f64 / self.capacity as f64
        }
    }

    /// Check if worker is healthy
    pub fn is_healthy(&self, timeout_seconds: u64) -> bool {
        let now = Utc::now();
        let elapsed = now
            .signed_duration_since(self.last_heartbeat)
            .num_seconds() as u64;

        elapsed < timeout_seconds
            && (self.status == WorkerStatus::Idle || self.status == WorkerStatus::Busy)
    }
}

/// Worker capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerCapabilities {
    /// Supported job types
    pub job_types: Vec<String>,
    /// Supported providers
    pub providers: Vec<String>,
    /// Available memory (bytes)
    pub memory_bytes: u64,
    /// Available CPU cores
    pub cpu_cores: usize,
    /// GPU availability
    pub has_gpu: bool,
    /// Custom capabilities
    pub custom: std::collections::HashMap<String, serde_json::Value>,
}

impl Default for WorkerCapabilities {
    fn default() -> Self {
        Self {
            job_types: vec![
                "benchmark".to_string(),
                "evaluation".to_string(),
                "completion".to_string(),
            ],
            providers: vec![],
            memory_bytes: 8 * 1024 * 1024 * 1024, // 8GB
            cpu_cores: num_cpus::get(),
            has_gpu: false,
            custom: std::collections::HashMap::new(),
        }
    }
}

/// Coordinator statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoordinatorStats {
    /// Total workers registered
    pub total_workers: usize,
    /// Active workers
    pub active_workers: usize,
    /// Total jobs submitted
    pub total_jobs: u64,
    /// Jobs pending
    pub pending_jobs: usize,
    /// Jobs running
    pub running_jobs: usize,
    /// Jobs completed
    pub completed_jobs: u64,
    /// Jobs failed
    pub failed_jobs: u64,
    /// Average job duration (seconds)
    pub avg_job_duration: f64,
    /// Total tasks executed
    pub total_tasks: u64,
    /// Coordinator uptime (seconds)
    pub uptime_seconds: u64,
}

impl Default for CoordinatorStats {
    fn default() -> Self {
        Self {
            total_workers: 0,
            active_workers: 0,
            total_jobs: 0,
            pending_jobs: 0,
            running_jobs: 0,
            completed_jobs: 0,
            failed_jobs: 0,
            avg_job_duration: 0.0,
            total_tasks: 0,
            uptime_seconds: 0,
        }
    }
}

/// Task execution context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskContext {
    /// Task ID
    pub task_id: TaskId,
    /// Job ID
    pub job_id: JobId,
    /// Worker ID
    pub worker_id: NodeId,
    /// Task type
    pub task_type: String,
    /// Task payload
    pub payload: serde_json::Value,
    /// Task metadata
    pub metadata: std::collections::HashMap<String, String>,
    /// Retry count
    pub retry_count: u32,
    /// Max retries
    pub max_retries: u32,
    /// Timeout (seconds)
    pub timeout_seconds: u64,
}

/// Task result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    /// Task ID
    pub task_id: TaskId,
    /// Success flag
    pub success: bool,
    /// Result data
    pub result: Option<serde_json::Value>,
    /// Error message
    pub error: Option<String>,
    /// Execution time (milliseconds)
    pub execution_time_ms: u64,
    /// Worker ID
    pub worker_id: NodeId,
    /// Completed at
    pub completed_at: DateTime<Utc>,
}

impl TaskResult {
    /// Create a success result
    pub fn success(
        task_id: TaskId,
        worker_id: NodeId,
        result: serde_json::Value,
        execution_time_ms: u64,
    ) -> Self {
        Self {
            task_id,
            success: true,
            result: Some(result),
            error: None,
            execution_time_ms,
            worker_id,
            completed_at: Utc::now(),
        }
    }

    /// Create a failure result
    pub fn failure(
        task_id: TaskId,
        worker_id: NodeId,
        error: String,
        execution_time_ms: u64,
    ) -> Self {
        Self {
            task_id,
            success: false,
            result: None,
            error: Some(error),
            execution_time_ms,
            worker_id,
            completed_at: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_worker_info_availability() {
        let mut worker = WorkerInfo {
            id: "worker-1".to_string(),
            address: "localhost:50052".to_string(),
            status: WorkerStatus::Idle,
            capacity: 10,
            current_tasks: 5,
            completed_tasks: 100,
            failed_tasks: 2,
            tags: vec![],
            metadata: std::collections::HashMap::new(),
            last_heartbeat: Utc::now(),
            registered_at: Utc::now(),
        };

        assert!(worker.is_available());
        assert_eq!(worker.load(), 0.5);

        worker.current_tasks = 10;
        assert!(!worker.is_available());
        assert_eq!(worker.load(), 1.0);

        worker.status = WorkerStatus::Busy;
        assert!(!worker.is_available());
    }

    #[test]
    fn test_worker_info_health() {
        let worker = WorkerInfo {
            id: "worker-1".to_string(),
            address: "localhost:50052".to_string(),
            status: WorkerStatus::Idle,
            capacity: 10,
            current_tasks: 0,
            completed_tasks: 0,
            failed_tasks: 0,
            tags: vec![],
            metadata: std::collections::HashMap::new(),
            last_heartbeat: Utc::now(),
            registered_at: Utc::now(),
        };

        assert!(worker.is_healthy(10));
    }

    #[test]
    fn test_task_result() {
        let success = TaskResult::success(
            "task-1".to_string(),
            "worker-1".to_string(),
            serde_json::json!({"result": "ok"}),
            100,
        );

        assert!(success.success);
        assert!(success.result.is_some());
        assert!(success.error.is_none());

        let failure = TaskResult::failure(
            "task-2".to_string(),
            "worker-1".to_string(),
            "execution failed".to_string(),
            50,
        );

        assert!(!failure.success);
        assert!(failure.result.is_none());
        assert!(failure.error.is_some());
    }
}

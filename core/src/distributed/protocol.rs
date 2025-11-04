// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Protocol definitions for coordinator-worker communication.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::distributed::types::{JobId, NodeId, TaskId, WorkerCapabilities};

/// Register worker request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterRequest {
    /// Worker ID
    pub worker_id: NodeId,
    /// Worker address
    pub address: String,
    /// Worker capacity
    pub capacity: usize,
    /// Worker capabilities
    pub capabilities: WorkerCapabilities,
    /// Worker tags
    pub tags: Vec<String>,
    /// Worker metadata
    pub metadata: HashMap<String, String>,
}

/// Register worker response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterResponse {
    /// Registration success
    pub success: bool,
    /// Coordinator version
    pub coordinator_version: String,
    /// Assigned worker ID (may differ from requested)
    pub assigned_worker_id: NodeId,
    /// Heartbeat interval (seconds)
    pub heartbeat_interval: u64,
    /// Message
    pub message: String,
}

/// Deregister worker request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeregisterRequest {
    /// Worker ID
    pub worker_id: NodeId,
    /// Reason for deregistration
    pub reason: String,
}

/// Deregister worker response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeregisterResponse {
    /// Deregistration success
    pub success: bool,
    /// Message
    pub message: String,
}

/// Health check request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckRequest {
    /// Worker ID
    pub worker_id: NodeId,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

/// Health check response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResponse {
    /// Health status
    pub healthy: bool,
    /// Worker status
    pub status: String,
    /// Current tasks
    pub current_tasks: usize,
    /// Available capacity
    pub available_capacity: usize,
    /// CPU usage (0.0-1.0)
    pub cpu_usage: f64,
    /// Memory usage (0.0-1.0)
    pub memory_usage: f64,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

/// Heartbeat request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatRequest {
    /// Worker ID
    pub worker_id: NodeId,
    /// Worker status
    pub status: String,
    /// Current tasks
    pub current_tasks: usize,
    /// Completed tasks since last heartbeat
    pub completed_tasks_delta: u64,
    /// Failed tasks since last heartbeat
    pub failed_tasks_delta: u64,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

/// Heartbeat response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatResponse {
    /// Acknowledgment
    pub acknowledged: bool,
    /// Coordinator has pending tasks for this worker
    pub has_pending_tasks: bool,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

/// Job submission request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobRequest {
    /// Job type
    pub job_type: String,
    /// Job payload
    pub payload: serde_json::Value,
    /// Job priority (higher = more important)
    pub priority: i32,
    /// Required worker tags
    pub required_tags: Vec<String>,
    /// Job metadata
    pub metadata: HashMap<String, String>,
    /// Job timeout (seconds)
    pub timeout_seconds: u64,
    /// Max retries
    pub max_retries: u32,
}

impl JobRequest {
    /// Create a new job request builder
    pub fn builder() -> JobRequestBuilder {
        JobRequestBuilder::default()
    }
}

/// Job request builder
#[derive(Default)]
pub struct JobRequestBuilder {
    job_type: Option<String>,
    payload: Option<serde_json::Value>,
    priority: i32,
    required_tags: Vec<String>,
    metadata: HashMap<String, String>,
    timeout_seconds: u64,
    max_retries: u32,
}

impl JobRequestBuilder {
    pub fn job_type(mut self, job_type: impl Into<String>) -> Self {
        self.job_type = Some(job_type.into());
        self
    }

    pub fn payload(mut self, payload: serde_json::Value) -> Self {
        self.payload = Some(payload);
        self
    }

    pub fn priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    pub fn required_tags(mut self, tags: Vec<String>) -> Self {
        self.required_tags = tags;
        self
    }

    pub fn metadata(mut self, metadata: HashMap<String, String>) -> Self {
        self.metadata = metadata;
        self
    }

    pub fn timeout_seconds(mut self, seconds: u64) -> Self {
        self.timeout_seconds = seconds;
        self
    }

    pub fn max_retries(mut self, retries: u32) -> Self {
        self.max_retries = retries;
        self
    }

    pub fn build(self) -> JobRequest {
        JobRequest {
            job_type: self.job_type.unwrap_or_else(|| "default".to_string()),
            payload: self.payload.unwrap_or(serde_json::json!({})),
            priority: self.priority,
            required_tags: self.required_tags,
            metadata: self.metadata,
            timeout_seconds: self.timeout_seconds,
            max_retries: self.max_retries,
        }
    }
}

/// Job submission response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobResponse {
    /// Job ID
    pub job_id: JobId,
    /// Submission success
    pub success: bool,
    /// Message
    pub message: String,
    /// Estimated completion time
    pub estimated_completion: Option<DateTime<Utc>>,
}

/// Task assignment request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskRequest {
    /// Task ID
    pub task_id: TaskId,
    /// Job ID
    pub job_id: JobId,
    /// Task type
    pub task_type: String,
    /// Task payload
    pub payload: serde_json::Value,
    /// Task metadata
    pub metadata: HashMap<String, String>,
    /// Timeout (seconds)
    pub timeout_seconds: u64,
    /// Retry count
    pub retry_count: u32,
}

/// Task response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResponse {
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
    /// Completed at
    pub completed_at: DateTime<Utc>,
}

/// Request task assignment from coordinator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullTaskRequest {
    /// Worker ID
    pub worker_id: NodeId,
    /// Number of tasks requested
    pub count: usize,
    /// Worker capabilities
    pub capabilities: Vec<String>,
}

/// Task assignment response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullTaskResponse {
    /// Assigned tasks
    pub tasks: Vec<TaskRequest>,
    /// Message
    pub message: String,
}

/// Job status query request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobStatusRequest {
    /// Job ID
    pub job_id: JobId,
}

/// Job status response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobStatusResponse {
    /// Job ID
    pub job_id: JobId,
    /// Job status
    pub status: String,
    /// Progress (0.0-1.0)
    pub progress: f64,
    /// Result (if completed)
    pub result: Option<serde_json::Value>,
    /// Error (if failed)
    pub error: Option<String>,
    /// Created at
    pub created_at: DateTime<Utc>,
    /// Started at
    pub started_at: Option<DateTime<Utc>>,
    /// Completed at
    pub completed_at: Option<DateTime<Utc>>,
}

/// Cancel job request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelJobRequest {
    /// Job ID
    pub job_id: JobId,
    /// Reason for cancellation
    pub reason: String,
}

/// Cancel job response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelJobResponse {
    /// Cancellation success
    pub success: bool,
    /// Message
    pub message: String,
}

/// List workers request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListWorkersRequest {
    /// Filter by status
    pub status_filter: Option<String>,
    /// Filter by tags
    pub tag_filter: Vec<String>,
}

/// List workers response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListWorkersResponse {
    /// Workers
    pub workers: Vec<WorkerSummary>,
    /// Total count
    pub total: usize,
}

/// Worker summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerSummary {
    /// Worker ID
    pub worker_id: NodeId,
    /// Worker address
    pub address: String,
    /// Worker status
    pub status: String,
    /// Current tasks
    pub current_tasks: usize,
    /// Capacity
    pub capacity: usize,
    /// Load (0.0-1.0)
    pub load: f64,
    /// Tags
    pub tags: Vec<String>,
    /// Last heartbeat
    pub last_heartbeat: DateTime<Utc>,
}

/// Cluster statistics request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterStatsRequest {}

/// Cluster statistics response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterStatsResponse {
    /// Total workers
    pub total_workers: usize,
    /// Active workers
    pub active_workers: usize,
    /// Total jobs
    pub total_jobs: u64,
    /// Pending jobs
    pub pending_jobs: usize,
    /// Running jobs
    pub running_jobs: usize,
    /// Completed jobs
    pub completed_jobs: u64,
    /// Failed jobs
    pub failed_jobs: u64,
    /// Average job duration (seconds)
    pub avg_job_duration: f64,
    /// Cluster uptime (seconds)
    pub uptime_seconds: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_job_request_builder() {
        let job = JobRequest::builder()
            .job_type("benchmark")
            .payload(serde_json::json!({"model": "gpt-4"}))
            .priority(10)
            .timeout_seconds(300)
            .max_retries(3)
            .build();

        assert_eq!(job.job_type, "benchmark");
        assert_eq!(job.priority, 10);
        assert_eq!(job.timeout_seconds, 300);
        assert_eq!(job.max_retries, 3);
    }

    #[test]
    fn test_heartbeat_request() {
        let heartbeat = HeartbeatRequest {
            worker_id: "worker-1".to_string(),
            status: "idle".to_string(),
            current_tasks: 0,
            completed_tasks_delta: 5,
            failed_tasks_delta: 0,
            timestamp: Utc::now(),
        };

        assert_eq!(heartbeat.worker_id, "worker-1");
        assert_eq!(heartbeat.current_tasks, 0);
    }
}

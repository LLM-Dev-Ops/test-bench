// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Coordinator node implementation.

use anyhow::Result;
use chrono::Utc;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, error};

use crate::distributed::{
    cluster::{ClusterState, ClusterMetrics},
    health::HealthMonitor,
    jobs::{Job, JobQueue, JobStatus},
    protocol::*,
    types::*,
    DEFAULT_COORDINATOR_PORT, DEFAULT_HEARTBEAT_INTERVAL,
    DEFAULT_HEALTH_CHECK_TIMEOUT, DEFAULT_MAX_RETRIES,
};

/// Coordinator configuration
#[derive(Debug, Clone)]
pub struct CoordinatorConfig {
    /// Bind address
    pub bind_address: SocketAddr,
    /// Heartbeat interval (seconds)
    pub heartbeat_interval: u64,
    /// Health check timeout (seconds)
    pub health_check_timeout: u64,
    /// Unhealthy worker threshold (seconds)
    pub unhealthy_threshold: u64,
    /// Max retries for failed tasks
    pub max_retries: u32,
    /// Max completed jobs to keep
    pub max_completed_jobs: usize,
}

impl Default for CoordinatorConfig {
    fn default() -> Self {
        Self {
            bind_address: SocketAddr::from(([0, 0, 0, 0], DEFAULT_COORDINATOR_PORT)),
            heartbeat_interval: DEFAULT_HEARTBEAT_INTERVAL,
            health_check_timeout: DEFAULT_HEALTH_CHECK_TIMEOUT,
            unhealthy_threshold: 30,
            max_retries: DEFAULT_MAX_RETRIES,
            max_completed_jobs: 1000,
        }
    }
}

impl CoordinatorConfig {
    /// Create a configuration builder
    pub fn builder() -> CoordinatorConfigBuilder {
        CoordinatorConfigBuilder::default()
    }
}

/// Coordinator configuration builder
#[derive(Default)]
pub struct CoordinatorConfigBuilder {
    config: CoordinatorConfig,
}

impl CoordinatorConfigBuilder {
    pub fn bind_address(mut self, addr: SocketAddr) -> Self {
        self.config.bind_address = addr;
        self
    }

    pub fn heartbeat_interval(mut self, seconds: u64) -> Self {
        self.config.heartbeat_interval = seconds;
        self
    }

    pub fn health_check_timeout(mut self, seconds: u64) -> Self {
        self.config.health_check_timeout = seconds;
        self
    }

    pub fn unhealthy_threshold(mut self, seconds: u64) -> Self {
        self.config.unhealthy_threshold = seconds;
        self
    }

    pub fn max_retries(mut self, retries: u32) -> Self {
        self.config.max_retries = retries;
        self
    }

    pub fn build(self) -> CoordinatorConfig {
        self.config
    }
}

/// Coordinator node
pub struct Coordinator {
    /// Configuration
    config: CoordinatorConfig,
    /// Cluster state
    cluster: Arc<ClusterState>,
    /// Job queue
    jobs: Arc<JobQueue>,
    /// Health monitor
    health_monitor: Arc<HealthMonitor>,
    /// Task assignments (task_id -> worker_id)
    task_assignments: Arc<RwLock<HashMap<TaskId, NodeId>>>,
}

impl Coordinator {
    /// Create a new coordinator
    pub fn new(config: CoordinatorConfig) -> Self {
        let cluster = Arc::new(ClusterState::new());
        let jobs = Arc::new(JobQueue::new(config.max_completed_jobs));

        let health_monitor = Arc::new(HealthMonitor::new(
            cluster.clone(),
            config.heartbeat_interval,
            config.health_check_timeout,
            config.unhealthy_threshold,
        ));

        Self {
            config,
            cluster,
            jobs,
            health_monitor,
            task_assignments: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Start the coordinator
    pub async fn start(self: Arc<Self>) -> Result<()> {
        info!("Starting coordinator on {}", self.config.bind_address);

        // Start health monitor
        let health_monitor = self.health_monitor.clone();
        tokio::spawn(async move {
            health_monitor.start().await;
        });

        // Start task scheduler
        let coordinator = self.clone();
        tokio::spawn(async move {
            coordinator.task_scheduler_loop().await;
        });

        info!("Coordinator started successfully");

        // Keep the coordinator running
        tokio::signal::ctrl_c().await?;
        info!("Shutting down coordinator");

        Ok(())
    }

    /// Register a worker
    pub fn register_worker(&self, request: RegisterRequest) -> RegisterResponse {
        info!("Registering worker: {}", request.worker_id);

        let worker = WorkerInfo {
            id: request.worker_id.clone(),
            address: request.address,
            status: WorkerStatus::Idle,
            capacity: request.capacity,
            current_tasks: 0,
            completed_tasks: 0,
            failed_tasks: 0,
            tags: request.tags,
            metadata: request.metadata,
            last_heartbeat: Utc::now(),
            registered_at: Utc::now(),
        };

        self.cluster.register_worker(worker);

        RegisterResponse {
            success: true,
            coordinator_version: crate::distributed::DISTRIBUTED_VERSION.to_string(),
            assigned_worker_id: request.worker_id,
            heartbeat_interval: self.config.heartbeat_interval,
            message: "Worker registered successfully".to_string(),
        }
    }

    /// Deregister a worker
    pub fn deregister_worker(&self, request: DeregisterRequest) -> DeregisterResponse {
        info!("Deregistering worker: {} (reason: {})", request.worker_id, request.reason);

        if let Some(_worker) = self.cluster.deregister_worker(&request.worker_id) {
            DeregisterResponse {
                success: true,
                message: "Worker deregistered successfully".to_string(),
            }
        } else {
            DeregisterResponse {
                success: false,
                message: "Worker not found".to_string(),
            }
        }
    }

    /// Handle heartbeat from worker
    pub fn handle_heartbeat(&self, request: HeartbeatRequest) -> HeartbeatResponse {
        self.cluster.update_worker_heartbeat(&request.worker_id);

        // Check if there are pending tasks
        let has_pending = !self.jobs.list_pending().is_empty();

        HeartbeatResponse {
            acknowledged: true,
            has_pending_tasks: has_pending,
            timestamp: Utc::now(),
        }
    }

    /// Submit a job
    pub fn submit_job(&self, request: JobRequest) -> JobResponse {
        let job_id = uuid::Uuid::new_v4().to_string();

        info!("Submitting job: {} (type: {})", job_id, request.job_type);

        let mut job = Job::from_request(job_id.clone(), request);

        // Create a task for this job (simplified - could create multiple tasks)
        let task_id = uuid::Uuid::new_v4().to_string();
        job.tasks.push(task_id);

        self.jobs.submit(job);
        self.cluster.increment_jobs();

        JobResponse {
            job_id,
            success: true,
            message: "Job submitted successfully".to_string(),
            estimated_completion: None,
        }
    }

    /// Get job status
    pub fn get_job_status(&self, request: JobStatusRequest) -> Option<JobStatusResponse> {
        let job = self.jobs.get(&request.job_id)?;

        Some(JobStatusResponse {
            job_id: job.id.clone(),
            status: job.status.to_string(),
            progress: job.progress(),
            result: job.result,
            error: job.error,
            created_at: job.created_at,
            started_at: job.started_at,
            completed_at: job.completed_at,
        })
    }

    /// Cancel a job
    pub fn cancel_job(&self, request: CancelJobRequest) -> CancelJobResponse {
        if self.jobs.cancel(&request.job_id, request.reason) {
            CancelJobResponse {
                success: true,
                message: "Job cancelled successfully".to_string(),
            }
        } else {
            CancelJobResponse {
                success: false,
                message: "Job not found or already completed".to_string(),
            }
        }
    }

    /// Pull tasks for a worker
    pub fn pull_tasks(&self, request: PullTaskRequest) -> PullTaskResponse {
        let mut tasks = Vec::new();

        // Get worker info
        let worker = match self.cluster.get_worker(&request.worker_id) {
            Some(w) => w,
            None => {
                return PullTaskResponse {
                    tasks,
                    message: "Worker not found".to_string(),
                };
            }
        };

        // Get available capacity
        let available = worker.capacity.saturating_sub(worker.current_tasks);
        let count = request.count.min(available);

        for _ in 0..count {
            if let Some(mut job) = self.jobs.next() {
                // Get the first task from the job
                if let Some(task_id) = job.tasks.first() {
                    let task_request = TaskRequest {
                        task_id: task_id.clone(),
                        job_id: job.id.clone(),
                        task_type: job.job_type.clone(),
                        payload: job.payload.clone(),
                        metadata: job.metadata.clone(),
                        timeout_seconds: job.timeout_seconds,
                        retry_count: job.retry_count,
                    };

                    tasks.push(task_request);

                    // Record assignment
                    let task_id = task_id.clone();
                    let worker_id = request.worker_id.clone();
                    tokio::spawn({
                        let assignments = self.task_assignments.clone();
                        async move {
                            assignments.write().await.insert(task_id, worker_id);
                        }
                    });

                    // Update cluster state
                    self.cluster.increment_worker_tasks(&request.worker_id);
                }
            } else {
                break;
            }
        }

        let task_count = tasks.len();
        PullTaskResponse {
            tasks,
            message: format!("Assigned {} tasks", task_count),
        }
    }

    /// Handle task completion
    pub async fn complete_task(&self, task_id: &TaskId, result: TaskResponse) {
        // Get worker assignment
        let worker_id = {
            let assignments = self.task_assignments.read().await;
            assignments.get(task_id).cloned()
        };

        if let Some(worker_id) = worker_id {
            // Update cluster state
            self.cluster.decrement_worker_tasks(&worker_id, result.success);

            // Find the job
            // In a real implementation, we'd maintain a task_id -> job_id mapping
            // For now, we'll update the job queue directly

            if result.success {
                self.cluster.increment_completed_jobs();
            } else {
                self.cluster.increment_failed_jobs();
            }

            // Remove assignment
            self.task_assignments.write().await.remove(task_id);
        }
    }

    /// List workers
    pub fn list_workers(&self, request: ListWorkersRequest) -> ListWorkersResponse {
        let mut workers = if let Some(status) = &request.status_filter {
            let status_enum = match status.as_str() {
                "idle" => WorkerStatus::Idle,
                "busy" => WorkerStatus::Busy,
                "failed" => WorkerStatus::Failed,
                _ => WorkerStatus::Idle,
            };
            self.cluster.list_workers_by_status(status_enum)
        } else {
            self.cluster.list_workers()
        };

        // Apply tag filter
        if !request.tag_filter.is_empty() {
            workers.retain(|w| {
                request.tag_filter.iter().all(|t| w.tags.contains(t))
            });
        }

        let summaries: Vec<WorkerSummary> = workers
            .into_iter()
            .map(|w| WorkerSummary {
                worker_id: w.id.clone(),
                address: w.address.clone(),
                status: w.status.to_string(),
                current_tasks: w.current_tasks,
                capacity: w.capacity,
                load: w.load(),
                tags: w.tags,
                last_heartbeat: w.last_heartbeat,
            })
            .collect();

        let total = summaries.len();

        ListWorkersResponse {
            workers: summaries,
            total,
        }
    }

    /// Get cluster statistics
    pub fn get_cluster_stats(&self) -> ClusterStatsResponse {
        let metrics = self.cluster.metrics();
        let queue_stats = self.jobs.stats();

        ClusterStatsResponse {
            total_workers: metrics.total_workers,
            active_workers: metrics.active_workers,
            total_jobs: metrics.total_jobs,
            pending_jobs: queue_stats.pending_jobs,
            running_jobs: queue_stats.running_jobs,
            completed_jobs: metrics.completed_jobs,
            failed_jobs: metrics.failed_jobs,
            avg_job_duration: 0.0, // Would calculate from job history
            uptime_seconds: metrics.uptime_seconds,
        }
    }

    /// Task scheduler loop
    async fn task_scheduler_loop(&self) {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(1));

        loop {
            interval.tick().await;

            // In a real implementation, this would:
            // 1. Check for pending jobs
            // 2. Find available workers
            // 3. Assign tasks to workers
            // 4. Handle task timeouts
            // 5. Retry failed tasks

            // For now, workers pull tasks, so this is just a placeholder
        }
    }

    /// Get cluster metrics
    pub fn metrics(&self) -> ClusterMetrics {
        self.cluster.metrics()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coordinator_creation() {
        let config = CoordinatorConfig::default();
        let coordinator = Coordinator::new(config);

        assert_eq!(coordinator.jobs.stats().pending_jobs, 0);
    }

    #[test]
    fn test_worker_registration() {
        let coordinator = Coordinator::new(CoordinatorConfig::default());

        let request = RegisterRequest {
            worker_id: "worker-1".to_string(),
            address: "localhost:50052".to_string(),
            capacity: 10,
            capabilities: WorkerCapabilities::default(),
            tags: vec![],
            metadata: HashMap::new(),
        };

        let response = coordinator.register_worker(request);

        assert!(response.success);
        assert_eq!(response.assigned_worker_id, "worker-1");

        let workers = coordinator.cluster.list_workers();
        assert_eq!(workers.len(), 1);
    }

    #[test]
    fn test_job_submission() {
        let coordinator = Coordinator::new(CoordinatorConfig::default());

        let request = JobRequest::builder()
            .job_type("benchmark")
            .payload(serde_json::json!({"test": "data"}))
            .build();

        let response = coordinator.submit_job(request);

        assert!(response.success);
        assert!(!response.job_id.is_empty());

        let stats = coordinator.jobs.stats();
        assert_eq!(stats.pending_jobs, 1);
    }
}

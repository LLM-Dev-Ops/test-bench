// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Worker node implementation.

use anyhow::Result;
use chrono::Utc;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;
use tokio::time::interval;
use tracing::{info, warn, error, debug};

use crate::distributed::{
    protocol::*,
    types::*,
    DEFAULT_HEARTBEAT_INTERVAL,
};

/// Worker configuration
#[derive(Debug, Clone)]
pub struct WorkerConfig {
    /// Worker ID
    pub worker_id: NodeId,
    /// Coordinator address
    pub coordinator_address: String,
    /// Worker bind address
    pub bind_address: String,
    /// Worker capacity (max concurrent tasks)
    pub capacity: usize,
    /// Worker capabilities
    pub capabilities: WorkerCapabilities,
    /// Worker tags
    pub tags: Vec<String>,
    /// Worker metadata
    pub metadata: HashMap<String, String>,
    /// Heartbeat interval (seconds)
    pub heartbeat_interval: u64,
    /// Task pull interval (seconds)
    pub task_pull_interval: u64,
}

impl Default for WorkerConfig {
    fn default() -> Self {
        Self {
            worker_id: format!("worker-{}", uuid::Uuid::new_v4()),
            coordinator_address: format!("http://localhost:{}", crate::distributed::DEFAULT_COORDINATOR_PORT),
            bind_address: "0.0.0.0:0".to_string(),
            capacity: num_cpus::get(),
            capabilities: WorkerCapabilities::default(),
            tags: vec![],
            metadata: HashMap::new(),
            heartbeat_interval: DEFAULT_HEARTBEAT_INTERVAL,
            task_pull_interval: 1,
        }
    }
}

impl WorkerConfig {
    /// Create a configuration builder
    pub fn builder() -> WorkerConfigBuilder {
        WorkerConfigBuilder::default()
    }
}

/// Worker configuration builder
#[derive(Default)]
pub struct WorkerConfigBuilder {
    config: WorkerConfig,
}

impl WorkerConfigBuilder {
    pub fn worker_id(mut self, id: impl Into<String>) -> Self {
        self.config.worker_id = id.into();
        self
    }

    pub fn coordinator_address(mut self, addr: impl Into<String>) -> Self {
        self.config.coordinator_address = addr.into();
        self
    }

    pub fn bind_address(mut self, addr: impl Into<String>) -> Self {
        self.config.bind_address = addr.into();
        self
    }

    pub fn capacity(mut self, capacity: usize) -> Self {
        self.config.capacity = capacity;
        self
    }

    pub fn capabilities(mut self, capabilities: WorkerCapabilities) -> Self {
        self.config.capabilities = capabilities;
        self
    }

    pub fn tags(mut self, tags: Vec<String>) -> Self {
        self.config.tags = tags;
        self
    }

    pub fn metadata(mut self, metadata: HashMap<String, String>) -> Self {
        self.config.metadata = metadata;
        self
    }

    pub fn build(self) -> WorkerConfig {
        self.config
    }
}

/// Task executor trait
#[async_trait::async_trait]
pub trait TaskExecutor: Send + Sync {
    /// Execute a task
    async fn execute(&self, task: TaskRequest) -> Result<TaskResponse>;
}

/// Default task executor
pub struct DefaultTaskExecutor;

#[async_trait::async_trait]
impl TaskExecutor for DefaultTaskExecutor {
    async fn execute(&self, task: TaskRequest) -> Result<TaskResponse> {
        // Simulate task execution
        tokio::time::sleep(Duration::from_millis(100)).await;

        Ok(TaskResponse {
            task_id: task.task_id,
            success: true,
            result: Some(serde_json::json!({
                "status": "completed",
                "message": "Task executed successfully"
            })),
            error: None,
            execution_time_ms: 100,
            completed_at: Utc::now(),
        })
    }
}

/// Worker node
pub struct Worker {
    /// Configuration
    config: WorkerConfig,
    /// Task executor
    executor: Arc<dyn TaskExecutor>,
    /// Semaphore for controlling concurrency
    semaphore: Arc<Semaphore>,
    /// Current task count
    current_tasks: Arc<AtomicUsize>,
    /// Completed tasks
    completed_tasks: Arc<AtomicUsize>,
    /// Failed tasks
    failed_tasks: Arc<AtomicUsize>,
    /// Running tasks
    running_tasks: Arc<RwLock<HashMap<TaskId, tokio::task::JoinHandle<()>>>>,
    /// Worker status
    status: Arc<RwLock<WorkerStatus>>,
}

impl Worker {
    /// Create a new worker
    pub fn new(config: WorkerConfig) -> Self {
        Self::with_executor(config, Arc::new(DefaultTaskExecutor))
    }

    /// Create a new worker with custom executor
    pub fn with_executor(config: WorkerConfig, executor: Arc<dyn TaskExecutor>) -> Self {
        let semaphore = Arc::new(Semaphore::new(config.capacity));

        Self {
            config,
            executor,
            semaphore,
            current_tasks: Arc::new(AtomicUsize::new(0)),
            completed_tasks: Arc::new(AtomicUsize::new(0)),
            failed_tasks: Arc::new(AtomicUsize::new(0)),
            running_tasks: Arc::new(RwLock::new(HashMap::new())),
            status: Arc::new(RwLock::new(WorkerStatus::Starting)),
        }
    }

    /// Start the worker
    pub async fn start(self: Arc<Self>) -> Result<()> {
        info!("Starting worker: {}", self.config.worker_id);

        // Register with coordinator
        self.register().await?;

        *self.status.write() = WorkerStatus::Idle;

        // Start heartbeat loop
        let worker = self.clone();
        tokio::spawn(async move {
            worker.heartbeat_loop().await;
        });

        // Start task pulling loop
        let worker = self.clone();
        tokio::spawn(async move {
            worker.task_pull_loop().await;
        });

        info!("Worker started: {}", self.config.worker_id);

        // Keep the worker running
        tokio::signal::ctrl_c().await?;

        info!("Shutting down worker: {}", self.config.worker_id);
        self.shutdown().await?;

        Ok(())
    }

    /// Register with coordinator
    async fn register(&self) -> Result<()> {
        info!("Registering with coordinator: {}", self.config.coordinator_address);

        let request = RegisterRequest {
            worker_id: self.config.worker_id.clone(),
            address: self.config.bind_address.clone(),
            capacity: self.config.capacity,
            capabilities: self.config.capabilities.clone(),
            tags: self.config.tags.clone(),
            metadata: self.config.metadata.clone(),
        };

        // In a real implementation, this would make an HTTP/gRPC call
        // For now, just log the registration
        debug!("Registration request: {:?}", request);

        Ok(())
    }

    /// Deregister from coordinator
    async fn deregister(&self, reason: String) -> Result<()> {
        info!("Deregistering from coordinator: {}", reason);

        let request = DeregisterRequest {
            worker_id: self.config.worker_id.clone(),
            reason,
        };

        // In a real implementation, this would make an HTTP/gRPC call
        debug!("Deregistration request: {:?}", request);

        Ok(())
    }

    /// Heartbeat loop
    async fn heartbeat_loop(&self) {
        let mut ticker = interval(Duration::from_secs(self.config.heartbeat_interval));

        loop {
            ticker.tick().await;

            let request = HeartbeatRequest {
                worker_id: self.config.worker_id.clone(),
                status: self.status.read().to_string(),
                current_tasks: self.current_tasks.load(Ordering::Relaxed),
                completed_tasks_delta: 0, // Would track delta
                failed_tasks_delta: 0,
                timestamp: Utc::now(),
            };

            // In a real implementation, this would make an HTTP/gRPC call
            debug!("Heartbeat: {:?}", request);
        }
    }

    /// Task pulling loop
    async fn task_pull_loop(&self) {
        let mut ticker = interval(Duration::from_secs(self.config.task_pull_interval));

        loop {
            ticker.tick().await;

            // Check if we have capacity
            let current = self.current_tasks.load(Ordering::Relaxed);
            if current >= self.config.capacity {
                continue;
            }

            // Request tasks
            let available = self.config.capacity - current;
            let request = PullTaskRequest {
                worker_id: self.config.worker_id.clone(),
                count: available,
                capabilities: self.config.capabilities.job_types.clone(),
            };

            // In a real implementation, this would make an HTTP/gRPC call
            // For now, just simulate no tasks available
            debug!("Pulling {} tasks", available);
        }
    }

    /// Execute a task
    async fn execute_task(self: Arc<Self>, task: TaskRequest) {
        let task_id = task.task_id.clone();

        // Acquire semaphore
        let permit = match self.semaphore.acquire().await {
            Ok(p) => p,
            Err(e) => {
                error!("Failed to acquire semaphore: {}", e);
                return;
            }
        };

        self.current_tasks.fetch_add(1, Ordering::Relaxed);

        // Update status
        if self.current_tasks.load(Ordering::Relaxed) > 0 {
            *self.status.write() = WorkerStatus::Busy;
        }

        info!("Executing task: {}", task_id);

        // Execute task
        let result = match self.executor.execute(task).await {
            Ok(response) => {
                if response.success {
                    self.completed_tasks.fetch_add(1, Ordering::Relaxed);
                    info!("Task completed: {}", task_id);
                } else {
                    self.failed_tasks.fetch_add(1, Ordering::Relaxed);
                    warn!("Task failed: {} - {:?}", task_id, response.error);
                }
                response
            }
            Err(e) => {
                self.failed_tasks.fetch_add(1, Ordering::Relaxed);
                error!("Task execution error: {} - {}", task_id, e);

                TaskResponse {
                    task_id: task_id.clone(),
                    success: false,
                    result: None,
                    error: Some(e.to_string()),
                    execution_time_ms: 0,
                    completed_at: Utc::now(),
                }
            }
        };

        // Report result to coordinator
        // In a real implementation, this would make an HTTP/gRPC call
        debug!("Task result: {:?}", result);

        self.current_tasks.fetch_sub(1, Ordering::Relaxed);

        // Update status
        if self.current_tasks.load(Ordering::Relaxed) == 0 {
            *self.status.write() = WorkerStatus::Idle;
        }

        drop(permit);
    }

    /// Shutdown the worker
    async fn shutdown(&self) -> Result<()> {
        *self.status.write() = WorkerStatus::ShuttingDown;

        // Wait for all running tasks to complete
        info!("Waiting for {} tasks to complete", self.current_tasks.load(Ordering::Relaxed));

        while self.current_tasks.load(Ordering::Relaxed) > 0 {
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        // Deregister
        self.deregister("Normal shutdown".to_string()).await?;

        *self.status.write() = WorkerStatus::Offline;

        Ok(())
    }

    /// Get worker statistics
    pub fn stats(&self) -> WorkerStats {
        WorkerStats {
            worker_id: self.config.worker_id.clone(),
            status: *self.status.read(),
            capacity: self.config.capacity,
            current_tasks: self.current_tasks.load(Ordering::Relaxed),
            completed_tasks: self.completed_tasks.load(Ordering::Relaxed),
            failed_tasks: self.failed_tasks.load(Ordering::Relaxed),
        }
    }
}

/// Worker statistics
#[derive(Debug, Clone)]
pub struct WorkerStats {
    pub worker_id: NodeId,
    pub status: WorkerStatus,
    pub capacity: usize,
    pub current_tasks: usize,
    pub completed_tasks: usize,
    pub failed_tasks: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_worker_config() {
        let config = WorkerConfig::builder()
            .worker_id("test-worker")
            .coordinator_address("http://localhost:50051")
            .capacity(5)
            .build();

        assert_eq!(config.worker_id, "test-worker");
        assert_eq!(config.coordinator_address, "http://localhost:50051");
        assert_eq!(config.capacity, 5);
    }

    #[test]
    fn test_worker_creation() {
        let config = WorkerConfig::default();
        let worker = Worker::new(config);

        let stats = worker.stats();
        assert_eq!(stats.current_tasks, 0);
        assert_eq!(stats.completed_tasks, 0);
    }

    #[tokio::test]
    async fn test_task_executor() {
        let executor = DefaultTaskExecutor;

        let task = TaskRequest {
            task_id: "task-1".to_string(),
            job_id: "job-1".to_string(),
            task_type: "test".to_string(),
            payload: serde_json::json!({}),
            metadata: HashMap::new(),
            timeout_seconds: 30,
            retry_count: 0,
        };

        let result = executor.execute(task).await.unwrap();
        assert!(result.success);
    }
}

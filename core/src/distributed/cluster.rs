// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Cluster state management.

use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use crate::distributed::types::{NodeId, WorkerInfo, WorkerStatus};

/// Cluster state
pub struct ClusterState {
    /// Workers in the cluster
    workers: Arc<DashMap<NodeId, WorkerInfo>>,
    /// Cluster start time
    start_time: DateTime<Utc>,
    /// Total jobs submitted
    total_jobs: Arc<AtomicU64>,
    /// Total jobs completed
    completed_jobs: Arc<AtomicU64>,
    /// Total jobs failed
    failed_jobs: Arc<AtomicU64>,
    /// Total tasks executed
    total_tasks: Arc<AtomicU64>,
}

impl ClusterState {
    /// Create a new cluster state
    pub fn new() -> Self {
        Self {
            workers: Arc::new(DashMap::new()),
            start_time: Utc::now(),
            total_jobs: Arc::new(AtomicU64::new(0)),
            completed_jobs: Arc::new(AtomicU64::new(0)),
            failed_jobs: Arc::new(AtomicU64::new(0)),
            total_tasks: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Register a worker
    pub fn register_worker(&self, worker: WorkerInfo) {
        self.workers.insert(worker.id.clone(), worker);
    }

    /// Deregister a worker
    pub fn deregister_worker(&self, worker_id: &NodeId) -> Option<WorkerInfo> {
        self.workers.remove(worker_id).map(|(_, w)| w)
    }

    /// Get worker by ID
    pub fn get_worker(&self, worker_id: &NodeId) -> Option<WorkerInfo> {
        self.workers.get(worker_id).map(|w| w.clone())
    }

    /// Update worker status
    pub fn update_worker_status(&self, worker_id: &NodeId, status: WorkerStatus) {
        if let Some(mut worker) = self.workers.get_mut(worker_id) {
            worker.status = status;
        }
    }

    /// Update worker heartbeat
    pub fn update_worker_heartbeat(&self, worker_id: &NodeId) {
        if let Some(mut worker) = self.workers.get_mut(worker_id) {
            worker.last_heartbeat = Utc::now();
        }
    }

    /// Increment worker task count
    pub fn increment_worker_tasks(&self, worker_id: &NodeId) {
        if let Some(mut worker) = self.workers.get_mut(worker_id) {
            worker.current_tasks += 1;
            if worker.current_tasks > 0 {
                worker.status = WorkerStatus::Busy;
            }
        }
    }

    /// Decrement worker task count
    pub fn decrement_worker_tasks(&self, worker_id: &NodeId, success: bool) {
        if let Some(mut worker) = self.workers.get_mut(worker_id) {
            if worker.current_tasks > 0 {
                worker.current_tasks -= 1;
            }

            if success {
                worker.completed_tasks += 1;
            } else {
                worker.failed_tasks += 1;
            }

            if worker.current_tasks == 0 {
                worker.status = WorkerStatus::Idle;
            }
        }
    }

    /// List all workers
    pub fn list_workers(&self) -> Vec<WorkerInfo> {
        self.workers.iter().map(|w| w.value().clone()).collect()
    }

    /// List workers by status
    pub fn list_workers_by_status(&self, status: WorkerStatus) -> Vec<WorkerInfo> {
        self.workers
            .iter()
            .filter(|w| w.status == status)
            .map(|w| w.value().clone())
            .collect()
    }

    /// List workers by tag
    pub fn list_workers_by_tag(&self, tag: &str) -> Vec<WorkerInfo> {
        self.workers
            .iter()
            .filter(|w| w.tags.contains(&tag.to_string()))
            .map(|w| w.value().clone())
            .collect()
    }

    /// Get available workers
    pub fn get_available_workers(&self) -> Vec<WorkerInfo> {
        self.workers
            .iter()
            .filter(|w| w.is_available())
            .map(|w| w.value().clone())
            .collect()
    }

    /// Get least loaded worker
    pub fn get_least_loaded_worker(&self) -> Option<WorkerInfo> {
        self.get_available_workers()
            .into_iter()
            .min_by(|a, b| a.load().partial_cmp(&b.load()).unwrap())
    }

    /// Get least loaded worker with tags
    pub fn get_least_loaded_worker_with_tags(&self, tags: &[String]) -> Option<WorkerInfo> {
        self.get_available_workers()
            .into_iter()
            .filter(|w| {
                if tags.is_empty() {
                    true
                } else {
                    tags.iter().all(|t| w.tags.contains(t))
                }
            })
            .min_by(|a, b| a.load().partial_cmp(&b.load()).unwrap())
    }

    /// Mark worker as failed
    pub fn mark_worker_failed(&self, worker_id: &NodeId) {
        self.update_worker_status(worker_id, WorkerStatus::Failed);
    }

    /// Remove unhealthy workers
    pub fn remove_unhealthy_workers(&self, timeout_seconds: u64) -> Vec<NodeId> {
        let mut removed = Vec::new();

        self.workers.retain(|id, worker| {
            if !worker.is_healthy(timeout_seconds) {
                removed.push(id.clone());
                false
            } else {
                true
            }
        });

        removed
    }

    /// Get cluster metrics
    pub fn metrics(&self) -> ClusterMetrics {
        let workers: Vec<WorkerInfo> = self.list_workers();

        let active_workers = workers
            .iter()
            .filter(|w| w.status == WorkerStatus::Idle || w.status == WorkerStatus::Busy)
            .count();

        let total_capacity: usize = workers.iter().map(|w| w.capacity).sum();
        let used_capacity: usize = workers.iter().map(|w| w.current_tasks).sum();

        let uptime = Utc::now()
            .signed_duration_since(self.start_time)
            .num_seconds() as u64;

        ClusterMetrics {
            total_workers: workers.len(),
            active_workers,
            idle_workers: self.list_workers_by_status(WorkerStatus::Idle).len(),
            busy_workers: self.list_workers_by_status(WorkerStatus::Busy).len(),
            failed_workers: self.list_workers_by_status(WorkerStatus::Failed).len(),
            total_capacity,
            used_capacity,
            available_capacity: total_capacity.saturating_sub(used_capacity),
            cluster_load: if total_capacity > 0 {
                used_capacity as f64 / total_capacity as f64
            } else {
                0.0
            },
            total_jobs: self.total_jobs.load(Ordering::Relaxed),
            completed_jobs: self.completed_jobs.load(Ordering::Relaxed),
            failed_jobs: self.failed_jobs.load(Ordering::Relaxed),
            total_tasks: self.total_tasks.load(Ordering::Relaxed),
            uptime_seconds: uptime,
        }
    }

    /// Increment job counter
    pub fn increment_jobs(&self) {
        self.total_jobs.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment completed jobs
    pub fn increment_completed_jobs(&self) {
        self.completed_jobs.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment failed jobs
    pub fn increment_failed_jobs(&self) {
        self.failed_jobs.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment task counter
    pub fn increment_tasks(&self, count: u64) {
        self.total_tasks.fetch_add(count, Ordering::Relaxed);
    }
}

impl Default for ClusterState {
    fn default() -> Self {
        Self::new()
    }
}

/// Cluster metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterMetrics {
    /// Total workers
    pub total_workers: usize,
    /// Active workers (idle + busy)
    pub active_workers: usize,
    /// Idle workers
    pub idle_workers: usize,
    /// Busy workers
    pub busy_workers: usize,
    /// Failed workers
    pub failed_workers: usize,
    /// Total capacity (max concurrent tasks)
    pub total_capacity: usize,
    /// Used capacity (current tasks)
    pub used_capacity: usize,
    /// Available capacity
    pub available_capacity: usize,
    /// Cluster load (0.0-1.0)
    pub cluster_load: f64,
    /// Total jobs submitted
    pub total_jobs: u64,
    /// Jobs completed
    pub completed_jobs: u64,
    /// Jobs failed
    pub failed_jobs: u64,
    /// Total tasks executed
    pub total_tasks: u64,
    /// Cluster uptime (seconds)
    pub uptime_seconds: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_worker(id: &str, capacity: usize, current_tasks: usize) -> WorkerInfo {
        WorkerInfo {
            id: id.to_string(),
            address: "localhost:50052".to_string(),
            status: if current_tasks == 0 {
                WorkerStatus::Idle
            } else {
                WorkerStatus::Busy
            },
            capacity,
            current_tasks,
            completed_tasks: 0,
            failed_tasks: 0,
            tags: vec![],
            metadata: std::collections::HashMap::new(),
            last_heartbeat: Utc::now(),
            registered_at: Utc::now(),
        }
    }

    #[test]
    fn test_cluster_state() {
        let cluster = ClusterState::new();

        let worker = create_test_worker("worker-1", 10, 0);
        cluster.register_worker(worker);

        assert_eq!(cluster.list_workers().len(), 1);

        let worker = cluster.get_worker(&"worker-1".to_string()).unwrap();
        assert_eq!(worker.id, "worker-1");

        cluster.deregister_worker(&"worker-1".to_string());
        assert_eq!(cluster.list_workers().len(), 0);
    }

    #[test]
    fn test_least_loaded_worker() {
        let cluster = ClusterState::new();

        cluster.register_worker(create_test_worker("worker-1", 10, 5));
        cluster.register_worker(create_test_worker("worker-2", 10, 2));
        cluster.register_worker(create_test_worker("worker-3", 10, 8));

        let worker = cluster.get_least_loaded_worker().unwrap();
        assert_eq!(worker.id, "worker-2");
    }

    #[test]
    fn test_worker_task_management() {
        let cluster = ClusterState::new();

        let worker = create_test_worker("worker-1", 10, 0);
        cluster.register_worker(worker);

        cluster.increment_worker_tasks(&"worker-1".to_string());

        let worker = cluster.get_worker(&"worker-1".to_string()).unwrap();
        assert_eq!(worker.current_tasks, 1);
        assert_eq!(worker.status, WorkerStatus::Busy);

        cluster.decrement_worker_tasks(&"worker-1".to_string(), true);

        let worker = cluster.get_worker(&"worker-1".to_string()).unwrap();
        assert_eq!(worker.current_tasks, 0);
        assert_eq!(worker.completed_tasks, 1);
        assert_eq!(worker.status, WorkerStatus::Idle);
    }

    #[test]
    fn test_cluster_metrics() {
        let cluster = ClusterState::new();

        cluster.register_worker(create_test_worker("worker-1", 10, 5));
        cluster.register_worker(create_test_worker("worker-2", 10, 2));

        let metrics = cluster.metrics();
        assert_eq!(metrics.total_workers, 2);
        assert_eq!(metrics.total_capacity, 20);
        assert_eq!(metrics.used_capacity, 7);
        assert_eq!(metrics.available_capacity, 13);
        assert!((metrics.cluster_load - 0.35).abs() < 0.01);
    }
}

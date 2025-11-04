// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Health monitoring and checking.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::interval;
use tracing::{debug, warn, info};

use crate::distributed::cluster::ClusterState;
use crate::distributed::types::NodeId;

/// Health status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HealthStatus {
    /// Healthy
    Healthy,
    /// Degraded performance
    Degraded,
    /// Unhealthy
    Unhealthy,
}

/// Health check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    /// Worker ID
    pub worker_id: NodeId,
    /// Health status
    pub status: HealthStatus,
    /// Response time (milliseconds)
    pub response_time_ms: u64,
    /// CPU usage (0.0-1.0)
    pub cpu_usage: f64,
    /// Memory usage (0.0-1.0)
    pub memory_usage: f64,
    /// Current tasks
    pub current_tasks: usize,
    /// Error message
    pub error: Option<String>,
    /// Checked at
    pub checked_at: DateTime<Utc>,
}

/// Health monitor
pub struct HealthMonitor {
    /// Cluster state
    cluster: Arc<ClusterState>,
    /// Health check interval (seconds)
    check_interval: u64,
    /// Health check timeout (seconds)
    check_timeout: u64,
    /// Unhealthy threshold
    unhealthy_threshold: u64,
}

impl HealthMonitor {
    /// Create a new health monitor
    pub fn new(
        cluster: Arc<ClusterState>,
        check_interval: u64,
        check_timeout: u64,
        unhealthy_threshold: u64,
    ) -> Self {
        Self {
            cluster,
            check_interval,
            check_timeout,
            unhealthy_threshold,
        }
    }

    /// Start health monitoring
    pub async fn start(self: Arc<Self>) {
        let mut ticker = interval(Duration::from_secs(self.check_interval));

        loop {
            ticker.tick().await;
            self.check_all_workers().await;
        }
    }

    /// Check all workers
    async fn check_all_workers(&self) {
        let workers = self.cluster.list_workers();

        debug!("Checking health of {} workers", workers.len());

        for worker in workers {
            // Check if worker is healthy based on last heartbeat
            if !worker.is_healthy(self.check_timeout) {
                warn!(
                    "Worker {} is unhealthy (last heartbeat: {:?})",
                    worker.id, worker.last_heartbeat
                );

                // Mark as failed
                self.cluster.mark_worker_failed(&worker.id);
            }
        }

        // Remove workers that haven't sent heartbeat in a long time
        let removed = self.cluster.remove_unhealthy_workers(self.unhealthy_threshold);

        if !removed.is_empty() {
            info!("Removed {} unhealthy workers: {:?}", removed.len(), removed);
        }
    }

    /// Check specific worker
    pub async fn check_worker(&self, worker_id: &NodeId) -> Option<HealthCheckResult> {
        let worker = self.cluster.get_worker(worker_id)?;

        let status = if worker.is_healthy(self.check_timeout) {
            HealthStatus::Healthy
        } else if worker.is_healthy(self.unhealthy_threshold) {
            HealthStatus::Degraded
        } else {
            HealthStatus::Unhealthy
        };

        let now = Utc::now();
        let response_time = now
            .signed_duration_since(worker.last_heartbeat)
            .num_milliseconds() as u64;

        Some(HealthCheckResult {
            worker_id: worker_id.clone(),
            status,
            response_time_ms: response_time,
            cpu_usage: 0.0, // Would be reported by worker
            memory_usage: 0.0, // Would be reported by worker
            current_tasks: worker.current_tasks,
            error: None,
            checked_at: now,
        })
    }

    /// Get cluster health status
    pub fn cluster_health(&self) -> ClusterHealthStatus {
        let metrics = self.cluster.metrics();

        let healthy_workers = self.cluster.list_workers()
            .iter()
            .filter(|w| w.is_healthy(self.check_timeout))
            .count();

        let degraded_workers = self.cluster.list_workers()
            .iter()
            .filter(|w| {
                !w.is_healthy(self.check_timeout)
                    && w.is_healthy(self.unhealthy_threshold)
            })
            .count();

        let unhealthy_workers = metrics.total_workers
            - healthy_workers
            - degraded_workers;

        let overall_status = if unhealthy_workers > 0 {
            HealthStatus::Unhealthy
        } else if degraded_workers > 0 || metrics.cluster_load > 0.9 {
            HealthStatus::Degraded
        } else {
            HealthStatus::Healthy
        };

        ClusterHealthStatus {
            overall_status,
            healthy_workers,
            degraded_workers,
            unhealthy_workers,
            total_workers: metrics.total_workers,
            cluster_load: metrics.cluster_load,
            checked_at: Utc::now(),
        }
    }
}

/// Cluster health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterHealthStatus {
    /// Overall cluster health
    pub overall_status: HealthStatus,
    /// Number of healthy workers
    pub healthy_workers: usize,
    /// Number of degraded workers
    pub degraded_workers: usize,
    /// Number of unhealthy workers
    pub unhealthy_workers: usize,
    /// Total workers
    pub total_workers: usize,
    /// Cluster load (0.0-1.0)
    pub cluster_load: f64,
    /// Checked at
    pub checked_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::distributed::types::{WorkerInfo, WorkerStatus};
    use std::collections::HashMap;

    #[test]
    fn test_health_monitor_creation() {
        let cluster = Arc::new(ClusterState::new());
        let monitor = HealthMonitor::new(cluster, 5, 10, 30);

        assert_eq!(monitor.check_interval, 5);
        assert_eq!(monitor.check_timeout, 10);
        assert_eq!(monitor.unhealthy_threshold, 30);
    }

    #[test]
    fn test_cluster_health() {
        let cluster = Arc::new(ClusterState::new());

        // Add healthy worker
        let worker = WorkerInfo {
            id: "worker-1".to_string(),
            address: "localhost:50052".to_string(),
            status: WorkerStatus::Idle,
            capacity: 10,
            current_tasks: 0,
            completed_tasks: 0,
            failed_tasks: 0,
            tags: vec![],
            metadata: HashMap::new(),
            last_heartbeat: Utc::now(),
            registered_at: Utc::now(),
        };

        cluster.register_worker(worker);

        let monitor = HealthMonitor::new(cluster, 5, 10, 30);
        let health = monitor.cluster_health();

        assert_eq!(health.total_workers, 1);
        assert_eq!(health.healthy_workers, 1);
        assert_eq!(health.overall_status, HealthStatus::Healthy);
    }
}

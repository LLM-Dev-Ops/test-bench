// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Job management and scheduling.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, VecDeque};
use std::sync::Arc;
use parking_lot::RwLock;

use crate::distributed::types::{JobId, TaskId, TaskResult};
use crate::distributed::protocol::JobRequest;

/// Job priority (higher = more important)
pub type JobPriority = i32;

/// Job status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobStatus {
    /// Job is pending execution
    Pending,
    /// Job is currently running
    Running,
    /// Job completed successfully
    Completed,
    /// Job failed
    Failed,
    /// Job was cancelled
    Cancelled,
}

impl std::fmt::Display for JobStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "pending"),
            Self::Running => write!(f, "running"),
            Self::Completed => write!(f, "completed"),
            Self::Failed => write!(f, "failed"),
            Self::Cancelled => write!(f, "cancelled"),
        }
    }
}

/// Job information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    /// Job ID
    pub id: JobId,
    /// Job type
    pub job_type: String,
    /// Job payload
    pub payload: serde_json::Value,
    /// Job priority
    pub priority: JobPriority,
    /// Required worker tags
    pub required_tags: Vec<String>,
    /// Job metadata
    pub metadata: HashMap<String, String>,
    /// Job status
    pub status: JobStatus,
    /// Job result
    pub result: Option<serde_json::Value>,
    /// Job error
    pub error: Option<String>,
    /// Timeout (seconds)
    pub timeout_seconds: u64,
    /// Max retries
    pub max_retries: u32,
    /// Current retry count
    pub retry_count: u32,
    /// Tasks in this job
    pub tasks: Vec<TaskId>,
    /// Completed tasks
    pub completed_tasks: usize,
    /// Failed tasks
    pub failed_tasks: usize,
    /// Created at
    pub created_at: DateTime<Utc>,
    /// Started at
    pub started_at: Option<DateTime<Utc>>,
    /// Completed at
    pub completed_at: Option<DateTime<Utc>>,
}

impl Job {
    /// Create a new job from request
    pub fn from_request(id: JobId, request: JobRequest) -> Self {
        Self {
            id,
            job_type: request.job_type,
            payload: request.payload,
            priority: request.priority,
            required_tags: request.required_tags,
            metadata: request.metadata,
            status: JobStatus::Pending,
            result: None,
            error: None,
            timeout_seconds: request.timeout_seconds,
            max_retries: request.max_retries,
            retry_count: 0,
            tasks: Vec::new(),
            completed_tasks: 0,
            failed_tasks: 0,
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
        }
    }

    /// Get job progress (0.0 to 1.0)
    pub fn progress(&self) -> f64 {
        if self.tasks.is_empty() {
            0.0
        } else {
            self.completed_tasks as f64 / self.tasks.len() as f64
        }
    }

    /// Check if job is complete
    pub fn is_complete(&self) -> bool {
        !self.tasks.is_empty() && self.completed_tasks == self.tasks.len()
    }

    /// Mark job as running
    pub fn mark_running(&mut self) {
        self.status = JobStatus::Running;
        self.started_at = Some(Utc::now());
    }

    /// Mark job as completed
    pub fn mark_completed(&mut self, result: serde_json::Value) {
        self.status = JobStatus::Completed;
        self.result = Some(result);
        self.completed_at = Some(Utc::now());
    }

    /// Mark job as failed
    pub fn mark_failed(&mut self, error: String) {
        self.status = JobStatus::Failed;
        self.error = Some(error);
        self.completed_at = Some(Utc::now());
    }

    /// Mark job as cancelled
    pub fn mark_cancelled(&mut self, reason: String) {
        self.status = JobStatus::Cancelled;
        self.error = Some(reason);
        self.completed_at = Some(Utc::now());
    }
}

/// Priority queue entry
#[derive(Debug, Clone)]
struct PriorityJob {
    job_id: JobId,
    priority: JobPriority,
    created_at: DateTime<Utc>,
}

impl PartialEq for PriorityJob {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority && self.created_at == other.created_at
    }
}

impl Eq for PriorityJob {}

impl PartialOrd for PriorityJob {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PriorityJob {
    fn cmp(&self, other: &Self) -> Ordering {
        // Higher priority first
        match self.priority.cmp(&other.priority) {
            Ordering::Equal => {
                // Earlier created_at first (FIFO for same priority)
                other.created_at.cmp(&self.created_at)
            }
            other => other,
        }
    }
}

/// Job queue with priority scheduling
pub struct JobQueue {
    /// Priority queue
    queue: Arc<RwLock<BinaryHeap<PriorityJob>>>,
    /// Job storage
    jobs: Arc<RwLock<HashMap<JobId, Job>>>,
    /// Running jobs
    running: Arc<RwLock<HashMap<JobId, Job>>>,
    /// Completed jobs (limited size)
    completed: Arc<RwLock<VecDeque<Job>>>,
    /// Max completed jobs to keep
    max_completed: usize,
}

impl JobQueue {
    /// Create a new job queue
    pub fn new(max_completed: usize) -> Self {
        Self {
            queue: Arc::new(RwLock::new(BinaryHeap::new())),
            jobs: Arc::new(RwLock::new(HashMap::new())),
            running: Arc::new(RwLock::new(HashMap::new())),
            completed: Arc::new(RwLock::new(VecDeque::new())),
            max_completed,
        }
    }

    /// Submit a job to the queue
    pub fn submit(&self, job: Job) -> JobId {
        let job_id = job.id.clone();
        let priority = job.priority;
        let created_at = job.created_at;

        // Store job
        self.jobs.write().insert(job_id.clone(), job);

        // Add to priority queue
        self.queue.write().push(PriorityJob {
            job_id: job_id.clone(),
            priority,
            created_at,
        });

        job_id
    }

    /// Get next job from queue
    pub fn next(&self) -> Option<Job> {
        let mut queue = self.queue.write();

        while let Some(pj) = queue.pop() {
            let mut jobs = self.jobs.write();

            if let Some(mut job) = jobs.remove(&pj.job_id) {
                job.mark_running();
                let job_id = job.id.clone();
                let job_clone = job.clone();

                // Move to running
                self.running.write().insert(job_id, job);

                return Some(job_clone);
            }
        }

        None
    }

    /// Get job by ID
    pub fn get(&self, job_id: &JobId) -> Option<Job> {
        // Check pending
        if let Some(job) = self.jobs.read().get(job_id) {
            return Some(job.clone());
        }

        // Check running
        if let Some(job) = self.running.read().get(job_id) {
            return Some(job.clone());
        }

        // Check completed
        self.completed
            .read()
            .iter()
            .find(|j| &j.id == job_id)
            .cloned()
    }

    /// Update job status
    pub fn update(&self, job_id: &JobId, update_fn: impl FnOnce(&mut Job)) {
        if let Some(job) = self.running.write().get_mut(job_id) {
            update_fn(job);
        }
    }

    /// Complete a job
    pub fn complete(&self, job_id: &JobId, result: serde_json::Value) {
        if let Some(mut job) = self.running.write().remove(job_id) {
            job.mark_completed(result);

            // Add to completed queue
            let mut completed = self.completed.write();
            completed.push_back(job);

            // Trim if needed
            while completed.len() > self.max_completed {
                completed.pop_front();
            }
        }
    }

    /// Fail a job
    pub fn fail(&self, job_id: &JobId, error: String) {
        if let Some(mut job) = self.running.write().remove(job_id) {
            job.mark_failed(error);

            // Add to completed queue
            let mut completed = self.completed.write();
            completed.push_back(job);

            // Trim if needed
            while completed.len() > self.max_completed {
                completed.pop_front();
            }
        }
    }

    /// Cancel a job
    pub fn cancel(&self, job_id: &JobId, reason: String) -> bool {
        // Try to remove from pending
        let mut jobs = self.jobs.write();
        if let Some(mut job) = jobs.remove(job_id) {
            job.mark_cancelled(reason);

            let mut completed = self.completed.write();
            completed.push_back(job);

            while completed.len() > self.max_completed {
                completed.pop_front();
            }

            return true;
        }
        drop(jobs);

        // Try to remove from running
        if let Some(mut job) = self.running.write().remove(job_id) {
            job.mark_cancelled(reason);

            let mut completed = self.completed.write();
            completed.push_back(job);

            while completed.len() > self.max_completed {
                completed.pop_front();
            }

            return true;
        }

        false
    }

    /// Get queue statistics
    pub fn stats(&self) -> QueueStats {
        let pending = self.jobs.read().len();
        let running = self.running.read().len();
        let completed = self.completed.read().len();

        QueueStats {
            pending_jobs: pending,
            running_jobs: running,
            completed_jobs: completed,
        }
    }

    /// List all pending jobs
    pub fn list_pending(&self) -> Vec<Job> {
        self.jobs.read().values().cloned().collect()
    }

    /// List all running jobs
    pub fn list_running(&self) -> Vec<Job> {
        self.running.read().values().cloned().collect()
    }
}

/// Queue statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueStats {
    pub pending_jobs: usize,
    pub running_jobs: usize,
    pub completed_jobs: usize,
}

impl Default for JobQueue {
    fn default() -> Self {
        Self::new(1000)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_job_creation() {
        let request = JobRequest::builder()
            .job_type("benchmark")
            .payload(serde_json::json!({"test": "data"}))
            .priority(10)
            .build();

        let job = Job::from_request("job-1".to_string(), request);

        assert_eq!(job.id, "job-1");
        assert_eq!(job.job_type, "benchmark");
        assert_eq!(job.priority, 10);
        assert_eq!(job.status, JobStatus::Pending);
    }

    #[test]
    fn test_job_progress() {
        let mut job = Job::from_request(
            "job-1".to_string(),
            JobRequest::builder().build(),
        );

        assert_eq!(job.progress(), 0.0);

        job.tasks = vec!["task-1".to_string(), "task-2".to_string()];
        assert_eq!(job.progress(), 0.0);

        job.completed_tasks = 1;
        assert_eq!(job.progress(), 0.5);

        job.completed_tasks = 2;
        assert_eq!(job.progress(), 1.0);
        assert!(job.is_complete());
    }

    #[test]
    fn test_job_queue() {
        let queue = JobQueue::new(100);

        // Submit jobs with different priorities
        let job1 = Job::from_request(
            "job-1".to_string(),
            JobRequest::builder().priority(5).build(),
        );
        let job2 = Job::from_request(
            "job-2".to_string(),
            JobRequest::builder().priority(10).build(),
        );
        let job3 = Job::from_request(
            "job-3".to_string(),
            JobRequest::builder().priority(1).build(),
        );

        queue.submit(job1);
        queue.submit(job2);
        queue.submit(job3);

        // Should get highest priority first
        let next = queue.next().unwrap();
        assert_eq!(next.id, "job-2");
        assert_eq!(next.priority, 10);

        let next = queue.next().unwrap();
        assert_eq!(next.id, "job-1");
        assert_eq!(next.priority, 5);

        let next = queue.next().unwrap();
        assert_eq!(next.id, "job-3");
        assert_eq!(next.priority, 1);

        assert!(queue.next().is_none());
    }

    #[test]
    fn test_job_completion() {
        let queue = JobQueue::new(100);

        let job = Job::from_request(
            "job-1".to_string(),
            JobRequest::builder().build(),
        );
        queue.submit(job);

        let job = queue.next().unwrap();
        queue.complete(&job.id, serde_json::json!({"result": "success"}));

        let completed = queue.get(&"job-1".to_string()).unwrap();
        assert_eq!(completed.status, JobStatus::Completed);
        assert!(completed.result.is_some());
    }
}

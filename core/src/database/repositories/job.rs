// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Job repository.

use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::database::{
    error::DatabaseResult,
    models::{JobRecord, NewJob, UpdateJob},
};

/// Job repository
pub struct JobRepository {
    pool: PgPool,
}

impl JobRepository {
    /// Create a new job repository
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Create a new job
    pub async fn create(&self, job: NewJob) -> DatabaseResult<JobRecord> {
        let record = sqlx::query_as!(
            JobRecord,
            r#"
            INSERT INTO jobs (
                id, job_type, payload, priority, status,
                retry_count, max_retries, timeout_seconds,
                created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING *
            "#,
            Uuid::new_v4(),
            job.job_type,
            job.payload,
            job.priority,
            "pending",
            0,
            job.max_retries,
            job.timeout_seconds,
            Utc::now(),
            Utc::now(),
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(record)
    }

    /// Get job by ID
    pub async fn get(&self, id: Uuid) -> DatabaseResult<JobRecord> {
        let record = sqlx::query_as!(
            JobRecord,
            r#"
            SELECT * FROM jobs WHERE id = $1
            "#,
            id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(record)
    }

    /// Update job
    pub async fn update(&self, id: Uuid, update: UpdateJob) -> DatabaseResult<JobRecord> {
        let record = sqlx::query_as!(
            JobRecord,
            r#"
            UPDATE jobs
            SET updated_at = $1,
                status = COALESCE($2, status),
                result = COALESCE($3, result),
                error = COALESCE($4, error),
                retry_count = COALESCE($5, retry_count),
                assigned_worker_id = COALESCE($6, assigned_worker_id),
                started_at = COALESCE($7, started_at),
                completed_at = COALESCE($8, completed_at)
            WHERE id = $9
            RETURNING *
            "#,
            Utc::now(),
            update.status,
            update.result,
            update.error,
            update.retry_count,
            update.assigned_worker_id,
            update.started_at,
            update.completed_at,
            id,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(record)
    }

    /// Delete job
    pub async fn delete(&self, id: Uuid) -> DatabaseResult<()> {
        sqlx::query!(
            r#"
            DELETE FROM jobs WHERE id = $1
            "#,
            id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// List pending jobs with priority order
    pub async fn list_pending(&self, limit: i64) -> DatabaseResult<Vec<JobRecord>> {
        let records = sqlx::query_as!(
            JobRecord,
            r#"
            SELECT * FROM jobs
            WHERE status = 'pending'
            ORDER BY priority DESC, created_at ASC
            LIMIT $1
            "#,
            limit
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(records)
    }

    /// List running jobs
    pub async fn list_running(&self, limit: i64) -> DatabaseResult<Vec<JobRecord>> {
        let records = sqlx::query_as!(
            JobRecord,
            r#"
            SELECT * FROM jobs
            WHERE status = 'running'
            ORDER BY started_at DESC
            LIMIT $1
            "#,
            limit
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(records)
    }

    /// List jobs by status
    pub async fn list_by_status(&self, status: &str, limit: i64) -> DatabaseResult<Vec<JobRecord>> {
        let records = sqlx::query_as!(
            JobRecord,
            r#"
            SELECT * FROM jobs
            WHERE status = $1
            ORDER BY created_at DESC
            LIMIT $2
            "#,
            status,
            limit
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(records)
    }

    /// List jobs by worker
    pub async fn list_by_worker(&self, worker_id: Uuid, limit: i64) -> DatabaseResult<Vec<JobRecord>> {
        let records = sqlx::query_as!(
            JobRecord,
            r#"
            SELECT * FROM jobs
            WHERE assigned_worker_id = $1
            ORDER BY created_at DESC
            LIMIT $2
            "#,
            worker_id,
            limit
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(records)
    }

    /// List jobs that need retry
    pub async fn list_retry_candidates(&self, limit: i64) -> DatabaseResult<Vec<JobRecord>> {
        let records = sqlx::query_as!(
            JobRecord,
            r#"
            SELECT * FROM jobs
            WHERE status = 'failed'
              AND retry_count < max_retries
            ORDER BY priority DESC, created_at ASC
            LIMIT $1
            "#,
            limit
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(records)
    }

    /// Count jobs by status
    pub async fn count_by_status(&self, status: &str) -> DatabaseResult<i64> {
        let result = sqlx::query!(
            r#"
            SELECT COUNT(*) as count FROM jobs WHERE status = $1
            "#,
            status
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(result.count.unwrap_or(0))
    }

    /// Get job statistics
    pub async fn get_stats(&self) -> DatabaseResult<JobStats> {
        let result = sqlx::query!(
            r#"
            SELECT
                COUNT(*) FILTER (WHERE status = 'pending') as pending,
                COUNT(*) FILTER (WHERE status = 'running') as running,
                COUNT(*) FILTER (WHERE status = 'completed') as completed,
                COUNT(*) FILTER (WHERE status = 'failed') as failed,
                COUNT(*) FILTER (WHERE status = 'cancelled') as cancelled
            FROM jobs
            "#
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(JobStats {
            pending: result.pending.unwrap_or(0),
            running: result.running.unwrap_or(0),
            completed: result.completed.unwrap_or(0),
            failed: result.failed.unwrap_or(0),
            cancelled: result.cancelled.unwrap_or(0),
        })
    }

    /// Clean up old completed jobs
    pub async fn cleanup_old_jobs(&self, days: i32) -> DatabaseResult<u64> {
        let result = sqlx::query!(
            r#"
            DELETE FROM jobs
            WHERE status IN ('completed', 'failed', 'cancelled')
              AND completed_at < NOW() - INTERVAL '1 day' * $1
            "#,
            days
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }
}

/// Job statistics
#[derive(Debug, Clone)]
pub struct JobStats {
    pub pending: i64,
    pub running: i64,
    pub completed: i64,
    pub failed: i64,
    pub cancelled: i64,
}

impl JobStats {
    pub fn total(&self) -> i64 {
        self.pending + self.running + self.completed + self.failed + self.cancelled
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_job_stats() {
        let stats = JobStats {
            pending: 10,
            running: 5,
            completed: 100,
            failed: 2,
            cancelled: 1,
        };

        assert_eq!(stats.total(), 118);
    }
}

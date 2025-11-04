// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Worker repository.

use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::database::{
    error::DatabaseResult,
    models::{WorkerRecord, NewWorker, UpdateWorker},
};

/// Worker repository
pub struct WorkerRepository {
    pool: PgPool,
}

impl WorkerRepository {
    /// Create a new worker repository
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Create a new worker
    pub async fn create(&self, worker: NewWorker) -> DatabaseResult<WorkerRecord> {
        let record = sqlx::query_as!(
            WorkerRecord,
            r#"
            INSERT INTO workers (
                id, worker_id, address, status, capacity,
                current_tasks, completed_tasks, failed_tasks,
                tags, metadata, last_heartbeat, registered_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            RETURNING *
            "#,
            Uuid::new_v4(),
            worker.worker_id,
            worker.address,
            "idle",
            worker.capacity,
            0,
            0_i64,
            0_i64,
            &worker.tags,
            worker.metadata,
            Utc::now(),
            Utc::now(),
            Utc::now(),
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(record)
    }

    /// Get worker by ID
    pub async fn get(&self, id: Uuid) -> DatabaseResult<WorkerRecord> {
        let record = sqlx::query_as!(
            WorkerRecord,
            r#"
            SELECT * FROM workers WHERE id = $1
            "#,
            id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(record)
    }

    /// Get worker by worker_id
    pub async fn get_by_worker_id(&self, worker_id: &str) -> DatabaseResult<WorkerRecord> {
        let record = sqlx::query_as!(
            WorkerRecord,
            r#"
            SELECT * FROM workers WHERE worker_id = $1
            "#,
            worker_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(record)
    }

    /// Update worker
    pub async fn update(&self, id: Uuid, update: UpdateWorker) -> DatabaseResult<WorkerRecord> {
        let record = sqlx::query_as!(
            WorkerRecord,
            r#"
            UPDATE workers
            SET updated_at = $1,
                status = COALESCE($2, status),
                current_tasks = COALESCE($3, current_tasks),
                completed_tasks = COALESCE($4, completed_tasks),
                failed_tasks = COALESCE($5, failed_tasks),
                last_heartbeat = COALESCE($6, last_heartbeat)
            WHERE id = $7
            RETURNING *
            "#,
            Utc::now(),
            update.status,
            update.current_tasks,
            update.completed_tasks,
            update.failed_tasks,
            update.last_heartbeat,
            id,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(record)
    }

    /// Update worker heartbeat
    pub async fn update_heartbeat(&self, worker_id: &str) -> DatabaseResult<()> {
        sqlx::query!(
            r#"
            UPDATE workers
            SET last_heartbeat = $1, updated_at = $1
            WHERE worker_id = $2
            "#,
            Utc::now(),
            worker_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Delete worker
    pub async fn delete(&self, id: Uuid) -> DatabaseResult<()> {
        sqlx::query!(
            r#"
            DELETE FROM workers WHERE id = $1
            "#,
            id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// List all workers
    pub async fn list_all(&self) -> DatabaseResult<Vec<WorkerRecord>> {
        let records = sqlx::query_as!(
            WorkerRecord,
            r#"
            SELECT * FROM workers
            ORDER BY registered_at DESC
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(records)
    }

    /// List workers by status
    pub async fn list_by_status(&self, status: &str) -> DatabaseResult<Vec<WorkerRecord>> {
        let records = sqlx::query_as!(
            WorkerRecord,
            r#"
            SELECT * FROM workers
            WHERE status = $1
            ORDER BY registered_at DESC
            "#,
            status
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(records)
    }

    /// List active workers (healthy heartbeat)
    pub async fn list_active(&self, timeout_seconds: i64) -> DatabaseResult<Vec<WorkerRecord>> {
        let records = sqlx::query_as!(
            WorkerRecord,
            r#"
            SELECT * FROM workers
            WHERE last_heartbeat > NOW() - INTERVAL '1 second' * $1
              AND status IN ('idle', 'busy')
            ORDER BY last_heartbeat DESC
            "#,
            timeout_seconds
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(records)
    }

    /// Remove stale workers
    pub async fn remove_stale(&self, timeout_seconds: i64) -> DatabaseResult<u64> {
        let result = sqlx::query!(
            r#"
            DELETE FROM workers
            WHERE last_heartbeat < NOW() - INTERVAL '1 second' * $1
            "#,
            timeout_seconds
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }
}

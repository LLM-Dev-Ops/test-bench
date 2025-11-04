// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Benchmark repository.

use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::database::{
    error::DatabaseResult,
    models::{BenchmarkRecord, NewBenchmark, UpdateBenchmark},
};

/// Benchmark repository
pub struct BenchmarkRepository {
    pool: PgPool,
}

impl BenchmarkRepository {
    /// Create a new benchmark repository
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Create a new benchmark
    pub async fn create(&self, benchmark: NewBenchmark) -> DatabaseResult<BenchmarkRecord> {
        let record = sqlx::query_as!(
            BenchmarkRecord,
            r#"
            INSERT INTO benchmarks (
                id, name, description, provider, model, dataset,
                status, total_iterations, completed_iterations, failed_iterations,
                created_at, updated_at, created_by
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            RETURNING *
            "#,
            Uuid::new_v4(),
            benchmark.name,
            benchmark.description,
            benchmark.provider,
            benchmark.model,
            benchmark.dataset,
            "pending",
            benchmark.total_iterations,
            0,
            0,
            Utc::now(),
            Utc::now(),
            benchmark.created_by,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(record)
    }

    /// Get benchmark by ID
    pub async fn get(&self, id: Uuid) -> DatabaseResult<BenchmarkRecord> {
        let record = sqlx::query_as!(
            BenchmarkRecord,
            r#"
            SELECT * FROM benchmarks WHERE id = $1
            "#,
            id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(record)
    }

    /// Update benchmark
    pub async fn update(&self, id: Uuid, update: UpdateBenchmark) -> DatabaseResult<BenchmarkRecord> {
        let mut query = String::from("UPDATE benchmarks SET updated_at = $1");
        let mut param_count = 1;
        let mut params: Vec<String> = vec![];

        if let Some(status) = &update.status {
            param_count += 1;
            query.push_str(&format!(", status = ${}", param_count));
            params.push(status.clone());
        }

        if let Some(completed) = update.completed_iterations {
            param_count += 1;
            query.push_str(&format!(", completed_iterations = ${}", param_count));
            params.push(completed.to_string());
        }

        if let Some(failed) = update.failed_iterations {
            param_count += 1;
            query.push_str(&format!(", failed_iterations = ${}", param_count));
            params.push(failed.to_string());
        }

        query.push_str(&format!(" WHERE id = ${} RETURNING *", param_count + 1));

        // Note: This is a simplified version. In production, use a proper query builder
        // or sqlx macros with all parameters properly bound.

        let record = sqlx::query_as!(
            BenchmarkRecord,
            r#"
            UPDATE benchmarks
            SET updated_at = $1,
                status = COALESCE($2, status),
                completed_iterations = COALESCE($3, completed_iterations),
                failed_iterations = COALESCE($4, failed_iterations),
                results = COALESCE($5, results),
                metrics = COALESCE($6, metrics),
                error = COALESCE($7, error),
                started_at = COALESCE($8, started_at),
                completed_at = COALESCE($9, completed_at)
            WHERE id = $10
            RETURNING *
            "#,
            Utc::now(),
            update.status,
            update.completed_iterations,
            update.failed_iterations,
            update.results,
            update.metrics,
            update.error,
            update.started_at,
            update.completed_at,
            id,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(record)
    }

    /// Delete benchmark
    pub async fn delete(&self, id: Uuid) -> DatabaseResult<()> {
        sqlx::query!(
            r#"
            DELETE FROM benchmarks WHERE id = $1
            "#,
            id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// List benchmarks with pagination
    pub async fn list(&self, limit: i64, offset: i64) -> DatabaseResult<Vec<BenchmarkRecord>> {
        let records = sqlx::query_as!(
            BenchmarkRecord,
            r#"
            SELECT * FROM benchmarks
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(records)
    }

    /// List recent benchmarks
    pub async fn list_recent(&self, limit: i64) -> DatabaseResult<Vec<BenchmarkRecord>> {
        self.list(limit, 0).await
    }

    /// List benchmarks by status
    pub async fn list_by_status(&self, status: &str, limit: i64) -> DatabaseResult<Vec<BenchmarkRecord>> {
        let records = sqlx::query_as!(
            BenchmarkRecord,
            r#"
            SELECT * FROM benchmarks
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

    /// List benchmarks by provider
    pub async fn list_by_provider(&self, provider: &str, limit: i64) -> DatabaseResult<Vec<BenchmarkRecord>> {
        let records = sqlx::query_as!(
            BenchmarkRecord,
            r#"
            SELECT * FROM benchmarks
            WHERE provider = $1
            ORDER BY created_at DESC
            LIMIT $2
            "#,
            provider,
            limit
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(records)
    }

    /// Count benchmarks
    pub async fn count(&self) -> DatabaseResult<i64> {
        let result = sqlx::query!(
            r#"
            SELECT COUNT(*) as count FROM benchmarks
            "#
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(result.count.unwrap_or(0))
    }

    /// Count benchmarks by status
    pub async fn count_by_status(&self, status: &str) -> DatabaseResult<i64> {
        let result = sqlx::query!(
            r#"
            SELECT COUNT(*) as count FROM benchmarks WHERE status = $1
            "#,
            status
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(result.count.unwrap_or(0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Integration tests would go here
    // Requires a test database to be set up
}

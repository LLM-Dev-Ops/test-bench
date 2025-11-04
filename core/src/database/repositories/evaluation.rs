// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Evaluation repository.

use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::database::{
    error::DatabaseResult,
    models::{EvaluationRecord, NewEvaluation},
};

/// Evaluation repository
pub struct EvaluationRepository {
    pool: PgPool,
}

impl EvaluationRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, evaluation: NewEvaluation) -> DatabaseResult<EvaluationRecord> {
        let record = sqlx::query_as!(
            EvaluationRecord,
            r#"
            INSERT INTO evaluations (
                id, benchmark_id, provider, model, input, output,
                expected, metrics, score, metadata, created_at, created_by
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            RETURNING *
            "#,
            Uuid::new_v4(),
            evaluation.benchmark_id,
            evaluation.provider,
            evaluation.model,
            evaluation.input,
            evaluation.output,
            evaluation.expected,
            evaluation.metrics,
            evaluation.score,
            evaluation.metadata,
            Utc::now(),
            evaluation.created_by,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(record)
    }

    pub async fn get(&self, id: Uuid) -> DatabaseResult<EvaluationRecord> {
        let record = sqlx::query_as!(
            EvaluationRecord,
            "SELECT * FROM evaluations WHERE id = $1",
            id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(record)
    }

    pub async fn list_by_benchmark(&self, benchmark_id: Uuid, limit: i64) -> DatabaseResult<Vec<EvaluationRecord>> {
        let records = sqlx::query_as!(
            EvaluationRecord,
            "SELECT * FROM evaluations WHERE benchmark_id = $1 ORDER BY created_at DESC LIMIT $2",
            benchmark_id,
            limit
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(records)
    }

    pub async fn delete(&self, id: Uuid) -> DatabaseResult<()> {
        sqlx::query!("DELETE FROM evaluations WHERE id = $1", id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Audit log repository.

use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::database::{
    error::DatabaseResult,
    models::{AuditLogRecord, NewAuditLog},
};

/// Audit repository
pub struct AuditRepository {
    pool: PgPool,
}

impl AuditRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, log: NewAuditLog) -> DatabaseResult<AuditLogRecord> {
        let record = sqlx::query_as!(
            AuditLogRecord,
            r#"
            INSERT INTO audit_logs (
                id, user_id, action, entity_type, entity_id,
                changes, ip_address, user_agent, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING *
            "#,
            Uuid::new_v4(),
            log.user_id,
            log.action,
            log.entity_type,
            log.entity_id,
            log.changes,
            log.ip_address,
            log.user_agent,
            Utc::now(),
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(record)
    }

    pub async fn list_by_user(&self, user_id: Uuid, limit: i64) -> DatabaseResult<Vec<AuditLogRecord>> {
        let records = sqlx::query_as!(
            AuditLogRecord,
            "SELECT * FROM audit_logs WHERE user_id = $1 ORDER BY created_at DESC LIMIT $2",
            user_id,
            limit
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(records)
    }

    pub async fn list_by_entity(&self, entity_type: &str, entity_id: Uuid, limit: i64) -> DatabaseResult<Vec<AuditLogRecord>> {
        let records = sqlx::query_as!(
            AuditLogRecord,
            "SELECT * FROM audit_logs WHERE entity_type = $1 AND entity_id = $2 ORDER BY created_at DESC LIMIT $3",
            entity_type,
            entity_id,
            limit
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(records)
    }

    pub async fn cleanup_old_logs(&self, days: i32) -> DatabaseResult<u64> {
        let result = sqlx::query!(
            "DELETE FROM audit_logs WHERE created_at < NOW() - INTERVAL '1 day' * $1",
            days
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }
}

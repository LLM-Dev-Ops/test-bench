// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! User repository.

use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::database::{
    error::DatabaseResult,
    models::{UserRecord, NewUser, ApiKeyRecord, NewApiKey},
};

/// User repository
pub struct UserRepository {
    pool: PgPool,
}

impl UserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, user: NewUser) -> DatabaseResult<UserRecord> {
        let record = sqlx::query_as!(
            UserRecord,
            r#"
            INSERT INTO users (
                id, email, username, password_hash, role,
                active, email_verified, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING *
            "#,
            Uuid::new_v4(),
            user.email,
            user.username,
            user.password_hash,
            user.role,
            true,
            false,
            Utc::now(),
            Utc::now(),
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(record)
    }

    pub async fn get(&self, id: Uuid) -> DatabaseResult<UserRecord> {
        let record = sqlx::query_as!(
            UserRecord,
            "SELECT * FROM users WHERE id = $1",
            id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(record)
    }

    pub async fn get_by_email(&self, email: &str) -> DatabaseResult<UserRecord> {
        let record = sqlx::query_as!(
            UserRecord,
            "SELECT * FROM users WHERE email = $1",
            email
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(record)
    }

    pub async fn update_last_login(&self, id: Uuid) -> DatabaseResult<()> {
        sqlx::query!(
            "UPDATE users SET last_login = $1, updated_at = $1 WHERE id = $2",
            Utc::now(),
            id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn create_api_key(&self, key: NewApiKey) -> DatabaseResult<ApiKeyRecord> {
        let record = sqlx::query_as!(
            ApiKeyRecord,
            r#"
            INSERT INTO api_keys (
                id, user_id, key_hash, name, active, expires_at, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING *
            "#,
            Uuid::new_v4(),
            key.user_id,
            key.key_hash,
            key.name,
            true,
            key.expires_at,
            Utc::now(),
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(record)
    }

    pub async fn list_api_keys(&self, user_id: Uuid) -> DatabaseResult<Vec<ApiKeyRecord>> {
        let records = sqlx::query_as!(
            ApiKeyRecord,
            "SELECT * FROM api_keys WHERE user_id = $1 ORDER BY created_at DESC",
            user_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(records)
    }
}

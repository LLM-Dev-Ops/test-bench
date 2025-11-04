// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Database connection and pool management.

use sqlx::{postgres::PgPoolOptions, PgPool, Postgres};
use std::sync::Arc;
use tracing::{info, debug};

use crate::database::{
    config::DatabaseConfig,
    error::{DatabaseError, DatabaseResult},
    repositories::*,
};

/// Database connection with repository access
#[derive(Clone)]
pub struct Database {
    pool: PgPool,
    benchmarks: Arc<BenchmarkRepository>,
    evaluations: Arc<EvaluationRepository>,
    jobs: Arc<JobRepository>,
    workers: Arc<WorkerRepository>,
    users: Arc<UserRepository>,
    audit: Arc<AuditRepository>,
}

impl Database {
    /// Connect to the database
    pub async fn connect(config: DatabaseConfig) -> DatabaseResult<Self> {
        info!("Connecting to database: {}", config.connection_url_safe());

        let pool = PgPoolOptions::new()
            .max_connections(config.pool_size)
            .acquire_timeout(config.connect_timeout_duration())
            .idle_timeout(config.idle_timeout_duration())
            .max_lifetime(config.max_lifetime_duration())
            .connect(&config.connection_url())
            .await
            .map_err(|e| DatabaseError::ConnectionError(e.to_string()))?;

        info!("Database connected successfully");

        // Create repositories
        let benchmarks = Arc::new(BenchmarkRepository::new(pool.clone()));
        let evaluations = Arc::new(EvaluationRepository::new(pool.clone()));
        let jobs = Arc::new(JobRepository::new(pool.clone()));
        let workers = Arc::new(WorkerRepository::new(pool.clone()));
        let users = Arc::new(UserRepository::new(pool.clone()));
        let audit = Arc::new(AuditRepository::new(pool.clone()));

        Ok(Self {
            pool,
            benchmarks,
            evaluations,
            jobs,
            workers,
            users,
            audit,
        })
    }

    /// Run database migrations
    pub async fn migrate(&self) -> DatabaseResult<()> {
        info!("Running database migrations");

        sqlx::migrate!("./migrations")
            .run(&self.pool)
            .await?;

        info!("Database migrations completed");

        Ok(())
    }

    /// Get the connection pool
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Get benchmark repository
    pub fn benchmarks(&self) -> &BenchmarkRepository {
        &self.benchmarks
    }

    /// Get evaluation repository
    pub fn evaluations(&self) -> &EvaluationRepository {
        &self.evaluations
    }

    /// Get job repository
    pub fn jobs(&self) -> &JobRepository {
        &self.jobs
    }

    /// Get worker repository
    pub fn workers(&self) -> &WorkerRepository {
        &self.workers
    }

    /// Get user repository
    pub fn users(&self) -> &UserRepository {
        &self.users
    }

    /// Get audit repository
    pub fn audit(&self) -> &AuditRepository {
        &self.audit
    }

    /// Check database health
    pub async fn health_check(&self) -> DatabaseResult<bool> {
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await?;

        Ok(true)
    }

    /// Get database statistics
    pub async fn stats(&self) -> DatabaseResult<DatabaseStats> {
        let row = sqlx::query!(
            r#"
            SELECT
                (SELECT COUNT(*) FROM benchmarks) as benchmark_count,
                (SELECT COUNT(*) FROM evaluations) as evaluation_count,
                (SELECT COUNT(*) FROM jobs) as job_count,
                (SELECT COUNT(*) FROM workers) as worker_count,
                (SELECT COUNT(*) FROM users) as user_count,
                (SELECT pg_database_size(current_database())) as database_size
            "#
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(DatabaseStats {
            benchmark_count: row.benchmark_count.unwrap_or(0),
            evaluation_count: row.evaluation_count.unwrap_or(0),
            job_count: row.job_count.unwrap_or(0),
            worker_count: row.worker_count.unwrap_or(0),
            user_count: row.user_count.unwrap_or(0),
            database_size_bytes: row.database_size.unwrap_or(0),
        })
    }

    /// Close the database connection
    pub async fn close(&self) {
        debug!("Closing database connection");
        self.pool.close().await;
    }
}

/// Database statistics
#[derive(Debug, Clone)]
pub struct DatabaseStats {
    pub benchmark_count: i64,
    pub evaluation_count: i64,
    pub job_count: i64,
    pub worker_count: i64,
    pub user_count: i64,
    pub database_size_bytes: i64,
}

impl DatabaseStats {
    /// Get database size in megabytes
    pub fn database_size_mb(&self) -> f64 {
        self.database_size_bytes as f64 / 1024.0 / 1024.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stats_size_calculation() {
        let stats = DatabaseStats {
            benchmark_count: 100,
            evaluation_count: 500,
            job_count: 1000,
            worker_count: 10,
            user_count: 50,
            database_size_bytes: 10485760, // 10 MB
        };

        assert_eq!(stats.database_size_mb(), 10.0);
    }
}

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Database error types.

use thiserror::Error;

/// Database result type
pub type DatabaseResult<T> = Result<T, DatabaseError>;

/// Database errors
#[derive(Debug, Error)]
pub enum DatabaseError {
    /// SQL execution error
    #[error("SQL error: {0}")]
    SqlError(#[from] sqlx::Error),

    /// Migration error
    #[error("Migration error: {0}")]
    MigrationError(#[from] sqlx::migrate::MigrateError),

    /// Not found error
    #[error("Not found: {0}")]
    NotFound(String),

    /// Duplicate key error
    #[error("Duplicate key: {0}")]
    DuplicateKey(String),

    /// Invalid input error
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Connection error
    #[error("Connection error: {0}")]
    ConnectionError(String),

    /// Transaction error
    #[error("Transaction error: {0}")]
    TransactionError(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Internal error
    #[error("Internal error: {0}")]
    InternalError(String),
}

impl DatabaseError {
    /// Check if error is a not found error
    pub fn is_not_found(&self) -> bool {
        matches!(self, Self::NotFound(_))
    }

    /// Check if error is a duplicate key error
    pub fn is_duplicate_key(&self) -> bool {
        matches!(self, Self::DuplicateKey(_))
    }

    /// Check if error is a connection error
    pub fn is_connection_error(&self) -> bool {
        matches!(self, Self::ConnectionError(_))
    }
}

/// Convert sqlx row not found to our NotFound error
impl From<sqlx::Error> for DatabaseError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => {
                Self::NotFound("Row not found".to_string())
            }
            sqlx::Error::Database(db_err) => {
                // Check for constraint violations
                if let Some(code) = db_err.code() {
                    if code == "23505" {  // Unique violation
                        return Self::DuplicateKey(db_err.message().to_string());
                    }
                }
                Self::SqlError(sqlx::Error::Database(db_err))
            }
            _ => Self::SqlError(err),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_types() {
        let err = DatabaseError::NotFound("test".to_string());
        assert!(err.is_not_found());
        assert!(!err.is_duplicate_key());

        let err = DatabaseError::DuplicateKey("test".to_string());
        assert!(err.is_duplicate_key());
        assert!(!err.is_not_found());
    }
}

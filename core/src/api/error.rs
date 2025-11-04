// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! API error handling.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use std::fmt;

use crate::api::models::ErrorResponse;

/// API result type
pub type ApiResult<T> = Result<T, ApiError>;

/// API error types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ApiError {
    /// Bad request (400)
    BadRequest(String),
    /// Unauthorized (401)
    Unauthorized(String),
    /// Forbidden (403)
    Forbidden(String),
    /// Not found (404)
    NotFound(String),
    /// Conflict (409)
    Conflict(String),
    /// Rate limit exceeded (429)
    RateLimitExceeded(String),
    /// Internal server error (500)
    InternalError(String),
    /// Service unavailable (503)
    ServiceUnavailable(String),
    /// Custom error with status code
    Custom { code: u16, message: String },
}

impl ApiError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::BadRequest(_) => StatusCode::BAD_REQUEST,
            Self::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            Self::Forbidden(_) => StatusCode::FORBIDDEN,
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::Conflict(_) => StatusCode::CONFLICT,
            Self::RateLimitExceeded(_) => StatusCode::TOO_MANY_REQUESTS,
            Self::InternalError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::ServiceUnavailable(_) => StatusCode::SERVICE_UNAVAILABLE,
            Self::Custom { code, .. } => StatusCode::from_u16(*code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
        }
    }

    pub fn error_code(&self) -> &str {
        match self {
            Self::BadRequest(_) => "BAD_REQUEST",
            Self::Unauthorized(_) => "UNAUTHORIZED",
            Self::Forbidden(_) => "FORBIDDEN",
            Self::NotFound(_) => "NOT_FOUND",
            Self::Conflict(_) => "CONFLICT",
            Self::RateLimitExceeded(_) => "RATE_LIMIT_EXCEEDED",
            Self::InternalError(_) => "INTERNAL_ERROR",
            Self::ServiceUnavailable(_) => "SERVICE_UNAVAILABLE",
            Self::Custom { .. } => "CUSTOM_ERROR",
        }
    }

    pub fn message(&self) -> &str {
        match self {
            Self::BadRequest(msg) => msg,
            Self::Unauthorized(msg) => msg,
            Self::Forbidden(msg) => msg,
            Self::NotFound(msg) => msg,
            Self::Conflict(msg) => msg,
            Self::RateLimitExceeded(msg) => msg,
            Self::InternalError(msg) => msg,
            Self::ServiceUnavailable(msg) => msg,
            Self::Custom { message, .. } => message,
        }
    }
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.error_code(), self.message())
    }
}

impl std::error::Error for ApiError {}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let error_response = ErrorResponse::new(self.message(), self.error_code());

        (status, Json(error_response)).into_response()
    }
}

// Conversions from other error types
impl From<anyhow::Error> for ApiError {
    fn from(err: anyhow::Error) -> Self {
        Self::InternalError(err.to_string())
    }
}

impl From<std::io::Error> for ApiError {
    fn from(err: std::io::Error) -> Self {
        Self::InternalError(err.to_string())
    }
}

impl From<serde_json::Error> for ApiError {
    fn from(err: serde_json::Error) -> Self {
        Self::BadRequest(format!("Invalid JSON: {}", err))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_error_status_code() {
        assert_eq!(
            ApiError::BadRequest("test".to_string()).status_code(),
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            ApiError::Unauthorized("test".to_string()).status_code(),
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            ApiError::NotFound("test".to_string()).status_code(),
            StatusCode::NOT_FOUND
        );
    }

    #[test]
    fn test_api_error_code() {
        assert_eq!(
            ApiError::BadRequest("test".to_string()).error_code(),
            "BAD_REQUEST"
        );
        assert_eq!(
            ApiError::RateLimitExceeded("test".to_string()).error_code(),
            "RATE_LIMIT_EXCEEDED"
        );
    }

    #[test]
    fn test_api_error_display() {
        let error = ApiError::NotFound("Resource not found".to_string());
        assert_eq!(error.to_string(), "NOT_FOUND: Resource not found");
    }
}

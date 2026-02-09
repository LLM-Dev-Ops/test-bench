//! Axum middleware for execution context extraction and enforcement.
//!
//! This middleware intercepts incoming HTTP requests and:
//! 1. Extracts ExecutionContext from headers (X-Execution-Id, X-Parent-Span-Id)
//! 2. Rejects requests that lack a valid parent_span_id
//! 3. Injects the context into request extensions for handlers to use

use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

use super::context::{extract_from_headers, ExecutionContext};

/// Axum middleware layer that enforces execution context on all requests.
///
/// Requests without valid X-Execution-Id and X-Parent-Span-Id headers
/// are rejected with 400 Bad Request.
///
/// The health check endpoint (/health) is exempted.
pub async fn execution_context_layer(
    request: Request,
    next: Next,
) -> Response {
    // Exempt health check from execution context requirement
    if request.uri().path() == "/health" || request.uri().path() == "/" {
        return next.run(request).await;
    }

    match extract_from_headers(request.headers()) {
        Ok(ctx) => {
            let mut request = request;
            request.extensions_mut().insert(ctx);
            next.run(request).await
        }
        Err(e) => {
            let body = json!({
                "error": "ExecutionContextError",
                "message": e.message,
                "hint": "Provide X-Execution-Id and X-Parent-Span-Id headers. \
                         This repository is a Foundational Execution Unit and \
                         rejects execution without a valid parent context."
            });
            (StatusCode::BAD_REQUEST, Json(body)).into_response()
        }
    }
}

/// Convenience alias for the execution context middleware layer.
pub type ExecutionLayer = axum::middleware::FromFnLayer<
    fn(Request, Next) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>>,
    (),
    Request,
>;

/// Extract ExecutionContext from request extensions (after middleware).
///
/// Use in handlers:
/// ```rust,ignore
/// async fn my_handler(
///     Extension(ctx): Extension<ExecutionContext>,
/// ) -> impl IntoResponse {
///     // ctx.execution_id, ctx.parent_span_id available
/// }
/// ```
pub fn get_execution_context(extensions: &axum::http::Extensions) -> Option<&ExecutionContext> {
    extensions.get::<ExecutionContext>()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderMap;

    #[test]
    fn test_extract_from_headers_valid() {
        let mut headers = HeaderMap::new();
        headers.insert("X-Execution-Id", "exec-123".parse().unwrap());
        headers.insert("X-Parent-Span-Id", "span-456".parse().unwrap());

        let ctx = extract_from_headers(&headers).unwrap();
        assert_eq!(ctx.execution_id, "exec-123");
        assert_eq!(ctx.parent_span_id, "span-456");
    }

    #[test]
    fn test_extract_from_headers_missing() {
        let headers = HeaderMap::new();
        assert!(extract_from_headers(&headers).is_err());
    }
}

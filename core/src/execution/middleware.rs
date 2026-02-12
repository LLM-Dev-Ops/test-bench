//! Axum middleware for execution context extraction, enforcement,
//! and policy-gated intent validation.
//!
//! This middleware intercepts incoming HTTP requests and:
//! 1. Extracts ExecutionContext from headers (X-Execution-Id, X-Parent-Span-Id)
//! 2. Rejects requests that lack a valid parent_span_id
//! 3. Enforces policy gates when X-Simulation-Intent header is present
//! 4. Accepts exemption headers for mandatory intents without execution context
//! 5. Injects the context (and any exemption record) into request extensions

use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use chrono::Utc;
use serde_json::json;

use super::context::{extract_from_headers, ExecutionContext};
use super::policy::{default_policy, ExemptionRecord, SimulationIntent};

/// Axum middleware layer that enforces execution context and policy gates.
///
/// Requests without valid X-Execution-Id and X-Parent-Span-Id headers
/// are rejected with 400 Bad Request, unless:
/// - The request is to /health or /
/// - The request declares a mandatory intent via X-Simulation-Intent
///   with valid exemption headers (X-Exemption-Reason, X-Exemption-Granted-By),
///   in which case it is allowed with an ExemptionRecord injected.
///
/// A mandatory intent without context AND without exemption returns 403.
pub async fn execution_context_layer(
    request: Request,
    next: Next,
) -> Response {
    // Exempt health check from execution context requirement
    if request.uri().path() == "/health" || request.uri().path() == "/" {
        return next.run(request).await;
    }

    // Extract optional intent header
    let intent_header = request
        .headers()
        .get("X-Simulation-Intent")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    match extract_from_headers(request.headers()) {
        Ok(ctx) => {
            let mut request = request;
            request.extensions_mut().insert(ctx);
            // If intent header present, inject parsed intent for downstream handlers
            if let Some(ref intent_str) = intent_header {
                if let Ok(intent) = serde_json::from_str::<SimulationIntent>(
                    &format!("\"{}\"", intent_str),
                ) {
                    request.extensions_mut().insert(intent);
                }
            }
            next.run(request).await
        }
        Err(e) => {
            // No valid execution context -- check if this is a policy-gated intent
            if let Some(ref intent_str) = intent_header {
                if let Ok(intent) = serde_json::from_str::<SimulationIntent>(
                    &format!("\"{}\"", intent_str),
                ) {
                    let policy = default_policy();
                    if policy.is_mandatory(&intent) {
                        // Check for exemption headers
                        let exemption_reason = request
                            .headers()
                            .get("X-Exemption-Reason")
                            .and_then(|v| v.to_str().ok())
                            .map(|s| s.to_string());
                        let exemption_by = request
                            .headers()
                            .get("X-Exemption-Granted-By")
                            .and_then(|v| v.to_str().ok())
                            .map(|s| s.to_string());

                        if let (Some(reason), Some(granted_by)) = (exemption_reason, exemption_by) {
                            // Allow with exemption record injected
                            let record = ExemptionRecord {
                                intent,
                                reason,
                                granted_by,
                                granted_at: Utc::now(),
                                expires_at: None,
                                execution_id: "EXEMPTED".to_string(),
                                parent_span_id: "EXEMPTED".to_string(),
                            };
                            let mut request = request;
                            request.extensions_mut().insert(record);
                            return next.run(request).await;
                        }

                        // Mandatory intent, no context, no exemption -> 403
                        let body = json!({
                            "error": "PolicyGateError",
                            "intent": intent_str,
                            "message": format!(
                                "Intent '{}' has mandatory policy gate. \
                                 Provide X-Execution-Id and X-Parent-Span-Id headers, \
                                 or provide X-Exemption-Reason and X-Exemption-Granted-By headers.",
                                intent_str
                            ),
                        });
                        return (StatusCode::FORBIDDEN, Json(body)).into_response();
                    }
                }
            }

            // Original behavior for non-intent requests or non-mandatory intents
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
pub fn get_execution_context(extensions: &axum::http::Extensions) -> Option<&ExecutionContext> {
    extensions.get::<ExecutionContext>()
}

/// Extract ExemptionRecord from request extensions (after middleware).
/// Present only when a mandatory intent was exempted.
pub fn get_exemption_record(extensions: &axum::http::Extensions) -> Option<&ExemptionRecord> {
    extensions.get::<ExemptionRecord>()
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

    #[test]
    fn test_mandatory_intent_detected_by_policy() {
        let policy = default_policy();
        let intent: SimulationIntent =
            serde_json::from_str("\"benchmark\"").unwrap();
        assert!(policy.is_mandatory(&intent));
    }

    #[test]
    fn test_recommended_intent_not_mandatory() {
        let policy = default_policy();
        let intent: SimulationIntent =
            serde_json::from_str("\"complete\"").unwrap();
        assert!(!policy.is_mandatory(&intent));
    }
}

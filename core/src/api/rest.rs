// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! REST API implementation.

use axum::{
    Router,
    routing::{get, post},
    extract::{State, Path, Query},
    Json,
};
use utoipa::OpenApi;
use serde::Deserialize;
use chrono::Utc;

use crate::api::{
    models::*,
    error::{ApiError, ApiResult},
};
use std::sync::Arc;

/// Query parameters for pagination
#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    #[serde(default = "default_page")]
    pub page: usize,
    #[serde(default = "default_page_size")]
    pub page_size: usize,
}

fn default_page() -> usize { 1 }
fn default_page_size() -> usize { 20 }

/// REST API routes
pub struct RestApi;

impl RestApi {
    /// Create REST API router
    pub fn router<S: Clone + Send + Sync + 'static>() -> Router<Arc<S>> {
        Router::new()
            .route("/health", get(health_check))
            .route("/v1/completions", post(create_completion::<S>))
            .route("/v1/evaluations", post(create_evaluation::<S>))
            .route("/v1/benchmarks", post(create_benchmark::<S>))
            .route("/v1/benchmarks/:id", get(get_benchmark::<S>))
            .route("/v1/plugins", get(list_plugins::<S>))
            .route("/v1/plugins/:id/execute", post(execute_plugin::<S>))
    }
}

/// Health check endpoint
#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Service is healthy", body = HealthResponse)
    )
)]
async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        version: crate::api::API_VERSION.to_string(),
        timestamp: Utc::now(),
        uptime_seconds: 0, // Would calculate actual uptime
    })
}

/// Create completion
#[utoipa::path(
    post,
    path = "/v1/completions",
    request_body = CompletionRequest,
    responses(
        (status = 200, description = "Completion created", body = CompletionResponse),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 500, description = "Internal error", body = ErrorResponse)
    )
)]
async fn create_completion<S: Clone + Send + Sync>(
    State(_state): State<Arc<S>>,
    Json(request): Json<CompletionRequest>,
) -> ApiResult<Json<CompletionResponse>> {
    // Implementation would call provider
    let response = CompletionResponse {
        text: "Generated response".to_string(),
        model: request.model,
        usage: Some(TokenUsage {
            prompt_tokens: 10,
            completion_tokens: 20,
            total_tokens: 30,
        }),
        request_id: uuid::Uuid::new_v4().to_string(),
        created_at: Utc::now(),
    };

    Ok(Json(response))
}

/// Create evaluation
async fn create_evaluation<S: Clone + Send + Sync>(
    State(_state): State<Arc<S>>,
    Json(_request): Json<EvaluationRequest>,
) -> ApiResult<Json<EvaluationResponse>> {
    let response = EvaluationResponse {
        score: 0.85,
        metrics: std::collections::HashMap::new(),
        evaluation_id: uuid::Uuid::new_v4().to_string(),
        created_at: Utc::now(),
    };

    Ok(Json(response))
}

/// Create benchmark
async fn create_benchmark<S: Clone + Send + Sync>(
    State(_state): State<Arc<S>>,
    Json(_request): Json<BenchmarkRequest>,
) -> ApiResult<Json<BenchmarkResponse>> {
    let response = BenchmarkResponse {
        benchmark_id: uuid::Uuid::new_v4().to_string(),
        status: BenchmarkStatus::Pending,
        progress: 0.0,
        results: None,
    };

    Ok(Json(response))
}

/// Get benchmark
async fn get_benchmark<S: Clone + Send + Sync>(
    State(_state): State<Arc<S>>,
    Path(id): Path<String>,
) -> ApiResult<Json<BenchmarkResponse>> {
    let response = BenchmarkResponse {
        benchmark_id: id,
        status: BenchmarkStatus::Running,
        progress: 45.0,
        results: None,
    };

    Ok(Json(response))
}

/// List plugins
async fn list_plugins<S: Clone + Send + Sync>(
    State(_state): State<Arc<S>>,
    Query(params): Query<PaginationParams>,
) -> ApiResult<Json<PaginatedResponse<PluginInfo>>> {
    let items = vec![];
    let response = PaginatedResponse::new(items, 0, params.page, params.page_size);

    Ok(Json(response))
}

/// Execute plugin
async fn execute_plugin<S: Clone + Send + Sync>(
    State(_state): State<Arc<S>>,
    Path(_id): Path<String>,
    Json(_request): Json<PluginExecuteRequest>,
) -> ApiResult<Json<PluginExecuteResponse>> {
    let response = PluginExecuteResponse {
        output: serde_json::json!({}),
        execution_time_ms: 100,
        request_id: uuid::Uuid::new_v4().to_string(),
    };

    Ok(Json(response))
}

/// OpenAPI documentation
#[derive(OpenApi)]
#[openapi(
    paths(
        health_check,
        create_completion,
    ),
    components(
        schemas(
            HealthResponse,
            CompletionRequest,
            CompletionResponse,
            TokenUsage,
            ErrorResponse,
        )
    ),
    tags(
        (name = "health", description = "Health check endpoints"),
        (name = "completions", description = "LLM completion endpoints"),
        (name = "evaluations", description = "Evaluation endpoints"),
        (name = "benchmarks", description = "Benchmark endpoints"),
        (name = "plugins", description = "Plugin endpoints"),
    )
)]
pub struct ApiDoc;

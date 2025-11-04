// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! API server implementation.

use axum::{
    Router,
    Extension,
    routing::get,
};
use std::{net::SocketAddr, sync::Arc};
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::{
    trace::TraceLayer,
    compression::CompressionLayer,
};
use tracing::info;
use anyhow::Result;

use crate::api::{
    auth::AuthService,
    middleware::CorsConfig,
    rest::RestApi,
    graphql::GraphQLApi,
    websocket::{ws_router, WsState},
};

/// API server configuration
#[derive(Debug, Clone)]
pub struct ApiConfig {
    /// Server bind address
    pub bind_address: SocketAddr,

    /// Enable REST API
    pub enable_rest: bool,

    /// Enable GraphQL API
    pub enable_graphql: bool,

    /// Enable WebSocket API
    pub enable_websocket: bool,

    /// Enable Swagger UI
    pub enable_swagger: bool,

    /// CORS configuration
    pub cors: CorsConfig,

    /// JWT secret key
    pub jwt_secret: String,

    /// JWT token expiration (seconds)
    pub jwt_expiration: i64,

    /// Rate limit: requests per second
    pub rate_limit_rps: Option<u64>,

    /// Rate limit: burst size
    pub rate_limit_burst: Option<u32>,

    /// WebSocket channel capacity
    pub ws_channel_capacity: usize,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            bind_address: "0.0.0.0:3000".parse().unwrap(),
            enable_rest: true,
            enable_graphql: true,
            enable_websocket: true,
            enable_swagger: true,
            cors: CorsConfig::default(),
            jwt_secret: "change_this_in_production_use_env_var".to_string(),
            jwt_expiration: 3600, // 1 hour
            rate_limit_rps: Some(100),
            rate_limit_burst: Some(50),
            ws_channel_capacity: 1000,
        }
    }
}

impl ApiConfig {
    /// Create a builder for ApiConfig
    pub fn builder() -> ApiConfigBuilder {
        ApiConfigBuilder::default()
    }
}

/// API server configuration builder
#[derive(Default)]
pub struct ApiConfigBuilder {
    config: ApiConfig,
}

impl ApiConfigBuilder {
    pub fn bind_address(mut self, addr: SocketAddr) -> Self {
        self.config.bind_address = addr;
        self
    }

    pub fn enable_rest(mut self, enable: bool) -> Self {
        self.config.enable_rest = enable;
        self
    }

    pub fn enable_graphql(mut self, enable: bool) -> Self {
        self.config.enable_graphql = enable;
        self
    }

    pub fn enable_websocket(mut self, enable: bool) -> Self {
        self.config.enable_websocket = enable;
        self
    }

    pub fn enable_swagger(mut self, enable: bool) -> Self {
        self.config.enable_swagger = enable;
        self
    }

    pub fn cors(mut self, cors: CorsConfig) -> Self {
        self.config.cors = cors;
        self
    }

    pub fn jwt_secret(mut self, secret: String) -> Self {
        self.config.jwt_secret = secret;
        self
    }

    pub fn jwt_expiration(mut self, seconds: i64) -> Self {
        self.config.jwt_expiration = seconds;
        self
    }

    pub fn rate_limit(mut self, rps: u64, burst: u32) -> Self {
        self.config.rate_limit_rps = Some(rps);
        self.config.rate_limit_burst = Some(burst);
        self
    }

    pub fn build(self) -> ApiConfig {
        self.config
    }
}

/// Shared application state
#[derive(Clone)]
pub struct AppState {
    /// Authentication service
    pub auth: Arc<AuthService>,

    /// WebSocket state
    pub ws_state: Arc<WsState>,

    /// GraphQL schema
    pub graphql_schema: crate::api::graphql::GraphQLSchema,
}

/// API server
pub struct ApiServer {
    config: ApiConfig,
    state: Arc<AppState>,
}

impl ApiServer {
    /// Create a new API server
    pub fn new(config: ApiConfig) -> Self {
        let auth = Arc::new(AuthService::new(
            config.jwt_secret.clone(),
            config.jwt_expiration,
        ));

        let ws_state = Arc::new(WsState::new(config.ws_channel_capacity));

        let graphql_schema = GraphQLApi::schema();

        let state = Arc::new(AppState {
            auth,
            ws_state,
            graphql_schema,
        });

        Self { config, state }
    }

    /// Build the application router
    fn build_router(&self) -> Router {
        let mut app = Router::new();

        // Health check endpoint (always enabled)
        app = app.route("/", get(|| async { "LLM Test Bench API" }));

        // REST API
        if self.config.enable_rest {
            info!("Enabling REST API at /v1/*");
            let rest_router = RestApi::router::<AppState>()
                .with_state(self.state.clone());
            app = app.merge(rest_router);
        }

        // GraphQL API
        if self.config.enable_graphql {
            info!("Enabling GraphQL API at /graphql");
            let graphql_router = self.build_graphql_router();
            app = app.nest("/graphql", graphql_router);
        }

        // WebSocket API
        if self.config.enable_websocket {
            info!("Enabling WebSocket API at /ws");
            let ws_router = ws_router::<AppState>()
                .with_state(self.state.ws_state.clone());
            app = app.merge(ws_router);
        }

        // Swagger UI
        if self.config.enable_swagger {
            info!("Enabling Swagger UI at /swagger-ui");
            app = app.merge(self.build_swagger_router());
        }

        // Add middleware layers
        app = app.layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CompressionLayer::new())
                .layer(self.config.cors.to_layer())
                .layer(Extension(self.state.clone()))
        );

        app
    }

    /// Build GraphQL router
    fn build_graphql_router(&self) -> Router {
        use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
        use async_graphql::http::GraphiQLSource;

        let schema = self.state.graphql_schema.clone();

        async fn graphql_handler(
            schema: axum::extract::Extension<async_graphql::Schema<QueryRoot, EmptyMutation, EmptySubscription>>,
            req: GraphQLRequest,
        ) -> GraphQLResponse {
            schema.execute(req.into_inner()).await.into()
        }

        async fn graphiql_handler() -> axum::response::Html<String> {
            axum::response::Html(
                GraphiQLSource::build()
                    .endpoint("/graphql")
                    .finish()
            )
        }

        Router::new()
            .route("/", axum::routing::post(graphql_handler))
            .route("/", axum::routing::get(graphiql_handler))
            .layer(axum::extract::Extension(schema))
    }

    /// Build Swagger UI router
    fn build_swagger_router(&self) -> Router {
        use utoipa::OpenApi;
        use utoipa_swagger_ui::SwaggerUi;

        let openapi = crate::api::rest::ApiDoc::openapi();

        SwaggerUi::new("/swagger-ui")
            .url("/api-docs/openapi.json", openapi)
            .into()
    }

    /// Start the API server
    pub async fn start(self) -> Result<()> {
        let addr = self.config.bind_address;
        let app = self.build_router();

        info!("Starting LLM Test Bench API server");
        info!("Listening on http://{}", addr);

        if self.config.enable_rest {
            info!("  REST API: http://{}/v1", addr);
        }
        if self.config.enable_graphql {
            info!("  GraphQL: http://{}/graphql", addr);
        }
        if self.config.enable_websocket {
            info!("  WebSocket: ws://{}/ws", addr);
        }
        if self.config.enable_swagger {
            info!("  Swagger UI: http://{}/swagger-ui", addr);
        }

        let listener = TcpListener::bind(addr).await?;
        axum::serve(listener, app).await?;

        Ok(())
    }

    /// Get a reference to the application state
    pub fn state(&self) -> &Arc<AppState> {
        &self.state
    }

    /// Get a reference to the configuration
    pub fn config(&self) -> &ApiConfig {
        &self.config
    }
}

/// Builder for ApiServer
pub struct ApiServerBuilder {
    config: ApiConfig,
}

impl Default for ApiServerBuilder {
    fn default() -> Self {
        Self {
            config: ApiConfig::default(),
        }
    }
}

impl ApiServerBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the configuration
    pub fn config(mut self, config: ApiConfig) -> Self {
        self.config = config;
        self
    }

    /// Set the bind address
    pub fn bind_address(mut self, addr: SocketAddr) -> Self {
        self.config.bind_address = addr;
        self
    }

    /// Build the server
    pub fn build(self) -> ApiServer {
        ApiServer::new(self.config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_config_default() {
        let config = ApiConfig::default();
        assert!(config.enable_rest);
        assert!(config.enable_graphql);
        assert!(config.enable_websocket);
        assert_eq!(config.jwt_expiration, 3600);
    }

    #[test]
    fn test_api_config_builder() {
        let config = ApiConfig::builder()
            .enable_rest(true)
            .enable_graphql(false)
            .jwt_expiration(7200)
            .build();

        assert!(config.enable_rest);
        assert!(!config.enable_graphql);
        assert_eq!(config.jwt_expiration, 7200);
    }

    #[test]
    fn test_api_server_creation() {
        let config = ApiConfig::default();
        let server = ApiServer::new(config);

        assert!(server.config().enable_rest);
        assert!(server.config().enable_graphql);
    }

    #[test]
    fn test_api_server_builder() {
        let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
        let server = ApiServerBuilder::new()
            .bind_address(addr)
            .build();

        assert_eq!(server.config().bind_address, addr);
    }
}

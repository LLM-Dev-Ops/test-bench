// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! WebSocket server for real-time monitoring dashboards.

use anyhow::Result;
use std::sync::Arc;
use parking_lot::RwLock;
use tokio::sync::broadcast;
use tokio::task::JoinHandle;
use serde::{Deserialize, Serialize};
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use futures::{sink::SinkExt, stream::StreamExt};

use crate::monitoring::events::{EventBus, MonitoringEvent};

/// WebSocket server configuration
#[derive(Debug, Clone)]
pub struct WebSocketConfig {
    /// Port to listen on
    pub port: u16,
    /// Enable the server
    pub enabled: bool,
}

impl Default for WebSocketConfig {
    fn default() -> Self {
        Self {
            port: 8080,
            enabled: true,
        }
    }
}

/// WebSocket message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum WebSocketMessage {
    /// Monitoring event
    Event(MonitoringEvent),
    /// Ping/pong for keep-alive
    Ping,
    /// Pong response
    Pong,
    /// Subscription request
    Subscribe { event_types: Vec<String> },
    /// Unsubscribe request
    Unsubscribe { event_types: Vec<String> },
    /// Connection established
    Connected { client_id: String },
    /// Error message
    Error { message: String },
}

/// WebSocket server state
#[derive(Clone)]
struct ServerState {
    event_bus: Arc<EventBus>,
    active_connections: Arc<RwLock<usize>>,
}

/// WebSocket server for real-time event streaming
pub struct WebSocketServer {
    config: WebSocketConfig,
    event_bus: Arc<EventBus>,
    server_handle: Arc<RwLock<Option<JoinHandle<()>>>>,
    active_connections: Arc<RwLock<usize>>,
}

impl WebSocketServer {
    /// Create a new WebSocket server
    pub async fn new(config: WebSocketConfig, event_bus: Arc<EventBus>) -> Result<Self> {
        Ok(Self {
            config,
            event_bus,
            server_handle: Arc::new(RwLock::new(None)),
            active_connections: Arc::new(RwLock::new(0)),
        })
    }

    /// Start the WebSocket server
    pub async fn start(&self) -> Result<()> {
        if !self.config.enabled {
            tracing::debug!("WebSocket server is disabled");
            return Ok(());
        }

        let state = ServerState {
            event_bus: self.event_bus.clone(),
            active_connections: self.active_connections.clone(),
        };

        let app = Router::new()
            .route("/ws", get(ws_handler))
            .route("/health", get(health_handler))
            .with_state(state);

        let addr = format!("0.0.0.0:{}", self.config.port).parse()?;
        tracing::info!("Starting WebSocket server on {}", addr);

        let server = tokio::spawn(async move {
            let listener = tokio::net::TcpListener::bind(addr)
                .await
                .expect("Failed to bind WebSocket server");

            axum::serve(listener, app)
                .await
                .expect("WebSocket server error");
        });

        let mut handle = self.server_handle.write();
        *handle = Some(server);

        Ok(())
    }

    /// Stop the WebSocket server
    pub async fn stop(&self) -> Result<()> {
        let mut handle = self.server_handle.write();
        if let Some(h) = handle.take() {
            h.abort();
        }
        Ok(())
    }

    /// Get active connection count
    pub fn active_connections(&self) -> usize {
        *self.active_connections.read()
    }
}

/// WebSocket upgrade handler
async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<ServerState>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

/// Health check endpoint
async fn health_handler(State(state): State<ServerState>) -> impl IntoResponse {
    let connections = *state.active_connections.read();
    axum::Json(serde_json::json!({
        "status": "healthy",
        "active_connections": connections,
    }))
}

/// Handle WebSocket connection
async fn handle_socket(socket: WebSocket, state: ServerState) {
    // Increment connection count
    {
        let mut count = state.active_connections.write();
        *count += 1;
    }

    let client_id = generate_client_id();
    tracing::info!("New WebSocket connection: {}", client_id);

    let (mut sender, mut receiver) = socket.split();

    // Subscribe to events
    let mut event_rx = state.event_bus.subscribe();

    // Send connection message
    let connected_msg = WebSocketMessage::Connected {
        client_id: client_id.clone(),
    };
    if let Ok(json) = serde_json::to_string(&connected_msg) {
        let _ = sender.send(Message::Text(json)).await;
    }

    // Spawn task to forward events to client
    let mut send_task = tokio::spawn(async move {
        loop {
            tokio::select! {
                // Receive event from event bus
                Ok(event) = event_rx.recv() => {
                    let ws_msg = WebSocketMessage::Event(event);
                    if let Ok(json) = serde_json::to_string(&ws_msg) {
                        if sender.send(Message::Text(json)).await.is_err() {
                            break;
                        }
                    }
                }
                // Send periodic ping
                _ = tokio::time::sleep(tokio::time::Duration::from_secs(30)) => {
                    let ping_msg = WebSocketMessage::Ping;
                    if let Ok(json) = serde_json::to_string(&ping_msg) {
                        if sender.send(Message::Text(json)).await.is_err() {
                            break;
                        }
                    }
                }
            }
        }
    });

    // Receive messages from client
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Text(text) => {
                    if let Ok(ws_msg) = serde_json::from_str::<WebSocketMessage>(&text) {
                        match ws_msg {
                            WebSocketMessage::Ping => {
                                tracing::trace!("Received ping from {}", client_id);
                            }
                            WebSocketMessage::Pong => {
                                tracing::trace!("Received pong from {}", client_id);
                            }
                            WebSocketMessage::Subscribe { event_types } => {
                                tracing::debug!("Client {} subscribed to: {:?}", client_id, event_types);
                            }
                            WebSocketMessage::Unsubscribe { event_types } => {
                                tracing::debug!("Client {} unsubscribed from: {:?}", client_id, event_types);
                            }
                            _ => {}
                        }
                    }
                }
                Message::Close(_) => {
                    tracing::info!("Client {} disconnected", client_id);
                    break;
                }
                _ => {}
            }
        }
    });

    // Wait for either task to finish
    tokio::select! {
        _ = &mut send_task => {
            recv_task.abort();
        }
        _ = &mut recv_task => {
            send_task.abort();
        }
    }

    // Decrement connection count
    {
        let mut count = state.active_connections.write();
        *count = count.saturating_sub(1);
    }

    tracing::info!("WebSocket connection closed: {}", client_id);
}

/// Generate a unique client ID
fn generate_client_id() -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let id = COUNTER.fetch_add(1, Ordering::SeqCst);
    format!("client_{}", id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::monitoring::events::EventBus;

    #[test]
    fn test_websocket_config() {
        let config = WebSocketConfig {
            port: 8081,
            enabled: true,
        };
        assert_eq!(config.port, 8081);
        assert!(config.enabled);
    }

    #[tokio::test]
    async fn test_websocket_server_creation() {
        let config = WebSocketConfig {
            port: 8082,
            enabled: false,
        };
        let event_bus = Arc::new(EventBus::new());
        let server = WebSocketServer::new(config, event_bus).await;
        assert!(server.is_ok());
    }

    #[tokio::test]
    async fn test_active_connections() {
        let config = WebSocketConfig {
            port: 8083,
            enabled: false,
        };
        let event_bus = Arc::new(EventBus::new());
        let server = WebSocketServer::new(config, event_bus).await.unwrap();
        assert_eq!(server.active_connections(), 0);
    }

    #[test]
    fn test_websocket_message_serialization() {
        let msg = WebSocketMessage::Connected {
            client_id: "test123".to_string(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("Connected"));
        assert!(json.contains("test123"));
    }

    #[test]
    fn test_client_id_generation() {
        let id1 = generate_client_id();
        let id2 = generate_client_id();
        assert_ne!(id1, id2);
    }
}

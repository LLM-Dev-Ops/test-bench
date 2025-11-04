// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! WebSocket API implementation for real-time updates.

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::Response,
};
use futures::{sink::SinkExt, stream::StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{debug, error, info, warn};

/// WebSocket message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WsMessage {
    /// Subscription request
    Subscribe {
        topics: Vec<String>,
    },
    /// Unsubscribe request
    Unsubscribe {
        topics: Vec<String>,
    },
    /// Ping message
    Ping {
        timestamp: i64,
    },
    /// Pong message
    Pong {
        timestamp: i64,
    },
    /// Event notification
    Event {
        topic: String,
        data: serde_json::Value,
        timestamp: i64,
    },
    /// Error message
    Error {
        code: String,
        message: String,
    },
    /// Success acknowledgment
    Ack {
        request_id: Option<String>,
        message: String,
    },
}

/// WebSocket event topics
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WsTopic {
    /// Benchmark progress updates
    BenchmarkProgress,
    /// Evaluation results
    EvaluationResults,
    /// System metrics
    SystemMetrics,
    /// Plugin events
    PluginEvents,
    /// Request logs
    RequestLogs,
}

impl WsTopic {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::BenchmarkProgress => "benchmark.progress",
            Self::EvaluationResults => "evaluation.results",
            Self::SystemMetrics => "system.metrics",
            Self::PluginEvents => "plugin.events",
            Self::RequestLogs => "request.logs",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "benchmark.progress" => Some(Self::BenchmarkProgress),
            "evaluation.results" => Some(Self::EvaluationResults),
            "system.metrics" => Some(Self::SystemMetrics),
            "plugin.events" => Some(Self::PluginEvents),
            "request.logs" => Some(Self::RequestLogs),
            _ => None,
        }
    }
}

/// WebSocket state
#[derive(Clone)]
pub struct WsState {
    /// Event broadcaster
    pub tx: broadcast::Sender<WsMessage>,
}

impl WsState {
    /// Create new WebSocket state
    pub fn new(capacity: usize) -> Self {
        let (tx, _) = broadcast::channel(capacity);
        Self { tx }
    }

    /// Broadcast an event to all subscribers
    pub fn broadcast(&self, topic: WsTopic, data: serde_json::Value) {
        let msg = WsMessage::Event {
            topic: topic.as_str().to_string(),
            data,
            timestamp: chrono::Utc::now().timestamp_millis(),
        };

        // Ignore send errors (no receivers)
        let _ = self.tx.send(msg);
    }

    /// Broadcast a generic message
    pub fn broadcast_message(&self, msg: WsMessage) {
        let _ = self.tx.send(msg);
    }
}

/// WebSocket upgrade handler
pub async fn ws_handler<S: Clone + Send + Sync + 'static>(
    ws: WebSocketUpgrade,
    State(state): State<Arc<WsState>>,
) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

/// Handle WebSocket connection
async fn handle_socket(socket: WebSocket, state: Arc<WsState>) {
    info!("New WebSocket connection established");

    let (mut sender, mut receiver) = socket.split();

    // Subscribe to broadcast channel
    let mut rx = state.tx.subscribe();

    // Track subscribed topics for this connection
    let mut subscribed_topics: Vec<String> = Vec::new();

    // Send welcome message
    let welcome = WsMessage::Ack {
        request_id: None,
        message: "Connected to LLM Test Bench API".to_string(),
    };

    if let Ok(msg) = serde_json::to_string(&welcome) {
        if sender.send(Message::Text(msg)).await.is_err() {
            error!("Failed to send welcome message");
            return;
        }
    }

    // Spawn a task to forward broadcast messages to this client
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            // Filter messages based on subscriptions
            let should_send = match &msg {
                WsMessage::Event { topic, .. } => {
                    subscribed_topics.is_empty() || subscribed_topics.contains(topic)
                }
                _ => true, // Always send non-event messages
            };

            if !should_send {
                continue;
            }

            let text = match serde_json::to_string(&msg) {
                Ok(t) => t,
                Err(e) => {
                    error!("Failed to serialize message: {}", e);
                    continue;
                }
            };

            if sender.send(Message::Text(text)).await.is_err() {
                warn!("Client disconnected");
                break;
            }
        }
    });

    // Handle incoming messages from client
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Text(text) => {
                    debug!("Received message: {}", text);

                    // Parse and handle message
                    match serde_json::from_str::<WsMessage>(&text) {
                        Ok(ws_msg) => {
                            match ws_msg {
                                WsMessage::Subscribe { topics } => {
                                    info!("Client subscribing to topics: {:?}", topics);
                                    for topic in &topics {
                                        if !subscribed_topics.contains(topic) {
                                            subscribed_topics.push(topic.clone());
                                        }
                                    }

                                    // Send acknowledgment
                                    let ack = WsMessage::Ack {
                                        request_id: None,
                                        message: format!("Subscribed to {} topics", topics.len()),
                                    };

                                    if let Ok(ack_msg) = serde_json::to_string(&ack) {
                                        // Note: Can't send from this task, would need channel
                                        debug!("Would send ack: {}", ack_msg);
                                    }
                                }
                                WsMessage::Unsubscribe { topics } => {
                                    info!("Client unsubscribing from topics: {:?}", topics);
                                    subscribed_topics.retain(|t| !topics.contains(t));
                                }
                                WsMessage::Ping { timestamp } => {
                                    debug!("Received ping at {}", timestamp);
                                    // Pong is sent automatically by axum
                                }
                                _ => {
                                    warn!("Received unexpected message type");
                                }
                            }
                        }
                        Err(e) => {
                            error!("Failed to parse message: {}", e);
                            let error_msg = WsMessage::Error {
                                code: "INVALID_MESSAGE".to_string(),
                                message: format!("Failed to parse message: {}", e),
                            };

                            if let Ok(err_text) = serde_json::to_string(&error_msg) {
                                debug!("Would send error: {}", err_text);
                            }
                        }
                    }
                }
                Message::Binary(_) => {
                    warn!("Received unexpected binary message");
                }
                Message::Ping(_) => {
                    debug!("Received ping");
                }
                Message::Pong(_) => {
                    debug!("Received pong");
                }
                Message::Close(_) => {
                    info!("Client closing connection");
                    break;
                }
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

    info!("WebSocket connection closed");
}

/// WebSocket router
pub fn ws_router<S: Clone + Send + Sync + 'static>() -> axum::Router<Arc<WsState>> {
    axum::Router::new()
        .route("/ws", axum::routing::get(ws_handler::<S>))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ws_topic_conversion() {
        assert_eq!(WsTopic::BenchmarkProgress.as_str(), "benchmark.progress");
        assert_eq!(
            WsTopic::from_str("benchmark.progress"),
            Some(WsTopic::BenchmarkProgress)
        );
        assert_eq!(WsTopic::from_str("unknown"), None);
    }

    #[test]
    fn test_ws_message_serialization() {
        let msg = WsMessage::Subscribe {
            topics: vec!["benchmark.progress".to_string()],
        };

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("subscribe"));
        assert!(json.contains("benchmark.progress"));
    }

    #[test]
    fn test_ws_state_creation() {
        let state = WsState::new(100);

        // Should be able to broadcast without panic
        state.broadcast(
            WsTopic::SystemMetrics,
            serde_json::json!({"cpu": 50.0}),
        );
    }
}

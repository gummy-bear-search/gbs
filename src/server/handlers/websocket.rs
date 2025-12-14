//! WebSocket handler for real-time updates

use axum::{
    extract::{State, ws::{WebSocketUpgrade, Message}},
    response::Response,
};
use futures_util::{SinkExt, StreamExt};
use tracing::{info, debug, error};

use crate::server::AppState;

pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> Response {
    info!("WebSocket connection requested");
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

/// Handle WebSocket connection
async fn handle_socket(socket: axum::extract::ws::WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();

    info!("WebSocket connection established");

    // Send initial cluster health
    let health = serde_json::json!({
        "type": "cluster_health",
        "data": {
            "status": "green",
            "number_of_nodes": 1,
            "number_of_data_nodes": 1,
            "active_primary_shards": 0,
            "active_shards": 0,
        }
    });

    if let Err(e) = sender.send(Message::Text(health.to_string())).await {
        error!("Failed to send initial health: {}", e);
        return;
    }

    // Send initial cluster stats
    let stats = state.storage.get_cluster_stats(&state.es_version).await;
    let stats_msg = serde_json::json!({
        "type": "cluster_stats",
        "data": stats
    });

    if let Err(e) = sender.send(Message::Text(stats_msg.to_string())).await {
        error!("Failed to send initial stats: {}", e);
        return;
    }

    // Send initial indices list
    let indices_stats = state.storage.get_indices_stats().await;
    let indices_msg = serde_json::json!({
        "type": "indices",
        "data": indices_stats
    });

    if let Err(e) = sender.send(Message::Text(indices_msg.to_string())).await {
        error!("Failed to send initial indices: {}", e);
        return;
    }

    // Handle incoming messages
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));

    loop {
        tokio::select! {
            // Handle incoming messages from client
            msg = receiver.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        debug!("Received WebSocket message: {}", text);
                        // Handle client messages (e.g., subscribe to specific events)
                        // For now, we'll just log them
                    }
                    Some(Ok(Message::Close(_))) => {
                        info!("WebSocket connection closed by client");
                        break;
                    }
                    Some(Err(e)) => {
                        error!("WebSocket error: {}", e);
                        break;
                    }
                    None => {
                        info!("WebSocket stream ended");
                        break;
                    }
                    _ => {}
                }
            }
            // Periodic updates
            _ = interval.tick() => {
                // Send periodic updates
                let health = serde_json::json!({
                    "type": "cluster_health",
                    "data": {
                        "status": "green",
                        "number_of_nodes": 1,
                        "number_of_data_nodes": 1,
                        "active_primary_shards": 0,
                        "active_shards": 0,
                    }
                });

                if sender.send(Message::Text(health.to_string())).await.is_err() {
                    break;
                }

                let stats = state.storage.get_cluster_stats(&state.es_version).await;
                let stats_msg = serde_json::json!({
                    "type": "cluster_stats",
                    "data": stats
                });

                if sender.send(Message::Text(stats_msg.to_string())).await.is_err() {
                    break;
                }

                let indices_stats = state.storage.get_indices_stats().await;
                let indices_msg = serde_json::json!({
                    "type": "indices",
                    "data": indices_stats
                });

                if sender.send(Message::Text(indices_msg.to_string())).await.is_err() {
                    break;
                }
            }
        }
    }

    info!("WebSocket connection closed");
}

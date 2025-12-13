//! WebSocket routes

use axum::{
    routing::get,
    Router,
};

use crate::server::{handlers, AppState};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/_ws", get(handlers::websocket_handler))
}

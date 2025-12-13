//! Index refresh routes

use axum::{
    routing::post,
    Router,
};

use crate::server::{handlers, AppState};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/:index/_refresh", post(handlers::refresh_index))
        .route("/_refresh", post(handlers::refresh_all))
}

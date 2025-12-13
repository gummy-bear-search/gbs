//! Bulk operation routes

use axum::{
    routing::post,
    Router,
};

use crate::server::{handlers, AppState};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/:index/_bulk", post(handlers::bulk_operations))
        .route("/_bulk", post(handlers::bulk_operations))
}

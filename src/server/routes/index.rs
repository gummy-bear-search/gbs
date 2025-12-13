//! Index management routes

use axum::{
    routing::{get, put, delete, head},
    Router,
};

use crate::server::{handlers, AppState};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/:index", put(handlers::create_index))
        .route("/:index", head(handlers::check_index))
        .route("/:index", get(handlers::get_index))
        .route("/:index", delete(handlers::delete_index))
        .route("/:index/_mapping", put(handlers::update_mapping))
        .route("/:index/_settings", put(handlers::update_settings))
}

//! Document operation routes

use axum::{
    routing::{get, post, put, delete},
    Router,
};

use crate::server::{handlers, AppState};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/:index/_doc/:id", put(handlers::index_document))
        .route("/:index/_doc/:id", get(handlers::get_document))
        .route("/:index/_doc/:id", delete(handlers::delete_document))
        .route("/:index/_doc", post(handlers::create_document))
}

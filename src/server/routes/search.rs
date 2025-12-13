//! Search operation routes

use axum::{
    routing::{get, post},
    Router,
};

use crate::server::{handlers, AppState};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/:index/_search", get(handlers::search_get))
        .route("/:index/_search", post(handlers::search_post))
        .route("/_search", post(handlers::search_multi_index))
}

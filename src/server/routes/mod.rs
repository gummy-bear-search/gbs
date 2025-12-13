//! Route definitions for Gummy Search API
//!
//! Routes are organized by category into separate modules for better maintainability.

mod web;
mod cluster;
mod index;
mod document;
mod search;
mod bulk;
mod refresh;
mod websocket;

use axum::Router;
use tower_http::{
    cors::CorsLayer,
    trace::TraceLayer,
    services::ServeDir,
};

use crate::server::AppState;

/// Create the main router with all routes
pub fn create_router(state: AppState) -> Router {
    Router::new()
        .merge(web::routes())
        .merge(cluster::routes())
        .merge(index::routes())
        .merge(document::routes())
        .merge(search::routes())
        .merge(bulk::routes())
        .merge(refresh::routes())
        .merge(websocket::routes())
        .nest_service("/static", ServeDir::new("static"))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

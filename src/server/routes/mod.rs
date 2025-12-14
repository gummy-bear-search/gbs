//! Route definitions for Gummy Bear Search API
//!
//! Routes are organized by category into separate modules for better maintainability.

mod bulk;
mod cluster;
mod document;
mod index;
mod refresh;
mod search;
mod web;
mod websocket;

use axum::Router;
use tower_http::{cors::CorsLayer, services::ServeDir, trace::TraceLayer};

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

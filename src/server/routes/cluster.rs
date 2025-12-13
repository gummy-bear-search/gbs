//! Cluster management routes

use axum::{
    routing::get,
    Router,
};

use crate::server::{handlers, AppState};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/_cluster/health", get(handlers::cluster_health))
        .route("/_cluster/stats", get(handlers::cluster_stats))
        .route("/_cat/indices", get(handlers::cat_indices))
        .route("/_aliases", get(handlers::get_aliases))
}

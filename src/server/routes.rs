//! Route definitions for Gummy Search API

use axum::{
    routing::{get, post, put, delete, head},
    Router,
};
use tower_http::{
    cors::CorsLayer,
    trace::TraceLayer,
    services::ServeDir,
};

use crate::server::{AppState, handlers, root, web_index};

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/", get(root))
        .route("/web/", get(web_index))
        .route("/web", get(web_index))
        .nest_service("/static", ServeDir::new("static"))
        .route("/_cluster/health", get(handlers::cluster_health))
        .route("/_cluster/stats", get(handlers::cluster_stats))
        .route("/_cat/indices", get(handlers::cat_indices))
        .route("/_aliases", get(handlers::get_aliases))
        .route("/:index", put(handlers::create_index))
        .route("/:index", head(handlers::check_index))
        .route("/:index", get(handlers::get_index))
        .route("/:index", delete(handlers::delete_index))
        .route("/:index/_mapping", put(handlers::update_mapping))
        .route("/:index/_settings", put(handlers::update_settings))
        .route("/:index/_doc/:id", put(handlers::index_document))
        .route("/:index/_doc/:id", get(handlers::get_document))
        .route("/:index/_doc/:id", delete(handlers::delete_document))
        .route("/:index/_doc", post(handlers::create_document))
        .route("/:index/_bulk", post(handlers::bulk_operations))
        .route("/_bulk", post(handlers::bulk_operations))
        .route("/:index/_search", get(handlers::search_get))
        .route("/:index/_search", post(handlers::search_post))
        .route("/_search", post(handlers::search_multi_index))
        .route("/:index/_refresh", post(handlers::refresh_index))
        .route("/_refresh", post(handlers::refresh_all))
        .route("/_ws", get(handlers::websocket_handler))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

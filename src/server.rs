use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post, put, delete, head},
    Router,
};
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::info;

use crate::error::{GummySearchError, Result};
use crate::storage::Storage;

#[derive(Clone)]
pub struct AppState {
    pub storage: Arc<Storage>,
}

pub async fn create_app(state: AppState) -> Router {
    Router::new()
        .route("/", get(root))
        .route("/_cluster/health", get(cluster_health))
        .route("/:index", put(create_index))
        .route("/:index", head(check_index))
        .route("/:index", get(get_index))
        .route("/:index", delete(delete_index))
        .route("/:index/_mapping", put(update_mapping))
        .route("/:index/_settings", put(update_settings))
        .route("/:index/_doc/:id", put(index_document))
        .route("/:index/_doc/:id", get(get_document))
        .route("/:index/_doc/:id", delete(delete_document))
        .route("/:index/_doc", post(create_document))
        .route("/:index/_bulk", post(bulk_operations))
        .route("/_bulk", post(bulk_operations))
        .route("/:index/_search", get(search_get))
        .route("/:index/_search", post(search_post))
        .route("/_search", post(search_multi_index))
        .route("/:index/_refresh", post(refresh_index))
        .route("/_refresh", post(refresh_all))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

async fn root() -> &'static str {
    "Gummy Search - Elasticsearch-compatible search engine"
}

#[axum::debug_handler]
async fn cluster_health(State(_state): State<AppState>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "green",
        "number_of_nodes": 1,
        "number_of_data_nodes": 1,
        "active_primary_shards": 0,
        "active_shards": 0,
        "relocating_shards": 0,
        "initializing_shards": 0,
        "unassigned_shards": 0,
        "delayed_unassigned_shards": 0,
        "number_of_pending_tasks": 0,
        "number_of_in_flight_fetch": 0,
        "task_max_waiting_in_queue_millis": 0,
        "active_shards_percent_as_number": 100.0
    }))
}

async fn create_index(
    State(state): State<AppState>,
    Path(index): Path<String>,
    body: Option<Json<serde_json::Value>>,
) -> Result<StatusCode> {
    info!("Creating index: {}", index);

    let settings = body.as_ref().and_then(|b| b.get("settings").cloned());
    let mappings = body.as_ref().and_then(|b| b.get("mappings").cloned());

    state.storage.create_index(&index, settings, mappings).await?;

    Ok(StatusCode::OK)
}

async fn check_index(
    State(state): State<AppState>,
    Path(index): Path<String>,
) -> StatusCode {
    match state.storage.index_exists(&index).await {
        Ok(true) => StatusCode::OK,
        _ => StatusCode::NOT_FOUND,
    }
}

async fn get_index(
    State(state): State<AppState>,
    Path(index): Path<String>,
) -> Result<Json<serde_json::Value>> {
    let index_info = state.storage.get_index(&index).await?;
    Ok(Json(index_info))
}

async fn delete_index(
    State(state): State<AppState>,
    Path(index): Path<String>,
) -> Result<StatusCode> {
    if index == "_all" {
        // TODO: Implement delete all indices
        return Err(GummySearchError::InvalidRequest(
            "DELETE /_all not yet implemented".to_string(),
        ));
    }

    state.storage.delete_index(&index).await?;
    Ok(StatusCode::OK)
}

async fn update_mapping(
    State(_state): State<AppState>,
    Path(index): Path<String>,
    _body: Json<serde_json::Value>,
) -> Result<StatusCode> {
    info!("Updating mapping for index: {}", index);
    // TODO: Implement mapping update
    Err(GummySearchError::InvalidRequest(
        "Mapping update not yet implemented".to_string(),
    ))
}

async fn update_settings(
    State(_state): State<AppState>,
    Path(index): Path<String>,
    _body: Json<serde_json::Value>,
) -> Result<StatusCode> {
    info!("Updating settings for index: {}", index);
    // TODO: Implement settings update
    Err(GummySearchError::InvalidRequest(
        "Settings update not yet implemented".to_string(),
    ))
}

async fn index_document(
    State(state): State<AppState>,
    Path((index, id)): Path<(String, String)>,
    body: Json<serde_json::Value>,
) -> Result<StatusCode> {
    info!("Indexing document {} in index {}", id, index);
    state.storage.index_document(&index, &id, body.0).await?;
    Ok(StatusCode::CREATED)
}

async fn create_document(
    State(state): State<AppState>,
    Path(index): Path<String>,
    body: Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>> {
    info!("Creating document in index {}", index);
    let id = state.storage.create_document(&index, body.0).await?;
    Ok(Json(serde_json::json!({
        "_index": index,
        "_type": "_doc",
        "_id": id,
        "_version": 1,
        "result": "created"
    })))
}

async fn get_document(
    State(state): State<AppState>,
    Path((index, id)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>> {
    let doc = state.storage.get_document(&index, &id).await?;
    Ok(Json(doc))
}

async fn delete_document(
    State(state): State<AppState>,
    Path((index, id)): Path<(String, String)>,
) -> Result<StatusCode> {
    state.storage.delete_document(&index, &id).await?;
    Ok(StatusCode::OK)
}

async fn bulk_operations(
    State(_state): State<AppState>,
    Path(index): Path<Option<String>>,
    body: String,
) -> Result<Json<serde_json::Value>> {
    info!("Bulk operations for index: {:?}", index);
    // TODO: Implement bulk operations
    Err(GummySearchError::InvalidRequest(
        "Bulk operations not yet implemented".to_string(),
    ))
}

async fn search_get(
    State(_state): State<AppState>,
    Path(index): Path<String>,
) -> Result<Json<serde_json::Value>> {
    info!("Search GET for index: {}", index);
    // TODO: Implement search
    Err(GummySearchError::InvalidRequest(
        "Search not yet implemented".to_string(),
    ))
}

async fn search_post(
    State(_state): State<AppState>,
    Path(index): Path<String>,
    body: Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>> {
    info!("Search POST for index: {}", index);
    // TODO: Implement search
    Err(GummySearchError::InvalidRequest(
        "Search not yet implemented".to_string(),
    ))
}

async fn search_multi_index(
    State(_state): State<AppState>,
    body: Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>> {
    info!("Multi-index search");
    // TODO: Implement multi-index search
    Err(GummySearchError::InvalidRequest(
        "Multi-index search not yet implemented".to_string(),
    ))
}

async fn refresh_index(
    State(_state): State<AppState>,
    Path(index): Path<String>,
) -> Result<StatusCode> {
    info!("Refreshing index: {}", index);
    // TODO: Implement refresh
    Ok(StatusCode::OK)
}

async fn refresh_all(State(_state): State<AppState>) -> Result<StatusCode> {
    info!("Refreshing all indices");
    // TODO: Implement refresh all
    Ok(StatusCode::OK)
}

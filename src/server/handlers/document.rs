//! Document management handlers

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use tracing::{info, debug};

use crate::error::Result;
use crate::server::AppState;

pub async fn index_document(
    State(state): State<AppState>,
    Path((index, id)): Path<(String, String)>,
    body: Json<serde_json::Value>,
) -> Result<StatusCode> {
    info!("Indexing document {} in index {}", id, index);
    state.storage.index_document(&index, &id, body.0).await?;
    Ok(StatusCode::CREATED)
}

pub async fn create_document(
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

pub async fn get_document(
    State(state): State<AppState>,
    Path((index, id)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>> {
    debug!("Getting document '{}' from index '{}'", id, index);
    let doc = state.storage.get_document(&index, &id).await?;
    debug!("Document '{}' retrieved successfully", id);
    Ok(Json(doc))
}

pub async fn delete_document(
    State(state): State<AppState>,
    Path((index, id)): Path<(String, String)>,
) -> Result<StatusCode> {
    state.storage.delete_document(&index, &id).await?;
    Ok(StatusCode::OK)
}

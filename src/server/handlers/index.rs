//! Index management handlers

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use tracing::{info, debug, error};

use crate::error::{GummySearchError, Result};
use crate::server::AppState;

pub async fn create_index(
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

pub async fn check_index(
    State(state): State<AppState>,
    Path(index): Path<String>,
) -> StatusCode {
    debug!("Checking existence of index: {}", index);
    match state.storage.index_exists(&index).await {
        Ok(true) => {
            debug!("Index '{}' exists", index);
            StatusCode::OK
        }
        Ok(false) => {
            debug!("Index '{}' does not exist", index);
            StatusCode::NOT_FOUND
        }
        Err(e) => {
            error!("Error checking index '{}': {}", index, e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

pub async fn get_index(
    State(state): State<AppState>,
    Path(index): Path<String>,
) -> Result<Json<serde_json::Value>> {
    let index_info = state.storage.get_index(&index).await?;
    Ok(Json(index_info))
}

pub async fn delete_index(
    State(state): State<AppState>,
    Path(index): Path<String>,
) -> Result<StatusCode> {
    if index == "_all" {
        // Delete all indices - dangerous operation
        state.storage.delete_all_indices().await?;
        Ok(StatusCode::OK)
    } else {
        state.storage.delete_index(&index).await?;
        Ok(StatusCode::OK)
    }
}

pub async fn update_mapping(
    State(state): State<AppState>,
    Path(index): Path<String>,
    body: Json<serde_json::Value>,
) -> Result<StatusCode> {
    info!("Updating mapping for index: {}", index);

    // Extract mappings from body
    let new_mappings = body.get("properties")
        .or_else(|| body.get("mappings").and_then(|m| m.get("properties")))
        .cloned();

    if let Some(mappings) = new_mappings {
        state.storage.update_mapping(&index, mappings).await?;
        Ok(StatusCode::OK)
    } else {
        Err(GummySearchError::InvalidRequest(
            "Missing 'properties' or 'mappings.properties' in request body".to_string(),
        ))
    }
}

pub async fn update_settings(
    State(state): State<AppState>,
    Path(index): Path<String>,
    body: Json<serde_json::Value>,
) -> Result<StatusCode> {
    info!("Updating settings for index: {}", index);

    state.storage.update_settings(&index, body.0).await?;
    Ok(StatusCode::OK)
}

pub async fn refresh_index(
    State(_state): State<AppState>,
    Path(index): Path<String>,
) -> Result<StatusCode> {
    info!("Refreshing index: {}", index);
    // Refresh is a no-op for in-memory storage
    // In a persistent storage backend, this would flush changes to disk
    // For now, we just return OK as changes are immediately available
    Ok(StatusCode::OK)
}

pub async fn refresh_all(State(_state): State<AppState>) -> Result<StatusCode> {
    info!("Refreshing all indices");
    // Refresh all is a no-op for in-memory storage
    // In a persistent storage backend, this would flush all changes to disk
    Ok(StatusCode::OK)
}

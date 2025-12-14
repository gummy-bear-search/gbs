//! Bulk operations handler

use axum::{
    extract::{Path, State, Query},
    response::Json,
    body::Body,
};
use std::collections::{HashMap, HashSet};
use tracing::{info, debug, warn};

use crate::error::Result;
use crate::server::AppState;
use crate::bulk_ops::{
    parse_bulk_ndjson, BulkAction, BulkResponse, BulkItemResponse,
    BulkOperationResult, BulkError, ShardsInfo,
};

pub async fn bulk_operations(
    State(state): State<AppState>,
    Path(index): Path<Option<String>>,
    Query(params): Query<HashMap<String, String>>,
    body: Body,
) -> Result<Json<BulkResponse>> {
    info!("Bulk operations for index: {:?}", index);

    // Convert body to String
    let body_bytes = axum::body::to_bytes(body, usize::MAX).await
        .map_err(|e| crate::error::GummySearchError::InvalidRequest(format!("Failed to read body: {}", e)))?;
    let body_str = String::from_utf8(body_bytes.to_vec())
        .map_err(|e| crate::error::GummySearchError::InvalidRequest(format!("Invalid UTF-8 in body: {}", e)))?;

    debug!("Bulk request body length: {} bytes", body_str.len());

    // Check refresh parameter
    let refresh = params.get("refresh")
        .map(|s| s.as_str())
        .unwrap_or("false");

    let start_time = std::time::Instant::now();
    let actions = parse_bulk_ndjson(&body_str, index.as_deref())?;

    let mut items = Vec::new();
    let mut has_errors = false;
    let mut affected_indices = HashSet::new();

    for action in actions {
        // Extract action type and identifiers before executing
        let (action_type, index_name, id) = match &action {
            BulkAction::Index { index, id, .. } => {
                affected_indices.insert(index.clone());
                ("index", index.clone(), id.clone())
            },
            BulkAction::Create { index, id, .. } => {
                affected_indices.insert(index.clone());
                ("create", index.clone(), id.clone())
            },
            BulkAction::Update { index, id, .. } => {
                affected_indices.insert(index.clone());
                ("update", index.clone(), Some(id.clone()))
            },
            BulkAction::Delete { index, id, .. } => {
                affected_indices.insert(index.clone());
                ("delete", index.clone(), Some(id.clone()))
            },
        };

        let result = match state.storage.execute_bulk_action(action).await {
            Ok((idx_name, doc_id, status, result)) => {
                BulkOperationResult {
                    index: idx_name,
                    r#type: "_doc".to_string(),
                    id: doc_id,
                    version: Some(1),
                    result,
                    shards: Some(ShardsInfo {
                        total: 1,
                        successful: 1,
                        failed: 0,
                    }),
                    status,
                    error: None,
                }
            }
            Err(e) => {
                has_errors = true;
                let doc_id = id.unwrap_or_else(|| "unknown".to_string());

                BulkOperationResult {
                    index: index_name,
                    r#type: "_doc".to_string(),
                    id: doc_id,
                    version: None,
                    result: None,
                    shards: Some(ShardsInfo {
                        total: 1,
                        successful: 0,
                        failed: 1,
                    }),
                    status: 400,
                    error: Some(BulkError {
                        r#type: "invalid_request_exception".to_string(),
                        reason: e.to_string(),
                    }),
                }
            }
        };

        let item_response = match action_type {
            "index" => BulkItemResponse::Index { index: result },
            "create" => BulkItemResponse::Create { create: result },
            "update" => BulkItemResponse::Update { update: result },
            "delete" => BulkItemResponse::Delete { delete: result },
            _ => unreachable!(),
        };

        items.push(item_response);
    }

    let took = start_time.elapsed().as_millis() as u32;

    // Handle refresh parameter
    if refresh == "true" || refresh == "wait_for" {
        debug!("Refreshing {} indices after bulk operations", affected_indices.len());
        // Flush to persistent storage if available
        if let Err(e) = state.storage.flush().await {
            warn!("Failed to flush after bulk operations: {}", e);
        }
        // Refresh each affected index
        for index_name in &affected_indices {
            if let Err(e) = state.storage.refresh_index(index_name).await {
                warn!("Failed to refresh index '{}' after bulk operations: {}", index_name, e);
            }
        }
        debug!("Refresh completed for bulk operations");
    }

    Ok(Json(BulkResponse {
        took,
        errors: has_errors,
        items,
    }))
}

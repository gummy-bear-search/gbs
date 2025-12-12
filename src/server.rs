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
use tracing::{info, debug, error, warn};

use crate::error::{GummySearchError, Result};
use crate::storage::Storage;
use crate::bulk_ops::{parse_bulk_ndjson, BulkAction, BulkResponse, BulkItemResponse, BulkOperationResult, BulkError, ShardsInfo};

#[derive(Clone)]
pub struct AppState {
    pub storage: Arc<Storage>,
}

pub async fn create_app(state: AppState) -> Router {
    Router::new()
        .route("/", get(root))
        .route("/_cluster/health", get(cluster_health))
        .route("/_cluster/stats", get(cluster_stats))
        .route("/_cat/indices", get(cat_indices))
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

async fn cluster_stats(State(state): State<AppState>) -> Result<Json<serde_json::Value>> {
    info!("Getting cluster statistics");
    let stats = state.storage.get_cluster_stats().await;
    Ok(Json(stats))
}

async fn cat_indices(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<String> {
    info!("Getting indices list (cat format)");
    let stats = state.storage.get_indices_stats().await;

    // Check if verbose mode (v parameter)
    let verbose = params.contains_key("v");

    if verbose {
        // Header row
        let mut output = String::from("health status index uuid pri rep docs.count store.size\n");

        // Data rows
        for (name, doc_count) in stats {
            output.push_str(&format!(
                "green   open   {}   -   1   0   {}b\n",
                name, doc_count
            ));
        }

        Ok(output)
    } else {
        // Simple format: just index names
        let output: Vec<String> = stats.iter().map(|(name, _)| name.clone()).collect();
        Ok(output.join("\n") + "\n")
    }
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
        // Delete all indices - dangerous operation
        state.storage.delete_all_indices().await?;
        Ok(StatusCode::OK)
    } else {
        state.storage.delete_index(&index).await?;
        Ok(StatusCode::OK)
    }
}

async fn update_mapping(
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

async fn update_settings(
    State(state): State<AppState>,
    Path(index): Path<String>,
    body: Json<serde_json::Value>,
) -> Result<StatusCode> {
    info!("Updating settings for index: {}", index);

    // Extract settings from body
    let new_settings = body.get("settings")
        .cloned()
        .or_else(|| Some(body.0.clone()));

    if let Some(settings) = new_settings {
        state.storage.update_settings(&index, settings).await?;
        Ok(StatusCode::OK)
    } else {
        Err(GummySearchError::InvalidRequest(
            "Missing 'settings' in request body".to_string(),
        ))
    }
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
    debug!("Getting document '{}' from index '{}'", id, index);
    let doc = state.storage.get_document(&index, &id).await?;
    debug!("Document '{}' retrieved successfully", id);
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
    State(state): State<AppState>,
    Path(index): Path<Option<String>>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
    body: String,
) -> Result<Json<BulkResponse>> {
    info!("Bulk operations for index: {:?}", index);
    debug!("Bulk request body length: {} bytes", body.len());

    // Check refresh parameter
    let refresh = params.get("refresh")
        .map(|s| s.as_str())
        .unwrap_or("false");

    let start_time = std::time::Instant::now();
    let actions = parse_bulk_ndjson(&body, index.as_deref())?;

    let mut items = Vec::new();
    let mut has_errors = false;
    let mut affected_indices = std::collections::HashSet::new();

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

async fn search_get(
    State(state): State<AppState>,
    Path(index): Path<String>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>> {
    info!("Search GET for index: {}", index);
    debug!("Search query parameters: {:?}", params);

    // Parse query from query parameters or use match_all
    let query = if let Some(q) = params.get("q") {
        debug!("Using query string: {}", q);
        // Simple query string: search in all fields
        serde_json::json!({
            "match": {
                "_all": q
            }
        })
    } else {
        debug!("No query string, using match_all");
        // Default to match_all
        serde_json::json!({
            "match_all": {}
        })
    };

    let from = params.get("from").and_then(|s| s.parse::<u32>().ok());
    let size = params.get("size").and_then(|s| s.parse::<u32>().ok());
    let sort = None; // TODO: Parse sort from query params if needed
    let source_filter = None; // TODO: Parse _source from query params if needed
    let highlight = None; // TODO: Parse highlight from query params if needed

    let result = state.storage.search(&index, &query, from, size, sort, source_filter, highlight).await?;
    Ok(Json(result))
}

async fn search_post(
    State(state): State<AppState>,
    Path(index): Path<String>,
    body: Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>> {
    info!("Search POST for index: {}", index);
    debug!("Search query: {}", serde_json::to_string(&body.0).unwrap_or_default());

    let query = body.get("query").cloned().unwrap_or_else(|| {
        serde_json::json!({ "match_all": {} })
    });
    let from = body.get("from").and_then(|v| v.as_u64()).map(|v| v as u32);
    let size = body.get("size").and_then(|v| v.as_u64()).map(|v| v as u32);
    let sort = body.get("sort");
    let source_filter = body.get("_source");
    let highlight = body.get("highlight");

    let result = state.storage.search(&index, &query, from, size, sort, source_filter, highlight).await?;
    Ok(Json(result))
}

async fn search_multi_index(
    State(state): State<AppState>,
    body: Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>> {
    info!("Multi-index search");

    let query = body.get("query").cloned().unwrap_or_else(|| {
        serde_json::json!({ "match_all": {} })
    });
    let from = body.get("from").and_then(|v| v.as_u64()).map(|v| v as u32);
    let size = body.get("size").and_then(|v| v.as_u64()).map(|v| v as u32);
    let sort = body.get("sort");
    let source_filter = body.get("_source");
    let highlight = body.get("highlight");

    // Determine which indices to search
    let index_names = if let Some(index_spec) = body.get("index") {
        // Support index specification in request body
        if let Some(index_str) = index_spec.as_str() {
            // Handle comma-separated list or wildcard pattern
            if index_str.contains(',') {
                // Comma-separated list: "index1,index2"
                index_str.split(',')
                    .map(|s| s.trim().to_string())
                    .collect()
            } else if index_str.contains('*') || index_str.contains('?') {
                // Wildcard pattern: "logs-*"
                state.storage.match_indices(index_str).await
            } else {
                // Single index name
                vec![index_str.to_string()]
            }
        } else if let Some(index_array) = index_spec.as_array() {
            // Array of index names: ["index1", "index2"]
            index_array.iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.to_string())
                .collect()
        } else {
            // Default: search all indices
            state.storage.list_indices().await
        }
    } else {
        // No index specified: search all indices
        state.storage.list_indices().await
    };

    debug!("Searching {} indices: {:?}", index_names.len(), index_names);

    let mut all_hits: Vec<serde_json::Value> = Vec::new();
    let mut total_took = 0u32;

    for index_name in &index_names {
        let index_result = state.storage.search(index_name, &query, None, None, sort, source_filter, highlight).await?;
        if let Some(hits) = index_result.get("hits")
            .and_then(|h| h.get("hits"))
            .and_then(|h| h.as_array()) {
            all_hits.extend_from_slice(hits);
        }
        if let Some(took) = index_result.get("took").and_then(|t| t.as_u64()) {
            total_took = total_took.max(took as u32);
        }
    }

    // Sort all hits by score
    all_hits.sort_by(|a, b| {
        let score_a = a.get("_score").and_then(|s| s.as_f64()).unwrap_or(0.0);
        let score_b = b.get("_score").and_then(|s| s.as_f64()).unwrap_or(0.0);
        score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
    });

    // Apply pagination
    let from_val = from.unwrap_or(0) as usize;
    let size_val = size.unwrap_or(10) as usize;
    let total = all_hits.len();
    let paginated_hits: Vec<_> = all_hits.into_iter()
        .skip(from_val)
        .take(size_val)
        .collect();

    let max_score = paginated_hits.first()
        .and_then(|h| h.get("_score").and_then(|s| s.as_f64()));

    Ok(Json(serde_json::json!({
        "took": total_took,
        "timed_out": false,
        "_shards": {
            "total": index_names.len() as u32,
            "successful": index_names.len() as u32,
            "skipped": 0,
            "failed": 0
        },
        "hits": {
            "total": {
                "value": total,
                "relation": "eq"
            },
            "max_score": max_score,
            "hits": paginated_hits
        }
    })))
}

async fn refresh_index(
    State(_state): State<AppState>,
    Path(index): Path<String>,
) -> Result<StatusCode> {
    info!("Refreshing index: {}", index);
    // Refresh is a no-op for in-memory storage
    // In a persistent storage backend, this would flush changes to disk
    // For now, we just return OK as changes are immediately available
    Ok(StatusCode::OK)
}

async fn refresh_all(State(_state): State<AppState>) -> Result<StatusCode> {
    info!("Refreshing all indices");
    // Refresh all is a no-op for in-memory storage
    // In a persistent storage backend, this would flush all changes to disk
    Ok(StatusCode::OK)
}

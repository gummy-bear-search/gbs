//! Search handlers

use axum::{
    extract::{Path, State, Query},
    response::Json,
};
use std::collections::HashMap;
use tracing::{info, debug};

use crate::error::Result;
use crate::server::AppState;

pub async fn search_get(
    State(state): State<AppState>,
    Path(index): Path<String>,
    Query(params): Query<HashMap<String, String>>,
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

pub async fn search_post(
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

pub async fn search_multi_index(
    State(state): State<AppState>,
    body: Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>> {
    info!("Multi-index search");
    debug!("Search query: {}", serde_json::to_string(&body.0).unwrap_or_default());

    // Extract indices from body or use _all
    let indices = body.get("indices")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect::<Vec<_>>())
        .unwrap_or_else(|| vec!["*".to_string()]);

    let query = body.get("query").cloned().unwrap_or_else(|| {
        serde_json::json!({ "match_all": {} })
    });
    let from = body.get("from").and_then(|v| v.as_u64()).map(|v| v as u32);
    let size = body.get("size").and_then(|v| v.as_u64()).map(|v| v as u32);
    let sort = body.get("sort");
    let source_filter = body.get("_source");
    let highlight = body.get("highlight");

    // Search across all matching indices
    let mut all_hits: Vec<serde_json::Value> = Vec::new();
    let mut total = 0;

    for index_pattern in &indices {
        let matched_indices = state.storage.match_indices(index_pattern).await;
        debug!("Pattern '{}' matched {} indices", index_pattern, matched_indices.len());

        for index_name in matched_indices {
            match state.storage.search(&index_name, &query, from, size, sort, source_filter, highlight).await {
                Ok(result) => {
                    if let Some(hits_obj) = result.get("hits") {
                        if let Some(hits_array) = hits_obj.get("hits").and_then(|h| h.as_array()) {
                            all_hits.extend(hits_array.iter().cloned());
                        }
                        if let Some(total_obj) = hits_obj.get("total") {
                            if let Some(total_val) = total_obj.get("value").and_then(|v| v.as_u64()) {
                                total += total_val as usize;
                            }
                        }
                    }
                }
                Err(e) => {
                    debug!("Error searching index '{}': {}", index_name, e);
                    // Continue with other indices
                }
            }
        }
    }

    // Sort all hits by score (descending)
    all_hits.sort_by(|a, b| {
        let score_a = a.get("_score").and_then(|s| s.as_f64()).unwrap_or(0.0);
        let score_b = b.get("_score").and_then(|s| s.as_f64()).unwrap_or(0.0);
        score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
    });

    // Apply pagination to combined results
    let from_val = from.unwrap_or(0) as usize;
    let size_val = size.unwrap_or(10) as usize;
    let paginated_hits: Vec<_> = all_hits.into_iter()
        .skip(from_val)
        .take(size_val)
        .collect();

    let max_score = paginated_hits.first()
        .and_then(|h| h.get("_score").and_then(|s| s.as_f64()));

    Ok(Json(serde_json::json!({
        "took": 0,
        "timed_out": false,
        "_shards": {
            "total": 1,
            "successful": 1,
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

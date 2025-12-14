//! Cluster management handlers

use axum::{
    extract::{State, Query},
    response::Json,
};
use std::collections::HashMap;
use tracing::info;

use crate::error::Result;
use crate::server::AppState;

#[axum::debug_handler]
pub async fn cluster_health(State(_state): State<AppState>) -> Json<serde_json::Value> {
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

pub async fn cluster_stats(State(state): State<AppState>) -> Result<Json<serde_json::Value>> {
    info!("Getting cluster statistics");
    let stats = state.storage.get_cluster_stats(&state.es_version).await;
    Ok(Json(stats))
}

pub async fn cat_indices(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
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

pub async fn get_aliases(State(state): State<AppState>) -> Result<Json<serde_json::Value>> {
    info!("Getting index aliases");
    let aliases = state.storage.get_aliases().await;
    Ok(Json(aliases))
}

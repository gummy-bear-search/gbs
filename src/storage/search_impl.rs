//! Search implementation for Storage

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info};

use crate::error::{GbsError, Result};
use crate::storage::search::{
    compare_documents, filter_source, highlight_document, score_document,
};
use crate::storage::Index;

/// Search documents in an index
///
/// Supports:
/// - match query (text search in specified field)
/// - match_all query (return all documents)
/// - term query (exact match)
/// - bool query (must, should, must_not, filter)
/// - Pagination (from, size)
/// - Sorting
/// - _source filtering
/// - Highlighting
pub async fn search(
    indices: &Arc<RwLock<HashMap<String, Index>>>,
    index_name: &str,
    query: &serde_json::Value,
    from: Option<u32>,
    size: Option<u32>,
    sort: Option<&serde_json::Value>,
    source_filter: Option<&serde_json::Value>,
    highlight: Option<&serde_json::Value>,
) -> Result<serde_json::Value> {
    debug!(
        "Searching index '{}' with query: {}",
        index_name,
        serde_json::to_string(query).unwrap_or_default()
    );
    let indices_guard = indices.read().await;
    let index = indices_guard.get(index_name).ok_or_else(|| {
        error!("Index '{}' not found for search", index_name);
        GbsError::IndexNotFound(index_name.to_string())
    })?;

    let start_time = std::time::Instant::now();
    let total_docs = index.documents.len();
    debug!(
        "Searching {} documents in index '{}'",
        total_docs, index_name
    );

    // Collect all documents with their IDs
    let mut scored_docs: Vec<(String, serde_json::Value, f64)> = Vec::new();

    for (id, doc) in &index.documents {
        let score = score_document(doc, query)?;
        if score > 0.0 {
            scored_docs.push((id.clone(), doc.clone(), score));
        }
    }

    // Sort by score (descending) first, then apply custom sorting if specified
    scored_docs.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));

    // Apply custom sorting if specified
    if let Some(sort_spec) = sort {
        if let Some(sort_array) = sort_spec.as_array() {
            for sort_item in sort_array.iter().rev() {
                scored_docs.sort_by(|a, b| compare_documents(&a.1, &b.1, sort_item));
            }
        } else if sort_spec.is_object() {
            // Single sort field
            scored_docs.sort_by(|a, b| compare_documents(&a.1, &b.1, sort_spec));
        }
    }

    // Apply pagination
    let from_val = from.unwrap_or(0) as usize;
    let size_val = size.unwrap_or(10) as usize;
    let total = scored_docs.len();
    let paginated_docs: Vec<_> = scored_docs
        .into_iter()
        .skip(from_val)
        .take(size_val)
        .collect();

    let max_score = if paginated_docs.is_empty() {
        None
    } else {
        Some(paginated_docs[0].2)
    };

    // Build hits with _source filtering and highlighting
    let hits: Vec<serde_json::Value> = paginated_docs
        .into_iter()
        .map(|(id, doc, score)| {
            let filtered_source = filter_source(&doc, source_filter);
            let mut hit = serde_json::json!({
                "_index": index_name,
                "_type": "_doc",
                "_id": id,
                "_score": score,
                "_source": filtered_source
            });

            // Add highlighting if configured
            if let Some(highlight_config) = highlight {
                if let Some(highlight_result) = highlight_document(&doc, query, highlight_config) {
                    hit.as_object_mut()
                        .unwrap()
                        .insert("highlight".to_string(), highlight_result);
                }
            }

            hit
        })
        .collect();

    let took = start_time.elapsed().as_millis() as u32;
    let elapsed = start_time.elapsed();

    info!(
        "Search completed for index '{}': {} results in {:?} (from: {}, size: {})",
        index_name,
        total,
        elapsed,
        from.unwrap_or(0),
        size.unwrap_or(10)
    );
    debug!(
        "Search returned {} hits out of {} total documents",
        hits.len(),
        total_docs
    );

    Ok(serde_json::json!({
        "took": took,
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
            "hits": hits
        }
    }))
}

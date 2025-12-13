use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, debug, error};

use crate::error::{GummySearchError, Result};
use crate::bulk_ops::BulkAction;
use crate::storage_backend::SledBackend;

// Declare submodules
mod index;
mod search;
mod index_ops;
mod document_ops;
mod stats;

// Re-export Index
pub use index::Index;

// Import search functionality from submodules
use search::{
    score_document, highlight_document, filter_source, compare_documents
};

// Import operations from submodules
use index_ops::*;
use document_ops::*;
use stats::*;

// Index struct is now in storage/index.rs

#[derive(Clone)]
pub struct Storage {
    indices: Arc<RwLock<HashMap<String, Index>>>,
    pub(crate) backend: Option<Arc<SledBackend>>,
}

impl Storage {
    /// Create a new in-memory storage (no persistence)
    pub fn new() -> Self {
        Self {
            indices: Arc::new(RwLock::new(HashMap::new())),
            backend: None,
        }
    }

    /// Create a new storage with Sled persistence
    pub fn with_sled<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path_str = path.as_ref().to_string_lossy().to_string();
        info!("Initializing Sled storage backend at: {}", path_str);
        let backend = Arc::new(SledBackend::new(path)?);
        info!("Sled storage backend initialized successfully");
        Ok(Self {
            indices: Arc::new(RwLock::new(HashMap::new())),
            backend: Some(backend),
        })
    }

    /// Flush pending writes to disk (for persistent storage)
    pub async fn flush(&self) -> Result<()> {
        if let Some(backend) = &self.backend {
            let backend_clone = backend.clone();
            tokio::task::spawn_blocking(move || {
                backend_clone.flush()
            }).await.map_err(GummySearchError::TaskJoin)??;
        }
        Ok(())
    }

    /// Refresh an index (flush changes to persistent storage)
    pub async fn refresh_index(&self, index_name: &str) -> Result<()> {
        debug!("Refreshing index: {}", index_name);
        // For persistent storage, flush to disk
        self.flush().await?;
        info!("Index '{}' refreshed successfully", index_name);
        Ok(())
    }

    /// Load indices from backend (call this after creating with sled)
    pub async fn load_from_backend(&self) -> Result<()> {
        if let Some(backend) = &self.backend {
            info!("Loading indices from persistent storage");
            let start = std::time::Instant::now();
            let indices_data = tokio::task::spawn_blocking({
                let backend = backend.clone();
                move || {
                    let indices = backend.list_indices()?;
                    debug!("Found {} indices in persistent storage", indices.len());
                    let mut loaded = HashMap::new();

                    for index_name in indices {
                        debug!("Loading index: {}", index_name);
                        if let Some((settings, mappings)) = backend.load_index_metadata(&index_name)? {
                            let mut index = Index::new(index_name.clone(), settings, mappings);

                            let documents = backend.load_all_documents(&index_name)?;
                            let doc_count = documents.len();
                            debug!("Loading {} documents for index: {}", doc_count, index_name);
                            for (doc_id, doc) in documents {
                                index.documents.insert(doc_id, doc);
                            }

                            loaded.insert(index_name.clone(), index);
                            info!("Loaded index '{}' with {} documents", index_name, doc_count);
                        }
                    }

                    Ok::<_, GummySearchError>(loaded)
                }
            }).await.map_err(GummySearchError::TaskJoin)??;

            let mut indices = self.indices.write().await;
            let count = indices_data.len();
            *indices = indices_data;
            let elapsed = start.elapsed();
            info!("Loaded {} indices from persistent storage in {:?}", count, elapsed);
        } else {
            debug!("No persistent backend configured, skipping load");
        }
        Ok(())
    }

    pub async fn create_index(
        &self,
        name: &str,
        settings: Option<serde_json::Value>,
        mappings: Option<serde_json::Value>,
    ) -> Result<()> {
        create_index(&self.indices, &self.backend, name, settings, mappings).await
    }

    pub async fn index_exists(&self, name: &str) -> Result<bool> {
        index_exists(&self.indices, name).await
    }

    pub async fn list_indices(&self) -> Vec<String> {
        list_indices(&self.indices).await
    }

    /// Match index names against a pattern (supports * and ? wildcards)
    pub async fn match_indices(&self, pattern: &str) -> Vec<String> {
        match_indices(&self.indices, pattern).await
    }

    /// Get statistics for all indices
    pub async fn get_indices_stats(&self) -> Vec<(String, usize)> {
        get_indices_stats(&self.indices).await
    }

    /// Get aliases for all indices
    pub async fn get_aliases(&self) -> serde_json::Value {
        get_aliases(&self.indices).await
    }

    /// Get cluster statistics
    pub async fn get_cluster_stats(&self) -> serde_json::Value {
        get_cluster_stats(&self.indices).await
    }

    pub async fn update_mapping(
        &self,
        index_name: &str,
        new_mappings: serde_json::Value,
    ) -> Result<()> {
        update_mapping(&self.indices, &self.backend, index_name, new_mappings).await
    }

    pub async fn update_settings(
        &self,
        index_name: &str,
        new_settings: serde_json::Value,
    ) -> Result<()> {
        update_settings(&self.indices, &self.backend, index_name, new_settings).await
    }

    pub async fn delete_all_indices(&self) -> Result<()> {
        delete_all_indices(&self.indices, &self.backend).await
    }

    pub async fn get_index(&self, name: &str) -> Result<serde_json::Value> {
        get_index(&self.indices, name).await
    }

    pub async fn delete_index(&self, name: &str) -> Result<()> {
        delete_index(&self.indices, &self.backend, name).await
    }

    pub async fn index_document(
        &self,
        index_name: &str,
        id: &str,
        document: serde_json::Value,
    ) -> Result<()> {
        index_document(&self.indices, &self.backend, index_name, id, document).await
    }

    pub async fn create_document(
        &self,
        index_name: &str,
        document: serde_json::Value,
    ) -> Result<String> {
        create_document(&self.indices, &self.backend, index_name, document).await
    }

    pub async fn get_document(
        &self,
        index_name: &str,
        id: &str,
    ) -> Result<serde_json::Value> {
        get_document(&self.indices, index_name, id).await
    }

    pub async fn delete_document(
        &self,
        index_name: &str,
        id: &str,
    ) -> Result<()> {
        delete_document(&self.indices, &self.backend, index_name, id).await
    }

    pub async fn execute_bulk_action(&self, action: BulkAction) -> Result<(String, String, u16, Option<String>)> {
        execute_bulk_action(&self.indices, &self.backend, action).await
    }

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
    pub async fn search(
        &self,
        index_name: &str,
        query: &serde_json::Value,
        from: Option<u32>,
        size: Option<u32>,
        sort: Option<&serde_json::Value>,
        source_filter: Option<&serde_json::Value>,
        highlight: Option<&serde_json::Value>,
    ) -> Result<serde_json::Value> {
        debug!("Searching index '{}' with query: {}", index_name, serde_json::to_string(query).unwrap_or_default());
        let indices = self.indices.read().await;
        let index = indices
            .get(index_name)
            .ok_or_else(|| {
                error!("Index '{}' not found for search", index_name);
                GummySearchError::IndexNotFound(index_name.to_string())
            })?;

        let start_time = std::time::Instant::now();
        let total_docs = index.documents.len();
        debug!("Searching {} documents in index '{}'", total_docs, index_name);

        // Collect all documents with their IDs
        let mut scored_docs: Vec<(String, serde_json::Value, f64)> = Vec::new();

        for (id, doc) in &index.documents {
            let score = score_document(doc, query)?;
            if score > 0.0 {
                scored_docs.push((id.clone(), doc.clone(), score));
            }
        }

        // Sort by score (descending) first, then apply custom sorting if specified
        scored_docs.sort_by(|a, b| {
            b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal)
        });

        // Apply custom sorting if specified
        if let Some(sort_spec) = sort {
            if let Some(sort_array) = sort_spec.as_array() {
                for sort_item in sort_array.iter().rev() {
                    scored_docs.sort_by(|a, b| {
                        compare_documents(&a.1, &b.1, sort_item)
                    });
                }
            } else if sort_spec.is_object() {
                // Single sort field
                scored_docs.sort_by(|a, b| {
                    compare_documents(&a.1, &b.1, sort_spec)
                });
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
                        hit.as_object_mut().unwrap().insert("highlight".to_string(), highlight_result);
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
        debug!("Search returned {} hits out of {} total documents", hits.len(), total_docs);

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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_search_match_query() {
        let storage = Storage::new();

        // Create index
        storage.create_index("test_index", None, None).await.unwrap();

        // Index documents
        storage.index_document("test_index", "1", serde_json::json!({
            "title": "Rust Programming",
            "content": "Learn Rust programming language"
        })).await.unwrap();

        storage.index_document("test_index", "2", serde_json::json!({
            "title": "Python Tutorial",
            "content": "Learn Python programming"
        })).await.unwrap();

        // Search for "Rust"
        let query = serde_json::json!({
            "match": {
                "title": "Rust"
            }
        });

        let result = storage.search("test_index", &query, None, None, None, None, None).await.unwrap();
        let hits = result.get("hits").and_then(|h| h.get("hits")).and_then(|h| h.as_array()).unwrap();

        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].get("_id").and_then(|id| id.as_str()).unwrap(), "1");
    }

    #[tokio::test]
    async fn test_search_match_all() {
        let storage = Storage::new();

        storage.create_index("test_index", None, None).await.unwrap();
        storage.index_document("test_index", "1", serde_json::json!({"title": "Doc 1"})).await.unwrap();
        storage.index_document("test_index", "2", serde_json::json!({"title": "Doc 2"})).await.unwrap();

        let query = serde_json::json!({
            "match_all": {}
        });

        let result = storage.search("test_index", &query, None, None, None, None, None).await.unwrap();
        let hits = result.get("hits").and_then(|h| h.get("hits")).and_then(|h| h.as_array()).unwrap();

        assert_eq!(hits.len(), 2);
    }

    #[tokio::test]
    async fn test_search_pagination() {
        let storage = Storage::new();

        storage.create_index("test_index", None, None).await.unwrap();
        for i in 1..=10 {
            storage.index_document("test_index", &i.to_string(), serde_json::json!({
                "title": format!("Doc {}", i)
            })).await.unwrap();
        }

        let query = serde_json::json!({ "match_all": {} });

        // First page
        let result = storage.search("test_index", &query, Some(0), Some(5), None, None, None).await.unwrap();
        let hits = result.get("hits").and_then(|h| h.get("hits")).and_then(|h| h.as_array()).unwrap();
        assert_eq!(hits.len(), 5);

        // Second page
        let result = storage.search("test_index", &query, Some(5), Some(5), None, None, None).await.unwrap();
        let hits = result.get("hits").and_then(|h| h.get("hits")).and_then(|h| h.as_array()).unwrap();
        assert_eq!(hits.len(), 5);
    }

    #[tokio::test]
    async fn test_search_term_query() {
        let storage = Storage::new();

        storage.create_index("test_index", None, None).await.unwrap();
        storage.index_document("test_index", "1", serde_json::json!({
            "status": "active",
            "name": "Test"
        })).await.unwrap();
        storage.index_document("test_index", "2", serde_json::json!({
            "status": "inactive",
            "name": "Test"
        })).await.unwrap();

        let query = serde_json::json!({
            "term": {
                "status": "active"
            }
        });

        let result = storage.search("test_index", &query, None, None, None, None, None).await.unwrap();
        let hits = result.get("hits").and_then(|h| h.get("hits")).and_then(|h| h.as_array()).unwrap();

        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].get("_id").and_then(|id| id.as_str()).unwrap(), "1");
    }

    #[tokio::test]
    async fn test_search_bool_query() {
        let storage = Storage::new();

        storage.create_index("test_index", None, None).await.unwrap();
        storage.index_document("test_index", "1", serde_json::json!({
            "title": "Rust Guide",
            "status": "published"
        })).await.unwrap();
        storage.index_document("test_index", "2", serde_json::json!({
            "title": "Python Guide",
            "status": "draft"
        })).await.unwrap();

        let query = serde_json::json!({
            "bool": {
                "must": [
                    { "match": { "title": "Guide" } }
                ],
                "filter": [
                    { "term": { "status": "published" } }
                ]
            }
        });

        let result = storage.search("test_index", &query, None, None, None, None, None).await.unwrap();
        let hits = result.get("hits").and_then(|h| h.get("hits")).and_then(|h| h.as_array()).unwrap();

        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].get("_id").and_then(|id| id.as_str()).unwrap(), "1");
    }

    #[tokio::test]
    async fn test_update_mapping() {
        let storage = Storage::new();

        storage.create_index("test_index", None, None).await.unwrap();

        let new_mappings = serde_json::json!({
            "title": { "type": "text" },
            "count": { "type": "integer" }
        });

        storage.update_mapping("test_index", new_mappings.clone()).await.unwrap();

        let index_info = storage.get_index("test_index").await.unwrap();
        let mappings = index_info.get("test_index")
            .and_then(|idx| idx.get("mappings"))
            .and_then(|m| m.get("properties"));

        assert!(mappings.is_some());
    }

    #[tokio::test]
    async fn test_update_settings() {
        let storage = Storage::new();

        storage.create_index("test_index", None, None).await.unwrap();

        let new_settings = serde_json::json!({
            "number_of_shards": 2,
            "number_of_replicas": 1
        });

        storage.update_settings("test_index", new_settings.clone()).await.unwrap();

        let index_info = storage.get_index("test_index").await.unwrap();
        let settings = index_info.get("test_index")
            .and_then(|idx| idx.get("settings"));

        assert!(settings.is_some());
        assert_eq!(settings.and_then(|s| s.get("number_of_shards")).and_then(|v| v.as_u64()).unwrap(), 2);
    }

    #[tokio::test]
    async fn test_delete_all_indices() {
        let storage = Storage::new();

        storage.create_index("index1", None, None).await.unwrap();
        storage.create_index("index2", None, None).await.unwrap();

        assert_eq!(storage.list_indices().await.len(), 2);

        storage.delete_all_indices().await.unwrap();

        assert_eq!(storage.list_indices().await.len(), 0);
    }

    #[tokio::test]
    async fn test_search_sorting() {
        let storage = Storage::new();

        storage.create_index("test_index", None, None).await.unwrap();
        storage.index_document("test_index", "1", serde_json::json!({
            "name": "Alice",
            "age": 30
        })).await.unwrap();
        storage.index_document("test_index", "2", serde_json::json!({
            "name": "Bob",
            "age": 25
        })).await.unwrap();
        storage.index_document("test_index", "3", serde_json::json!({
            "name": "Charlie",
            "age": 35
        })).await.unwrap();

        let query = serde_json::json!({ "match_all": {} });
        let sort = serde_json::json!({
            "age": {
                "order": "asc"
            }
        });

        let result = storage.search("test_index", &query, None, None, Some(&sort), None, None).await.unwrap();
        let hits = result.get("hits").and_then(|h| h.get("hits")).and_then(|h| h.as_array()).unwrap();

        assert_eq!(hits.len(), 3);
        assert_eq!(hits[0].get("_id").and_then(|id| id.as_str()).unwrap(), "2"); // Bob, age 25
        assert_eq!(hits[1].get("_id").and_then(|id| id.as_str()).unwrap(), "1"); // Alice, age 30
        assert_eq!(hits[2].get("_id").and_then(|id| id.as_str()).unwrap(), "3"); // Charlie, age 35
    }

    #[tokio::test]
    async fn test_search_match_phrase() {
        let storage = Storage::new();

        storage.create_index("test_index", None, None).await.unwrap();
        storage.index_document("test_index", "1", serde_json::json!({
            "title": "Rust Programming Guide",
            "content": "Learn Rust programming language"
        })).await.unwrap();

        storage.index_document("test_index", "2", serde_json::json!({
            "title": "Python Tutorial",
            "content": "Learn Python programming"
        })).await.unwrap();

        // Match phrase query - exact phrase match
        let query = serde_json::json!({
            "match_phrase": {
                "title": "Rust Programming"
            }
        });

        let result = storage.search("test_index", &query, None, None, None, None, None).await.unwrap();
        let hits = result.get("hits").and_then(|h| h.get("hits")).and_then(|h| h.as_array()).unwrap();

        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].get("_id").and_then(|id| id.as_str()).unwrap(), "1");
    }

    #[tokio::test]
    async fn test_search_multi_match() {
        let storage = Storage::new();

        storage.create_index("test_index", None, None).await.unwrap();
        storage.index_document("test_index", "1", serde_json::json!({
            "title": "Rust Guide",
            "description": "Learn Rust",
            "tags": "programming"
        })).await.unwrap();

        storage.index_document("test_index", "2", serde_json::json!({
            "title": "Python Tutorial",
            "description": "Learn Python",
            "tags": "tutorial"
        })).await.unwrap();

        // Multi-match query - search across multiple fields
        let query = serde_json::json!({
            "multi_match": {
                "query": "Rust",
                "fields": ["title", "description"]
            }
        });

        let result = storage.search("test_index", &query, None, None, None, None, None).await.unwrap();
        let hits = result.get("hits").and_then(|h| h.get("hits")).and_then(|h| h.as_array()).unwrap();

        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].get("_id").and_then(|id| id.as_str()).unwrap(), "1");
    }

    #[tokio::test]
    async fn test_search_range() {
        let storage = Storage::new();

        storage.create_index("test_index", None, None).await.unwrap();
        storage.index_document("test_index", "1", serde_json::json!({
            "name": "Alice",
            "age": 25
        })).await.unwrap();
        storage.index_document("test_index", "2", serde_json::json!({
            "name": "Bob",
            "age": 30
        })).await.unwrap();
        storage.index_document("test_index", "3", serde_json::json!({
            "name": "Charlie",
            "age": 35
        })).await.unwrap();

        // Range query - age between 28 and 40
        let query = serde_json::json!({
            "range": {
                "age": {
                    "gte": 28,
                    "lte": 40
                }
            }
        });

        let result = storage.search("test_index", &query, None, None, None, None, None).await.unwrap();
        let hits = result.get("hits").and_then(|h| h.get("hits")).and_then(|h| h.as_array()).unwrap();

        assert_eq!(hits.len(), 2);
        let ids: Vec<&str> = hits.iter()
            .map(|h| h.get("_id").and_then(|id| id.as_str()).unwrap())
            .collect();
        assert!(ids.contains(&"2")); // Bob, age 30
        assert!(ids.contains(&"3")); // Charlie, age 35
    }

    #[tokio::test]
    async fn test_search_range_gt_lt() {
        let storage = Storage::new();

        storage.create_index("test_index", None, None).await.unwrap();
        storage.index_document("test_index", "1", serde_json::json!({
            "price": 10.0
        })).await.unwrap();
        storage.index_document("test_index", "2", serde_json::json!({
            "price": 20.0
        })).await.unwrap();
        storage.index_document("test_index", "3", serde_json::json!({
            "price": 30.0
        })).await.unwrap();

        // Range query with gt and lt
        let query = serde_json::json!({
            "range": {
                "price": {
                    "gt": 10.0,
                    "lt": 30.0
                }
            }
        });

        let result = storage.search("test_index", &query, None, None, None, None, None).await.unwrap();
        let hits = result.get("hits").and_then(|h| h.get("hits")).and_then(|h| h.as_array()).unwrap();

        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].get("_id").and_then(|id| id.as_str()).unwrap(), "2"); // price 20.0
    }
}

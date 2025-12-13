use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use tracing::{info, debug, warn, error};
use regex::Regex;
use chrono::Utc;

use crate::error::{GummySearchError, Result};
use crate::bulk_ops::BulkAction;
use crate::storage_backend::SledBackend;

// Declare submodules
mod index;
mod search;

// Re-export Index
pub use index::Index;

// Import search functionality from submodules
use search::{
    score_document, highlight_document, filter_source, compare_documents
};

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
        info!("Creating index: {}", name);
        let mut indices = self.indices.write().await;

        if indices.contains_key(name) {
            warn!("Attempted to create index '{}' that already exists", name);
            return Err(GummySearchError::InvalidRequest(
                format!("Index {} already exists", name),
            ));
        }

        // Persist to backend if available
        if let Some(backend) = &self.backend {
            debug!("Persisting index '{}' to storage backend", name);
            let backend_clone = backend.clone();
            let name_str = name.to_string();
            let settings_clone = settings.clone();
            let mappings_clone = mappings.clone();

            tokio::task::spawn_blocking(move || {
                backend_clone.store_index_metadata(&name_str, settings_clone.as_ref(), mappings_clone.as_ref())
            }).await.map_err(GummySearchError::TaskJoin)??;
            debug!("Index '{}' persisted successfully", name);
        }

        let index = Index::new(name.to_string(), settings, mappings);
        indices.insert(name.to_string(), index);
        info!("Index '{}' created successfully", name);

        Ok(())
    }

    pub async fn index_exists(&self, name: &str) -> Result<bool> {
        let indices = self.indices.read().await;
        Ok(indices.contains_key(name))
    }

    pub async fn list_indices(&self) -> Vec<String> {
        let indices = self.indices.read().await;
        indices.keys().cloned().collect()
    }

    /// Match index names against a pattern (supports * and ? wildcards)
    pub async fn match_indices(&self, pattern: &str) -> Vec<String> {
        let all_indices = self.list_indices().await;

        // Convert pattern to regex
        let mut regex_pattern = String::new();
        for c in pattern.chars() {
            match c {
                '*' => regex_pattern.push_str(".*"),
                '?' => regex_pattern.push('.'),
                '.' => regex_pattern.push_str(r"\."),
                '+' => regex_pattern.push_str(r"\+"),
                '(' => regex_pattern.push_str(r"\("),
                ')' => regex_pattern.push_str(r"\)"),
                '[' => regex_pattern.push_str(r"\["),
                ']' => regex_pattern.push_str(r"\]"),
                '{' => regex_pattern.push_str(r"\{"),
                '}' => regex_pattern.push_str(r"\}"),
                '^' => regex_pattern.push_str(r"\^"),
                '$' => regex_pattern.push_str(r"\$"),
                '|' => regex_pattern.push_str(r"\|"),
                '\\' => regex_pattern.push_str(r"\\"),
                _ => {
                    let mut buf = [0; 4];
                    let s = c.encode_utf8(&mut buf);
                    regex_pattern.push_str(&regex::escape(s));
                }
            }
        }

        let full_pattern = format!("^{}$", regex_pattern);

        match Regex::new(&full_pattern) {
            Ok(re) => {
                all_indices.into_iter()
                    .filter(|name| re.is_match(name))
                    .collect()
            }
            Err(_) => {
                // Invalid pattern, return empty
                Vec::new()
            }
        }
    }

    /// Get statistics for all indices
    pub async fn get_indices_stats(&self) -> Vec<(String, usize)> {
        let indices = self.indices.read().await;
        indices.iter()
            .map(|(name, index)| (name.clone(), index.documents.len()))
            .collect()
    }

    /// Get aliases for all indices
    /// Returns a JSON object mapping index names to their aliases
    pub async fn get_aliases(&self) -> serde_json::Value {
        let indices = self.indices.read().await;
        let mut result = serde_json::Map::new();

        for (index_name, index) in indices.iter() {
            let mut aliases_map = serde_json::Map::new();
            for alias in &index.aliases {
                aliases_map.insert(alias.clone(), serde_json::json!({}));
            }

            result.insert(
                index_name.clone(),
                serde_json::json!({
                    "aliases": serde_json::Value::Object(aliases_map)
                }),
            );
        }

        serde_json::Value::Object(result)
    }

    /// Get cluster statistics
    pub async fn get_cluster_stats(&self) -> serde_json::Value {
        let indices = self.indices.read().await;
        let total_indices = indices.len();
        let total_docs: usize = indices.values().map(|idx| idx.documents.len()).sum();

        serde_json::json!({
            "cluster_name": "gummy-search",
            "cluster_uuid": "gummy-search-cluster",
            "timestamp": Utc::now().timestamp_millis(),
            "status": "green",
            "indices": {
                "count": total_indices,
                "shards": {
                    "total": total_indices,
                    "primaries": total_indices,
                    "replication": 0,
                    "index": {
                        "shards": {
                            "min": 1,
                            "max": 1,
                            "avg": 1.0
                        },
                        "primaries": {
                            "min": 1,
                            "max": 1,
                            "avg": 1.0
                        },
                        "replication": {
                            "min": 0,
                            "max": 0,
                            "avg": 0.0
                        }
                    }
                },
                "docs": {
                    "count": total_docs,
                    "deleted": 0
                },
                "store": {
                    "size_in_bytes": 0,
                    "throttle_time_in_millis": 0
                },
                "fielddata": {
                    "memory_size_in_bytes": 0,
                    "evictions": 0
                },
                "query_cache": {
                    "memory_size_in_bytes": 0,
                    "total_count": 0,
                    "hit_count": 0,
                    "miss_count": 0,
                    "cache_size": 0,
                    "cache_count": 0,
                    "evictions": 0
                },
                "completion": {
                    "size_in_bytes": 0
                },
                "segments": {
                    "count": 0,
                    "memory_in_bytes": 0,
                    "terms_memory_in_bytes": 0,
                    "stored_fields_memory_in_bytes": 0,
                    "term_vectors_memory_in_bytes": 0,
                    "norms_memory_in_bytes": 0,
                    "points_memory_in_bytes": 0,
                    "doc_values_memory_in_bytes": 0,
                    "index_writer_memory_in_bytes": 0,
                    "version_map_memory_in_bytes": 0,
                    "fixed_bit_set_memory_in_bytes": 0,
                    "max_unsafe_auto_id_timestamp": -1,
                    "file_sizes": {}
                },
                "mappings": {
                    "field_types": [],
                    "runtime_field_types": []
                },
                "analysis": {
                    "char_filter_types": [],
                    "tokenizer_types": [],
                    "filter_types": [],
                    "analyzer_types": [],
                    "built_in_char_filters": [],
                    "built_in_tokenizers": [],
                    "built_in_filters": [],
                    "built_in_analyzers": []
                }
            },
            "nodes": {
                "count": {
                    "total": 1,
                    "data": 1,
                    "coordinating_only": 0,
                    "master": 1,
                    "ingest": 1
                },
                "versions": ["8.0.0"],
                "os": {
                    "available_processors": num_cpus::get(),
                    "allocated_processors": num_cpus::get(),
                    "names": []
                },
                "process": {
                    "cpu": {
                        "percent": 0
                    },
                    "open_file_descriptors": {
                        "min": 0,
                        "max": 0,
                        "avg": 0
                    }
                },
                "jvm": {
                    "max_uptime_in_millis": 0,
                    "versions": []
                },
                "fs": {
                    "total_in_bytes": 0,
                    "free_in_bytes": 0,
                    "available_in_bytes": 0
                },
                "plugins": [],
                "network_types": {}
            }
        })
    }

    pub async fn update_mapping(
        &self,
        index_name: &str,
        new_mappings: serde_json::Value,
    ) -> Result<()> {
        info!("Updating mapping for index: {}", index_name);
        debug!("New mapping: {}", serde_json::to_string(&new_mappings).unwrap_or_default());

        let mut indices = self.indices.write().await;
        let index = indices
            .get_mut(index_name)
            .ok_or_else(|| {
                error!("Index '{}' not found when updating mapping", index_name);
                GummySearchError::IndexNotFound(index_name.to_string())
            })?;

        // Update mappings - merge with existing if present
        if let Some(existing_mappings) = &mut index.mappings {
            if let (Some(existing_obj), Some(new_obj)) = (existing_mappings.as_object_mut(), new_mappings.as_object()) {
                // Merge properties
                if let Some(existing_props) = existing_obj.get_mut("properties") {
                    if let Some(existing_props_obj) = existing_props.as_object_mut() {
                        for (key, value) in new_obj {
                            existing_props_obj.insert(key.clone(), value.clone());
                        }
                    }
                } else {
                    // No existing properties, set new ones
                    existing_obj.insert("properties".to_string(), serde_json::Value::Object(new_obj.clone()));
                }
            } else {
                // Replace entire mappings
                index.mappings = Some(new_mappings.clone());
            }
        } else {
            // No existing mappings, set new ones
            index.mappings = Some(serde_json::json!({
                "properties": new_mappings
            }));
        }

        // Persist updated mappings to backend
        if let Some(backend) = &self.backend {
            debug!("Persisting updated mapping for index '{}' to storage backend", index_name);
            let backend_clone = backend.clone();
            let index_name_str = index_name.to_string();
            let final_mappings = index.mappings.clone();
            let settings = index.settings.clone();

            tokio::task::spawn_blocking(move || {
                backend_clone.store_index_metadata(&index_name_str, settings.as_ref(), final_mappings.as_ref())
            }).await.map_err(GummySearchError::TaskJoin)??;
            debug!("Mapping for index '{}' persisted successfully", index_name);
        }

        info!("Mapping updated successfully for index '{}'", index_name);
        Ok(())
    }

    pub async fn update_settings(
        &self,
        index_name: &str,
        new_settings: serde_json::Value,
    ) -> Result<()> {
        info!("Updating settings for index: {}", index_name);
        debug!("New settings: {}", serde_json::to_string(&new_settings).unwrap_or_default());

        let mut indices = self.indices.write().await;
        let index = indices
            .get_mut(index_name)
            .ok_or_else(|| {
                error!("Index '{}' not found when updating settings", index_name);
                GummySearchError::IndexNotFound(index_name.to_string())
            })?;

        // Update settings - merge with existing if present
        if let Some(existing_settings) = &mut index.settings {
            if let (Some(existing_obj), Some(new_obj)) = (existing_settings.as_object_mut(), new_settings.as_object()) {
                // Merge settings
                for (key, value) in new_obj {
                    existing_obj.insert(key.clone(), value.clone());
                }
            } else {
                // Replace entire settings
                index.settings = Some(new_settings.clone());
            }
        } else {
            // No existing settings, set new ones
            index.settings = Some(new_settings.clone());
        }

        // Persist updated settings to backend
        if let Some(backend) = &self.backend {
            debug!("Persisting updated settings for index '{}' to storage backend", index_name);
            let backend_clone = backend.clone();
            let index_name_str = index_name.to_string();
            let final_settings = index.settings.clone();
            let mappings = index.mappings.clone();

            tokio::task::spawn_blocking(move || {
                backend_clone.store_index_metadata(&index_name_str, final_settings.as_ref(), mappings.as_ref())
            }).await.map_err(GummySearchError::TaskJoin)??;
            debug!("Settings for index '{}' persisted successfully", index_name);
        }

        info!("Settings updated successfully for index '{}'", index_name);
        Ok(())
    }

    pub async fn delete_all_indices(&self) -> Result<()> {
        warn!("Deleting all indices - this is a destructive operation!");

        let indices = self.indices.read().await;
        let count = indices.len();
        let index_names: Vec<String> = indices.keys().cloned().collect();
        drop(indices);

        // Delete all from backend if available
        if let Some(backend) = &self.backend {
            debug!("Deleting {} indices from storage backend", count);
            let backend_clone = backend.clone();
            let indices_list = tokio::task::spawn_blocking({
                let backend = backend.clone();
                move || backend.list_indices()
            }).await.map_err(GummySearchError::TaskJoin)??;

            for index_name in indices_list {
                let backend_clone = backend_clone.clone();
                tokio::task::spawn_blocking(move || {
                    backend_clone.delete_index_metadata(&index_name)
                }).await.map_err(GummySearchError::TaskJoin)??;
            }
            debug!("All indices deleted from storage backend");
        }

        let mut indices = self.indices.write().await;
        indices.clear();
        info!("Deleted all {} indices: {:?}", count, index_names);
        Ok(())
    }

    pub async fn get_index(&self, name: &str) -> Result<serde_json::Value> {
        let indices = self.indices.read().await;
        let index = indices
            .get(name)
            .ok_or_else(|| GummySearchError::IndexNotFound(name.to_string()))?;

        Ok(serde_json::json!({
            name: {
                "settings": index.settings,
                "mappings": index.mappings,
                "aliases": {}
            }
        }))
    }

    pub async fn delete_index(&self, name: &str) -> Result<()> {
        info!("Deleting index: {}", name);

        // Delete from backend if available
        if let Some(backend) = &self.backend {
            debug!("Deleting index '{}' from storage backend", name);
            let backend_clone = backend.clone();
            let name_str = name.to_string();

            tokio::task::spawn_blocking(move || {
                backend_clone.delete_index_metadata(&name_str)
            }).await.map_err(GummySearchError::TaskJoin)??;
            debug!("Index '{}' deleted from storage backend", name);
        }

        let mut indices = self.indices.write().await;
        let doc_count = indices.get(name).map(|idx| idx.documents.len()).unwrap_or(0);
        indices
            .remove(name)
            .ok_or_else(|| {
                warn!("Attempted to delete non-existent index: {}", name);
                GummySearchError::IndexNotFound(name.to_string())
            })?;

        info!("Index '{}' deleted successfully (had {} documents)", name, doc_count);
        Ok(())
    }

    pub async fn index_document(
        &self,
        index_name: &str,
        id: &str,
        document: serde_json::Value,
    ) -> Result<()> {
        debug!("Indexing document '{}' in index '{}'", id, index_name);

        // Persist to backend if available
        if let Some(backend) = &self.backend {
            let backend_clone = backend.clone();
            let index_name_str = index_name.to_string();
            let id_str = id.to_string();
            let doc_clone = document.clone();

            tokio::task::spawn_blocking(move || {
                backend_clone.store_document(&index_name_str, &id_str, &doc_clone)
            }).await.map_err(GummySearchError::TaskJoin)??;
            debug!("Document '{}' persisted to storage backend", id);
        }

        let mut indices = self.indices.write().await;
        let index = indices
            .get_mut(index_name)
            .ok_or_else(|| {
                error!("Index '{}' not found when indexing document '{}'", index_name, id);
                GummySearchError::IndexNotFound(index_name.to_string())
            })?;

        index.documents.insert(id.to_string(), document);
        debug!("Document '{}' indexed successfully in index '{}'", id, index_name);
        Ok(())
    }

    pub async fn create_document(
        &self,
        index_name: &str,
        document: serde_json::Value,
    ) -> Result<String> {
        let id = Uuid::new_v4().to_string();
        self.index_document(index_name, &id, document).await?;
        Ok(id)
    }

    pub async fn get_document(
        &self,
        index_name: &str,
        id: &str,
    ) -> Result<serde_json::Value> {
        let indices = self.indices.read().await;
        let index = indices
            .get(index_name)
            .ok_or_else(|| GummySearchError::IndexNotFound(index_name.to_string()))?;

        let doc = index
            .documents
            .get(id)
            .ok_or_else(|| GummySearchError::DocumentNotFound(id.to_string()))?;

        Ok(serde_json::json!({
            "_index": index_name,
            "_type": "_doc",
            "_id": id,
            "_version": 1,
            "_source": doc
        }))
    }

    pub async fn delete_document(
        &self,
        index_name: &str,
        id: &str,
    ) -> Result<()> {
        debug!("Deleting document '{}' from index '{}'", id, index_name);

        // Delete from backend if available
        if let Some(backend) = &self.backend {
            let backend_clone = backend.clone();
            let index_name_str = index_name.to_string();
            let id_str = id.to_string();

            tokio::task::spawn_blocking(move || {
                backend_clone.delete_document(&index_name_str, &id_str)
            }).await.map_err(GummySearchError::TaskJoin)??;
            debug!("Document '{}' deleted from storage backend", id);
        }

        let mut indices = self.indices.write().await;
        let index = indices
            .get_mut(index_name)
            .ok_or_else(|| {
                error!("Index '{}' not found when deleting document '{}'", index_name, id);
                GummySearchError::IndexNotFound(index_name.to_string())
            })?;

        index
            .documents
            .remove(id)
            .ok_or_else(|| {
                warn!("Document '{}' not found in index '{}'", id, index_name);
                GummySearchError::DocumentNotFound(id.to_string())
            })?;

        info!("Document '{}' deleted from index '{}'", id, index_name);
        Ok(())
    }

    pub async fn execute_bulk_action(&self, action: BulkAction) -> Result<(String, String, u16, Option<String>)> {
        match action {
            BulkAction::Index { index, id, document } => {
                let doc_id = id.unwrap_or_else(|| Uuid::new_v4().to_string());
                self.index_document(&index, &doc_id, document).await?;
                Ok((index, doc_id, 201, Some("created".to_string())))
            }
            BulkAction::Create { index, id, document } => {
                let doc_id = id.unwrap_or_else(|| Uuid::new_v4().to_string());
                // Check if document exists
                if self.index_exists(&index).await? {
                    let indices = self.indices.read().await;
                    if let Some(idx) = indices.get(&index) {
                        if idx.documents.contains_key(&doc_id) {
                            return Err(GummySearchError::InvalidRequest(
                                format!("Document {} already exists", doc_id)
                            ));
                        }
                    }
                }
                self.index_document(&index, &doc_id, document).await?;
                Ok((index, doc_id, 201, Some("created".to_string())))
            }
            BulkAction::Update { index, id, document } => {
                // For update, we merge with existing document or create new
                let indices = self.indices.read().await;
                let existing = indices.get(&index)
                    .and_then(|idx| idx.documents.get(&id).cloned());
                drop(indices);

                let updated_doc = if let Some(mut existing_doc) = existing {
                    // Merge: if document is an object, merge fields
                    if let (Some(existing_obj), Some(new_obj)) = (existing_doc.as_object_mut(), document.as_object()) {
                        for (k, v) in new_obj {
                            existing_obj.insert(k.clone(), v.clone());
                        }
                        serde_json::Value::Object(existing_obj.clone())
                    } else {
                        document
                    }
                } else {
                    document
                };

                self.index_document(&index, &id, updated_doc).await?;
                Ok((index, id, 200, Some("updated".to_string())))
            }
            BulkAction::Delete { index, id } => {
                self.delete_document(&index, &id).await?;
                Ok((index, id, 200, Some("deleted".to_string())))
            }
        }
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

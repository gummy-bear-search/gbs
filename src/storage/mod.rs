//! Storage module for Gummy Search
//!
//! This module provides the main Storage struct and public API for managing
//! indices, documents, and search operations.

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

use crate::error::Result;
use crate::bulk_ops::BulkAction;
use crate::storage_backend::SledBackend;

// Declare submodules
mod index;
mod search;
mod index_ops;
mod document_ops;
mod stats;
mod persistence;
mod search_impl;

// Re-export Index
pub use index::Index;

// Search functionality is used in search_impl.rs, not here

// Import operations from submodules
use index_ops::*;
use document_ops::*;
use stats::*;
use persistence::*;
use search_impl::*;

/// Main Storage struct for Gummy Search
///
/// Manages indices, documents, and provides search functionality.
/// Supports both in-memory and persistent (Sled) storage backends.
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
        flush(&self.backend).await
    }

    /// Refresh an index (flush changes to persistent storage)
    pub async fn refresh_index(&self, index_name: &str) -> Result<()> {
        refresh_index(&self.indices, &self.backend, index_name).await
    }

    /// Load indices from backend (call this after creating with sled)
    pub async fn load_from_backend(&self) -> Result<()> {
        load_from_backend(&self.indices, &self.backend).await
    }

    // Index operations
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
    pub async fn get_cluster_stats(&self, es_version: &str) -> serde_json::Value {
        get_cluster_stats(&self.indices, es_version).await
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

    // Document operations
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
    /// - Highlighting
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
        search(&self.indices, index_name, query, from, size, sort, source_filter, highlight).await
    }
}

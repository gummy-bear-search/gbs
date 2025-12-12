use crate::error::{GummySearchError, Result};
use serde_json;
use sled::Db;
use std::path::Path;
use std::sync::Arc;

/// Key prefixes for different data types
const INDEX_PREFIX: &str = "index:";
const DOC_PREFIX: &str = "doc:";

/// Convert sled error to GummySearchError
fn sled_error(e: sled::Error) -> GummySearchError {
    GummySearchError::Storage(format!("Sled error: {}", e))
}

/// Sled-based persistent storage backend
pub struct SledBackend {
    db: Arc<Db>,
}

impl SledBackend {
    /// Create a new Sled backend with the given data directory
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let db = sled::open(path)
            .map_err(|e| GummySearchError::Storage(format!("Failed to open sled database: {}", e)))?;
        Ok(Self {
            db: Arc::new(db),
        })
    }

    /// Get the sled database instance
    pub fn db(&self) -> &Db {
        &self.db
    }

    /// Store index metadata
    pub fn store_index_metadata(
        &self,
        index_name: &str,
        settings: Option<&serde_json::Value>,
        mappings: Option<&serde_json::Value>,
    ) -> Result<()> {
        let key = format!("{}:{}", INDEX_PREFIX, index_name);
        let metadata = serde_json::json!({
            "name": index_name,
            "settings": settings,
            "mappings": mappings
        });
        let value = serde_json::to_vec(&metadata)?;
        self.db.insert(key.as_bytes(), value)
            .map_err(|e| GummySearchError::Storage(format!("Failed to store index metadata: {}", e)))?;
        self.db.flush()
            .map_err(|e| GummySearchError::Storage(format!("Failed to flush database: {}", e)))?;
        Ok(())
    }

    /// Load index metadata
    pub fn load_index_metadata(&self, index_name: &str) -> Result<Option<(Option<serde_json::Value>, Option<serde_json::Value>)>> {
        let key = format!("{}:{}", INDEX_PREFIX, index_name);
        if let Some(value) = self.db.get(key.as_bytes())
            .map_err(sled_error)? {
            let metadata: serde_json::Value = serde_json::from_slice(&value)?;
            let settings = metadata.get("settings").cloned();
            let mappings = metadata.get("mappings").cloned();
            Ok(Some((settings, mappings)))
        } else {
            Ok(None)
        }
    }

    /// List all index names
    pub fn list_indices(&self) -> Result<Vec<String>> {
        let mut indices = Vec::new();
        for result in self.db.scan_prefix(INDEX_PREFIX.as_bytes()) {
            let (key, _) = result.map_err(sled_error)?;
            if let Ok(key_str) = std::str::from_utf8(&key) {
                // Key format is "index:name", so strip the prefix and colon
                if let Some(index_name) = key_str.strip_prefix(INDEX_PREFIX) {
                    // Remove the colon if present
                    let name = index_name.strip_prefix(':').unwrap_or(index_name);
                    indices.push(name.to_string());
                }
            }
        }
        Ok(indices)
    }

    /// Delete index metadata
    pub fn delete_index_metadata(&self, index_name: &str) -> Result<()> {
        let key = format!("{}:{}", INDEX_PREFIX, index_name);
        self.db.remove(key.as_bytes())
            .map_err(sled_error)?;

        // Also delete all documents for this index
        let doc_prefix = format!("{}:{}:", DOC_PREFIX, index_name);
        let mut to_remove = Vec::new();
        for result in self.db.scan_prefix(doc_prefix.as_bytes()) {
            let (key, _) = result.map_err(sled_error)?;
            to_remove.push(key);
        }
        for key in to_remove {
            self.db.remove(key)
                .map_err(sled_error)?;
        }

        self.db.flush()
            .map_err(sled_error)?;
        Ok(())
    }

    /// Store a document
    pub fn store_document(
        &self,
        index_name: &str,
        doc_id: &str,
        document: &serde_json::Value,
    ) -> Result<()> {
        let key = format!("{}:{}:{}", DOC_PREFIX, index_name, doc_id);
        let value = serde_json::to_vec(document)?;
        self.db.insert(key.as_bytes(), value)
            .map_err(sled_error)?;
        // Don't flush on every document write for performance
        Ok(())
    }

    /// Load a document
    pub fn load_document(&self, index_name: &str, doc_id: &str) -> Result<Option<serde_json::Value>> {
        let key = format!("{}:{}:{}", DOC_PREFIX, index_name, doc_id);
        if let Some(value) = self.db.get(key.as_bytes())
            .map_err(sled_error)? {
            let doc: serde_json::Value = serde_json::from_slice(&value)?;
            Ok(Some(doc))
        } else {
            Ok(None)
        }
    }

    /// Delete a document
    pub fn delete_document(&self, index_name: &str, doc_id: &str) -> Result<()> {
        let key = format!("{}:{}:{}", DOC_PREFIX, index_name, doc_id);
        self.db.remove(key.as_bytes())
            .map_err(sled_error)?;
        Ok(())
    }

    /// Load all documents for an index
    pub fn load_all_documents(&self, index_name: &str) -> Result<Vec<(String, serde_json::Value)>> {
        let prefix = format!("{}:{}:", DOC_PREFIX, index_name);
        let mut documents = Vec::new();

        for result in self.db.scan_prefix(prefix.as_bytes()) {
            let (key, value) = result.map_err(sled_error)?;
            if let Ok(key_str) = std::str::from_utf8(&key) {
                if let Some(suffix) = key_str.strip_prefix(&prefix) {
                    let doc: serde_json::Value = serde_json::from_slice(&value)?;
                    documents.push((suffix.to_string(), doc));
                }
            }
        }

        Ok(documents)
    }

    /// Flush pending writes to disk
    pub fn flush(&self) -> Result<()> {
        self.db.flush()
            .map_err(sled_error)?;
        Ok(())
    }
}

impl std::fmt::Debug for SledBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SledBackend").finish()
    }
}

impl Clone for SledBackend {
    fn clone(&self) -> Self {
        Self {
            db: Arc::clone(&self.db),
        }
    }
}

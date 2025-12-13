//! Document management operations

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use tracing::{info, debug, warn, error};

use crate::error::{GummySearchError, Result};
use crate::storage::Index;
use crate::storage_backend::SledBackend;
use crate::bulk_ops::BulkAction;

/// Index a document (create or update)
pub async fn index_document(
    indices: &Arc<RwLock<HashMap<String, Index>>>,
    backend: &Option<Arc<SledBackend>>,
    index_name: &str,
    id: &str,
    document: serde_json::Value,
) -> Result<()> {
    debug!("Indexing document '{}' in index '{}'", id, index_name);

    // Persist to backend if available
    if let Some(backend) = backend {
        let backend_clone = backend.clone();
        let index_name_str = index_name.to_string();
        let id_str = id.to_string();
        let doc_clone = document.clone();

        tokio::task::spawn_blocking(move || {
            backend_clone.store_document(&index_name_str, &id_str, &doc_clone)
        }).await.map_err(GummySearchError::TaskJoin)??;
        debug!("Document '{}' persisted to storage backend", id);
    }

    let mut indices_guard = indices.write().await;
    let index = indices_guard
        .get_mut(index_name)
        .ok_or_else(|| {
            error!("Index '{}' not found when indexing document '{}'", index_name, id);
            GummySearchError::IndexNotFound(index_name.to_string())
        })?;

    index.documents.insert(id.to_string(), document);
    debug!("Document '{}' indexed successfully in index '{}'", id, index_name);
    Ok(())
}

/// Create a document with auto-generated ID
pub async fn create_document(
    indices: &Arc<RwLock<HashMap<String, Index>>>,
    backend: &Option<Arc<SledBackend>>,
    index_name: &str,
    document: serde_json::Value,
) -> Result<String> {
    let id = Uuid::new_v4().to_string();
    index_document(indices, backend, index_name, &id, document).await?;
    Ok(id)
}

/// Get a document by ID
pub async fn get_document(
    indices: &Arc<RwLock<HashMap<String, Index>>>,
    index_name: &str,
    id: &str,
) -> Result<serde_json::Value> {
    let indices_guard = indices.read().await;
    let index = indices_guard
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

/// Delete a document
pub async fn delete_document(
    indices: &Arc<RwLock<HashMap<String, Index>>>,
    backend: &Option<Arc<SledBackend>>,
    index_name: &str,
    id: &str,
) -> Result<()> {
    debug!("Deleting document '{}' from index '{}'", id, index_name);

    // Delete from backend if available
    if let Some(backend) = backend {
        let backend_clone = backend.clone();
        let index_name_str = index_name.to_string();
        let id_str = id.to_string();

        tokio::task::spawn_blocking(move || {
            backend_clone.delete_document(&index_name_str, &id_str)
        }).await.map_err(GummySearchError::TaskJoin)??;
        debug!("Document '{}' deleted from storage backend", id);
    }

    let mut indices_guard = indices.write().await;
    let index = indices_guard
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

/// Execute a bulk action
pub async fn execute_bulk_action(
    indices: &Arc<RwLock<HashMap<String, Index>>>,
    backend: &Option<Arc<SledBackend>>,
    action: BulkAction,
) -> Result<(String, String, u16, Option<String>)> {
    match action {
        BulkAction::Index { index, id, document } => {
            let doc_id = id.unwrap_or_else(|| Uuid::new_v4().to_string());
            index_document(indices, backend, &index, &doc_id, document).await?;
            Ok((index, doc_id, 201, Some("created".to_string())))
        }
        BulkAction::Create { index, id, document } => {
            let doc_id = id.unwrap_or_else(|| Uuid::new_v4().to_string());
            // Check if document exists
            if index_exists(indices, &index).await? {
                let indices_guard = indices.read().await;
                if let Some(idx) = indices_guard.get(&index) {
                    if idx.documents.contains_key(&doc_id) {
                        return Err(GummySearchError::InvalidRequest(
                            format!("Document {} already exists", doc_id)
                        ));
                    }
                }
            }
            index_document(indices, backend, &index, &doc_id, document).await?;
            Ok((index, doc_id, 201, Some("created".to_string())))
        }
        BulkAction::Update { index, id, document } => {
            // For update, we merge with existing document or create new
            let indices_guard = indices.read().await;
            let existing = indices_guard.get(&index)
                .and_then(|idx| idx.documents.get(&id).cloned());
            drop(indices_guard);

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

            index_document(indices, backend, &index, &id, updated_doc).await?;
            Ok((index, id, 200, Some("updated".to_string())))
        }
        BulkAction::Delete { index, id } => {
            delete_document(indices, backend, &index, &id).await?;
            Ok((index, id, 200, Some("deleted".to_string())))
        }
    }
}

// Helper function for index_exists (needed by execute_bulk_action)
async fn index_exists(
    indices: &Arc<RwLock<HashMap<String, Index>>>,
    name: &str,
) -> Result<bool> {
    let indices_guard = indices.read().await;
    Ok(indices_guard.contains_key(name))
}

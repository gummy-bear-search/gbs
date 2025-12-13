//! Persistence operations for Storage

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, debug};

use crate::error::{GummySearchError, Result};
use crate::storage::Index;
use crate::storage_backend::SledBackend;

/// Flush pending writes to disk (for persistent storage)
pub async fn flush(backend: &Option<Arc<SledBackend>>) -> Result<()> {
    if let Some(backend) = backend {
        let backend_clone = backend.clone();
        tokio::task::spawn_blocking(move || {
            backend_clone.flush()
        }).await.map_err(GummySearchError::TaskJoin)??;
    }
    Ok(())
}

/// Refresh an index (flush changes to persistent storage)
pub async fn refresh_index(
    _indices: &Arc<RwLock<HashMap<String, Index>>>,
    backend: &Option<Arc<SledBackend>>,
    index_name: &str,
) -> Result<()> {
    debug!("Refreshing index: {}", index_name);
    // For persistent storage, flush to disk
    flush(backend).await?;
    info!("Index '{}' refreshed successfully", index_name);
    Ok(())
}

/// Load indices from backend (call this after creating with sled)
pub async fn load_from_backend(
    indices: &Arc<RwLock<HashMap<String, Index>>>,
    backend: &Option<Arc<SledBackend>>,
) -> Result<()> {
    if let Some(backend) = backend {
        info!("Loading indices from persistent storage");
        let start = std::time::Instant::now();
        let indices_data = tokio::task::spawn_blocking({
            let backend = backend.clone();
            move || {
                let indices_list = backend.list_indices()?;
                debug!("Found {} indices in persistent storage", indices_list.len());
                let mut loaded = HashMap::new();

                for index_name in indices_list {
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

        let mut indices_guard = indices.write().await;
        let count = indices_data.len();
        *indices_guard = indices_data;
        let elapsed = start.elapsed();
        info!("Loaded {} indices from persistent storage in {:?}", count, elapsed);
    } else {
        debug!("No persistent backend configured, skipping load");
    }
    Ok(())
}

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::error::{GummySearchError, Result};
use crate::bulk_ops::BulkAction;

#[derive(Clone, Debug)]
pub struct Index {
    pub name: String,
    pub settings: Option<serde_json::Value>,
    pub mappings: Option<serde_json::Value>,
    pub documents: HashMap<String, serde_json::Value>,
}

impl Index {
    pub fn new(name: String, settings: Option<serde_json::Value>, mappings: Option<serde_json::Value>) -> Self {
        Self {
            name,
            settings,
            mappings,
            documents: HashMap::new(),
        }
    }
}

#[derive(Clone)]
pub struct Storage {
    indices: Arc<RwLock<HashMap<String, Index>>>,
}

impl Storage {
    pub fn new() -> Self {
        Self {
            indices: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn create_index(
        &self,
        name: &str,
        settings: Option<serde_json::Value>,
        mappings: Option<serde_json::Value>,
    ) -> Result<()> {
        let mut indices = self.indices.write().await;

        if indices.contains_key(name) {
            return Err(GummySearchError::InvalidRequest(
                format!("Index {} already exists", name),
            ));
        }

        let index = Index::new(name.to_string(), settings, mappings);
        indices.insert(name.to_string(), index);

        Ok(())
    }

    pub async fn index_exists(&self, name: &str) -> Result<bool> {
        let indices = self.indices.read().await;
        Ok(indices.contains_key(name))
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
        let mut indices = self.indices.write().await;
        indices
            .remove(name)
            .ok_or_else(|| GummySearchError::IndexNotFound(name.to_string()))?;
        Ok(())
    }

    pub async fn index_document(
        &self,
        index_name: &str,
        id: &str,
        document: serde_json::Value,
    ) -> Result<()> {
        let mut indices = self.indices.write().await;
        let index = indices
            .get_mut(index_name)
            .ok_or_else(|| GummySearchError::IndexNotFound(index_name.to_string()))?;

        index.documents.insert(id.to_string(), document);
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
        let mut indices = self.indices.write().await;
        let index = indices
            .get_mut(index_name)
            .ok_or_else(|| GummySearchError::IndexNotFound(index_name.to_string()))?;

        index
            .documents
            .remove(id)
            .ok_or_else(|| GummySearchError::DocumentNotFound(id.to_string()))?;

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
}

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::error::{GummySearchError, Result};

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
}

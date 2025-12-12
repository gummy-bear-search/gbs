use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::error::{GummySearchError, Result};
use crate::bulk_ops::BulkAction;
use crate::storage_backend::SledBackend;

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
        let backend = Arc::new(SledBackend::new(path)?);
        Ok(Self {
            indices: Arc::new(RwLock::new(HashMap::new())),
            backend: Some(backend),
        })
    }

    /// Flush pending writes to disk (for persistent storage)
    pub fn flush(&self) -> Result<()> {
        if let Some(backend) = &self.backend {
            backend.flush()?;
        }
        Ok(())
    }

    /// Load indices from backend (call this after creating with sled)
    pub async fn load_from_backend(&self) -> Result<()> {
        if let Some(backend) = &self.backend {
            let indices_data = tokio::task::spawn_blocking({
                let backend = backend.clone();
                move || {
                    let indices = backend.list_indices()?;
                    let mut loaded = HashMap::new();

                    for index_name in indices {
                        if let Some((settings, mappings)) = backend.load_index_metadata(&index_name)? {
                            let mut index = Index::new(index_name.clone(), settings, mappings);

                            let documents = backend.load_all_documents(&index_name)?;
                            for (doc_id, doc) in documents {
                                index.documents.insert(doc_id, doc);
                            }

                            loaded.insert(index_name, index);
                        }
                    }

                    Ok::<_, GummySearchError>(loaded)
                }
            }).await.map_err(GummySearchError::TaskJoin)??;

            let mut indices = self.indices.write().await;
            *indices = indices_data;
        }
        Ok(())
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

        // Persist to backend if available
        if let Some(backend) = &self.backend {
            let backend_clone = backend.clone();
            let name_str = name.to_string();
            let settings_clone = settings.clone();
            let mappings_clone = mappings.clone();

            tokio::task::spawn_blocking(move || {
                backend_clone.store_index_metadata(&name_str, settings_clone.as_ref(), mappings_clone.as_ref())
            }).await.map_err(GummySearchError::TaskJoin)??;
        }

        let index = Index::new(name.to_string(), settings, mappings);
        indices.insert(name.to_string(), index);

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

    pub async fn update_mapping(
        &self,
        index_name: &str,
        new_mappings: serde_json::Value,
    ) -> Result<()> {
        let mut indices = self.indices.write().await;
        let index = indices
            .get_mut(index_name)
            .ok_or_else(|| GummySearchError::IndexNotFound(index_name.to_string()))?;

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
            let backend_clone = backend.clone();
            let index_name_str = index_name.to_string();
            let final_mappings = index.mappings.clone();
            let settings = index.settings.clone();

            tokio::task::spawn_blocking(move || {
                backend_clone.store_index_metadata(&index_name_str, settings.as_ref(), final_mappings.as_ref())
            }).await.map_err(GummySearchError::TaskJoin)??;
        }

        Ok(())
    }

    pub async fn update_settings(
        &self,
        index_name: &str,
        new_settings: serde_json::Value,
    ) -> Result<()> {
        let mut indices = self.indices.write().await;
        let index = indices
            .get_mut(index_name)
            .ok_or_else(|| GummySearchError::IndexNotFound(index_name.to_string()))?;

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
            let backend_clone = backend.clone();
            let index_name_str = index_name.to_string();
            let final_settings = index.settings.clone();
            let mappings = index.mappings.clone();

            tokio::task::spawn_blocking(move || {
                backend_clone.store_index_metadata(&index_name_str, final_settings.as_ref(), mappings.as_ref())
            }).await.map_err(GummySearchError::TaskJoin)??;
        }

        Ok(())
    }

    pub async fn delete_all_indices(&self) -> Result<()> {
        // Delete all from backend if available
        if let Some(backend) = &self.backend {
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
        }

        let mut indices = self.indices.write().await;
        indices.clear();
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
        // Delete from backend if available
        if let Some(backend) = &self.backend {
            let backend_clone = backend.clone();
            let name_str = name.to_string();

            tokio::task::spawn_blocking(move || {
                backend_clone.delete_index_metadata(&name_str)
            }).await.map_err(GummySearchError::TaskJoin)??;
        }

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
        // Persist to backend if available
        if let Some(backend) = &self.backend {
            let backend_clone = backend.clone();
            let index_name_str = index_name.to_string();
            let id_str = id.to_string();
            let doc_clone = document.clone();

            tokio::task::spawn_blocking(move || {
                backend_clone.store_document(&index_name_str, &id_str, &doc_clone)
            }).await.map_err(GummySearchError::TaskJoin)??;
        }

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
        // Delete from backend if available
        if let Some(backend) = &self.backend {
            let backend_clone = backend.clone();
            let index_name_str = index_name.to_string();
            let id_str = id.to_string();

            tokio::task::spawn_blocking(move || {
                backend_clone.delete_document(&index_name_str, &id_str)
            }).await.map_err(GummySearchError::TaskJoin)??;
        }

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

    /// Search documents in an index
    ///
    /// Supports:
    /// - match query (text search in specified field)
    /// - match_all query (return all documents)
    /// - term query (exact match)
    /// - bool query (must, should, must_not, filter)
    /// - Pagination (from, size)
    /// - Sorting
    pub async fn search(
        &self,
        index_name: &str,
        query: &serde_json::Value,
        from: Option<u32>,
        size: Option<u32>,
        sort: Option<&serde_json::Value>,
    ) -> Result<serde_json::Value> {
        let indices = self.indices.read().await;
        let index = indices
            .get(index_name)
            .ok_or_else(|| GummySearchError::IndexNotFound(index_name.to_string()))?;

        let start_time = std::time::Instant::now();

        // Collect all documents with their IDs
        let mut scored_docs: Vec<(String, serde_json::Value, f64)> = Vec::new();

        for (id, doc) in &index.documents {
            let score = self.score_document(doc, query)?;
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
                        self.compare_documents(&a.1, &b.1, sort_item)
                    });
                }
            } else if sort_spec.is_object() {
                // Single sort field
                scored_docs.sort_by(|a, b| {
                    self.compare_documents(&a.1, &b.1, sort_spec)
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

        // Build hits
        let hits: Vec<serde_json::Value> = paginated_docs
            .into_iter()
            .map(|(id, doc, score)| {
                serde_json::json!({
                    "_index": index_name,
                    "_type": "_doc",
                    "_id": id,
                    "_score": score,
                    "_source": doc
                })
            })
            .collect();

        let took = start_time.elapsed().as_millis() as u32;

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

    /// Score a document against a query
    fn score_document(&self, doc: &serde_json::Value, query: &serde_json::Value) -> Result<f64> {
        if let Some(query_obj) = query.as_object() {
            // Handle match_all query (no query or empty query)
            if query_obj.is_empty() {
                return Ok(1.0);
            }

            // Handle match query: { "match": { "field": "query text" } }
            if let Some(match_query) = query_obj.get("match") {
                if let Some(match_obj) = match_query.as_object() {
                    for (field, query_value) in match_obj {
                        let query_text = if let Some(q) = query_value.as_object() {
                            q.get("query").and_then(|v| v.as_str()).unwrap_or("")
                        } else {
                            query_value.as_str().unwrap_or("")
                        };

                        if let Some(score) = self.match_field(doc, field, query_text) {
                            return Ok(score);
                        }
                    }
                }
            }

            // Handle term query: { "term": { "field": "value" } }
            if let Some(term_query) = query_obj.get("term") {
                if let Some(term_obj) = term_query.as_object() {
                    for (field, value) in term_obj {
                        if self.term_match(doc, field, value) {
                            return Ok(1.0);
                        }
                    }
                }
            }

            // Handle bool query
            if let Some(bool_query) = query_obj.get("bool") {
                return self.score_bool_query(doc, bool_query);
            }

            // Handle match_all query: { "match_all": {} }
            if query_obj.contains_key("match_all") {
                return Ok(1.0);
            }
        }

        // Default: no match
        Ok(0.0)
    }

    /// Match a field against query text (case-insensitive substring match)
    fn match_field(&self, doc: &serde_json::Value, field: &str, query_text: &str) -> Option<f64> {
        if query_text.is_empty() {
            return Some(1.0);
        }

        // Handle _all field - search in all fields
        if field == "_all" || field == "*" {
            return self.match_all_fields(doc, query_text);
        }

        let field_value = self.get_field_value(doc, field)?;
        let field_str = match field_value {
            serde_json::Value::String(s) => s.to_lowercase(),
            serde_json::Value::Number(n) => n.to_string(),
            serde_json::Value::Bool(b) => b.to_string(),
            _ => return None,
        };

        let query_lower = query_text.to_lowercase();

        // Simple scoring: 1.0 if exact match, 0.8 if contains, 0.5 if word match
        if field_str == query_lower {
            Some(1.0)
        } else if field_str.contains(&query_lower) {
            Some(0.8)
        } else {
            // Check for word matches
            let words: Vec<&str> = query_lower.split_whitespace().collect();
            let field_words: Vec<&str> = field_str.split_whitespace().collect();
            let matches = words.iter()
                .filter(|w| field_words.iter().any(|fw| fw.contains(*w)))
                .count();
            if matches > 0 {
                Some(0.5 * (matches as f64 / words.len() as f64))
            } else {
                None
            }
        }
    }

    /// Match query text against all fields in a document
    fn match_all_fields(&self, doc: &serde_json::Value, query_text: &str) -> Option<f64> {
        if query_text.is_empty() {
            return Some(1.0);
        }

        let query_lower = query_text.to_lowercase();
        let mut max_score = 0.0;

        // Recursively search all string/number values in the document
        self.search_value(doc, &query_lower, &mut max_score);

        if max_score > 0.0 {
            Some(max_score)
        } else {
            None
        }
    }

    /// Recursively search a JSON value for the query text
    fn search_value(&self, value: &serde_json::Value, query: &str, max_score: &mut f64) {
        match value {
            serde_json::Value::String(s) => {
                let s_lower = s.to_lowercase();
                let score = if s_lower == *query {
                    1.0
                } else if s_lower.contains(query) {
                    0.8
                } else {
                    // Check for word matches
                    let words: Vec<&str> = query.split_whitespace().collect();
                    let field_words: Vec<&str> = s_lower.split_whitespace().collect();
                    let matches = words.iter()
                        .filter(|w| field_words.iter().any(|fw| fw.contains(*w)))
                        .count();
                    if matches > 0 {
                        0.5 * (matches as f64 / words.len() as f64)
                    } else {
                        0.0
                    }
                };
                *max_score = max_score.max(score);
            }
            serde_json::Value::Number(n) => {
                let n_str = n.to_string();
                if n_str.contains(query) {
                    *max_score = max_score.max(0.5);
                }
            }
            serde_json::Value::Object(map) => {
                for v in map.values() {
                    self.search_value(v, query, max_score);
                }
            }
            serde_json::Value::Array(arr) => {
                for v in arr {
                    self.search_value(v, query, max_score);
                }
            }
            _ => {}
        }
    }

    /// Check if a field matches a term exactly
    fn term_match(&self, doc: &serde_json::Value, field: &str, value: &serde_json::Value) -> bool {
        if let Some(field_value) = self.get_field_value(doc, field) {
            *field_value == *value
        } else {
            false
        }
    }

    /// Score a bool query
    fn score_bool_query(&self, doc: &serde_json::Value, bool_query: &serde_json::Value) -> Result<f64> {
        if let Some(bool_obj) = bool_query.as_object() {
            let mut score = 0.0;
            let mut must_match = true;

            // Handle must clauses (all must match)
            if let Some(must) = bool_obj.get("must") {
                if let Some(must_array) = must.as_array() {
                    for clause in must_array {
                        let clause_score = self.score_document(doc, clause)?;
                        if clause_score == 0.0 {
                            must_match = false;
                            break;
                        }
                        score += clause_score;
                    }
                }
            }

            if !must_match {
                return Ok(0.0);
            }

            // Handle should clauses (at least one should match, or boost score)
            if let Some(should) = bool_obj.get("should") {
                if let Some(should_array) = should.as_array() {
                    let mut should_score = 0.0;
                    for clause in should_array {
                        should_score += self.score_document(doc, clause)?;
                    }
                    if should_score > 0.0 {
                        score += should_score * 0.5; // Boost for should matches
                    }
                }
            }

            // Handle must_not clauses (none should match)
            if let Some(must_not) = bool_obj.get("must_not") {
                if let Some(must_not_array) = must_not.as_array() {
                    for clause in must_not_array {
                        let clause_score = self.score_document(doc, clause)?;
                        if clause_score > 0.0 {
                            return Ok(0.0); // Document matches must_not, exclude it
                        }
                    }
                }
            }

            // Handle filter clauses (must match, but don't affect score)
            if let Some(filter) = bool_obj.get("filter") {
                if let Some(filter_array) = filter.as_array() {
                    for clause in filter_array {
                        let clause_score = self.score_document(doc, clause)?;
                        if clause_score == 0.0 {
                            return Ok(0.0); // Filter doesn't match, exclude
                        }
                    }
                }
            }

            Ok(if score > 0.0 { score } else { 1.0 }) // At least 1.0 if all filters pass
        } else {
            Ok(0.0)
        }
    }

    /// Get a field value from a document (supports nested fields with dot notation)
    fn get_field_value<'a>(&self, doc: &'a serde_json::Value, field: &str) -> Option<&'a serde_json::Value> {
        if field == "_all" || field == "*" {
            return Some(doc);
        }

        let parts: Vec<&str> = field.split('.').collect();
        let mut current = doc;

        for part in parts {
            if let Some(obj) = current.as_object() {
                current = obj.get(part)?;
            } else {
                return None;
            }
        }

        Some(current)
    }

    /// Compare two documents for sorting
    fn compare_documents(&self, a: &serde_json::Value, b: &serde_json::Value, sort_spec: &serde_json::Value) -> std::cmp::Ordering {
        if let Some(sort_obj) = sort_spec.as_object() {
            for (field, order_spec) in sort_obj {
                let order = if let Some(order_obj) = order_spec.as_object() {
                    order_obj.get("order")
                        .and_then(|o| o.as_str())
                        .unwrap_or("asc")
                } else {
                    order_spec.as_str().unwrap_or("asc")
                };

                let a_val = self.get_field_value(a, field);
                let b_val = self.get_field_value(b, field);

                let cmp = match (a_val, b_val) {
                    (Some(serde_json::Value::String(a_str)), Some(serde_json::Value::String(b_str))) => {
                        a_str.cmp(b_str)
                    }
                    (Some(serde_json::Value::Number(a_num)), Some(serde_json::Value::Number(b_num))) => {
                        if let (Some(a_f64), Some(b_f64)) = (a_num.as_f64(), b_num.as_f64()) {
                            a_f64.partial_cmp(&b_f64).unwrap_or(std::cmp::Ordering::Equal)
                        } else {
                            std::cmp::Ordering::Equal
                        }
                    }
                    (Some(_), None) => std::cmp::Ordering::Greater,
                    (None, Some(_)) => std::cmp::Ordering::Less,
                    _ => std::cmp::Ordering::Equal,
                };

                if cmp != std::cmp::Ordering::Equal {
                    return if order == "desc" {
                        cmp.reverse()
                    } else {
                        cmp
                    };
                }
            }
        }

        std::cmp::Ordering::Equal
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

        let result = storage.search("test_index", &query, None, None, None).await.unwrap();
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

        let result = storage.search("test_index", &query, None, None, None).await.unwrap();
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
        let result = storage.search("test_index", &query, Some(0), Some(5), None).await.unwrap();
        let hits = result.get("hits").and_then(|h| h.get("hits")).and_then(|h| h.as_array()).unwrap();
        assert_eq!(hits.len(), 5);

        // Second page
        let result = storage.search("test_index", &query, Some(5), Some(5), None).await.unwrap();
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

        let result = storage.search("test_index", &query, None, None, None).await.unwrap();
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

        let result = storage.search("test_index", &query, None, None, None).await.unwrap();
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

        let result = storage.search("test_index", &query, None, None, Some(&sort)).await.unwrap();
        let hits = result.get("hits").and_then(|h| h.get("hits")).and_then(|h| h.as_array()).unwrap();

        assert_eq!(hits.len(), 3);
        assert_eq!(hits[0].get("_id").and_then(|id| id.as_str()).unwrap(), "2"); // Bob, age 25
        assert_eq!(hits[1].get("_id").and_then(|id| id.as_str()).unwrap(), "1"); // Alice, age 30
        assert_eq!(hits[2].get("_id").and_then(|id| id.as_str()).unwrap(), "3"); // Charlie, age 35
    }
}

//! Index management operations

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use regex::Regex;
use tracing::{info, debug, warn, error};

use crate::error::{GummySearchError, Result};
use crate::storage::Index;
use crate::storage_backend::SledBackend;

/// Create a new index
pub async fn create_index(
    indices: &Arc<RwLock<HashMap<String, Index>>>,
    backend: &Option<Arc<SledBackend>>,
    name: &str,
    settings: Option<serde_json::Value>,
    mappings: Option<serde_json::Value>,
) -> Result<()> {
    info!("Creating index: {}", name);
    let mut indices_guard = indices.write().await;

    if indices_guard.contains_key(name) {
        warn!("Attempted to create index '{}' that already exists", name);
        return Err(GummySearchError::InvalidRequest(
            format!("Index {} already exists", name),
        ));
    }

    // Persist to backend if available
    if let Some(backend) = backend {
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
    indices_guard.insert(name.to_string(), index);
    info!("Index '{}' created successfully", name);

    Ok(())
}

/// Check if an index exists
pub async fn index_exists(
    indices: &Arc<RwLock<HashMap<String, Index>>>,
    name: &str,
) -> Result<bool> {
    let indices_guard = indices.read().await;
    Ok(indices_guard.contains_key(name))
}

/// List all indices
pub async fn list_indices(
    indices: &Arc<RwLock<HashMap<String, Index>>>,
) -> Vec<String> {
    let indices_guard = indices.read().await;
    indices_guard.keys().cloned().collect()
}

/// Match index names against a pattern (supports * and ? wildcards)
pub async fn match_indices(
    indices: &Arc<RwLock<HashMap<String, Index>>>,
    pattern: &str,
) -> Vec<String> {
    let all_indices = list_indices(indices).await;

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
pub async fn get_indices_stats(
    indices: &Arc<RwLock<HashMap<String, Index>>>,
) -> Vec<(String, usize)> {
    let indices_guard = indices.read().await;
    indices_guard.iter()
        .map(|(name, index)| (name.clone(), index.documents.len()))
        .collect()
}

/// Get aliases for all indices
pub async fn get_aliases(
    indices: &Arc<RwLock<HashMap<String, Index>>>,
) -> serde_json::Value {
    let indices_guard = indices.read().await;
    let mut result = serde_json::Map::new();

    for (index_name, index) in indices_guard.iter() {
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

/// Get an index
pub async fn get_index(
    indices: &Arc<RwLock<HashMap<String, Index>>>,
    name: &str,
) -> Result<serde_json::Value> {
    let indices_guard = indices.read().await;
    let index = indices_guard
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

/// Delete an index
pub async fn delete_index(
    indices: &Arc<RwLock<HashMap<String, Index>>>,
    backend: &Option<Arc<SledBackend>>,
    name: &str,
) -> Result<()> {
    info!("Deleting index: {}", name);

    // Delete from backend if available
    if let Some(backend) = backend {
        debug!("Deleting index '{}' from storage backend", name);
        let backend_clone = backend.clone();
        let name_str = name.to_string();

        tokio::task::spawn_blocking(move || {
            backend_clone.delete_index_metadata(&name_str)
        }).await.map_err(GummySearchError::TaskJoin)??;
        debug!("Index '{}' deleted from storage backend", name);
    }

    let mut indices_guard = indices.write().await;
    let doc_count = indices_guard.get(name).map(|idx| idx.documents.len()).unwrap_or(0);
    indices_guard
        .remove(name)
        .ok_or_else(|| {
            warn!("Attempted to delete non-existent index: {}", name);
            GummySearchError::IndexNotFound(name.to_string())
        })?;

    info!("Index '{}' deleted successfully (had {} documents)", name, doc_count);
    Ok(())
}

/// Delete all indices
pub async fn delete_all_indices(
    indices: &Arc<RwLock<HashMap<String, Index>>>,
    backend: &Option<Arc<SledBackend>>,
) -> Result<()> {
    warn!("Deleting all indices - this is a destructive operation!");

    let indices_guard = indices.read().await;
    let count = indices_guard.len();
    let index_names: Vec<String> = indices_guard.keys().cloned().collect();
    drop(indices_guard);

    // Delete all from backend if available
    if let Some(backend) = backend {
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

    let mut indices_guard = indices.write().await;
    indices_guard.clear();
    info!("Deleted all {} indices: {:?}", count, index_names);
    Ok(())
}

/// Update index mapping
pub async fn update_mapping(
    indices: &Arc<RwLock<HashMap<String, Index>>>,
    backend: &Option<Arc<SledBackend>>,
    index_name: &str,
    new_mappings: serde_json::Value,
) -> Result<()> {
    info!("Updating mapping for index: {}", index_name);
    debug!("New mapping: {}", serde_json::to_string(&new_mappings).unwrap_or_default());

    let mut indices_guard = indices.write().await;
    let index = indices_guard
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
    if let Some(backend) = backend {
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

/// Update index settings
pub async fn update_settings(
    indices: &Arc<RwLock<HashMap<String, Index>>>,
    backend: &Option<Arc<SledBackend>>,
    index_name: &str,
    new_settings: serde_json::Value,
) -> Result<()> {
    info!("Updating settings for index: {}", index_name);
    debug!("New settings: {}", serde_json::to_string(&new_settings).unwrap_or_default());

    let mut indices_guard = indices.write().await;
    let index = indices_guard
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
    if let Some(backend) = backend {
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

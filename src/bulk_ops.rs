use serde::Serialize;
use serde_json::Value;

use crate::error::{GbsError, Result};

#[derive(Debug, Clone)]
pub enum BulkAction {
    Index {
        index: String,
        id: Option<String>,
        document: Value,
    },
    Create {
        index: String,
        id: Option<String>,
        document: Value,
    },
    Update {
        index: String,
        id: String,
        document: Value,
    },
    Delete {
        index: String,
        id: String,
    },
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum BulkItemResponse {
    Index { index: BulkOperationResult },
    Create { create: BulkOperationResult },
    Update { update: BulkOperationResult },
    Delete { delete: BulkOperationResult },
}

#[derive(Debug, Serialize)]
pub struct BulkOperationResult {
    #[serde(rename = "_index")]
    pub index: String,
    #[serde(rename = "_type")]
    pub r#type: String,
    #[serde(rename = "_id")]
    pub id: String,
    #[serde(rename = "_version", skip_serializing_if = "Option::is_none")]
    pub version: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<String>,
    #[serde(rename = "_shards", skip_serializing_if = "Option::is_none")]
    pub shards: Option<ShardsInfo>,
    pub status: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<BulkError>,
}

#[derive(Debug, Serialize)]
pub struct ShardsInfo {
    pub total: u32,
    pub successful: u32,
    pub failed: u32,
}

#[derive(Debug, Serialize)]
pub struct BulkError {
    #[serde(rename = "type")]
    pub r#type: String,
    pub reason: String,
}

#[derive(Debug, Serialize)]
pub struct BulkResponse {
    pub took: u32,
    pub errors: bool,
    pub items: Vec<BulkItemResponse>,
}

pub fn parse_bulk_ndjson(body: &str, default_index: Option<&str>) -> Result<Vec<BulkAction>> {
    let lines: Vec<&str> = body.lines().filter(|l| !l.trim().is_empty()).collect();
    let mut actions = Vec::new();
    let mut i = 0;

    while i < lines.len() {
        let action_line = lines[i];
        let action_json: Value = serde_json::from_str(action_line)
            .map_err(|e| GbsError::InvalidRequest(format!("Invalid JSON in bulk action: {}", e)))?;

        // Determine action type and extract parameters
        let (action_type, index, id, document): (_, String, Option<String>, Value) =
            if let Some(index_obj) = action_json.get("index") {
                let index_name = index_obj
                    .get("_index")
                    .and_then(|v| v.as_str())
                    .or(default_index)
                    .ok_or_else(|| {
                        GbsError::InvalidRequest("Missing _index in bulk action".to_string())
                    })?
                    .to_string();
                let id = index_obj
                    .get("_id")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                i += 1;
                if i >= lines.len() {
                    return Err(GbsError::InvalidRequest(
                        "Missing document for index action".to_string(),
                    ));
                }
                let doc: Value = serde_json::from_str(lines[i]).map_err(|e| {
                    GbsError::InvalidRequest(format!("Invalid document JSON: {}", e))
                })?;

                ("index", index_name, id, doc)
            } else if let Some(create_obj) = action_json.get("create") {
                let index_name = create_obj
                    .get("_index")
                    .and_then(|v| v.as_str())
                    .or(default_index)
                    .ok_or_else(|| {
                        GbsError::InvalidRequest("Missing _index in bulk action".to_string())
                    })?
                    .to_string();
                let id = create_obj
                    .get("_id")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                i += 1;
                if i >= lines.len() {
                    return Err(GbsError::InvalidRequest(
                        "Missing document for create action".to_string(),
                    ));
                }
                let doc: Value = serde_json::from_str(lines[i]).map_err(|e| {
                    GbsError::InvalidRequest(format!("Invalid document JSON: {}", e))
                })?;

                ("create", index_name, id, doc)
            } else if let Some(update_obj) = action_json.get("update") {
                let index_name = update_obj
                    .get("_index")
                    .and_then(|v| v.as_str())
                    .or(default_index)
                    .ok_or_else(|| {
                        GbsError::InvalidRequest("Missing _index in bulk action".to_string())
                    })?
                    .to_string();
                let id = update_obj
                    .get("_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        GbsError::InvalidRequest("Missing _id in update action".to_string())
                    })?
                    .to_string();

                i += 1;
                if i >= lines.len() {
                    return Err(GbsError::InvalidRequest(
                        "Missing document for update action".to_string(),
                    ));
                }
                let doc_wrapper: Value = serde_json::from_str(lines[i]).map_err(|e| {
                    GbsError::InvalidRequest(format!("Invalid document JSON: {}", e))
                })?;

                // Extract "doc" field from update wrapper, or use the whole thing
                let doc = doc_wrapper.get("doc").cloned().unwrap_or(doc_wrapper);

                ("update", index_name, Some(id), doc)
            } else if let Some(delete_obj) = action_json.get("delete") {
                let index_name = delete_obj
                    .get("_index")
                    .and_then(|v| v.as_str())
                    .or(default_index)
                    .ok_or_else(|| {
                        GbsError::InvalidRequest("Missing _index in bulk action".to_string())
                    })?
                    .to_string();
                let id = delete_obj
                    .get("_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        GbsError::InvalidRequest("Missing _id in delete action".to_string())
                    })?
                    .to_string();

                ("delete", index_name, Some(id), Value::Null)
            } else {
                return Err(GbsError::InvalidRequest(format!(
                    "Unknown bulk action: {}",
                    action_line
                )));
            };

        // Create the action
        let action = match action_type {
            "index" => BulkAction::Index {
                index,
                id,
                document,
            },
            "create" => BulkAction::Create {
                index,
                id,
                document,
            },
            "update" => BulkAction::Update {
                index,
                id: id.ok_or_else(|| {
                    GbsError::InvalidRequest("Missing _id in update action".to_string())
                })?,
                document,
            },
            "delete" => BulkAction::Delete {
                index,
                id: id.ok_or_else(|| {
                    GbsError::InvalidRequest("Missing _id in delete action".to_string())
                })?,
            },
            _ => unreachable!(),
        };

        actions.push(action);
        i += 1;
    }

    Ok(actions)
}

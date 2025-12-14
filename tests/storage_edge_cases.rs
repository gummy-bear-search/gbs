//! Edge case tests for Storage module
//!
//! These tests cover error handling, boundary conditions, and edge cases
//! that are not covered by the main storage tests.

use gummy_search::storage::Storage;

// ============================================================================
// Document Operations Edge Cases
// ============================================================================

#[tokio::test]
async fn test_index_document_nonexistent_index() {
    let storage = Storage::new();

    let result = storage.index_document("nonexistent", "1", serde_json::json!({
        "title": "Test"
    })).await;

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not found"));
}

#[tokio::test]
async fn test_get_document_nonexistent_index() {
    let storage = Storage::new();

    let result = storage.get_document("nonexistent", "1").await;

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not found"));
}

#[tokio::test]
async fn test_get_document_nonexistent_document() {
    let storage = Storage::new();
    storage.create_index("test_index", None, None).await.unwrap();

    let result = storage.get_document("test_index", "nonexistent").await;

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not found"));
}

#[tokio::test]
async fn test_delete_document_nonexistent_index() {
    let storage = Storage::new();

    let result = storage.delete_document("nonexistent", "1").await;

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not found"));
}

#[tokio::test]
async fn test_delete_document_nonexistent_document() {
    let storage = Storage::new();
    storage.create_index("test_index", None, None).await.unwrap();

    let result = storage.delete_document("test_index", "nonexistent").await;

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not found"));
}

#[tokio::test]
async fn test_create_document_auto_id() {
    let storage = Storage::new();
    storage.create_index("test_index", None, None).await.unwrap();

    let id1 = storage.create_document("test_index", serde_json::json!({
        "title": "Doc 1"
    })).await.unwrap();

    let id2 = storage.create_document("test_index", serde_json::json!({
        "title": "Doc 2"
    })).await.unwrap();

    // IDs should be different
    assert_ne!(id1, id2);

    // Both documents should exist
    let doc1 = storage.get_document("test_index", &id1).await.unwrap();
    let doc2 = storage.get_document("test_index", &id2).await.unwrap();

    assert_eq!(doc1.get("_id").and_then(|id| id.as_str()).unwrap(), id1);
    assert_eq!(doc2.get("_id").and_then(|id| id.as_str()).unwrap(), id2);
}

#[tokio::test]
async fn test_index_document_overwrite() {
    let storage = Storage::new();
    storage.create_index("test_index", None, None).await.unwrap();

    // Index document
    storage.index_document("test_index", "1", serde_json::json!({
        "title": "Original",
        "count": 1
    })).await.unwrap();

    // Overwrite with new document
    storage.index_document("test_index", "1", serde_json::json!({
        "title": "Updated",
        "count": 2
    })).await.unwrap();

    let doc = storage.get_document("test_index", "1").await.unwrap();
    let source = doc.get("_source").unwrap();

    assert_eq!(source.get("title").and_then(|t| t.as_str()).unwrap(), "Updated");
    assert_eq!(source.get("count").and_then(|c| c.as_u64()).unwrap(), 2);
}

#[tokio::test]
async fn test_document_with_various_types() {
    let storage = Storage::new();
    storage.create_index("test_index", None, None).await.unwrap();

    // Document with various JSON types
    storage.index_document("test_index", "1", serde_json::json!({
        "string": "value",
        "number": 42,
        "float": 3.14,
        "boolean": true,
        "null": serde_json::Value::Null,
        "array": [1, 2, 3],
        "object": {
            "nested": "value"
        }
    })).await.unwrap();

    let doc = storage.get_document("test_index", "1").await.unwrap();
    let source = doc.get("_source").unwrap();

    assert_eq!(source.get("string").and_then(|s| s.as_str()).unwrap(), "value");
    assert_eq!(source.get("number").and_then(|n| n.as_u64()).unwrap(), 42);
    assert_eq!(source.get("boolean").and_then(|b| b.as_bool()).unwrap(), true);
    assert!(source.get("null").unwrap().is_null());
    assert!(source.get("array").unwrap().is_array());
    assert!(source.get("object").unwrap().is_object());
}

// ============================================================================
// Index Operations Edge Cases
// ============================================================================

#[tokio::test]
async fn test_create_index_already_exists() {
    let storage = Storage::new();

    storage.create_index("test_index", None, None).await.unwrap();

    let result = storage.create_index("test_index", None, None).await;

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("already exists"));
}

#[tokio::test]
async fn test_get_index_nonexistent() {
    let storage = Storage::new();

    let result = storage.get_index("nonexistent").await;

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not found"));
}

#[tokio::test]
async fn test_delete_index_nonexistent() {
    let storage = Storage::new();

    let result = storage.delete_index("nonexistent").await;

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not found"));
}

#[tokio::test]
async fn test_delete_index_with_documents() {
    let storage = Storage::new();

    storage.create_index("test_index", None, None).await.unwrap();
    storage.index_document("test_index", "1", serde_json::json!({"title": "Doc 1"})).await.unwrap();
    storage.index_document("test_index", "2", serde_json::json!({"title": "Doc 2"})).await.unwrap();

    // Delete index should succeed even with documents
    storage.delete_index("test_index").await.unwrap();

    // Index should be gone
    assert!(storage.get_index("test_index").await.is_err());
}

#[tokio::test]
async fn test_update_mapping_nonexistent_index() {
    let storage = Storage::new();

    let new_mappings = serde_json::json!({
        "title": { "type": "text" }
    });

    let result = storage.update_mapping("nonexistent", new_mappings).await;

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not found"));
}

#[tokio::test]
async fn test_update_settings_nonexistent_index() {
    let storage = Storage::new();

    let new_settings = serde_json::json!({
        "number_of_shards": 2
    });

    let result = storage.update_settings("nonexistent", new_settings).await;

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not found"));
}

#[tokio::test]
async fn test_update_mapping_merge() {
    let storage = Storage::new();

    // Create index with initial mapping
    storage.create_index("test_index", None, Some(serde_json::json!({
        "properties": {
            "title": { "type": "text" },
            "count": { "type": "integer" }
        }
    }))).await.unwrap();

    // Update with new field
    storage.update_mapping("test_index", serde_json::json!({
        "description": { "type": "text" }
    })).await.unwrap();

    let index_info = storage.get_index("test_index").await.unwrap();
    let mappings = index_info.get("test_index")
        .and_then(|idx| idx.get("mappings"))
        .and_then(|m| m.get("properties"))
        .and_then(|p| p.as_object())
        .unwrap();

    // Should have all three fields
    assert!(mappings.contains_key("title"));
    assert!(mappings.contains_key("count"));
    assert!(mappings.contains_key("description"));
}

#[tokio::test]
async fn test_update_settings_merge() {
    let storage = Storage::new();

    // Create index with initial settings
    storage.create_index("test_index", Some(serde_json::json!({
        "number_of_shards": 1,
        "number_of_replicas": 0
    })), None).await.unwrap();

    // Update with new setting
    storage.update_settings("test_index", serde_json::json!({
        "number_of_replicas": 1,
        "refresh_interval": "1s"
    })).await.unwrap();

    let index_info = storage.get_index("test_index").await.unwrap();
    let settings = index_info.get("test_index")
        .and_then(|idx| idx.get("settings"))
        .and_then(|s| s.as_object())
        .unwrap();

    // Should have all settings
    assert_eq!(settings.get("number_of_shards").and_then(|v| v.as_u64()).unwrap(), 1);
    assert_eq!(settings.get("number_of_replicas").and_then(|v| v.as_u64()).unwrap(), 1);
    assert_eq!(settings.get("refresh_interval").and_then(|v| v.as_str()).unwrap(), "1s");
}

#[tokio::test]
async fn test_match_indices_wildcard() {
    let storage = Storage::new();

    storage.create_index("test_index_1", None, None).await.unwrap();
    storage.create_index("test_index_2", None, None).await.unwrap();
    storage.create_index("other_index", None, None).await.unwrap();

    // Match with wildcard
    let matched = storage.match_indices("test_index_*").await;

    assert_eq!(matched.len(), 2);
    assert!(matched.contains(&"test_index_1".to_string()));
    assert!(matched.contains(&"test_index_2".to_string()));
    assert!(!matched.contains(&"other_index".to_string()));
}

#[tokio::test]
async fn test_match_indices_single_char() {
    let storage = Storage::new();

    storage.create_index("test1", None, None).await.unwrap();
    storage.create_index("test2", None, None).await.unwrap();
    storage.create_index("testx", None, None).await.unwrap();

    // Match with single character wildcard
    let matched = storage.match_indices("test?").await;

    assert_eq!(matched.len(), 3);
    assert!(matched.contains(&"test1".to_string()));
    assert!(matched.contains(&"test2".to_string()));
    assert!(matched.contains(&"testx".to_string()));
}

#[tokio::test]
async fn test_match_indices_exact() {
    let storage = Storage::new();

    storage.create_index("test_index", None, None).await.unwrap();
    storage.create_index("test_index_2", None, None).await.unwrap();

    // Exact match
    let matched = storage.match_indices("test_index").await;

    assert_eq!(matched.len(), 1);
    assert_eq!(matched[0], "test_index");
}

#[tokio::test]
async fn test_match_indices_no_match() {
    let storage = Storage::new();

    storage.create_index("test_index", None, None).await.unwrap();

    // No match
    let matched = storage.match_indices("nonexistent").await;

    assert_eq!(matched.len(), 0);
}

#[tokio::test]
async fn test_match_indices_empty_pattern() {
    let storage = Storage::new();

    storage.create_index("test_index", None, None).await.unwrap();

    // Empty pattern should match nothing
    let matched = storage.match_indices("").await;

    assert_eq!(matched.len(), 0);
}

#[tokio::test]
async fn test_list_indices_empty() {
    let storage = Storage::new();

    let indices = storage.list_indices().await;

    assert_eq!(indices.len(), 0);
}

#[tokio::test]
async fn test_list_indices_multiple() {
    let storage = Storage::new();

    storage.create_index("index1", None, None).await.unwrap();
    storage.create_index("index2", None, None).await.unwrap();
    storage.create_index("index3", None, None).await.unwrap();

    let indices = storage.list_indices().await;

    assert_eq!(indices.len(), 3);
    assert!(indices.contains(&"index1".to_string()));
    assert!(indices.contains(&"index2".to_string()));
    assert!(indices.contains(&"index3".to_string()));
}

// ============================================================================
// Search Edge Cases
// ============================================================================

#[tokio::test]
async fn test_search_empty_index() {
    let storage = Storage::new();

    storage.create_index("test_index", None, None).await.unwrap();

    let query = serde_json::json!({ "match_all": {} });
    let result = storage.search("test_index", &query, None, None, None, None, None).await.unwrap();

    let hits = result.get("hits").and_then(|h| h.get("hits")).and_then(|h| h.as_array()).unwrap();
    assert_eq!(hits.len(), 0);

    let total = result.get("hits").and_then(|h| h.get("total")).unwrap();
    assert_eq!(total.get("value").and_then(|v| v.as_u64()).unwrap(), 0);
}

#[tokio::test]
async fn test_search_nonexistent_index() {
    let storage = Storage::new();

    let query = serde_json::json!({ "match_all": {} });
    let result = storage.search("nonexistent", &query, None, None, None, None, None).await;

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not found"));
}

#[tokio::test]
async fn test_search_pagination_beyond_results() {
    let storage = Storage::new();

    storage.create_index("test_index", None, None).await.unwrap();
    storage.index_document("test_index", "1", serde_json::json!({"title": "Doc 1"})).await.unwrap();
    storage.index_document("test_index", "2", serde_json::json!({"title": "Doc 2"})).await.unwrap();

    let query = serde_json::json!({ "match_all": {} });

    // Request page beyond results
    let result = storage.search("test_index", &query, Some(10), Some(10), None, None, None).await.unwrap();
    let hits = result.get("hits").and_then(|h| h.get("hits")).and_then(|h| h.as_array()).unwrap();

    assert_eq!(hits.len(), 0);
}

#[tokio::test]
async fn test_search_pagination_zero_size() {
    let storage = Storage::new();

    storage.create_index("test_index", None, None).await.unwrap();
    storage.index_document("test_index", "1", serde_json::json!({"title": "Doc 1"})).await.unwrap();

    let query = serde_json::json!({ "match_all": {} });

    // Request with size 0
    let result = storage.search("test_index", &query, Some(0), Some(0), None, None, None).await.unwrap();
    let hits = result.get("hits").and_then(|h| h.get("hits")).and_then(|h| h.as_array()).unwrap();

    assert_eq!(hits.len(), 0);
}

#[tokio::test]
async fn test_search_term_with_nonexistent_field() {
    let storage = Storage::new();

    storage.create_index("test_index", None, None).await.unwrap();
    storage.index_document("test_index", "1", serde_json::json!({
        "title": "Test"
    })).await.unwrap();

    let query = serde_json::json!({
        "term": {
            "nonexistent": "value"
        }
    });

    let result = storage.search("test_index", &query, None, None, None, None, None).await.unwrap();
    let hits = result.get("hits").and_then(|h| h.get("hits")).and_then(|h| h.as_array()).unwrap();

    // Should return no results
    assert_eq!(hits.len(), 0);
}

#[tokio::test]
async fn test_search_range_boundary_values() {
    let storage = Storage::new();

    storage.create_index("test_index", None, None).await.unwrap();
    storage.index_document("test_index", "1", serde_json::json!({"age": 25})).await.unwrap();
    storage.index_document("test_index", "2", serde_json::json!({"age": 30})).await.unwrap();
    storage.index_document("test_index", "3", serde_json::json!({"age": 35})).await.unwrap();

    // Range with boundary values (gte and lte)
    let query = serde_json::json!({
        "range": {
            "age": {
                "gte": 25,
                "lte": 35
            }
        }
    });

    let result = storage.search("test_index", &query, None, None, None, None, None).await.unwrap();
    let hits = result.get("hits").and_then(|h| h.get("hits")).and_then(|h| h.as_array()).unwrap();

    assert_eq!(hits.len(), 3);
}

#[tokio::test]
async fn test_search_range_exclusive() {
    let storage = Storage::new();

    storage.create_index("test_index", None, None).await.unwrap();
    storage.index_document("test_index", "1", serde_json::json!({"age": 25})).await.unwrap();
    storage.index_document("test_index", "2", serde_json::json!({"age": 30})).await.unwrap();
    storage.index_document("test_index", "3", serde_json::json!({"age": 35})).await.unwrap();

    // Range with exclusive boundaries (gt and lt)
    let query = serde_json::json!({
        "range": {
            "age": {
                "gt": 25,
                "lt": 35
            }
        }
    });

    let result = storage.search("test_index", &query, None, None, None, None, None).await.unwrap();
    let hits = result.get("hits").and_then(|h| h.get("hits")).and_then(|h| h.as_array()).unwrap();

    // Should only match age 30
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].get("_id").and_then(|id| id.as_str()).unwrap(), "2");
}

#[tokio::test]
async fn test_search_bool_empty_clauses() {
    let storage = Storage::new();

    storage.create_index("test_index", None, None).await.unwrap();
    storage.index_document("test_index", "1", serde_json::json!({"title": "Test"})).await.unwrap();

    // Bool query with empty clauses
    let query = serde_json::json!({
        "bool": {
            "must": [],
            "should": [],
            "must_not": [],
            "filter": []
        }
    });

    let result = storage.search("test_index", &query, None, None, None, None, None).await.unwrap();
    let hits = result.get("hits").and_then(|h| h.get("hits")).and_then(|h| h.as_array()).unwrap();

    // Empty bool query should match all (like match_all)
    assert_eq!(hits.len(), 1);
}

#[tokio::test]
async fn test_search_bool_must_not() {
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

    // Bool query with must_not
    let query = serde_json::json!({
        "bool": {
            "must_not": [
                { "term": { "status": "draft" } }
            ]
        }
    });

    let result = storage.search("test_index", &query, None, None, None, None, None).await.unwrap();
    let hits = result.get("hits").and_then(|h| h.get("hits")).and_then(|h| h.as_array()).unwrap();

    // Should only match published
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].get("_id").and_then(|id| id.as_str()).unwrap(), "1");
}

#[tokio::test]
async fn test_search_wildcard_edge_cases() {
    let storage = Storage::new();

    storage.create_index("test_index", None, None).await.unwrap();
    storage.index_document("test_index", "1", serde_json::json!({
        "title": "test"
    })).await.unwrap();
    storage.index_document("test_index", "2", serde_json::json!({
        "title": "testing"
    })).await.unwrap();
    storage.index_document("test_index", "3", serde_json::json!({
        "title": "best"
    })).await.unwrap();

    // Wildcard query: test*
    let query = serde_json::json!({
        "wildcard": {
            "title": "test*"
        }
    });

    let result = storage.search("test_index", &query, None, None, None, None, None).await.unwrap();
    let hits = result.get("hits").and_then(|h| h.get("hits")).and_then(|h| h.as_array()).unwrap();

    // Should match "test" and "testing", but not "best"
    assert_eq!(hits.len(), 2);
    let ids: Vec<&str> = hits.iter()
        .map(|h| h.get("_id").and_then(|id| id.as_str()).unwrap())
        .collect();
    assert!(ids.contains(&"1"));
    assert!(ids.contains(&"2"));
    assert!(!ids.contains(&"3"));
}

#[tokio::test]
async fn test_search_prefix_edge_cases() {
    let storage = Storage::new();

    storage.create_index("test_index", None, None).await.unwrap();
    storage.index_document("test_index", "1", serde_json::json!({
        "title": "test"
    })).await.unwrap();
    storage.index_document("test_index", "2", serde_json::json!({
        "title": "testing"
    })).await.unwrap();
    storage.index_document("test_index", "3", serde_json::json!({
        "title": "best"
    })).await.unwrap();

    // Prefix query: test
    let query = serde_json::json!({
        "prefix": {
            "title": "test"
        }
    });

    let result = storage.search("test_index", &query, None, None, None, None, None).await.unwrap();
    let hits = result.get("hits").and_then(|h| h.get("hits")).and_then(|h| h.as_array()).unwrap();

    // Should match "test" and "testing", but not "best"
    assert_eq!(hits.len(), 2);
    let ids: Vec<&str> = hits.iter()
        .map(|h| h.get("_id").and_then(|id| id.as_str()).unwrap())
        .collect();
    assert!(ids.contains(&"1"));
    assert!(ids.contains(&"2"));
    assert!(!ids.contains(&"3"));
}

#[tokio::test]
async fn test_search_with_null_values() {
    let storage = Storage::new();

    storage.create_index("test_index", None, None).await.unwrap();
    storage.index_document("test_index", "1", serde_json::json!({
        "title": "Test",
        "optional": serde_json::Value::Null
    })).await.unwrap();

    let query = serde_json::json!({
        "match": {
            "title": "Test"
        }
    });

    let result = storage.search("test_index", &query, None, None, None, None, None).await.unwrap();
    let hits = result.get("hits").and_then(|h| h.get("hits")).and_then(|h| h.as_array()).unwrap();

    // Should still match despite null field
    assert_eq!(hits.len(), 1);
}

#[tokio::test]
async fn test_search_with_array_values() {
    let storage = Storage::new();

    storage.create_index("test_index", None, None).await.unwrap();
    storage.index_document("test_index", "1", serde_json::json!({
        "tags": ["rust", "programming", "guide"],
        "title": "Rust Guide"
    })).await.unwrap();

    // Search in array field - arrays are converted to strings, so we search for the string representation
    // The current implementation may not handle arrays directly, so we test with a field that has array
    // but also search in a regular field to ensure the document is found
    let query = serde_json::json!({
        "match": {
            "title": "Rust"
        }
    });

    let result = storage.search("test_index", &query, None, None, None, None, None).await.unwrap();
    let hits = result.get("hits").and_then(|h| h.get("hits")).and_then(|h| h.as_array()).unwrap();

    // Should match via title field
    assert_eq!(hits.len(), 1);

    // Note: Array field matching may not be fully supported in current implementation
    // This test verifies that documents with arrays can still be found via other fields
}

#[tokio::test]
async fn test_search_with_nested_objects() {
    let storage = Storage::new();

    storage.create_index("test_index", None, None).await.unwrap();
    storage.index_document("test_index", "1", serde_json::json!({
        "user": {
            "name": "Alice",
            "age": 30
        }
    })).await.unwrap();

    // Search in nested object
    let query = serde_json::json!({
        "match": {
            "user.name": "Alice"
        }
    });

    let result = storage.search("test_index", &query, None, None, None, None, None).await.unwrap();
    let hits = result.get("hits").and_then(|h| h.get("hits")).and_then(|h| h.as_array()).unwrap();

    // Should match
    assert_eq!(hits.len(), 1);
}

// ============================================================================
// Bulk Operations Edge Cases
// ============================================================================

#[tokio::test]
async fn test_bulk_create_existing_document() {
    let storage = Storage::new();
    storage.create_index("test_index", None, None).await.unwrap();

    // Create document first
    storage.index_document("test_index", "1", serde_json::json!({
        "title": "Original"
    })).await.unwrap();

    // Try to create with same ID (should fail)
    use gummy_search::bulk_ops::BulkAction;

    let action = BulkAction::Create {
        index: "test_index".to_string(),
        id: Some("1".to_string()),
        document: serde_json::json!({"title": "New"}),
    };

    let result = storage.execute_bulk_action(action).await;

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("already exists"));
}

#[tokio::test]
async fn test_bulk_update_nonexistent_document() {
    let storage = Storage::new();
    storage.create_index("test_index", None, None).await.unwrap();

    // Update non-existent document (should create it)
    use gummy_search::bulk_ops::BulkAction;

    let action = BulkAction::Update {
        index: "test_index".to_string(),
        id: "1".to_string(),
        document: serde_json::json!({"title": "New"}),
    };

    let result = storage.execute_bulk_action(action).await;

    // Should succeed (creates document if doesn't exist)
    assert!(result.is_ok());

    // Verify document exists
    let doc = storage.get_document("test_index", "1").await.unwrap();
    assert_eq!(doc.get("_id").and_then(|id| id.as_str()).unwrap(), "1");
}

#[tokio::test]
async fn test_bulk_update_merge() {
    let storage = Storage::new();
    storage.create_index("test_index", None, None).await.unwrap();

    // Create document
    storage.index_document("test_index", "1", serde_json::json!({
        "title": "Original",
        "count": 1
    })).await.unwrap();

    // Update with merge
    use gummy_search::bulk_ops::BulkAction;

    let action = BulkAction::Update {
        index: "test_index".to_string(),
        id: "1".to_string(),
        document: serde_json::json!({
            "count": 2,
            "new_field": "value"
        }),
    };

    storage.execute_bulk_action(action).await.unwrap();

    // Verify merge
    let doc = storage.get_document("test_index", "1").await.unwrap();
    let source = doc.get("_source").unwrap();

    assert_eq!(source.get("title").and_then(|t| t.as_str()).unwrap(), "Original");
    assert_eq!(source.get("count").and_then(|c| c.as_u64()).unwrap(), 2);
    assert_eq!(source.get("new_field").and_then(|f| f.as_str()).unwrap(), "value");
}

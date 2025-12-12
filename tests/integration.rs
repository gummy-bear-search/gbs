// Integration tests for Gummy Search
// These tests verify the storage layer which is the core functionality
// For full HTTP endpoint tests, use a running server instance

#[cfg(test)]
mod tests {
    use gummy_search::storage::Storage;
    use serde_json;

    // Integration test: End-to-end search workflow
    #[tokio::test]
    async fn test_integration_search_workflow() {
        let storage = Storage::new();

        // Create index
        storage.create_index("test_index", None, None).await.unwrap();

        // Index multiple documents
        storage.index_document("test_index", "1", serde_json::json!({
            "title": "Rust Programming Guide",
            "author": "John Doe",
            "tags": ["rust", "programming"]
        })).await.unwrap();

        storage.index_document("test_index", "2", serde_json::json!({
            "title": "Python Tutorial",
            "author": "Jane Smith",
            "tags": ["python", "tutorial"]
        })).await.unwrap();

        // Search for "Rust"
        let query = serde_json::json!({
            "match": {
                "title": "Rust"
            }
        });

        let result = storage.search("test_index", &query, None, None, None).await.unwrap();
        let hits = result.get("hits")
            .and_then(|h| h.get("hits"))
            .and_then(|h| h.as_array())
            .unwrap();

        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].get("_id").and_then(|id| id.as_str()).unwrap(), "1");

        // Test pagination
        let query_all = serde_json::json!({ "match_all": {} });
        let result = storage.search("test_index", &query_all, Some(0), Some(1), None).await.unwrap();
        let hits = result.get("hits")
            .and_then(|h| h.get("hits"))
            .and_then(|h| h.as_array())
            .unwrap();
        assert_eq!(hits.len(), 1);
    }

    // Integration test: Mapping and settings updates
    #[tokio::test]
    async fn test_integration_mapping_settings() {
        let storage = Storage::new();

        storage.create_index("test_index", None, None).await.unwrap();

        // Update mapping
        let mappings = serde_json::json!({
            "title": { "type": "text" },
            "count": { "type": "integer" }
        });
        storage.update_mapping("test_index", mappings).await.unwrap();

        // Update settings
        let settings = serde_json::json!({
            "number_of_shards": 2
        });
        storage.update_settings("test_index", settings).await.unwrap();

        // Verify updates
        let index_info = storage.get_index("test_index").await.unwrap();
        assert!(index_info.get("test_index").is_some());
    }

    // Integration test: Multi-index operations
    #[tokio::test]
    async fn test_integration_multi_index() {
        let storage = Storage::new();

        // Create multiple indices
        storage.create_index("index1", None, None).await.unwrap();
        storage.create_index("index2", None, None).await.unwrap();

        // Add documents to each
        storage.index_document("index1", "1", serde_json::json!({"title": "Doc 1"})).await.unwrap();
        storage.index_document("index2", "2", serde_json::json!({"title": "Doc 2"})).await.unwrap();

        // Verify we can list indices
        let indices = storage.list_indices().await;
        assert_eq!(indices.len(), 2);
        assert!(indices.contains(&"index1".to_string()));
        assert!(indices.contains(&"index2".to_string()));

        // Delete all indices
        storage.delete_all_indices().await.unwrap();
        assert_eq!(storage.list_indices().await.len(), 0);
    }
}

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

        let result = storage.search("test_index", &query, None, None, None, None, None).await.unwrap();
        let hits = result.get("hits")
            .and_then(|h| h.get("hits"))
            .and_then(|h| h.as_array())
            .unwrap();

        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].get("_id").and_then(|id| id.as_str()).unwrap(), "1");

        // Test pagination
        let query_all = serde_json::json!({ "match_all": {} });
        let result = storage.search("test_index", &query_all, Some(0), Some(1), None, None, None).await.unwrap();
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

    // Integration test: Wildcard query
    #[tokio::test]
    async fn test_integration_wildcard_query() {
        let storage = Storage::new();
        storage.create_index("test_index", None, None).await.unwrap();

        // Index documents with various titles
        storage.index_document("test_index", "1", serde_json::json!({
            "title": "Rust Programming",
            "category": "programming"
        })).await.unwrap();

        storage.index_document("test_index", "2", serde_json::json!({
            "title": "Python Tutorial",
            "category": "tutorial"
        })).await.unwrap();

        storage.index_document("test_index", "3", serde_json::json!({
            "title": "Rusty Code",
            "category": "code"
        })).await.unwrap();

        // Test wildcard: *rust* should match "Rust Programming" and "Rusty Code"
        let query = serde_json::json!({
            "wildcard": {
                "title": "*rust*"
            }
        });

        let result = storage.search("test_index", &query, None, None, None, None, None).await.unwrap();
        let hits = result.get("hits")
            .and_then(|h| h.get("hits"))
            .and_then(|h| h.as_array())
            .unwrap();

        assert_eq!(hits.len(), 2);
        let ids: Vec<&str> = hits.iter()
            .map(|h| h.get("_id").and_then(|id| id.as_str()).unwrap())
            .collect();
        assert!(ids.contains(&"1"));
        assert!(ids.contains(&"3"));

        // Test wildcard: rust? should match "rusty" (if we had such a document)
        // Test wildcard: py* should match "Python Tutorial"
        let query2 = serde_json::json!({
            "wildcard": {
                "title": "py*"
            }
        });

        let result2 = storage.search("test_index", &query2, None, None, None, None, None).await.unwrap();
        let hits2 = result2.get("hits")
            .and_then(|h| h.get("hits"))
            .and_then(|h| h.as_array())
            .unwrap();

        assert_eq!(hits2.len(), 1);
        assert_eq!(hits2[0].get("_id").and_then(|id| id.as_str()).unwrap(), "2");
    }

    // Integration test: Prefix query
    #[tokio::test]
    async fn test_integration_prefix_query() {
        let storage = Storage::new();
        storage.create_index("test_index", None, None).await.unwrap();

        storage.index_document("test_index", "1", serde_json::json!({
            "title": "Rust Programming",
            "author": "John Doe"
        })).await.unwrap();

        storage.index_document("test_index", "2", serde_json::json!({
            "title": "Python Tutorial",
            "author": "Jane Smith"
        })).await.unwrap();

        storage.index_document("test_index", "3", serde_json::json!({
            "title": "Rusty Code",
            "author": "Bob Johnson"
        })).await.unwrap();

        // Test prefix: "rust" should match "Rust Programming" and "Rusty Code" (case-insensitive)
        let query = serde_json::json!({
            "prefix": {
                "title": "rust"
            }
        });

        let result = storage.search("test_index", &query, None, None, None, None, None).await.unwrap();
        let hits = result.get("hits")
            .and_then(|h| h.get("hits"))
            .and_then(|h| h.as_array())
            .unwrap();

        assert_eq!(hits.len(), 2);
        let ids: Vec<&str> = hits.iter()
            .map(|h| h.get("_id").and_then(|id| id.as_str()).unwrap())
            .collect();
        assert!(ids.contains(&"1"));
        assert!(ids.contains(&"3"));

        // Test prefix: "py" should match "Python Tutorial"
        let query2 = serde_json::json!({
            "prefix": {
                "title": "py"
            }
        });

        let result2 = storage.search("test_index", &query2, None, None, None, None, None).await.unwrap();
        let hits2 = result2.get("hits")
            .and_then(|h| h.get("hits"))
            .and_then(|h| h.as_array())
            .unwrap();

        assert_eq!(hits2.len(), 1);
        assert_eq!(hits2[0].get("_id").and_then(|id| id.as_str()).unwrap(), "2");
    }

    // Integration test: Terms query
    #[tokio::test]
    async fn test_integration_terms_query() {
        let storage = Storage::new();
        storage.create_index("test_index", None, None).await.unwrap();

        storage.index_document("test_index", "1", serde_json::json!({
            "title": "Rust Programming",
            "status": "published",
            "category": "programming"
        })).await.unwrap();

        storage.index_document("test_index", "2", serde_json::json!({
            "title": "Python Tutorial",
            "status": "draft",
            "category": "tutorial"
        })).await.unwrap();

        storage.index_document("test_index", "3", serde_json::json!({
            "title": "Java Guide",
            "status": "published",
            "category": "programming"
        })).await.unwrap();

        // Test terms: match documents with status "published" or "draft"
        let query = serde_json::json!({
            "terms": {
                "status": ["published", "draft"]
            }
        });

        let result = storage.search("test_index", &query, None, None, None, None, None).await.unwrap();
        let hits = result.get("hits")
            .and_then(|h| h.get("hits"))
            .and_then(|h| h.as_array())
            .unwrap();

        assert_eq!(hits.len(), 3); // All documents match

        // Test terms: match documents with category "programming"
        let query2 = serde_json::json!({
            "terms": {
                "category": ["programming"]
            }
        });

        let result2 = storage.search("test_index", &query2, None, None, None, None, None).await.unwrap();
        let hits2 = result2.get("hits")
            .and_then(|h| h.get("hits"))
            .and_then(|h| h.as_array())
            .unwrap();

        assert_eq!(hits2.len(), 2);
        let ids: Vec<&str> = hits2.iter()
            .map(|h| h.get("_id").and_then(|id| id.as_str()).unwrap())
            .collect();
        assert!(ids.contains(&"1"));
        assert!(ids.contains(&"3"));

        // Test terms with numeric values
        storage.index_document("test_index", "4", serde_json::json!({
            "title": "Go Basics",
            "year": 2023,
            "rating": 5
        })).await.unwrap();

        storage.index_document("test_index", "5", serde_json::json!({
            "title": "C++ Advanced",
            "year": 2024,
            "rating": 4
        })).await.unwrap();

        let query3 = serde_json::json!({
            "terms": {
                "year": [2023, 2024]
            }
        });

        let result3 = storage.search("test_index", &query3, None, None, None, None, None).await.unwrap();
        let hits3 = result3.get("hits")
            .and_then(|h| h.get("hits"))
            .and_then(|h| h.as_array())
            .unwrap();

        assert_eq!(hits3.len(), 2);
    }

    // Integration test: _source filtering
    #[tokio::test]
    async fn test_integration_source_filtering() {
        let storage = Storage::new();
        storage.create_index("test_index", None, None).await.unwrap();

        storage.index_document("test_index", "1", serde_json::json!({
            "title": "Rust Programming",
            "author": "John Doe",
            "year": 2023,
            "tags": ["rust", "programming"]
        })).await.unwrap();

        storage.index_document("test_index", "2", serde_json::json!({
            "title": "Python Tutorial",
            "author": "Jane Smith",
            "year": 2024,
            "tags": ["python", "tutorial"]
        })).await.unwrap();

        // Test: _source = false (exclude all fields)
        let query = serde_json::json!({
            "match_all": {}
        });
        let source_filter = Some(serde_json::json!(false));

        let result = storage.search("test_index", &query, None, None, None, source_filter.as_ref(), None).await.unwrap();
        let hits = result.get("hits")
            .and_then(|h| h.get("hits"))
            .and_then(|h| h.as_array())
            .unwrap();

        assert_eq!(hits.len(), 2);
        let source1 = hits[0].get("_source").unwrap();
        assert!(source1.as_object().unwrap().is_empty());

        // Test: _source = ["title", "author"] (include only specified fields)
        let source_filter2 = Some(serde_json::json!(["title", "author"]));
        let result2 = storage.search("test_index", &query, None, None, None, source_filter2.as_ref(), None).await.unwrap();
        let hits2 = result2.get("hits")
            .and_then(|h| h.get("hits"))
            .and_then(|h| h.as_array())
            .unwrap();

        assert_eq!(hits2.len(), 2);
        let source2 = hits2[0].get("_source").unwrap();
        let source_obj = source2.as_object().unwrap();
        assert!(source_obj.contains_key("title"));
        assert!(source_obj.contains_key("author"));
        assert!(!source_obj.contains_key("year"));
        assert!(!source_obj.contains_key("tags"));

        // Test: _source = {"includes": ["title"], "excludes": ["author"]}
        let source_filter3 = Some(serde_json::json!({
            "includes": ["title"],
            "excludes": ["author"]
        }));
        let result3 = storage.search("test_index", &query, None, None, None, source_filter3.as_ref(), None).await.unwrap();
        let hits3 = result3.get("hits")
            .and_then(|h| h.get("hits"))
            .and_then(|h| h.as_array())
            .unwrap();

        assert_eq!(hits3.len(), 2);
        let source3 = hits3[0].get("_source").unwrap();
        let source_obj3 = source3.as_object().unwrap();
        assert!(source_obj3.contains_key("title"));
        assert!(!source_obj3.contains_key("author"));
    }

    // Integration test: Search highlighting
    #[tokio::test]
    async fn test_integration_highlighting() {
        let storage = Storage::new();
        storage.create_index("test_index", None, None).await.unwrap();

        storage.index_document("test_index", "1", serde_json::json!({
            "title": "Rust Programming Guide",
            "body": "Learn Rust programming language"
        })).await.unwrap();

        storage.index_document("test_index", "2", serde_json::json!({
            "title": "Python Tutorial",
            "body": "Python is a great language"
        })).await.unwrap();

        // Test highlighting with match query
        let query = serde_json::json!({
            "match": {
                "title": "Rust"
            }
        });

        let highlight_config = Some(serde_json::json!({
            "fields": {
                "title": {}
            }
        }));

        let result = storage.search("test_index", &query, None, None, None, None, highlight_config.as_ref()).await.unwrap();
        let hits = result.get("hits")
            .and_then(|h| h.get("hits"))
            .and_then(|h| h.as_array())
            .unwrap();

        assert_eq!(hits.len(), 1);
        let highlight = hits[0].get("highlight");
        assert!(highlight.is_some());

        let highlight_obj = highlight.unwrap().as_object().unwrap();
        assert!(highlight_obj.contains_key("title"));
        let title_highlight = highlight_obj.get("title").unwrap().as_array().unwrap();
        assert!(title_highlight[0].as_str().unwrap().contains("<em>Rust</em>"));

        // Test highlighting with multiple fields
        let query2 = serde_json::json!({
            "match": {
                "body": "programming"
            }
        });

        let highlight_config2 = Some(serde_json::json!({
            "fields": {
                "title": {},
                "body": {}
            }
        }));

        let result2 = storage.search("test_index", &query2, None, None, None, None, highlight_config2.as_ref()).await.unwrap();
        let hits2 = result2.get("hits")
            .and_then(|h| h.get("hits"))
            .and_then(|h| h.as_array())
            .unwrap();

        assert_eq!(hits2.len(), 1);
        let highlight2 = hits2[0].get("highlight").unwrap().as_object().unwrap();
        assert!(highlight2.contains_key("body"));
        let body_highlight = highlight2.get("body").unwrap().as_array().unwrap();
        assert!(body_highlight[0].as_str().unwrap().contains("<em>programming</em>"));
    }

    // Integration test: Wildcard index patterns
    #[tokio::test]
    async fn test_integration_wildcard_indices() {
        let storage = Storage::new();

        // Create multiple indices with patterns
        storage.create_index("logs-2024-01", None, None).await.unwrap();
        storage.create_index("logs-2024-02", None, None).await.unwrap();
        storage.create_index("logs-2023-12", None, None).await.unwrap();
        storage.create_index("data-2024", None, None).await.unwrap();

        // Add documents to each index
        storage.index_document("logs-2024-01", "1", serde_json::json!({
            "message": "Log entry 1"
        })).await.unwrap();

        storage.index_document("logs-2024-02", "2", serde_json::json!({
            "message": "Log entry 2"
        })).await.unwrap();

        storage.index_document("data-2024", "3", serde_json::json!({
            "message": "Data entry"
        })).await.unwrap();

        // Test wildcard pattern: logs-2024-*
        let matched = storage.match_indices("logs-2024-*").await;
        assert_eq!(matched.len(), 2);
        assert!(matched.contains(&"logs-2024-01".to_string()));
        assert!(matched.contains(&"logs-2024-02".to_string()));
        assert!(!matched.contains(&"logs-2023-12".to_string()));

        // Test wildcard pattern: logs-*
        let matched2 = storage.match_indices("logs-*").await;
        assert_eq!(matched2.len(), 3);
        assert!(matched2.contains(&"logs-2024-01".to_string()));
        assert!(matched2.contains(&"logs-2024-02".to_string()));
        assert!(matched2.contains(&"logs-2023-12".to_string()));

        // Test wildcard pattern: *-2024
        let matched3 = storage.match_indices("*-2024").await;
        assert_eq!(matched3.len(), 1);
        assert!(matched3.contains(&"data-2024".to_string()));

        // Test single character wildcard: logs-202?-01
        let matched4 = storage.match_indices("logs-202?-01").await;
        assert_eq!(matched4.len(), 1);
        assert!(matched4.contains(&"logs-2024-01".to_string()));
    }
}

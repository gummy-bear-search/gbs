//! Unit tests for Storage module

use gbs::storage::Storage;

#[tokio::test]
async fn test_search_match_query() {
    let storage = Storage::new();

    // Create index
    storage
        .create_index("test_index", None, None)
        .await
        .unwrap();

    // Index documents
    storage
        .index_document(
            "test_index",
            "1",
            serde_json::json!({
                "title": "Rust Programming",
                "content": "Learn Rust programming language"
            }),
        )
        .await
        .unwrap();

    storage
        .index_document(
            "test_index",
            "2",
            serde_json::json!({
                "title": "Python Tutorial",
                "content": "Learn Python programming"
            }),
        )
        .await
        .unwrap();

    // Search for "Rust"
    let query = serde_json::json!({
        "match": {
            "title": "Rust"
        }
    });

    let result = storage
        .search("test_index", &query, None, None, None, None, None)
        .await
        .unwrap();
    let hits = result
        .get("hits")
        .and_then(|h| h.get("hits"))
        .and_then(|h| h.as_array())
        .unwrap();

    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].get("_id").and_then(|id| id.as_str()).unwrap(), "1");
}

#[tokio::test]
async fn test_search_match_all() {
    let storage = Storage::new();

    storage
        .create_index("test_index", None, None)
        .await
        .unwrap();
    storage
        .index_document("test_index", "1", serde_json::json!({"title": "Doc 1"}))
        .await
        .unwrap();
    storage
        .index_document("test_index", "2", serde_json::json!({"title": "Doc 2"}))
        .await
        .unwrap();

    let query = serde_json::json!({
        "match_all": {}
    });

    let result = storage
        .search("test_index", &query, None, None, None, None, None)
        .await
        .unwrap();
    let hits = result
        .get("hits")
        .and_then(|h| h.get("hits"))
        .and_then(|h| h.as_array())
        .unwrap();

    assert_eq!(hits.len(), 2);
}

#[tokio::test]
async fn test_search_pagination() {
    let storage = Storage::new();

    storage
        .create_index("test_index", None, None)
        .await
        .unwrap();
    for i in 1..=10 {
        storage
            .index_document(
                "test_index",
                &i.to_string(),
                serde_json::json!({
                    "title": format!("Doc {}", i)
                }),
            )
            .await
            .unwrap();
    }

    let query = serde_json::json!({ "match_all": {} });

    // First page
    let result = storage
        .search("test_index", &query, Some(0), Some(5), None, None, None)
        .await
        .unwrap();
    let hits = result
        .get("hits")
        .and_then(|h| h.get("hits"))
        .and_then(|h| h.as_array())
        .unwrap();
    assert_eq!(hits.len(), 5);

    // Second page
    let result = storage
        .search("test_index", &query, Some(5), Some(5), None, None, None)
        .await
        .unwrap();
    let hits = result
        .get("hits")
        .and_then(|h| h.get("hits"))
        .and_then(|h| h.as_array())
        .unwrap();
    assert_eq!(hits.len(), 5);
}

#[tokio::test]
async fn test_search_term_query() {
    let storage = Storage::new();

    storage
        .create_index("test_index", None, None)
        .await
        .unwrap();
    storage
        .index_document(
            "test_index",
            "1",
            serde_json::json!({
                "status": "active",
                "name": "Test"
            }),
        )
        .await
        .unwrap();
    storage
        .index_document(
            "test_index",
            "2",
            serde_json::json!({
                "status": "inactive",
                "name": "Test"
            }),
        )
        .await
        .unwrap();

    let query = serde_json::json!({
        "term": {
            "status": "active"
        }
    });

    let result = storage
        .search("test_index", &query, None, None, None, None, None)
        .await
        .unwrap();
    let hits = result
        .get("hits")
        .and_then(|h| h.get("hits"))
        .and_then(|h| h.as_array())
        .unwrap();

    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].get("_id").and_then(|id| id.as_str()).unwrap(), "1");
}

#[tokio::test]
async fn test_search_bool_query() {
    let storage = Storage::new();

    storage
        .create_index("test_index", None, None)
        .await
        .unwrap();
    storage
        .index_document(
            "test_index",
            "1",
            serde_json::json!({
                "title": "Rust Guide",
                "status": "published"
            }),
        )
        .await
        .unwrap();
    storage
        .index_document(
            "test_index",
            "2",
            serde_json::json!({
                "title": "Python Guide",
                "status": "draft"
            }),
        )
        .await
        .unwrap();

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

    let result = storage
        .search("test_index", &query, None, None, None, None, None)
        .await
        .unwrap();
    let hits = result
        .get("hits")
        .and_then(|h| h.get("hits"))
        .and_then(|h| h.as_array())
        .unwrap();

    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].get("_id").and_then(|id| id.as_str()).unwrap(), "1");
}

#[tokio::test]
async fn test_update_mapping() {
    let storage = Storage::new();

    storage
        .create_index("test_index", None, None)
        .await
        .unwrap();

    let new_mappings = serde_json::json!({
        "title": { "type": "text" },
        "count": { "type": "integer" }
    });

    storage
        .update_mapping("test_index", new_mappings.clone())
        .await
        .unwrap();

    let index_info = storage.get_index("test_index").await.unwrap();
    let mappings = index_info
        .get("test_index")
        .and_then(|idx| idx.get("mappings"))
        .and_then(|m| m.get("properties"));

    assert!(mappings.is_some());
}

#[tokio::test]
async fn test_update_settings() {
    let storage = Storage::new();

    storage
        .create_index("test_index", None, None)
        .await
        .unwrap();

    let new_settings = serde_json::json!({
        "number_of_shards": 2,
        "number_of_replicas": 1
    });

    storage
        .update_settings("test_index", new_settings.clone())
        .await
        .unwrap();

    let index_info = storage.get_index("test_index").await.unwrap();
    let settings = index_info
        .get("test_index")
        .and_then(|idx| idx.get("settings"));

    assert!(settings.is_some());
    assert_eq!(
        settings
            .and_then(|s| s.get("number_of_shards"))
            .and_then(|v| v.as_u64())
            .unwrap(),
        2
    );
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

    storage
        .create_index("test_index", None, None)
        .await
        .unwrap();
    storage
        .index_document(
            "test_index",
            "1",
            serde_json::json!({
                "name": "Alice",
                "age": 30
            }),
        )
        .await
        .unwrap();
    storage
        .index_document(
            "test_index",
            "2",
            serde_json::json!({
                "name": "Bob",
                "age": 25
            }),
        )
        .await
        .unwrap();
    storage
        .index_document(
            "test_index",
            "3",
            serde_json::json!({
                "name": "Charlie",
                "age": 35
            }),
        )
        .await
        .unwrap();

    let query = serde_json::json!({ "match_all": {} });
    let sort = serde_json::json!({
        "age": {
            "order": "asc"
        }
    });

    let result = storage
        .search("test_index", &query, None, None, Some(&sort), None, None)
        .await
        .unwrap();
    let hits = result
        .get("hits")
        .and_then(|h| h.get("hits"))
        .and_then(|h| h.as_array())
        .unwrap();

    assert_eq!(hits.len(), 3);
    assert_eq!(hits[0].get("_id").and_then(|id| id.as_str()).unwrap(), "2"); // Bob, age 25
    assert_eq!(hits[1].get("_id").and_then(|id| id.as_str()).unwrap(), "1"); // Alice, age 30
    assert_eq!(hits[2].get("_id").and_then(|id| id.as_str()).unwrap(), "3"); // Charlie, age 35
}

#[tokio::test]
async fn test_search_match_phrase() {
    let storage = Storage::new();

    storage
        .create_index("test_index", None, None)
        .await
        .unwrap();
    storage
        .index_document(
            "test_index",
            "1",
            serde_json::json!({
                "title": "Rust Programming Guide",
                "content": "Learn Rust programming language"
            }),
        )
        .await
        .unwrap();

    storage
        .index_document(
            "test_index",
            "2",
            serde_json::json!({
                "title": "Python Tutorial",
                "content": "Learn Python programming"
            }),
        )
        .await
        .unwrap();

    // Match phrase query - exact phrase match
    let query = serde_json::json!({
        "match_phrase": {
            "title": "Rust Programming"
        }
    });

    let result = storage
        .search("test_index", &query, None, None, None, None, None)
        .await
        .unwrap();
    let hits = result
        .get("hits")
        .and_then(|h| h.get("hits"))
        .and_then(|h| h.as_array())
        .unwrap();

    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].get("_id").and_then(|id| id.as_str()).unwrap(), "1");
}

#[tokio::test]
async fn test_search_multi_match() {
    let storage = Storage::new();

    storage
        .create_index("test_index", None, None)
        .await
        .unwrap();
    storage
        .index_document(
            "test_index",
            "1",
            serde_json::json!({
                "title": "Rust Guide",
                "description": "Learn Rust",
                "tags": "programming"
            }),
        )
        .await
        .unwrap();

    storage
        .index_document(
            "test_index",
            "2",
            serde_json::json!({
                "title": "Python Tutorial",
                "description": "Learn Python",
                "tags": "tutorial"
            }),
        )
        .await
        .unwrap();

    // Multi-match query - search across multiple fields
    let query = serde_json::json!({
        "multi_match": {
            "query": "Rust",
            "fields": ["title", "description"]
        }
    });

    let result = storage
        .search("test_index", &query, None, None, None, None, None)
        .await
        .unwrap();
    let hits = result
        .get("hits")
        .and_then(|h| h.get("hits"))
        .and_then(|h| h.as_array())
        .unwrap();

    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].get("_id").and_then(|id| id.as_str()).unwrap(), "1");
}

#[tokio::test]
async fn test_search_range() {
    let storage = Storage::new();

    storage
        .create_index("test_index", None, None)
        .await
        .unwrap();
    storage
        .index_document(
            "test_index",
            "1",
            serde_json::json!({
                "name": "Alice",
                "age": 25
            }),
        )
        .await
        .unwrap();
    storage
        .index_document(
            "test_index",
            "2",
            serde_json::json!({
                "name": "Bob",
                "age": 30
            }),
        )
        .await
        .unwrap();
    storage
        .index_document(
            "test_index",
            "3",
            serde_json::json!({
                "name": "Charlie",
                "age": 35
            }),
        )
        .await
        .unwrap();

    // Range query - age between 28 and 40
    let query = serde_json::json!({
        "range": {
            "age": {
                "gte": 28,
                "lte": 40
            }
        }
    });

    let result = storage
        .search("test_index", &query, None, None, None, None, None)
        .await
        .unwrap();
    let hits = result
        .get("hits")
        .and_then(|h| h.get("hits"))
        .and_then(|h| h.as_array())
        .unwrap();

    assert_eq!(hits.len(), 2);
    let ids: Vec<&str> = hits
        .iter()
        .map(|h| h.get("_id").and_then(|id| id.as_str()).unwrap())
        .collect();
    assert!(ids.contains(&"2")); // Bob, age 30
    assert!(ids.contains(&"3")); // Charlie, age 35
}

#[tokio::test]
async fn test_search_range_gt_lt() {
    let storage = Storage::new();

    storage
        .create_index("test_index", None, None)
        .await
        .unwrap();
    storage
        .index_document(
            "test_index",
            "1",
            serde_json::json!({
                "price": 10.0
            }),
        )
        .await
        .unwrap();
    storage
        .index_document(
            "test_index",
            "2",
            serde_json::json!({
                "price": 20.0
            }),
        )
        .await
        .unwrap();
    storage
        .index_document(
            "test_index",
            "3",
            serde_json::json!({
                "price": 30.0
            }),
        )
        .await
        .unwrap();

    // Range query with gt and lt
    let query = serde_json::json!({
        "range": {
            "price": {
                "gt": 10.0,
                "lt": 30.0
            }
        }
    });

    let result = storage
        .search("test_index", &query, None, None, None, None, None)
        .await
        .unwrap();
    let hits = result
        .get("hits")
        .and_then(|h| h.get("hits"))
        .and_then(|h| h.as_array())
        .unwrap();

    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].get("_id").and_then(|id| id.as_str()).unwrap(), "2"); // price 20.0
}

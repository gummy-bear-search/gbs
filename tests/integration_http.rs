//! HTTP endpoint integration tests
//!
//! These tests verify the HTTP API layer using axum-test to test handlers
//! without requiring a running server.

use axum_test::TestServer;
use axum_test::http::StatusCode;
use gummy_search::server::{create_router, AppState};
use gummy_search::storage::Storage;
use serde_json::json;
use std::sync::Arc;

/// Helper function to create a test server with in-memory storage
fn create_test_server() -> TestServer {
    let storage = Storage::new();
    let state = AppState {
        storage: Arc::new(storage),
        es_version: "6.8.23".to_string(),
    };
    let app = create_router(state);
    // For axum 0.7, use axum-test 16 which is compatible
    // The router can be used directly with axum-test 16
    TestServer::new(app).unwrap()
}

#[tokio::test]
async fn test_cluster_health() {
    let server = create_test_server();

    let response = server.get("/_cluster/health").await;

    response.assert_status_ok();
    let body: serde_json::Value = response.json();
    assert_eq!(body["status"], "green");
    assert_eq!(body["number_of_nodes"], 1);
    assert_eq!(body["number_of_data_nodes"], 1);
}

#[tokio::test]
async fn test_cluster_stats() {
    let server = create_test_server();

    let response = server.get("/_cluster/stats").await;

    response.assert_status_ok();
    let body: serde_json::Value = response.json();
    assert_eq!(body["cluster_name"], "gummy-search");
    assert_eq!(body["status"], "green");
    assert!(body["nodes"]["versions"].is_array());
    assert_eq!(body["nodes"]["versions"][0], "6.8.23");
}

#[tokio::test]
async fn test_create_index() {
    let server = create_test_server();

    let index_body = json!({
        "settings": {
            "number_of_shards": 1,
            "number_of_replicas": 0
        },
        "mappings": {
            "properties": {
                "title": { "type": "text" },
                "body": { "type": "text" }
            }
        }
    });

    let response = server
        .put("/test_index")
        .json(&index_body)
        .await;

    response.assert_status_ok();
}

#[tokio::test]
async fn test_create_index_already_exists() {
    let server = create_test_server();

    let index_body = json!({
        "settings": {
            "number_of_shards": 1
        }
    });

    // Create index first time
    let response1 = server
        .put("/test_index")
        .json(&index_body)
        .await;
    response1.assert_status_ok();

    // Try to create again - should fail
    let response2 = server
        .put("/test_index")
        .json(&index_body)
        .await;
    response2.assert_status(StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_get_index() {
    let server = create_test_server();

    // Create index first
    let index_body = json!({
        "settings": { "number_of_shards": 1 },
        "mappings": {
            "properties": {
                "title": { "type": "text" }
            }
        }
    });

    server.put("/test_index").json(&index_body).await;

    // Get index
    let response = server.get("/test_index").await;
    response.assert_status_ok();

    let body: serde_json::Value = response.json();
    assert!(body["test_index"].is_object());
    assert!(body["test_index"]["mappings"].is_object());
}

#[tokio::test]
async fn test_get_index_not_found() {
    let server = create_test_server();

    let response = server.get("/nonexistent_index").await;
    response.assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_check_index_exists() {
    let server = create_test_server();

    // Create index
    let index_body = json!({
        "settings": { "number_of_shards": 1 }
    });
    server.put("/test_index").json(&index_body).await;

    // Check existence with HEAD - use GET and check status
    // Note: axum-test doesn't have direct HEAD support, so we verify via GET
    let response = server.get("/test_index").await;
    response.assert_status_ok();
}

#[tokio::test]
async fn test_check_index_not_exists() {
    let server = create_test_server();

    // Note: axum-test doesn't have direct HEAD support, so we verify via GET
    let response = server.get("/nonexistent_index").await;
    response.assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_delete_index() {
    let server = create_test_server();

    // Create index
    let index_body = json!({
        "settings": { "number_of_shards": 1 }
    });
    server.put("/test_index").json(&index_body).await;

    // Delete index
    let response = server.delete("/test_index").await;
    response.assert_status_ok();

    // Verify it's gone
    let check_response = server.get("/test_index").await;
    check_response.assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_delete_index_not_found() {
    let server = create_test_server();

    let response = server.delete("/nonexistent_index").await;
    response.assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_index_document() {
    let server = create_test_server();

    // Create index
    let index_body = json!({
        "settings": { "number_of_shards": 1 }
    });
    server.put("/test_index").json(&index_body).await;

    // Index document
    let doc = json!({
        "title": "Test Document",
        "body": "This is a test"
    });

    let response = server
        .put("/test_index/_doc/1")
        .json(&doc)
        .await;

    // PUT with new document returns 201 (Created) - no body, just status
    response.assert_status(StatusCode::CREATED);
}

#[tokio::test]
async fn test_get_document() {
    let server = create_test_server();

    // Create index and document
    let index_body = json!({
        "settings": { "number_of_shards": 1 }
    });
    server.put("/test_index").json(&index_body).await;

    let doc = json!({
        "title": "Test Document",
        "body": "This is a test"
    });
    server.put("/test_index/_doc/1").json(&doc).await;

    // Get document
    let response = server.get("/test_index/_doc/1").await;
    response.assert_status_ok();

    let body: serde_json::Value = response.json();
    assert_eq!(body["_id"], "1");
    assert_eq!(body["_source"]["title"], "Test Document");
}

#[tokio::test]
async fn test_get_document_not_found() {
    let server = create_test_server();

    // Create index
    let index_body = json!({
        "settings": { "number_of_shards": 1 }
    });
    server.put("/test_index").json(&index_body).await;

    // Try to get non-existent document
    let response = server.get("/test_index/_doc/999").await;
    response.assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_create_document_auto_id() {
    let server = create_test_server();

    // Create index
    let index_body = json!({
        "settings": { "number_of_shards": 1 }
    });
    server.put("/test_index").json(&index_body).await;

    // Create document without ID (auto-generate)
    let doc = json!({
        "title": "Auto ID Document",
        "body": "This has an auto-generated ID"
    });

    let response = server
        .post("/test_index/_doc")
        .json(&doc)
        .await;

    response.assert_status_ok();
    let body: serde_json::Value = response.json();
    assert!(body["_id"].is_string());
    assert!(!body["_id"].as_str().unwrap().is_empty());
}

#[tokio::test]
async fn test_delete_document() {
    let server = create_test_server();

    // Create index and document
    let index_body = json!({
        "settings": { "number_of_shards": 1 }
    });
    server.put("/test_index").json(&index_body).await;

    let doc = json!({
        "title": "To Be Deleted",
        "body": "This will be deleted"
    });
    server.put("/test_index/_doc/1").json(&doc).await;

    // Delete document
    let response = server.delete("/test_index/_doc/1").await;
    response.assert_status_ok();

    // Verify it's gone
    let get_response = server.get("/test_index/_doc/1").await;
    get_response.assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_search_match_query() {
    let server = create_test_server();

    // Create index
    let index_body = json!({
        "settings": { "number_of_shards": 1 }
    });
    server.put("/test_index").json(&index_body).await;

    // Index documents
    server.put("/test_index/_doc/1")
        .json(&json!({ "title": "Rust Programming", "body": "Learn Rust" }))
        .await;
    server.put("/test_index/_doc/2")
        .json(&json!({ "title": "Python Tutorial", "body": "Learn Python" }))
        .await;

    // Search
    let query = json!({
        "query": {
            "match": {
                "title": "Rust"
            }
        }
    });

    let response = server
        .post("/test_index/_search")
        .json(&query)
        .await;

    response.assert_status_ok();
    let body: serde_json::Value = response.json();
    assert!(body["hits"]["hits"].is_array());
    let hits = body["hits"]["hits"].as_array().unwrap();
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0]["_id"], "1");
}

#[tokio::test]
async fn test_search_match_all() {
    let server = create_test_server();

    // Create index and documents
    let index_body = json!({
        "settings": { "number_of_shards": 1 }
    });
    server.put("/test_index").json(&index_body).await;

    server.put("/test_index/_doc/1")
        .json(&json!({ "title": "Doc 1" }))
        .await;
    server.put("/test_index/_doc/2")
        .json(&json!({ "title": "Doc 2" }))
        .await;

    // Search all
    let query = json!({
        "query": {
            "match_all": {}
        }
    });

    let response = server
        .post("/test_index/_search")
        .json(&query)
        .await;

    response.assert_status_ok();
    let body: serde_json::Value = response.json();
    let hits = body["hits"]["hits"].as_array().unwrap();
    assert_eq!(hits.len(), 2);
}

#[tokio::test]
async fn test_search_with_pagination() {
    let server = create_test_server();

    // Create index and documents
    let index_body = json!({
        "settings": { "number_of_shards": 1 }
    });
    server.put("/test_index").json(&index_body).await;

    for i in 1..=5 {
        server.put(&format!("/test_index/_doc/{}", i))
            .json(&json!({ "title": format!("Doc {}", i) }))
            .await;
    }

    // Search with pagination
    let query = json!({
        "query": { "match_all": {} },
        "from": 0,
        "size": 2
    });

    let response = server
        .post("/test_index/_search")
        .json(&query)
        .await;

    response.assert_status_ok();
    let body: serde_json::Value = response.json();
    let hits = body["hits"]["hits"].as_array().unwrap();
    assert_eq!(hits.len(), 2);
    assert_eq!(body["hits"]["total"]["value"], 5);
}

#[tokio::test]
async fn test_cat_indices() {
    let server = create_test_server();

    // Create some indices
    let index_body = json!({
        "settings": { "number_of_shards": 1 }
    });
    server.put("/index1").json(&index_body).await;
    server.put("/index2").json(&index_body).await;

    // Get indices list
    let response = server.get("/_cat/indices?v").await;
    response.assert_status_ok();

    let body = response.text();
    assert!(body.contains("index1"));
    assert!(body.contains("index2"));
}

#[tokio::test]
async fn test_get_aliases() {
    let server = create_test_server();

    // Create index
    let index_body = json!({
        "settings": { "number_of_shards": 1 }
    });
    server.put("/test_index").json(&index_body).await;

    // Get aliases
    let response = server.get("/_aliases").await;
    response.assert_status_ok();

    let body: serde_json::Value = response.json();
    assert!(body.is_object());
    // Current implementation returns empty aliases, which is valid
}

#[tokio::test]
async fn test_update_mapping() {
    let server = create_test_server();

    // Create index
    let index_body = json!({
        "settings": { "number_of_shards": 1 },
        "mappings": {
            "properties": {
                "title": { "type": "text" }
            }
        }
    });
    server.put("/test_index").json(&index_body).await;

    // Update mapping
    let new_mapping = json!({
        "properties": {
            "title": { "type": "text" },
            "body": { "type": "text" }
        }
    });

    let response = server
        .put("/test_index/_mapping")
        .json(&new_mapping)
        .await;

    response.assert_status_ok();
}

#[tokio::test]
async fn test_update_settings() {
    let server = create_test_server();

    // Create index
    let index_body = json!({
        "settings": { "number_of_shards": 1 }
    });
    server.put("/test_index").json(&index_body).await;

    // Update settings
    let new_settings = json!({
        "analysis": {
            "analyzer": {
                "custom": {
                    "type": "custom",
                    "tokenizer": "standard"
                }
            }
        }
    });

    let response = server
        .put("/test_index/_settings")
        .json(&new_settings)
        .await;

    response.assert_status_ok();
}

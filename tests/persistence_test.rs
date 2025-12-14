// Test persistent storage with Sled

#[cfg(test)]
mod tests {
    use gbs::storage::Storage;
    use serde_json;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_persistence_across_restarts() {
        // Create a temporary directory for the test
        let temp_dir = TempDir::new().unwrap();
        let data_path = temp_dir.path().join("test_db");

        // Create storage and add data
        {
            let storage = Storage::with_sled(&data_path).unwrap();
            storage.load_from_backend().await.unwrap();

            // Create index
            storage
                .create_index("test_index", None, None)
                .await
                .unwrap();

            // Add document
            storage
                .index_document(
                    "test_index",
                    "1",
                    serde_json::json!({
                        "title": "Test Document",
                        "content": "This should persist"
                    }),
                )
                .await
                .unwrap();

            // Flush to ensure data is written
            storage.flush().await.unwrap();
        }

        // Create new storage instance (simulating restart)
        {
            let storage = Storage::with_sled(&data_path).unwrap();
            storage.load_from_backend().await.unwrap();

            // Verify index exists
            assert!(storage.index_exists("test_index").await.unwrap());

            // Verify document exists
            let doc = storage.get_document("test_index", "1").await.unwrap();
            let source = doc.get("_source").unwrap();
            assert_eq!(
                source.get("title").unwrap().as_str().unwrap(),
                "Test Document"
            );
        }
    }

    #[tokio::test]
    async fn test_persistence_multiple_indices() {
        let temp_dir = TempDir::new().unwrap();
        let data_path = temp_dir.path().join("test_db");

        {
            let storage = Storage::with_sled(&data_path).unwrap();
            storage.load_from_backend().await.unwrap();

            storage.create_index("index1", None, None).await.unwrap();
            storage.create_index("index2", None, None).await.unwrap();

            storage
                .index_document("index1", "1", serde_json::json!({"data": "one"}))
                .await
                .unwrap();
            storage
                .index_document("index2", "2", serde_json::json!({"data": "two"}))
                .await
                .unwrap();

            // Flush to ensure data is written
            storage.flush().await.unwrap();
        }

        {
            let storage = Storage::with_sled(&data_path).unwrap();
            storage.load_from_backend().await.unwrap();

            let indices = storage.list_indices().await;
            assert_eq!(indices.len(), 2);
            assert!(indices.contains(&"index1".to_string()));
            assert!(indices.contains(&"index2".to_string()));
        }
    }
}

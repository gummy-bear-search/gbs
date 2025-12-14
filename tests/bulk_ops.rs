//! Unit tests for bulk operations parsing

use gbs::bulk_ops::{parse_bulk_ndjson, BulkAction};

#[test]
fn test_parse_bulk_index_operations() {
    let bulk_body = r#"{"index":{"_index":"test_index","_id":"1"}}
{"title":"Document 1","body":"Content 1"}
{"index":{"_index":"test_index","_id":"2"}}
{"title":"Document 2","body":"Content 2"}
"#;

    let actions = parse_bulk_ndjson(bulk_body, None).unwrap();
    assert_eq!(actions.len(), 2);

    match &actions[0] {
        BulkAction::Index {
            index,
            id,
            document,
        } => {
            assert_eq!(index, "test_index");
            assert_eq!(id, &Some("1".to_string()));
            assert_eq!(document["title"], "Document 1");
            assert_eq!(document["body"], "Content 1");
        }
        _ => panic!("Expected Index action"),
    }

    match &actions[1] {
        BulkAction::Index {
            index,
            id,
            document,
        } => {
            assert_eq!(index, "test_index");
            assert_eq!(id, &Some("2".to_string()));
            assert_eq!(document["title"], "Document 2");
        }
        _ => panic!("Expected Index action"),
    }
}

#[test]
fn test_parse_bulk_create_operations() {
    let bulk_body = r#"{"create":{"_index":"test_index","_id":"1"}}
{"title":"Create Doc 1","body":"Content 1"}
{"create":{"_index":"test_index","_id":"2"}}
{"title":"Create Doc 2","body":"Content 2"}
"#;

    let actions = parse_bulk_ndjson(bulk_body, None).unwrap();
    assert_eq!(actions.len(), 2);

    match &actions[0] {
        BulkAction::Create {
            index,
            id,
            document,
        } => {
            assert_eq!(index, "test_index");
            assert_eq!(id, &Some("1".to_string()));
            assert_eq!(document["title"], "Create Doc 1");
        }
        _ => panic!("Expected Create action"),
    }
}

#[test]
fn test_parse_bulk_update_operations() {
    let bulk_body = r#"{"update":{"_index":"test_index","_id":"1"}}
{"doc":{"title":"Updated Title","body":"Updated content"}}
"#;

    let actions = parse_bulk_ndjson(bulk_body, None).unwrap();
    assert_eq!(actions.len(), 1);

    match &actions[0] {
        BulkAction::Update {
            index,
            id,
            document,
        } => {
            assert_eq!(index, "test_index");
            assert_eq!(id, "1");
            // Update should extract "doc" field
            assert_eq!(document["title"], "Updated Title");
            assert_eq!(document["body"], "Updated content");
        }
        _ => panic!("Expected Update action"),
    }
}

#[test]
fn test_parse_bulk_delete_operations() {
    let bulk_body = r#"{"delete":{"_index":"test_index","_id":"1"}}
{"delete":{"_index":"test_index","_id":"2"}}
"#;

    let actions = parse_bulk_ndjson(bulk_body, None).unwrap();
    assert_eq!(actions.len(), 2);

    match &actions[0] {
        BulkAction::Delete { index, id } => {
            assert_eq!(index, "test_index");
            assert_eq!(id, "1");
        }
        _ => panic!("Expected Delete action"),
    }

    match &actions[1] {
        BulkAction::Delete { index, id } => {
            assert_eq!(index, "test_index");
            assert_eq!(id, "2");
        }
        _ => panic!("Expected Delete action"),
    }
}

#[test]
fn test_parse_bulk_mixed_operations() {
    let bulk_body = r#"{"index":{"_index":"test_index","_id":"1"}}
{"title":"Indexed Doc"}
{"create":{"_index":"test_index","_id":"2"}}
{"title":"Created Doc"}
{"update":{"_index":"test_index","_id":"1"}}
{"doc":{"title":"Updated"}}
{"delete":{"_index":"test_index","_id":"2"}}
"#;

    let actions = parse_bulk_ndjson(bulk_body, None).unwrap();
    assert_eq!(actions.len(), 4);

    match &actions[0] {
        BulkAction::Index { .. } => {}
        _ => panic!("Expected Index action"),
    }

    match &actions[1] {
        BulkAction::Create { .. } => {}
        _ => panic!("Expected Create action"),
    }

    match &actions[2] {
        BulkAction::Update { .. } => {}
        _ => panic!("Expected Update action"),
    }

    match &actions[3] {
        BulkAction::Delete { .. } => {}
        _ => panic!("Expected Delete action"),
    }
}

#[test]
fn test_parse_bulk_with_default_index() {
    let bulk_body = r#"{"index":{"_id":"1"}}
{"title":"Doc 1"}
{"index":{"_id":"2"}}
{"title":"Doc 2"}
"#;

    let actions = parse_bulk_ndjson(bulk_body, Some("default_index")).unwrap();
    assert_eq!(actions.len(), 2);

    match &actions[0] {
        BulkAction::Index { index, id, .. } => {
            assert_eq!(index, "default_index");
            assert_eq!(id, &Some("1".to_string()));
        }
        _ => panic!("Expected Index action"),
    }
}

#[test]
fn test_parse_bulk_auto_generated_id() {
    let bulk_body = r#"{"index":{"_index":"test_index"}}
{"title":"Auto ID Doc","body":"Content"}
"#;

    let actions = parse_bulk_ndjson(bulk_body, None).unwrap();
    assert_eq!(actions.len(), 1);

    match &actions[0] {
        BulkAction::Index {
            index,
            id,
            document,
        } => {
            assert_eq!(index, "test_index");
            assert_eq!(id, &None); // No ID specified, should be None
            assert_eq!(document["title"], "Auto ID Doc");
        }
        _ => panic!("Expected Index action"),
    }
}

#[test]
fn test_parse_bulk_update_without_doc_wrapper() {
    let bulk_body = r#"{"update":{"_index":"test_index","_id":"1"}}
{"title":"Direct Update","body":"Content"}
"#;

    let actions = parse_bulk_ndjson(bulk_body, None).unwrap();
    assert_eq!(actions.len(), 1);

    match &actions[0] {
        BulkAction::Update {
            index,
            id,
            document,
        } => {
            assert_eq!(index, "test_index");
            assert_eq!(id, "1");
            // Should use the whole document if "doc" wrapper is not present
            assert_eq!(document["title"], "Direct Update");
        }
        _ => panic!("Expected Update action"),
    }
}

#[test]
fn test_parse_bulk_missing_document() {
    let bulk_body = r#"{"index":{"_index":"test_index","_id":"1"}}
"#;

    let result = parse_bulk_ndjson(bulk_body, None);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Missing document"));
}

#[test]
fn test_parse_bulk_invalid_json() {
    let bulk_body = r#"{"index":{"_index":"test_index","_id":"1"}
invalid json
"#;

    let result = parse_bulk_ndjson(bulk_body, None);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Invalid JSON"));
}

#[test]
fn test_parse_bulk_missing_index() {
    let bulk_body = r#"{"index":{"_id":"1"}}
{"title":"Doc"}
"#;

    let result = parse_bulk_ndjson(bulk_body, None);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Missing _index"));
}

#[test]
fn test_parse_bulk_missing_id_for_update() {
    let bulk_body = r#"{"update":{"_index":"test_index"}}
{"doc":{"title":"Updated"}}
"#;

    let result = parse_bulk_ndjson(bulk_body, None);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Missing _id"));
}

#[test]
fn test_parse_bulk_missing_id_for_delete() {
    let bulk_body = r#"{"delete":{"_index":"test_index"}}
"#;

    let result = parse_bulk_ndjson(bulk_body, None);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Missing _id"));
}

#[test]
fn test_parse_bulk_unknown_action() {
    let bulk_body = r#"{"unknown":{"_index":"test_index","_id":"1"}}
{"title":"Doc"}
"#;

    let result = parse_bulk_ndjson(bulk_body, None);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Unknown bulk action"));
}

#[test]
fn test_parse_bulk_empty_body() {
    let bulk_body = "";
    let actions = parse_bulk_ndjson(bulk_body, None).unwrap();
    assert_eq!(actions.len(), 0);
}

#[test]
fn test_parse_bulk_with_whitespace() {
    let bulk_body = r#"
{"index":{"_index":"test_index","_id":"1"}}

{"title":"Doc 1"}


{"index":{"_index":"test_index","_id":"2"}}
{"title":"Doc 2"}

"#;

    let actions = parse_bulk_ndjson(bulk_body, None).unwrap();
    assert_eq!(actions.len(), 2); // Empty lines should be filtered out
}

#[test]
fn test_parse_bulk_large_batch() {
    let mut bulk_body = String::new();
    for i in 1..=10 {
        bulk_body.push_str(&format!(
            r#"{{"index":{{"_index":"test_index","_id":"{}"}}}}
{{"title":"Doc {}","body":"Content {}"}}
"#,
            i, i, i
        ));
    }

    let actions = parse_bulk_ndjson(&bulk_body, None).unwrap();
    assert_eq!(actions.len(), 10);

    for (i, action) in actions.iter().enumerate() {
        match action {
            BulkAction::Index {
                index,
                id,
                document,
            } => {
                assert_eq!(index, "test_index");
                assert_eq!(id, &Some(format!("{}", i + 1)));
                assert_eq!(document["title"], format!("Doc {}", i + 1));
            }
            _ => panic!("Expected Index action"),
        }
    }
}

#[test]
fn test_parse_bulk_create_missing_document() {
    let bulk_body = r#"{"create":{"_index":"test_index","_id":"1"}}
"#;

    let result = parse_bulk_ndjson(bulk_body, None);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Missing document"));
}

#[test]
fn test_parse_bulk_update_missing_document() {
    let bulk_body = r#"{"update":{"_index":"test_index","_id":"1"}}
"#;

    let result = parse_bulk_ndjson(bulk_body, None);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Missing document"));
}

#[test]
fn test_parse_bulk_index_with_default_index() {
    let bulk_body = r#"{"index":{"_id":"1"}}
{"title":"Doc 1"}
"#;

    let actions = parse_bulk_ndjson(bulk_body, Some("default_index")).unwrap();
    assert_eq!(actions.len(), 1);

    match &actions[0] {
        BulkAction::Index { index, .. } => {
            assert_eq!(index, "default_index");
        }
        _ => panic!("Expected Index action"),
    }
}

#[test]
fn test_parse_bulk_create_with_default_index() {
    let bulk_body = r#"{"create":{"_id":"1"}}
{"title":"Doc 1"}
"#;

    let actions = parse_bulk_ndjson(bulk_body, Some("default_index")).unwrap();
    assert_eq!(actions.len(), 1);

    match &actions[0] {
        BulkAction::Create { index, .. } => {
            assert_eq!(index, "default_index");
        }
        _ => panic!("Expected Create action"),
    }
}

#[test]
fn test_parse_bulk_update_with_default_index() {
    let bulk_body = r#"{"update":{"_id":"1"}}
{"doc":{"title":"Updated"}}
"#;

    let actions = parse_bulk_ndjson(bulk_body, Some("default_index")).unwrap();
    assert_eq!(actions.len(), 1);

    match &actions[0] {
        BulkAction::Update { index, .. } => {
            assert_eq!(index, "default_index");
        }
        _ => panic!("Expected Update action"),
    }
}

#[test]
fn test_parse_bulk_delete_with_default_index() {
    let bulk_body = r#"{"delete":{"_id":"1"}}
"#;

    let actions = parse_bulk_ndjson(bulk_body, Some("default_index")).unwrap();
    assert_eq!(actions.len(), 1);

    match &actions[0] {
        BulkAction::Delete { index, .. } => {
            assert_eq!(index, "default_index");
        }
        _ => panic!("Expected Delete action"),
    }
}

#[test]
fn test_parse_bulk_invalid_action_json() {
    let bulk_body = r#"{"index":invalid json}
{"title":"Doc"}
"#;

    let result = parse_bulk_ndjson(bulk_body, None);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Invalid JSON"));
}

#[test]
fn test_parse_bulk_invalid_document_json() {
    let bulk_body = r#"{"index":{"_index":"test_index","_id":"1"}}
invalid json document
"#;

    let result = parse_bulk_ndjson(bulk_body, None);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Invalid document JSON"));
}

#[test]
fn test_parse_bulk_update_with_nested_doc() {
    let bulk_body = r#"{"update":{"_index":"test_index","_id":"1"}}
{"doc":{"nested":{"field":"value"}}}
"#;

    let actions = parse_bulk_ndjson(bulk_body, None).unwrap();
    assert_eq!(actions.len(), 1);

    match &actions[0] {
        BulkAction::Update { document, .. } => {
            assert_eq!(document["nested"]["field"], "value");
        }
        _ => panic!("Expected Update action"),
    }
}

#[test]
fn test_parse_bulk_update_without_doc_field() {
    let bulk_body = r#"{"update":{"_index":"test_index","_id":"1"}}
{"field":"direct_value"}
"#;

    let actions = parse_bulk_ndjson(bulk_body, None).unwrap();
    assert_eq!(actions.len(), 1);

    match &actions[0] {
        BulkAction::Update { document, .. } => {
            // Should use the whole document when "doc" wrapper is not present
            assert_eq!(document["field"], "direct_value");
        }
        _ => panic!("Expected Update action"),
    }
}

#[test]
fn test_parse_bulk_complex_document() {
    let bulk_body = r#"{"index":{"_index":"test_index","_id":"1"}}
{"title":"Doc","tags":["tag1","tag2"],"metadata":{"author":"John","year":2024},"count":42}
"#;

    let actions = parse_bulk_ndjson(bulk_body, None).unwrap();
    assert_eq!(actions.len(), 1);

    match &actions[0] {
        BulkAction::Index { document, .. } => {
            assert_eq!(document["title"], "Doc");
            assert_eq!(document["tags"][0], "tag1");
            assert_eq!(document["metadata"]["author"], "John");
            assert_eq!(document["count"], 42);
        }
        _ => panic!("Expected Index action"),
    }
}

#[test]
fn test_parse_bulk_multiple_indices() {
    let bulk_body = r#"{"index":{"_index":"index1","_id":"1"}}
{"title":"Doc 1"}
{"index":{"_index":"index2","_id":"2"}}
{"title":"Doc 2"}
"#;

    let actions = parse_bulk_ndjson(bulk_body, None).unwrap();
    assert_eq!(actions.len(), 2);

    match &actions[0] {
        BulkAction::Index { index, .. } => assert_eq!(index, "index1"),
        _ => panic!("Expected Index action"),
    }

    match &actions[1] {
        BulkAction::Index { index, .. } => assert_eq!(index, "index2"),
        _ => panic!("Expected Index action"),
    }
}

#[test]
fn test_parse_bulk_only_whitespace() {
    let bulk_body = "   \n  \t  \n  ";
    let actions = parse_bulk_ndjson(bulk_body, None).unwrap();
    assert_eq!(actions.len(), 0);
}

#[test]
fn test_parse_bulk_trailing_newlines() {
    let bulk_body = r#"{"index":{"_index":"test_index","_id":"1"}}
{"title":"Doc"}


"#;

    let actions = parse_bulk_ndjson(bulk_body, None).unwrap();
    assert_eq!(actions.len(), 1);
}

#[test]
fn test_parse_bulk_mixed_with_default_index() {
    let bulk_body = r#"{"index":{"_id":"1"}}
{"title":"Indexed"}
{"create":{"_id":"2"}}
{"title":"Created"}
{"update":{"_id":"1"}}
{"doc":{"title":"Updated"}}
{"delete":{"_id":"2"}}
"#;

    let actions = parse_bulk_ndjson(bulk_body, Some("default_index")).unwrap();
    assert_eq!(actions.len(), 4);

    // All should use default_index
    for action in &actions {
        match action {
            BulkAction::Index { index, .. } => assert_eq!(index, "default_index"),
            BulkAction::Create { index, .. } => assert_eq!(index, "default_index"),
            BulkAction::Update { index, .. } => assert_eq!(index, "default_index"),
            BulkAction::Delete { index, .. } => assert_eq!(index, "default_index"),
        }
    }
}

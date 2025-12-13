//! Utility functions for search operations

/// Get a field value from a document (supports nested fields with dot notation)
pub fn get_field_value<'a>(doc: &'a serde_json::Value, field: &str) -> Option<&'a serde_json::Value> {
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

/// Filter _source field based on _source specification
///
/// Supports:
/// - true: include all fields (default)
/// - false: exclude all fields (return empty object)
/// - ["field1", "field2"]: include only specified fields
/// - {"includes": ["field1"], "excludes": ["field2"]}: include/exclude pattern
pub fn filter_source(doc: &serde_json::Value, source_filter: Option<&serde_json::Value>) -> serde_json::Value {
    let Some(filter) = source_filter else {
        return doc.clone();
    };

    // Handle boolean values
    if let Some(include) = filter.as_bool() {
        if include {
            return doc.clone();
        } else {
            return serde_json::json!({});
        }
    }

    // Handle array of field names
    if let Some(fields) = filter.as_array() {
        let mut result = serde_json::Map::new();
        if let Some(doc_obj) = doc.as_object() {
            for field in fields {
                if let Some(field_name) = field.as_str() {
                    if let Some(value) = doc_obj.get(field_name) {
                        result.insert(field_name.to_string(), value.clone());
                    }
                }
            }
        }
        return serde_json::Value::Object(result);
    }

    // Handle object with includes/excludes
    if let Some(filter_obj) = filter.as_object() {
        let mut result = doc.clone();

        // Apply excludes first
        if let Some(excludes) = filter_obj.get("excludes") {
            if let Some(exclude_array) = excludes.as_array() {
                if let Some(result_obj) = result.as_object_mut() {
                    for exclude_field in exclude_array {
                        if let Some(field_name) = exclude_field.as_str() {
                            result_obj.remove(field_name);
                        }
                    }
                }
            }
        }

        // Apply includes (if specified, only include those fields)
        if let Some(includes) = filter_obj.get("includes") {
            if let Some(include_array) = includes.as_array() {
                let mut filtered = serde_json::Map::new();
                if let Some(result_obj) = result.as_object() {
                    for include_field in include_array {
                        if let Some(field_name) = include_field.as_str() {
                            if let Some(value) = result_obj.get(field_name) {
                                filtered.insert(field_name.to_string(), value.clone());
                            }
                        }
                    }
                }
                result = serde_json::Value::Object(filtered);
            }
        }

        return result;
    }

    // Default: return full document
    doc.clone()
}

/// Compare two documents for sorting
pub fn compare_documents(
    a: &serde_json::Value,
    b: &serde_json::Value,
    sort_spec: &serde_json::Value,
) -> std::cmp::Ordering {
    if let Some(sort_obj) = sort_spec.as_object() {
        for (field, order_spec) in sort_obj {
            let order = if let Some(order_obj) = order_spec.as_object() {
                order_obj.get("order")
                    .and_then(|o| o.as_str())
                    .unwrap_or("asc")
            } else {
                order_spec.as_str().unwrap_or("asc")
            };

            let a_val = get_field_value(a, field);
            let b_val = get_field_value(b, field);

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

            match order {
                "desc" => return cmp.reverse(),
                _ => return cmp,
            }
        }
    }

    std::cmp::Ordering::Equal
}

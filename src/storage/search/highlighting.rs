//! Highlighting functionality for search results

use super::utils::get_field_value;

/// Highlight matched terms in a document based on query and highlight configuration
///
/// Returns a JSON object with highlighted fields, e.g.:
/// {
///   "title": ["This is a <em>search</em> result"],
///   "body": ["Some <em>search</em> content here"]
/// }
pub fn highlight_document(
    doc: &serde_json::Value,
    query: &serde_json::Value,
    highlight_config: &serde_json::Value,
) -> Option<serde_json::Value> {
    // Extract fields to highlight
    let fields_to_highlight = if let Some(config_obj) = highlight_config.as_object() {
        if let Some(fields) = config_obj.get("fields") {
            if let Some(fields_obj) = fields.as_object() {
                fields_obj.keys().collect::<Vec<_>>()
            } else {
                return None;
            }
        } else {
            return None;
        }
    } else {
        return None;
    };

    // Extract highlight tags (default: <em></em>)
    let pre_tag = highlight_config
        .as_object()
        .and_then(|c| c.get("pre_tags"))
        .and_then(|t| t.as_array())
        .and_then(|a| a.get(0))
        .and_then(|v| v.as_str())
        .unwrap_or("<em>");

    let post_tag = highlight_config
        .as_object()
        .and_then(|c| c.get("post_tags"))
        .and_then(|t| t.as_array())
        .and_then(|a| a.get(0))
        .and_then(|v| v.as_str())
        .unwrap_or("</em>");

    // Extract query terms from the query
    let query_terms = extract_query_terms(query);
    if query_terms.is_empty() {
        return None;
    }

    // Build highlight result
    let mut highlight_result = serde_json::Map::new();

    for field in fields_to_highlight {
        if let Some(field_value) = get_field_value(doc, field) {
            if let Some(field_str) = field_value.as_str() {
                let highlighted = highlight_text(field_str, &query_terms, pre_tag, post_tag);
                if !highlighted.is_empty() {
                    highlight_result.insert(field.to_string(), serde_json::json!([highlighted]));
                }
            }
        }
    }

    if highlight_result.is_empty() {
        None
    } else {
        Some(serde_json::Value::Object(highlight_result))
    }
}

/// Extract search terms from a query
pub fn extract_query_terms(query: &serde_json::Value) -> Vec<String> {
    let mut terms = Vec::new();

    if let Some(query_obj) = query.as_object() {
        // Handle match query
        if let Some(match_query) = query_obj.get("match") {
            if let Some(match_obj) = match_query.as_object() {
                for (_, query_value) in match_obj {
                    if let Some(q) = query_value.as_object() {
                        if let Some(query_text) = q.get("query").and_then(|v| v.as_str()) {
                            terms.extend(tokenize_query(query_text));
                        }
                    } else if let Some(query_text) = query_value.as_str() {
                        terms.extend(tokenize_query(query_text));
                    }
                }
            }
        }

        // Handle match_phrase query
        if let Some(match_phrase_query) = query_obj.get("match_phrase") {
            if let Some(match_phrase_obj) = match_phrase_query.as_object() {
                for (_, query_value) in match_phrase_obj {
                    if let Some(q) = query_value.as_object() {
                        if let Some(query_text) = q.get("query").and_then(|v| v.as_str()) {
                            terms.extend(tokenize_query(query_text));
                        }
                    } else if let Some(query_text) = query_value.as_str() {
                        terms.extend(tokenize_query(query_text));
                    }
                }
            }
        }

        // Handle multi_match query
        if let Some(multi_match_query) = query_obj.get("multi_match") {
            if let Some(multi_match_obj) = multi_match_query.as_object() {
                if let Some(query_text) = multi_match_obj.get("query").and_then(|v| v.as_str()) {
                    terms.extend(tokenize_query(query_text));
                }
            }
        }

        // Handle term query
        if let Some(term_query) = query_obj.get("term") {
            if let Some(term_obj) = term_query.as_object() {
                for (_, value) in term_obj {
                    if let Some(term_str) = value.as_str() {
                        terms.push(term_str.to_lowercase());
                    }
                }
            }
        }

        // Handle bool query - recursively extract from nested queries
        if let Some(bool_query) = query_obj.get("bool") {
            if let Some(bool_obj) = bool_query.as_object() {
                for clause_type in ["must", "should", "must_not", "filter"] {
                    if let Some(clauses) = bool_obj.get(clause_type) {
                        if let Some(clauses_array) = clauses.as_array() {
                            for clause in clauses_array {
                                terms.extend(extract_query_terms(clause));
                            }
                        }
                    }
                }
            }
        }
    }

    terms
}

/// Tokenize a query string into terms (simple whitespace splitting)
pub fn tokenize_query(query: &str) -> Vec<String> {
    query
        .split_whitespace()
        .map(|s| s.to_lowercase())
        .collect()
}

/// Highlight text by wrapping matched terms with tags
pub fn highlight_text(text: &str, terms: &[String], pre_tag: &str, post_tag: &str) -> String {
    if terms.is_empty() {
        return text.to_string();
    }

    let text_lower = text.to_lowercase();
    let mut result = String::new();
    let mut last_end = 0;

    // Find all matches and their positions
    let mut matches: Vec<(usize, usize, &str)> = Vec::new();
    for term in terms {
        let term_lower = term.to_lowercase();
        let mut start = 0;
        while let Some(pos) = text_lower[start..].find(&term_lower) {
            let actual_pos = start + pos;
            let end = actual_pos + term.len();
            matches.push((actual_pos, end, &text[actual_pos..end]));
            start = end;
        }
    }

    // Sort matches by position
    matches.sort_by_key(|(start, _, _)| *start);

    // Remove overlapping matches (keep the first one)
    let mut non_overlapping: Vec<(usize, usize, &str)> = Vec::new();
    for (start, end, matched_text) in matches {
        if non_overlapping.is_empty() || start >= non_overlapping.last().unwrap().1 {
            non_overlapping.push((start, end, matched_text));
        }
    }

    // Build highlighted string
    for (start, end, matched_text) in non_overlapping {
        // Add text before match
        if start > last_end {
            result.push_str(&text[last_end..start]);
        }
        // Add highlighted match
        result.push_str(pre_tag);
        result.push_str(matched_text);
        result.push_str(post_tag);
        last_end = end;
    }

    // Add remaining text
    if last_end < text.len() {
        result.push_str(&text[last_end..]);
    }

    result
}

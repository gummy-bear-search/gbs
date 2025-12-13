//! Query parsing and scoring

use crate::error::Result;
use super::matchers::*;

/// Score a document against a query
pub fn score_document(doc: &serde_json::Value, query: &serde_json::Value) -> Result<f64> {
    if let Some(query_obj) = query.as_object() {
        // Handle match_all query (no query or empty query)
        if query_obj.is_empty() {
            return Ok(1.0);
        }

        // Handle match query: { "match": { "field": "query text" } }
        if let Some(match_query) = query_obj.get("match") {
            if let Some(match_obj) = match_query.as_object() {
                for (field, query_value) in match_obj {
                    let query_text = if let Some(q) = query_value.as_object() {
                        q.get("query").and_then(|v| v.as_str()).unwrap_or("")
                    } else {
                        query_value.as_str().unwrap_or("")
                    };

                    if let Some(score) = match_field(doc, field, query_text) {
                        return Ok(score);
                    }
                }
            }
        }

        // Handle match_phrase query: { "match_phrase": { "field": "exact phrase" } }
        if let Some(match_phrase_query) = query_obj.get("match_phrase") {
            if let Some(match_phrase_obj) = match_phrase_query.as_object() {
                for (field, query_value) in match_phrase_obj {
                    let query_text = if let Some(q) = query_value.as_object() {
                        q.get("query").and_then(|v| v.as_str()).unwrap_or("")
                    } else {
                        query_value.as_str().unwrap_or("")
                    };

                    if let Some(score) = match_phrase_field(doc, field, query_text) {
                        return Ok(score);
                    }
                }
            }
        }

        // Handle multi_match query: { "multi_match": { "query": "text", "fields": ["field1", "field2"] } }
        if let Some(multi_match_query) = query_obj.get("multi_match") {
            if let Some(multi_match_obj) = multi_match_query.as_object() {
                let query_text = multi_match_obj.get("query")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                let fields = if let Some(fields_val) = multi_match_obj.get("fields") {
                    if let Some(fields_array) = fields_val.as_array() {
                        fields_array.iter()
                            .filter_map(|f| f.as_str())
                            .collect::<Vec<_>>()
                    } else if let Some(field_str) = fields_val.as_str() {
                        vec![field_str]
                    } else {
                        vec!["_all"]
                    }
                } else {
                    vec!["_all"]
                };

                if let Some(score) = multi_match_fields(doc, &fields, query_text) {
                    return Ok(score);
                }
            }
        }

        // Handle range query: { "range": { "field": { "gte": 10, "lte": 20 } } }
        if let Some(range_query) = query_obj.get("range") {
            if let Some(range_obj) = range_query.as_object() {
                for (field, range_spec) in range_obj {
                    if let Some(range_params) = range_spec.as_object() {
                        if range_match(doc, field, range_params) {
                            return Ok(1.0);
                        }
                    }
                }
            }
        }

        // Handle term query: { "term": { "field": "value" } }
        if let Some(term_query) = query_obj.get("term") {
            if let Some(term_obj) = term_query.as_object() {
                for (field, value) in term_obj {
                    if term_match(doc, field, value) {
                        return Ok(1.0);
                    }
                }
            }
        }

        // Handle terms query: { "terms": { "field": ["value1", "value2"] } }
        if let Some(terms_query) = query_obj.get("terms") {
            if let Some(terms_obj) = terms_query.as_object() {
                for (field, values) in terms_obj {
                    if let Some(values_array) = values.as_array() {
                        if terms_match(doc, field, values_array) {
                            return Ok(1.0);
                        }
                    }
                }
            }
        }

        // Handle prefix query: { "prefix": { "field": "prefix" } }
        if let Some(prefix_query) = query_obj.get("prefix") {
            if let Some(prefix_obj) = prefix_query.as_object() {
                for (field, prefix_value) in prefix_obj {
                    let prefix_str = if let Some(p) = prefix_value.as_object() {
                        p.get("value").and_then(|v| v.as_str()).unwrap_or("")
                    } else {
                        prefix_value.as_str().unwrap_or("")
                    };
                    if prefix_match(doc, field, prefix_str) {
                        return Ok(1.0);
                    }
                }
            }
        }

        // Handle wildcard query: { "wildcard": { "field": "pat*ern" } }
        if let Some(wildcard_query) = query_obj.get("wildcard") {
            if let Some(wildcard_obj) = wildcard_query.as_object() {
                for (field, pattern_value) in wildcard_obj {
                    let pattern_str = if let Some(p) = pattern_value.as_object() {
                        p.get("value").and_then(|v| v.as_str()).unwrap_or("")
                    } else {
                        pattern_value.as_str().unwrap_or("")
                    };
                    if wildcard_match(doc, field, pattern_str) {
                        return Ok(1.0);
                    }
                }
            }
        }

        // Handle bool query
        if let Some(bool_query) = query_obj.get("bool") {
            return score_bool_query(doc, bool_query);
        }

        // Handle match_all query: { "match_all": {} }
        if query_obj.contains_key("match_all") {
            return Ok(1.0);
        }
    }

    // Default: no match
    Ok(0.0)
}

/// Score a bool query
pub fn score_bool_query(doc: &serde_json::Value, bool_query: &serde_json::Value) -> Result<f64> {
    if let Some(bool_obj) = bool_query.as_object() {
        let mut score = 0.0;
        let mut must_match = true;

        // Handle must clauses (all must match)
        if let Some(must) = bool_obj.get("must") {
            if let Some(must_array) = must.as_array() {
                for clause in must_array {
                    let clause_score = score_document(doc, clause)?;
                    if clause_score == 0.0 {
                        must_match = false;
                        break;
                    }
                    score += clause_score;
                }
            }
        }

        if !must_match {
            return Ok(0.0);
        }

        // Handle should clauses (at least one should match, or boost score)
        if let Some(should) = bool_obj.get("should") {
            if let Some(should_array) = should.as_array() {
                let mut should_score = 0.0;
                for clause in should_array {
                    should_score += score_document(doc, clause)?;
                }
                if should_score > 0.0 {
                    score += should_score * 0.5; // Boost for should matches
                }
            }
        }

        // Handle must_not clauses (none should match)
        if let Some(must_not) = bool_obj.get("must_not") {
            if let Some(must_not_array) = must_not.as_array() {
                for clause in must_not_array {
                    let clause_score = score_document(doc, clause)?;
                    if clause_score > 0.0 {
                        return Ok(0.0); // Document matches must_not, exclude it
                    }
                }
            }
        }

        // Handle filter clauses (must match, but don't affect score)
        if let Some(filter) = bool_obj.get("filter") {
            if let Some(filter_array) = filter.as_array() {
                for clause in filter_array {
                    let clause_score = score_document(doc, clause)?;
                    if clause_score == 0.0 {
                        return Ok(0.0); // Filter doesn't match, exclude
                    }
                }
            }
        }

        Ok(if score > 0.0 { score } else { 1.0 }) // At least 1.0 if all filters pass
    } else {
        Ok(0.0)
    }
}

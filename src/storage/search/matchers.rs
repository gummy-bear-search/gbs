//! Query matchers for different query types

use regex::Regex;
use super::utils::get_field_value;

/// Match a field against query text (case-insensitive substring match)
pub fn match_field(doc: &serde_json::Value, field: &str, query_text: &str) -> Option<f64> {
    if query_text.is_empty() {
        return Some(1.0);
    }

    // Handle _all field - search in all fields
    if field == "_all" || field == "*" {
        return match_all_fields(doc, query_text);
    }

    let field_value = get_field_value(doc, field)?;
    let field_str = match field_value {
        serde_json::Value::String(s) => s.to_lowercase(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        _ => return None,
    };

    let query_lower = query_text.to_lowercase();

    // Simple scoring: 1.0 if exact match, 0.8 if contains, 0.5 if word match
    if field_str == query_lower {
        Some(1.0)
    } else if field_str.contains(&query_lower) {
        Some(0.8)
    } else {
        // Check for word matches
        let words: Vec<&str> = query_lower.split_whitespace().collect();
        let field_words: Vec<&str> = field_str.split_whitespace().collect();
        let matches = words.iter()
            .filter(|w| field_words.iter().any(|fw| fw.contains(*w)))
            .count();
        if matches > 0 {
            Some(0.5 * (matches as f64 / words.len() as f64))
        } else {
            None
        }
    }
}

/// Match query text against all fields in a document
pub fn match_all_fields(doc: &serde_json::Value, query_text: &str) -> Option<f64> {
    if query_text.is_empty() {
        return Some(1.0);
    }

    let query_lower = query_text.to_lowercase();
    let mut max_score = 0.0;

    // Recursively search all string/number values in the document
    search_value(doc, &query_lower, &mut max_score);

    if max_score > 0.0 {
        Some(max_score)
    } else {
        None
    }
}

/// Recursively search a JSON value for the query text
fn search_value(value: &serde_json::Value, query: &str, max_score: &mut f64) {
    match value {
        serde_json::Value::String(s) => {
            let s_lower = s.to_lowercase();
            let score = if s_lower == *query {
                1.0
            } else if s_lower.contains(query) {
                0.8
            } else {
                // Check for word matches
                let words: Vec<&str> = query.split_whitespace().collect();
                let field_words: Vec<&str> = s_lower.split_whitespace().collect();
                let matches = words.iter()
                    .filter(|w| field_words.iter().any(|fw| fw.contains(*w)))
                    .count();
                if matches > 0 {
                    0.5 * (matches as f64 / words.len() as f64)
                } else {
                    0.0
                }
            };
            *max_score = max_score.max(score);
        }
        serde_json::Value::Number(n) => {
            let n_str = n.to_string();
            if n_str.contains(query) {
                *max_score = max_score.max(0.5);
            }
        }
        serde_json::Value::Object(map) => {
            for v in map.values() {
                search_value(v, query, max_score);
            }
        }
        serde_json::Value::Array(arr) => {
            for v in arr {
                search_value(v, query, max_score);
            }
        }
        _ => {}
    }
}

/// Check if a field matches a term exactly
pub fn term_match(doc: &serde_json::Value, field: &str, value: &serde_json::Value) -> bool {
    if let Some(field_value) = get_field_value(doc, field) {
        *field_value == *value
    } else {
        false
    }
}

/// Match a field against an exact phrase (words must appear in order)
pub fn match_phrase_field(doc: &serde_json::Value, field: &str, phrase: &str) -> Option<f64> {
    if phrase.is_empty() {
        return Some(1.0);
    }

    // Handle _all field - search in all fields
    if field == "_all" || field == "*" {
        return match_phrase_all_fields(doc, phrase);
    }

    let field_value = get_field_value(doc, field)?;
    let field_str = match field_value {
        serde_json::Value::String(s) => s.to_lowercase(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        _ => return None,
    };

    let phrase_lower = phrase.to_lowercase();

    // Exact phrase match gets highest score
    if field_str.contains(&phrase_lower) {
        // Check if it's an exact phrase (words in order)
        let phrase_words: Vec<&str> = phrase_lower.split_whitespace().collect();
        if phrase_words.len() == 1 {
            // Single word - same as match
            Some(1.0)
        } else {
            // Multi-word phrase - check if words appear in order
            let field_words: Vec<&str> = field_str.split_whitespace().collect();
            if words_in_order(&field_words, &phrase_words) {
                Some(1.0)
            } else {
                // Phrase words exist but not in order - lower score
                Some(0.6)
            }
        }
    } else {
        None
    }
}

/// Check if phrase words appear in order in field words
fn words_in_order(field_words: &[&str], phrase_words: &[&str]) -> bool {
    if phrase_words.is_empty() {
        return true;
    }

    let mut phrase_idx = 0;
    for field_word in field_words {
        if phrase_idx < phrase_words.len() && field_word.contains(phrase_words[phrase_idx]) {
            phrase_idx += 1;
            if phrase_idx == phrase_words.len() {
                return true;
            }
        }
    }
    false
}

/// Match phrase against all fields in a document
pub fn match_phrase_all_fields(doc: &serde_json::Value, phrase: &str) -> Option<f64> {
    if phrase.is_empty() {
        return Some(1.0);
    }

    let phrase_lower = phrase.to_lowercase();
    let mut max_score = 0.0;

    // Recursively search all string values in the document
    search_phrase_value(doc, &phrase_lower, &mut max_score);

    if max_score > 0.0 {
        Some(max_score)
    } else {
        None
    }
}

/// Recursively search a JSON value for the exact phrase
fn search_phrase_value(value: &serde_json::Value, phrase: &str, max_score: &mut f64) {
    match value {
        serde_json::Value::String(s) => {
            let s_lower = s.to_lowercase();
            if s_lower.contains(phrase) {
                let phrase_words: Vec<&str> = phrase.split_whitespace().collect();
                let field_words: Vec<&str> = s_lower.split_whitespace().collect();
                if phrase_words.len() == 1 || words_in_order(&field_words, &phrase_words) {
                    *max_score = max_score.max(1.0);
                } else {
                    *max_score = max_score.max(0.6);
                }
            }
        }
        serde_json::Value::Object(map) => {
            for v in map.values() {
                search_phrase_value(v, phrase, max_score);
            }
        }
        serde_json::Value::Array(arr) => {
            for v in arr {
                search_phrase_value(v, phrase, max_score);
            }
        }
        _ => {}
    }
}

/// Match query text against multiple fields (returns highest score)
pub fn multi_match_fields(doc: &serde_json::Value, fields: &[&str], query_text: &str) -> Option<f64> {
    if query_text.is_empty() {
        return Some(1.0);
    }

    let mut max_score: f64 = 0.0;
    for field in fields {
        if let Some(score) = match_field(doc, field, query_text) {
            max_score = max_score.max(score);
        }
    }

    if max_score > 0.0 {
        Some(max_score)
    } else {
        None
    }
}

/// Check if a field value matches a range query
pub fn range_match(
    doc: &serde_json::Value,
    field: &str,
    range_params: &serde_json::Map<String, serde_json::Value>,
) -> bool {
    let field_value = match get_field_value(doc, field) {
        Some(v) => v,
        None => return false,
    };

    // Extract numeric value
    let num_value = match field_value {
        serde_json::Value::Number(n) => n.as_f64(),
        serde_json::Value::String(s) => s.parse::<f64>().ok(),
        _ => return false,
    };

    let num_value = match num_value {
        Some(v) => v,
        None => return false,
    };

    // Check range conditions
    if let Some(gte) = range_params.get("gte") {
        if let Some(gte_val) = gte.as_f64() {
            if num_value < gte_val {
                return false;
            }
        }
    }

    if let Some(gt) = range_params.get("gt") {
        if let Some(gt_val) = gt.as_f64() {
            if num_value <= gt_val {
                return false;
            }
        }
    }

    if let Some(lte) = range_params.get("lte") {
        if let Some(lte_val) = lte.as_f64() {
            if num_value > lte_val {
                return false;
            }
        }
    }

    if let Some(lt) = range_params.get("lt") {
        if let Some(lt_val) = lt.as_f64() {
            if num_value >= lt_val {
                return false;
            }
        }
    }

    true
}

/// Match a field against a wildcard pattern (* matches any sequence, ? matches any single character)
pub fn wildcard_match(doc: &serde_json::Value, field: &str, pattern: &str) -> bool {
    if pattern.is_empty() {
        return true;
    }

    let field_value = match get_field_value(doc, field) {
        Some(v) => v,
        None => return false,
    };

    let field_str = match field_value {
        serde_json::Value::String(s) => s.to_lowercase(), // Case-insensitive matching
        serde_json::Value::Number(_n) => return false, // Wildcard only works on strings
        serde_json::Value::Bool(_b) => return false,
        _ => return false,
    };

    let pattern_lower = pattern.to_lowercase();

    // Convert wildcard pattern to regex
    // * -> .* (matches any sequence)
    // ? -> . (matches any single character)
    // Escape other regex special characters
    let mut regex_pattern = String::new();
    for c in pattern_lower.chars() {
        match c {
            '*' => regex_pattern.push_str(".*"),
            '?' => regex_pattern.push('.'),
            '.' => regex_pattern.push_str(r"\."),
            '+' => regex_pattern.push_str(r"\+"),
            '(' => regex_pattern.push_str(r"\("),
            ')' => regex_pattern.push_str(r"\)"),
            '[' => regex_pattern.push_str(r"\["),
            ']' => regex_pattern.push_str(r"\]"),
            '{' => regex_pattern.push_str(r"\{"),
            '}' => regex_pattern.push_str(r"\}"),
            '^' => regex_pattern.push_str(r"\^"),
            '$' => regex_pattern.push_str(r"\$"),
            '|' => regex_pattern.push_str(r"\|"),
            '\\' => regex_pattern.push_str(r"\\"),
            _ => {
                let mut buf = [0; 4];
                let s = c.encode_utf8(&mut buf);
                regex_pattern.push_str(&regex::escape(s));
            }
        }
    }

    // Anchor the pattern to match the entire string
    let full_pattern = format!("^{}$", regex_pattern);

    match Regex::new(&full_pattern) {
        Ok(re) => re.is_match(&field_str),
        Err(_) => false, // Invalid regex pattern
    }
}

/// Match a field against a prefix (case-insensitive)
pub fn prefix_match(doc: &serde_json::Value, field: &str, prefix: &str) -> bool {
    if prefix.is_empty() {
        return true;
    }

    let field_value = match get_field_value(doc, field) {
        Some(v) => v,
        None => return false,
    };

    let field_str = match field_value {
        serde_json::Value::String(s) => s.to_lowercase(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        _ => return false,
    };

    let prefix_lower = prefix.to_lowercase();
    field_str.starts_with(&prefix_lower)
}

/// Match a field against any of multiple values
pub fn terms_match(doc: &serde_json::Value, field: &str, values: &[serde_json::Value]) -> bool {
    if values.is_empty() {
        return true;
    }

    let field_value = match get_field_value(doc, field) {
        Some(v) => v,
        None => return false,
    };

    // Check if field value matches any of the provided values
    for value in values {
        if *field_value == *value {
            return true;
        }
    }

    false
}

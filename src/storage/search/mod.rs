//! Search functionality for Gummy Search
//!
//! This module contains all search-related logic including query parsing,
//! document scoring, highlighting, and source filtering.

mod query;
mod matchers;
mod highlighting;
mod utils;

pub use query::{score_document, score_bool_query};
pub use matchers::{
    match_field, match_all_fields, term_match, match_phrase_field,
    match_phrase_all_fields, multi_match_fields, range_match,
    wildcard_match, prefix_match, terms_match,
};
pub use highlighting::{highlight_document, extract_query_terms, tokenize_query, highlight_text};
pub use utils::{get_field_value, filter_source, compare_documents};

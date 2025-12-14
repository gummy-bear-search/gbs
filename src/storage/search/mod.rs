//! Search functionality for Gummy Bear Search
//!
//! This module contains all search-related logic including query parsing,
//! document scoring, highlighting, and source filtering.

mod highlighting;
mod matchers;
mod query;
mod utils;

// Only export functions that are used outside this module
pub use highlighting::highlight_document;
pub use query::score_document;
pub use utils::{compare_documents, filter_source};

//! Search functionality for Gummy Search
//!
//! This module contains all search-related logic including query parsing,
//! document scoring, highlighting, and source filtering.

mod query;
mod matchers;
mod highlighting;
mod utils;

// Only export functions that are used outside this module
pub use query::score_document;
pub use highlighting::highlight_document;
pub use utils::{filter_source, compare_documents};

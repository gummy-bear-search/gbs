//! Web interface handlers

use crate::error::{GbsError, Result};
use axum::response::Html;
use std::fs;

/// Handler for the root endpoint
pub async fn root() -> &'static str {
    "Gummy Bear Search - Elasticsearch-compatible search engine"
}

/// Handler for the web dashboard index page
pub async fn web_index() -> Result<Html<String>> {
    // Read the index.html file
    let html_content = fs::read_to_string("static/index.html")
        .map_err(|e| GbsError::Storage(format!("Failed to read index.html: {}", e)))?;

    Ok(Html(html_content))
}

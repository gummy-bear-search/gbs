//! HTTP server module for Gummy Search

mod handlers;
mod routes;

pub use routes::create_router;
pub use handlers::*;

// Re-export create_router as create_app for backward compatibility
pub use routes::create_router as create_app;

use std::sync::Arc;
use crate::storage::Storage;

#[derive(Clone)]
pub struct AppState {
    pub storage: Arc<Storage>,
}

// Utility handlers that don't fit in other categories
pub async fn root() -> &'static str {
    "Gummy Search - Elasticsearch-compatible search engine"
}

pub async fn web_index() -> crate::error::Result<axum::response::Html<String>> {
    use std::fs;
    use crate::error::GummySearchError;

    // Read the index.html file
    let html_content = fs::read_to_string("static/index.html")
        .map_err(|e| GummySearchError::Storage(format!("Failed to read index.html: {}", e)))?;

    Ok(axum::response::Html(html_content))
}

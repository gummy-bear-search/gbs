//! HTTP server module for Gummy Bear Search

mod handlers;
mod routes;

pub use handlers::*;
pub use routes::create_router;

// Re-export create_router as create_app for backward compatibility
pub use routes::create_router as create_app;

use crate::storage::Storage;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub storage: Arc<Storage>,
    pub es_version: String,
}

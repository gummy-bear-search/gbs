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
    pub es_version: String,
}

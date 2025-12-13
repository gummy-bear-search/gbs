pub mod client;
pub mod index;
pub mod document;
pub mod bulk;
pub mod bulk_ops;
pub mod models;
pub mod error;
pub mod server;
pub use server::AppState;
pub mod storage;
pub mod storage_backend;
pub mod config;

pub use error::{GummySearchError, Result};

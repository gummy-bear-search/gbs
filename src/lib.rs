pub mod client;
pub mod index;
pub mod document;
pub mod search;
pub mod bulk;
pub mod bulk_ops;
pub mod models;
pub mod error;
pub mod server;
pub mod storage;
pub mod storage_backend;

pub use error::{GummySearchError, Result};

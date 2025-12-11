pub mod client;
pub mod index;
pub mod document;
pub mod search;
pub mod bulk;
pub mod models;
pub mod error;

pub use client::GummySearchClient;
pub use error::{GummySearchError, Result};

use crate::error::{GummySearchError, Result};
use crate::index::IndexOperations;
use crate::document::DocumentOperations;
use crate::search::SearchOperations;
use crate::bulk::BulkOperations;

pub struct GummySearchClient {
    base_url: String,
    client: reqwest::Client,
}

impl GummySearchClient {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            client: reqwest::Client::new(),
        }
    }

    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }
}

impl IndexOperations for GummySearchClient {}
impl DocumentOperations for GummySearchClient {}
impl SearchOperations for GummySearchClient {}
impl BulkOperations for GummySearchClient {}

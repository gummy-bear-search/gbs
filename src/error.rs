use thiserror::Error;

pub type Result<T> = std::result::Result<T, GummySearchError>;

#[derive(Error, Debug)]
pub enum GummySearchError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Elasticsearch error: {0}")]
    Elasticsearch(String),

    #[error("Index not found: {0}")]
    IndexNotFound(String),

    #[error("Document not found: {0}")]
    DocumentNotFound(String),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),
}

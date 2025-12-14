use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, GbsError>;

#[derive(Error, Debug)]
pub enum GbsError {
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

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Task join error: {0}")]
    TaskJoin(#[from] tokio::task::JoinError),
}

impl IntoResponse for GbsError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            GbsError::IndexNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            GbsError::DocumentNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            GbsError::InvalidRequest(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            GbsError::Elasticsearch(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            GbsError::Json(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            GbsError::Storage(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            GbsError::TaskJoin(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };

        let body = serde_json::json!({
            "error": {
                "type": "error",
                "reason": error_message
            }
        });

        (status, axum::Json(body)).into_response()
    }
}

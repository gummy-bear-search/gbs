//! Web interface routes

use axum::{
    routing::get,
    Router,
};

use crate::server::{handlers::web::{root, web_index}, AppState};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(root))
        .route("/web/", get(web_index))
        .route("/web", get(web_index))
}

//! HTTP handlers for Gummy Bear Search API

pub mod bulk;
pub mod cluster;
pub mod document;
pub mod index;
pub mod search;
pub mod web;
pub mod websocket;

pub use bulk::*;
pub use cluster::*;
pub use document::*;
pub use index::*;
pub use search::*;
pub use web::*;
pub use websocket::*;

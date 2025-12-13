//! HTTP handlers for Gummy Search API

pub mod cluster;
pub mod index;
pub mod document;
pub mod search;
pub mod bulk;
pub mod websocket;

pub use cluster::*;
pub use index::*;
pub use document::*;
pub use search::*;
pub use bulk::*;
pub use websocket::*;

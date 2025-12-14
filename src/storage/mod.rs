//! Storage module for Gummy Search
//!
//! This module provides the main Storage struct and public API for managing
//! indices, documents, and search operations.

// Declare submodules
mod index;
mod search;
mod index_ops;
mod document_ops;
mod stats;
mod persistence;
mod search_impl;
mod storage;

// Re-export Index
pub use index::Index;

// Re-export Storage
pub use storage::Storage;

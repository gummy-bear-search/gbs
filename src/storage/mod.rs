//! Storage module for Gummy Bear Search
//!
//! This module provides the main Storage struct and public API for managing
//! indices, documents, and search operations.

// Declare submodules
mod document_ops;
mod index;
mod index_ops;
mod persistence;
mod search;
mod search_impl;
mod stats;
mod storage;

// Re-export Index
pub use index::Index;

// Re-export Storage
pub use storage::Storage;

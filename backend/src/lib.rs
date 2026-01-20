/// GCP IAM Search Backend API
///
/// High-performance REST API for searching GCP IAM roles and permissions.
/// Uses a hybrid search engine with multiple index types for fast queries.
///
/// # Modules
/// - `models` - Data types and structures
/// - `search` - Search engine implementation
/// - `error` - Error handling

pub mod models;
pub mod search;
pub mod error;

pub use error::{ApiError, Result};
pub use models::{SearchRequest, SearchMode, SearchResult, ApiResponse};
pub use search::SearchEngine;

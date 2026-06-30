//! Internal error types for wellformed operations.

use thiserror::Error;

/// Errors that can occur during wellformed operations.
#[derive(Debug, Error)]
pub enum WelError {
    /// Invalid JSON Pointer syntax.
    #[error("invalid JSON pointer: {0}")]
    InvalidPointer(String),

    /// Path resolution failed.
    #[error("path not found: {0}")]
    PathNotFound(String),

    /// Type mismatch during validation or transform.
    #[error("type mismatch at {path}: expected {expected}, got {actual}")]
    TypeMismatch {
        path: String,
        expected: String,
        actual: String,
    },

    /// Invalid regex pattern.
    #[error("invalid regex pattern: {0}")]
    InvalidRegex(#[from] regex::Error),

    /// Transform failed.
    #[error("transform failed at {path}: {message}")]
    TransformFailed { path: String, message: String },

    /// Unknown named predicate.
    #[error("unknown predicate: {0}")]
    UnknownPredicate(String),

    /// Maximum validation recursion depth exceeded (cyclic or pathologically
    /// nested schema/data). Returned instead of overflowing the stack.
    #[error("maximum validation depth exceeded at {0}")]
    RecursionLimit(String),

    /// JSON serialization/deserialization error.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

/// Result type for wellformed operations.
pub type Result<T> = std::result::Result<T, WelError>;

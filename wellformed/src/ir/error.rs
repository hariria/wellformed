//! Error types for form validation.
//!
//! This module defines the structured error types used in IR constraints
//! and returned by the validation runtime.

use serde::{Deserialize, Serialize};

/// Error severity level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ErrorSeverity {
    /// Hard error - validation fails.
    #[default]
    Error,
    /// Warning - validation passes but with caveats.
    Warning,
}

/// Error metadata defined in the IR.
///
/// This is the error specification attached to constraints in the schema.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErrorMeta {
    /// Stable error code (e.g., "PAYEE_TIN_INVALID").
    pub code: String,

    /// Human-readable message template.
    pub message: String,

    /// Optional path override. If not set, uses the constraint's context path.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,

    /// Error severity.
    #[serde(default)]
    pub severity: ErrorSeverity,

    /// Optional help text with fix suggestions.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub help: Option<String>,

    /// Optional source identifier (e.g., "irs", "company").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

impl ErrorMeta {
    /// Create a new error with the given code and message.
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            path: None,
            severity: ErrorSeverity::Error,
            help: None,
            source: None,
        }
    }

    /// Set the severity to warning.
    pub fn warning(mut self) -> Self {
        self.severity = ErrorSeverity::Warning;
        self
    }

    /// Add help text.
    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }

    /// Add source identifier.
    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }

    /// Override the path.
    pub fn with_path(mut self, path: impl Into<String>) -> Self {
        self.path = Some(path.into());
        self
    }
}

/// A form validation error (runtime output).
///
/// This is the concrete error returned by the validation runtime,
/// with the path fully resolved.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FormError {
    /// Stable error code.
    pub code: String,

    /// Human-readable message.
    pub message: String,

    /// JSON Pointer path to the failing field.
    pub path: String,

    /// Error severity.
    pub severity: ErrorSeverity,

    /// Optional help text.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub help: Option<String>,

    /// Optional source identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

impl FormError {
    /// Create a FormError from ErrorMeta with a resolved path.
    pub fn from_meta(meta: &ErrorMeta, resolved_path: &str) -> Self {
        Self {
            code: meta.code.clone(),
            message: meta.message.clone(),
            path: meta
                .path
                .clone()
                .unwrap_or_else(|| resolved_path.to_string()),
            severity: meta.severity,
            help: meta.help.clone(),
            source: meta.source.clone(),
        }
    }

    /// Create a simple error.
    pub fn new(
        code: impl Into<String>,
        message: impl Into<String>,
        path: impl Into<String>,
    ) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            path: path.into(),
            severity: ErrorSeverity::Error,
            help: None,
            source: None,
        }
    }

    /// Check if this is a hard error (not a warning).
    pub fn is_error(&self) -> bool {
        self.severity == ErrorSeverity::Error
    }

    /// Check if this is a warning.
    pub fn is_warning(&self) -> bool {
        self.severity == ErrorSeverity::Warning
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_meta_builder() {
        let meta = ErrorMeta::new("TEST_ERROR", "Test message")
            .warning()
            .with_help("Try this fix")
            .with_source("test");

        assert_eq!(meta.code, "TEST_ERROR");
        assert_eq!(meta.severity, ErrorSeverity::Warning);
        assert_eq!(meta.help, Some("Try this fix".to_string()));
        assert_eq!(meta.source, Some("test".to_string()));
    }

    #[test]
    fn test_form_error_from_meta() {
        let meta = ErrorMeta::new("FIELD_REQUIRED", "Field is required")
            .with_help("Please provide a value");

        let error = FormError::from_meta(&meta, "/foo/bar");

        assert_eq!(error.code, "FIELD_REQUIRED");
        assert_eq!(error.path, "/foo/bar");
        assert_eq!(error.severity, ErrorSeverity::Error);
    }

    #[test]
    fn test_form_error_path_override() {
        let meta = ErrorMeta::new("FIELD_REQUIRED", "Field is required").with_path("/custom/path");

        let error = FormError::from_meta(&meta, "/foo/bar");

        assert_eq!(error.path, "/custom/path");
    }

    #[test]
    fn test_serde_roundtrip() {
        let error = FormError::new("TEST", "Test", "/path");
        let json = serde_json::to_string(&error).unwrap();
        let parsed: FormError = serde_json::from_str(&json).unwrap();
        assert_eq!(error, parsed);
    }
}

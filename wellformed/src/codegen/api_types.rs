//! Shared API types for generated code.
//!
//! These types are used by all generated API handlers.

/// Generate the shared API types code.
pub fn generate_shared_types() -> String {
    let mut code = String::new();

    code.push_str(
        "// ============================================================================\n",
    );
    code.push_str("// Shared API Types\n");
    code.push_str(
        "// ============================================================================\n\n",
    );

    // PDF render params
    code.push_str(
        r#"/// PDF rendering query parameters.
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct PdfRenderParams {
    /// Page number to render (0-indexed). If omitted, renders all pages with fields.
    pub page: Option<usize>,
    /// Skip validation before rendering (useful for previewing incomplete forms).
    pub skip_validation: bool,
}

"#,
    );

    // Error response
    code.push_str(
        r#"/// API error response following JSON:API format.
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub errors: Vec<ApiErrorDetail>,
}

/// Individual error detail.
#[derive(Debug, Serialize)]
pub struct ApiErrorDetail {
    /// HTTP status code as string.
    pub status: String,
    /// Error type/title.
    pub title: String,
    /// Detailed error message.
    pub detail: String,
    /// JSON pointer to error source (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<ErrorSource>,
}

/// Error source pointer.
#[derive(Debug, Serialize)]
pub struct ErrorSource {
    /// JSON pointer to the field that caused the error.
    pub pointer: String,
}

"#,
    );

    // API error
    code.push_str(
        r#"/// API error type for handlers.
pub struct ApiError {
    pub status: StatusCode,
    pub errors: Vec<ApiErrorDetail>,
}

impl ApiError {
    pub fn bad_request(message: &str) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            errors: vec![ApiErrorDetail {
                status: "400".to_string(),
                title: "Bad Request".to_string(),
                detail: message.to_string(),
                source: None,
            }],
        }
    }

    pub fn not_found(id: &str) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            errors: vec![ApiErrorDetail {
                status: "404".to_string(),
                title: "Not Found".to_string(),
                detail: format!("Resource not found: {}", id),
                source: None,
            }],
        }
    }

    pub fn validation(result: &ValidationResult) -> Self {
        Self::validation_with_field_map(result, &std::collections::HashMap::new())
    }

    pub fn validation_with_field_map(
        result: &ValidationResult,
        field_map: &std::collections::HashMap<&str, &str>,
    ) -> Self {
        let errors = result
            .errors
            .iter()
            .map(|e| {
                // Transform path from schema keys to human-readable names
                let pointer = transform_error_path(&e.path.to_string(), field_map);
                ApiErrorDetail {
                    status: "400".to_string(),
                    title: "Validation Error".to_string(),
                    detail: e.message.clone(),
                    source: Some(ErrorSource { pointer }),
                }
            })
            .collect();

        Self {
            status: StatusCode::BAD_REQUEST,
            errors,
        }
    }

    pub fn internal(message: &str) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            errors: vec![ApiErrorDetail {
                status: "500".to_string(),
                title: "Internal Server Error".to_string(),
                detail: message.to_string(),
                source: None,
            }],
        }
    }

    /// Extract the error details for use in bulk responses.
    pub fn to_error_details(self) -> Vec<ApiErrorDetail> {
        self.errors
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let body = Json(ErrorResponse { errors: self.errors });
        (self.status, body).into_response()
    }
}

impl From<RepositoryError> for ApiError {
    fn from(err: RepositoryError) -> Self {
        match err {
            RepositoryError::NotFound(id) => Self::not_found(&id),
            RepositoryError::Internal(msg) => Self {
                status: StatusCode::INTERNAL_SERVER_ERROR,
                errors: vec![ApiErrorDetail {
                    status: "500".to_string(),
                    title: "Internal Server Error".to_string(),
                    detail: msg,
                    source: None,
                }],
            },
        }
    }
}

"#,
    );

    // Repository error
    code.push_str(
        r#"/// Repository error type.
#[derive(Debug)]
pub enum RepositoryError {
    NotFound(String),
    Internal(String),
}

impl std::fmt::Display for RepositoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RepositoryError::NotFound(id) => write!(f, "Not found: {}", id),
            RepositoryError::Internal(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::error::Error for RepositoryError {}

/// Transform a JSON pointer path from schema keys to human-readable field names.
fn transform_error_path(
    path: &str,
    field_map: &std::collections::HashMap<&str, &str>,
) -> String {
    // Path format is "/field" or "/field/nested" etc.
    // We only need to transform the top-level field name
    if let Some(stripped) = path.strip_prefix('/') {
        let parts: Vec<&str> = stripped.split('/').collect();
        if let Some(first) = parts.first() {
            if let Some(human_name) = field_map.get(*first) {
                // Reconstruct path with human-readable name
                let mut new_path = format!("/{}", human_name);
                for part in parts.iter().skip(1) {
                    new_path.push('/');
                    new_path.push_str(part);
                }
                return new_path;
            }
        }
    }
    // Return original path if no mapping found
    path.to_string()
}
"#,
    );

    code
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_shared_types() {
        let code = generate_shared_types();
        assert!(code.contains("pub struct ErrorResponse"));
        assert!(code.contains("pub struct ApiError"));
        assert!(code.contains("pub enum RepositoryError"));
    }
}

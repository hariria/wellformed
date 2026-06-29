//! Form-oriented runtime helpers for generated wellformed schemas.
//!
//! These types are intentionally framework-neutral so generated form modules can
//! expose metadata, schema JSON, validation errors, and state shape without
//! depending on a UI framework.

use crate::{FormError, Result, Schema, ValidationResult};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Schema JSON embedded at compile time by `wellformed_macros::wellformed!`.
///
/// This is intentionally a small value handle. It can be used in `const` items
/// and parsed into a runtime [`Schema`] when validation runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EmbeddedSchema {
    schema_json: &'static str,
}

impl EmbeddedSchema {
    /// Create an embedded schema handle from canonical wellformed IR JSON.
    pub const fn new(schema_json: &'static str) -> Self {
        Self { schema_json }
    }

    /// Return the embedded schema JSON.
    pub const fn schema_json(self) -> &'static str {
        self.schema_json
    }

    /// Parse the embedded JSON into a runtime schema.
    pub fn schema(self) -> Result<Schema> {
        Ok(serde_json::from_str(self.schema_json)?)
    }

    /// Validate and normalize a JSON value in place.
    pub fn validate(self, value: &mut Value) -> Result<ValidationResult> {
        let schema = self.schema()?;
        crate::validate(&schema, value)
    }

    /// Parse, validate, and normalize a JSON string.
    pub fn validate_json(self, json: &str) -> Result<(ValidationResult, Value)> {
        let mut value: Value = serde_json::from_str(json)?;
        let result = self.validate(&mut value)?;
        Ok((result, value))
    }
}

/// Coarse field kind used by generated UI/client metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FieldKind {
    String,
    Number,
    Integer,
    Boolean,
    Money,
    Currency,
    Decimal,
    Percentage,
    Date,
    Object,
    Array,
    Tuple,
    Enum,
    Literal,
    Json,
}

/// Static field metadata emitted by `wellformed_macros::form_schema!`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct FieldSpec {
    /// Schema key and submitted form field name.
    pub name: &'static str,
    /// Rust field name generated for the typed values struct.
    pub rust_name: &'static str,
    /// Optional human-readable label from the schema.
    pub label: Option<&'static str>,
    /// Optional description from the schema.
    pub description: Option<&'static str>,
    /// Whether the schema marks this field as required.
    pub required: bool,
    /// Coarse field kind for UI and client runtimes.
    pub kind: FieldKind,
    /// Optional schema section identifier.
    pub section: Option<&'static str>,
    /// Stable DOM id for the field's error message.
    pub error_id: &'static str,
}

impl FieldSpec {
    /// Return the label if present, otherwise the submitted field name.
    pub const fn label_or_name(self) -> &'static str {
        match self.label {
            Some(label) => label,
            None => self.name,
        }
    }
}

/// Static client/runtime metadata for a generated form schema.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ClientFormSpec {
    pub id: &'static str,
    pub title: Option<&'static str>,
    pub description: Option<&'static str>,
    pub schema_json: &'static str,
    pub fields: &'static [FieldSpec],
    /// Client module path/asset stem when client helpers were generated.
    pub client_module: Option<&'static str>,
}

/// Validation issues grouped with the submitted values that produced them.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FormErrors {
    /// Normalized values after wellformed transforms ran.
    pub values: Value,
    /// Hard validation errors.
    pub errors: Vec<FormError>,
    /// Non-blocking validation warnings.
    pub warnings: Vec<FormError>,
}

impl FormErrors {
    /// Build an empty error set for the given values.
    pub fn empty(values: Value) -> Self {
        Self {
            values,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// Build form errors from a validation result and normalized values.
    pub fn from_validation(values: Value, result: ValidationResult) -> Self {
        Self {
            values,
            errors: result.errors,
            warnings: result.warnings,
        }
    }

    /// Build a form error set for deserialization/coercion failures.
    pub fn from_message(
        values: Value,
        code: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            values,
            errors: vec![FormError::new(code, message, "")],
            warnings: Vec::new(),
        }
    }

    /// Whether the form has no hard validation errors.
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    /// Return the first error for a submitted field name.
    pub fn first_for(&self, name: &str) -> Option<&FormError> {
        let pointer = field_pointer(name);
        self.errors
            .iter()
            .find(|error| path_matches_field(&error.path, &pointer))
    }
}

impl Default for FormErrors {
    fn default() -> Self {
        Self::empty(Value::Object(Default::default()))
    }
}

/// Server/client form state shape for progressive form rendering.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FormState<T> {
    pub values: T,
    pub submitted_values: Value,
    pub errors: FormErrors,
    pub touched: Vec<String>,
    pub dirty: Vec<String>,
    pub submitting: bool,
    pub submit_count: u32,
}

impl<T> FormState<T> {
    pub fn new(values: T) -> Self {
        Self {
            values,
            submitted_values: Value::Object(Default::default()),
            errors: FormErrors::default(),
            touched: Vec::new(),
            dirty: Vec::new(),
            submitting: false,
            submit_count: 0,
        }
    }

    pub fn with_submitted_values(mut self, submitted_values: Value) -> Self {
        self.submitted_values = submitted_values;
        self
    }

    pub fn with_errors(mut self, errors: FormErrors) -> Self {
        self.errors = errors;
        self
    }

    pub fn with_touched(mut self, touched: impl Into<Vec<String>>) -> Self {
        self.touched = touched.into();
        self
    }

    pub fn with_dirty(mut self, dirty: impl Into<Vec<String>>) -> Self {
        self.dirty = dirty.into();
        self
    }

    pub fn with_submitting(mut self, submitting: bool) -> Self {
        self.submitting = submitting;
        self
    }

    pub fn with_submit_count(mut self, submit_count: u32) -> Self {
        self.submit_count = submit_count;
        self
    }

    pub fn field<'a>(&'a self, spec: &'static FieldSpec) -> FieldState<'a> {
        FieldState {
            spec,
            submitted_value: self.submitted_value(spec.name),
            error: self.errors.first_for(spec.name),
            touched: self.touched.iter().any(|name| name == spec.name),
            dirty: self.dirty.iter().any(|name| name == spec.name),
        }
    }

    pub fn submitted_value(&self, name: &str) -> Option<&Value> {
        self.submitted_values.pointer(&field_pointer(name))
    }
}

/// Derived state for a single field.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FieldState<'a> {
    pub spec: &'static FieldSpec,
    pub submitted_value: Option<&'a Value>,
    pub error: Option<&'a FormError>,
    pub touched: bool,
    pub dirty: bool,
}

impl<'a> FieldState<'a> {
    pub fn invalid(self) -> bool {
        self.error.is_some()
    }

    pub fn error_id(self) -> &'static str {
        self.spec.error_id
    }

    pub fn error_message(self) -> Option<&'a str> {
        self.error.map(|error| error.message.as_str())
    }

    pub fn value_json(self) -> Option<&'a Value> {
        self.submitted_value
    }

    pub fn value_str(self) -> &'a str {
        self.submitted_value
            .and_then(Value::as_str)
            .unwrap_or_default()
    }

    pub fn value_bool(self) -> bool {
        self.submitted_value
            .and_then(Value::as_bool)
            .unwrap_or(false)
    }
}

fn field_pointer(name: &str) -> String {
    let mut pointer = String::with_capacity(name.len() + 1);
    pointer.push('/');
    for ch in name.chars() {
        match ch {
            '~' => pointer.push_str("~0"),
            '/' => pointer.push_str("~1"),
            _ => pointer.push(ch),
        }
    }
    pointer
}

fn path_matches_field(path: &str, field_pointer: &str) -> bool {
    if path == field_pointer {
        return true;
    }

    path.strip_prefix(field_pointer)
        .map(|rest| rest.starts_with('/'))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FormError;
    use serde_json::json;

    static NAME_FIELD: FieldSpec = FieldSpec {
        name: "name",
        rust_name: "name",
        label: Some("Full name"),
        description: None,
        required: true,
        kind: FieldKind::String,
        section: None,
        error_id: "signup-name-error",
    };

    static SLASH_FIELD: FieldSpec = FieldSpec {
        name: "billing/address",
        rust_name: "billing_address",
        label: None,
        description: None,
        required: true,
        kind: FieldKind::Object,
        section: Some("billing"),
        error_id: "billing-address-error",
    };

    #[test]
    fn field_spec_uses_label_when_present() {
        assert_eq!(NAME_FIELD.label_or_name(), "Full name");
        assert_eq!(SLASH_FIELD.label_or_name(), "billing/address");
    }

    #[test]
    fn first_for_matches_exact_and_nested_field_paths() {
        let errors = FormErrors {
            values: json!({ "name": { "first": "" }, "nameplate": "" }),
            errors: vec![
                FormError::new("OTHER", "not this field", "/nameplate"),
                FormError::new("REQUIRED", "name required", "/name/first"),
            ],
            warnings: Vec::new(),
        };

        let error = errors.first_for("name").expect("name field error");
        assert_eq!(error.code, "REQUIRED");
    }

    #[test]
    fn first_for_escapes_json_pointer_field_names() {
        let errors = FormErrors {
            values: json!({ "billing/address": "" }),
            errors: vec![FormError::new(
                "REQUIRED",
                "billing address required",
                "/billing~1address",
            )],
            warnings: Vec::new(),
        };

        let error = errors
            .first_for("billing/address")
            .expect("escaped field error");
        assert_eq!(error.message, "billing address required");
    }

    #[test]
    fn form_state_derives_field_error_and_flags() {
        let submitted = json!({ "name": "" });
        let state = FormState::new(json!({ "name": "" }))
            .with_submitted_values(submitted.clone())
            .with_errors(FormErrors {
                values: json!({ "name": "" }),
                errors: vec![FormError::new("REQUIRED", "name required", "/name")],
                warnings: Vec::new(),
            })
            .with_touched(vec!["name".to_string()])
            .with_dirty(vec!["name".to_string()]);

        let field = state.field(&NAME_FIELD);

        assert!(field.invalid());
        assert!(field.touched);
        assert!(field.dirty);
        assert_eq!(field.error_id(), "signup-name-error");
        assert_eq!(field.error_message(), Some("name required"));
        assert_eq!(field.value_str(), "");
        assert_eq!(field.value_json(), submitted.pointer("/name"));
        assert_eq!(field.error.expect("field error").code, "REQUIRED");
    }

    #[test]
    fn form_state_keeps_invalid_submitted_values_for_rerender() {
        let submitted = json!({ "name": 42 });
        let errors = FormErrors {
            values: submitted.clone(),
            errors: vec![FormError::new("TYPE", "name must be a string", "/name")],
            warnings: Vec::new(),
        };
        let state = FormState::new(json!({}))
            .with_submitted_values(errors.values.clone())
            .with_errors(errors)
            .with_touched(vec!["name".to_string()])
            .with_dirty(vec!["name".to_string()])
            .with_submitting(true)
            .with_submit_count(1);
        let field = state.field(&NAME_FIELD);

        assert_eq!(field.value_json(), submitted.pointer("/name"));
        assert_eq!(field.value_str(), "");
        assert!(field.touched);
        assert!(field.dirty);
        assert!(state.submitting);
        assert_eq!(state.submit_count, 1);
    }
}

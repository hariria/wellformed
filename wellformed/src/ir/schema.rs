//! Top-level schema definitions.
//!
//! A Schema is the complete specification for a form or data structure,
//! including named type definitions and the root type.

use super::types::TypeSchema;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// A complete form schema specification.
///
/// The schema contains named type definitions that can be referenced
/// using `$ref`, plus a root type for the top-level structure.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Schema {
    /// Schema version (for compatibility checking).
    pub version: String,

    /// Optional schema identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Optional human-readable title.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// Optional description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// IRS form metadata (for tax forms).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub irs_form: Option<IrsFormMetadata>,

    /// PDF template information for rendering.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pdf_template: Option<PdfTemplate>,

    /// CSV import configuration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub import: Option<ImportConfig>,

    /// Named type definitions (ordered for deterministic output).
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub definitions: IndexMap<String, TypeSchema>,

    /// Form sections for UI grouping (canonical ordering).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub sections: BTreeMap<String, SectionDefinition>,

    /// The root type schema.
    pub root: TypeSchema,
}

/// IRS form metadata.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IrsFormMetadata {
    /// Form name (e.g., "1099-INT").
    pub name: String,

    /// Form title (e.g., "Interest Income").
    pub title: String,

    /// Revision string (e.g., "Rev. January 2024").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub revision: Option<String>,

    /// Revision date in YYYY-MM format.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub revision_date: Option<String>,

    /// OMB number (e.g., "1545-0112").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub omb_number: Option<String>,

    /// Catalog number (e.g., "14410K").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cat_number: Option<String>,
}

/// PDF template metadata for rendering.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PdfTemplate {
    /// SHA256 hash of the PDF file (for integrity verification).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,

    /// Filename of the PDF template (relative to the form's directory).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub filename: Option<String>,

    /// Path to the PDF template file (relative to crate root).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,

    /// Source URI where the PDF was obtained.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_uri: Option<String>,
}

/// Configuration for CSV import endpoints.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImportConfig {
    /// Whether CSV import is enabled for this form.
    #[serde(default)]
    pub enabled: bool,

    /// Maximum number of rows allowed in a single import (default: unlimited).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_rows: Option<usize>,

    /// Maximum file size in bytes (default: 50MB).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_file_size: Option<usize>,

    /// Custom CSV column mappings (field_name -> csv_column_name).
    /// If not specified, field names are used as column headers.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub column_mappings: BTreeMap<String, String>,

    /// Required CSV columns that must be present.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required_columns: Vec<String>,

    /// Description of the import endpoint for OpenAPI docs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl Default for ImportConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            max_rows: None,
            max_file_size: Some(50 * 1024 * 1024), // 50MB default
            column_mappings: BTreeMap::new(),
            required_columns: Vec::new(),
            description: None,
        }
    }
}

impl ImportConfig {
    /// Create an enabled import config with defaults.
    pub fn enabled() -> Self {
        Self {
            enabled: true,
            ..Default::default()
        }
    }

    /// Set the maximum number of rows.
    pub fn with_max_rows(mut self, max: usize) -> Self {
        self.max_rows = Some(max);
        self
    }

    /// Set the maximum file size.
    pub fn with_max_file_size(mut self, max: usize) -> Self {
        self.max_file_size = Some(max);
        self
    }

    /// Add a column mapping.
    pub fn with_column_mapping(
        mut self,
        field: impl Into<String>,
        column: impl Into<String>,
    ) -> Self {
        self.column_mappings.insert(field.into(), column.into());
        self
    }

    /// Set required columns.
    pub fn with_required_columns(mut self, columns: Vec<String>) -> Self {
        self.required_columns = columns;
        self
    }
}

/// Definition of a form section for UI grouping.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SectionDefinition {
    /// Human-readable section title.
    pub title: String,

    /// Display order (lower numbers appear first).
    pub order: u32,

    /// Optional description of the section.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl Schema {
    /// Create a new schema with a root type.
    pub fn new(version: impl Into<String>, root: TypeSchema) -> Self {
        Self {
            version: version.into(),
            id: None,
            title: None,
            description: None,
            irs_form: None,
            pdf_template: None,
            import: None,
            definitions: IndexMap::new(),
            sections: BTreeMap::new(),
            root,
        }
    }

    /// Set the schema ID.
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Set the schema title.
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set the schema description.
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Add a type definition.
    pub fn define(mut self, name: impl Into<String>, schema: TypeSchema) -> Self {
        self.definitions.insert(name.into(), schema);
        self
    }

    /// Get a type definition by name.
    pub fn get_definition(&self, name: &str) -> Option<&TypeSchema> {
        self.definitions.get(name)
    }

    /// Resolve a reference to its schema.
    ///
    /// Returns None if the reference doesn't exist.
    pub fn resolve_ref(&self, name: &str) -> Option<&TypeSchema> {
        self.definitions.get(name)
    }
}

/// Metadata about a schema (for schema registries).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SchemaMeta {
    /// Schema identifier.
    pub id: String,

    /// Schema version.
    pub version: String,

    /// Human-readable title.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// Description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Form year, commonly used by tax or annual reporting schemas.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tax_year: Option<u16>,

    /// Form type (e.g., "1099-NEC", "1099-K").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub form_type: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::types::{ObjectSchema, StringSchema};

    #[test]
    fn test_schema_new() {
        let schema = Schema::new("1.0.0", TypeSchema::object())
            .with_id("test-schema")
            .with_title("Test Schema");

        assert_eq!(schema.version, "1.0.0");
        assert_eq!(schema.id, Some("test-schema".to_string()));
        assert_eq!(schema.title, Some("Test Schema".to_string()));
    }

    #[test]
    fn test_schema_with_definitions() {
        let schema = Schema::new("1.0.0", TypeSchema::ref_to("Person")).define(
            "Person",
            TypeSchema::Object(
                ObjectSchema::new()
                    .property("name", TypeSchema::string())
                    .property("age", TypeSchema::integer()),
            ),
        );

        assert!(schema.get_definition("Person").is_some());
        assert!(schema.get_definition("Unknown").is_none());
    }

    #[test]
    fn test_schema_serde_roundtrip() {
        let schema = Schema::new(
            "1.0.0",
            TypeSchema::Object(
                ObjectSchema::new()
                    .property("tin", TypeSchema::String(StringSchema::new()))
                    .property("name", TypeSchema::string()),
            ),
        )
        .with_id("1099-nec-2025")
        .with_title("Form 1099-NEC (2025)");

        let json = serde_json::to_string_pretty(&schema).unwrap();
        let parsed: Schema = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, schema);
    }

    #[test]
    fn test_schema_json_format() {
        let schema = Schema::new("1.0.0", TypeSchema::string());
        let json = serde_json::to_string_pretty(&schema).unwrap();

        // Should produce clean, deterministic JSON
        assert!(json.contains("\"version\""));
        assert!(json.contains("\"root\""));
    }
}

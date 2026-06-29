//! Type layer definitions for the IR.
//!
//! This module defines the type system for form schemas, including
//! primitives, domain types, and composite types.

use super::predicate::Constraint;
use super::transform::Transform;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A type schema defining the structure and validation rules for a value.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TypeSchema {
    // ========================================================================
    // Primitive types
    // ========================================================================
    /// String type with optional transforms and constraints.
    String(StringSchema),

    /// Number type (floating point).
    Number(NumberSchema),

    /// Integer type (platform-dependent, use specific types for precision).
    Integer(IntegerSchema),

    /// Signed 32-bit integer (-2,147,483,648 to 2,147,483,647).
    Int32(Int32Schema),

    /// Signed 64-bit integer (-9,223,372,036,854,775,808 to 9,223,372,036,854,775,807).
    Int64(Int64Schema),

    /// Unsigned 32-bit integer (0 to 4,294,967,295).
    Uint32(Uint32Schema),

    /// Unsigned 64-bit integer (0 to 18,446,744,073,709,551,615).
    Uint64(Uint64Schema),

    /// Boolean type.
    Boolean(BooleanSchema),

    // ========================================================================
    // Domain-specific types
    // ========================================================================
    /// Money type (fixed-point decimal, stored as cents).
    Money(MoneySchema),

    /// Currency type with ISO 4217 code.
    Currency(CurrencySchema),

    /// Decimal type with configurable precision and scale.
    Decimal(DecimalSchema),

    /// Percentage type (0-100 or 0-1 format).
    Percentage(PercentageSchema),

    /// Date type (validated format).
    Date(DateSchema),

    // ========================================================================
    // Composite types
    // ========================================================================
    /// Object type with named properties.
    Object(ObjectSchema),

    /// Array type with item schema.
    Array(ArraySchema),

    /// Tuple type with fixed-position item schemas.
    Tuple(TupleSchema),

    /// Enum type (one of a fixed set of values).
    Enum(EnumSchema),

    /// Literal type matching exactly one value.
    Literal(LiteralSchema),

    /// Never type (no value is valid).
    Never(NeverSchema),

    /// Union type (one of several schemas).
    Union(UnionSchema),

    /// Intersection type (must satisfy all schemas).
    Intersection(IntersectionSchema),

    /// Record type (object with dynamic keys and uniform value schema).
    Record(RecordSchema),

    /// Preprocess wrapper (applies transforms before validating nested schema).
    Preprocess(PreprocessSchema),

    /// Catch wrapper (returns fallback value if nested schema validation fails).
    Catch(CatchSchema),

    // ========================================================================
    // Reference type
    // ========================================================================
    /// Reference to a named type definition.
    Ref {
        /// Name of the referenced type.
        #[serde(rename = "$ref")]
        name: String,
    },

    // ========================================================================
    // Any type (for extensibility)
    // ========================================================================
    /// Any JSON value (minimal validation).
    Any(AnySchema),
}

// ============================================================================
// String Schema
// ============================================================================

/// Schema for string values.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct StringSchema {
    /// Transforms to apply before validation.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub transforms: Vec<Transform>,

    /// Validation constraints.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub constraints: Vec<Constraint>,

    /// Optional description for documentation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Optional example value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub example: Option<String>,
}

impl StringSchema {
    /// Create a new empty string schema.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a transform.
    pub fn transform(mut self, t: Transform) -> Self {
        self.transforms.push(t);
        self
    }

    /// Add a constraint.
    pub fn constraint(mut self, c: Constraint) -> Self {
        self.constraints.push(c);
        self
    }

    /// Set description.
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }
}

// ============================================================================
// Number Schema
// ============================================================================

/// Schema for number (floating-point) values.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct NumberSchema {
    /// Transforms to apply before validation.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub transforms: Vec<Transform>,

    /// Validation constraints.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub constraints: Vec<Constraint>,

    /// Optional description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl NumberSchema {
    pub fn new() -> Self {
        Self::default()
    }
}

// ============================================================================
// Integer Schema
// ============================================================================

/// Schema for integer values.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct IntegerSchema {
    /// Transforms to apply before validation.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub transforms: Vec<Transform>,

    /// Validation constraints.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub constraints: Vec<Constraint>,

    /// Optional description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl IntegerSchema {
    pub fn new() -> Self {
        Self::default()
    }
}

// ============================================================================
// Specific Integer Schemas
// ============================================================================

/// Schema for signed 32-bit integer values.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Int32Schema {
    /// Transforms to apply before validation.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub transforms: Vec<Transform>,

    /// Validation constraints.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub constraints: Vec<Constraint>,

    /// Optional description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl Int32Schema {
    pub fn new() -> Self {
        Self::default()
    }
}

/// Schema for signed 64-bit integer values.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Int64Schema {
    /// Transforms to apply before validation.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub transforms: Vec<Transform>,

    /// Validation constraints.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub constraints: Vec<Constraint>,

    /// Optional description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl Int64Schema {
    pub fn new() -> Self {
        Self::default()
    }
}

/// Schema for unsigned 32-bit integer values.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Uint32Schema {
    /// Transforms to apply before validation.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub transforms: Vec<Transform>,

    /// Validation constraints.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub constraints: Vec<Constraint>,

    /// Optional description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl Uint32Schema {
    pub fn new() -> Self {
        Self::default()
    }
}

/// Schema for unsigned 64-bit integer values.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Uint64Schema {
    /// Transforms to apply before validation.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub transforms: Vec<Transform>,

    /// Validation constraints.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub constraints: Vec<Constraint>,

    /// Optional description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl Uint64Schema {
    pub fn new() -> Self {
        Self::default()
    }
}

// ============================================================================
// Boolean Schema
// ============================================================================

/// Schema for boolean values.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct BooleanSchema {
    /// Optional description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl BooleanSchema {
    pub fn new() -> Self {
        Self::default()
    }
}

// ============================================================================
// Money Schema
// ============================================================================

/// Schema for money values (fixed-point decimal).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MoneySchema {
    /// Number of decimal places (default 2).
    #[serde(default = "default_scale")]
    pub scale: u8,

    /// Transforms to apply before validation.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub transforms: Vec<Transform>,

    /// Validation constraints.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub constraints: Vec<Constraint>,

    /// Optional description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

fn default_scale() -> u8 {
    2
}

impl Default for MoneySchema {
    fn default() -> Self {
        Self {
            scale: 2,
            transforms: Vec::new(),
            constraints: Vec::new(),
            description: None,
        }
    }
}

impl MoneySchema {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_scale(mut self, scale: u8) -> Self {
        self.scale = scale;
        self
    }
}

// ============================================================================
// Currency Schema
// ============================================================================

/// Schema for currency values with ISO 4217 currency code.
///
/// Unlike Money (which just stores a numeric value in cents), Currency
/// includes the currency code for multi-currency support.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CurrencySchema {
    /// ISO 4217 currency code (e.g., "USD", "EUR", "GBP", "JPY").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,

    /// Number of decimal places (default 2, varies by currency: JPY=0, BHD=3).
    #[serde(default = "default_scale")]
    pub scale: u8,

    /// Transforms to apply before validation.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub transforms: Vec<Transform>,

    /// Validation constraints.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub constraints: Vec<Constraint>,

    /// Optional description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl Default for CurrencySchema {
    fn default() -> Self {
        Self {
            code: None,
            scale: 2,
            transforms: Vec::new(),
            constraints: Vec::new(),
            description: None,
        }
    }
}

impl CurrencySchema {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the ISO 4217 currency code.
    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());
        self
    }

    /// Set the number of decimal places.
    pub fn with_scale(mut self, scale: u8) -> Self {
        self.scale = scale;
        self
    }
}

// ============================================================================
// Decimal Schema
// ============================================================================

/// Schema for decimal values with configurable precision.
///
/// This is useful for financial calculations where exact decimal
/// representation is required (unlike floating point).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct DecimalSchema {
    /// Maximum number of digits in total (precision).
    /// For example, precision=10 allows up to 10 total digits.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub precision: Option<u8>,

    /// Number of digits after the decimal point (scale).
    /// For example, scale=4 allows up to 4 decimal places (e.g., tax rates like 0.0625).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scale: Option<u8>,

    /// Transforms to apply before validation.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub transforms: Vec<Transform>,

    /// Validation constraints.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub constraints: Vec<Constraint>,

    /// Optional description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl DecimalSchema {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set precision (total digits).
    pub fn with_precision(mut self, precision: u8) -> Self {
        self.precision = Some(precision);
        self
    }

    /// Set scale (decimal places).
    pub fn with_scale(mut self, scale: u8) -> Self {
        self.scale = Some(scale);
        self
    }
}

// ============================================================================
// Percentage Schema
// ============================================================================

/// Format for percentage values.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PercentageFormat {
    /// Percentage as 0-100 (e.g., 6.25 means 6.25%)
    Whole,
    /// Percentage as 0-1 decimal (e.g., 0.0625 means 6.25%)
    #[default]
    Decimal,
}

/// Schema for percentage values.
///
/// Useful for tax rates, ownership percentages, withholding rates, etc.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PercentageSchema {
    /// The format for the percentage value.
    #[serde(default)]
    pub format: PercentageFormat,

    /// Whether to allow values over 100% (or 1.0 in decimal format).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub allow_over_100: bool,

    /// Number of decimal places for precision (e.g., 4 for 0.0625).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scale: Option<u8>,

    /// Transforms to apply before validation.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub transforms: Vec<Transform>,

    /// Validation constraints.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub constraints: Vec<Constraint>,

    /// Optional description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl PercentageSchema {
    pub fn new() -> Self {
        Self::default()
    }

    /// Use whole number format (0-100).
    pub fn whole() -> Self {
        Self {
            format: PercentageFormat::Whole,
            ..Self::default()
        }
    }

    /// Use decimal format (0-1).
    pub fn decimal() -> Self {
        Self {
            format: PercentageFormat::Decimal,
            ..Self::default()
        }
    }

    /// Allow values over 100%.
    pub fn allow_over_100(mut self) -> Self {
        self.allow_over_100 = true;
        self
    }

    /// Set decimal precision.
    pub fn with_scale(mut self, scale: u8) -> Self {
        self.scale = Some(scale);
        self
    }
}

// ============================================================================
// Date Schema
// ============================================================================

/// Schema for date values.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct DateSchema {
    /// Expected input format (strftime-style). If set, input will be parsed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,

    /// Transforms to apply before validation.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub transforms: Vec<Transform>,

    /// Validation constraints.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub constraints: Vec<Constraint>,

    /// Optional description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl DateSchema {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_format(mut self, format: impl Into<String>) -> Self {
        self.format = Some(format.into());
        self
    }
}

// ============================================================================
// Object Schema
// ============================================================================

/// Schema for object (struct) values.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct ObjectSchema {
    /// Field definitions (ordered map for deterministic output).
    /// Serialized as `properties` to match the portable JSON IR.
    /// Also accepts legacy `fields` on input.
    #[serde(
        rename = "properties",
        default,
        alias = "fields",
        skip_serializing_if = "IndexMap::is_empty"
    )]
    pub fields: IndexMap<String, PropertySchema>,

    /// Page definitions for PDF rendering.
    /// Each page can specify which fields to render and their positions.
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub pages: IndexMap<String, PageSchema>,

    /// AcroForm field mappings for PDF form filling.
    /// Maps schema fields to PDF AcroForm field IDs.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub acroform_mappings: Vec<AcroFormFieldComposition>,

    /// Whether additional properties are allowed.
    #[serde(default = "default_false", skip_serializing_if = "is_false")]
    pub additional_properties: bool,

    /// Behavior for unknown keys not defined in `fields`.
    ///
    /// If omitted, runtime behavior falls back to `additional_properties`
    /// for backwards compatibility.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unknown_keys: Option<UnknownKeysBehavior>,

    /// Schema for unknown keys (equivalent to Zod's catchall).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub catchall: Option<Box<TypeSchema>>,

    /// Cross-field validation rules.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rules: Vec<Constraint>,

    /// Optional description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Schema for a PDF page defining which fields appear and their render positions.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct PageSchema {
    /// Human-readable name for this page (e.g., "Copy A", "Copy B").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Fields to render on this page, with their render metadata.
    /// Keys are field names from the parent object's `fields`.
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub fields: IndexMap<String, RenderMetadata>,
}

/// Defines how schema fields compose into a single AcroForm PDF field.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct AcroFormFieldComposition {
    /// AcroForm field ID in the PDF, for example `f1_1[0]`.
    pub field_id: String,

    /// Page number (1-indexed).
    pub page: u32,

    /// Copy identifier (e.g., "A", "B", "C").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub copy: Option<String>,

    /// Schema field keys that compose into this AcroForm field.
    /// Usually a single field, but can be multiple for concatenated values.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub compose: Vec<String>,

    /// Separator to use when composing multiple fields (simple join).
    /// Ignored if `format` is specified.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub separator: String,

    /// Format string for composing multiple fields.
    /// Uses `{field_key}` placeholders, e.g., "{F}\n{G}\n{M}\n{N}\n{O}, {P} {Q}".
    /// Takes precedence over `separator` when specified.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
}

fn default_false() -> bool {
    false
}

fn is_false(b: &bool) -> bool {
    !*b
}

/// Behavior for unknown keys on object schemas.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UnknownKeysBehavior {
    /// Reject unknown keys with validation errors.
    Strict,
    /// Keep unknown keys as-is.
    Passthrough,
    /// Remove unknown keys from the normalized value.
    Strip,
}

impl ObjectSchema {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a required field.
    pub fn field(mut self, name: impl Into<String>, schema: TypeSchema) -> Self {
        self.fields.insert(
            name.into(),
            PropertySchema {
                schema,
                required: true,
                description: None,
                label: None,
                render: None,
                acroform: None,
                section: None,
            },
        );
        self
    }

    /// Add a required property (alias for field, for backwards compatibility).
    pub fn property(self, name: impl Into<String>, schema: TypeSchema) -> Self {
        self.field(name, schema)
    }

    /// Add an optional field.
    pub fn optional_field(mut self, name: impl Into<String>, schema: TypeSchema) -> Self {
        self.fields.insert(
            name.into(),
            PropertySchema {
                schema,
                required: false,
                description: None,
                label: None,
                render: None,
                acroform: None,
                section: None,
            },
        );
        self
    }

    /// Add an optional property (alias for optional_field, for backwards compatibility).
    pub fn optional_property(self, name: impl Into<String>, schema: TypeSchema) -> Self {
        self.optional_field(name, schema)
    }

    /// Add a page definition for PDF rendering.
    pub fn page(mut self, page_num: impl Into<String>, page_schema: PageSchema) -> Self {
        self.pages.insert(page_num.into(), page_schema);
        self
    }

    /// Add a cross-field rule.
    pub fn rule(mut self, constraint: Constraint) -> Self {
        self.rules.push(constraint);
        self
    }

    /// Allow additional properties.
    pub fn allow_additional(mut self) -> Self {
        self.additional_properties = true;
        self.unknown_keys = Some(UnknownKeysBehavior::Passthrough);
        self
    }

    /// Reject unknown keys.
    pub fn strict(mut self) -> Self {
        self.additional_properties = false;
        self.unknown_keys = Some(UnknownKeysBehavior::Strict);
        self
    }

    /// Keep unknown keys.
    pub fn passthrough(mut self) -> Self {
        self.additional_properties = true;
        self.unknown_keys = Some(UnknownKeysBehavior::Passthrough);
        self
    }

    /// Remove unknown keys.
    pub fn strip(mut self) -> Self {
        self.additional_properties = false;
        self.unknown_keys = Some(UnknownKeysBehavior::Strip);
        self
    }

    /// Validate unknown keys with a catchall schema.
    pub fn catchall(mut self, schema: TypeSchema) -> Self {
        self.catchall = Some(Box::new(schema));
        self.additional_properties = true;
        self.unknown_keys = Some(UnknownKeysBehavior::Passthrough);
        self
    }
}

impl PageSchema {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the page name.
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Add a field with its render metadata.
    pub fn field(mut self, name: impl Into<String>, render: RenderMetadata) -> Self {
        self.fields.insert(name.into(), render);
        self
    }
}

/// A property in an object schema.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PropertySchema {
    /// The schema for this property's value.
    #[serde(flatten)]
    pub schema: TypeSchema,

    /// Whether this property is required.
    #[serde(default = "default_true", skip_serializing_if = "is_true")]
    pub required: bool,

    /// Optional description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Human-readable label for the field (used for Rust field name generation).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,

    /// Optional rendering metadata for PDF overlay.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub render: Option<RenderMetadata>,

    /// Optional AcroForm field metadata.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub acroform: Option<AcroFormMetadata>,

    /// Optional section identifier for UI grouping.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub section: Option<String>,
}

/// Metadata for rendering a field on a PDF.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RenderMetadata {
    /// Render type: "text" or "checkbox"
    #[serde(rename = "type")]
    pub render_type: String,

    /// Page number (0-indexed)
    #[serde(default)]
    pub page: u32,

    /// X position in PDF points
    pub x: f32,

    /// Y position in PDF points
    pub y: f32,

    /// Font size in points (for text)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub font_size: Option<f32>,

    /// Font type: "regular", "bold", "italic", "ocr_a"
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub font: Option<String>,

    /// Color as hex string, e.g., "#000000"
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,

    /// Horizontal alignment: "left", "right", "center"
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub align: Option<String>,

    /// Vertical alignment: "baseline", "top", "bottom"
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub v_align: Option<String>,

    /// Horizontal scale (1.0 = 100%)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub h_scale: Option<f32>,

    /// Width (for checkboxes)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub width: Option<f32>,

    /// Height (for checkboxes)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub height: Option<f32>,

    /// Maximum width before text wraps
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_width: Option<f32>,

    /// Whether text can span multiple lines
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub multiline: Option<bool>,

    /// Line height for multiline text
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line_height: Option<f32>,

    /// Box number on form (e.g., "1", "2a")
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub box_number: Option<String>,
}

/// Metadata for AcroForm field mapping.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AcroFormMetadata {
    /// The AcroForm field ID (e.g., "f1_2", "c1_1")
    pub field_id: String,

    /// Field type: "text", "checkbox", "radio", etc.
    pub field_type: String,

    /// Optional copy suffix for multi-copy forms (e.g., "CopyA", "CopyB")
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub copy_suffix: Option<String>,
}

fn default_true() -> bool {
    true
}

fn is_true(b: &bool) -> bool {
    *b
}

// ============================================================================
// Array Schema
// ============================================================================

/// Schema for array values.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArraySchema {
    /// Schema for array items.
    pub items: Box<TypeSchema>,

    /// Minimum number of items.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_items: Option<usize>,

    /// Maximum number of items.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_items: Option<usize>,

    /// Validation constraints on the array itself.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub constraints: Vec<Constraint>,

    /// Optional description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl ArraySchema {
    pub fn new(items: TypeSchema) -> Self {
        Self {
            items: Box::new(items),
            min_items: None,
            max_items: None,
            constraints: Vec::new(),
            description: None,
        }
    }

    pub fn min_items(mut self, n: usize) -> Self {
        self.min_items = Some(n);
        self
    }

    pub fn max_items(mut self, n: usize) -> Self {
        self.max_items = Some(n);
        self
    }
}

// ============================================================================
// Tuple Schema
// ============================================================================

/// Schema for tuple values (fixed-length arrays with per-index types).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TupleSchema {
    /// Schemas for tuple items, in order.
    pub items: Vec<TypeSchema>,

    /// Optional description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl TupleSchema {
    pub fn new(items: Vec<TypeSchema>) -> Self {
        Self {
            items,
            description: None,
        }
    }
}

// ============================================================================
// Enum Schema
// ============================================================================

/// Schema for enum values (one of a fixed set).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnumSchema {
    /// Allowed values.
    pub values: Vec<Value>,

    /// Optional description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl EnumSchema {
    pub fn new(values: Vec<Value>) -> Self {
        Self {
            values,
            description: None,
        }
    }

    pub fn from_strings(values: &[&str]) -> Self {
        Self {
            values: values
                .iter()
                .map(|s| Value::String(s.to_string()))
                .collect(),
            description: None,
        }
    }
}

// ============================================================================
// Literal Schema
// ============================================================================

/// Schema for literal values (must match exactly).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LiteralSchema {
    /// The literal value to match.
    pub value: Value,

    /// Optional description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl LiteralSchema {
    pub fn new(value: Value) -> Self {
        Self {
            value,
            description: None,
        }
    }
}

// ============================================================================
// Never Schema
// ============================================================================

/// Schema for never values (always invalid).
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct NeverSchema {
    /// Optional description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl NeverSchema {
    pub fn new() -> Self {
        Self::default()
    }
}

// ============================================================================
// Union Schema
// ============================================================================

/// Schema for union types (one of several schemas).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnionSchema {
    /// Possible schemas.
    #[serde(rename = "oneOf")]
    pub variants: Vec<TypeSchema>,

    /// Optional discriminator field for tagged unions.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub discriminator: Option<String>,

    /// Optional description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl UnionSchema {
    pub fn new(variants: Vec<TypeSchema>) -> Self {
        Self {
            variants,
            discriminator: None,
            description: None,
        }
    }

    pub fn discriminated(variants: Vec<TypeSchema>, discriminator: impl Into<String>) -> Self {
        Self {
            variants,
            discriminator: Some(discriminator.into()),
            description: None,
        }
    }
}

// ============================================================================
// Intersection Schema
// ============================================================================

/// Schema for intersection types (all schemas must match).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IntersectionSchema {
    /// Schemas that must all validate.
    #[serde(rename = "allOf")]
    pub variants: Vec<TypeSchema>,

    /// Optional description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl IntersectionSchema {
    pub fn new(variants: Vec<TypeSchema>) -> Self {
        Self {
            variants,
            description: None,
        }
    }
}

// ============================================================================
// Record Schema
// ============================================================================

/// Schema for record/object maps with dynamic keys.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecordSchema {
    /// Schema for record values.
    pub value: Box<TypeSchema>,

    /// Optional schema for record keys (validated against each key string).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub key: Option<Box<TypeSchema>>,

    /// Whether known-key records should be treated as partial.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub partial: bool,

    /// Optional description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl RecordSchema {
    pub fn new(value: TypeSchema) -> Self {
        Self {
            value: Box::new(value),
            key: None,
            partial: false,
            description: None,
        }
    }

    pub fn with_key(mut self, key: TypeSchema) -> Self {
        self.key = Some(Box::new(key));
        self
    }

    pub fn partial(mut self) -> Self {
        self.partial = true;
        self
    }
}

// ============================================================================
// Preprocess/Catch Schema
// ============================================================================

/// Schema wrapper that applies transforms before validating an inner schema.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PreprocessSchema {
    /// Transforms to apply to the incoming value.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub transforms: Vec<Transform>,

    /// The nested schema to validate after preprocessing.
    pub schema: Box<TypeSchema>,

    /// Optional description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl PreprocessSchema {
    pub fn new(schema: TypeSchema) -> Self {
        Self {
            transforms: Vec::new(),
            schema: Box::new(schema),
            description: None,
        }
    }

    pub fn transform(mut self, transform: Transform) -> Self {
        self.transforms.push(transform);
        self
    }
}

/// Schema wrapper that provides a fallback value when validation fails.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CatchSchema {
    /// The nested schema to validate.
    pub schema: Box<TypeSchema>,

    /// Fallback value returned if nested validation fails.
    pub value: Value,

    /// Optional description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl CatchSchema {
    pub fn new(schema: TypeSchema, value: Value) -> Self {
        Self {
            schema: Box::new(schema),
            value,
            description: None,
        }
    }
}

// ============================================================================
// Any Schema
// ============================================================================

/// Schema for any JSON value.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct AnySchema {
    /// Optional description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl AnySchema {
    pub fn new() -> Self {
        Self::default()
    }
}

// ============================================================================
// TypeSchema constructors
// ============================================================================

impl TypeSchema {
    /// Create a string schema.
    pub fn string() -> Self {
        Self::String(StringSchema::new())
    }

    /// Create a number schema.
    pub fn number() -> Self {
        Self::Number(NumberSchema::new())
    }

    /// Create an integer schema.
    pub fn integer() -> Self {
        Self::Integer(IntegerSchema::new())
    }

    /// Create a signed 32-bit integer schema.
    pub fn int32() -> Self {
        Self::Int32(Int32Schema::new())
    }

    /// Create a signed 64-bit integer schema.
    pub fn int64() -> Self {
        Self::Int64(Int64Schema::new())
    }

    /// Create an unsigned 32-bit integer schema.
    pub fn uint32() -> Self {
        Self::Uint32(Uint32Schema::new())
    }

    /// Create an unsigned 64-bit integer schema.
    pub fn uint64() -> Self {
        Self::Uint64(Uint64Schema::new())
    }

    /// Create a boolean schema.
    pub fn boolean() -> Self {
        Self::Boolean(BooleanSchema::new())
    }

    /// Create a money schema.
    pub fn money() -> Self {
        Self::Money(MoneySchema::new())
    }

    /// Create a currency schema.
    pub fn currency() -> Self {
        Self::Currency(CurrencySchema::new())
    }

    /// Create a currency schema with a specific ISO 4217 code.
    pub fn currency_with_code(code: impl Into<String>) -> Self {
        Self::Currency(CurrencySchema::new().with_code(code))
    }

    /// Create a decimal schema.
    pub fn decimal() -> Self {
        Self::Decimal(DecimalSchema::new())
    }

    /// Create a decimal schema with specific precision and scale.
    pub fn decimal_with(precision: u8, scale: u8) -> Self {
        Self::Decimal(
            DecimalSchema::new()
                .with_precision(precision)
                .with_scale(scale),
        )
    }

    /// Create a percentage schema (decimal format 0-1).
    pub fn percentage() -> Self {
        Self::Percentage(PercentageSchema::new())
    }

    /// Create a percentage schema with whole number format (0-100).
    pub fn percentage_whole() -> Self {
        Self::Percentage(PercentageSchema::whole())
    }

    /// Create a date schema.
    pub fn date() -> Self {
        Self::Date(DateSchema::new())
    }

    /// Create an object schema.
    pub fn object() -> Self {
        Self::Object(ObjectSchema::new())
    }

    /// Create an array schema.
    pub fn array(items: TypeSchema) -> Self {
        Self::Array(ArraySchema::new(items))
    }

    /// Create a tuple schema.
    pub fn tuple(items: Vec<TypeSchema>) -> Self {
        Self::Tuple(TupleSchema::new(items))
    }

    /// Create an enum schema from strings.
    pub fn enum_values(values: &[&str]) -> Self {
        Self::Enum(EnumSchema::from_strings(values))
    }

    /// Create an intersection schema.
    pub fn intersection(variants: Vec<TypeSchema>) -> Self {
        Self::Intersection(IntersectionSchema::new(variants))
    }

    /// Create a record schema with a value schema.
    pub fn record(value: TypeSchema) -> Self {
        Self::Record(RecordSchema::new(value))
    }

    /// Create a preprocess wrapper for a schema.
    pub fn preprocess(schema: TypeSchema, transforms: Vec<Transform>) -> Self {
        Self::Preprocess(PreprocessSchema {
            transforms,
            schema: Box::new(schema),
            description: None,
        })
    }

    /// Create a catch wrapper for a schema with fallback value.
    pub fn catch(schema: TypeSchema, value: Value) -> Self {
        Self::Catch(CatchSchema::new(schema, value))
    }

    /// Create a literal schema.
    pub fn literal(value: Value) -> Self {
        Self::Literal(LiteralSchema::new(value))
    }

    /// Create a nullable schema (`schema | null`).
    pub fn nullable(schema: TypeSchema) -> Self {
        Self::Union(UnionSchema::new(vec![
            schema,
            TypeSchema::literal(Value::Null),
        ]))
    }

    /// Create a never schema.
    pub fn never() -> Self {
        Self::Never(NeverSchema::new())
    }

    /// Create a reference to a named type.
    pub fn ref_to(name: impl Into<String>) -> Self {
        Self::Ref { name: name.into() }
    }

    /// Create an any schema.
    pub fn any() -> Self {
        Self::Any(AnySchema::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::error::ErrorMeta;
    use crate::ir::predicate::Predicate;

    #[test]
    fn test_string_schema_serde() {
        let schema =
            TypeSchema::String(StringSchema::new().transform(Transform::trim()).constraint(
                Constraint::new(
                    Predicate::regex(r"^\d{9}$"),
                    ErrorMeta::new("INVALID", "Invalid format"),
                ),
            ));

        let json = serde_json::to_string_pretty(&schema).unwrap();
        assert!(json.contains("string"));
        assert!(json.contains("trim"));

        let parsed: TypeSchema = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, schema);
    }

    #[test]
    fn test_object_schema_serde() {
        let schema = TypeSchema::Object(
            ObjectSchema::new()
                .property("name", TypeSchema::string())
                .optional_property("age", TypeSchema::integer())
                .strip()
                .catchall(TypeSchema::integer()),
        );

        let json = serde_json::to_string_pretty(&schema).unwrap();
        let parsed: TypeSchema = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, schema);
    }

    #[test]
    fn test_object_schema_serializes_properties_key() {
        let schema = TypeSchema::Object(ObjectSchema::new().property("name", TypeSchema::string()));

        let json = serde_json::to_string(&schema).unwrap();
        assert!(json.contains("\"properties\""));
        assert!(!json.contains("\"fields\""));

        let legacy = r#"{"type":"object","fields":{"name":{"type":"string"}}}"#;
        let parsed: TypeSchema = serde_json::from_str(legacy).unwrap();
        assert_eq!(parsed, schema);
    }

    #[test]
    fn test_array_schema_serde() {
        let schema = TypeSchema::array(TypeSchema::string());
        let json = serde_json::to_string(&schema).unwrap();
        let parsed: TypeSchema = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, schema);
    }

    #[test]
    fn test_ref_schema_serde() {
        let schema = TypeSchema::ref_to("Payee");
        let json = serde_json::to_string(&schema).unwrap();
        assert!(json.contains("$ref"));
        assert!(json.contains("Payee"));
    }

    #[test]
    fn test_enum_schema() {
        let schema = TypeSchema::enum_values(&["SSN", "EIN", "ITIN"]);
        let json = serde_json::to_string(&schema).unwrap();
        assert!(json.contains("SSN"));
        assert!(json.contains("EIN"));
    }

    #[test]
    fn test_literal_schema_serde() {
        let schema = TypeSchema::literal(serde_json::json!("active"));
        let json = serde_json::to_string(&schema).unwrap();
        let parsed: TypeSchema = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, schema);
    }

    #[test]
    fn test_never_schema_serde() {
        let schema = TypeSchema::never();
        let json = serde_json::to_string(&schema).unwrap();
        let parsed: TypeSchema = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, schema);
    }

    #[test]
    fn test_tuple_schema_serde() {
        let schema = TypeSchema::tuple(vec![TypeSchema::string(), TypeSchema::integer()]);
        let json = serde_json::to_string(&schema).unwrap();
        let parsed: TypeSchema = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, schema);
    }

    #[test]
    fn test_nullable_schema() {
        let schema = TypeSchema::nullable(TypeSchema::string());
        let json = serde_json::to_string(&schema).unwrap();
        let parsed: TypeSchema = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, schema);
    }

    #[test]
    fn test_intersection_schema_serde() {
        let schema =
            TypeSchema::intersection(vec![TypeSchema::string(), TypeSchema::literal(Value::Null)]);
        let json = serde_json::to_string(&schema).unwrap();
        let parsed: TypeSchema = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, schema);
    }

    #[test]
    fn test_record_schema_serde() {
        let schema = TypeSchema::Record(
            RecordSchema::new(TypeSchema::integer()).with_key(TypeSchema::string()),
        );
        let json = serde_json::to_string(&schema).unwrap();
        let parsed: TypeSchema = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, schema);
    }

    #[test]
    fn test_preprocess_catch_schema_serde() {
        let schema = TypeSchema::catch(
            TypeSchema::preprocess(TypeSchema::string(), vec![Transform::trim()]),
            serde_json::json!("fallback"),
        );
        let json = serde_json::to_string(&schema).unwrap();
        let parsed: TypeSchema = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, schema);
    }
}

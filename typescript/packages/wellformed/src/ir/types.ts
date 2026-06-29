/**
 * IR Type definitions matching the Rust wellformed crate.
 *
 * These types represent the portable, declarative schema format
 * that can be serialized to JSON and evaluated in any runtime.
 *
 * IMPORTANT: These types must match the Rust serde serialization format exactly.
 */

// ============================================================================
// Error Types
// ============================================================================

export type ErrorSeverity = "error" | "warning";

export interface ErrorMeta {
  code: string;
  message: string;
  path?: string;
  severity?: ErrorSeverity;
  help?: string;
  source?: string;
}

export interface FormError {
  code: string;
  message: string;
  path: string;
  severity: ErrorSeverity;
  help?: string;
  source?: string;
}

// ============================================================================
// Transform Types
// ============================================================================

// Rust uses #[serde(tag = "fn")] so transforms use "fn" not "type"
export type Transform =
  | { fn: "trim" }
  | { fn: "collapse_whitespace" }
  | { fn: "digits_only" }
  | { fn: "upper" }
  | { fn: "lower" }
  | { fn: "money_to_cents"; scale?: number }
  | { fn: "date_parse"; format: string }
  | { fn: "replace"; pattern: string; replacement: string }
  | { fn: "normalize_flight_number" }
  | { fn: "normalize_icd10" }
  | { fn: "normalize_cpt" }
  | { fn: "normalize_hcpcs" }
  | { fn: "normalize_ndc11" }
  | { fn: "default"; value: unknown }
  | { fn: "phone_us" }
  | { fn: "phone_e164" }
  | { fn: "card_mask_last4" }
  | { fn: "format_ssn" }
  | { fn: "format_ein" }
  | { fn: "mask_ssn" }
  | { fn: "mask_ein" }
  | { fn: "format_iban" }
  | { fn: "format_credit_card" }
  | { fn: "format_thousands"; separator?: string }
  | { fn: "format_decimal"; places: number };

// ============================================================================
// Predicate Types
// ============================================================================

export type TemplateLiteralPart =
  | { kind: "literal"; value: string }
  | { kind: "digits"; min?: number; max?: number }
  | { kind: "ascii_letters"; min?: number; max?: number }
  | { kind: "ascii_alphanumeric"; min?: number; max?: number }
  | { kind: "uppercase"; min?: number; max?: number }
  | { kind: "lowercase"; min?: number; max?: number }
  | { kind: "hex"; min?: number; max?: number };

export type Predicate =
  // Constants
  | { type: "true" }
  | { type: "false" }
  // String predicates
  | { type: "regex"; pattern: string; flags?: string }
  | { type: "template_literal"; parts: TemplateLiteralPart[] }
  | { type: "min_len"; len: number }
  | { type: "max_len"; len: number }
  // Numeric predicates
  | { type: "range"; min?: number; max?: number }
  // Path-based predicates
  | { type: "exists"; path: string }
  | { type: "eq"; path: string; value: unknown }
  | { type: "in"; path: string; values: unknown[] }
  | { type: "required_with"; field: string; with: string }
  | { type: "required_without"; field: string; without: string }
  | { type: "exactly_one_of"; paths: string[] }
  // Cross-field predicates
  | { type: "eq_fields"; left: string; right: string }
  | { type: "gt_field"; left: string; right: string }
  | { type: "gte_field"; left: string; right: string }
  | { type: "lt_field"; left: string; right: string }
  | { type: "lte_field"; left: string; right: string }
  // Computed predicates
  | { type: "sum_equals"; paths: string[]; target: string }
  | { type: "sum_equals_value"; paths: string[]; value: number }
  // Boolean combinators
  | { type: "and"; predicates: Predicate[] }
  | { type: "or"; predicates: Predicate[] }
  | { type: "not"; predicate: Predicate }
  | { type: "implies"; if: Predicate; then: Predicate }
  // Named predicates
  | { type: "call"; name: string; args?: unknown };

export interface Constraint {
  id?: string;
  pred: Predicate;
  error: ErrorMeta;
}

// ============================================================================
// Type Schema - Flattened format matching Rust's serde output
// ============================================================================

// Base fields for schemas that support transforms/constraints
interface SchemaFields {
  transforms?: Transform[];
  constraints?: Constraint[];
  description?: string;
}

interface StringSchemaFields extends SchemaFields {
  example?: string;
}

export interface RenderMetadata {
  type: string;
  page?: number;
  x: number;
  y: number;
  font_size?: number;
  font?: string;
  color?: string;
  align?: string;
  v_align?: string;
  h_scale?: number;
  width?: number;
  height?: number;
  max_width?: number;
  multiline?: boolean;
  line_height?: number;
  box_number?: string;
}

export interface AcroFormMetadata {
  field_id: string;
  field_type: string;
  copy_suffix?: string;
}

export interface PageSchema {
  name?: string;
  fields?: Record<string, RenderMetadata>;
}

export interface AcroFormCompositionMetadata {
  field_id: string;
  page: number;
  copy?: string;
  compose?: string[];
  separator?: string;
  format?: string;
}

// Property in an object - flattened TypeSchema with required flag
// Rust uses #[serde(flatten)] so the type fields are merged with required
export type PropertySchema = TypeSchema & {
  required?: boolean; // defaults to true in Rust
  description?: string;
  label?: string;
  render?: RenderMetadata;
  acroform?: AcroFormMetadata;
  section?: string;
};

export type UnknownKeysBehavior = "strict" | "passthrough" | "strip";

// Object schema - properties are PropertySchema (flattened)
export interface ObjectSchemaFields {
  properties?: Record<string, PropertySchema>;
  fields?: Record<string, PropertySchema>; // legacy alias accepted from older Rust IR
  pages?: Record<string, PageSchema>;
  acroform_mappings?: AcroFormCompositionMetadata[];
  additional_properties?: boolean; // Rust uses snake_case
  unknown_keys?: UnknownKeysBehavior;
  catchall?: TypeSchema;
  rules?: Constraint[];
  description?: string;
}

// Array schema fields
export interface ArraySchemaFields {
  items: TypeSchema;
  min_items?: number; // Rust uses snake_case
  max_items?: number;
  constraints?: Constraint[];
  description?: string;
}

// Enum schema fields
export interface EnumSchemaFields {
  values: unknown[]; // Rust supports any JSON values, not just strings
  description?: string;
}

// Literal schema fields
export interface LiteralSchemaFields {
  value: unknown;
  description?: string;
}

// Tuple schema fields
export interface TupleSchemaFields {
  items: TypeSchema[];
  description?: string;
}

// Union schema fields
export interface UnionSchemaFields {
  oneOf: TypeSchema[]; // Rust uses #[serde(rename = "oneOf")]
  discriminator?: string;
  description?: string;
}

// Intersection schema fields
export interface IntersectionSchemaFields {
  allOf: TypeSchema[]; // Rust uses #[serde(rename = "allOf")]
  description?: string;
}

// Record schema fields
export interface RecordSchemaFields {
  value: TypeSchema;
  key?: TypeSchema;
  partial?: boolean;
  description?: string;
}

// Preprocess wrapper schema fields
export interface PreprocessSchemaFields {
  transforms?: Transform[];
  schema: TypeSchema;
  description?: string;
}

// Catch wrapper schema fields
export interface CatchSchemaFields {
  schema: TypeSchema;
  value: unknown;
  description?: string;
}

// Percentage format type
export type PercentageFormat = "decimal" | "whole";

// Decimal schema fields
export interface DecimalSchemaFields extends SchemaFields {
  precision?: number; // Total digits
  scale?: number; // Decimal places
}

// Percentage schema fields
export interface PercentageSchemaFields extends SchemaFields {
  format?: PercentageFormat; // "decimal" (0-1) or "whole" (0-100)
  allow_over_100?: boolean;
  scale?: number; // Decimal places
}

// Currency schema fields
export interface CurrencySchemaFields extends SchemaFields {
  code?: string; // ISO 4217 currency code (e.g., "USD", "EUR", "GBP")
  scale?: number; // Decimal places (default 2, varies by currency: JPY=0, BHD=3)
}

// TypeSchema - discriminated union with flattened schema fields
// Rust uses #[serde(tag = "type", rename_all = "snake_case")]
export type TypeSchema =
  // Primitive types
  | ({ type: "string" } & StringSchemaFields)
  | ({ type: "number" } & SchemaFields)
  | ({ type: "integer" } & SchemaFields)
  // Specific integer types
  | ({ type: "int32" } & SchemaFields)
  | ({ type: "int64" } & SchemaFields)
  | ({ type: "uint32" } & SchemaFields)
  | ({ type: "uint64" } & SchemaFields)
  // Boolean
  | ({ type: "boolean" } & { description?: string })
  // Domain-specific numeric types
  | ({ type: "money"; scale?: number } & SchemaFields)
  | ({ type: "currency" } & CurrencySchemaFields)
  | ({ type: "decimal" } & DecimalSchemaFields)
  | ({ type: "percentage" } & PercentageSchemaFields)
  // Date
  | ({ type: "date"; format?: string } & SchemaFields)
  // Composite types
  | ({ type: "object" } & ObjectSchemaFields)
  | ({ type: "array" } & ArraySchemaFields)
  | ({ type: "tuple" } & TupleSchemaFields)
  | ({ type: "enum" } & EnumSchemaFields)
  | ({ type: "literal" } & LiteralSchemaFields)
  | ({ type: "never" } & { description?: string })
  | ({ type: "union" } & UnionSchemaFields)
  | ({ type: "intersection" } & IntersectionSchemaFields)
  | ({ type: "record" } & RecordSchemaFields)
  | ({ type: "preprocess" } & PreprocessSchemaFields)
  | ({ type: "catch" } & CatchSchemaFields)
  // Reference
  | { type: "ref"; $ref: string } // Rust uses #[serde(rename = "$ref")]
  // Any
  | ({ type: "any" } & { description?: string });

// ============================================================================
// Schema (Top-level)
// ============================================================================

export interface TypeDef {
  name: string;
  schema: TypeSchema;
}

export interface IrsFormMetadata {
  name: string;
  title: string;
  revision?: string;
  revision_date?: string;
  omb_number?: string;
  cat_number?: string;
}

export interface PdfTemplate {
  hash?: string;
  filename?: string;
  path?: string;
  source_uri?: string;
}

export interface ImportConfig {
  enabled?: boolean;
  max_rows?: number;
  max_file_size?: number;
  column_mappings?: Record<string, string>;
  required_columns?: string[];
  description?: string;
}

export interface SectionDefinition {
  title: string;
  order: number;
  description?: string;
}

export interface Schema {
  version: string;
  id?: string;
  title?: string;
  description?: string;
  irs_form?: IrsFormMetadata;
  pdf_template?: PdfTemplate;
  import?: ImportConfig;
  definitions?: Record<string, TypeSchema>; // Rust uses a map, not array
  sections?: Record<string, SectionDefinition>;
  root: TypeSchema;
}

//! wellformed
//!
//! A portable FormSpec IR with Zod-like authoring for defining form schemas,
//! transforms, constraints, and error metadata.
//!
//! ## Overview
//!
//! wellformed provides a declarative, portable way to define form validation schemas
//! that can be:
//! - Serialized to JSON for cross-language interop
//! - Executed in the Rust runtime
//! - Used to generate code for other languages (TypeScript, OpenAPI, etc.)
//!
//! ## Key Features
//!
//! - **Type Layer**: Define structure with primitives, objects, arrays, enums
//! - **Transform Layer**: Normalize data before validation (trim, digits_only, etc.)
//! - **Constraint Layer**: Portable predicate AST for validation rules
//! - **Error Layer**: Structured, stable error codes with help text
//!
//! ## Optional Features
//!
//! - **`address`**: Enables address parsing predicates through the `postal`
//!   crate. This feature requires the native libpostal C library, headers, and
//!   parser data to be installed on the target system.
//!
//! ## Example
//!
//! ```rust
//! use wellformed::ir::*;
//! use wellformed::runtime::validate;
//! use serde_json::json;
//!
//! // Define a schema
//! let schema = Schema::new(
//!     "1.0.0",
//!     TypeSchema::Object(
//!         ObjectSchema::new()
//!             .property(
//!                 "tin",
//!                 TypeSchema::String(
//!                     StringSchema::new()
//!                         .transform(Transform::digits_only())
//!                         .constraint(Constraint::new(
//!                             Predicate::regex(r"^\d{9}$"),
//!                             ErrorMeta::new("TIN_INVALID", "TIN must be 9 digits"),
//!                         )),
//!                 ),
//!             )
//!             .property("name", TypeSchema::string()),
//!     ),
//! );
//!
//! // Validate a value
//! let mut value = json!({
//!     "tin": "123-45-6789",
//!     "name": "Alice"
//! });
//!
//! let result = validate(&schema, &mut value).unwrap();
//! assert!(result.is_valid());
//! assert_eq!(value["tin"], json!("123456789")); // Transformed
//! ```
//!
//! See <https://wellformed.net/docs/rust-runtime> for Rust runtime usage.

pub mod codegen;
pub mod error;
pub mod form;
#[cfg(test)]
mod interop_test;
pub mod ir;
pub mod path;
pub mod runtime;

// Re-export commonly used types at the crate root
pub use error::{Result, WelError};
pub use form::{
    ClientFormSpec, EmbeddedSchema, FieldKind, FieldSpec, FieldState, FormErrors, FormState,
};
pub use ir::{
    Constraint, ErrorMeta, ErrorSeverity, FormError, Predicate, Schema, Transform, TypeSchema,
};
pub use path::{JsonPointer, Segment};
pub use runtime::{validate, validate_with_registry, ValidationResult};

// Address feature re-exports
#[cfg(feature = "address")]
pub use runtime::register_address_predicates;

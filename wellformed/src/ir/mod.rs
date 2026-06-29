//! IR (Intermediate Representation) types for form schemas.
//!
//! This module defines the portable, declarative IR that can be:
//! - Serialized to JSON for cross-language interop
//! - Evaluated by the Rust runtime
//! - Used to generate code for other languages

mod error;
mod predicate;
mod schema;
mod transform;
mod types;

pub use error::{ErrorMeta, ErrorSeverity, FormError};
pub use predicate::{Constraint, Predicate, TemplateLiteralPart};
pub use schema::{ImportConfig, IrsFormMetadata, PdfTemplate, Schema, SchemaMeta};
pub use transform::Transform;
pub use types::{
    AcroFormFieldComposition, AcroFormMetadata, AnySchema, ArraySchema, BooleanSchema, CatchSchema,
    CurrencySchema, DateSchema, DecimalSchema, EnumSchema, Int32Schema, Int64Schema, IntegerSchema,
    IntersectionSchema, LiteralSchema, MoneySchema, NeverSchema, NumberSchema, ObjectSchema,
    PageSchema, PercentageFormat, PercentageSchema, PreprocessSchema, PropertySchema, RecordSchema,
    RenderMetadata, StringSchema, TupleSchema, TypeSchema, Uint32Schema, Uint64Schema, UnionSchema,
    UnknownKeysBehavior,
};

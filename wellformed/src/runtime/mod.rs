//! Runtime for executing wellformed schemas.
//!
//! This module provides the runtime components for:
//! - Applying transforms to normalize data
//! - Evaluating predicates against values
//! - Validating values against schemas

pub mod predicate;
pub mod transform;
pub mod validate;

pub use predicate::{EvalContext, NamedPredicate, PredicateRegistry};
pub use transform::{apply_transform, apply_transforms};
pub use validate::{validate, validate_with_registry, ValidationResult, Validator};

#[cfg(feature = "address")]
pub use predicate::register_address_predicates;

//! Code generation from wellformed schemas.
//!
//! This module provides utilities for generating Rust code from wellformed IR schemas.
//! It is primarily used by the `wellformed_macros` crate to implement the
//! `wellformed!`, `form_schema!`, and `wel_schema!` proc macros.
//!
//! # Example
//!
//! ```ignore
//! use wellformed::codegen::generate_all;
//! use wellformed::codegen::CodegenOptions;
//! use wellformed::Schema;
//!
//! let schema: Schema = serde_json::from_str(json)?;
//! let generated = generate_all(&schema, json, &CodegenOptions::default());
//! println!("{}", generated.code);
//! ```

mod api;
mod api_types;
mod rust;
mod util;

pub use api::generate_api;
pub use api_types::generate_shared_types;
pub use rust::{generate_all, generate_form_module, CodegenOptions, GeneratedCode};
pub use util::{derive_struct_name, to_pascal_case, to_snake_case};

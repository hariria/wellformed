//! # wellformed_validate
//!
//! High-throughput validation primitives for wellformed schemas.
//!
//! This crate provides extremely fast validation primitives optimized for
//! high-throughput structured-data processing. Key features:
//!
//! - **SIMD TIN validation**: 1.4ns per SSN/EIN (~700M validations/sec)
//! - **Zero-allocation hot path**: Static registry, no per-call allocations
//! - **Batch-oriented API**: Process thousands of forms with cache-optimal layout
//! - **Hand-written validators**: 90x faster than regex for short strings
//!
//! ## Performance
//!
//! These figures are local benchmark results for short ASCII identifiers. Run
//! the included criterion benchmarks on your deployment hardware before using
//! them for capacity planning.
//!
//! | Validator | Latency | Throughput |
//! |-----------|---------|------------|
//! | `is_ssn_format` | 1.4 ns | 700M/sec |
//! | `is_ein_format` | 1.4 ns | 700M/sec |
//! | `is_zip_format` | 0.95 ns | 1B/sec |
//! | `validate_ssn` (full) | 2.9 ns | 345M/sec |
//! | Batch (10K forms) | 48 µs | 209M/sec |
//!
//! ## Quick Start
//!
//! ```rust
//! use wellformed_validate::{tin, patterns};
//!
//! // Ultra-fast format checks (no regex, hand-written)
//! assert!(patterns::is_ssn_format("123-45-6789"));
//! assert!(patterns::is_ein_format("12-3456789"));
//! assert!(patterns::is_zip_format("12345"));
//!
//! // Full TIN validation with IRS rules
//! assert!(tin::validate_ssn("123-45-6789"));
//! assert!(tin::validate_ein("12-3456789"));
//!
//! // Batch validation (SIMD accelerated)
//! let tins = vec!["123456789", "987654321", "111223333"];
//! let results = tin::validate_batch(&tins);
//! ```
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    wellformed_validate                         │
//! ├─────────────────────────────────────────────────────────────┤
//! │  tin.rs          │ SIMD TIN/SSN/EIN validation              │
//! │  registry.rs     │ LazyLock predicate registry              │
//! │  patterns.rs     │ Fast format checks (no regex)            │
//! │  batch.rs        │ SoA batch validation                     │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## When to Use Vectorscan
//!
//! The optional `vectorscan` feature is for scanning **large documents** (KB/MB)
//! for multiple patterns. For validating short strings like TINs, the hand-written
//! validators are 5,700x faster.

#![cfg_attr(feature = "nightly", feature(portable_simd))]

pub mod error;
pub mod registry;
pub mod tin;

pub mod patterns;

pub mod batch;

// Re-exports for convenience
pub use error::{ValidationError, ValidationResult};
pub use patterns::{is_ein_format, is_email_format, is_ssn_format, is_zip_format};
pub use registry::REGISTRY;
pub use tin::{TinKind, TinValidator};

/// Validate a TIN string, returning true if valid.
///
/// This is the fastest single-TIN validation path.
/// For bulk validation, use [`tin::validate_batch`].
#[inline]
pub fn validate_tin(tin: &str) -> bool {
    tin::validate_any(tin)
}

/// Validate an SSN string.
#[inline]
pub fn validate_ssn(ssn: &str) -> bool {
    tin::validate_ssn(ssn)
}

/// Validate an EIN string.
#[inline]
pub fn validate_ein(ein: &str) -> bool {
    tin::validate_ein(ein)
}

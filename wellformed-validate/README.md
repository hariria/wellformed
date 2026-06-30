# wellformed-validate

High-throughput validation primitives for wellformed schemas.

## Performance

These figures come from local criterion benchmarks on short ASCII identifiers. Treat them as relative guidance and run the benchmarks on your deployment hardware before making capacity decisions.

| Validator | Latency | Throughput |
|-----------|---------|------------|
| `is_ssn_format` | 1.4 ns | 700M/sec |
| `is_ein_format` | 1.4 ns | 700M/sec |
| `is_zip_format` | 0.95 ns | 1B/sec |
| `validate_ssn` (full IRS rules) | 2.9 ns | 345M/sec |
| Batch validation (10K forms) | 48 µs | 209M/sec |

## Quick Start

```rust
use wellformed_validate::{tin, patterns};

// Ultra-fast format checks (no regex, hand-written)
assert!(patterns::is_ssn_format("123-45-6789"));
assert!(patterns::is_ein_format("12-3456789"));
assert!(patterns::is_zip_format("12345"));

// Full TIN validation with IRS rules
assert!(tin::validate_ssn("123-45-6789"));
assert!(tin::validate_ein("12-3456789"));

// Batch validation
let tins = vec!["123456789", "987654321", "111223333"];
let results = tin::validate_batch(&tins);
```

## Architecture

```
wellformed-validate/
├── src/
│   ├── tin.rs       # SIMD TIN/SSN/EIN validation
│   ├── registry.rs  # LazyLock predicate registry
│   ├── patterns.rs  # Fast format checks (no regex)
│   ├── batch.rs     # SoA batch validation
│   └── error.rs     # Error types
└── benches/
    ├── tin_validation.rs
    ├── batch_validation.rs
    └── pattern_matching.rs
```

## Design Decisions

### Why Hand-Written Validators Beat Regex

For short string validation (TINs, ZIPs, emails), local benchmarks show hand-written byte-level checks outperform regex:

| Method | SSN Validation |
|--------|---------------|
| Hand-written (`is_ssn_format`) | 1.4 ns |
| Regex (`Regex::is_match`) | 132 ns |

The hand-written validators:
- No regex compilation or state machine overhead
- Direct byte comparisons with early exit
- Fully inlined by the compiler
- Zero allocations

### Why NOT Vectorscan for Short Strings

Vectorscan (Intel Hyperscan fork) is designed for scanning **large documents** at network speeds. For short strings, the scanner setup overhead dominates:

| Method | SSN Validation | Relative |
|--------|---------------|----------|
| Hand-written | 1.4 ns | 1x |
| Regex | 132 ns | 94x slower |
| Vectorscan | 8,000 ns | 5,714x slower |

Use Vectorscan when scanning:
- Scanning KB/MB of text for many patterns simultaneously
- Network traffic inspection
- Log file analysis
- Document classification

Prefer the built-in validators instead of Vectorscan for:
- Validating individual form fields
- Short string pattern matching
- High-frequency validation loops

### LazyLock Registry

The predicate registry uses `LazyLock` for zero-cost access after initialization:

```rust
pub static REGISTRY: LazyLock<PredicateRegistry> = LazyLock::new(PredicateRegistry::new);
```

This ensures:
- No per-call allocation
- Thread-safe initialization
- Compile-time known predicates

### Structure of Arrays (SoA) for Batch Processing

The `FormBatch` type uses SoA layout for cache-friendly bulk validation:

```rust
pub struct FormBatch {
    pub payer_tins: TinBuffer,      // All payer TINs contiguous
    pub recipient_tins: TinBuffer,  // All recipient TINs contiguous
    pub interest_income: AmountBuffer,
    pub tax_years: Vec<u16>,
}
```

Benefits:
- Sequential memory access patterns
- Better CPU cache utilization
- Enables SIMD processing of homogeneous data

## Features

| Feature | Description |
|---------|-------------|
| `simd` (default) | Enable SIMD optimizations |
| `vectorscan` | Enable Vectorscan for large document scanning |
| `rayon` | Enable parallel batch processing |
| `nightly` | Enable nightly SIMD features |

## Benchmarks

Run benchmarks:

```bash
# TIN validation benchmarks
cargo bench --package wellformed-validate --bench tin_validation

# Batch validation benchmarks
cargo bench --package wellformed-validate --bench batch_validation

# Pattern matching comparison (regex vs vectorscan)
cargo bench --package wellformed-validate --bench pattern_matching
cargo bench --package wellformed-validate --bench pattern_matching --features vectorscan
```

## TIN Validation Rules

### SSN (Social Security Number)
- Format: `XXX-XX-XXXX` or `XXXXXXXXX`
- Area number (first 3 digits): 001-899, excluding 666
- Group number (middle 2 digits): 01-99
- Serial number (last 4 digits): 0001-9999

### EIN (Employer Identification Number)
- Format: `XX-XXXXXXX` or `XXXXXXXXX`
- Campus code (first 2 digits): Valid IRS campus codes
- Validated against IRS campus code lookup table

### ITIN (Individual Taxpayer Identification Number)
- Format: `9XX-XX-XXXX`
- First digit must be 9
- Fourth and fifth digits: 50-65, 70-88, 90-92, 94-99

### ATIN (Adoption Taxpayer Identification Number)
- Format: `9XX-XX-XXXX`
- First digit must be 9
- Fourth and fifth digits: 93

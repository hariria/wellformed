//! Predicate evaluation and named predicate registry.
//!
//! This module implements the runtime evaluation of predicate ASTs.
//!
//! ## Built-in Predicates
//!
//! The following named predicates are available via `PredicateRegistry::with_builtins()`:
//!
//! ### TIN Validation
//! - `is_tin` - Validate any TIN (SSN, EIN, ITIN, ATIN)
//! - `is_ssn` - Validate Social Security Number
//! - `is_ein` - Validate Employer Identification Number
//! - `is_itin` - Validate Individual Taxpayer Identification Number
//! - `is_atin` - Validate Adoption Taxpayer Identification Number
//! - `luhn` - Validate Luhn checksum
//!
//! ### Financial Identifiers
//! - `is_cusip` - Validate CUSIP security identifier
//! - `is_aba_routing` - Validate ABA routing number
//! - `is_mcc` - Validate Merchant Category Code
//! - `is_account_number` - Validate account number format
//!
//! ### Product / Ecommerce
//! - `is_vin` - Validate Vehicle Identification Number (17 chars, check digit)
//! - `is_upc` - Validate UPC-A barcode (12 digits, check digit)
//! - `is_ean` - Validate EAN-13 barcode (13 digits, check digit)
//! - `is_isbn` - Validate ISBN-10 or ISBN-13
//!
//! ### Payment Cards
//! - `is_credit_card` - Validate credit card number (Luhn + IIN prefix/length)
//! - `is_cvv` - Validate CVV code (3 or 4 digits)
//! - `is_card_expiry` - Validate card expiry date (MM/YY or MM/YYYY)
//!
//! ### International Banking
//! - `is_iban` - Validate IBAN (mod-97 checksum + country-specific length)
//! - `is_bic` - Validate BIC/SWIFT code (8 or 11 chars)
//! - `is_swift` - Alias for `is_bic`
//!
//! ### Date & Time Validation
//! - `is_date` - Validate date format (MM/DD/YYYY, YYYY-MM-DD, etc.)
//! - `is_time` - Validate time format (HH:MM, HH:MM:SS, 12h/24h)
//! - `is_iso_datetime` - Validate ISO 8601 datetime
//! - `is_tax_year` - Validate tax year (2020-2100)
//! - `date_in_range` - Validate date within range
//! - `date_before` - Validate date is before another
//! - `date_after` - Validate date is after another
//! - `time_before` - Validate time is before another
//! - `time_after` - Validate time is after another
//! - `time_in_range` - Validate time within range (supports overnight wrapping)
//!
//! ### Amount Validation
//! - `is_non_negative` - Validate non-negative number
//! - `is_positive` - Validate positive number
//! - `is_negative` - Validate negative number
//! - `is_non_positive` - Validate non-positive number
//! - `is_percentage` - Validate percentage (0-100 or 0-1)
//! - `is_money_format` - Validate money string format
//! - `is_multiple_of` - Validate value is divisible by a provided step
//! - `less_than_or_equal` - Compare numeric values
//! - `greater_than_or_equal` - Compare numeric values
//!
//! ### Reference Data
//! - `is_country_code` - Validate ISO 3166-1 alpha-2 country code
//! - `is_currency_code` - Validate ISO 4217 currency code
//! - `is_w2_box12_code` - Validate W-2 Box 12 code
//! - `is_1099b_code` - Validate 1099-B transaction code
//! - `is_filing_status` - Validate tax filing status
//!
//! ### Color Validation
//! - `is_hex_color` - Validate hex color (#RGB, #RRGGBB, with optional alpha)
//! - `is_rgb_color` - Validate RGB/RGBA color string
//! - `is_hsl_color` - Validate HSL/HSLA color string
//!
//! ### Decimal Validation
//! - `is_decimal_places` - Validate exact or max number of decimal places
//!
//! ### Insurance / Healthcare
//! - `is_npi` - Validate National Provider Identifier (10 digits, Luhn with 80840 prefix)
//! - `is_dea_number` - Validate DEA registration number (2 letters + 7 digits, check digit)
//! - `is_icd10_code` - Validate ICD-10 diagnosis code
//! - `is_cpt_code` - Validate CPT procedure code
//! - `is_hcpcs_code` - Validate HCPCS Level II code
//! - `is_ndc_code` - Validate NDC code (10-digit or 11-digit)
//!
//! ### Numeric Types
//! - `is_integer` - Validate value is a whole number
//! - `is_float` - Validate value is a valid floating point number
//! - `is_u8` - Validate value fits in u8 (0 to 255)
//! - `is_u16` - Validate value fits in u16 (0 to 65535)
//! - `is_u32` - Validate value fits in u32 (0 to 4294967295)
//! - `is_u64` - Validate value fits in u64 (0 to 18446744073709551615)
//! - `is_i8` - Validate value fits in i8 (-128 to 127)
//! - `is_i16` - Validate value fits in i16 (-32768 to 32767)
//! - `is_i32` - Validate value fits in i32 (-2147483648 to 2147483647)
//! - `is_i64` - Validate value fits in i64
//!
//! ### Encoding / Crypto
//! - `is_base58` - Validate base58-encoded string (Bitcoin alphabet)
//! - `is_base64` - Validate base64-encoded string
//! - `is_bitcoin_address` - Validate Bitcoin address (P2PKH, P2SH, Bech32, Taproot)
//! - `is_ethereum_address` - Validate Ethereum address (0x + 40 hex)
//! - `is_solana_address` - Validate Solana address (base58, 32-44 chars)
//! - `is_jwt` - Validate compact JWT structure
//! - `is_hash` - Validate hexadecimal digest hash
//!
//! ### Text Analysis
//! - `is_rtl` - Detect RTL (right-to-left) text (Arabic, Hebrew, etc.)
//! - `is_ltr` - Validate text is exclusively LTR (no RTL characters)
//! - `starts_with` - Validate string prefix
//! - `ends_with` - Validate string suffix
//! - `contains` - Validate string contains a substring
//! - `is_alpha` - Validate ASCII letters only (A-Za-z)
//! - `is_digits` - Validate ASCII digits only (0-9)
//! - `is_alphanumeric` - Validate ASCII letters and digits only
//! - `is_alpha_spaces` - Validate ASCII letters and spaces only (at least one letter)
//! - `is_alphanumeric_spaces` - Validate ASCII letters, digits, and spaces only (at least one letter or digit)
//! - `is_name_chars` - Validate ASCII letters, hyphens, and apostrophes only (at least one letter)
//! - `is_uppercase` - Validate uppercase ASCII letters only (A-Z)
//! - `is_lowercase` - Validate lowercase ASCII letters only (a-z)
//! - `is_title_case` - Validate title case (uppercase letter followed by lowercase letters)
//!
//! ### Contact Information
//! - `phone_number` - Validate phone number (US or international)
//! - `phone_number_us` - Validate US phone number only (rejects international prefix)
//! - `is_phone` - Backwards-compat alias for `phone_number`
//! - `is_email` - Validate email address
//! - `is_url` - Validate URL
//! - `is_uuid` - Validate UUID (any version or specific version)
//! - `is_ip` - Validate IPv4 or IPv6 address
//! - `is_cidr` - Validate CIDR block
//! - `is_mac_address` - Validate MAC address
//! - `is_street_address` - Validate street address (starts with digit, contains letters)
//!
//! ### Aviation / Travel
//! - `is_iata_airport_code` - Validate IATA airport code (3 letters, e.g., SFO)
//! - `is_icao_airport_code` - Validate ICAO airport code (4 letters, e.g., KSFO)
//! - `is_airport_code` - Validate airport code with selectable system (IATA/ICAO/ANY)
//! - `is_iata_airline_code` - Validate IATA airline code (2 chars, e.g., UA)
//! - `is_icao_airline_code` - Validate ICAO airline code (3 letters, e.g., UAL)
//! - `is_airline_code` - Validate airline code with selectable system (IATA/ICAO/ANY)
//! - `is_flight_number` - Validate flight number (e.g., UA123, UAL1234A)
//!
//! ### Address (requires "address" feature)
//! - `is_parseable_address` - Validate parseable address
//! - `has_address_component` - Check for address component
//! - `is_us_address` - Validate US address
//! - `is_us_zip` - Validate US ZIP code
//! - `is_us_state` - Validate US state code

mod amount;
mod aviation;
mod color;
mod contact;
mod crypto;
mod date;
mod financial;
mod insurance;
mod numeric;
mod reference;
mod registry;
mod text;
mod tin;

#[cfg(feature = "address")]
mod address;

use crate::error::{Result, WelError};
use crate::ir::{Predicate, TemplateLiteralPart};
use crate::path::JsonPointer;
use crate::runtime::validate::json_value_eq;
use memchr::memchr;
use regex::Regex;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{LazyLock, RwLock};

pub use registry::{NamedPredicate, PredicateRegistry};

// ============================================================================
// Static Registry (initialized once at startup)
// ============================================================================

/// Global predicate registry, initialized once at startup.
///
/// This avoids the cost of creating a new registry for every validation call.
/// The LazyLock ensures thread-safe, one-time initialization.
pub static REGISTRY: LazyLock<PredicateRegistry> = LazyLock::new(PredicateRegistry::with_builtins);

/// Global regex cache for compiled patterns.
///
/// Regex compilation is expensive (~20-50µs per pattern). By caching compiled
/// regexes globally, we avoid recompiling the same patterns across validations.
/// This provides ~100x speedup for repeated validations with the same patterns.
static REGEX_CACHE: LazyLock<RwLock<HashMap<String, Regex>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

// Re-export registration functions for custom registry building
pub use amount::register_amount_predicates;
pub use aviation::register_aviation_predicates;
pub use color::register_color_predicates;
pub use contact::register_contact_predicates;
pub use crypto::register_crypto_predicates;
pub use date::register_date_predicates;
pub use financial::register_financial_predicates;
pub use insurance::register_insurance_predicates;
pub use numeric::register_numeric_predicates;
pub use reference::register_reference_predicates;
pub use text::register_text_predicates;
pub use tin::register_tin_predicates;

#[cfg(feature = "address")]
pub use address::register_address_predicates;

// ============================================================================
// PredicateRegistry Builder
// ============================================================================

impl PredicateRegistry {
    /// Create a registry with built-in predicates.
    ///
    /// This includes all predicates for:
    /// - TIN validation (SSN, EIN, ITIN, ATIN, Luhn)
    /// - Financial identifiers (CUSIP, ABA routing, MCC)
    /// - Date validation
    /// - Amount/money validation
    /// - Reference data (country codes, form codes)
    /// - Contact information (phone, email, URL)
    /// - Address validation (when "address" feature is enabled)
    pub fn with_builtins() -> Self {
        let mut registry = Self::new();

        // Core TIN predicates
        register_tin_predicates(&mut registry);

        // Financial identifiers
        register_financial_predicates(&mut registry);

        // Date predicates
        register_date_predicates(&mut registry);

        // Amount/money predicates
        register_amount_predicates(&mut registry);

        // Reference data predicates
        register_reference_predicates(&mut registry);

        // Aviation predicates
        register_aviation_predicates(&mut registry);

        // Color predicates
        register_color_predicates(&mut registry);

        // Insurance predicates
        register_insurance_predicates(&mut registry);

        // Numeric type predicates
        register_numeric_predicates(&mut registry);

        // Encoding/crypto predicates
        register_crypto_predicates(&mut registry);

        // Text predicates
        register_text_predicates(&mut registry);

        // Contact predicates
        register_contact_predicates(&mut registry);

        // Address predicates (optional feature)
        #[cfg(feature = "address")]
        register_address_predicates(&mut registry);

        registry
    }
}

// ============================================================================
// Evaluation Context
// ============================================================================

/// Maximum predicate-tree recursion depth. Bounds the boolean combinators
/// (`and`/`or`/`not`/`implies`) so a deeply nested predicate returns a clean
/// error instead of overflowing the stack (a DoS vector).
const MAX_PREDICATE_DEPTH: usize = 128;

/// Context for predicate evaluation.
pub struct EvalContext<'a> {
    /// Registry of named predicates.
    pub registry: &'a PredicateRegistry,
    /// Current predicate-tree recursion depth.
    depth: usize,
}

impl<'a> EvalContext<'a> {
    /// Create a new evaluation context.
    pub fn new(registry: &'a PredicateRegistry) -> Self {
        Self { registry, depth: 0 }
    }
}

/// Get or compile a regex pattern from the global cache.
///
/// Uses a read-write lock for concurrent access. Most calls will be
/// cache hits (read lock), with occasional misses (write lock).
fn get_cached_regex(pattern: &str, flags: Option<&str>) -> Result<Regex> {
    let key = format!("{}:{}", pattern, flags.unwrap_or(""));

    // Fast path: check read lock first
    {
        let cache = REGEX_CACHE.read().unwrap();
        if let Some(regex) = cache.get(&key) {
            return Ok(regex.clone());
        }
    }

    // Slow path: compile and insert with write lock
    let regex = compile_regex(pattern, flags)?;
    {
        let mut cache = REGEX_CACHE.write().unwrap();
        // Double-check in case another thread inserted while we were compiling
        cache.entry(key).or_insert_with(|| regex.clone());
    }
    Ok(regex)
}

/// Compile a regex pattern with optional flags.
fn compile_regex(pattern: &str, flags: Option<&str>) -> Result<Regex> {
    let mut builder = regex::RegexBuilder::new(pattern);

    if let Some(flags) = flags {
        for flag in flags.chars() {
            match flag {
                'i' => {
                    builder.case_insensitive(true);
                }
                'm' => {
                    builder.multi_line(true);
                }
                's' => {
                    builder.dot_matches_new_line(true);
                }
                _ => {} // Ignore unknown flags
            }
        }
    }

    builder.build().map_err(WelError::InvalidRegex)
}

// ============================================================================
// Predicate Evaluation
// ============================================================================

/// Evaluate a predicate against a value.
///
/// The value is the "current" value being validated (at the constraint's scope).
/// Path-based predicates use relative JSON Pointer paths from this value.
pub fn evaluate(pred: &Predicate, value: &Value, ctx: &mut EvalContext) -> Result<bool> {
    ctx.depth += 1;
    if ctx.depth > MAX_PREDICATE_DEPTH {
        ctx.depth -= 1;
        return Err(WelError::RecursionLimit("predicate".to_string()));
    }
    let result = evaluate_inner(pred, value, ctx);
    ctx.depth -= 1;
    result
}

fn evaluate_inner(pred: &Predicate, value: &Value, ctx: &mut EvalContext) -> Result<bool> {
    match pred {
        Predicate::True => Ok(true),
        Predicate::False => Ok(false),

        // String predicates
        Predicate::Regex { pattern, flags } => {
            let regex = get_cached_regex(pattern, flags.as_deref())?;
            Ok(match value {
                Value::String(s) => regex.is_match(s),
                // Non-strings pass regex constraints (constraint not applicable)
                _ => true,
            })
        }

        Predicate::TemplateLiteral { parts } => Ok(match value {
            Value::String(s) => matches_template_literal(s.as_bytes(), parts),
            // Non-strings pass string constraints (constraint not applicable)
            _ => true,
        }),

        Predicate::MinLen { len } => Ok(get_length(value).is_some_and(|l| l >= *len)),

        Predicate::MaxLen { len } => Ok(get_length(value).is_some_and(|l| l <= *len)),

        // Numeric predicates
        Predicate::Range { min, max } => {
            let n = value.as_f64();
            Ok(n.is_some_and(|n| {
                let above_min = min.is_none_or(|m| n >= m);
                let below_max = max.is_none_or(|m| n <= m);
                above_min && below_max
            }))
        }

        // Path-based predicates
        Predicate::Exists { path } => {
            let ptr = JsonPointer::parse(path)?;
            let results = ptr.resolve(value);
            Ok(!results.is_empty() && results.iter().all(|v| !v.is_null()))
        }

        Predicate::Eq {
            path,
            value: expected,
        } => {
            let ptr = JsonPointer::parse(path)?;
            let results = ptr.resolve(value);
            Ok(!results.is_empty() && results.iter().all(|v| json_value_eq(v, expected)))
        }

        Predicate::In { path, values } => {
            let ptr = JsonPointer::parse(path)?;
            let results = ptr.resolve(value);
            Ok(!results.is_empty()
                && results
                    .iter()
                    .all(|v| values.iter().any(|x| json_value_eq(x, v))))
        }

        Predicate::RequiredWith { field, with } => {
            let with_exists = path_exists(value, with)?;
            let field_exists = path_exists(value, field)?;
            Ok(!with_exists || field_exists)
        }

        Predicate::RequiredWithout { field, without } => {
            let without_exists = path_exists(value, without)?;
            let field_exists = path_exists(value, field)?;
            Ok(without_exists || field_exists)
        }

        Predicate::ExactlyOneOf { paths } => {
            let mut present = 0usize;
            for path in paths {
                if path_exists(value, path)? {
                    present += 1;
                    if present > 1 {
                        return Ok(false);
                    }
                }
            }
            Ok(present == 1)
        }

        // Cross-field predicates
        Predicate::EqFields { left, right } => {
            let left_ptr = JsonPointer::parse(left)?;
            let right_ptr = JsonPointer::parse(right)?;
            let left_vals = left_ptr.resolve(value);
            let right_vals = right_ptr.resolve(value);
            // Both must exist and be equal
            Ok(!left_vals.is_empty()
                && !right_vals.is_empty()
                && left_vals.len() == right_vals.len()
                && left_vals
                    .iter()
                    .zip(right_vals.iter())
                    .all(|(l, r)| json_value_eq(l, r)))
        }

        Predicate::GtField { left, right } => compare_fields(value, left, right, |l, r| l > r),

        Predicate::GteField { left, right } => compare_fields(value, left, right, |l, r| l >= r),

        Predicate::LtField { left, right } => compare_fields(value, left, right, |l, r| l < r),

        Predicate::LteField { left, right } => compare_fields(value, left, right, |l, r| l <= r),

        Predicate::SumEquals { paths, target } => {
            let target_ptr = JsonPointer::parse(target)?;
            let target_vals = target_ptr.resolve(value);
            if target_vals.is_empty() {
                return Ok(false);
            }
            let target_val = target_vals[0].as_f64();
            if target_val.is_none() {
                return Ok(false);
            }

            let mut sum = 0.0;
            for path in paths {
                let ptr = JsonPointer::parse(path)?;
                let vals = ptr.resolve(value);
                if vals.is_empty() {
                    return Ok(false);
                }
                if let Some(n) = vals[0].as_f64() {
                    sum += n;
                } else {
                    return Ok(false);
                }
            }

            // Use epsilon comparison for floating point
            Ok((sum - target_val.unwrap()).abs() < 1e-10)
        }

        Predicate::SumEqualsValue {
            paths,
            value: expected,
        } => {
            let mut sum = 0.0;
            for path in paths {
                let ptr = JsonPointer::parse(path)?;
                let vals = ptr.resolve(value);
                if vals.is_empty() {
                    return Ok(false);
                }
                if let Some(n) = vals[0].as_f64() {
                    sum += n;
                } else {
                    return Ok(false);
                }
            }

            // Use epsilon comparison for floating point
            Ok((sum - expected).abs() < 1e-10)
        }

        // Boolean combinators
        Predicate::And { predicates } => {
            for p in predicates {
                if !evaluate(p, value, ctx)? {
                    return Ok(false);
                }
            }
            Ok(true)
        }

        Predicate::Or { predicates } => {
            for p in predicates {
                if evaluate(p, value, ctx)? {
                    return Ok(true);
                }
            }
            Ok(false)
        }

        Predicate::Not { predicate } => {
            let result = evaluate(predicate, value, ctx)?;
            Ok(!result)
        }

        Predicate::Implies {
            antecedent,
            consequent,
        } => {
            // P => Q is equivalent to !P || Q
            let p = evaluate(antecedent, value, ctx)?;
            if !p {
                return Ok(true); // Antecedent false, implication is true
            }
            evaluate(consequent, value, ctx)
        }

        // Named predicates
        Predicate::Call { name, args } => {
            let predicate = ctx
                .registry
                .get(name)
                .ok_or_else(|| WelError::UnknownPredicate(name.clone()))?;
            Ok(predicate.evaluate(value, args))
        }
    }
}

fn matches_template_literal(input: &[u8], parts: &[TemplateLiteralPart]) -> bool {
    matches_template_from(input, 0, parts, 0)
}

fn matches_template_from(
    input: &[u8],
    input_pos: usize,
    parts: &[TemplateLiteralPart],
    part_pos: usize,
) -> bool {
    if part_pos == parts.len() {
        return input_pos == input.len();
    }

    let Some(part) = parts.get(part_pos) else {
        return false;
    };

    match part {
        TemplateLiteralPart::Literal { value } => {
            let literal = value.as_bytes();
            if input[input_pos..].starts_with(literal) {
                matches_template_from(input, input_pos + literal.len(), parts, part_pos + 1)
            } else {
                false
            }
        }
        _ => match_template_segment(input, input_pos, parts, part_pos),
    }
}

fn match_template_segment(
    input: &[u8],
    input_pos: usize,
    parts: &[TemplateLiteralPart],
    part_pos: usize,
) -> bool {
    let Some(part) = parts.get(part_pos) else {
        return false;
    };

    let Some((min_len, max_len)) = template_segment_bounds(part) else {
        return false;
    };

    let remaining = input.len().saturating_sub(input_pos);
    let max_len = max_len.min(remaining);
    if min_len > max_len {
        return false;
    }

    let run_len = template_segment_run_len(part, &input[input_pos..]);
    if run_len < min_len {
        return false;
    }
    let max_run = max_len.min(run_len);

    // Tail segment: consume the remainder exactly.
    if part_pos + 1 == parts.len() {
        let tail_len = remaining;
        return tail_len >= min_len && tail_len <= max_run;
    }

    let next_part = &parts[part_pos + 1];
    if let TemplateLiteralPart::Literal { value } = next_part {
        let next_lit = value.as_bytes();
        if !next_lit.is_empty() {
            let mut search_start = input_pos + min_len;
            while search_start <= input_pos + max_run {
                let Some(found) = find_literal_at_or_after(input, search_start, next_lit) else {
                    break;
                };
                let consumed = found - input_pos;
                if consumed > max_run {
                    break;
                }
                if matches_template_from(input, found, parts, part_pos + 1) {
                    return true;
                }
                search_start = found + 1;
            }
            return false;
        }
    }

    // No literal boundary to accelerate against: try bounded lengths (greedy).
    for consumed in (min_len..=max_run).rev() {
        if matches_template_from(input, input_pos + consumed, parts, part_pos + 1) {
            return true;
        }
    }
    false
}

fn template_segment_bounds(part: &TemplateLiteralPart) -> Option<(usize, usize)> {
    let (min, max) = match part {
        TemplateLiteralPart::Digits { min, max }
        | TemplateLiteralPart::AsciiLetters { min, max }
        | TemplateLiteralPart::AsciiAlphanumeric { min, max }
        | TemplateLiteralPart::Uppercase { min, max }
        | TemplateLiteralPart::Lowercase { min, max }
        | TemplateLiteralPart::Hex { min, max } => (min.unwrap_or(1), max.unwrap_or(usize::MAX)),
        TemplateLiteralPart::Literal { .. } => return None,
    };
    if min > max {
        None
    } else {
        Some((min, max))
    }
}

fn template_segment_run_len(part: &TemplateLiteralPart, input: &[u8]) -> usize {
    let mut len = 0usize;
    for &byte in input {
        if template_segment_matches_byte(part, byte) {
            len += 1;
        } else {
            break;
        }
    }
    len
}

fn template_segment_matches_byte(part: &TemplateLiteralPart, byte: u8) -> bool {
    match part {
        TemplateLiteralPart::Digits { .. } => byte.is_ascii_digit(),
        TemplateLiteralPart::AsciiLetters { .. } => byte.is_ascii_alphabetic(),
        TemplateLiteralPart::AsciiAlphanumeric { .. } => byte.is_ascii_alphanumeric(),
        TemplateLiteralPart::Uppercase { .. } => byte.is_ascii_uppercase(),
        TemplateLiteralPart::Lowercase { .. } => byte.is_ascii_lowercase(),
        TemplateLiteralPart::Hex { .. } => byte.is_ascii_hexdigit(),
        TemplateLiteralPart::Literal { .. } => false,
    }
}

fn find_literal_at_or_after(input: &[u8], mut at: usize, needle: &[u8]) -> Option<usize> {
    if needle.is_empty() {
        return Some(at.min(input.len()));
    }
    if at >= input.len() {
        return None;
    }

    let first = needle[0];
    loop {
        let rel = memchr(first, &input[at..])?;
        let idx = at + rel;
        if input[idx..].starts_with(needle) {
            return Some(idx);
        }
        at = idx + 1;
        if at >= input.len() {
            return None;
        }
    }
}

/// Get the length of a string or array.
fn get_length(value: &Value) -> Option<usize> {
    match value {
        Value::String(s) => Some(s.chars().count()),
        Value::Array(a) => Some(a.len()),
        _ => None,
    }
}

/// Check whether a JSON pointer resolves to at least one non-null value.
fn path_exists(value: &Value, path: &str) -> Result<bool> {
    let ptr = JsonPointer::parse(path)?;
    let results = ptr.resolve(value);
    Ok(!results.is_empty() && results.iter().all(|v| !v.is_null()))
}

/// Compare two numeric fields using a comparison function.
fn compare_fields<F>(value: &Value, left: &str, right: &str, cmp: F) -> Result<bool>
where
    F: Fn(f64, f64) -> bool,
{
    let left_ptr = JsonPointer::parse(left)?;
    let right_ptr = JsonPointer::parse(right)?;
    let left_vals = left_ptr.resolve(value);
    let right_vals = right_ptr.resolve(value);

    if left_vals.is_empty() || right_vals.is_empty() {
        return Ok(false);
    }

    let left_val = left_vals[0].as_f64();
    let right_val = right_vals[0].as_f64();

    match (left_val, right_val) {
        (Some(l), Some(r)) => Ok(cmp(l, r)),
        _ => Ok(false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_regex_match() {
        let registry = PredicateRegistry::new();
        let mut ctx = EvalContext::new(&registry);

        let pred = Predicate::regex(r"^\d{9}$");
        assert!(evaluate(&pred, &json!("123456789"), &mut ctx).unwrap());
        assert!(!evaluate(&pred, &json!("12345678"), &mut ctx).unwrap());
        assert!(!evaluate(&pred, &json!("1234567890"), &mut ctx).unwrap());
    }

    #[test]
    fn test_regex_case_insensitive() {
        let registry = PredicateRegistry::new();
        let mut ctx = EvalContext::new(&registry);

        let pred = Predicate::regex_with_flags("^hello$", "i");
        assert!(evaluate(&pred, &json!("HELLO"), &mut ctx).unwrap());
        assert!(evaluate(&pred, &json!("hello"), &mut ctx).unwrap());
        assert!(evaluate(&pred, &json!("HeLLo"), &mut ctx).unwrap());
    }

    #[test]
    fn test_template_literal_match() {
        let registry = PredicateRegistry::new();
        let mut ctx = EvalContext::new(&registry);

        let pred = Predicate::template_literal(vec![
            TemplateLiteralPart::literal("SFO-"),
            TemplateLiteralPart::digits(Some(3), Some(4)),
            TemplateLiteralPart::literal("-"),
            TemplateLiteralPart::uppercase(Some(2), Some(2)),
        ]);

        assert!(evaluate(&pred, &json!("SFO-123-AB"), &mut ctx).unwrap());
        assert!(evaluate(&pred, &json!("SFO-1234-ZZ"), &mut ctx).unwrap());
        assert!(!evaluate(&pred, &json!("SFO-12-AB"), &mut ctx).unwrap());
        assert!(!evaluate(&pred, &json!("SFO-123-abc"), &mut ctx).unwrap());
    }

    #[test]
    fn test_template_literal_backtracking_with_literal_boundary() {
        let registry = PredicateRegistry::new();
        let mut ctx = EvalContext::new(&registry);

        let pred = Predicate::template_literal(vec![
            TemplateLiteralPart::ascii_letters(Some(1), Some(3)),
            TemplateLiteralPart::literal("-"),
            TemplateLiteralPart::digits(Some(2), Some(2)),
        ]);

        assert!(evaluate(&pred, &json!("AB-12"), &mut ctx).unwrap());
        assert!(evaluate(&pred, &json!("A-12"), &mut ctx).unwrap());
        assert!(!evaluate(&pred, &json!("ABCD-12"), &mut ctx).unwrap());
    }

    #[test]
    fn test_min_max_len() {
        let registry = PredicateRegistry::new();
        let mut ctx = EvalContext::new(&registry);

        let min = Predicate::min_len(3);
        let max = Predicate::max_len(5);

        assert!(evaluate(&min, &json!("abc"), &mut ctx).unwrap());
        assert!(!evaluate(&min, &json!("ab"), &mut ctx).unwrap());

        assert!(evaluate(&max, &json!("abcde"), &mut ctx).unwrap());
        assert!(!evaluate(&max, &json!("abcdef"), &mut ctx).unwrap());
    }

    #[test]
    fn test_range() {
        let registry = PredicateRegistry::new();
        let mut ctx = EvalContext::new(&registry);

        let pred = Predicate::range(Some(0.0), Some(100.0));
        assert!(evaluate(&pred, &json!(50), &mut ctx).unwrap());
        assert!(evaluate(&pred, &json!(0), &mut ctx).unwrap());
        assert!(evaluate(&pred, &json!(100), &mut ctx).unwrap());
        assert!(!evaluate(&pred, &json!(-1), &mut ctx).unwrap());
        assert!(!evaluate(&pred, &json!(101), &mut ctx).unwrap());
    }

    #[test]
    fn test_exists() {
        let registry = PredicateRegistry::new();
        let mut ctx = EvalContext::new(&registry);

        let pred = Predicate::exists("/foo");
        assert!(evaluate(&pred, &json!({"foo": "bar"}), &mut ctx).unwrap());
        assert!(!evaluate(&pred, &json!({"foo": null}), &mut ctx).unwrap());
        assert!(!evaluate(&pred, &json!({"bar": "baz"}), &mut ctx).unwrap());
    }

    #[test]
    fn test_eq() {
        let registry = PredicateRegistry::new();
        let mut ctx = EvalContext::new(&registry);

        let pred = Predicate::eq("/status", json!("active"));
        assert!(evaluate(&pred, &json!({"status": "active"}), &mut ctx).unwrap());
        assert!(!evaluate(&pred, &json!({"status": "inactive"}), &mut ctx).unwrap());
    }

    #[test]
    fn test_in() {
        let registry = PredicateRegistry::new();
        let mut ctx = EvalContext::new(&registry);

        let pred = Predicate::in_values("/state", vec![json!("CA"), json!("NY"), json!("TX")]);
        assert!(evaluate(&pred, &json!({"state": "CA"}), &mut ctx).unwrap());
        assert!(!evaluate(&pred, &json!({"state": "FL"}), &mut ctx).unwrap());
    }

    #[test]
    fn test_and() {
        let registry = PredicateRegistry::new();
        let mut ctx = EvalContext::new(&registry);

        let pred = Predicate::and(vec![Predicate::min_len(1), Predicate::max_len(10)]);
        assert!(evaluate(&pred, &json!("hello"), &mut ctx).unwrap());
        assert!(!evaluate(&pred, &json!(""), &mut ctx).unwrap());
        assert!(!evaluate(&pred, &json!("this is way too long"), &mut ctx).unwrap());
    }

    #[test]
    fn test_or() {
        let registry = PredicateRegistry::new();
        let mut ctx = EvalContext::new(&registry);

        let pred = Predicate::or(vec![
            Predicate::eq("/type", json!("A")),
            Predicate::eq("/type", json!("B")),
        ]);
        assert!(evaluate(&pred, &json!({"type": "A"}), &mut ctx).unwrap());
        assert!(evaluate(&pred, &json!({"type": "B"}), &mut ctx).unwrap());
        assert!(!evaluate(&pred, &json!({"type": "C"}), &mut ctx).unwrap());
    }

    #[test]
    fn test_not() {
        let registry = PredicateRegistry::new();
        let mut ctx = EvalContext::new(&registry);

        let pred = Predicate::not(Predicate::exists("/deleted"));
        assert!(evaluate(&pred, &json!({"active": true}), &mut ctx).unwrap());
        assert!(!evaluate(&pred, &json!({"deleted": true}), &mut ctx).unwrap());
    }

    #[test]
    fn test_implies() {
        let registry = PredicateRegistry::new();
        let mut ctx = EvalContext::new(&registry);

        // If isForeign is false, then zip must exist
        let pred = Predicate::implies(
            Predicate::eq("/isForeign", json!(false)),
            Predicate::exists("/zip"),
        );

        // Antecedent true, consequent true -> true
        assert!(evaluate(
            &pred,
            &json!({"isForeign": false, "zip": "12345"}),
            &mut ctx
        )
        .unwrap());

        // Antecedent true, consequent false -> false
        assert!(!evaluate(&pred, &json!({"isForeign": false}), &mut ctx).unwrap());

        // Antecedent false -> true (regardless of consequent)
        assert!(evaluate(&pred, &json!({"isForeign": true}), &mut ctx).unwrap());
    }

    #[test]
    fn test_named_predicates_with_builtins() {
        let registry = PredicateRegistry::with_builtins();
        let mut ctx = EvalContext::new(&registry);

        let pred = Predicate::call("is_tin", json!({"kind": "ANY"}));

        // Valid SSN
        assert!(evaluate(&pred, &json!("123-45-6789"), &mut ctx).unwrap());

        // All zeros is invalid
        assert!(!evaluate(&pred, &json!("000-00-0000"), &mut ctx).unwrap());
    }

    #[test]
    fn test_unknown_predicate() {
        let registry = PredicateRegistry::new();
        let mut ctx = EvalContext::new(&registry);

        let pred = Predicate::call_no_args("unknown_predicate");
        let result = evaluate(&pred, &json!("test"), &mut ctx);
        assert!(matches!(result, Err(WelError::UnknownPredicate(_))));
    }

    #[test]
    fn test_eq_fields() {
        let registry = PredicateRegistry::new();
        let mut ctx = EvalContext::new(&registry);

        let pred = Predicate::EqFields {
            left: "/a".to_string(),
            right: "/b".to_string(),
        };

        assert!(evaluate(&pred, &json!({"a": 100, "b": 100}), &mut ctx).unwrap());
        assert!(!evaluate(&pred, &json!({"a": 100, "b": 200}), &mut ctx).unwrap());
        assert!(!evaluate(&pred, &json!({"a": 100}), &mut ctx).unwrap());
    }

    #[test]
    fn test_field_comparisons() {
        let registry = PredicateRegistry::new();
        let mut ctx = EvalContext::new(&registry);

        // Greater than
        let gt = Predicate::GtField {
            left: "/a".to_string(),
            right: "/b".to_string(),
        };
        assert!(evaluate(&gt, &json!({"a": 100, "b": 50}), &mut ctx).unwrap());
        assert!(!evaluate(&gt, &json!({"a": 50, "b": 100}), &mut ctx).unwrap());

        // Greater than or equal
        let gte = Predicate::GteField {
            left: "/a".to_string(),
            right: "/b".to_string(),
        };
        assert!(evaluate(&gte, &json!({"a": 100, "b": 100}), &mut ctx).unwrap());
        assert!(evaluate(&gte, &json!({"a": 100, "b": 50}), &mut ctx).unwrap());
        assert!(!evaluate(&gte, &json!({"a": 50, "b": 100}), &mut ctx).unwrap());

        // Less than
        let lt = Predicate::LtField {
            left: "/a".to_string(),
            right: "/b".to_string(),
        };
        assert!(evaluate(&lt, &json!({"a": 50, "b": 100}), &mut ctx).unwrap());
        assert!(!evaluate(&lt, &json!({"a": 100, "b": 50}), &mut ctx).unwrap());

        // Less than or equal
        let lte = Predicate::LteField {
            left: "/a".to_string(),
            right: "/b".to_string(),
        };
        assert!(evaluate(&lte, &json!({"a": 100, "b": 100}), &mut ctx).unwrap());
        assert!(evaluate(&lte, &json!({"a": 50, "b": 100}), &mut ctx).unwrap());
        assert!(!evaluate(&lte, &json!({"a": 100, "b": 50}), &mut ctx).unwrap());
    }

    #[test]
    fn test_sum_equals() {
        let registry = PredicateRegistry::new();
        let mut ctx = EvalContext::new(&registry);

        let pred = Predicate::SumEquals {
            paths: vec!["/a".to_string(), "/b".to_string(), "/c".to_string()],
            target: "/total".to_string(),
        };

        assert!(evaluate(
            &pred,
            &json!({"a": 10, "b": 20, "c": 30, "total": 60}),
            &mut ctx
        )
        .unwrap());
        assert!(!evaluate(
            &pred,
            &json!({"a": 10, "b": 20, "c": 30, "total": 50}),
            &mut ctx
        )
        .unwrap());
    }

    #[test]
    fn test_sum_equals_value() {
        let registry = PredicateRegistry::new();
        let mut ctx = EvalContext::new(&registry);

        let pred = Predicate::SumEqualsValue {
            paths: vec![
                "/percent1".to_string(),
                "/percent2".to_string(),
                "/percent3".to_string(),
            ],
            value: 100.0,
        };

        assert!(evaluate(
            &pred,
            &json!({"percent1": 50, "percent2": 30, "percent3": 20}),
            &mut ctx
        )
        .unwrap());
        assert!(!evaluate(
            &pred,
            &json!({"percent1": 50, "percent2": 30, "percent3": 10}),
            &mut ctx
        )
        .unwrap());
    }

    #[test]
    fn test_required_with() {
        let registry = PredicateRegistry::new();
        let mut ctx = EvalContext::new(&registry);

        let pred = Predicate::RequiredWith {
            field: "/confirm_password".to_string(),
            with: "/password".to_string(),
        };

        assert!(evaluate(&pred, &json!({"password": null}), &mut ctx).unwrap());
        assert!(!evaluate(&pred, &json!({"password": "secret"}), &mut ctx).unwrap());
        assert!(evaluate(
            &pred,
            &json!({"password": "secret", "confirm_password": "secret"}),
            &mut ctx
        )
        .unwrap());
    }

    #[test]
    fn test_required_without() {
        let registry = PredicateRegistry::new();
        let mut ctx = EvalContext::new(&registry);

        let pred = Predicate::RequiredWithout {
            field: "/tax_id".to_string(),
            without: "/ssn".to_string(),
        };

        assert!(!evaluate(&pred, &json!({}), &mut ctx).unwrap());
        assert!(evaluate(&pred, &json!({"tax_id": "12-3456789"}), &mut ctx).unwrap());
        assert!(evaluate(&pred, &json!({"ssn": "123-45-6789"}), &mut ctx).unwrap());
    }

    #[test]
    fn test_exactly_one_of() {
        let registry = PredicateRegistry::new();
        let mut ctx = EvalContext::new(&registry);

        let pred = Predicate::ExactlyOneOf {
            paths: vec!["/ssn".to_string(), "/ein".to_string()],
        };

        assert!(evaluate(&pred, &json!({"ssn": "123-45-6789"}), &mut ctx).unwrap());
        assert!(evaluate(&pred, &json!({"ein": "12-3456789"}), &mut ctx).unwrap());
        assert!(!evaluate(
            &pred,
            &json!({"ssn": "123-45-6789", "ein": "12-3456789"}),
            &mut ctx
        )
        .unwrap());
        assert!(!evaluate(&pred, &json!({}), &mut ctx).unwrap());
    }
}

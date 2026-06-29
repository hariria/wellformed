//! Static predicate registry using LazyLock.
//!
//! This module provides a zero-allocation predicate registry that is
//! initialized once at startup and shared across all validation calls.
//!
//! ## Design
//!
//! Instead of creating a new `HashMap` for every validation call, we use
//! `LazyLock` to initialize the registry once. Predicates are stored in
//! a perfect hash table for O(1) lookup with minimal memory overhead.
//!
//! ## Usage
//!
//! ```rust
//! use wellformed_validate::registry::{REGISTRY, Predicate};
//! use serde_json::json;
//!
//! // Get a predicate by name
//! if let Some(pred) = REGISTRY.get("is_ssn") {
//!     let valid = pred.evaluate(&json!("123-45-6789"), &json!(null));
//!     assert!(valid);
//! }
//! ```

use serde_json::Value;
use std::sync::LazyLock;

use crate::tin::{validate_any, validate_atin, validate_ein, validate_itin, validate_ssn};

// ============================================================================
// Predicate Trait
// ============================================================================

/// A named predicate function.
pub trait Predicate: Send + Sync {
    /// The name of this predicate.
    fn name(&self) -> &'static str;

    /// Evaluate the predicate against a value.
    fn evaluate(&self, value: &Value, args: &Value) -> bool;
}

// ============================================================================
// Built-in Predicates
// ============================================================================

/// SSN validation predicate.
struct IsSsnPredicate;

impl Predicate for IsSsnPredicate {
    fn name(&self) -> &'static str {
        "is_ssn"
    }

    fn evaluate(&self, value: &Value, _args: &Value) -> bool {
        value.as_str().map(validate_ssn).unwrap_or(false)
    }
}

/// EIN validation predicate.
struct IsEinPredicate;

impl Predicate for IsEinPredicate {
    fn name(&self) -> &'static str {
        "is_ein"
    }

    fn evaluate(&self, value: &Value, _args: &Value) -> bool {
        value.as_str().map(validate_ein).unwrap_or(false)
    }
}

/// ITIN validation predicate.
struct IsItinPredicate;

impl Predicate for IsItinPredicate {
    fn name(&self) -> &'static str {
        "is_itin"
    }

    fn evaluate(&self, value: &Value, _args: &Value) -> bool {
        value.as_str().map(validate_itin).unwrap_or(false)
    }
}

/// ATIN validation predicate.
struct IsAtinPredicate;

impl Predicate for IsAtinPredicate {
    fn name(&self) -> &'static str {
        "is_atin"
    }

    fn evaluate(&self, value: &Value, _args: &Value) -> bool {
        value.as_str().map(validate_atin).unwrap_or(false)
    }
}

/// Any TIN validation predicate.
struct IsTinPredicate;

impl Predicate for IsTinPredicate {
    fn name(&self) -> &'static str {
        "is_tin"
    }

    fn evaluate(&self, value: &Value, args: &Value) -> bool {
        let s = match value.as_str() {
            Some(s) => s,
            None => return false,
        };

        // Check for specific kind if provided
        let kind = args.get("kind").and_then(|v| v.as_str()).unwrap_or("ANY");

        match kind {
            "SSN" => validate_ssn(s),
            "EIN" => validate_ein(s),
            "ITIN" => validate_itin(s),
            "ATIN" => validate_atin(s),
            _ => validate_any(s),
        }
    }
}

/// Non-negative number predicate.
struct IsNonNegativePredicate;

impl Predicate for IsNonNegativePredicate {
    fn name(&self) -> &'static str {
        "is_non_negative"
    }

    fn evaluate(&self, value: &Value, _args: &Value) -> bool {
        value.as_f64().map(|n| n >= 0.0).unwrap_or(false)
    }
}

/// Positive number predicate.
struct IsPositivePredicate;

impl Predicate for IsPositivePredicate {
    fn name(&self) -> &'static str {
        "is_positive"
    }

    fn evaluate(&self, value: &Value, _args: &Value) -> bool {
        value.as_f64().map(|n| n > 0.0).unwrap_or(false)
    }
}

/// Tax year validation predicate.
struct IsTaxYearPredicate;

impl Predicate for IsTaxYearPredicate {
    fn name(&self) -> &'static str {
        "is_tax_year"
    }

    fn evaluate(&self, value: &Value, _args: &Value) -> bool {
        let year = match value {
            Value::Number(n) => n.as_u64(),
            Value::String(s) => s.parse::<u64>().ok(),
            _ => None,
        };

        year.map(|y| (2020..=2100).contains(&y)).unwrap_or(false)
    }
}

/// US state code predicate.
struct IsUsStatePredicate;

impl Predicate for IsUsStatePredicate {
    fn name(&self) -> &'static str {
        "is_us_state"
    }

    fn evaluate(&self, value: &Value, _args: &Value) -> bool {
        static US_STATES: &[&str] = &[
            "AL", "AK", "AZ", "AR", "CA", "CO", "CT", "DE", "FL", "GA", "HI", "ID", "IL", "IN",
            "IA", "KS", "KY", "LA", "ME", "MD", "MA", "MI", "MN", "MS", "MO", "MT", "NE", "NV",
            "NH", "NJ", "NM", "NY", "NC", "ND", "OH", "OK", "OR", "PA", "RI", "SC", "SD", "TN",
            "TX", "UT", "VT", "VA", "WA", "WV", "WI", "WY", "DC", "PR", "VI", "GU", "AS",
            "MP", // Territories
        ];

        value
            .as_str()
            .map(|s| US_STATES.contains(&s.to_uppercase().as_str()))
            .unwrap_or(false)
    }
}

/// US ZIP code predicate.
struct IsUsZipPredicate;

impl Predicate for IsUsZipPredicate {
    fn name(&self) -> &'static str {
        "is_us_zip"
    }

    fn evaluate(&self, value: &Value, _args: &Value) -> bool {
        let s = match value.as_str() {
            Some(s) => s,
            None => return false,
        };

        // 5 digits or 5+4 format
        let bytes = s.as_bytes();

        if bytes.len() == 5 {
            bytes.iter().all(|b| b.is_ascii_digit())
        } else if bytes.len() == 10 && bytes[5] == b'-' {
            bytes[..5].iter().all(|b| b.is_ascii_digit())
                && bytes[6..].iter().all(|b| b.is_ascii_digit())
        } else {
            false
        }
    }
}

/// Email validation predicate (basic check).
struct IsEmailPredicate;

impl Predicate for IsEmailPredicate {
    fn name(&self) -> &'static str {
        "is_email"
    }

    fn evaluate(&self, value: &Value, _args: &Value) -> bool {
        let s = match value.as_str() {
            Some(s) => s,
            None => return false,
        };

        // Basic email check: contains @ with text before and after
        let at_pos = s.find('@');
        match at_pos {
            Some(pos) if pos > 0 && pos < s.len() - 1 => {
                let domain = &s[pos + 1..];
                domain.contains('.') && !domain.starts_with('.') && !domain.ends_with('.')
            }
            _ => false,
        }
    }
}

/// Phone number validation predicate.
struct IsPhonePredicate;

impl Predicate for IsPhonePredicate {
    fn name(&self) -> &'static str {
        "is_phone"
    }

    fn evaluate(&self, value: &Value, _args: &Value) -> bool {
        let s = match value.as_str() {
            Some(s) => s,
            None => return false,
        };

        // Count digits
        let digit_count = s.chars().filter(|c| c.is_ascii_digit()).count();

        // US phone: 10 digits, international: 7-15 digits
        (7..=15).contains(&digit_count)
    }
}

/// Luhn checksum validation predicate.
struct LuhnPredicate;

impl Predicate for LuhnPredicate {
    fn name(&self) -> &'static str {
        "luhn"
    }

    fn evaluate(&self, value: &Value, _args: &Value) -> bool {
        let s = match value.as_str() {
            Some(s) => s,
            None => return false,
        };

        let digits: Vec<u32> = s
            .chars()
            .filter(|c| c.is_ascii_digit())
            .filter_map(|c| c.to_digit(10))
            .collect();

        if digits.is_empty() {
            return false;
        }

        let sum: u32 = digits
            .iter()
            .rev()
            .enumerate()
            .map(|(i, &d)| {
                if i % 2 == 1 {
                    let doubled = d * 2;
                    if doubled > 9 {
                        doubled - 9
                    } else {
                        doubled
                    }
                } else {
                    d
                }
            })
            .sum();

        sum % 10 == 0
    }
}

// ============================================================================
// Registry
// ============================================================================

/// Static array of all built-in predicates.
static PREDICATES: &[&dyn Predicate] = &[
    &IsSsnPredicate,
    &IsEinPredicate,
    &IsItinPredicate,
    &IsAtinPredicate,
    &IsTinPredicate,
    &IsNonNegativePredicate,
    &IsPositivePredicate,
    &IsTaxYearPredicate,
    &IsUsStatePredicate,
    &IsUsZipPredicate,
    &IsEmailPredicate,
    &IsPhonePredicate,
    &LuhnPredicate,
];

/// Predicate registry using a simple HashMap for reliable lookups.
/// The LazyLock ensures this is only created once at startup.
pub struct PredicateRegistry {
    /// Predicates stored in a HashMap for reliable O(1) lookup.
    predicates: std::collections::HashMap<&'static str, &'static dyn Predicate>,
}

impl PredicateRegistry {
    /// Create a new registry with all built-in predicates.
    fn new() -> Self {
        let mut predicates = std::collections::HashMap::with_capacity(PREDICATES.len());

        for pred in PREDICATES {
            predicates.insert(pred.name(), *pred);
        }

        Self { predicates }
    }

    /// Get a predicate by name.
    #[inline]
    pub fn get(&self, name: &str) -> Option<&'static dyn Predicate> {
        self.predicates.get(name).copied()
    }

    /// Check if a predicate exists.
    #[inline]
    pub fn contains(&self, name: &str) -> bool {
        self.predicates.contains_key(name)
    }

    /// Iterate over all predicates.
    pub fn iter(&self) -> impl Iterator<Item = &'static dyn Predicate> + '_ {
        self.predicates.values().copied()
    }
}

/// Global predicate registry, initialized once at startup.
pub static REGISTRY: LazyLock<PredicateRegistry> = LazyLock::new(PredicateRegistry::new);

// ============================================================================
// Convenience Functions
// ============================================================================

/// Evaluate a named predicate.
#[inline]
pub fn evaluate(name: &str, value: &Value, args: &Value) -> Option<bool> {
    REGISTRY.get(name).map(|p| p.evaluate(value, args))
}

/// Check if a named predicate exists.
#[inline]
pub fn exists(name: &str) -> bool {
    REGISTRY.contains(name)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_registry_lookup() {
        assert!(REGISTRY.contains("is_ssn"));
        assert!(REGISTRY.contains("is_ein"));
        assert!(REGISTRY.contains("is_tin"));
        assert!(REGISTRY.contains("luhn"));
        assert!(!REGISTRY.contains("nonexistent"));
    }

    #[test]
    fn test_is_ssn() {
        let pred = REGISTRY.get("is_ssn").unwrap();
        assert!(pred.evaluate(&json!("123-45-6789"), &json!(null)));
        assert!(!pred.evaluate(&json!("000-00-0000"), &json!(null)));
    }

    #[test]
    fn test_is_tin() {
        let pred = REGISTRY.get("is_tin").unwrap();

        // SSN
        assert!(pred.evaluate(&json!("123-45-6789"), &json!({"kind": "SSN"})));

        // EIN
        assert!(pred.evaluate(&json!("12-3456789"), &json!({"kind": "EIN"})));

        // Any
        assert!(pred.evaluate(&json!("123456789"), &json!({"kind": "ANY"})));
    }

    #[test]
    fn test_is_tax_year() {
        let pred = REGISTRY.get("is_tax_year").unwrap();
        assert!(pred.evaluate(&json!(2024), &json!(null)));
        assert!(pred.evaluate(&json!("2024"), &json!(null)));
        assert!(!pred.evaluate(&json!(1999), &json!(null)));
        assert!(!pred.evaluate(&json!(2200), &json!(null)));
    }

    #[test]
    fn test_is_us_state() {
        let pred = REGISTRY.get("is_us_state").unwrap();
        assert!(pred.evaluate(&json!("CA"), &json!(null)));
        assert!(pred.evaluate(&json!("NY"), &json!(null)));
        assert!(pred.evaluate(&json!("ca"), &json!(null))); // Case insensitive
        assert!(!pred.evaluate(&json!("XX"), &json!(null)));
    }

    #[test]
    fn test_is_us_zip() {
        let pred = REGISTRY.get("is_us_zip").unwrap();
        assert!(pred.evaluate(&json!("12345"), &json!(null)));
        assert!(pred.evaluate(&json!("12345-6789"), &json!(null)));
        assert!(!pred.evaluate(&json!("1234"), &json!(null)));
        assert!(!pred.evaluate(&json!("123456"), &json!(null)));
    }

    #[test]
    fn test_is_email() {
        let pred = REGISTRY.get("is_email").unwrap();
        assert!(pred.evaluate(&json!("test@example.com"), &json!(null)));
        assert!(pred.evaluate(&json!("a@b.c"), &json!(null)));
        assert!(!pred.evaluate(&json!("invalid"), &json!(null)));
        assert!(!pred.evaluate(&json!("@example.com"), &json!(null)));
        assert!(!pred.evaluate(&json!("test@"), &json!(null)));
    }

    #[test]
    fn test_luhn() {
        let pred = REGISTRY.get("luhn").unwrap();
        // Valid Luhn numbers
        assert!(pred.evaluate(&json!("79927398713"), &json!(null)));
        assert!(pred.evaluate(&json!("4532015112830366"), &json!(null)));
        // Invalid
        assert!(!pred.evaluate(&json!("79927398710"), &json!(null)));
    }

    #[test]
    fn test_evaluate_function() {
        assert_eq!(
            evaluate("is_ssn", &json!("123-45-6789"), &json!(null)),
            Some(true)
        );
        assert_eq!(evaluate("nonexistent", &json!("test"), &json!(null)), None);
    }
}

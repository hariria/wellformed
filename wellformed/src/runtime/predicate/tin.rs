//! TIN (Taxpayer Identification Number) validation predicates.
//!
//! This module delegates to wellformed_validate for SIMD-optimized TIN validation.
//! The inline validators achieve ~1.4ns per validation (700M/sec).

use super::registry::{NamedPredicate, PredicateRegistry};
use serde_json::Value;
use std::sync::Arc;
use wellformed_validate::tin;

/// Register all TIN-related predicates.
pub fn register_tin_predicates(registry: &mut PredicateRegistry) {
    registry.register(Arc::new(IsTinPredicate));
    registry.register(Arc::new(IsSsnPredicate));
    registry.register(Arc::new(IsEinPredicate));
    registry.register(Arc::new(IsItinPredicate));
    registry.register(Arc::new(IsAtinPredicate));
    registry.register(Arc::new(LuhnPredicate));
}

// ============================================================================
// Predicate Implementations (delegating to wellformed_validate)
// ============================================================================

/// Validate a TIN (Taxpayer Identification Number).
/// Delegates to wellformed_validate's SIMD-optimized implementation.
struct IsTinPredicate;

impl NamedPredicate for IsTinPredicate {
    fn name(&self) -> &str {
        "is_tin"
    }

    fn evaluate(&self, value: &Value, args: &Value) -> bool {
        let s = match value.as_str() {
            Some(s) => s,
            None => return false,
        };

        // Check kind if specified
        let kind = args.get("kind").and_then(|v| v.as_str()).unwrap_or("ANY");

        match kind {
            "SSN" => tin::validate_ssn(s),
            "EIN" => tin::validate_ein(s),
            "ITIN" => tin::validate_itin(s),
            "ATIN" => tin::validate_atin(s),
            _ => tin::validate_any(s),
        }
    }
}

/// Validate an SSN specifically.
struct IsSsnPredicate;

impl NamedPredicate for IsSsnPredicate {
    fn name(&self) -> &str {
        "is_ssn"
    }

    fn evaluate(&self, value: &Value, _args: &Value) -> bool {
        value.as_str().map(tin::validate_ssn).unwrap_or(false)
    }
}

/// Validate an EIN specifically.
struct IsEinPredicate;

impl NamedPredicate for IsEinPredicate {
    fn name(&self) -> &str {
        "is_ein"
    }

    fn evaluate(&self, value: &Value, _args: &Value) -> bool {
        value.as_str().map(tin::validate_ein).unwrap_or(false)
    }
}

/// Validate an ITIN specifically.
struct IsItinPredicate;

impl NamedPredicate for IsItinPredicate {
    fn name(&self) -> &str {
        "is_itin"
    }

    fn evaluate(&self, value: &Value, _args: &Value) -> bool {
        value.as_str().map(tin::validate_itin).unwrap_or(false)
    }
}

/// Validate an ATIN specifically.
struct IsAtinPredicate;

impl NamedPredicate for IsAtinPredicate {
    fn name(&self) -> &str {
        "is_atin"
    }

    fn evaluate(&self, value: &Value, _args: &Value) -> bool {
        value.as_str().map(tin::validate_atin).unwrap_or(false)
    }
}

/// Luhn checksum validation.
struct LuhnPredicate;

impl NamedPredicate for LuhnPredicate {
    fn name(&self) -> &str {
        "luhn"
    }

    fn evaluate(&self, value: &Value, _args: &Value) -> bool {
        let s = match value.as_str() {
            Some(s) => s,
            None => return false,
        };
        luhn_check(s)
    }
}

// ============================================================================
// Luhn Implementation (kept here as it's not in wellformed_validate yet)
// ============================================================================

pub(crate) fn luhn_check(s: &str) -> bool {
    let mut saw_digit = false;
    let mut sum = 0u32;
    let mut should_double = false;

    for byte in s.bytes().rev() {
        if !byte.is_ascii_digit() {
            continue;
        }

        saw_digit = true;
        let mut digit = (byte - b'0') as u32;
        if should_double {
            digit *= 2;
            if digit > 9 {
                digit -= 9;
            }
        }
        sum += digit;
        should_double = !should_double;
    }

    saw_digit && sum % 10 == 0
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_is_tin() {
        let pred = IsTinPredicate;

        // Valid SSN
        assert!(pred.evaluate(&json!("123-45-6789"), &json!({"kind": "ANY"})));

        // All zeros is invalid
        assert!(!pred.evaluate(&json!("000-00-0000"), &json!({"kind": "ANY"})));

        // Wrong length
        assert!(!pred.evaluate(&json!("12345678"), &json!({"kind": "ANY"})));
    }

    #[test]
    fn test_is_ssn() {
        let pred = IsSsnPredicate;

        // Valid SSN (area 123, group 45, serial 6789)
        assert!(pred.evaluate(&json!("123456789"), &json!(null)));

        // Invalid area (000)
        assert!(!pred.evaluate(&json!("000456789"), &json!(null)));

        // Invalid area (666)
        assert!(!pred.evaluate(&json!("666456789"), &json!(null)));

        // Invalid area (900-999)
        assert!(!pred.evaluate(&json!("900456789"), &json!(null)));
    }

    #[test]
    fn test_luhn() {
        let pred = LuhnPredicate;

        // Valid Luhn numbers
        assert!(pred.evaluate(&json!("79927398713"), &json!(null)));
        assert!(pred.evaluate(&json!("4532015112830366"), &json!(null)));

        // Invalid Luhn
        assert!(!pred.evaluate(&json!("79927398710"), &json!(null)));
    }
}

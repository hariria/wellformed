//! Insurance and healthcare validation predicates.
//!
//! Validates NPI, DEA, ICD-10, CPT, HCPCS, and NDC codes.

use super::registry::{NamedPredicate, PredicateRegistry};
use super::tin::luhn_check;
use serde_json::Value;
use std::sync::Arc;

/// Register all insurance predicates.
pub fn register_insurance_predicates(registry: &mut PredicateRegistry) {
    registry.register(Arc::new(IsNpiPredicate));
    registry.register(Arc::new(IsDeaNumberPredicate));
    registry.register(Arc::new(IsIcd10CodePredicate));
    registry.register(Arc::new(IsCptCodePredicate));
    registry.register(Arc::new(IsHcpcsCodePredicate));
    registry.register(Arc::new(IsNdcCodePredicate));
}

// ============================================================================
// NPI (National Provider Identifier)
// ============================================================================

/// Validate a National Provider Identifier (NPI).
///
/// NPI is a 10-digit number used to identify healthcare providers in the US.
/// Uses the Luhn algorithm with the constant prefix "80840".
///
/// Validation: prepend "80840" to the 10-digit NPI, then verify Luhn.
struct IsNpiPredicate;

impl NamedPredicate for IsNpiPredicate {
    fn name(&self) -> &str {
        "is_npi"
    }

    fn evaluate(&self, value: &Value, _args: &Value) -> bool {
        let s = match value.as_str() {
            Some(s) => s.trim().replace(['-', ' '], ""),
            None => return false,
        };

        // Must be exactly 10 digits
        if s.len() != 10 || !s.chars().all(|c| c.is_ascii_digit()) {
            return false;
        }

        // NPI Luhn: prefix with "80840" then run standard Luhn
        let prefixed = format!("80840{}", s);
        luhn_check(&prefixed)
    }
}

// ============================================================================
// DEA Number
// ============================================================================

/// Validate a DEA (Drug Enforcement Administration) registration number.
///
/// Format: 2 letters followed by 7 digits.
/// - First letter: registrant type (A, B, C, D, E, F, G, H, J, K, L, M, P, R, S, T, U, X)
/// - Second letter: first letter of registrant's last name
///
/// Check digit: (d1 + d3 + d5 + 2*(d2 + d4 + d6)) mod 10 = d7
struct IsDeaNumberPredicate;

/// Valid DEA registrant type codes.
const DEA_TYPE_CODES: &[char] = &[
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'J', 'K', 'L', 'M', 'P', 'R', 'S', 'T', 'U', 'X',
];

impl NamedPredicate for IsDeaNumberPredicate {
    fn name(&self) -> &str {
        "is_dea_number"
    }

    fn evaluate(&self, value: &Value, _args: &Value) -> bool {
        let s = match value.as_str() {
            Some(s) => s.trim().to_uppercase(),
            None => return false,
        };

        // Must be exactly 9 characters: 2 letters + 7 digits
        if s.len() != 9 {
            return false;
        }

        let chars: Vec<char> = s.chars().collect();

        // First char: valid registrant type
        if !DEA_TYPE_CODES.contains(&chars[0]) {
            return false;
        }

        // Second char: must be a letter
        if !chars[1].is_ascii_alphabetic() {
            return false;
        }

        // Remaining 7 must be digits
        let digits: Vec<u32> = match chars[2..].iter().map(|c| c.to_digit(10)).collect() {
            Some(d) => d,
            None => return false,
        };

        // Check digit: (d1 + d3 + d5 + 2*(d2 + d4 + d6)) mod 10 = d7
        let sum = digits[0] + digits[2] + digits[4] + 2 * (digits[1] + digits[3] + digits[5]);
        sum % 10 == digits[6]
    }
}

// ============================================================================
// ICD-10
// ============================================================================

/// Validate an ICD-10 diagnosis code.
///
/// Examples: E11.9, U07.1, S72.001A
///
/// Optional args:
/// - `strict_format` (bool): if true, when there are >3 trailing chars, a dot
///   must be present after the first 3 characters.
struct IsIcd10CodePredicate;

impl NamedPredicate for IsIcd10CodePredicate {
    fn name(&self) -> &str {
        "is_icd10_code"
    }

    fn evaluate(&self, value: &Value, args: &Value) -> bool {
        let s = match value.as_str() {
            Some(s) => s.trim().to_uppercase(),
            None => return false,
        };

        let strict_format = args
            .get("strict_format")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        // Canonical with optional dot after 3 chars
        if !is_icd10_syntax(&s) {
            return false;
        }

        if strict_format {
            let plain_len = s.chars().filter(|c| *c != '.').count();
            if plain_len > 3 && !s.contains('.') {
                return false;
            }
        }

        true
    }
}

fn is_icd10_syntax(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }

    let chars: Vec<char> = s.chars().collect();
    // At least 3 chars before optional dot suffix.
    if chars.len() < 3 {
        return false;
    }

    // First char alpha, next two alnum.
    if !chars[0].is_ascii_alphabetic()
        || !chars[1].is_ascii_alphanumeric()
        || !chars[2].is_ascii_alphanumeric()
    {
        return false;
    }

    // Optional suffix.
    if chars.len() == 3 {
        return true;
    }

    // Dotted format: XXX.Y up to XXX.YYYY
    if chars[3] == '.' {
        let suffix = &chars[4..];
        return (1..=4).contains(&suffix.len()) && suffix.iter().all(|c| c.is_ascii_alphanumeric());
    }

    // Undotted extended form: XXXY up to XXXYYYY
    let suffix = &chars[3..];
    (1..=4).contains(&suffix.len()) && suffix.iter().all(|c| c.is_ascii_alphanumeric())
}

// ============================================================================
// CPT
// ============================================================================

/// Validate a CPT code.
///
/// Supports:
/// - Category I: 5 digits (e.g. 99213)
/// - Category II: 4 digits + F (e.g. 1234F)
/// - Category III: 4 digits + T (e.g. 0123T)
///
/// Optional args:
/// - `allow_category_ii` (bool), default true
/// - `allow_category_iii` (bool), default true
struct IsCptCodePredicate;

impl NamedPredicate for IsCptCodePredicate {
    fn name(&self) -> &str {
        "is_cpt_code"
    }

    fn evaluate(&self, value: &Value, args: &Value) -> bool {
        let s = match value.as_str() {
            Some(s) => s.trim().to_uppercase(),
            None => return false,
        };

        let allow_category_ii = args
            .get("allow_category_ii")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        let allow_category_iii = args
            .get("allow_category_iii")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        if s.len() != 5 {
            return false;
        }

        if s.chars().all(|c| c.is_ascii_digit()) {
            return true;
        }

        let chars: Vec<char> = s.chars().collect();
        let first_four_digits = chars[..4].iter().all(|c| c.is_ascii_digit());
        if !first_four_digits {
            return false;
        }

        let last = chars[4];
        (allow_category_ii && last == 'F') || (allow_category_iii && last == 'T')
    }
}

// ============================================================================
// HCPCS
// ============================================================================

/// Validate an HCPCS Level II code.
///
/// Format: one letter A-V followed by 4 digits (e.g. A0428, J3490).
struct IsHcpcsCodePredicate;

impl NamedPredicate for IsHcpcsCodePredicate {
    fn name(&self) -> &str {
        "is_hcpcs_code"
    }

    fn evaluate(&self, value: &Value, _args: &Value) -> bool {
        let s = match value.as_str() {
            Some(s) => s.trim().to_uppercase(),
            None => return false,
        };

        if s.len() != 5 {
            return false;
        }

        let chars: Vec<char> = s.chars().collect();
        let first = chars[0];
        if !('A'..='V').contains(&first) {
            return false;
        }

        chars[1..].iter().all(|c| c.is_ascii_digit())
    }
}

// ============================================================================
// NDC
// ============================================================================

/// Validate an NDC code (National Drug Code).
///
/// Supports:
/// - 11-digit normalized format (5-4-2, digits only)
/// - 10-digit source formats (4-4-2, 5-3-2, 5-4-1) with or without hyphens
///
/// Optional args:
/// - `format` ("10" | "11"), default accepts either
struct IsNdcCodePredicate;

impl NamedPredicate for IsNdcCodePredicate {
    fn name(&self) -> &str {
        "is_ndc_code"
    }

    fn evaluate(&self, value: &Value, args: &Value) -> bool {
        let s = match value.as_str() {
            Some(s) => s.trim(),
            None => return false,
        };

        let format = args
            .get("format")
            .and_then(|v| v.as_str())
            .unwrap_or("ANY")
            .to_uppercase();

        match format.as_str() {
            "10" => is_ndc10(s),
            "11" => is_ndc11(s),
            _ => is_ndc10(s) || is_ndc11(s),
        }
    }
}

fn is_ndc11(s: &str) -> bool {
    let digits: String = s.chars().filter(|c| c.is_ascii_digit()).collect();
    digits.len() == 11 && digits.chars().all(|c| c.is_ascii_digit())
}

fn is_ndc10(s: &str) -> bool {
    // Hyphenated canonical 10-digit patterns.
    let parts: Vec<&str> = s.split('-').collect();
    if parts.len() == 3 {
        let lens = [parts[0].len(), parts[1].len(), parts[2].len()];
        let valid_shape = lens == [4, 4, 2] || lens == [5, 3, 2] || lens == [5, 4, 1];
        return valid_shape && parts.iter().all(|p| p.chars().all(|c| c.is_ascii_digit()));
    }

    // Non-hyphenated 10-digit fallback.
    let digits: String = s.chars().filter(|c| c.is_ascii_digit()).collect();
    digits.len() == 10 && digits.chars().all(|c| c.is_ascii_digit())
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn eval(pred: &dyn NamedPredicate, value: &str, args: Value) -> bool {
        pred.evaluate(&json!(value), &args)
    }

    // --- NPI ---

    #[test]
    fn test_npi_valid() {
        let p = IsNpiPredicate;
        assert!(eval(&p, "1234567893", json!({})));
        assert!(eval(&p, "1245319599", json!({})));
        assert!(eval(&p, "1123456786", json!({})));
    }

    #[test]
    fn test_npi_with_formatting() {
        let p = IsNpiPredicate;
        // Spaces/hyphens stripped
        assert!(eval(&p, "1234567893", json!({})));
    }

    #[test]
    fn test_npi_invalid() {
        let p = IsNpiPredicate;
        // Wrong check digit
        assert!(!eval(&p, "1234567890", json!({})));
        // Too short
        assert!(!eval(&p, "123456789", json!({})));
        // Too long
        assert!(!eval(&p, "12345678901", json!({})));
        // Non-digit
        assert!(!eval(&p, "123456789A", json!({})));
    }

    // --- DEA Number ---

    #[test]
    fn test_dea_valid() {
        let p = IsDeaNumberPredicate;
        // AB1234563: (1+3+5) + 2*(2+4+6) = 9 + 24 = 33, 33 % 10 = 3
        assert!(eval(&p, "AB1234563", json!({})));
        // Case insensitive
        assert!(eval(&p, "ab1234563", json!({})));
    }

    #[test]
    fn test_dea_invalid() {
        let p = IsDeaNumberPredicate;
        // Wrong check digit
        assert!(!eval(&p, "AB1234560", json!({})));
        // Invalid type code
        assert!(!eval(&p, "ZB1234563", json!({})));
        // Too short
        assert!(!eval(&p, "AB123456", json!({})));
        // Non-digit in number part
        assert!(!eval(&p, "AB12345AB", json!({})));
    }

    // --- ICD-10 ---

    #[test]
    fn test_icd10_valid() {
        let p = IsIcd10CodePredicate;
        assert!(eval(&p, "E11.9", json!({})));
        assert!(eval(&p, "U07.1", json!({})));
        assert!(eval(&p, "S72001A", json!({}))); // undotted accepted by default
    }

    #[test]
    fn test_icd10_strict_format() {
        let p = IsIcd10CodePredicate;
        assert!(eval(&p, "S72.001A", json!({"strict_format": true})));
        assert!(!eval(&p, "S72001A", json!({"strict_format": true})));
    }

    #[test]
    fn test_icd10_invalid() {
        let p = IsIcd10CodePredicate;
        assert!(!eval(&p, "", json!({})));
        assert!(!eval(&p, "A1", json!({})));
        assert!(!eval(&p, "A1$", json!({})));
    }

    // --- CPT ---

    #[test]
    fn test_cpt_valid() {
        let p = IsCptCodePredicate;
        assert!(eval(&p, "99213", json!({})));
        assert!(eval(&p, "1234F", json!({})));
        assert!(eval(&p, "0123T", json!({})));
    }

    #[test]
    fn test_cpt_with_args() {
        let p = IsCptCodePredicate;
        assert!(!eval(&p, "1234F", json!({"allow_category_ii": false})));
        assert!(!eval(&p, "0123T", json!({"allow_category_iii": false})));
    }

    #[test]
    fn test_cpt_invalid() {
        let p = IsCptCodePredicate;
        assert!(!eval(&p, "1234", json!({})));
        assert!(!eval(&p, "ABCDE", json!({})));
        assert!(!eval(&p, "12345F", json!({})));
    }

    // --- HCPCS ---

    #[test]
    fn test_hcpcs_valid() {
        let p = IsHcpcsCodePredicate;
        assert!(eval(&p, "A0428", json!({})));
        assert!(eval(&p, "J3490", json!({})));
    }

    #[test]
    fn test_hcpcs_invalid() {
        let p = IsHcpcsCodePredicate;
        assert!(!eval(&p, "W1234", json!({}))); // out of A-V range
        assert!(!eval(&p, "A123", json!({})));
        assert!(!eval(&p, "A12B4", json!({})));
    }

    // --- NDC ---

    #[test]
    fn test_ndc_valid() {
        let p = IsNdcCodePredicate;
        assert!(eval(&p, "12345-6789-01", json!({}))); // 11-format hyphenated
        assert!(eval(&p, "12345678901", json!({}))); // 11-format plain
        assert!(eval(&p, "1234-5678-90", json!({}))); // 10-format hyphenated
        assert!(eval(&p, "1234567890", json!({}))); // 10-format plain
    }

    #[test]
    fn test_ndc_format_filter() {
        let p = IsNdcCodePredicate;
        assert!(eval(&p, "12345678901", json!({"format": "11"})));
        assert!(!eval(&p, "1234567890", json!({"format": "11"})));
        assert!(eval(&p, "1234567890", json!({"format": "10"})));
        assert!(!eval(&p, "12345678901", json!({"format": "10"})));
    }

    #[test]
    fn test_ndc_invalid() {
        let p = IsNdcCodePredicate;
        assert!(!eval(&p, "123-45-678", json!({})));
        assert!(!eval(&p, "ABCDEFGHIJK", json!({})));
    }
}

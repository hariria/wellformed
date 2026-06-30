//! Text analysis predicates.
//!
//! Detects text directionality (RTL/LTR) based on Unicode character ranges.

use super::registry::{NamedPredicate, PredicateRegistry};
use serde_json::Value;
use std::sync::Arc;

/// Register all text predicates.
pub fn register_text_predicates(registry: &mut PredicateRegistry) {
    registry.register(Arc::new(IsRtlPredicate));
    registry.register(Arc::new(IsLtrPredicate));
    registry.register(Arc::new(StartsWithPredicate));
    registry.register(Arc::new(EndsWithPredicate));
    registry.register(Arc::new(ContainsPredicate));
    registry.register(Arc::new(IsAlphaPredicate));
    registry.register(Arc::new(IsDigitsPredicate));
    registry.register(Arc::new(IsAlphanumericPredicate));
    registry.register(Arc::new(IsAlphaSpacesPredicate));
    registry.register(Arc::new(IsAlphanumericSpacesPredicate));
    registry.register(Arc::new(IsNameCharsPredicate));
    registry.register(Arc::new(IsUppercasePredicate));
    registry.register(Arc::new(IsLowercasePredicate));
    registry.register(Arc::new(IsTitleCasePredicate));
}

/// Check if a character is in an RTL Unicode script range.
fn is_rtl_char(c: char) -> bool {
    let cp = c as u32;
    matches!(cp,
        // Hebrew
        0x0590..=0x05FF |
        0xFB1D..=0xFB4F |
        // Arabic
        0x0600..=0x06FF |
        0x0750..=0x077F |
        0x08A0..=0x08FF |
        0xFB50..=0xFDFF |
        0xFE70..=0xFEFF |
        // Syriac
        0x0700..=0x074F |
        // Thaana
        0x0780..=0x07BF |
        // N'Ko
        0x07C0..=0x07FF |
        // Samaritan
        0x0800..=0x083F |
        // Mandaic
        0x0840..=0x085F |
        // RTL marks
        0x200F | 0x202B | 0x202E | 0x2067
    )
}

// ============================================================================
// is_rtl
// ============================================================================

/// Detect if a string contains RTL (right-to-left) characters.
///
/// Returns true if any character in the string belongs to an RTL script
/// (Arabic, Hebrew, Syriac, Thaana, N'Ko, etc.).
///
/// Optional args:
/// - `threshold` (number 0-1): Fraction of directional chars that must be RTL.
///   Default: any RTL char triggers true (threshold = 0).
struct IsRtlPredicate;

impl NamedPredicate for IsRtlPredicate {
    fn name(&self) -> &str {
        "is_rtl"
    }

    fn evaluate(&self, value: &Value, args: &Value) -> bool {
        let s = match value.as_str() {
            Some(s) => s.trim(),
            None => return false,
        };

        if s.is_empty() {
            return false;
        }

        let threshold = args
            .get("threshold")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        if threshold <= 0.0 {
            // Any RTL char → true
            return s.chars().any(is_rtl_char);
        }

        // Count directional chars
        let mut rtl_count = 0usize;
        let mut ltr_count = 0usize;
        for c in s.chars() {
            if is_rtl_char(c) {
                rtl_count += 1;
            } else if c.is_alphabetic() {
                ltr_count += 1;
            }
        }

        let total = rtl_count + ltr_count;
        if total == 0 {
            return false;
        }

        (rtl_count as f64 / total as f64) >= threshold
    }
}

// ============================================================================
// is_ltr
// ============================================================================

/// Detect if a string is exclusively LTR (left-to-right).
///
/// Returns true only if the string contains no RTL characters.
/// Useful for fields that must not contain RTL text.
struct IsLtrPredicate;

impl NamedPredicate for IsLtrPredicate {
    fn name(&self) -> &str {
        "is_ltr"
    }

    fn evaluate(&self, value: &Value, _args: &Value) -> bool {
        let s = match value.as_str() {
            Some(s) => s.trim(),
            None => return false,
        };

        if s.is_empty() {
            return false;
        }

        !s.chars().any(is_rtl_char)
    }
}

// ============================================================================
// String Pattern Predicates
// ============================================================================

/// Validate that a string starts with a given prefix.
///
/// Args:
/// - `value`: required prefix string
struct StartsWithPredicate;

impl NamedPredicate for StartsWithPredicate {
    fn name(&self) -> &str {
        "starts_with"
    }

    fn evaluate(&self, value: &Value, args: &Value) -> bool {
        let s = match value.as_str() {
            Some(s) => s,
            None => return false,
        };
        let prefix = match args.get("value").and_then(|v| v.as_str()) {
            Some(v) => v,
            None => return false,
        };
        s.starts_with(prefix)
    }
}

/// Validate that a string ends with a given suffix.
///
/// Args:
/// - `value`: required suffix string
struct EndsWithPredicate;

impl NamedPredicate for EndsWithPredicate {
    fn name(&self) -> &str {
        "ends_with"
    }

    fn evaluate(&self, value: &Value, args: &Value) -> bool {
        let s = match value.as_str() {
            Some(s) => s,
            None => return false,
        };
        let suffix = match args.get("value").and_then(|v| v.as_str()) {
            Some(v) => v,
            None => return false,
        };
        s.ends_with(suffix)
    }
}

/// Validate that a string contains a given substring.
///
/// Args:
/// - `value`: required substring
struct ContainsPredicate;

impl NamedPredicate for ContainsPredicate {
    fn name(&self) -> &str {
        "contains"
    }

    fn evaluate(&self, value: &Value, args: &Value) -> bool {
        let s = match value.as_str() {
            Some(s) => s,
            None => return false,
        };
        let needle = match args.get("value").and_then(|v| v.as_str()) {
            Some(v) => v,
            None => return false,
        };
        s.contains(needle)
    }
}

// ============================================================================
// Character Class Predicates (no regex — pure byte scanning)
// ============================================================================

/// Validate that a string contains only ASCII letters (A-Za-z), non-empty.
struct IsAlphaPredicate;

impl NamedPredicate for IsAlphaPredicate {
    fn name(&self) -> &str {
        "is_alpha"
    }

    fn evaluate(&self, value: &Value, _args: &Value) -> bool {
        let s = match value.as_str() {
            Some(s) => s.trim(),
            None => return false,
        };
        if s.is_empty() {
            return false;
        }
        for &b in s.as_bytes() {
            if !b.is_ascii_alphabetic() {
                return false;
            }
        }
        true
    }
}

/// Validate that a string contains only ASCII digits (0-9), non-empty.
struct IsDigitsPredicate;

impl NamedPredicate for IsDigitsPredicate {
    fn name(&self) -> &str {
        "is_digits"
    }

    fn evaluate(&self, value: &Value, _args: &Value) -> bool {
        let s = match value.as_str() {
            Some(s) => s.trim(),
            None => return false,
        };
        if s.is_empty() {
            return false;
        }
        for &b in s.as_bytes() {
            if !b.is_ascii_digit() {
                return false;
            }
        }
        true
    }
}

/// Validate that a string contains only ASCII letters and digits, non-empty.
struct IsAlphanumericPredicate;

impl NamedPredicate for IsAlphanumericPredicate {
    fn name(&self) -> &str {
        "is_alphanumeric"
    }

    fn evaluate(&self, value: &Value, _args: &Value) -> bool {
        let s = match value.as_str() {
            Some(s) => s.trim(),
            None => return false,
        };
        if s.is_empty() {
            return false;
        }
        for &b in s.as_bytes() {
            if !b.is_ascii_alphanumeric() {
                return false;
            }
        }
        true
    }
}

/// Validate that a string contains only ASCII letters and spaces, non-empty,
/// with at least one letter.
struct IsAlphaSpacesPredicate;

impl NamedPredicate for IsAlphaSpacesPredicate {
    fn name(&self) -> &str {
        "is_alpha_spaces"
    }

    fn evaluate(&self, value: &Value, _args: &Value) -> bool {
        let s = match value.as_str() {
            Some(s) => s.trim(),
            None => return false,
        };
        if s.is_empty() {
            return false;
        }
        let mut has_letter = false;
        for &b in s.as_bytes() {
            if b.is_ascii_alphabetic() {
                has_letter = true;
            } else if b != b' ' {
                return false;
            }
        }
        has_letter
    }
}

/// Validate that a string contains only ASCII letters, digits, and spaces,
/// non-empty, with at least one letter or digit.
struct IsAlphanumericSpacesPredicate;

impl NamedPredicate for IsAlphanumericSpacesPredicate {
    fn name(&self) -> &str {
        "is_alphanumeric_spaces"
    }

    fn evaluate(&self, value: &Value, _args: &Value) -> bool {
        let s = match value.as_str() {
            Some(s) => s.trim(),
            None => return false,
        };
        if s.is_empty() {
            return false;
        }
        let mut has_alnum = false;
        for &b in s.as_bytes() {
            if b.is_ascii_alphanumeric() {
                has_alnum = true;
            } else if b != b' ' {
                return false;
            }
        }
        has_alnum
    }
}

/// Validate that a string contains only ASCII letters, hyphens, and apostrophes,
/// non-empty, with at least one letter.
struct IsNameCharsPredicate;

impl NamedPredicate for IsNameCharsPredicate {
    fn name(&self) -> &str {
        "is_name_chars"
    }

    fn evaluate(&self, value: &Value, _args: &Value) -> bool {
        let s = match value.as_str() {
            Some(s) => s.trim(),
            None => return false,
        };
        if s.is_empty() {
            return false;
        }
        let mut has_letter = false;
        for &b in s.as_bytes() {
            if b.is_ascii_alphabetic() {
                has_letter = true;
            } else if b != b'-' && b != b'\'' {
                return false;
            }
        }
        has_letter
    }
}

/// Validate that a string contains only uppercase ASCII letters (A-Z), non-empty.
struct IsUppercasePredicate;

impl NamedPredicate for IsUppercasePredicate {
    fn name(&self) -> &str {
        "is_uppercase"
    }

    fn evaluate(&self, value: &Value, _args: &Value) -> bool {
        let s = match value.as_str() {
            Some(s) => s.trim(),
            None => return false,
        };
        if s.is_empty() {
            return false;
        }
        for &b in s.as_bytes() {
            if !b.is_ascii_uppercase() {
                return false;
            }
        }
        true
    }
}

/// Validate that a string contains only lowercase ASCII letters (a-z), non-empty.
struct IsLowercasePredicate;

impl NamedPredicate for IsLowercasePredicate {
    fn name(&self) -> &str {
        "is_lowercase"
    }

    fn evaluate(&self, value: &Value, _args: &Value) -> bool {
        let s = match value.as_str() {
            Some(s) => s.trim(),
            None => return false,
        };
        if s.is_empty() {
            return false;
        }
        for &b in s.as_bytes() {
            if !b.is_ascii_lowercase() {
                return false;
            }
        }
        true
    }
}

/// Validate that a string is title case: starts with an uppercase letter
/// followed by one or more lowercase letters.
struct IsTitleCasePredicate;

impl NamedPredicate for IsTitleCasePredicate {
    fn name(&self) -> &str {
        "is_title_case"
    }

    fn evaluate(&self, value: &Value, _args: &Value) -> bool {
        let s = match value.as_str() {
            Some(s) => s.trim(),
            None => return false,
        };
        let bytes = s.as_bytes();
        if bytes.len() < 2 {
            return false;
        }
        if !bytes[0].is_ascii_uppercase() {
            return false;
        }
        for &b in &bytes[1..] {
            if !b.is_ascii_lowercase() {
                return false;
            }
        }
        true
    }
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

    // --- is_rtl ---

    #[test]
    fn test_rtl_arabic() {
        let p = IsRtlPredicate;
        assert!(eval(&p, "مرحبا", json!({})));
        assert!(eval(&p, "Hello مرحبا", json!({}))); // mixed
    }

    #[test]
    fn test_rtl_hebrew() {
        let p = IsRtlPredicate;
        assert!(eval(&p, "שלום", json!({})));
        assert!(eval(&p, "Hello שלום world", json!({}))); // mixed
    }

    #[test]
    fn test_rtl_english() {
        let p = IsRtlPredicate;
        assert!(!eval(&p, "Hello world", json!({})));
        assert!(!eval(&p, "12345", json!({})));
    }

    #[test]
    fn test_rtl_threshold() {
        let p = IsRtlPredicate;
        // "مرحبا Hello" - 5 RTL, 5 LTR → 50%
        assert!(eval(&p, "مرحبا Hello", json!({"threshold": 0.5})));
        assert!(!eval(
            &p,
            "مرحبا Hello World Test",
            json!({"threshold": 0.8})
        ));
    }

    #[test]
    fn test_rtl_empty() {
        let p = IsRtlPredicate;
        assert!(!eval(&p, "", json!({})));
    }

    // --- is_ltr ---

    #[test]
    fn test_ltr_english() {
        let p = IsLtrPredicate;
        assert!(eval(&p, "Hello world", json!({})));
        assert!(eval(&p, "12345", json!({})));
        assert!(eval(&p, "Hello 123 world!", json!({})));
    }

    #[test]
    fn test_ltr_rejects_rtl() {
        let p = IsLtrPredicate;
        assert!(!eval(&p, "مرحبا", json!({})));
        assert!(!eval(&p, "Hello مرحبا", json!({})));
        assert!(!eval(&p, "שלום", json!({})));
    }

    #[test]
    fn test_ltr_empty() {
        let p = IsLtrPredicate;
        assert!(!eval(&p, "", json!({})));
    }

    #[test]
    fn test_starts_with() {
        let p = StartsWithPredicate;
        assert!(eval(&p, "foobar", json!({"value": "foo"})));
        assert!(!eval(&p, "foobar", json!({"value": "bar"})));
    }

    #[test]
    fn test_ends_with() {
        let p = EndsWithPredicate;
        assert!(eval(&p, "foobar", json!({"value": "bar"})));
        assert!(!eval(&p, "foobar", json!({"value": "foo"})));
    }

    #[test]
    fn test_contains() {
        let p = ContainsPredicate;
        assert!(eval(&p, "foobar", json!({"value": "oob"})));
        assert!(!eval(&p, "foobar", json!({"value": "baz"})));
    }

    // --- is_alpha ---

    #[test]
    fn test_alpha() {
        let p = IsAlphaPredicate;
        assert!(eval(&p, "Hello", json!({})));
        assert!(eval(&p, "abc", json!({})));
        assert!(eval(&p, "XYZ", json!({})));
        assert!(!eval(&p, "abc123", json!({})));
        assert!(!eval(&p, "hello world", json!({})));
        assert!(!eval(&p, "", json!({})));
        assert!(!eval(&p, "  ", json!({})));
        assert!(!eval(&p, "abc!", json!({})));
    }

    // --- is_digits ---

    #[test]
    fn test_digits() {
        let p = IsDigitsPredicate;
        assert!(eval(&p, "12345", json!({})));
        assert!(eval(&p, "0", json!({})));
        assert!(!eval(&p, "123a", json!({})));
        assert!(!eval(&p, "", json!({})));
        assert!(!eval(&p, "  ", json!({})));
        assert!(!eval(&p, "12.34", json!({})));
    }

    // --- is_alphanumeric ---

    #[test]
    fn test_alphanumeric() {
        let p = IsAlphanumericPredicate;
        assert!(eval(&p, "abc123", json!({})));
        assert!(eval(&p, "Hello", json!({})));
        assert!(eval(&p, "999", json!({})));
        assert!(!eval(&p, "abc 123", json!({})));
        assert!(!eval(&p, "abc-123", json!({})));
        assert!(!eval(&p, "", json!({})));
    }

    // --- is_alpha_spaces ---

    #[test]
    fn test_alpha_spaces() {
        let p = IsAlphaSpacesPredicate;
        assert!(eval(&p, "Hello World", json!({})));
        assert!(eval(&p, "abc", json!({})));
        assert!(!eval(&p, "abc123", json!({})));
        assert!(!eval(&p, "   ", json!({})));
        assert!(!eval(&p, "", json!({})));
        assert!(!eval(&p, "hello-world", json!({})));
    }

    // --- is_alphanumeric_spaces ---

    #[test]
    fn test_alphanumeric_spaces() {
        let p = IsAlphanumericSpacesPredicate;
        assert!(eval(&p, "Hello 123", json!({})));
        assert!(eval(&p, "abc", json!({})));
        assert!(eval(&p, "42", json!({})));
        assert!(!eval(&p, "   ", json!({})));
        assert!(!eval(&p, "", json!({})));
        assert!(!eval(&p, "hello-world", json!({})));
    }

    // --- is_name_chars ---

    #[test]
    fn test_name_chars() {
        let p = IsNameCharsPredicate;
        assert!(eval(&p, "O'Brien", json!({})));
        assert!(eval(&p, "Smith-Jones", json!({})));
        assert!(eval(&p, "Alice", json!({})));
        assert!(!eval(&p, "---", json!({})));
        assert!(!eval(&p, "'''", json!({})));
        assert!(!eval(&p, "", json!({})));
        assert!(!eval(&p, "John Doe", json!({})));
        assert!(!eval(&p, "abc123", json!({})));
    }

    // --- is_uppercase ---

    #[test]
    fn test_uppercase() {
        let p = IsUppercasePredicate;
        assert!(eval(&p, "ABC", json!({})));
        assert!(eval(&p, "Z", json!({})));
        assert!(!eval(&p, "Abc", json!({})));
        assert!(!eval(&p, "abc", json!({})));
        assert!(!eval(&p, "AB1", json!({})));
        assert!(!eval(&p, "", json!({})));
    }

    // --- is_lowercase ---

    #[test]
    fn test_lowercase() {
        let p = IsLowercasePredicate;
        assert!(eval(&p, "abc", json!({})));
        assert!(eval(&p, "z", json!({})));
        assert!(!eval(&p, "Abc", json!({})));
        assert!(!eval(&p, "ABC", json!({})));
        assert!(!eval(&p, "ab1", json!({})));
        assert!(!eval(&p, "", json!({})));
    }

    // --- is_title_case ---

    #[test]
    fn test_title_case() {
        let p = IsTitleCasePredicate;
        assert!(eval(&p, "Hello", json!({})));
        assert!(eval(&p, "Ab", json!({})));
        assert!(!eval(&p, "hello", json!({})));
        assert!(!eval(&p, "HELLO", json!({})));
        assert!(!eval(&p, "H", json!({})));
        assert!(!eval(&p, "", json!({})));
        assert!(!eval(&p, "HeLLo", json!({})));
    }
}

//! Amount and numeric validation predicates.
//!
//! Validates money amounts, percentages, and numeric constraints.

use super::registry::{NamedPredicate, PredicateRegistry};
use serde_json::Value;
use std::sync::Arc;

/// Register all amount predicates.
pub fn register_amount_predicates(registry: &mut PredicateRegistry) {
    registry.register(Arc::new(IsNonNegativePredicate));
    registry.register(Arc::new(IsPositivePredicate));
    registry.register(Arc::new(IsNegativePredicate));
    registry.register(Arc::new(IsNonPositivePredicate));
    registry.register(Arc::new(IsPercentagePredicate));
    registry.register(Arc::new(IsMoneyFormatPredicate));
    registry.register(Arc::new(IsMoneyNoSymbolPredicate));
    registry.register(Arc::new(IsMultipleOfPredicate));
    registry.register(Arc::new(LessThanOrEqualPredicate));
    registry.register(Arc::new(GreaterThanOrEqualPredicate));
    registry.register(Arc::new(FormatDecimal2Predicate));
    registry.register(Arc::new(IsDecimalPlacesPredicate));
}

// ============================================================================
// Is Non-Negative Predicate
// ============================================================================

/// Validate that a numeric value is non-negative (>= 0).
///
/// Works with numbers and string representations of numbers.
struct IsNonNegativePredicate;

impl NamedPredicate for IsNonNegativePredicate {
    fn name(&self) -> &str {
        "is_non_negative"
    }

    fn evaluate(&self, value: &Value, _args: &Value) -> bool {
        match value {
            Value::Number(n) => n.as_f64().is_some_and(|f| f >= 0.0),
            Value::String(s) => {
                // Try to parse as a number (handle money formats like "$1,234.56")
                let cleaned: String = s
                    .chars()
                    .filter(|c| c.is_ascii_digit() || *c == '.' || *c == '-')
                    .collect();
                cleaned.parse::<f64>().is_ok_and(|f| f >= 0.0)
            }
            _ => false,
        }
    }
}

// ============================================================================
// Is Positive Predicate
// ============================================================================

/// Validate that a numeric value is positive (> 0).
struct IsPositivePredicate;

impl NamedPredicate for IsPositivePredicate {
    fn name(&self) -> &str {
        "is_positive"
    }

    fn evaluate(&self, value: &Value, _args: &Value) -> bool {
        match value {
            Value::Number(n) => n.as_f64().is_some_and(|f| f > 0.0),
            Value::String(s) => {
                let cleaned: String = s
                    .chars()
                    .filter(|c| c.is_ascii_digit() || *c == '.' || *c == '-')
                    .collect();
                cleaned.parse::<f64>().is_ok_and(|f| f > 0.0)
            }
            _ => false,
        }
    }
}

// ============================================================================
// Is Negative Predicate
// ============================================================================

/// Validate that a numeric value is negative (< 0).
struct IsNegativePredicate;

impl NamedPredicate for IsNegativePredicate {
    fn name(&self) -> &str {
        "is_negative"
    }

    fn evaluate(&self, value: &Value, _args: &Value) -> bool {
        extract_number(value).is_some_and(|f| f < 0.0)
    }
}

// ============================================================================
// Is Non-Positive Predicate
// ============================================================================

/// Validate that a numeric value is non-positive (<= 0).
struct IsNonPositivePredicate;

impl NamedPredicate for IsNonPositivePredicate {
    fn name(&self) -> &str {
        "is_non_positive"
    }

    fn evaluate(&self, value: &Value, _args: &Value) -> bool {
        extract_number(value).is_some_and(|f| f <= 0.0)
    }
}

// ============================================================================
// Is Percentage Predicate
// ============================================================================

/// Validate that a value is a valid percentage.
///
/// Args:
/// - format: "decimal" (0.0-1.0) or "percent" (0-100), default "percent"
/// - allow_over_100: If true, allow percentages > 100%, default false
struct IsPercentagePredicate;

impl NamedPredicate for IsPercentagePredicate {
    fn name(&self) -> &str {
        "is_percentage"
    }

    fn evaluate(&self, value: &Value, args: &Value) -> bool {
        let n = match value {
            Value::Number(n) => n.as_f64(),
            Value::String(s) => {
                let cleaned: String = s
                    .trim()
                    .trim_end_matches('%')
                    .chars()
                    .filter(|c| c.is_ascii_digit() || *c == '.' || *c == '-')
                    .collect();
                cleaned.parse::<f64>().ok()
            }
            _ => return false,
        };

        let n = match n {
            Some(n) => n,
            None => return false,
        };

        let format = args
            .get("format")
            .and_then(|v| v.as_str())
            .unwrap_or("percent");

        let allow_over_100 = args
            .get("allow_over_100")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let (min, max) = match format {
            "decimal" => (0.0, if allow_over_100 { f64::MAX } else { 1.0 }),
            _ => (0.0, if allow_over_100 { f64::MAX } else { 100.0 }),
        };

        n >= min && n <= max
    }
}

// ============================================================================
// Is Money Format Predicate
// ============================================================================

/// Validate that a string is a valid money format.
///
/// Accepts formats like:
/// - "1234.56"
/// - "$1,234.56"
/// - "1234" (no cents)
/// - "-$1,234.56" (negative)
struct IsMoneyFormatPredicate;

impl NamedPredicate for IsMoneyFormatPredicate {
    fn name(&self) -> &str {
        "is_money_format"
    }

    fn evaluate(&self, value: &Value, args: &Value) -> bool {
        let s = match value.as_str() {
            Some(s) => s.trim(),
            None => return false,
        };

        if s.is_empty() {
            return false;
        }

        // Remove currency symbols and commas
        let cleaned: String = s
            .chars()
            .filter(|c| c.is_ascii_digit() || *c == '.' || *c == '-')
            .collect();

        if cleaned.is_empty() {
            return false;
        }

        // Must be parseable as a number
        let parsed = match cleaned.parse::<f64>() {
            Ok(n) => n,
            Err(_) => return false,
        };

        // Check for max decimal places (default 2 for cents)
        let max_decimals = args
            .get("max_decimals")
            .and_then(|v| v.as_u64())
            .unwrap_or(2) as usize;

        if let Some(dot_pos) = cleaned.find('.') {
            let decimals = cleaned.len() - dot_pos - 1;
            if decimals > max_decimals {
                return false;
            }
        }

        // Check if negatives are allowed
        let allow_negative = args
            .get("allow_negative")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        if !allow_negative && parsed < 0.0 {
            return false;
        }

        true
    }
}

// ============================================================================
// Is Money No Symbol Predicate
// ============================================================================

/// Validate a money amount without a leading currency symbol.
///
/// Accepts formats like:
/// - "1234.56"
/// - "1,234.56"
/// - "1234" (no cents)
/// - "-1,234.56" (negative, if allowed)
///
/// Pure byte scanning, no regex.
struct IsMoneyNoSymbolPredicate;

impl NamedPredicate for IsMoneyNoSymbolPredicate {
    fn name(&self) -> &str {
        "is_money_no_symbol"
    }

    fn evaluate(&self, value: &Value, args: &Value) -> bool {
        let s = match value.as_str() {
            Some(s) => s.trim(),
            None => return false,
        };

        let bytes = s.as_bytes();
        if bytes.is_empty() {
            return false;
        }

        let max_decimals = args
            .get("max_decimals")
            .and_then(|v| v.as_u64())
            .unwrap_or(2) as usize;

        let allow_negative = args
            .get("allow_negative")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let mut i = 0;

        // Optional leading minus
        if bytes[i] == b'-' {
            if !allow_negative {
                return false;
            }
            i += 1;
            if i >= bytes.len() {
                return false;
            }
        }

        // Must start with a digit
        if !bytes[i].is_ascii_digit() {
            return false;
        }

        // Integer part: digits with optional comma grouping
        let mut digits_since_comma = 0usize;
        let mut has_comma = false;
        let mut int_digits = 0usize;

        while i < bytes.len() {
            let b = bytes[i];
            if b.is_ascii_digit() {
                digits_since_comma += 1;
                int_digits += 1;
                i += 1;
            } else if b == b',' {
                if has_comma && digits_since_comma != 3 {
                    return false;
                }
                if !has_comma && int_digits > 3 {
                    return false;
                }
                has_comma = true;
                digits_since_comma = 0;
                i += 1;
            } else if b == b'.' {
                break;
            } else {
                return false;
            }
        }

        if has_comma && digits_since_comma != 3 {
            return false;
        }
        if int_digits == 0 {
            return false;
        }

        // Decimal part
        if i < bytes.len() && bytes[i] == b'.' {
            i += 1;
            let dec_start = i;
            while i < bytes.len() {
                if !bytes[i].is_ascii_digit() {
                    return false;
                }
                i += 1;
            }
            let decimals = i - dec_start;
            if decimals == 0 || decimals > max_decimals {
                return false;
            }
        }

        i == bytes.len()
    }
}

// ============================================================================
// Is Multiple-Of Predicate
// ============================================================================

/// Validate that a numeric value is a multiple of the provided step.
///
/// Args:
/// - value: divisor/step (required, non-zero)
struct IsMultipleOfPredicate;

impl NamedPredicate for IsMultipleOfPredicate {
    fn name(&self) -> &str {
        "is_multiple_of"
    }

    fn evaluate(&self, value: &Value, args: &Value) -> bool {
        let n = match extract_number(value) {
            Some(v) => v,
            None => return false,
        };

        let step = match args.get("value").and_then(extract_number_from_value) {
            Some(v) if v != 0.0 => v,
            _ => return false,
        };

        // Tolerate floating-point rounding errors.
        let ratio = n / step;
        let nearest = ratio.round();
        (ratio - nearest).abs() < 1e-9
    }
}

// ============================================================================
// Less Than Or Equal Predicate
// ============================================================================

/// Validate that a numeric value is <= a comparison value.
///
/// Args:
/// - value: The maximum value (inclusive)
///
/// Useful for: qualified dividends <= total dividends
struct LessThanOrEqualPredicate;

impl NamedPredicate for LessThanOrEqualPredicate {
    fn name(&self) -> &str {
        "less_than_or_equal"
    }

    fn evaluate(&self, value: &Value, args: &Value) -> bool {
        let n = extract_number(value);
        let max = args.get("value").and_then(extract_number_from_value);

        match (n, max) {
            (Some(n), Some(max)) => n <= max,
            _ => false,
        }
    }
}

// ============================================================================
// Greater Than Or Equal Predicate
// ============================================================================

/// Validate that a numeric value is >= a comparison value.
///
/// Args:
/// - value: The minimum value (inclusive)
struct GreaterThanOrEqualPredicate;

impl NamedPredicate for GreaterThanOrEqualPredicate {
    fn name(&self) -> &str {
        "greater_than_or_equal"
    }

    fn evaluate(&self, value: &Value, args: &Value) -> bool {
        let n = extract_number(value);
        let min = args.get("value").and_then(extract_number_from_value);

        match (n, min) {
            (Some(n), Some(min)) => n >= min,
            _ => false,
        }
    }
}

// ============================================================================
// Helpers
// ============================================================================

fn extract_number(value: &Value) -> Option<f64> {
    match value {
        Value::Number(n) => n.as_f64(),
        Value::String(s) => {
            let cleaned: String = s
                .chars()
                .filter(|c| c.is_ascii_digit() || *c == '.' || *c == '-')
                .collect();
            cleaned.parse::<f64>().ok()
        }
        _ => None,
    }
}

fn extract_number_from_value(value: &Value) -> Option<f64> {
    extract_number(value)
}

// ============================================================================
// Format Decimal-2 Predicate
// ============================================================================

/// Validate that a string value has exactly 2 decimal places.
///
/// Used by IRIS schema constraints like: {"type": "format", "value": "decimal-2"}
/// Empty strings and integers (no decimal point) are considered valid.
struct FormatDecimal2Predicate;

impl NamedPredicate for FormatDecimal2Predicate {
    fn name(&self) -> &str {
        "format:decimal-2"
    }

    fn evaluate(&self, value: &Value, _args: &Value) -> bool {
        let s = match value.as_str() {
            Some(s) => s.trim(),
            None => return true, // Non-string values pass (let type validation handle it)
        };

        // Empty string is valid (optional field)
        if s.is_empty() {
            return true;
        }

        // If there's no decimal point, it's valid (integer format)
        if !s.contains('.') {
            // Must still be a valid number
            return s.parse::<i64>().is_ok();
        }

        // Check decimal places
        if let Some(dot_pos) = s.rfind('.') {
            let decimals = s.len() - dot_pos - 1;
            // Must have exactly 2 decimal places and be a valid number
            decimals == 2 && s.parse::<f64>().is_ok()
        } else {
            false
        }
    }
}

// ============================================================================
// Decimal Places Predicate
// ============================================================================

/// Validate that a numeric string has an exact or max number of decimal places.
///
/// Args:
/// - `places` (number, required): exact number of decimal places required
/// - `max` (bool): if true, `places` is a max rather than exact match. Default: false.
///
/// Integers (no decimal point) are valid when places=0 or max=true.
struct IsDecimalPlacesPredicate;

impl NamedPredicate for IsDecimalPlacesPredicate {
    fn name(&self) -> &str {
        "is_decimal_places"
    }

    fn evaluate(&self, value: &Value, args: &Value) -> bool {
        let places = args.get("places").and_then(|v| v.as_u64()).unwrap_or(2) as usize;

        let is_max = args.get("max").and_then(|v| v.as_bool()).unwrap_or(false);

        let s = match value {
            Value::String(s) => s.trim().to_string(),
            Value::Number(n) => {
                if let Some(f) = n.as_f64() {
                    format!("{}", f)
                } else {
                    return false;
                }
            }
            _ => return false,
        };

        if s.is_empty() {
            return false;
        }

        // Remove leading minus for validation
        let s = s.trim_start_matches('-');

        // Must be a valid number
        if s.parse::<f64>().is_err() {
            return false;
        }

        let actual_places = if let Some(dot_pos) = s.rfind('.') {
            // Trim trailing zeros for comparison
            let decimal_part = &s[dot_pos + 1..];
            decimal_part.len()
        } else {
            0
        };

        if is_max {
            actual_places <= places
        } else {
            actual_places == places
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_is_non_negative() {
        let pred = IsNonNegativePredicate;

        assert!(pred.evaluate(&json!(0), &json!(null)));
        assert!(pred.evaluate(&json!(100), &json!(null)));
        assert!(pred.evaluate(&json!(0.0), &json!(null)));
        assert!(pred.evaluate(&json!("100.50"), &json!(null)));
        assert!(pred.evaluate(&json!("$1,234.56"), &json!(null)));

        assert!(!pred.evaluate(&json!(-1), &json!(null)));
        assert!(!pred.evaluate(&json!("-100.50"), &json!(null)));
    }

    #[test]
    fn test_is_positive() {
        let pred = IsPositivePredicate;

        assert!(pred.evaluate(&json!(1), &json!(null)));
        assert!(pred.evaluate(&json!(0.01), &json!(null)));

        assert!(!pred.evaluate(&json!(0), &json!(null)));
        assert!(!pred.evaluate(&json!(-1), &json!(null)));
    }

    #[test]
    fn test_is_negative() {
        let pred = IsNegativePredicate;

        assert!(pred.evaluate(&json!(-1), &json!(null)));
        assert!(pred.evaluate(&json!("-10.5"), &json!(null)));

        assert!(!pred.evaluate(&json!(0), &json!(null)));
        assert!(!pred.evaluate(&json!(1), &json!(null)));
    }

    #[test]
    fn test_is_non_positive() {
        let pred = IsNonPositivePredicate;

        assert!(pred.evaluate(&json!(-1), &json!(null)));
        assert!(pred.evaluate(&json!(0), &json!(null)));

        assert!(!pred.evaluate(&json!(1), &json!(null)));
    }

    #[test]
    fn test_is_percentage() {
        let pred = IsPercentagePredicate;

        // Percent format (0-100)
        assert!(pred.evaluate(&json!(50), &json!(null)));
        assert!(pred.evaluate(&json!(0), &json!(null)));
        assert!(pred.evaluate(&json!(100), &json!(null)));
        assert!(pred.evaluate(&json!("75%"), &json!(null)));

        assert!(!pred.evaluate(&json!(101), &json!(null)));
        assert!(!pred.evaluate(&json!(-1), &json!(null)));

        // Decimal format (0-1)
        assert!(pred.evaluate(&json!(0.5), &json!({"format": "decimal"})));
        assert!(pred.evaluate(&json!(1.0), &json!({"format": "decimal"})));
        assert!(!pred.evaluate(&json!(1.5), &json!({"format": "decimal"})));

        // Allow over 100
        assert!(pred.evaluate(&json!(150), &json!({"allow_over_100": true})));
    }

    #[test]
    fn test_is_money_format() {
        let pred = IsMoneyFormatPredicate;

        assert!(pred.evaluate(&json!("1234.56"), &json!(null)));
        assert!(pred.evaluate(&json!("$1,234.56"), &json!(null)));
        assert!(pred.evaluate(&json!("1234"), &json!(null)));
        assert!(pred.evaluate(&json!("-$1,234.56"), &json!(null)));

        // Too many decimals
        assert!(!pred.evaluate(&json!("1234.567"), &json!(null)));
        assert!(pred.evaluate(&json!("1234.567"), &json!({"max_decimals": 3})));

        // Negative not allowed
        assert!(!pred.evaluate(&json!("-100"), &json!({"allow_negative": false})));

        // Empty/invalid
        assert!(!pred.evaluate(&json!(""), &json!(null)));
        assert!(!pred.evaluate(&json!("abc"), &json!(null)));
    }

    #[test]
    fn test_is_multiple_of() {
        let pred = IsMultipleOfPredicate;

        assert!(pred.evaluate(&json!(10), &json!({"value": 5})));
        assert!(pred.evaluate(&json!("12.5"), &json!({"value": "2.5"})));
        assert!(pred.evaluate(&json!(0.3), &json!({"value": 0.1})));

        assert!(!pred.evaluate(&json!(11), &json!({"value": 5})));
        assert!(!pred.evaluate(&json!(10), &json!({"value": 0})));
        assert!(!pred.evaluate(&json!("abc"), &json!({"value": 5})));
    }

    #[test]
    fn test_less_than_or_equal() {
        let pred = LessThanOrEqualPredicate;

        assert!(pred.evaluate(&json!(50), &json!({"value": 100})));
        assert!(pred.evaluate(&json!(100), &json!({"value": 100})));
        assert!(!pred.evaluate(&json!(101), &json!({"value": 100})));

        // String values
        assert!(pred.evaluate(&json!("50.00"), &json!({"value": "100.00"})));
    }

    #[test]
    fn test_greater_than_or_equal() {
        let pred = GreaterThanOrEqualPredicate;

        assert!(pred.evaluate(&json!(100), &json!({"value": 50})));
        assert!(pred.evaluate(&json!(50), &json!({"value": 50})));
        assert!(!pred.evaluate(&json!(49), &json!({"value": 50})));
    }

    #[test]
    fn test_is_decimal_places() {
        let pred = IsDecimalPlacesPredicate;

        // Exact 2 decimal places
        assert!(pred.evaluate(&json!("123.45"), &json!({"places": 2})));
        assert!(pred.evaluate(&json!("0.99"), &json!({"places": 2})));
        assert!(!pred.evaluate(&json!("123.4"), &json!({"places": 2})));
        assert!(!pred.evaluate(&json!("123.456"), &json!({"places": 2})));

        // Exact 0 decimal places
        assert!(pred.evaluate(&json!("123"), &json!({"places": 0})));
        assert!(!pred.evaluate(&json!("123.4"), &json!({"places": 0})));

        // Exact 3 decimal places
        assert!(pred.evaluate(&json!("1.234"), &json!({"places": 3})));
        assert!(!pred.evaluate(&json!("1.23"), &json!({"places": 3})));

        // Max mode
        assert!(pred.evaluate(&json!("123"), &json!({"places": 2, "max": true})));
        assert!(pred.evaluate(&json!("123.4"), &json!({"places": 2, "max": true})));
        assert!(pred.evaluate(&json!("123.45"), &json!({"places": 2, "max": true})));
        assert!(!pred.evaluate(&json!("123.456"), &json!({"places": 2, "max": true})));

        // Negative numbers
        assert!(pred.evaluate(&json!("-123.45"), &json!({"places": 2})));

        // Number type
        assert!(pred.evaluate(&json!(123.45), &json!({"places": 2})));
    }
}

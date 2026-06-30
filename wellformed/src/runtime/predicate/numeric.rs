//! Numeric type validation predicates.
//!
//! Validates integer types (u8, u16, u32, u64, i8, i16, i32, i64),
//! generic integer, and float checks.

use super::registry::{NamedPredicate, PredicateRegistry};
use serde_json::Value;
use std::sync::Arc;

/// Register all numeric type predicates.
pub fn register_numeric_predicates(registry: &mut PredicateRegistry) {
    registry.register(Arc::new(IsIntegerPredicate));
    registry.register(Arc::new(IsFloatPredicate));
    registry.register(Arc::new(IntTypePredicate::<0, 255>("is_u8")));
    registry.register(Arc::new(IntTypePredicate::<0, 65535>("is_u16")));
    registry.register(Arc::new(IntTypePredicate::<0, 4294967295>("is_u32")));
    registry.register(Arc::new(U64Predicate));
    registry.register(Arc::new(IntTypePredicate::<
        { -128_i128 as u128 as i128 },
        127,
    >("is_i8")));
    registry.register(Arc::new(IntTypePredicate::<
        { -32768_i128 as u128 as i128 },
        32767,
    >("is_i16")));
    registry.register(Arc::new(IntTypePredicate::<
        { -2147483648_i128 as u128 as i128 },
        2147483647,
    >("is_i32")));
    registry.register(Arc::new(I64Predicate));
}

/// Extract a numeric value from a JSON value as f64.
fn extract_f64(value: &Value) -> Option<f64> {
    match value {
        Value::Number(n) => n.as_f64(),
        Value::String(s) => s.trim().parse::<f64>().ok(),
        _ => None,
    }
}

/// Extract an exact integer from a JSON value as i128 (for range checking).
fn extract_integer(value: &Value) -> Option<i128> {
    match value {
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Some(i as i128)
            } else if let Some(u) = n.as_u64() {
                Some(u as i128)
            } else if let Some(f) = n.as_f64() {
                if f.fract() == 0.0 && f.is_finite() {
                    Some(f as i128)
                } else {
                    None
                }
            } else {
                None
            }
        }
        Value::String(s) => {
            let s = s.trim();
            // Try parsing as integer first
            if let Ok(i) = s.parse::<i128>() {
                return Some(i);
            }
            if is_plain_integer_string(s) {
                return None;
            }
            // Try as float, check it's whole
            if let Ok(f) = s.parse::<f64>() {
                if f.fract() == 0.0 && f.is_finite() {
                    return Some(f as i128);
                }
            }
            None
        }
        _ => None,
    }
}

fn is_plain_integer_string(s: &str) -> bool {
    let rest = s.strip_prefix(['+', '-']).unwrap_or(s);
    !rest.is_empty() && rest.bytes().all(|b| b.is_ascii_digit())
}

// ============================================================================
// is_integer
// ============================================================================

struct IsIntegerPredicate;

impl NamedPredicate for IsIntegerPredicate {
    fn name(&self) -> &str {
        "is_integer"
    }

    fn evaluate(&self, value: &Value, _args: &Value) -> bool {
        extract_integer(value).is_some()
    }
}

// ============================================================================
// is_float
// ============================================================================

struct IsFloatPredicate;

impl NamedPredicate for IsFloatPredicate {
    fn name(&self) -> &str {
        "is_float"
    }

    fn evaluate(&self, value: &Value, _args: &Value) -> bool {
        extract_f64(value).is_some()
    }
}

// ============================================================================
// Integer type predicates (u8, u16, u32, i8, i16, i32)
// ============================================================================

/// Generic integer type predicate with compile-time min/max bounds.
struct IntTypePredicate<const MIN: i128, const MAX: i128>(&'static str);

impl<const MIN: i128, const MAX: i128> NamedPredicate for IntTypePredicate<MIN, MAX> {
    fn name(&self) -> &str {
        self.0
    }

    fn evaluate(&self, value: &Value, _args: &Value) -> bool {
        match extract_integer(value) {
            Some(i) => i >= MIN && i <= MAX,
            None => false,
        }
    }
}

// ============================================================================
// u64 (needs special handling: max = 18446744073709551615)
// ============================================================================

struct U64Predicate;

impl NamedPredicate for U64Predicate {
    fn name(&self) -> &str {
        "is_u64"
    }

    fn evaluate(&self, value: &Value, _args: &Value) -> bool {
        match value {
            Value::Number(n) => {
                if n.as_u64().is_some() {
                    return true;
                }
                if let Some(i) = n.as_i64() {
                    return i >= 0;
                }
                if let Some(f) = n.as_f64() {
                    return f.fract() == 0.0 && f >= 0.0 && f <= u64::MAX as f64;
                }
                false
            }
            Value::String(s) => {
                let s = s.trim();
                // Try u64 parse directly
                if s.parse::<u64>().is_ok() {
                    return true;
                }
                if is_plain_integer_string(s) {
                    return false;
                }
                // Try as float
                if let Ok(f) = s.parse::<f64>() {
                    return f.fract() == 0.0 && f >= 0.0 && f <= u64::MAX as f64;
                }
                false
            }
            _ => false,
        }
    }
}

// ============================================================================
// i64 (needs special handling for full range)
// ============================================================================

struct I64Predicate;

impl NamedPredicate for I64Predicate {
    fn name(&self) -> &str {
        "is_i64"
    }

    fn evaluate(&self, value: &Value, _args: &Value) -> bool {
        match value {
            Value::Number(n) => {
                if n.as_i64().is_some() {
                    return true;
                }
                // u64 values > i64::MAX are not i64
                if let Some(u) = n.as_u64() {
                    return u <= i64::MAX as u64;
                }
                if let Some(f) = n.as_f64() {
                    return f.fract() == 0.0 && f >= i64::MIN as f64 && f <= i64::MAX as f64;
                }
                false
            }
            Value::String(s) => {
                let s = s.trim();
                if s.parse::<i64>().is_ok() {
                    return true;
                }
                if is_plain_integer_string(s) {
                    return false;
                }
                if let Ok(f) = s.parse::<f64>() {
                    return f.fract() == 0.0 && f >= i64::MIN as f64 && f <= i64::MAX as f64;
                }
                false
            }
            _ => false,
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn eval(pred: &dyn NamedPredicate, value: Value) -> bool {
        pred.evaluate(&value, &json!({}))
    }

    #[test]
    fn test_is_integer() {
        let p = IsIntegerPredicate;
        assert!(eval(&p, json!(42)));
        assert!(eval(&p, json!(0)));
        assert!(eval(&p, json!(-100)));
        assert!(eval(&p, json!("42")));
        assert!(eval(&p, json!("0")));
        assert!(eval(&p, json!(42.0))); // whole float

        assert!(!eval(&p, json!(1.25)));
        assert!(!eval(&p, json!("3.14")));
        assert!(!eval(&p, json!("abc")));
        assert!(!eval(&p, json!(null)));
    }

    #[test]
    fn test_is_float() {
        let p = IsFloatPredicate;
        assert!(eval(&p, json!(1.25)));
        assert!(eval(&p, json!(42)));
        assert!(eval(&p, json!("3.14")));
        assert!(eval(&p, json!("42")));

        assert!(!eval(&p, json!("abc")));
        assert!(!eval(&p, json!(null)));
    }

    #[test]
    fn test_is_u8() {
        let p = IntTypePredicate::<0, 255>("is_u8");
        assert!(eval(&p, json!(0)));
        assert!(eval(&p, json!(255)));
        assert!(eval(&p, json!("128")));

        assert!(!eval(&p, json!(-1)));
        assert!(!eval(&p, json!(256)));
        assert!(!eval(&p, json!(1.25)));
    }

    #[test]
    fn test_is_u16() {
        let p = IntTypePredicate::<0, 65535>("is_u16");
        assert!(eval(&p, json!(0)));
        assert!(eval(&p, json!(65535)));

        assert!(!eval(&p, json!(-1)));
        assert!(!eval(&p, json!(65536)));
    }

    #[test]
    fn test_is_u32() {
        let p = IntTypePredicate::<0, 4294967295>("is_u32");
        assert!(eval(&p, json!(0)));
        assert!(eval(&p, json!(4294967295_u64)));

        assert!(!eval(&p, json!(-1)));
        assert!(!eval(&p, json!(4294967296_u64)));
    }

    #[test]
    fn test_is_u64() {
        let p = U64Predicate;
        assert!(eval(&p, json!(0)));
        assert!(eval(&p, json!(18446744073709551615_u64)));
        assert!(eval(&p, json!("18446744073709551615")));

        assert!(!eval(&p, json!(-1)));
        assert!(!eval(&p, json!("-1")));
        assert!(!eval(&p, json!("18446744073709551616")));
        assert!(!eval(&p, json!(1.25)));
    }

    #[test]
    fn test_is_i8() {
        let p = IntTypePredicate::<-128, 127>("is_i8");
        assert!(eval(&p, json!(-128)));
        assert!(eval(&p, json!(127)));
        assert!(eval(&p, json!(0)));

        assert!(!eval(&p, json!(-129)));
        assert!(!eval(&p, json!(128)));
    }

    #[test]
    fn test_is_i16() {
        let p = IntTypePredicate::<-32768, 32767>("is_i16");
        assert!(eval(&p, json!(-32768)));
        assert!(eval(&p, json!(32767)));

        assert!(!eval(&p, json!(-32769)));
        assert!(!eval(&p, json!(32768)));
    }

    #[test]
    fn test_is_i32() {
        let p = IntTypePredicate::<-2147483648, 2147483647>("is_i32");
        assert!(eval(&p, json!(-2147483648_i64)));
        assert!(eval(&p, json!(2147483647)));

        assert!(!eval(&p, json!(-2147483649_i64)));
        assert!(!eval(&p, json!(2147483648_i64)));
    }

    #[test]
    fn test_is_i64() {
        let p = I64Predicate;
        assert!(eval(&p, json!(-9223372036854775808_i64)));
        assert!(eval(&p, json!(9223372036854775807_i64)));
        assert!(eval(&p, json!(0)));

        assert!(!eval(&p, json!(18446744073709551615_u64))); // u64::MAX > i64::MAX
        assert!(!eval(&p, json!("9223372036854775808")));
        assert!(!eval(&p, json!("-9223372036854775809")));
        assert!(!eval(&p, json!(1.25)));
    }

    #[test]
    fn test_string_parsing() {
        let p = IntTypePredicate::<0, 255>("is_u8");
        assert!(eval(&p, json!("0")));
        assert!(eval(&p, json!("255")));
        assert!(eval(&p, json!(" 128 "))); // trimmed
        assert!(!eval(&p, json!("256")));
        assert!(!eval(&p, json!("abc")));
        assert!(!eval(&p, json!("3.14")));
    }
}

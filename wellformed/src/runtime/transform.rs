//! Transform execution.
//!
//! This module implements the runtime execution of transforms.
//! Transforms are applied in order and modify values in place.

use crate::error::{Result, WelError};
use crate::ir::Transform;
use serde_json::Value;

/// Apply a sequence of transforms to a value.
///
/// Transforms are applied in order. The value is modified in place.
pub fn apply_transforms(value: &mut Value, transforms: &[Transform], path: &str) -> Result<()> {
    for transform in transforms {
        apply_transform(value, transform, path)?;
    }
    Ok(())
}

/// Apply a single transform to a value.
pub fn apply_transform(value: &mut Value, transform: &Transform, path: &str) -> Result<()> {
    match transform {
        Transform::Trim => {
            if let Value::String(s) = value {
                *s = s.trim().to_string();
            }
        }

        Transform::CollapseWhitespace => {
            if let Value::String(s) = value {
                *s = collapse_whitespace(s);
            }
        }

        Transform::DigitsOnly => {
            if let Value::String(s) = value {
                *s = s.chars().filter(|c| c.is_ascii_digit()).collect();
            }
        }

        Transform::Upper => {
            if let Value::String(s) = value {
                *s = s.to_uppercase();
            }
        }

        Transform::Lower => {
            if let Value::String(s) = value {
                *s = s.to_lowercase();
            }
        }

        Transform::MoneyToCents { scale } => {
            apply_money_to_cents(value, *scale, path)?;
        }

        Transform::DateParse { format } => {
            apply_date_parse(value, format, path)?;
        }

        Transform::Replace {
            pattern,
            replacement,
        } => {
            if let Value::String(s) = value {
                *s = s.replace(pattern, replacement);
            }
        }

        Transform::Default { value: default } => {
            if value.is_null() {
                *value = default.clone();
            }
        }

        Transform::PhoneUs => {
            if let Value::String(s) = value {
                let digits: String = s.chars().filter(|c| c.is_ascii_digit()).collect();
                let normalized = if digits.len() == 11 && digits.starts_with('1') {
                    &digits[1..]
                } else {
                    &digits
                };
                if normalized.len() == 10 {
                    *s = format!(
                        "({}) {}-{}",
                        &normalized[0..3],
                        &normalized[3..6],
                        &normalized[6..10]
                    );
                }
                // Otherwise leave value as-is
            }
        }

        Transform::PhoneE164 => {
            if let Value::String(s) = value {
                if let Some(normalized) = normalize_phone_e164(s) {
                    *s = normalized;
                }
            }
        }

        Transform::NormalizeFlightNumber => {
            if let Value::String(s) = value {
                let normalized: String = s
                    .trim()
                    .chars()
                    .filter(|c| !c.is_ascii_whitespace() && *c != '-')
                    .collect::<String>()
                    .to_uppercase();
                *s = normalized;
            }
        }

        Transform::NormalizeIcd10 => {
            if let Value::String(s) = value {
                if let Some(normalized) = normalize_icd10_code(s) {
                    *s = normalized;
                }
            }
        }

        Transform::NormalizeCpt => {
            if let Value::String(s) = value {
                if let Some(normalized) = normalize_cpt_code(s) {
                    *s = normalized;
                }
            }
        }

        Transform::NormalizeHcpcs => {
            if let Value::String(s) = value {
                if let Some(normalized) = normalize_hcpcs_code(s) {
                    *s = normalized;
                }
            }
        }

        Transform::NormalizeNdc11 => {
            if let Value::String(s) = value {
                if let Some(normalized) = normalize_ndc11_code(s) {
                    *s = normalized;
                }
            }
        }

        Transform::CardMaskLast4 => {
            if let Value::String(s) = value {
                let digits: String = s.chars().filter(|c| c.is_ascii_digit()).collect();
                if digits.len() > 4 {
                    let masked_count = digits.len() - 4;
                    *s = format!("{}{}", "*".repeat(masked_count), &digits[masked_count..]);
                } else {
                    *s = digits;
                }
            }
        }

        Transform::FormatSsn => {
            if let Value::String(s) = value {
                let digits: String = s.chars().filter(|c| c.is_ascii_digit()).collect();
                if digits.len() == 9 {
                    *s = format!("{}-{}-{}", &digits[0..3], &digits[3..5], &digits[5..9]);
                }
            }
        }

        Transform::FormatEin => {
            if let Value::String(s) = value {
                let digits: String = s.chars().filter(|c| c.is_ascii_digit()).collect();
                if digits.len() == 9 {
                    *s = format!("{}-{}", &digits[0..2], &digits[2..9]);
                }
            }
        }

        Transform::MaskSsn => {
            if let Value::String(s) = value {
                let digits: String = s.chars().filter(|c| c.is_ascii_digit()).collect();
                if digits.len() == 9 {
                    *s = format!("***-**-{}", &digits[5..9]);
                }
            }
        }

        Transform::MaskEin => {
            if let Value::String(s) = value {
                let digits: String = s.chars().filter(|c| c.is_ascii_digit()).collect();
                if digits.len() == 9 {
                    *s = format!("**-***{}", &digits[5..9]);
                }
            }
        }

        Transform::FormatIban => {
            if let Value::String(s) = value {
                let clean: String = s
                    .chars()
                    .filter(|c| !c.is_whitespace())
                    .collect::<String>()
                    .to_uppercase();
                if clean.len() >= 5 {
                    let mut result = String::with_capacity(clean.len() + clean.len() / 4);
                    for (i, c) in clean.chars().enumerate() {
                        if i > 0 && i % 4 == 0 {
                            result.push(' ');
                        }
                        result.push(c);
                    }
                    *s = result;
                }
            }
        }

        Transform::FormatCreditCard => {
            if let Value::String(s) = value {
                let digits: String = s.chars().filter(|c| c.is_ascii_digit()).collect();
                if digits.len() >= 13 {
                    let mut result = String::with_capacity(digits.len() + digits.len() / 4);
                    for (i, c) in digits.chars().enumerate() {
                        if i > 0 && i % 4 == 0 {
                            result.push(' ');
                        }
                        result.push(c);
                    }
                    *s = result;
                }
            }
        }

        Transform::FormatThousands { separator } => match value {
            Value::String(s) => {
                if let Some(formatted) = format_with_thousands(s, separator) {
                    *s = formatted;
                }
            }
            Value::Number(n) => {
                if let Some(f) = n.as_f64() {
                    let s = f.to_string();
                    if let Some(formatted) = format_with_thousands(&s, separator) {
                        *value = Value::String(formatted);
                    }
                }
            }
            _ => {}
        },

        Transform::FormatDecimal { places } => {
            match value {
                Value::String(s) => {
                    // Try to parse as number
                    let cleaned: String = s
                        .chars()
                        .filter(|c| c.is_ascii_digit() || *c == '.' || *c == '-')
                        .collect();
                    if let Ok(f) = cleaned.parse::<f64>() {
                        *s = format!("{:.prec$}", f, prec = *places as usize);
                    }
                }
                Value::Number(n) => {
                    if let Some(f) = n.as_f64() {
                        *value = Value::String(format!("{:.prec$}", f, prec = *places as usize));
                    }
                }
                _ => {}
            }
        }
    }
    Ok(())
}

/// Collapse multiple whitespace characters into single spaces.
fn collapse_whitespace(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut prev_was_whitespace = false;

    for c in s.chars() {
        if c.is_whitespace() {
            if !prev_was_whitespace {
                result.push(' ');
                prev_was_whitespace = true;
            }
        } else {
            result.push(c);
            prev_was_whitespace = false;
        }
    }

    result.trim().to_string()
}

/// Normalize a phone number to E.164 where possible.
fn normalize_phone_e164(input: &str) -> Option<String> {
    let digits: String = input.chars().filter(|c| c.is_ascii_digit()).collect();
    let trimmed = input.trim();

    if trimmed.starts_with('+') {
        if (7..=15).contains(&digits.len()) {
            return Some(format!("+{}", digits));
        }
        return None;
    }

    if digits.len() == 10 {
        return Some(format!("+1{}", digits));
    }

    if digits.len() == 11 && digits.starts_with('1') {
        return Some(format!("+{}", digits));
    }

    None
}

fn normalize_icd10_code(input: &str) -> Option<String> {
    let compact: String = input
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect::<String>()
        .to_uppercase();

    if !is_icd10_plain(&compact) {
        return None;
    }

    if compact.len() > 3 {
        Some(format!("{}.{}", &compact[0..3], &compact[3..]))
    } else {
        Some(compact)
    }
}

fn is_icd10_plain(s: &str) -> bool {
    if !(3..=7).contains(&s.len()) {
        return false;
    }

    let chars: Vec<char> = s.chars().collect();
    chars[0].is_ascii_alphabetic()
        && chars[1].is_ascii_alphanumeric()
        && chars[2].is_ascii_alphanumeric()
        && chars[3..].iter().all(|c| c.is_ascii_alphanumeric())
}

fn normalize_cpt_code(input: &str) -> Option<String> {
    let compact: String = input
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect::<String>()
        .to_uppercase();

    if compact.len() != 5 {
        return None;
    }

    if compact.chars().all(|c| c.is_ascii_digit()) {
        return Some(compact);
    }

    let chars: Vec<char> = compact.chars().collect();
    if !chars[0..4].iter().all(|c| c.is_ascii_digit()) {
        return None;
    }

    if chars[4] == 'F' || chars[4] == 'T' {
        Some(compact)
    } else {
        None
    }
}

fn normalize_hcpcs_code(input: &str) -> Option<String> {
    let compact: String = input
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect::<String>()
        .to_uppercase();

    if compact.len() != 5 {
        return None;
    }

    let chars: Vec<char> = compact.chars().collect();
    if !('A'..='V').contains(&chars[0]) {
        return None;
    }

    if chars[1..].iter().all(|c| c.is_ascii_digit()) {
        Some(compact)
    } else {
        None
    }
}

fn normalize_ndc11_code(input: &str) -> Option<String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }

    // Already normalized 11-digit (with or without hyphens).
    let digits: String = trimmed.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.len() == 11 {
        return Some(digits);
    }

    // Hyphenated 10-digit inputs can be normalized by left-padding one segment.
    let parts: Vec<&str> = trimmed.split('-').collect();
    if parts.len() != 3 || !parts.iter().all(|p| p.chars().all(|c| c.is_ascii_digit())) {
        return None;
    }

    match (parts[0].len(), parts[1].len(), parts[2].len()) {
        (4, 4, 2) => Some(format!("0{}{}{}", parts[0], parts[1], parts[2])),
        (5, 3, 2) => Some(format!("{}0{}{}", parts[0], parts[1], parts[2])),
        (5, 4, 1) => Some(format!("{}{}0{}", parts[0], parts[1], parts[2])),
        (5, 4, 2) => Some(format!("{}{}{}", parts[0], parts[1], parts[2])),
        _ => None,
    }
}

/// Convert a money string to cents (integer).
fn apply_money_to_cents(value: &mut Value, scale: u8, path: &str) -> Result<()> {
    let cents = match value {
        Value::String(s) => {
            parse_money_string(s, scale).ok_or_else(|| WelError::TransformFailed {
                path: path.to_string(),
                message: format!("invalid money format: {s}"),
            })?
        }
        Value::Number(n) => {
            if let Some(f) = n.as_f64() {
                let multiplier = 10_f64.powi(scale as i32);
                (f * multiplier).round() as i64
            } else {
                return Err(WelError::TransformFailed {
                    path: path.to_string(),
                    message: "number out of range".to_string(),
                });
            }
        }
        Value::Null => return Ok(()), // Leave null as null
        _ => {
            return Err(WelError::TransformFailed {
                path: path.to_string(),
                message: format!(
                    "expected string or number, got {:?}",
                    value_type_name(value)
                ),
            })
        }
    };

    *value = Value::Number(cents.into());
    Ok(())
}

/// Parse a money string like "123.45" or "$1,234.56" to cents.
fn parse_money_string(s: &str, scale: u8) -> Option<i64> {
    // Remove currency symbols, commas, and whitespace
    let cleaned: String = s
        .chars()
        .filter(|c| c.is_ascii_digit() || *c == '.' || *c == '-')
        .collect();

    if cleaned.is_empty() {
        return None;
    }

    let is_negative = cleaned.starts_with('-');
    let cleaned = cleaned.trim_start_matches('-');

    // Parse as float and convert to cents
    let f: f64 = cleaned.parse().ok()?;
    let multiplier = 10_f64.powi(scale as i32);
    let cents = (f * multiplier).round() as i64;

    Some(if is_negative { -cents } else { cents })
}

/// Parse a date string and convert to canonical format (YYYY-MM-DD).
fn apply_date_parse(value: &mut Value, format: &str, path: &str) -> Result<()> {
    if let Value::String(s) = value {
        if s.is_empty() {
            return Ok(());
        }

        // For now, we support a few common formats
        // A full implementation would use chrono's strptime
        let canonical =
            parse_date_with_format(s, format).ok_or_else(|| WelError::TransformFailed {
                path: path.to_string(),
                message: format!("invalid date format: {s} (expected {format})"),
            })?;

        *s = canonical;
    }
    Ok(())
}

/// Parse a date string with the given format and return canonical YYYY-MM-DD.
fn parse_date_with_format(s: &str, format: &str) -> Option<String> {
    // Simple parser for common formats
    // In production, use chrono::NaiveDate::parse_from_str

    match format {
        "%m/%d/%Y" | "MM/DD/YYYY" => {
            let (month, day, year) = parse_three_date_parts(s, '/')?;
            is_valid_date(year, month, day).then(|| format!("{year:04}-{month:02}-{day:02}"))
        }
        "%m-%d-%Y" | "MM-DD-YYYY" => {
            let (month, day, year) = parse_three_date_parts(s, '-')?;
            is_valid_date(year, month, day).then(|| format!("{year:04}-{month:02}-{day:02}"))
        }
        "%Y-%m-%d" | "YYYY-MM-DD" => {
            // Already canonical, just validate
            let (year, month, day) = parse_three_date_parts(s, '-')?;
            is_valid_date(year, month, day).then(|| format!("{year:04}-{month:02}-{day:02}"))
        }
        "%d/%m/%Y" | "DD/MM/YYYY" => {
            let (day, month, year) = parse_three_date_parts(s, '/')?;
            is_valid_date(year, month, day).then(|| format!("{year:04}-{month:02}-{day:02}"))
        }
        "%d-%m-%Y" | "DD-MM-YYYY" => {
            let (day, month, year) = parse_three_date_parts(s, '-')?;
            is_valid_date(year, month, day).then(|| format!("{year:04}-{month:02}-{day:02}"))
        }
        "MMDDYYYY" => {
            if s.len() != 8 || !s.chars().all(|c| c.is_ascii_digit()) {
                return None;
            }
            let month: u32 = s[0..2].parse().ok()?;
            let day: u32 = s[2..4].parse().ok()?;
            let year: u32 = s[4..8].parse().ok()?;
            is_valid_date(year, month, day).then(|| format!("{year:04}-{month:02}-{day:02}"))
        }
        _ => None, // Unsupported format
    }
}

fn parse_three_date_parts(s: &str, sep: char) -> Option<(u32, u32, u32)> {
    let parts: Vec<&str> = s.split(sep).collect();
    if parts.len() != 3 || parts.iter().any(|part| part.is_empty()) {
        return None;
    }

    Some((
        parts[0].parse().ok()?,
        parts[1].parse().ok()?,
        parts[2].parse().ok()?,
    ))
}

fn is_valid_date(year: u32, month: u32, day: u32) -> bool {
    if !(1..=12).contains(&month) || day == 0 {
        return false;
    }

    let days_in_month = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap_year(year) => 29,
        2 => 28,
        _ => return false,
    };

    day <= days_in_month
}

fn is_leap_year(year: u32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

/// Format a number string with thousands separators.
fn format_with_thousands(s: &str, separator: &str) -> Option<String> {
    // Clean input: remove existing commas/spaces, keep digits, dot, minus
    let cleaned: String = s
        .chars()
        .filter(|c| c.is_ascii_digit() || *c == '.' || *c == '-')
        .collect();

    if cleaned.is_empty() {
        return None;
    }

    let is_negative = cleaned.starts_with('-');
    let cleaned = cleaned.trim_start_matches('-');

    // Split on decimal point
    let (integer_part, decimal_part) = if let Some(dot_pos) = cleaned.find('.') {
        (&cleaned[..dot_pos], Some(&cleaned[dot_pos..]))
    } else {
        (cleaned, None)
    };

    if integer_part.is_empty() || !integer_part.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }

    // Add thousands separators from right to left
    let mut result = String::with_capacity(integer_part.len() + integer_part.len() / 3 + 5);
    for (i, c) in integer_part.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push_str(&separator.chars().rev().collect::<String>());
        }
        result.push(c);
    }
    let result: String = result.chars().rev().collect();

    let mut formatted = if is_negative {
        format!("-{}", result)
    } else {
        result
    };

    if let Some(dec) = decimal_part {
        formatted.push_str(dec);
    }

    Some(formatted)
}

/// Get a human-readable name for a JSON value type.
fn value_type_name(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_trim() {
        let mut value = json!("  hello world  ");
        apply_transform(&mut value, &Transform::Trim, "").unwrap();
        assert_eq!(value, json!("hello world"));
    }

    #[test]
    fn test_collapse_whitespace() {
        let mut value = json!("  hello   world  \n\t foo  ");
        apply_transform(&mut value, &Transform::CollapseWhitespace, "").unwrap();
        assert_eq!(value, json!("hello world foo"));
    }

    #[test]
    fn test_digits_only() {
        let mut value = json!("123-45-6789");
        apply_transform(&mut value, &Transform::DigitsOnly, "").unwrap();
        assert_eq!(value, json!("123456789"));
    }

    #[test]
    fn test_upper() {
        let mut value = json!("Hello World");
        apply_transform(&mut value, &Transform::Upper, "").unwrap();
        assert_eq!(value, json!("HELLO WORLD"));
    }

    #[test]
    fn test_lower() {
        let mut value = json!("Hello World");
        apply_transform(&mut value, &Transform::Lower, "").unwrap();
        assert_eq!(value, json!("hello world"));
    }

    #[test]
    fn test_money_to_cents_string() {
        let mut value = json!("123.45");
        apply_transform(&mut value, &Transform::MoneyToCents { scale: 2 }, "").unwrap();
        assert_eq!(value, json!(12345));
    }

    #[test]
    fn test_money_to_cents_with_currency() {
        let mut value = json!("$1,234.56");
        apply_transform(&mut value, &Transform::MoneyToCents { scale: 2 }, "").unwrap();
        assert_eq!(value, json!(123456));
    }

    #[test]
    fn test_money_to_cents_number() {
        let mut value = json!(123.45);
        apply_transform(&mut value, &Transform::MoneyToCents { scale: 2 }, "").unwrap();
        assert_eq!(value, json!(12345));
    }

    #[test]
    fn test_money_to_cents_negative() {
        let mut value = json!("-50.00");
        apply_transform(&mut value, &Transform::MoneyToCents { scale: 2 }, "").unwrap();
        assert_eq!(value, json!(-5000));
    }

    #[test]
    fn test_date_parse_mm_dd_yyyy() {
        let mut value = json!("12/25/2024");
        apply_transform(
            &mut value,
            &Transform::DateParse {
                format: "%m/%d/%Y".to_string(),
            },
            "",
        )
        .unwrap();
        assert_eq!(value, json!("2024-12-25"));
    }

    #[test]
    fn test_date_parse_already_canonical() {
        let mut value = json!("2024-12-25");
        apply_transform(
            &mut value,
            &Transform::DateParse {
                format: "%Y-%m-%d".to_string(),
            },
            "",
        )
        .unwrap();
        assert_eq!(value, json!("2024-12-25"));
    }

    #[test]
    fn test_date_parse_common_aliases() {
        let mut value = json!("12/25/2024");
        apply_transform(
            &mut value,
            &Transform::DateParse {
                format: "MM/DD/YYYY".to_string(),
            },
            "",
        )
        .unwrap();
        assert_eq!(value, json!("2024-12-25"));

        let mut value = json!("2024-1-5");
        apply_transform(
            &mut value,
            &Transform::DateParse {
                format: "YYYY-MM-DD".to_string(),
            },
            "",
        )
        .unwrap();
        assert_eq!(value, json!("2024-01-05"));

        let mut value = json!("12252024");
        apply_transform(
            &mut value,
            &Transform::DateParse {
                format: "MMDDYYYY".to_string(),
            },
            "",
        )
        .unwrap();
        assert_eq!(value, json!("2024-12-25"));
    }

    #[test]
    fn test_date_parse_rejects_invalid_calendar_dates() {
        let mut value = json!("02/31/2024");
        let result = apply_transform(
            &mut value,
            &Transform::DateParse {
                format: "%m/%d/%Y".to_string(),
            },
            "",
        );
        assert!(result.is_err());
        assert_eq!(value, json!("02/31/2024"));
    }

    #[test]
    fn test_replace() {
        let mut value = json!("123-45-6789");
        apply_transform(
            &mut value,
            &Transform::Replace {
                pattern: "-".to_string(),
                replacement: "".to_string(),
            },
            "",
        )
        .unwrap();
        assert_eq!(value, json!("123456789"));
    }

    #[test]
    fn test_default_on_null() {
        let mut value = json!(null);
        apply_transform(
            &mut value,
            &Transform::Default {
                value: json!("default"),
            },
            "",
        )
        .unwrap();
        assert_eq!(value, json!("default"));
    }

    #[test]
    fn test_default_on_existing() {
        let mut value = json!("existing");
        apply_transform(
            &mut value,
            &Transform::Default {
                value: json!("default"),
            },
            "",
        )
        .unwrap();
        assert_eq!(value, json!("existing"));
    }

    #[test]
    fn test_apply_multiple_transforms() {
        let mut value = json!("  123-45-6789  ");
        let transforms = vec![Transform::Trim, Transform::DigitsOnly];
        apply_transforms(&mut value, &transforms, "").unwrap();
        assert_eq!(value, json!("123456789"));
    }

    #[test]
    fn test_transform_on_non_string_is_noop() {
        let mut value = json!(123);
        apply_transform(&mut value, &Transform::Trim, "").unwrap();
        assert_eq!(value, json!(123)); // Unchanged
    }

    #[test]
    fn test_phone_us_10_digits() {
        let mut value = json!("6501234567");
        apply_transform(&mut value, &Transform::PhoneUs, "").unwrap();
        assert_eq!(value, json!("(650) 123-4567"));
    }

    #[test]
    fn test_phone_us_11_digits_with_country_code() {
        let mut value = json!("16501234567");
        apply_transform(&mut value, &Transform::PhoneUs, "").unwrap();
        assert_eq!(value, json!("(650) 123-4567"));
    }

    #[test]
    fn test_phone_us_idempotent() {
        let mut value = json!("(650) 123-4567");
        apply_transform(&mut value, &Transform::PhoneUs, "").unwrap();
        assert_eq!(value, json!("(650) 123-4567"));
    }

    #[test]
    fn test_phone_us_too_few_digits() {
        let mut value = json!("123");
        apply_transform(&mut value, &Transform::PhoneUs, "").unwrap();
        assert_eq!(value, json!("123")); // Unchanged
    }

    #[test]
    fn test_phone_us_with_formatting() {
        let mut value = json!("650-123-4567");
        apply_transform(&mut value, &Transform::PhoneUs, "").unwrap();
        assert_eq!(value, json!("(650) 123-4567"));

        let mut value = json!("650.123.4567");
        apply_transform(&mut value, &Transform::PhoneUs, "").unwrap();
        assert_eq!(value, json!("(650) 123-4567"));
    }

    #[test]
    fn test_phone_e164_us_10_digits() {
        let mut value = json!("6501234567");
        apply_transform(&mut value, &Transform::PhoneE164, "").unwrap();
        assert_eq!(value, json!("+16501234567"));
    }

    #[test]
    fn test_phone_e164_us_11_digits() {
        let mut value = json!("1 (650) 123-4567");
        apply_transform(&mut value, &Transform::PhoneE164, "").unwrap();
        assert_eq!(value, json!("+16501234567"));
    }

    #[test]
    fn test_phone_e164_international_plus() {
        let mut value = json!("+44 20 7946 0958");
        apply_transform(&mut value, &Transform::PhoneE164, "").unwrap();
        assert_eq!(value, json!("+442079460958"));
    }

    #[test]
    fn test_phone_e164_invalid_kept_as_is() {
        let mut value = json!("1234");
        apply_transform(&mut value, &Transform::PhoneE164, "").unwrap();
        assert_eq!(value, json!("1234"));
    }

    #[test]
    fn test_normalize_flight_number() {
        let mut value = json!("ua 123");
        apply_transform(&mut value, &Transform::NormalizeFlightNumber, "").unwrap();
        assert_eq!(value, json!("UA123"));

        let mut value = json!("ual-1234a");
        apply_transform(&mut value, &Transform::NormalizeFlightNumber, "").unwrap();
        assert_eq!(value, json!("UAL1234A"));
    }

    #[test]
    fn test_normalize_icd10() {
        let mut value = json!("s72.001a");
        apply_transform(&mut value, &Transform::NormalizeIcd10, "").unwrap();
        assert_eq!(value, json!("S72.001A"));

        let mut value = json!("u071");
        apply_transform(&mut value, &Transform::NormalizeIcd10, "").unwrap();
        assert_eq!(value, json!("U07.1"));

        let mut value = json!("bad$");
        apply_transform(&mut value, &Transform::NormalizeIcd10, "").unwrap();
        assert_eq!(value, json!("BAD")); // Punctuation stripped, normalized
    }

    #[test]
    fn test_normalize_cpt() {
        let mut value = json!(" 99213 ");
        apply_transform(&mut value, &Transform::NormalizeCpt, "").unwrap();
        assert_eq!(value, json!("99213"));

        let mut value = json!("1234f");
        apply_transform(&mut value, &Transform::NormalizeCpt, "").unwrap();
        assert_eq!(value, json!("1234F"));

        let mut value = json!("12A3F");
        apply_transform(&mut value, &Transform::NormalizeCpt, "").unwrap();
        assert_eq!(value, json!("12A3F")); // Invalid shape -> no-op
    }

    #[test]
    fn test_normalize_hcpcs() {
        let mut value = json!(" a0428 ");
        apply_transform(&mut value, &Transform::NormalizeHcpcs, "").unwrap();
        assert_eq!(value, json!("A0428"));

        let mut value = json!("w1234");
        apply_transform(&mut value, &Transform::NormalizeHcpcs, "").unwrap();
        assert_eq!(value, json!("w1234")); // Out of A-V range -> no-op
    }

    #[test]
    fn test_normalize_ndc11() {
        let mut value = json!("12345-6789-01");
        apply_transform(&mut value, &Transform::NormalizeNdc11, "").unwrap();
        assert_eq!(value, json!("12345678901"));

        let mut value = json!("1234-5678-90");
        apply_transform(&mut value, &Transform::NormalizeNdc11, "").unwrap();
        assert_eq!(value, json!("01234567890"));

        let mut value = json!("1234567890");
        apply_transform(&mut value, &Transform::NormalizeNdc11, "").unwrap();
        assert_eq!(value, json!("1234567890")); // Ambiguous 10-digit plain -> no-op
    }

    #[test]
    fn test_card_mask_last4_plain() {
        let mut value = json!("4111111111111111");
        apply_transform(&mut value, &Transform::CardMaskLast4, "").unwrap();
        assert_eq!(value, json!("************1111"));
    }

    #[test]
    fn test_card_mask_last4_with_spaces() {
        let mut value = json!("4111 1111 1111 1111");
        apply_transform(&mut value, &Transform::CardMaskLast4, "").unwrap();
        assert_eq!(value, json!("************1111"));
    }

    #[test]
    fn test_card_mask_last4_short() {
        // 4 or fewer digits → return digits as-is
        let mut value = json!("1234");
        apply_transform(&mut value, &Transform::CardMaskLast4, "").unwrap();
        assert_eq!(value, json!("1234"));

        let mut value = json!("12");
        apply_transform(&mut value, &Transform::CardMaskLast4, "").unwrap();
        assert_eq!(value, json!("12"));
    }

    #[test]
    fn test_card_mask_last4_non_string() {
        let mut value = json!(4111111111111111_u64);
        apply_transform(&mut value, &Transform::CardMaskLast4, "").unwrap();
        assert_eq!(value, json!(4111111111111111_u64)); // Unchanged
    }

    #[test]
    fn test_format_ssn() {
        let mut value = json!("123456789");
        apply_transform(&mut value, &Transform::FormatSsn, "").unwrap();
        assert_eq!(value, json!("123-45-6789"));

        // Already formatted (extracts digits then re-formats)
        let mut value = json!("123-45-6789");
        apply_transform(&mut value, &Transform::FormatSsn, "").unwrap();
        assert_eq!(value, json!("123-45-6789"));

        // Wrong length → no-op
        let mut value = json!("12345");
        apply_transform(&mut value, &Transform::FormatSsn, "").unwrap();
        assert_eq!(value, json!("12345"));
    }

    #[test]
    fn test_format_ein() {
        let mut value = json!("123456789");
        apply_transform(&mut value, &Transform::FormatEin, "").unwrap();
        assert_eq!(value, json!("12-3456789"));

        // Already formatted
        let mut value = json!("12-3456789");
        apply_transform(&mut value, &Transform::FormatEin, "").unwrap();
        assert_eq!(value, json!("12-3456789"));

        // Wrong length → no-op
        let mut value = json!("12345");
        apply_transform(&mut value, &Transform::FormatEin, "").unwrap();
        assert_eq!(value, json!("12345"));
    }

    #[test]
    fn test_mask_ssn() {
        let mut value = json!("123-45-6789");
        apply_transform(&mut value, &Transform::MaskSsn, "").unwrap();
        assert_eq!(value, json!("***-**-6789"));

        let mut value = json!("123456789");
        apply_transform(&mut value, &Transform::MaskSsn, "").unwrap();
        assert_eq!(value, json!("***-**-6789"));

        // Wrong length → no-op
        let mut value = json!("12345");
        apply_transform(&mut value, &Transform::MaskSsn, "").unwrap();
        assert_eq!(value, json!("12345"));
    }

    #[test]
    fn test_mask_ein() {
        let mut value = json!("12-3456789");
        apply_transform(&mut value, &Transform::MaskEin, "").unwrap();
        assert_eq!(value, json!("**-***6789"));

        let mut value = json!("123456789");
        apply_transform(&mut value, &Transform::MaskEin, "").unwrap();
        assert_eq!(value, json!("**-***6789"));

        // Wrong length → no-op
        let mut value = json!("12345");
        apply_transform(&mut value, &Transform::MaskEin, "").unwrap();
        assert_eq!(value, json!("12345"));
    }

    #[test]
    fn test_format_iban() {
        let mut value = json!("GB29NWBK60161331926819");
        apply_transform(&mut value, &Transform::FormatIban, "").unwrap();
        assert_eq!(value, json!("GB29 NWBK 6016 1331 9268 19"));

        // Already formatted (strips spaces, re-formats)
        let mut value = json!("GB29 NWBK 6016 1331 9268 19");
        apply_transform(&mut value, &Transform::FormatIban, "").unwrap();
        assert_eq!(value, json!("GB29 NWBK 6016 1331 9268 19"));

        // Lowercase → uppercased
        let mut value = json!("gb29nwbk60161331926819");
        apply_transform(&mut value, &Transform::FormatIban, "").unwrap();
        assert_eq!(value, json!("GB29 NWBK 6016 1331 9268 19"));

        // Too short → no-op
        let mut value = json!("GB29");
        apply_transform(&mut value, &Transform::FormatIban, "").unwrap();
        assert_eq!(value, json!("GB29"));
    }

    #[test]
    fn test_format_credit_card() {
        let mut value = json!("4111111111111111");
        apply_transform(&mut value, &Transform::FormatCreditCard, "").unwrap();
        assert_eq!(value, json!("4111 1111 1111 1111"));

        // With existing formatting
        let mut value = json!("4111-1111-1111-1111");
        apply_transform(&mut value, &Transform::FormatCreditCard, "").unwrap();
        assert_eq!(value, json!("4111 1111 1111 1111"));

        // Amex (15 digits)
        let mut value = json!("371449635398431");
        apply_transform(&mut value, &Transform::FormatCreditCard, "").unwrap();
        assert_eq!(value, json!("3714 4963 5398 431"));

        // Too short → no-op
        let mut value = json!("1234");
        apply_transform(&mut value, &Transform::FormatCreditCard, "").unwrap();
        assert_eq!(value, json!("1234"));
    }

    #[test]
    fn test_format_thousands() {
        let mut value = json!("1234567.89");
        apply_transform(&mut value, &Transform::format_thousands(), "").unwrap();
        assert_eq!(value, json!("1,234,567.89"));

        let mut value = json!("1234567");
        apply_transform(&mut value, &Transform::format_thousands(), "").unwrap();
        assert_eq!(value, json!("1,234,567"));

        let mut value = json!("999");
        apply_transform(&mut value, &Transform::format_thousands(), "").unwrap();
        assert_eq!(value, json!("999"));

        let mut value = json!("-1234567.89");
        apply_transform(&mut value, &Transform::format_thousands(), "").unwrap();
        assert_eq!(value, json!("-1,234,567.89"));

        // Custom separator
        let mut value = json!("1234567");
        apply_transform(
            &mut value,
            &Transform::format_thousands_with_separator("."),
            "",
        )
        .unwrap();
        assert_eq!(value, json!("1.234.567"));

        // Number type
        let mut value = json!(1234567.89);
        apply_transform(&mut value, &Transform::format_thousands(), "").unwrap();
        assert_eq!(value, json!("1,234,567.89"));
    }

    #[test]
    fn test_format_decimal() {
        let mut value = json!("3.1");
        apply_transform(&mut value, &Transform::format_decimal(2), "").unwrap();
        assert_eq!(value, json!("3.10"));

        let mut value = json!("3");
        apply_transform(&mut value, &Transform::format_decimal(2), "").unwrap();
        assert_eq!(value, json!("3.00"));

        let mut value = json!("3.14159");
        apply_transform(&mut value, &Transform::format_decimal(2), "").unwrap();
        assert_eq!(value, json!("3.14"));

        let mut value = json!("3.145");
        apply_transform(&mut value, &Transform::format_decimal(2), "").unwrap();
        assert_eq!(value, json!("3.15")); // rounds up

        let mut value = json!("-1.5");
        apply_transform(&mut value, &Transform::format_decimal(3), "").unwrap();
        assert_eq!(value, json!("-1.500"));

        // Number type
        let mut value = json!(3.1);
        apply_transform(&mut value, &Transform::format_decimal(2), "").unwrap();
        assert_eq!(value, json!("3.10"));
    }
}

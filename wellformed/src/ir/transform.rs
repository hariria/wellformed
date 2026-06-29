//! Transform definitions for the IR.
//!
//! Transforms are pure, deterministic operations that normalize data
//! before validation. They run in order and modify the value in place.

use serde::{Deserialize, Serialize};

/// A data transformation operation.
///
/// Transforms are applied in order before validation constraints are checked.
/// All transforms are pure and deterministic.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "fn", rename_all = "snake_case")]
pub enum Transform {
    /// Trim leading and trailing whitespace.
    Trim,

    /// Collapse multiple whitespace characters into single spaces.
    CollapseWhitespace,

    /// Remove all non-digit characters.
    DigitsOnly,

    /// Convert to uppercase.
    Upper,

    /// Convert to lowercase.
    Lower,

    /// Convert a money string to cents (integer).
    ///
    /// E.g., "123.45" with scale=2 becomes 12345.
    MoneyToCents {
        /// Number of decimal places (default 2).
        #[serde(default = "default_scale")]
        scale: u8,
    },

    /// Parse a date string into canonical format (YYYY-MM-DD).
    DateParse {
        /// Input format (strftime-style).
        format: String,
    },

    /// Replace occurrences of a pattern.
    Replace {
        /// Pattern to find (literal string, not regex).
        pattern: String,
        /// Replacement string.
        replacement: String,
    },

    /// Set a default value if the field is null or missing.
    Default {
        /// The default value to use.
        value: serde_json::Value,
    },

    /// Format a US phone number as (XXX) XXX-XXXX.
    ///
    /// Extracts digits; if 11 digits starting with 1 drops the leading 1;
    /// if 10 digits formats as (XXX) XXX-XXXX; otherwise returns as-is.
    PhoneUs,

    /// Normalize a phone number to E.164 format.
    ///
    /// - If input starts with `+`, keeps digits and returns `+` + digits when 7-15 digits.
    /// - If input has 10 digits, assumes US and prefixes `+1`.
    /// - If input has 11 digits starting with 1, prefixes `+`.
    /// - Otherwise no-op.
    PhoneE164,

    /// Normalize a flight number.
    ///
    /// Uppercases and removes spaces/hyphens.
    /// E.g., `"ua 123"` -> `"UA123"`, `"ual-1234a"` -> `"UAL1234A"`.
    NormalizeFlightNumber,

    /// Normalize an ICD-10 code to canonical dotted uppercase form.
    ///
    /// Removes punctuation/whitespace and uppercases.
    /// If valid plain length is > 3, inserts dot after first 3 characters.
    NormalizeIcd10,

    /// Normalize a CPT code to uppercase alphanumeric form.
    ///
    /// Removes punctuation/whitespace and uppercases.
    NormalizeCpt,

    /// Normalize an HCPCS Level II code to uppercase alphanumeric form.
    ///
    /// Removes punctuation/whitespace and uppercases.
    NormalizeHcpcs,

    /// Normalize an NDC code to 11-digit (5-4-2) form when possible.
    ///
    /// Accepts 11-digit values as-is.
    /// For hyphenated 10-digit inputs, left-pads the short segment.
    NormalizeNdc11,

    /// Mask a card number, showing only the last 4 digits.
    ///
    /// Extracts digits, replaces all but the last 4 with `*`.
    /// E.g., `"4111111111111111"` → `"************1111"`.
    /// If 4 or fewer digits, returns them as-is.
    CardMaskLast4,

    /// Format digits as SSN: XXX-XX-XXXX.
    ///
    /// Extracts digits; if exactly 9, formats with dashes. Otherwise no-op.
    FormatSsn,

    /// Format digits as EIN: XX-XXXXXXX.
    ///
    /// Extracts digits; if exactly 9, formats with dash. Otherwise no-op.
    FormatEin,

    /// Mask an SSN, showing only last 4 digits: ***-**-XXXX.
    ///
    /// Extracts digits; if exactly 9, masks first 5. Otherwise no-op.
    MaskSsn,

    /// Mask an EIN, showing only last 4 digits: **-***XXXX.
    ///
    /// Extracts digits; if exactly 9, masks first 5. Otherwise no-op.
    MaskEin,

    /// Format an IBAN with spaces every 4 characters.
    ///
    /// Strips existing spaces, uppercases, then groups into blocks of 4.
    /// E.g., `"GB29NWBK60161331926819"` → `"GB29 NWBK 6016 1331 9268 19"`.
    FormatIban,

    /// Format a credit card number with spaces every 4 digits.
    ///
    /// Extracts digits, then groups into blocks of 4.
    /// E.g., `"4111111111111111"` → `"4111 1111 1111 1111"`.
    FormatCreditCard,

    /// Format a number with thousands separators.
    ///
    /// Parses number, applies comma grouping.
    /// E.g., `"1234567.89"` → `"1,234,567.89"`.
    /// Optional `separator` (default: `,`).
    FormatThousands {
        /// Thousands separator character (default: ",").
        #[serde(default = "default_separator")]
        separator: String,
    },

    /// Format a number to a fixed number of decimal places.
    ///
    /// E.g., `"3.1"` with places=2 → `"3.10"`, `"3"` with places=2 → `"3.00"`.
    FormatDecimal {
        /// Number of decimal places.
        places: u8,
    },
}

fn default_scale() -> u8 {
    2
}

fn default_separator() -> String {
    ",".to_string()
}

impl Transform {
    /// Create a Trim transform.
    pub fn trim() -> Self {
        Self::Trim
    }

    /// Create a DigitsOnly transform.
    pub fn digits_only() -> Self {
        Self::DigitsOnly
    }

    /// Create an Upper transform.
    pub fn upper() -> Self {
        Self::Upper
    }

    /// Create a Lower transform.
    pub fn lower() -> Self {
        Self::Lower
    }

    /// Create a MoneyToCents transform with default scale.
    pub fn money_to_cents() -> Self {
        Self::MoneyToCents { scale: 2 }
    }

    /// Create a MoneyToCents transform with custom scale.
    pub fn money_to_cents_with_scale(scale: u8) -> Self {
        Self::MoneyToCents { scale }
    }

    /// Create a DateParse transform.
    pub fn date_parse(format: impl Into<String>) -> Self {
        Self::DateParse {
            format: format.into(),
        }
    }

    /// Create a CollapseWhitespace transform.
    pub fn collapse_whitespace() -> Self {
        Self::CollapseWhitespace
    }

    /// Create a Replace transform.
    pub fn replace(pattern: impl Into<String>, replacement: impl Into<String>) -> Self {
        Self::Replace {
            pattern: pattern.into(),
            replacement: replacement.into(),
        }
    }

    /// Create a Default transform.
    pub fn default_value(value: serde_json::Value) -> Self {
        Self::Default { value }
    }

    /// Create a PhoneUs transform.
    pub fn phone_us() -> Self {
        Self::PhoneUs
    }

    /// Create a PhoneE164 transform.
    pub fn phone_e164() -> Self {
        Self::PhoneE164
    }

    /// Create a NormalizeFlightNumber transform.
    pub fn normalize_flight_number() -> Self {
        Self::NormalizeFlightNumber
    }

    /// Create a NormalizeIcd10 transform.
    pub fn normalize_icd10() -> Self {
        Self::NormalizeIcd10
    }

    /// Create a NormalizeCpt transform.
    pub fn normalize_cpt() -> Self {
        Self::NormalizeCpt
    }

    /// Create a NormalizeHcpcs transform.
    pub fn normalize_hcpcs() -> Self {
        Self::NormalizeHcpcs
    }

    /// Create a NormalizeNdc11 transform.
    pub fn normalize_ndc11() -> Self {
        Self::NormalizeNdc11
    }

    /// Create a CardMaskLast4 transform.
    pub fn card_mask_last4() -> Self {
        Self::CardMaskLast4
    }

    /// Create a FormatSsn transform.
    pub fn format_ssn() -> Self {
        Self::FormatSsn
    }

    /// Create a FormatEin transform.
    pub fn format_ein() -> Self {
        Self::FormatEin
    }

    /// Create a MaskSsn transform.
    pub fn mask_ssn() -> Self {
        Self::MaskSsn
    }

    /// Create a MaskEin transform.
    pub fn mask_ein() -> Self {
        Self::MaskEin
    }

    /// Create a FormatIban transform.
    pub fn format_iban() -> Self {
        Self::FormatIban
    }

    /// Create a FormatCreditCard transform.
    pub fn format_credit_card() -> Self {
        Self::FormatCreditCard
    }

    /// Create a FormatThousands transform with default separator.
    pub fn format_thousands() -> Self {
        Self::FormatThousands {
            separator: ",".to_string(),
        }
    }

    /// Create a FormatThousands transform with custom separator.
    pub fn format_thousands_with_separator(separator: impl Into<String>) -> Self {
        Self::FormatThousands {
            separator: separator.into(),
        }
    }

    /// Create a FormatDecimal transform.
    pub fn format_decimal(places: u8) -> Self {
        Self::FormatDecimal { places }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serde_trim() {
        let t = Transform::trim();
        let json = serde_json::to_string(&t).unwrap();
        assert_eq!(json, r#"{"fn":"trim"}"#);

        let parsed: Transform = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, t);
    }

    #[test]
    fn test_serde_money_to_cents() {
        let t = Transform::money_to_cents_with_scale(3);
        let json = serde_json::to_string(&t).unwrap();
        assert_eq!(json, r#"{"fn":"money_to_cents","scale":3}"#);

        let parsed: Transform = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, t);
    }

    #[test]
    fn test_serde_money_to_cents_default_scale() {
        let json = r#"{"fn":"money_to_cents"}"#;
        let parsed: Transform = serde_json::from_str(json).unwrap();
        assert_eq!(parsed, Transform::MoneyToCents { scale: 2 });
    }

    #[test]
    fn test_serde_date_parse() {
        let t = Transform::date_parse("%m/%d/%Y");
        let json = serde_json::to_string(&t).unwrap();
        assert!(json.contains("date_parse"));
        assert!(json.contains("%m/%d/%Y"));
    }

    #[test]
    fn test_serde_replace() {
        let t = Transform::replace("-", "");
        let json = serde_json::to_string(&t).unwrap();
        let parsed: Transform = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, t);
    }

    #[test]
    fn test_serde_phone_us() {
        let t = Transform::phone_us();
        let json = serde_json::to_string(&t).unwrap();
        assert_eq!(json, r#"{"fn":"phone_us"}"#);

        let parsed: Transform = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, t);
    }

    #[test]
    fn test_serde_phone_e164() {
        let t = Transform::phone_e164();
        let json = serde_json::to_string(&t).unwrap();
        assert_eq!(json, r#"{"fn":"phone_e164"}"#);

        let parsed: Transform = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, t);
    }

    #[test]
    fn test_serde_normalize_flight_number() {
        let t = Transform::normalize_flight_number();
        let json = serde_json::to_string(&t).unwrap();
        assert_eq!(json, r#"{"fn":"normalize_flight_number"}"#);

        let parsed: Transform = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, t);
    }

    #[test]
    fn test_serde_normalize_icd10() {
        let t = Transform::normalize_icd10();
        let json = serde_json::to_string(&t).unwrap();
        assert_eq!(json, r#"{"fn":"normalize_icd10"}"#);

        let parsed: Transform = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, t);
    }

    #[test]
    fn test_serde_normalize_cpt() {
        let t = Transform::normalize_cpt();
        let json = serde_json::to_string(&t).unwrap();
        assert_eq!(json, r#"{"fn":"normalize_cpt"}"#);

        let parsed: Transform = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, t);
    }

    #[test]
    fn test_serde_normalize_hcpcs() {
        let t = Transform::normalize_hcpcs();
        let json = serde_json::to_string(&t).unwrap();
        assert_eq!(json, r#"{"fn":"normalize_hcpcs"}"#);

        let parsed: Transform = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, t);
    }

    #[test]
    fn test_serde_normalize_ndc11() {
        let t = Transform::normalize_ndc11();
        let json = serde_json::to_string(&t).unwrap();
        assert_eq!(json, r#"{"fn":"normalize_ndc11"}"#);

        let parsed: Transform = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, t);
    }

    #[test]
    fn test_serde_card_mask_last4() {
        let t = Transform::card_mask_last4();
        let json = serde_json::to_string(&t).unwrap();
        assert_eq!(json, r#"{"fn":"card_mask_last4"}"#);

        let parsed: Transform = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, t);
    }

    #[test]
    fn test_serde_format_ssn() {
        let t = Transform::format_ssn();
        let json = serde_json::to_string(&t).unwrap();
        assert_eq!(json, r#"{"fn":"format_ssn"}"#);
        let parsed: Transform = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, t);
    }

    #[test]
    fn test_serde_format_ein() {
        let t = Transform::format_ein();
        let json = serde_json::to_string(&t).unwrap();
        assert_eq!(json, r#"{"fn":"format_ein"}"#);
        let parsed: Transform = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, t);
    }

    #[test]
    fn test_serde_mask_ssn() {
        let t = Transform::mask_ssn();
        let json = serde_json::to_string(&t).unwrap();
        assert_eq!(json, r#"{"fn":"mask_ssn"}"#);
        let parsed: Transform = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, t);
    }

    #[test]
    fn test_serde_mask_ein() {
        let t = Transform::mask_ein();
        let json = serde_json::to_string(&t).unwrap();
        assert_eq!(json, r#"{"fn":"mask_ein"}"#);
        let parsed: Transform = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, t);
    }

    #[test]
    fn test_serde_format_iban() {
        let t = Transform::format_iban();
        let json = serde_json::to_string(&t).unwrap();
        assert_eq!(json, r#"{"fn":"format_iban"}"#);
        let parsed: Transform = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, t);
    }

    #[test]
    fn test_serde_format_credit_card() {
        let t = Transform::format_credit_card();
        let json = serde_json::to_string(&t).unwrap();
        assert_eq!(json, r#"{"fn":"format_credit_card"}"#);
        let parsed: Transform = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, t);
    }

    #[test]
    fn test_serde_format_thousands() {
        let t = Transform::format_thousands();
        let json = serde_json::to_string(&t).unwrap();
        assert_eq!(json, r#"{"fn":"format_thousands","separator":","}"#);
        let parsed: Transform = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, t);
    }

    #[test]
    fn test_serde_format_thousands_default_separator() {
        let json = r#"{"fn":"format_thousands"}"#;
        let parsed: Transform = serde_json::from_str(json).unwrap();
        assert_eq!(
            parsed,
            Transform::FormatThousands {
                separator: ",".to_string()
            }
        );
    }

    #[test]
    fn test_serde_format_decimal() {
        let t = Transform::format_decimal(2);
        let json = serde_json::to_string(&t).unwrap();
        assert_eq!(json, r#"{"fn":"format_decimal","places":2}"#);
        let parsed: Transform = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, t);
    }
}

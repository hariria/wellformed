//! Financial identifier validation predicates.
//!
//! Validates CUSIP numbers, ABA routing numbers, and other financial identifiers.

use super::registry::{NamedPredicate, PredicateRegistry};
use serde_json::Value;
use std::sync::Arc;

/// Register all financial identifier predicates.
pub fn register_financial_predicates(registry: &mut PredicateRegistry) {
    registry.register(Arc::new(IsItinPredicate));
    registry.register(Arc::new(IsAtinPredicate));
    registry.register(Arc::new(IsCusipPredicate));
    registry.register(Arc::new(IsAbaRoutingPredicate));
    registry.register(Arc::new(IsMccPredicate));
    registry.register(Arc::new(IsAccountNumberPredicate));
    registry.register(Arc::new(IsCreditCardPredicate));
    registry.register(Arc::new(IsCvvPredicate));
    registry.register(Arc::new(IsCardExpiryPredicate));
    registry.register(Arc::new(IsIbanPredicate));
    registry.register(Arc::new(IsBicPredicate));
    registry.register(Arc::new(IsSwiftPredicate));
    registry.register(Arc::new(IsVinPredicate));
    registry.register(Arc::new(IsUpcPredicate));
    registry.register(Arc::new(IsEanPredicate));
    registry.register(Arc::new(IsIsbnPredicate));
}

// ============================================================================
// ITIN (Individual Taxpayer Identification Number)
// ============================================================================

/// Validate an ITIN specifically.
///
/// ITINs are 9 digits starting with 9, with 7 or 8 in the 4th position.
/// Format: 9XX-7X-XXXX or 9XX-8X-XXXX
struct IsItinPredicate;

impl NamedPredicate for IsItinPredicate {
    fn name(&self) -> &str {
        "is_itin"
    }

    fn evaluate(&self, value: &Value, _args: &Value) -> bool {
        let s = match value.as_str() {
            Some(s) => s,
            None => return false,
        };
        let digits: String = s.chars().filter(|c| c.is_ascii_digit()).collect();
        digits.len() == 9 && is_valid_itin(&digits)
    }
}

fn is_valid_itin(digits: &str) -> bool {
    if digits.len() != 9 {
        return false;
    }

    // ITIN starts with 9 and has 7 or 8 in the 4th position
    let first = digits.chars().next().unwrap();
    let fourth = digits.chars().nth(3).unwrap();

    first == '9' && (fourth == '7' || fourth == '8')
}

// ============================================================================
// ATIN (Adoption Taxpayer Identification Number)
// ============================================================================

/// Validate an ATIN specifically.
///
/// ATINs are 9 digits starting with 9, with 93 in positions 4-5.
/// Format: 9XX-93-XXXX
struct IsAtinPredicate;

impl NamedPredicate for IsAtinPredicate {
    fn name(&self) -> &str {
        "is_atin"
    }

    fn evaluate(&self, value: &Value, _args: &Value) -> bool {
        let s = match value.as_str() {
            Some(s) => s,
            None => return false,
        };
        let digits: String = s.chars().filter(|c| c.is_ascii_digit()).collect();
        digits.len() == 9 && is_valid_atin(&digits)
    }
}

fn is_valid_atin(digits: &str) -> bool {
    if digits.len() != 9 {
        return false;
    }

    // ATIN starts with 9 and has 93 in positions 4-5
    let first = digits.chars().next().unwrap();
    let fourth_fifth = &digits[3..5];

    first == '9' && fourth_fifth == "93"
}

// ============================================================================
// CUSIP (Committee on Uniform Securities Identification Procedures)
// ============================================================================

/// Validate a CUSIP number.
///
/// CUSIP is a 9-character alphanumeric identifier for securities.
/// Format: 6 chars (issuer) + 2 chars (issue) + 1 check digit
///
/// Used on 1099-B, 1099-INT, 1099-DIV for identifying securities.
struct IsCusipPredicate;

impl NamedPredicate for IsCusipPredicate {
    fn name(&self) -> &str {
        "is_cusip"
    }

    fn evaluate(&self, value: &Value, args: &Value) -> bool {
        let s = match value.as_str() {
            Some(s) => s.trim().to_uppercase(),
            None => return false,
        };

        // Must be exactly 9 characters
        if s.len() != 9 {
            return false;
        }

        // Must be alphanumeric (letters and digits only)
        if !s.chars().all(|c| c.is_ascii_alphanumeric()) {
            return false;
        }

        // Check if we should validate the check digit
        let validate_checksum = args
            .get("validate_checksum")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        if validate_checksum {
            validate_cusip_checksum(&s)
        } else {
            true
        }
    }
}

/// Validate CUSIP check digit using the Luhn-like algorithm.
///
/// The algorithm converts letters to numbers (A=10, B=11, ..., Z=35),
/// then applies a modified Luhn algorithm.
fn validate_cusip_checksum(cusip: &str) -> bool {
    let chars: Vec<char> = cusip.chars().collect();
    if chars.len() != 9 {
        return false;
    }

    let mut sum = 0;
    for (i, c) in chars[..8].iter().enumerate() {
        let val = if c.is_ascii_digit() {
            c.to_digit(10).unwrap() as usize
        } else if c.is_ascii_uppercase() {
            // A=10, B=11, ..., Z=35
            (*c as usize) - ('A' as usize) + 10
        } else {
            return false;
        };

        // Double every second digit (0-indexed, so odd positions)
        let val = if i % 2 == 1 { val * 2 } else { val };

        // Sum the digits
        sum += val / 10 + val % 10;
    }

    let check_digit = (10 - (sum % 10)) % 10;

    // Compare with the 9th character
    let expected = chars[8].to_digit(10);
    expected == Some(check_digit as u32)
}

// ============================================================================
// ABA Routing Number
// ============================================================================

/// Validate an ABA routing transit number (RTN).
///
/// ABA RTNs are 9-digit numbers used for US bank routing.
/// The first two digits indicate the Federal Reserve district.
/// Uses a weighted checksum algorithm.
///
/// Used on 1099-INT for payer bank routing.
struct IsAbaRoutingPredicate;

impl NamedPredicate for IsAbaRoutingPredicate {
    fn name(&self) -> &str {
        "is_aba_routing"
    }

    fn evaluate(&self, value: &Value, _args: &Value) -> bool {
        let s = match value.as_str() {
            Some(s) => s.trim(),
            None => return false,
        };

        // Extract digits only
        let digits: String = s.chars().filter(|c| c.is_ascii_digit()).collect();

        // Must be exactly 9 digits
        if digits.len() != 9 {
            return false;
        }

        // Validate Federal Reserve district (first 2 digits)
        let prefix: u32 = digits[0..2].parse().unwrap_or(0);
        // Valid prefixes: 00-12, 21-32, 61-72, 80
        let valid_prefix = matches!(prefix, 0..=12 | 21..=32 | 61..=72 | 80);
        if !valid_prefix {
            return false;
        }

        // Validate checksum using weighted sum
        // Formula: (3(d1+d4+d7) + 7(d2+d5+d8) + (d3+d6+d9)) mod 10 == 0
        let d: Vec<u32> = digits.chars().map(|c| c.to_digit(10).unwrap()).collect();

        let checksum = 3 * (d[0] + d[3] + d[6]) + 7 * (d[1] + d[4] + d[7]) + (d[2] + d[5] + d[8]);

        checksum % 10 == 0
    }
}

// ============================================================================
// MCC (Merchant Category Code)
// ============================================================================

/// Validate a Merchant Category Code (MCC).
///
/// MCCs are exactly 4 digits used on 1099-K.
struct IsMccPredicate;

impl NamedPredicate for IsMccPredicate {
    fn name(&self) -> &str {
        "is_mcc"
    }

    fn evaluate(&self, value: &Value, _args: &Value) -> bool {
        let s = match value.as_str() {
            Some(s) => s.trim(),
            None => return false,
        };

        // Must be exactly 4 digits
        s.len() == 4 && s.chars().all(|c| c.is_ascii_digit())
    }
}

// ============================================================================
// Account Number
// ============================================================================

/// Validate an account number format.
///
/// Account numbers are alphanumeric, 1-30 characters.
/// Args: { "min_len": 1, "max_len": 30 }
struct IsAccountNumberPredicate;

impl NamedPredicate for IsAccountNumberPredicate {
    fn name(&self) -> &str {
        "is_account_number"
    }

    fn evaluate(&self, value: &Value, args: &Value) -> bool {
        let s = match value.as_str() {
            Some(s) => s.trim(),
            None => return false,
        };

        let min_len = args.get("min_len").and_then(|v| v.as_u64()).unwrap_or(1) as usize;

        let max_len = args.get("max_len").and_then(|v| v.as_u64()).unwrap_or(30) as usize;

        if s.len() < min_len || s.len() > max_len {
            return false;
        }

        // Must be alphanumeric (some accounts allow hyphens)
        let allow_hyphens = args
            .get("allow_hyphens")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        s.chars()
            .all(|c| c.is_ascii_alphanumeric() || (allow_hyphens && c == '-'))
    }
}

// ============================================================================
// Credit Card Number
// ============================================================================

/// Validate a credit card number using Luhn check and IIN prefix/length rules.
///
/// Supported networks: Visa, Mastercard, Amex, Discover.
/// Args: `{ "network": "visa" | "mastercard" | "amex" | "discover" }` (optional)
struct IsCreditCardPredicate;

impl NamedPredicate for IsCreditCardPredicate {
    fn name(&self) -> &str {
        "is_credit_card"
    }

    fn evaluate(&self, value: &Value, args: &Value) -> bool {
        let s = match value.as_str() {
            Some(s) => s,
            None => return false,
        };

        let digits: String = s.chars().filter(|c| c.is_ascii_digit()).collect();

        // Luhn check first
        if !super::tin::luhn_check(&digits) {
            return false;
        }

        let network = args.get("network").and_then(|v| v.as_str());

        match network {
            Some("visa") => is_visa(&digits),
            Some("mastercard") => is_mastercard(&digits),
            Some("amex") => is_amex(&digits),
            Some("discover") => is_discover(&digits),
            _ => {
                is_visa(&digits)
                    || is_mastercard(&digits)
                    || is_amex(&digits)
                    || is_discover(&digits)
            }
        }
    }
}

fn is_visa(digits: &str) -> bool {
    digits.len() == 16 && digits.starts_with('4')
}

fn is_mastercard(digits: &str) -> bool {
    if digits.len() != 16 {
        return false;
    }
    // Prefix 51-55
    if let Ok(prefix2) = digits[0..2].parse::<u32>() {
        if (51..=55).contains(&prefix2) {
            return true;
        }
    }
    // Prefix 2221-2720
    if let Ok(prefix4) = digits[0..4].parse::<u32>() {
        if (2221..=2720).contains(&prefix4) {
            return true;
        }
    }
    false
}

fn is_amex(digits: &str) -> bool {
    digits.len() == 15 && (digits.starts_with("34") || digits.starts_with("37"))
}

fn is_discover(digits: &str) -> bool {
    if digits.len() != 16 {
        return false;
    }
    if digits.starts_with("6011") || digits.starts_with("65") {
        return true;
    }
    // 644-649
    if let Ok(prefix3) = digits[0..3].parse::<u32>() {
        if (644..=649).contains(&prefix3) {
            return true;
        }
    }
    false
}

// ============================================================================
// CVV (Card Verification Value)
// ============================================================================

/// Validate a CVV code.
///
/// All digits, length-based:
/// - No `network` arg → 3 or 4 digits
/// - `network: "amex"` → exactly 4 digits
/// - Any other `network` → exactly 3 digits
struct IsCvvPredicate;

impl NamedPredicate for IsCvvPredicate {
    fn name(&self) -> &str {
        "is_cvv"
    }

    fn evaluate(&self, value: &Value, args: &Value) -> bool {
        let s = match value.as_str() {
            Some(s) => s.trim(),
            None => return false,
        };

        if !s.chars().all(|c| c.is_ascii_digit()) || s.is_empty() {
            return false;
        }

        let network = args.get("network").and_then(|v| v.as_str());

        match network {
            Some("amex") => s.len() == 4,
            Some(_) => s.len() == 3,
            None => s.len() == 3 || s.len() == 4,
        }
    }
}

// ============================================================================
// Card Expiry (MM/YY or MM/YYYY)
// ============================================================================

/// Validate a card expiry date.
///
/// Format: `MM/YY` or `MM/YYYY` (slash-separated).
/// Month must be 01-12.
/// Args: `{ "reject_expired": true }` to compare against current date.
struct IsCardExpiryPredicate;

impl NamedPredicate for IsCardExpiryPredicate {
    fn name(&self) -> &str {
        "is_card_expiry"
    }

    fn evaluate(&self, value: &Value, args: &Value) -> bool {
        let s = match value.as_str() {
            Some(s) => s.trim(),
            None => return false,
        };

        let parts: Vec<&str> = s.split('/').collect();
        if parts.len() != 2 {
            return false;
        }

        let month_str = parts[0];
        let year_str = parts[1];

        // Month must be exactly 2 digits
        if month_str.len() != 2 || !month_str.chars().all(|c| c.is_ascii_digit()) {
            return false;
        }

        // Year must be 2 or 4 digits
        if (year_str.len() != 2 && year_str.len() != 4)
            || !year_str.chars().all(|c| c.is_ascii_digit())
        {
            return false;
        }

        let month: u32 = match month_str.parse() {
            Ok(m) => m,
            Err(_) => return false,
        };

        if !(1..=12).contains(&month) {
            return false;
        }

        let reject_expired = args
            .get("reject_expired")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        if reject_expired {
            let year: u32 = match year_str.parse() {
                Ok(y) => y,
                Err(_) => return false,
            };

            // Normalize 2-digit year to 4-digit
            let full_year = if year < 100 { 2000 + year } else { year };

            // Get current year/month using SystemTime + civil_from_days
            let (cur_year, cur_month) = current_year_month();

            // Card expires at the end of the given month
            if full_year < cur_year || (full_year == cur_year && month < cur_month) {
                return false;
            }
        }

        true
    }
}

/// Get current year and month from SystemTime using Howard Hinnant's civil_from_days algorithm.
fn current_year_month() -> (u32, u32) {
    use std::time::{SystemTime, UNIX_EPOCH};

    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let days = (secs / 86400) as i64;

    // Howard Hinnant's civil_from_days algorithm
    let z = days + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u32;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = (yoe as i64) + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = if m <= 2 { y + 1 } else { y };

    (year as u32, m)
}

// ============================================================================
// IBAN (International Bank Account Number)
// ============================================================================

/// Validate an IBAN (International Bank Account Number).
///
/// Checks:
/// - 2-letter country code + 2 check digits + BBAN
/// - Country-specific length (when known)
/// - Mod-97 checksum validation
///
/// Args: `{ "country": "DE" }` (optional) to restrict to a specific country.
struct IsIbanPredicate;

impl NamedPredicate for IsIbanPredicate {
    fn name(&self) -> &str {
        "is_iban"
    }

    fn evaluate(&self, value: &Value, args: &Value) -> bool {
        let s = match value.as_str() {
            Some(s) => s,
            None => return false,
        };

        let mut iban = Vec::with_capacity(s.len().min(34));
        for byte in s.bytes() {
            if byte.is_ascii_whitespace() {
                continue;
            }

            let upper = byte.to_ascii_uppercase();
            if !upper.is_ascii_alphanumeric() {
                return false;
            }

            iban.push(upper);
            if iban.len() > 34 {
                return false;
            }
        }

        // Minimum length: 2 country + 2 check + at least 1 BBAN = 5
        if iban.len() < 5 {
            return false;
        }

        // First 2 chars must be letters (country code)
        if !iban[0].is_ascii_uppercase() || !iban[1].is_ascii_uppercase() {
            return false;
        }
        let country = std::str::from_utf8(&iban[0..2]).unwrap();

        // Next 2 chars must be digits (check digits)
        if !iban[2].is_ascii_digit() || !iban[3].is_ascii_digit() {
            return false;
        }

        // Optional country filter
        if let Some(required_country) = args.get("country").and_then(|v| v.as_str()) {
            if !country.eq_ignore_ascii_case(required_country) {
                return false;
            }
        }

        // Check country-specific length if known
        if let Some(expected_len) = iban_country_length(country) {
            if iban.len() != expected_len {
                return false;
            }
        }

        // Mod-97 checksum: move first 4 chars to end, convert letters to numbers, mod 97 == 1
        iban_mod97(&iban)
    }
}

/// Mod-97 IBAN checksum validation.
///
/// Move first 4 characters to end, convert letters to two-digit numbers
/// (A=10, B=11, ..., Z=35), then check the resulting number mod 97 == 1.
fn iban_mod97(iban: &[u8]) -> bool {
    let mut remainder: u32 = 0;

    for &byte in &iban[4..] {
        remainder = iban_mod97_step(byte, remainder);
    }
    for &byte in &iban[..4] {
        remainder = iban_mod97_step(byte, remainder);
    }

    remainder == 1
}

fn iban_mod97_step(byte: u8, remainder: u32) -> u32 {
    if byte.is_ascii_digit() {
        return (remainder * 10 + u32::from(byte - b'0')) % 97;
    }

    // Letter becomes two digits: A=10, B=11, ..., Z=35.
    (remainder * 100 + u32::from(byte - b'A' + 10)) % 97
}

/// Get expected IBAN length for a given country code.
fn iban_country_length(country: &str) -> Option<usize> {
    match country {
        "AL" => Some(28),
        "AD" => Some(24),
        "AT" => Some(20),
        "AZ" => Some(28),
        "BH" => Some(22),
        "BY" => Some(28),
        "BE" => Some(16),
        "BA" => Some(20),
        "BR" => Some(29),
        "BG" => Some(22),
        "CR" => Some(22),
        "HR" => Some(21),
        "CY" => Some(28),
        "CZ" => Some(24),
        "DK" => Some(18),
        "DO" => Some(28),
        "EE" => Some(20),
        "FO" => Some(18),
        "FI" => Some(18),
        "FR" => Some(27),
        "GE" => Some(22),
        "DE" => Some(22),
        "GI" => Some(23),
        "GR" => Some(27),
        "GL" => Some(18),
        "GT" => Some(28),
        "HU" => Some(28),
        "IS" => Some(26),
        "IQ" => Some(23),
        "IE" => Some(22),
        "IL" => Some(23),
        "IT" => Some(27),
        "JO" => Some(30),
        "KZ" => Some(20),
        "XK" => Some(20),
        "KW" => Some(30),
        "LV" => Some(21),
        "LB" => Some(28),
        "LI" => Some(21),
        "LT" => Some(20),
        "LU" => Some(20),
        "MK" => Some(19),
        "MT" => Some(31),
        "MR" => Some(27),
        "MU" => Some(30),
        "MC" => Some(27),
        "MD" => Some(24),
        "ME" => Some(22),
        "NL" => Some(18),
        "NO" => Some(15),
        "PK" => Some(24),
        "PS" => Some(29),
        "PL" => Some(28),
        "PT" => Some(25),
        "QA" => Some(29),
        "RO" => Some(24),
        "LC" => Some(32),
        "SM" => Some(27),
        "SA" => Some(24),
        "RS" => Some(22),
        "SC" => Some(31),
        "SK" => Some(24),
        "SI" => Some(19),
        "ES" => Some(24),
        "SE" => Some(24),
        "CH" => Some(21),
        "TN" => Some(24),
        "TR" => Some(26),
        "AE" => Some(23),
        "GB" => Some(22),
        "VA" => Some(22),
        "VG" => Some(24),
        "UA" => Some(29),
        _ => None,
    }
}

// ============================================================================
// BIC / SWIFT Code
// ============================================================================

/// Validate a BIC (Bank Identifier Code) / SWIFT code.
///
/// Format: `BBBBCCLL[bbb]`
/// - BBBB: 4-letter bank code (alpha only)
/// - CC: 2-letter country code (alpha only)
/// - LL: 2-character location code (alphanumeric)
/// - bbb: optional 3-character branch code (alphanumeric)
///
/// Total length: 8 or 11 characters.
struct IsBicPredicate;

impl NamedPredicate for IsBicPredicate {
    fn name(&self) -> &str {
        "is_bic"
    }

    fn evaluate(&self, value: &Value, _args: &Value) -> bool {
        let s = match value.as_str() {
            Some(s) => s.trim().to_uppercase(),
            None => return false,
        };

        // Must be 8 or 11 characters
        if s.len() != 8 && s.len() != 11 {
            return false;
        }

        // Bank code: first 4 chars must be letters
        if !s[0..4].chars().all(|c| c.is_ascii_uppercase()) {
            return false;
        }

        // Country code: chars 4-5 must be letters
        if !s[4..6].chars().all(|c| c.is_ascii_uppercase()) {
            return false;
        }

        // Location code: chars 6-7 must be alphanumeric
        if !s[6..8].chars().all(|c| c.is_ascii_alphanumeric()) {
            return false;
        }

        // Branch code (if present): chars 8-10 must be alphanumeric
        if s.len() == 11 && !s[8..11].chars().all(|c| c.is_ascii_alphanumeric()) {
            return false;
        }

        true
    }
}

/// Backwards-compat alias: `is_swift` → same logic as `is_bic`.
struct IsSwiftPredicate;

impl NamedPredicate for IsSwiftPredicate {
    fn name(&self) -> &str {
        "is_swift"
    }

    fn evaluate(&self, value: &Value, args: &Value) -> bool {
        IsBicPredicate.evaluate(value, args)
    }
}

// ============================================================================
// VIN (Vehicle Identification Number)
// ============================================================================

/// Validate a Vehicle Identification Number.
///
/// - 17 characters, alphanumeric (excluding I, O, Q)
/// - Position 9 is a check digit (mod 11, where X = 10)
///
/// Args: `{ "validate_checksum": bool }` (default true)
struct IsVinPredicate;

impl NamedPredicate for IsVinPredicate {
    fn name(&self) -> &str {
        "is_vin"
    }

    fn evaluate(&self, value: &Value, args: &Value) -> bool {
        let s = match value.as_str() {
            Some(s) => s.trim().to_uppercase(),
            None => return false,
        };

        if s.len() != 17 {
            return false;
        }

        // VIN charset: 0-9, A-Z except I, O, Q
        if !s
            .chars()
            .all(|c| (c.is_ascii_alphanumeric()) && c != 'I' && c != 'O' && c != 'Q')
        {
            return false;
        }

        let validate_checksum = args
            .get("validate_checksum")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        if !validate_checksum {
            return true;
        }

        // VIN check digit (position 9, 0-indexed 8)
        vin_check_digit(&s)
    }
}

/// Transliteration values for VIN characters.
fn vin_transliterate(c: char) -> Option<u32> {
    match c {
        '0'..='9' => Some(c as u32 - '0' as u32),
        'A' | 'J' => Some(1),
        'B' | 'K' | 'S' => Some(2),
        'C' | 'L' | 'T' => Some(3),
        'D' | 'M' | 'U' => Some(4),
        'E' | 'N' | 'V' => Some(5),
        'F' | 'W' => Some(6),
        'G' | 'P' | 'X' => Some(7),
        'H' | 'Y' => Some(8),
        'R' | 'Z' => Some(9),
        _ => None, // I, O, Q are invalid
    }
}

/// VIN position weights.
const VIN_WEIGHTS: [u32; 17] = [8, 7, 6, 5, 4, 3, 2, 10, 0, 9, 8, 7, 6, 5, 4, 3, 2];

fn vin_check_digit(vin: &str) -> bool {
    let mut sum: u32 = 0;
    for (i, c) in vin.chars().enumerate() {
        if i == 8 {
            continue; // Skip check digit position
        }
        let val = match vin_transliterate(c) {
            Some(v) => v,
            None => return false,
        };
        sum += val * VIN_WEIGHTS[i];
    }

    let remainder = sum % 11;
    let expected = if remainder == 10 {
        'X'
    } else {
        char::from(b'0' + remainder as u8)
    };
    vin.chars().nth(8) == Some(expected)
}

// ============================================================================
// UPC (Universal Product Code)
// ============================================================================

/// Validate a UPC-A barcode.
///
/// - 12 digits
/// - Check digit: alternating weights 3,1 from position 1, mod 10
struct IsUpcPredicate;

impl NamedPredicate for IsUpcPredicate {
    fn name(&self) -> &str {
        "is_upc"
    }

    fn evaluate(&self, value: &Value, _args: &Value) -> bool {
        let s = match value.as_str() {
            Some(s) => s.trim(),
            None => return false,
        };

        let digits: Vec<u32> = s.chars().filter_map(|c| c.to_digit(10)).collect();
        if digits.len() != 12 {
            return false;
        }

        // Also check that original string (minus hyphens/spaces) is all digits of length 12
        let cleaned: String = s
            .chars()
            .filter(|c| !c.is_whitespace() && *c != '-')
            .collect();
        if cleaned.len() != 12 || !cleaned.chars().all(|c| c.is_ascii_digit()) {
            return false;
        }

        // UPC check: odd positions (1-indexed) × 3, even × 1
        let mut sum: u32 = 0;
        for (i, &d) in digits.iter().enumerate() {
            if i % 2 == 0 {
                sum += d * 3;
            } else {
                sum += d;
            }
        }
        sum % 10 == 0
    }
}

// ============================================================================
// EAN (European Article Number)
// ============================================================================

/// Validate an EAN-13 barcode.
///
/// - 13 digits
/// - Check digit: alternating weights 1,3, mod 10
struct IsEanPredicate;

impl NamedPredicate for IsEanPredicate {
    fn name(&self) -> &str {
        "is_ean"
    }

    fn evaluate(&self, value: &Value, _args: &Value) -> bool {
        let s = match value.as_str() {
            Some(s) => s.trim(),
            None => return false,
        };

        let digits: Vec<u32> = s.chars().filter_map(|c| c.to_digit(10)).collect();
        if digits.len() != 13 {
            return false;
        }

        let cleaned: String = s
            .chars()
            .filter(|c| !c.is_whitespace() && *c != '-')
            .collect();
        if cleaned.len() != 13 || !cleaned.chars().all(|c| c.is_ascii_digit()) {
            return false;
        }

        // EAN-13 check: positions alternate weight 1, 3
        let mut sum: u32 = 0;
        for (i, &d) in digits.iter().enumerate() {
            if i % 2 == 0 {
                sum += d;
            } else {
                sum += d * 3;
            }
        }
        sum % 10 == 0
    }
}

// ============================================================================
// ISBN (International Standard Book Number)
// ============================================================================

/// Validate an ISBN-10 or ISBN-13.
///
/// - ISBN-10: 10 chars (digits + optional X check), weighted sum mod 11
/// - ISBN-13: 13 digits, same check as EAN-13
///
/// Args: `{ "version": 10 | 13 }` (optional, default accepts both)
struct IsIsbnPredicate;

impl NamedPredicate for IsIsbnPredicate {
    fn name(&self) -> &str {
        "is_isbn"
    }

    fn evaluate(&self, value: &Value, args: &Value) -> bool {
        let s = match value.as_str() {
            Some(s) => s.trim(),
            None => return false,
        };

        let version = args.get("version").and_then(|v| v.as_u64());

        // Strip hyphens and spaces for checking
        let cleaned: String = s
            .chars()
            .filter(|c| *c != '-' && !c.is_whitespace())
            .collect();

        match version {
            Some(10) => isbn10_check(&cleaned),
            Some(13) => isbn13_check(&cleaned),
            _ => isbn10_check(&cleaned) || isbn13_check(&cleaned),
        }
    }
}

fn isbn10_check(s: &str) -> bool {
    if s.len() != 10 {
        return false;
    }

    // First 9 must be digits, last can be digit or X
    let chars: Vec<char> = s.chars().collect();
    if !chars[..9].iter().all(|c| c.is_ascii_digit()) {
        return false;
    }
    let last = chars[9];
    if !last.is_ascii_digit() && last != 'X' && last != 'x' {
        return false;
    }

    // Weighted sum: position 1 gets weight 10, position 2 gets 9, ..., position 10 gets 1
    let mut sum: u32 = 0;
    for (i, &c) in chars.iter().enumerate() {
        let val = if c == 'X' || c == 'x' {
            10
        } else {
            c.to_digit(10).unwrap()
        };
        sum += val * (10 - i as u32);
    }
    sum % 11 == 0
}

fn isbn13_check(s: &str) -> bool {
    if s.len() != 13 {
        return false;
    }
    if !s.chars().all(|c| c.is_ascii_digit()) {
        return false;
    }

    // Must start with 978 or 979
    if !s.starts_with("978") && !s.starts_with("979") {
        return false;
    }

    // Same as EAN-13 check
    let digits: Vec<u32> = s.chars().filter_map(|c| c.to_digit(10)).collect();
    let mut sum: u32 = 0;
    for (i, &d) in digits.iter().enumerate() {
        if i % 2 == 0 {
            sum += d;
        } else {
            sum += d * 3;
        }
    }
    sum % 10 == 0
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_is_itin() {
        let pred = IsItinPredicate;

        // Valid ITINs (9XX-7X-XXXX or 9XX-8X-XXXX)
        assert!(pred.evaluate(&json!("912-78-1234"), &json!(null)));
        assert!(pred.evaluate(&json!("900-70-0001"), &json!(null)));
        assert!(pred.evaluate(&json!("999-80-9999"), &json!(null)));

        // Invalid: doesn't start with 9
        assert!(!pred.evaluate(&json!("123-78-1234"), &json!(null)));

        // Invalid: 4th digit not 7 or 8
        assert!(!pred.evaluate(&json!("912-68-1234"), &json!(null)));

        // Invalid: wrong length
        assert!(!pred.evaluate(&json!("912-78-123"), &json!(null)));
    }

    #[test]
    fn test_is_atin() {
        let pred = IsAtinPredicate;

        // Valid ATINs (9XX-93-XXXX)
        assert!(pred.evaluate(&json!("912-93-1234"), &json!(null)));
        assert!(pred.evaluate(&json!("900-93-0001"), &json!(null)));

        // Invalid: doesn't start with 9
        assert!(!pred.evaluate(&json!("123-93-1234"), &json!(null)));

        // Invalid: positions 4-5 not 93
        assert!(!pred.evaluate(&json!("912-94-1234"), &json!(null)));
    }

    #[test]
    fn test_is_cusip() {
        let pred = IsCusipPredicate;

        // Valid CUSIPs
        assert!(pred.evaluate(&json!("037833100"), &json!(null))); // Apple
        assert!(pred.evaluate(&json!("594918104"), &json!(null))); // Microsoft

        // Case insensitive
        assert!(pred.evaluate(&json!("037833100"), &json!(null)));

        // Invalid: wrong length
        assert!(!pred.evaluate(&json!("03783310"), &json!(null)));
        assert!(!pred.evaluate(&json!("0378331001"), &json!(null)));

        // Invalid checksum
        assert!(!pred.evaluate(&json!("037833101"), &json!(null)));

        // Skip checksum validation
        assert!(pred.evaluate(&json!("037833101"), &json!({"validate_checksum": false})));
    }

    #[test]
    fn test_is_aba_routing() {
        let pred = IsAbaRoutingPredicate;

        // Valid routing numbers
        assert!(pred.evaluate(&json!("021000021"), &json!(null))); // Chase
        assert!(pred.evaluate(&json!("011401533"), &json!(null))); // Bank of America
        assert!(pred.evaluate(&json!("121000358"), &json!(null))); // Wells Fargo

        // With formatting
        assert!(pred.evaluate(&json!("021-000-021"), &json!(null)));

        // Invalid: wrong length
        assert!(!pred.evaluate(&json!("02100002"), &json!(null)));

        // Invalid: bad checksum
        assert!(!pred.evaluate(&json!("021000022"), &json!(null)));

        // Invalid: bad prefix
        assert!(!pred.evaluate(&json!("991000021"), &json!(null)));
    }

    #[test]
    fn test_is_mcc() {
        let pred = IsMccPredicate;

        // Valid MCCs
        assert!(pred.evaluate(&json!("5411"), &json!(null))); // Grocery stores
        assert!(pred.evaluate(&json!("5812"), &json!(null))); // Restaurants
        assert!(pred.evaluate(&json!("0000"), &json!(null)));
        assert!(pred.evaluate(&json!("9999"), &json!(null)));

        // Invalid: wrong length
        assert!(!pred.evaluate(&json!("541"), &json!(null)));
        assert!(!pred.evaluate(&json!("54111"), &json!(null)));

        // Invalid: non-digits
        assert!(!pred.evaluate(&json!("541A"), &json!(null)));
    }

    #[test]
    fn test_is_account_number() {
        let pred = IsAccountNumberPredicate;

        // Valid account numbers
        assert!(pred.evaluate(&json!("1234567890"), &json!(null)));
        assert!(pred.evaluate(&json!("ABC-123-XYZ"), &json!(null)));
        assert!(pred.evaluate(&json!("X"), &json!(null)));

        // Too short with custom min_len
        assert!(!pred.evaluate(&json!("12"), &json!({"min_len": 5})));

        // Too long with custom max_len
        assert!(!pred.evaluate(&json!("123456"), &json!({"max_len": 5})));

        // Empty is invalid
        assert!(!pred.evaluate(&json!(""), &json!(null)));

        // Disallow hyphens
        assert!(!pred.evaluate(&json!("ABC-123"), &json!({"allow_hyphens": false})));
    }

    #[test]
    fn test_is_credit_card() {
        let pred = IsCreditCardPredicate;

        // Valid Visa
        assert!(pred.evaluate(&json!("4532015112830366"), &json!(null)));
        assert!(pred.evaluate(&json!("4532015112830366"), &json!({"network": "visa"})));

        // Valid Amex
        assert!(pred.evaluate(&json!("371449635398431"), &json!(null)));
        assert!(pred.evaluate(&json!("371449635398431"), &json!({"network": "amex"})));

        // Valid Mastercard
        assert!(pred.evaluate(&json!("5425233430109903"), &json!(null)));
        assert!(pred.evaluate(
            &json!("5425233430109903"),
            &json!({"network": "mastercard"})
        ));

        // Valid Discover
        assert!(pred.evaluate(&json!("6011111111111117"), &json!(null)));
        assert!(pred.evaluate(&json!("6011111111111117"), &json!({"network": "discover"})));

        // With spaces/dashes (digits extracted)
        assert!(pred.evaluate(&json!("4532 0151 1283 0366"), &json!(null)));

        // Wrong network
        assert!(!pred.evaluate(&json!("4532015112830366"), &json!({"network": "amex"})));
        assert!(!pred.evaluate(&json!("371449635398431"), &json!({"network": "visa"})));

        // Invalid Luhn
        assert!(!pred.evaluate(&json!("4532015112830367"), &json!(null)));

        // Too short
        assert!(!pred.evaluate(&json!("4532"), &json!(null)));

        // Non-string
        assert!(!pred.evaluate(&json!(4532015112830366_u64), &json!(null)));
    }

    #[test]
    fn test_is_cvv() {
        let pred = IsCvvPredicate;

        // Valid 3-digit CVV (no network)
        assert!(pred.evaluate(&json!("123"), &json!(null)));
        // Valid 4-digit CVV (no network)
        assert!(pred.evaluate(&json!("1234"), &json!(null)));

        // Amex: exactly 4
        assert!(pred.evaluate(&json!("1234"), &json!({"network": "amex"})));
        assert!(!pred.evaluate(&json!("123"), &json!({"network": "amex"})));

        // Visa: exactly 3
        assert!(pred.evaluate(&json!("123"), &json!({"network": "visa"})));
        assert!(!pred.evaluate(&json!("1234"), &json!({"network": "visa"})));

        // Invalid: non-digits
        assert!(!pred.evaluate(&json!("12a"), &json!(null)));

        // Invalid: too short
        assert!(!pred.evaluate(&json!("12"), &json!(null)));

        // Invalid: too long
        assert!(!pred.evaluate(&json!("12345"), &json!(null)));

        // Non-string
        assert!(!pred.evaluate(&json!(123), &json!(null)));
    }

    #[test]
    fn test_is_card_expiry() {
        let pred = IsCardExpiryPredicate;

        // Valid MM/YY
        assert!(pred.evaluate(&json!("12/25"), &json!(null)));
        assert!(pred.evaluate(&json!("01/30"), &json!(null)));

        // Valid MM/YYYY
        assert!(pred.evaluate(&json!("12/2025"), &json!(null)));
        assert!(pred.evaluate(&json!("01/2030"), &json!(null)));

        // Invalid month
        assert!(!pred.evaluate(&json!("13/25"), &json!(null)));
        assert!(!pred.evaluate(&json!("00/25"), &json!(null)));

        // Invalid format
        assert!(!pred.evaluate(&json!("12-25"), &json!(null)));
        assert!(!pred.evaluate(&json!("12/2"), &json!(null)));
        assert!(!pred.evaluate(&json!("1/25"), &json!(null)));

        // Expired card with reject_expired
        assert!(!pred.evaluate(&json!("01/20"), &json!({"reject_expired": true})));
        assert!(!pred.evaluate(&json!("01/2020"), &json!({"reject_expired": true})));

        // Far future card with reject_expired
        assert!(pred.evaluate(&json!("12/99"), &json!({"reject_expired": true})));
        assert!(pred.evaluate(&json!("12/2099"), &json!({"reject_expired": true})));

        // Non-string
        assert!(!pred.evaluate(&json!(1225), &json!(null)));
    }

    #[test]
    fn test_is_iban() {
        let pred = IsIbanPredicate;

        // Valid IBANs
        assert!(pred.evaluate(&json!("GB29NWBK60161331926819"), &json!(null)));
        assert!(pred.evaluate(&json!("DE89370400440532013000"), &json!(null)));
        assert!(pred.evaluate(&json!("FR7630006000011234567890189"), &json!(null)));
        assert!(pred.evaluate(&json!("ES9121000418450200051332"), &json!(null)));
        assert!(pred.evaluate(&json!("IT60X0542811101000000123456"), &json!(null)));

        // With spaces
        assert!(pred.evaluate(&json!("GB29 NWBK 6016 1331 9268 19"), &json!(null)));
        assert!(pred.evaluate(&json!("DE89 3704 0044 0532 0130 00"), &json!(null)));

        // Country filter
        assert!(pred.evaluate(&json!("DE89370400440532013000"), &json!({"country": "DE"})));
        assert!(!pred.evaluate(&json!("DE89370400440532013000"), &json!({"country": "GB"})));

        // Invalid: bad checksum
        assert!(!pred.evaluate(&json!("GB29NWBK60161331926818"), &json!(null)));

        // Invalid: wrong country length
        assert!(!pred.evaluate(&json!("GB29NWBK6016133192681"), &json!(null)));

        // Invalid: too short
        assert!(!pred.evaluate(&json!("GB29"), &json!(null)));

        // Invalid: non-alphanumeric in BBAN
        assert!(!pred.evaluate(&json!("GB29NWBK6016!331926819"), &json!(null)));

        // Non-string
        assert!(!pred.evaluate(&json!(123456), &json!(null)));
    }

    #[test]
    fn test_is_bic() {
        let pred = IsBicPredicate;

        // Valid 8-character BICs
        assert!(pred.evaluate(&json!("DEUTDEFF"), &json!(null))); // Deutsche Bank
        assert!(pred.evaluate(&json!("BNPAFRPP"), &json!(null))); // BNP Paribas
        assert!(pred.evaluate(&json!("CHASUS33"), &json!(null))); // JPMorgan Chase

        // Valid 11-character BICs
        assert!(pred.evaluate(&json!("DEUTDEFF500"), &json!(null))); // Deutsche Bank branch
        assert!(pred.evaluate(&json!("COBADEFFXXX"), &json!(null))); // Commerzbank head office

        // Case insensitive (uppercased internally)
        assert!(pred.evaluate(&json!("deutdeff"), &json!(null)));

        // Invalid: wrong length
        assert!(!pred.evaluate(&json!("DEUTDE"), &json!(null)));
        assert!(!pred.evaluate(&json!("DEUTDEFF50"), &json!(null)));
        assert!(!pred.evaluate(&json!("DEUTDEFF5000"), &json!(null)));

        // Invalid: digits in bank code
        assert!(!pred.evaluate(&json!("D3UTDEFF"), &json!(null)));

        // Invalid: digits in country code
        assert!(!pred.evaluate(&json!("DEUT1EFF"), &json!(null)));

        // Non-string
        assert!(!pred.evaluate(&json!(12345678), &json!(null)));
    }

    #[test]
    fn test_is_vin() {
        let pred = IsVinPredicate;

        // Valid VINs (check digit at position 9)
        assert!(pred.evaluate(&json!("11111111111111111"), &json!(null)));
        assert!(pred.evaluate(&json!("1M8GDM9AXKP042788"), &json!(null)));

        // Without checksum validation
        assert!(pred.evaluate(
            &json!("12345678901234567"),
            &json!({"validate_checksum": false})
        ));

        // Invalid: wrong length
        assert!(!pred.evaluate(&json!("1234567890"), &json!(null)));

        // Invalid: contains I, O, or Q
        assert!(!pred.evaluate(&json!("1M8GDM9AXKPI42788"), &json!(null)));
        assert!(!pred.evaluate(&json!("1M8GDM9AXKPO42788"), &json!(null)));
        assert!(!pred.evaluate(&json!("1M8GDM9AXKPQ42788"), &json!(null)));

        // Invalid: bad check digit
        assert!(!pred.evaluate(&json!("1M8GDM9AXKP042789"), &json!(null)));

        // Non-string
        assert!(!pred.evaluate(&json!(12345), &json!(null)));
    }

    #[test]
    fn test_is_upc() {
        let pred = IsUpcPredicate;

        // Valid UPC-A
        assert!(pred.evaluate(&json!("036000291452"), &json!(null)));
        assert!(pred.evaluate(&json!("012345678905"), &json!(null)));

        // Invalid: wrong length
        assert!(!pred.evaluate(&json!("12345"), &json!(null)));

        // Invalid: bad check digit
        assert!(!pred.evaluate(&json!("036000291453"), &json!(null)));

        // Non-string
        assert!(!pred.evaluate(&json!(12345), &json!(null)));
    }

    #[test]
    fn test_is_ean() {
        let pred = IsEanPredicate;

        // Valid EAN-13
        assert!(pred.evaluate(&json!("4006381333931"), &json!(null)));
        assert!(pred.evaluate(&json!("5901234123457"), &json!(null)));

        // Invalid: wrong length
        assert!(!pred.evaluate(&json!("12345"), &json!(null)));

        // Invalid: bad check digit
        assert!(!pred.evaluate(&json!("4006381333932"), &json!(null)));

        // Non-string
        assert!(!pred.evaluate(&json!(12345), &json!(null)));
    }

    #[test]
    fn test_is_isbn() {
        let pred = IsIsbnPredicate;

        // Valid ISBN-10
        assert!(pred.evaluate(&json!("0306406152"), &json!(null)));
        assert!(pred.evaluate(&json!("0-306-40615-2"), &json!(null))); // With hyphens
        assert!(pred.evaluate(&json!("007462542X"), &json!(null))); // X check digit

        // Valid ISBN-13
        assert!(pred.evaluate(&json!("9780306406157"), &json!(null)));
        assert!(pred.evaluate(&json!("978-0-306-40615-7"), &json!(null))); // With hyphens

        // Version filter
        assert!(pred.evaluate(&json!("0306406152"), &json!({"version": 10})));
        assert!(!pred.evaluate(&json!("9780306406157"), &json!({"version": 10})));
        assert!(pred.evaluate(&json!("9780306406157"), &json!({"version": 13})));
        assert!(!pred.evaluate(&json!("0306406152"), &json!({"version": 13})));

        // Invalid: bad check digit
        assert!(!pred.evaluate(&json!("0306406153"), &json!(null)));
        assert!(!pred.evaluate(&json!("9780306406158"), &json!(null)));

        // Invalid: wrong length
        assert!(!pred.evaluate(&json!("12345"), &json!(null)));

        // Non-string
        assert!(!pred.evaluate(&json!(12345), &json!(null)));
    }
}

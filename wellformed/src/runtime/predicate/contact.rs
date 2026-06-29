//! Contact information validation predicates.
//!
//! Validates phone numbers, email addresses, and other contact fields.

use super::registry::{NamedPredicate, PredicateRegistry};
use memchr::memchr;
use serde_json::Value;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::sync::Arc;

/// Register all contact predicates.
pub fn register_contact_predicates(registry: &mut PredicateRegistry) {
    let phone_number = Arc::new(PhoneNumberPredicate);
    let phone_number_us = Arc::new(PhoneNumberUSPredicate);
    registry.register(phone_number.clone());
    registry.register(phone_number_us);
    // backwards-compat alias: is_phone → phone_number (general)
    registry.register_as("is_phone", phone_number);
    registry.register(Arc::new(IsEmailPredicate));
    registry.register(Arc::new(IsUrlPredicate));
    registry.register(Arc::new(IsUuidPredicate));
    registry.register(Arc::new(IsIpPredicate));
    registry.register(Arc::new(IsCidrPredicate));
    registry.register(Arc::new(IsMacAddressPredicate));
    registry.register(Arc::new(IsStreetAddressPredicate));
}

// ============================================================================
// Phone Number Predicates
// ============================================================================

/// Strict US phone number validation.
/// Rejects international prefix (+).
struct PhoneNumberUSPredicate;

impl NamedPredicate for PhoneNumberUSPredicate {
    fn name(&self) -> &str {
        "phone_number_us"
    }

    fn evaluate(&self, value: &Value, args: &Value) -> bool {
        let s = match value.as_str() {
            Some(s) => s.trim(),
            None => return false,
        };

        if s.is_empty() || s.starts_with('+') {
            return false;
        }

        validate_us_phone(s, args)
    }
}

/// General phone number validation.
/// Accepts US format OR international format (+ prefix, 7-15 digits).
struct PhoneNumberPredicate;

impl NamedPredicate for PhoneNumberPredicate {
    fn name(&self) -> &str {
        "phone_number"
    }

    fn evaluate(&self, value: &Value, args: &Value) -> bool {
        let s = match value.as_str() {
            Some(s) => s.trim(),
            None => return false,
        };

        if s.is_empty() {
            return false;
        }

        // International format: starts with +, 7-15 digits
        if s.starts_with('+') {
            return validate_international_phone(s);
        }

        // Otherwise validate as US phone
        validate_us_phone(s, args)
    }
}

/// Validate a US phone number.
///
/// Accepts formats:
/// - (123) 456-7890
/// - 123-456-7890
/// - 123.456.7890
/// - 1234567890
/// - +1 123 456 7890
fn validate_us_phone(s: &str, args: &Value) -> bool {
    let require_area_code = args
        .get("require_area_code")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    let mut digit_count = 0usize;
    let mut d0 = b'0';
    let mut d1 = b'0';
    let mut d3 = b'0';
    let mut d4 = b'0';

    for &b in s.as_bytes() {
        if !b.is_ascii_digit() {
            continue;
        }

        match digit_count {
            0 => d0 = b,
            1 => d1 = b,
            3 => d3 = b,
            4 => d4 = b,
            _ => {}
        }

        digit_count += 1;
        if digit_count > 11 {
            return false;
        }
    }

    match digit_count {
        // US phone: 10 digits
        10 => d0 != b'0' && d0 != b'1' && d3 != b'0' && d3 != b'1',
        // US phone with country code: 11 digits starting with 1
        11 => d0 == b'1' && d1 != b'0' && d1 != b'1' && d4 != b'0' && d4 != b'1',
        // 7 digits without area code if allowed
        7 if !require_area_code => d0 != b'0' && d0 != b'1',
        _ => false,
    }
}

/// Validate an international phone number.
///
/// Must start with + and contain 7-15 digits.
fn validate_international_phone(s: &str) -> bool {
    let s = s.trim();

    // Must start with +
    if !s.as_bytes().starts_with(b"+") {
        return false;
    }

    let mut digits = 0usize;
    for &b in s.as_bytes() {
        if b.is_ascii_digit() {
            digits += 1;
            if digits > 15 {
                return false;
            }
        }
    }

    // International numbers: 7-15 digits
    digits >= 7
}

// ============================================================================
// Email Predicate
// ============================================================================

/// Validate an email address with a byte-scanner fast path.
struct IsEmailPredicate;

impl NamedPredicate for IsEmailPredicate {
    fn name(&self) -> &str {
        "is_email"
    }

    fn evaluate(&self, value: &Value, args: &Value) -> bool {
        let s = match value.as_str() {
            Some(s) => s.trim(),
            None => return false,
        };

        if s.is_empty() {
            return false;
        }

        let bytes = s.as_bytes();
        let Some(at) = memchr(b'@', bytes) else {
            return false;
        };
        if memchr(b'@', &bytes[at + 1..]).is_some() {
            return false;
        };
        let local = &bytes[..at];
        let domain = &bytes[at + 1..];

        // Local part validation
        if local.is_empty() || local.len() > 64 {
            return false;
        }

        // Domain validation
        if domain.is_empty() || domain.len() > 253 {
            return false;
        }

        // Domain must contain at least one dot
        if memchr(b'.', domain).is_none() {
            return false;
        }

        // Domain can't start or end with dot or hyphen
        if domain[0] == b'.' || domain[domain.len() - 1] == b'.' {
            return false;
        }
        if domain[0] == b'-' || domain[domain.len() - 1] == b'-' {
            return false;
        }

        // Check TLD (last part after dot)
        let Some(last_dot) = domain.iter().rposition(|&b| b == b'.') else {
            return false;
        };
        let tld = &domain[last_dot + 1..];
        if tld.len() < 2 {
            return false;
        }

        // TLD must be alphabetic
        if !tld.iter().all(|b| b.is_ascii_alphabetic()) {
            return false;
        }

        // Check for valid characters in local part
        // Allow: a-z, A-Z, 0-9, ., _, %, +, -
        let valid_local = local.iter().all(|&b| {
            b.is_ascii_alphanumeric()
                || b == b'.'
                || b == b'_'
                || b == b'%'
                || b == b'+'
                || b == b'-'
        });
        if !valid_local {
            return false;
        }

        // Local can't start or end with dot
        if local[0] == b'.' || local[local.len() - 1] == b'.' {
            return false;
        }

        // Check for valid characters in domain
        let valid_domain = domain
            .iter()
            .all(|&b| b.is_ascii_alphanumeric() || b == b'.' || b == b'-');
        if !valid_domain {
            return false;
        }

        // Optional: check against known disposable email domains
        let block_disposable = args
            .get("block_disposable")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        if block_disposable
            && DISPOSABLE_EMAIL_DOMAINS
                .iter()
                .any(|d| bytes_ends_with_ignore_ascii_case(domain, d.as_bytes()))
        {
            return false;
        }

        true
    }
}

fn bytes_ends_with_ignore_ascii_case(bytes: &[u8], suffix: &[u8]) -> bool {
    if bytes.len() < suffix.len() {
        return false;
    }
    bytes[bytes.len() - suffix.len()..].eq_ignore_ascii_case(suffix)
}

// ============================================================================
// URL Predicate
// ============================================================================

/// Validate a URL.
///
/// Args:
/// - require_https: If true, only allow https:// (default false)
struct IsUrlPredicate;

impl NamedPredicate for IsUrlPredicate {
    fn name(&self) -> &str {
        "is_url"
    }

    fn evaluate(&self, value: &Value, args: &Value) -> bool {
        let s = match value.as_str() {
            Some(s) => s.trim(),
            None => return false,
        };

        if s.is_empty() {
            return false;
        }

        let require_https = args
            .get("require_https")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        // Check protocol
        let has_http = s.starts_with("http://");
        let has_https = s.starts_with("https://");

        if require_https && !has_https {
            return false;
        }

        if !has_http && !has_https {
            return false;
        }

        // Extract the rest after protocol
        let rest = if has_https { &s[8..] } else { &s[7..] };

        if rest.is_empty() {
            return false;
        }

        // Must have a domain part
        let domain_part = rest.split('/').next().unwrap();
        if domain_part.is_empty() {
            return false;
        }

        // Domain must contain valid characters
        let domain_without_port = domain_part.split(':').next().unwrap();
        if domain_without_port.is_empty() {
            return false;
        }

        // Basic domain validation
        domain_without_port
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-')
    }
}

// ============================================================================
// UUID Predicate
// ============================================================================

/// Validate a UUID string.
///
/// Accepts standard 8-4-4-4-12 hex format with dashes.
/// Args: `{ "version": 4 }` (optional) to restrict to a specific version.
struct IsUuidPredicate;

impl NamedPredicate for IsUuidPredicate {
    fn name(&self) -> &str {
        "is_uuid"
    }

    fn evaluate(&self, value: &Value, args: &Value) -> bool {
        let s = match value.as_str() {
            Some(s) => s.trim().to_lowercase(),
            None => return false,
        };

        // Must be exactly 36 characters: 8-4-4-4-12
        if s.len() != 36 {
            return false;
        }

        // Check dashes at correct positions
        let bytes = s.as_bytes();
        if bytes[8] != b'-' || bytes[13] != b'-' || bytes[18] != b'-' || bytes[23] != b'-' {
            return false;
        }

        // All other characters must be hex digits
        for (i, &b) in bytes.iter().enumerate() {
            if i == 8 || i == 13 || i == 18 || i == 23 {
                continue;
            }
            if !b.is_ascii_hexdigit() {
                return false;
            }
        }

        // Optional version check (version is the first nibble of the 3rd group, position 14)
        if let Some(version) = args.get("version").and_then(|v| v.as_u64()) {
            let version_char = bytes[14];
            let actual_version = if version_char.is_ascii_digit() {
                (version_char - b'0') as u64
            } else {
                (version_char - b'a' + 10) as u64
            };
            if actual_version != version {
                return false;
            }
        }

        true
    }
}

// ============================================================================
// IP / CIDR / MAC Predicates
// ============================================================================

/// Validate an IP address (IPv4 or IPv6).
///
/// Optional args:
/// - `version`: `"v4"` or `"v6"`
struct IsIpPredicate;

impl NamedPredicate for IsIpPredicate {
    fn name(&self) -> &str {
        "is_ip"
    }

    fn evaluate(&self, value: &Value, args: &Value) -> bool {
        let s = match value.as_str() {
            Some(s) => s.trim(),
            None => return false,
        };

        let version = args
            .get("version")
            .and_then(|v| v.as_str())
            .map(|v| v.to_lowercase());

        match version.as_deref() {
            Some("v4") => s.parse::<Ipv4Addr>().is_ok(),
            Some("v6") => s.parse::<Ipv6Addr>().is_ok(),
            _ => s.parse::<IpAddr>().is_ok(),
        }
    }
}

/// Validate a CIDR block (IPv4 or IPv6).
///
/// Optional args:
/// - `version`: `"v4"` or `"v6"`
struct IsCidrPredicate;

impl NamedPredicate for IsCidrPredicate {
    fn name(&self) -> &str {
        "is_cidr"
    }

    fn evaluate(&self, value: &Value, args: &Value) -> bool {
        let s = match value.as_str() {
            Some(s) => s.trim(),
            None => return false,
        };

        let (ip_part, prefix_part) = match s.split_once('/') {
            Some(parts) => parts,
            None => return false,
        };

        let prefix = match prefix_part.parse::<u8>() {
            Ok(p) => p,
            Err(_) => return false,
        };

        let version = args
            .get("version")
            .and_then(|v| v.as_str())
            .map(|v| v.to_lowercase());

        match version.as_deref() {
            Some("v4") => ip_part.parse::<Ipv4Addr>().is_ok() && prefix <= 32,
            Some("v6") => ip_part.parse::<Ipv6Addr>().is_ok() && prefix <= 128,
            _ => {
                if ip_part.parse::<Ipv4Addr>().is_ok() {
                    prefix <= 32
                } else if ip_part.parse::<Ipv6Addr>().is_ok() {
                    prefix <= 128
                } else {
                    false
                }
            }
        }
    }
}

/// Validate a MAC address.
///
/// Accepts:
/// - `aa:bb:cc:dd:ee:ff`
/// - `aa-bb-cc-dd-ee-ff`
/// - `aabb.ccdd.eeff` (Cisco style)
struct IsMacAddressPredicate;

impl NamedPredicate for IsMacAddressPredicate {
    fn name(&self) -> &str {
        "is_mac_address"
    }

    fn evaluate(&self, value: &Value, _args: &Value) -> bool {
        let s = match value.as_str() {
            Some(s) => s.trim(),
            None => return false,
        };

        if s.contains(':') || s.contains('-') {
            let delim = if s.contains(':') { ':' } else { '-' };
            let parts: Vec<&str> = s.split(delim).collect();
            return parts.len() == 6
                && parts
                    .iter()
                    .all(|p| p.len() == 2 && p.chars().all(|c| c.is_ascii_hexdigit()));
        }

        if s.contains('.') {
            let parts: Vec<&str> = s.split('.').collect();
            return parts.len() == 3
                && parts
                    .iter()
                    .all(|p| p.len() == 4 && p.chars().all(|c| c.is_ascii_hexdigit()));
        }

        false
    }
}

// ============================================================================
// Street Address Predicate
// ============================================================================

/// Validate a street address.
///
/// Checks that the value starts with a digit (street number), contains at
/// least one ASCII letter (street name), and is at least 5 characters long.
/// Pure byte-scanning — no regex.
struct IsStreetAddressPredicate;

impl NamedPredicate for IsStreetAddressPredicate {
    fn name(&self) -> &str {
        "is_street_address"
    }

    fn evaluate(&self, value: &Value, _args: &Value) -> bool {
        let s = match value.as_str() {
            Some(s) => s.trim(),
            None => return false,
        };

        let bytes = s.as_bytes();
        if bytes.len() < 5 {
            return false;
        }

        // Must start with a digit (street number)
        if !bytes[0].is_ascii_digit() {
            return false;
        }

        // Must contain at least one letter (street name)
        let mut has_letter = false;
        for &b in bytes {
            if b.is_ascii_alphabetic() {
                has_letter = true;
                break;
            }
        }

        has_letter
    }
}

// ============================================================================
// Constants
// ============================================================================

/// Common disposable email domains.
const DISPOSABLE_EMAIL_DOMAINS: &[&str] = &[
    "mailinator.com",
    "guerrillamail.com",
    "10minutemail.com",
    "tempmail.com",
    "throwaway.email",
    "yopmail.com",
    "fakeinbox.com",
    "trashmail.com",
];

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_phone_number_us() {
        let pred = PhoneNumberUSPredicate;

        // Valid US formats
        assert!(pred.evaluate(&json!("(555) 234-5678"), &json!(null)));
        assert!(pred.evaluate(&json!("555-234-5678"), &json!(null)));
        assert!(pred.evaluate(&json!("555.234.5678"), &json!(null)));
        assert!(pred.evaluate(&json!("5552345678"), &json!(null)));
        assert!(pred.evaluate(&json!("1-555-234-5678"), &json!(null)));

        // International prefix rejected
        assert!(!pred.evaluate(&json!("+1 555-234-5678"), &json!(null)));
        assert!(!pred.evaluate(&json!("+62 34234233"), &json!(null)));

        // Invalid: area code starts with 0 or 1
        assert!(!pred.evaluate(&json!("055-234-5678"), &json!(null)));
        assert!(!pred.evaluate(&json!("155-234-5678"), &json!(null)));

        // Invalid: exchange starts with 0 or 1
        assert!(!pred.evaluate(&json!("555-023-4567"), &json!(null)));
        assert!(!pred.evaluate(&json!("555-123-4567"), &json!(null)));

        // Too short
        assert!(!pred.evaluate(&json!("123"), &json!(null)));

        // Without area code
        assert!(!pred.evaluate(&json!("234-5678"), &json!(null)));
        assert!(pred.evaluate(&json!("234-5678"), &json!({"require_area_code": false})));
    }

    #[test]
    fn test_phone_number_general() {
        let pred = PhoneNumberPredicate;

        // US formats pass
        assert!(pred.evaluate(&json!("(555) 234-5678"), &json!(null)));
        assert!(pred.evaluate(&json!("555-234-5678"), &json!(null)));

        // International formats pass
        assert!(pred.evaluate(&json!("+1 555-234-5678"), &json!(null)));
        assert!(pred.evaluate(&json!("+62 34234233"), &json!(null)));
        assert!(pred.evaluate(&json!("+44 20 7946 0958"), &json!(null)));

        // Junk fails
        assert!(!pred.evaluate(&json!("123"), &json!(null)));
        assert!(!pred.evaluate(&json!("not-a-phone"), &json!(null)));
    }

    #[test]
    fn test_is_phone_backwards_compat() {
        // is_phone should be registered as alias for phone_number
        let registry = PredicateRegistry::with_builtins();
        assert!(registry.get("is_phone").is_some());
        assert!(registry.get("phone_number").is_some());
        assert!(registry.get("phone_number_us").is_some());

        let is_phone = registry.get("is_phone").unwrap();
        // US formats pass
        assert!(is_phone.evaluate(&json!("(555) 234-5678"), &json!(null)));
        // International formats now pass (same as phone_number)
        assert!(is_phone.evaluate(&json!("+1 555-234-5678"), &json!(null)));
        assert!(is_phone.evaluate(&json!("+62 34234233"), &json!(null)));
    }

    #[test]
    fn test_is_email() {
        let pred = IsEmailPredicate;

        // Valid emails
        assert!(pred.evaluate(&json!("user@example.com"), &json!(null)));
        assert!(pred.evaluate(&json!("user.name@example.com"), &json!(null)));
        assert!(pred.evaluate(&json!("user+tag@example.com"), &json!(null)));
        assert!(pred.evaluate(&json!("user@sub.example.com"), &json!(null)));

        // Invalid emails
        assert!(!pred.evaluate(&json!("user"), &json!(null)));
        assert!(!pred.evaluate(&json!("user@"), &json!(null)));
        assert!(!pred.evaluate(&json!("@example.com"), &json!(null)));
        assert!(!pred.evaluate(&json!("user@@example.com"), &json!(null)));
        assert!(!pred.evaluate(&json!("user@example"), &json!(null))); // No TLD
        assert!(!pred.evaluate(&json!(".user@example.com"), &json!(null)));
        assert!(!pred.evaluate(&json!("user.@example.com"), &json!(null)));

        // Block disposable
        assert!(!pred.evaluate(
            &json!("test@mailinator.com"),
            &json!({"block_disposable": true})
        ));
        assert!(!pred.evaluate(
            &json!("test@MAILINATOR.COM"),
            &json!({"block_disposable": true})
        ));
        assert!(pred.evaluate(
            &json!("test@mailinator.com"),
            &json!({"block_disposable": false})
        ));
    }

    #[test]
    fn test_is_url() {
        let pred = IsUrlPredicate;

        // Valid URLs
        assert!(pred.evaluate(&json!("http://example.com"), &json!(null)));
        assert!(pred.evaluate(&json!("https://example.com"), &json!(null)));
        assert!(pred.evaluate(&json!("https://example.com/path"), &json!(null)));
        assert!(pred.evaluate(&json!("https://sub.example.com"), &json!(null)));

        // Require HTTPS
        assert!(!pred.evaluate(
            &json!("http://example.com"),
            &json!({"require_https": true})
        ));
        assert!(pred.evaluate(
            &json!("https://example.com"),
            &json!({"require_https": true})
        ));

        // Invalid
        assert!(!pred.evaluate(&json!("example.com"), &json!(null)));
        assert!(!pred.evaluate(&json!("ftp://example.com"), &json!(null)));
        assert!(!pred.evaluate(&json!("http://"), &json!(null)));
    }

    #[test]
    fn test_is_uuid() {
        let pred = IsUuidPredicate;

        // Valid UUIDs
        assert!(pred.evaluate(&json!("550e8400-e29b-41d4-a716-446655440000"), &json!(null)));
        assert!(pred.evaluate(&json!("6ba7b810-9dad-11d1-80b4-00c04fd430c8"), &json!(null)));
        assert!(pred.evaluate(&json!("f47ac10b-58cc-4372-a567-0e02b2c3d479"), &json!(null)));

        // Case insensitive
        assert!(pred.evaluate(&json!("550E8400-E29B-41D4-A716-446655440000"), &json!(null)));

        // Version filter (v4)
        assert!(pred.evaluate(
            &json!("f47ac10b-58cc-4372-a567-0e02b2c3d479"),
            &json!({"version": 4})
        ));
        assert!(!pred.evaluate(
            &json!("6ba7b810-9dad-11d1-80b4-00c04fd430c8"),
            &json!({"version": 4})
        ));

        // Invalid: wrong length
        assert!(!pred.evaluate(&json!("550e8400-e29b-41d4-a716"), &json!(null)));

        // Invalid: missing dashes
        assert!(!pred.evaluate(&json!("550e8400e29b41d4a716446655440000"), &json!(null)));

        // Invalid: non-hex characters
        assert!(!pred.evaluate(&json!("550e8400-e29b-41d4-a716-44665544000g"), &json!(null)));

        // Non-string
        assert!(!pred.evaluate(&json!(12345), &json!(null)));
    }

    #[test]
    fn test_is_ip() {
        let pred = IsIpPredicate;
        assert!(pred.evaluate(&json!("192.168.1.1"), &json!(null)));
        assert!(pred.evaluate(&json!("2001:db8::1"), &json!(null)));
        assert!(!pred.evaluate(&json!("999.999.999.999"), &json!(null)));

        assert!(pred.evaluate(&json!("10.0.0.1"), &json!({"version": "v4"})));
        assert!(!pred.evaluate(&json!("2001:db8::1"), &json!({"version": "v4"})));
    }

    #[test]
    fn test_is_cidr() {
        let pred = IsCidrPredicate;
        assert!(pred.evaluate(&json!("192.168.1.0/24"), &json!(null)));
        assert!(pred.evaluate(&json!("2001:db8::/32"), &json!(null)));
        assert!(!pred.evaluate(&json!("192.168.1.1"), &json!(null)));
        assert!(!pred.evaluate(&json!("192.168.1.0/33"), &json!(null)));
    }

    #[test]
    fn test_is_mac_address() {
        let pred = IsMacAddressPredicate;
        assert!(pred.evaluate(&json!("aa:bb:cc:dd:ee:ff"), &json!(null)));
        assert!(pred.evaluate(&json!("AA-BB-CC-DD-EE-FF"), &json!(null)));
        assert!(pred.evaluate(&json!("aabb.ccdd.eeff"), &json!(null)));

        assert!(!pred.evaluate(&json!("aa:bb:cc:dd:ee"), &json!(null)));
        assert!(!pred.evaluate(&json!("not-a-mac"), &json!(null)));
    }
}

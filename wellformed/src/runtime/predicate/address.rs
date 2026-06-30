//! Address validation predicates using libpostal through the `postal` crate.
//!
//! These predicates are only available when the `address` feature is enabled.
//! The feature requires the native libpostal C library and parser data to be
//! installed on the target system.

use super::registry::{NamedPredicate, PredicateRegistry};
use postal::{Context, InitOptions, ParseAddressOptions};
use serde_json::Value;
use std::sync::{Arc, LazyLock, Mutex};

static POSTAL_CONTEXT: LazyLock<Result<Mutex<Context>, String>> = LazyLock::new(|| {
    let mut context = Context::new();
    context
        .init(InitOptions {
            expand_address: false,
            parse_address: true,
        })
        .map(|()| Mutex::new(context))
        .map_err(|err| err.to_string())
});

/// Register all address-related predicates.
pub fn register_address_predicates(registry: &mut PredicateRegistry) {
    registry.register(Arc::new(IsParseableAddressPredicate));
    registry.register(Arc::new(HasAddressComponentPredicate));
    registry.register(Arc::new(IsUsAddressPredicate));
    registry.register(Arc::new(AddressComponentMatchesPredicate));
    registry.register(Arc::new(IsUsZipPredicate));
    registry.register(Arc::new(IsUsStatePredicate));
}

// ============================================================================
// Predicate Implementations
// ============================================================================

/// Check if a string is a parseable address.
pub struct IsParseableAddressPredicate;

impl NamedPredicate for IsParseableAddressPredicate {
    fn name(&self) -> &str {
        "is_parseable_address"
    }

    fn evaluate(&self, value: &Value, args: &Value) -> bool {
        let s = match value.as_str() {
            Some(s) => s,
            None => return false,
        };

        if s.trim().is_empty() {
            return false;
        }

        parse_address_components(s, args).is_some_and(|parsed| !parsed.is_empty())
    }
}

/// Check if an address has a specific component.
/// Args: { "component": "road" | "city" | "state" | "postcode" | "country" | ... }
pub struct HasAddressComponentPredicate;

impl NamedPredicate for HasAddressComponentPredicate {
    fn name(&self) -> &str {
        "has_address_component"
    }

    fn evaluate(&self, value: &Value, args: &Value) -> bool {
        let s = match value.as_str() {
            Some(s) => s,
            None => return false,
        };

        let component = match args.get("component").and_then(|v| v.as_str()) {
            Some(c) => c,
            None => return false,
        };

        parse_address_components(s, args)
            .is_some_and(|parsed| component_value(&parsed, component).is_some())
    }
}

/// Check if an address has all required components for a US address.
/// Requires: road, city, state, postcode
pub struct IsUsAddressPredicate;

impl NamedPredicate for IsUsAddressPredicate {
    fn name(&self) -> &str {
        "is_us_address"
    }

    fn evaluate(&self, value: &Value, args: &Value) -> bool {
        let s = match value.as_str() {
            Some(s) => s,
            None => return false,
        };

        match parse_address_components(s, args) {
            Some(parsed) => {
                let has_road = component_value(&parsed, "road").is_some();
                let has_city = component_value(&parsed, "city").is_some();
                let has_state = component_value(&parsed, "state").is_some();
                let has_postcode = component_value(&parsed, "postcode").is_some();

                // Strict mode requires all components
                let strict = args.get("strict").and_then(|v| v.as_bool()).unwrap_or(true);

                if strict {
                    has_road && has_city && has_state && has_postcode
                } else {
                    // Lenient: at least city and state
                    has_city && has_state
                }
            }
            None => false,
        }
    }
}

/// Check if an address component matches a regex pattern.
/// Args: { "component": "postcode", "pattern": "^\\d{5}(-\\d{4})?$" }
pub struct AddressComponentMatchesPredicate;

impl NamedPredicate for AddressComponentMatchesPredicate {
    fn name(&self) -> &str {
        "address_component_matches"
    }

    fn evaluate(&self, value: &Value, args: &Value) -> bool {
        let s = match value.as_str() {
            Some(s) => s,
            None => return false,
        };

        let component = match args.get("component").and_then(|v| v.as_str()) {
            Some(c) => c,
            None => return false,
        };

        let pattern = match args.get("pattern").and_then(|v| v.as_str()) {
            Some(p) => p,
            None => return false,
        };

        let regex = match regex::Regex::new(pattern) {
            Ok(r) => r,
            Err(_) => return false,
        };

        parse_address_components(s, args).is_some_and(|parsed| {
            component_value(&parsed, component).is_some_and(|value| regex.is_match(value))
        })
    }
}

/// Validate US ZIP code format (5 or 9 digits).
///
/// This predicate does NOT use libpostal - it validates the ZIP format directly.
pub struct IsUsZipPredicate;

impl NamedPredicate for IsUsZipPredicate {
    fn name(&self) -> &str {
        "is_us_zip"
    }

    fn evaluate(&self, value: &Value, _args: &Value) -> bool {
        let s = match value.as_str() {
            Some(s) => s,
            None => return false,
        };

        // Extract digits only
        let digits: String = s.chars().filter(|c| c.is_ascii_digit()).collect();

        // Valid ZIP: 5 digits or 9 digits (ZIP+4)
        if digits.len() != 5 && digits.len() != 9 {
            return false;
        }

        // ZIP cannot be all zeros
        if digits.chars().all(|c| c == '0') {
            return false;
        }

        true
    }
}

/// Validate US state code (2 letters).
///
/// This predicate does NOT use libpostal - it validates state codes directly.
pub struct IsUsStatePredicate;

impl NamedPredicate for IsUsStatePredicate {
    fn name(&self) -> &str {
        "is_us_state"
    }

    fn evaluate(&self, value: &Value, _args: &Value) -> bool {
        let s = match value.as_str() {
            Some(s) => s.trim().to_uppercase(),
            None => return false,
        };

        US_STATE_CODES.contains(&s.as_str())
    }
}

// ============================================================================
// Helpers
// ============================================================================

type AddressComponents = Vec<(String, String)>;

fn parse_address_components(address: &str, _args: &Value) -> Option<AddressComponents> {
    let context = POSTAL_CONTEXT.as_ref().ok()?;
    let context = context.lock().ok()?;
    let mut options = ParseAddressOptions::new();
    let components = context.parse_address(address, &mut options).ok()?;
    Some(
        components
            .map(|component| (component.label.to_string(), component.value.to_string()))
            .collect(),
    )
}

fn component_value<'a>(components: &'a AddressComponents, component: &str) -> Option<&'a str> {
    components
        .iter()
        .find(|(label, _)| label == component)
        .map(|(_, value)| value.as_str())
}

/// Valid US state and territory codes.
const US_STATE_CODES: &[&str] = &[
    // States
    "AL", "AK", "AZ", "AR", "CA", "CO", "CT", "DE", "FL", "GA", "HI", "ID", "IL", "IN", "IA", "KS",
    "KY", "LA", "ME", "MD", "MA", "MI", "MN", "MS", "MO", "MT", "NE", "NV", "NH", "NJ", "NM", "NY",
    "NC", "ND", "OH", "OK", "OR", "PA", "RI", "SC", "SD", "TN", "TX", "UT", "VT", "VA", "WA", "WV",
    "WI", "WY", // District
    "DC", // Territories
    "AS", "GU", "MP", "PR", "VI", "UM", // Military
    "AA", "AE", "AP",
];

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use serial_test::serial;

    // Tests that use libpostal must run serially because libpostal's C library
    // is not thread-safe during initialization.

    #[test]
    #[serial]
    fn test_is_parseable_address() {
        let pred = IsParseableAddressPredicate;

        // Valid address should parse
        assert!(pred.evaluate(&json!("123 Main St, San Francisco, CA 94102"), &json!(null)));

        // Empty string should fail
        assert!(!pred.evaluate(&json!(""), &json!(null)));

        // Non-string should fail
        assert!(!pred.evaluate(&json!(123), &json!(null)));
    }

    #[test]
    #[serial]
    fn test_has_address_component() {
        let pred = HasAddressComponentPredicate;

        // Check for road component
        assert!(pred.evaluate(
            &json!("123 Main St, San Francisco, CA 94102"),
            &json!({"component": "road"})
        ));

        // Check for postcode component
        assert!(pred.evaluate(
            &json!("123 Main St, San Francisco, CA 94102"),
            &json!({"component": "postcode"})
        ));

        // Missing component should fail
        assert!(!pred.evaluate(
            &json!("123 Main St"),
            &json!({"component": "invalid_component_xyz"})
        ));
    }

    #[test]
    #[serial]
    fn test_is_us_address() {
        let pred = IsUsAddressPredicate;

        // Full US address (strict mode)
        assert!(pred.evaluate(
            &json!("123 Main St, San Francisco, CA 94102"),
            &json!({"strict": true})
        ));

        // Lenient mode - just city and state
        assert!(pred.evaluate(&json!("San Francisco, CA"), &json!({"strict": false})));
    }

    #[test]
    #[serial]
    fn test_address_component_matches() {
        let pred = AddressComponentMatchesPredicate;

        // Match ZIP code pattern
        assert!(pred.evaluate(
            &json!("123 Main St, San Francisco, CA 94102"),
            &json!({"component": "postcode", "pattern": r"^\d{5}$"})
        ));

        // Non-matching pattern
        assert!(!pred.evaluate(
            &json!("123 Main St, San Francisco, CA 94102"),
            &json!({"component": "postcode", "pattern": r"^XXXXX$"})
        ));
    }

    // Tests below don't use libpostal and can run in parallel

    #[test]
    fn test_is_us_zip() {
        let pred = IsUsZipPredicate;

        // Valid 5-digit ZIP
        assert!(pred.evaluate(&json!("94102"), &json!(null)));

        // Valid ZIP+4
        assert!(pred.evaluate(&json!("94102-1234"), &json!(null)));
        assert!(pred.evaluate(&json!("941021234"), &json!(null)));

        // Invalid: all zeros
        assert!(!pred.evaluate(&json!("00000"), &json!(null)));

        // Invalid: wrong length
        assert!(!pred.evaluate(&json!("9410"), &json!(null)));
        assert!(!pred.evaluate(&json!("9410212345"), &json!(null)));

        // Non-string should fail
        assert!(!pred.evaluate(&json!(94102), &json!(null)));
    }

    #[test]
    fn test_is_us_state() {
        // This test doesn't use libpostal
        let pred = IsUsStatePredicate;

        // Valid state codes
        assert!(pred.evaluate(&json!("CA"), &json!(null)));
        assert!(pred.evaluate(&json!("NY"), &json!(null)));
        assert!(pred.evaluate(&json!("TX"), &json!(null)));

        // Case insensitive
        assert!(pred.evaluate(&json!("ca"), &json!(null)));

        // District of Columbia
        assert!(pred.evaluate(&json!("DC"), &json!(null)));

        // Territories
        assert!(pred.evaluate(&json!("PR"), &json!(null)));
        assert!(pred.evaluate(&json!("GU"), &json!(null)));

        // Military codes
        assert!(pred.evaluate(&json!("AA"), &json!(null)));
        assert!(pred.evaluate(&json!("AE"), &json!(null)));
        assert!(pred.evaluate(&json!("AP"), &json!(null)));

        // Invalid
        assert!(!pred.evaluate(&json!("XX"), &json!(null)));
        assert!(!pred.evaluate(&json!("California"), &json!(null)));
    }
}

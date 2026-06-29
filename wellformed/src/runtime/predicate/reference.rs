//! Reference data validation predicates.
//!
//! Validates ISO country codes, W-2 box 12 codes, 1099-B transaction codes, etc.

use super::registry::{NamedPredicate, PredicateRegistry};
use serde_json::Value;
use std::sync::Arc;

/// Register all reference data predicates.
pub fn register_reference_predicates(registry: &mut PredicateRegistry) {
    registry.register(Arc::new(IsCountryCodePredicate));
    registry.register(Arc::new(IsCountryNamePredicate));
    registry.register(Arc::new(IsCurrencyCodePredicate));
    registry.register(Arc::new(IsStateNamePredicate));
    registry.register(Arc::new(IsUsStatePredicate));
    registry.register(Arc::new(IsUsZipPredicate));
    registry.register(Arc::new(IsW2Box12CodePredicate));
    registry.register(Arc::new(Is1099BCodePredicate));
    registry.register(Arc::new(IsFilingStatusPredicate));
}

// ============================================================================
// ISO Country Code (ISO 3166-1 alpha-2)
// ============================================================================

/// Validate an ISO 3166-1 alpha-2 country code.
///
/// Used on 1099-INT, 1099-DIV for foreign country fields.
struct IsCountryCodePredicate;

impl NamedPredicate for IsCountryCodePredicate {
    fn name(&self) -> &str {
        "is_country_code"
    }

    fn evaluate(&self, value: &Value, args: &Value) -> bool {
        let s = match value.as_str() {
            Some(s) => s.trim().to_uppercase(),
            None => return false,
        };

        if s.len() != 2 {
            return false;
        }

        // Check if we should exclude US (for "foreign country" fields)
        let exclude_us = args
            .get("exclude_us")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        if exclude_us && s == "US" {
            return false;
        }

        ISO_COUNTRY_CODES.contains(&s.as_str())
    }
}

// ============================================================================
// Country Name
// ============================================================================

/// Validate a country name (common English names).
struct IsCountryNamePredicate;

impl NamedPredicate for IsCountryNamePredicate {
    fn name(&self) -> &str {
        "is_country_name"
    }

    fn evaluate(&self, value: &Value, _args: &Value) -> bool {
        let s = match value.as_str() {
            Some(s) => s.trim(),
            None => return false,
        };

        if s.is_empty() {
            return false;
        }

        // Case-insensitive lookup
        COUNTRY_NAMES
            .iter()
            .any(|name| name.eq_ignore_ascii_case(s))
    }
}

// ============================================================================
// State Name
// ============================================================================

/// Validate a US state name (full name, case-insensitive).
struct IsStateNamePredicate;

impl NamedPredicate for IsStateNamePredicate {
    fn name(&self) -> &str {
        "is_state_name"
    }

    fn evaluate(&self, value: &Value, args: &Value) -> bool {
        let s = match value.as_str() {
            Some(s) => s.trim(),
            None => return false,
        };

        if s.is_empty() {
            return false;
        }

        let include_territories = args
            .get("include_territories")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        if US_STATE_NAMES
            .iter()
            .any(|name| name.eq_ignore_ascii_case(s))
        {
            return true;
        }

        if include_territories
            && US_TERRITORY_NAMES
                .iter()
                .any(|name| name.eq_ignore_ascii_case(s))
        {
            return true;
        }

        false
    }
}

// ============================================================================
// US State Code
// ============================================================================

/// Validate a US state, territory, or military postal code.
struct IsUsStatePredicate;

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
// US ZIP Code
// ============================================================================

/// Validate US ZIP code format (5 or 9 digits).
struct IsUsZipPredicate;

impl NamedPredicate for IsUsZipPredicate {
    fn name(&self) -> &str {
        "is_us_zip"
    }

    fn evaluate(&self, value: &Value, _args: &Value) -> bool {
        let s = match value.as_str() {
            Some(s) => s,
            None => return false,
        };

        let digits: String = s.chars().filter(|c| c.is_ascii_digit()).collect();
        if digits.len() != 5 && digits.len() != 9 {
            return false;
        }
        if digits.chars().all(|c| c == '0') {
            return false;
        }

        true
    }
}

// ============================================================================
// ISO Currency Code (ISO 4217)
// ============================================================================

/// Validate an ISO 4217 currency code.
struct IsCurrencyCodePredicate;

impl NamedPredicate for IsCurrencyCodePredicate {
    fn name(&self) -> &str {
        "is_currency_code"
    }

    fn evaluate(&self, value: &Value, _args: &Value) -> bool {
        let s = match value.as_str() {
            Some(s) => s.trim().to_uppercase(),
            None => return false,
        };

        if s.len() != 3 {
            return false;
        }

        ISO_CURRENCY_CODES.contains(&s.as_str())
    }
}

// ============================================================================
// W-2 Box 12 Codes
// ============================================================================

/// Validate a W-2 Box 12 code.
///
/// Box 12 codes are single or double letter codes (A-HH) that identify
/// various types of compensation and benefits.
struct IsW2Box12CodePredicate;

impl NamedPredicate for IsW2Box12CodePredicate {
    fn name(&self) -> &str {
        "is_w2_box12_code"
    }

    fn evaluate(&self, value: &Value, _args: &Value) -> bool {
        let s = match value.as_str() {
            Some(s) => s.trim().to_uppercase(),
            None => return false,
        };

        W2_BOX12_CODES.contains(&s.as_str())
    }
}

// ============================================================================
// 1099-B Transaction Codes
// ============================================================================

/// Validate a 1099-B transaction type code.
///
/// Codes indicate short-term vs long-term and basis reporting.
struct Is1099BCodePredicate;

impl NamedPredicate for Is1099BCodePredicate {
    fn name(&self) -> &str {
        "is_1099b_code"
    }

    fn evaluate(&self, value: &Value, args: &Value) -> bool {
        let s = match value.as_str() {
            Some(s) => s.trim().to_uppercase(),
            None => return false,
        };

        // Check specific code type if specified
        let code_type = args.get("type").and_then(|v| v.as_str());

        match code_type {
            Some("term") => TERM_CODES.contains(&s.as_str()),
            Some("basis") => BASIS_REPORTED_CODES.contains(&s.as_str()),
            Some("type") => TRANSACTION_TYPE_CODES.contains(&s.as_str()),
            None | Some(_) => {
                // Any valid 1099-B code
                TERM_CODES.contains(&s.as_str())
                    || BASIS_REPORTED_CODES.contains(&s.as_str())
                    || TRANSACTION_TYPE_CODES.contains(&s.as_str())
            }
        }
    }
}

// ============================================================================
// Filing Status
// ============================================================================

/// Validate a tax filing status code.
struct IsFilingStatusPredicate;

impl NamedPredicate for IsFilingStatusPredicate {
    fn name(&self) -> &str {
        "is_filing_status"
    }

    fn evaluate(&self, value: &Value, _args: &Value) -> bool {
        let s = match value.as_str() {
            Some(s) => s.trim().to_uppercase(),
            None => return false,
        };

        FILING_STATUSES.contains(&s.as_str())
    }
}

// ============================================================================
// Reference Data Constants
// ============================================================================

/// Common country names (English) used in tax and financial contexts.
const COUNTRY_NAMES: &[&str] = &[
    "United States",
    "Canada",
    "Mexico",
    "United Kingdom",
    "Germany",
    "France",
    "Italy",
    "Spain",
    "Japan",
    "China",
    "India",
    "Australia",
    "Brazil",
    "South Korea",
    "Russia",
    "Netherlands",
    "Switzerland",
    "Ireland",
    "Singapore",
    "Hong Kong",
];

/// US state names (50 states + DC).
const US_STATE_NAMES: &[&str] = &[
    "Alabama",
    "Alaska",
    "Arizona",
    "Arkansas",
    "California",
    "Colorado",
    "Connecticut",
    "Delaware",
    "District of Columbia",
    "Florida",
    "Georgia",
    "Hawaii",
    "Idaho",
    "Illinois",
    "Indiana",
    "Iowa",
    "Kansas",
    "Kentucky",
    "Louisiana",
    "Maine",
    "Maryland",
    "Massachusetts",
    "Michigan",
    "Minnesota",
    "Mississippi",
    "Missouri",
    "Montana",
    "Nebraska",
    "Nevada",
    "New Hampshire",
    "New Jersey",
    "New Mexico",
    "New York",
    "North Carolina",
    "North Dakota",
    "Ohio",
    "Oklahoma",
    "Oregon",
    "Pennsylvania",
    "Rhode Island",
    "South Carolina",
    "South Dakota",
    "Tennessee",
    "Texas",
    "Utah",
    "Vermont",
    "Virginia",
    "Washington",
    "West Virginia",
    "Wisconsin",
    "Wyoming",
];

/// US territory names commonly present on tax forms.
const US_TERRITORY_NAMES: &[&str] = &[
    "American Samoa",
    "Guam",
    "Northern Mariana Islands",
    "Puerto Rico",
    "U.S. Virgin Islands",
    "United States Minor Outlying Islands",
];

/// Valid US state, territory, and military postal codes.
const US_STATE_CODES: &[&str] = &[
    "AL", "AK", "AZ", "AR", "CA", "CO", "CT", "DE", "FL", "GA", "HI", "ID", "IL", "IN", "IA", "KS",
    "KY", "LA", "ME", "MD", "MA", "MI", "MN", "MS", "MO", "MT", "NE", "NV", "NH", "NJ", "NM", "NY",
    "NC", "ND", "OH", "OK", "OR", "PA", "RI", "SC", "SD", "TN", "TX", "UT", "VT", "VA", "WA", "WV",
    "WI", "WY", "DC", "AS", "GU", "MP", "PR", "VI", "UM", "AA", "AE", "AP",
];

/// ISO 3166-1 alpha-2 country codes (common subset for tax forms).
const ISO_COUNTRY_CODES: &[&str] = &[
    // Major countries frequently seen on tax forms
    "US", "CA", "MX", "GB", "DE", "FR", "IT", "ES", "JP", "CN", "IN", "AU", "BR", "KR", "RU",
    // European Union
    "AT", "BE", "BG", "HR", "CY", "CZ", "DK", "EE", "FI", "GR", "HU", "IE", "LV", "LT", "LU", "MT",
    "NL", "PL", "PT", "RO", "SK", "SI", "SE", // Other common
    "CH", "NO", "IL", "SG", "HK", "TW", "NZ", "ZA", "AE", "SA", "AR", "CL", "CO", "PE", "VE", "PH",
    "TH", "VN", "MY", "ID", "PK", "BD", "EG", "NG", "KE", "TR", "UA",
    // Tax havens / financial centers
    "BM", "KY", "VG", "BS", "PA", "LI", "MC", "AD", "GI", "JE", "GG", "IM", "LU", "MT", "CY",
    // Caribbean
    "JM", "TT", "BB", "PR", "VI", "CU", "DO", "HT", // Central America
    "GT", "HN", "SV", "NI", "CR", "BZ", // More countries for completeness
    "AF", "AL", "DZ", "AO", "AM", "AZ", "BH", "BY", "BJ", "BT", "BO", "BA", "BW", "BN", "BF", "BI",
    "KH", "CM", "CV", "CF", "TD", "CI", "CD", "CG", "DJ", "EC", "GQ", "ER", "ET", "FJ", "GA", "GM",
    "GE", "GH", "GN", "GW", "GY", "IQ", "IR", "IS", "JO", "KZ", "KW", "KG", "LA", "LB", "LS", "LR",
    "LY", "MK", "MG", "MW", "MV", "ML", "MR", "MU", "MD", "MN", "ME", "MA", "MZ", "MM", "NA", "NP",
    "NE", "KP", "OM", "QA", "RW", "RS", "SL", "SO", "SS", "SD", "SR", "SZ", "SY", "TJ", "TZ", "TG",
    "TO", "TN", "TM", "UG", "UY", "UZ", "VU", "YE", "ZM", "ZW",
];

/// ISO 4217 currency codes (common subset).
const ISO_CURRENCY_CODES: &[&str] = &[
    "USD", "EUR", "GBP", "JPY", "CNY", "CAD", "AUD", "CHF", "HKD", "SGD", "SEK", "NOK", "DKK",
    "NZD", "MXN", "BRL", "INR", "RUB", "ZAR", "KRW", "TWD", "THB", "MYR", "IDR", "PHP", "VND",
    "AED", "SAR", "ILS", "TRY", "PLN", "CZK", "HUF", "RON", "BGN", "HRK", "ISK", "CLP", "COP",
    "PEN", "ARS", "EGP", "NGN", "KES", "PKR", "BDT",
];

/// W-2 Box 12 codes.
///
/// See IRS instructions for W-2 for complete descriptions.
const W2_BOX12_CODES: &[&str] = &[
    // Single letter codes
    "A", // Uncollected social security or RRTA tax on tips
    "B", // Uncollected Medicare tax on tips
    "C", // Taxable cost of group-term life insurance over $50,000
    "D", // Elective deferrals under 401(k)
    "E", // Elective deferrals under 403(b)
    "F", // Elective deferrals under 408(k)(6) SARSEP
    "G", // Elective deferrals and employer contributions to 457(b)
    "H", // Elective deferrals under 501(c)(18)(D)
    "J", // Nontaxable sick pay
    "K", // 20% excise tax on excess golden parachute payments
    "L", // Substantiated employee business expense reimbursements
    "M", // Uncollected social security or RRTA tax on taxable cost of group-term life
    "N", // Uncollected Medicare tax on taxable cost of group-term life
    "P", // Excludable moving expense reimbursements
    "Q", // Nontaxable combat pay
    "R", // Employer contributions to Archer MSA
    "S", // Employee salary reduction contributions under 408(p) SIMPLE
    "T", // Adoption benefits
    "V", // Income from exercise of nonstatutory stock options
    "W", // Employer contributions to HSA
    "Y", // Deferrals under 409A nonqualified deferred compensation plan
    "Z", // Income under 409A on nonqualified deferred compensation plan
    // Double letter codes
    "AA", // Designated Roth contributions under 401(k)
    "BB", // Designated Roth contributions under 403(b)
    "DD", // Cost of employer-sponsored health coverage
    "EE", // Designated Roth contributions under governmental 457(b)
    "FF", // Permitted benefits under qualified small employer HRA
    "GG", // Income from qualified equity grants under 83(i)
    "HH", // Aggregate deferrals under 83(i) elections
];

/// 1099-B term codes (short-term vs long-term).
const TERM_CODES: &[&str] = &[
    "A", // Short-term, basis reported to IRS
    "B", // Short-term, basis NOT reported to IRS
    "C", // Short-term, basis reporting unknown
    "D", // Long-term, basis reported to IRS
    "E", // Long-term, basis NOT reported to IRS
    "F", // Long-term, basis reporting unknown
    "X", // Unable to determine term
];

/// 1099-B basis reported codes.
const BASIS_REPORTED_CODES: &[&str] = &[
    "1", // Basis reported to IRS
    "2", // Basis NOT reported to IRS
    "3", // Unknown
];

/// 1099-B transaction type codes.
const TRANSACTION_TYPE_CODES: &[&str] = &[
    "P", // Ordinary (regular sale)
    "S", // Short sale
    "C", // Collectibles (28% rate gain)
    "O", // QOF (Qualified Opportunity Fund)
    "W", // Wash sale (loss disallowed)
];

/// Tax filing status codes.
const FILING_STATUSES: &[&str] = &[
    "S",   // Single
    "MFJ", // Married Filing Jointly
    "MFS", // Married Filing Separately
    "HOH", // Head of Household
    "QW",  // Qualifying Widow(er)
    // Numeric versions (used in some systems)
    "1", // Single
    "2", // Married Filing Jointly
    "3", // Married Filing Separately
    "4", // Head of Household
    "5", // Qualifying Widow(er)
];

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_is_country_code() {
        let pred = IsCountryCodePredicate;

        assert!(pred.evaluate(&json!("US"), &json!(null)));
        assert!(pred.evaluate(&json!("us"), &json!(null))); // Case insensitive
        assert!(pred.evaluate(&json!("GB"), &json!(null)));
        assert!(pred.evaluate(&json!("JP"), &json!(null)));

        assert!(!pred.evaluate(&json!("XX"), &json!(null)));
        assert!(!pred.evaluate(&json!("USA"), &json!(null))); // 3 chars
        assert!(!pred.evaluate(&json!("U"), &json!(null))); // 1 char

        // Exclude US for foreign country fields
        assert!(!pred.evaluate(&json!("US"), &json!({"exclude_us": true})));
        assert!(pred.evaluate(&json!("GB"), &json!({"exclude_us": true})));
    }

    #[test]
    fn test_is_currency_code() {
        let pred = IsCurrencyCodePredicate;

        assert!(pred.evaluate(&json!("USD"), &json!(null)));
        assert!(pred.evaluate(&json!("usd"), &json!(null)));
        assert!(pred.evaluate(&json!("EUR"), &json!(null)));
        assert!(pred.evaluate(&json!("GBP"), &json!(null)));

        assert!(!pred.evaluate(&json!("XXX"), &json!(null)));
        assert!(!pred.evaluate(&json!("US"), &json!(null))); // 2 chars
    }

    #[test]
    fn test_is_us_state() {
        let pred = IsUsStatePredicate;

        assert!(pred.evaluate(&json!("CA"), &json!(null)));
        assert!(pred.evaluate(&json!("ca"), &json!(null)));
        assert!(pred.evaluate(&json!("PR"), &json!(null)));
        assert!(pred.evaluate(&json!("AE"), &json!(null)));

        assert!(!pred.evaluate(&json!("California"), &json!(null)));
        assert!(!pred.evaluate(&json!("XX"), &json!(null)));
    }

    #[test]
    fn test_is_us_zip() {
        let pred = IsUsZipPredicate;

        assert!(pred.evaluate(&json!("94105"), &json!(null)));
        assert!(pred.evaluate(&json!("94105-1234"), &json!(null)));

        assert!(!pred.evaluate(&json!("00000"), &json!(null)));
        assert!(!pred.evaluate(&json!("9410"), &json!(null)));
    }

    #[test]
    fn test_reference_registry_includes_us_state_and_zip() {
        let mut registry = PredicateRegistry::new();
        register_reference_predicates(&mut registry);

        assert!(registry.get("is_us_state").is_some());
        assert!(registry.get("is_us_zip").is_some());
    }

    #[test]
    fn test_is_w2_box12_code() {
        let pred = IsW2Box12CodePredicate;

        // Single letter codes
        assert!(pred.evaluate(&json!("D"), &json!(null))); // 401(k)
        assert!(pred.evaluate(&json!("W"), &json!(null))); // HSA

        // Double letter codes
        assert!(pred.evaluate(&json!("DD"), &json!(null))); // Health coverage
        assert!(pred.evaluate(&json!("AA"), &json!(null))); // Roth 401(k)

        // Case insensitive
        assert!(pred.evaluate(&json!("dd"), &json!(null)));

        // Invalid
        assert!(!pred.evaluate(&json!("X"), &json!(null))); // Not a valid code
        assert!(!pred.evaluate(&json!("DDD"), &json!(null)));
    }

    #[test]
    fn test_is_1099b_code() {
        let pred = Is1099BCodePredicate;

        // Term codes
        assert!(pred.evaluate(&json!("A"), &json!(null))); // Short-term
        assert!(pred.evaluate(&json!("D"), &json!(null))); // Long-term

        // Basis codes
        assert!(pred.evaluate(&json!("1"), &json!(null)));

        // Transaction type codes
        assert!(pred.evaluate(&json!("P"), &json!(null))); // Ordinary
        assert!(pred.evaluate(&json!("W"), &json!(null))); // Wash sale

        // Filter by type
        assert!(pred.evaluate(&json!("A"), &json!({"type": "term"})));
        assert!(!pred.evaluate(&json!("1"), &json!({"type": "term"})));
        assert!(pred.evaluate(&json!("1"), &json!({"type": "basis"})));
    }

    #[test]
    fn test_is_filing_status() {
        let pred = IsFilingStatusPredicate;

        assert!(pred.evaluate(&json!("S"), &json!(null)));
        assert!(pred.evaluate(&json!("MFJ"), &json!(null)));
        assert!(pred.evaluate(&json!("HOH"), &json!(null)));
        assert!(pred.evaluate(&json!("1"), &json!(null)));

        assert!(!pred.evaluate(&json!("SINGLE"), &json!(null)));
    }
}

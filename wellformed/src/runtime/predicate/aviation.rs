//! Aviation and travel code validation predicates.
//!
//! Validates airport codes, airline codes, and flight numbers.

use super::registry::{NamedPredicate, PredicateRegistry};
use serde_json::Value;
use std::sync::Arc;

/// Register all aviation predicates.
pub fn register_aviation_predicates(registry: &mut PredicateRegistry) {
    registry.register(Arc::new(IsIataAirportCodePredicate));
    registry.register(Arc::new(IsIcaoAirportCodePredicate));
    registry.register(Arc::new(IsAirportCodePredicate));
    registry.register(Arc::new(IsIataAirlineCodePredicate));
    registry.register(Arc::new(IsIcaoAirlineCodePredicate));
    registry.register(Arc::new(IsAirlineCodePredicate));
    registry.register(Arc::new(IsFlightNumberPredicate));
}

/// Validate an IATA airport code (3 letters, e.g., SFO).
///
/// Optional args:
/// - `known_only` (bool): If true, require membership in a curated known-code set.
struct IsIataAirportCodePredicate;

impl NamedPredicate for IsIataAirportCodePredicate {
    fn name(&self) -> &str {
        "is_iata_airport_code"
    }

    fn evaluate(&self, value: &Value, args: &Value) -> bool {
        let known_only = args
            .get("known_only")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        validate_iata_airport_code(value, known_only)
    }
}

/// Validate an ICAO airport code (4 letters, e.g., KSFO).
///
/// Optional args:
/// - `known_only` (bool): If true, require membership in a curated known-code set.
struct IsIcaoAirportCodePredicate;

impl NamedPredicate for IsIcaoAirportCodePredicate {
    fn name(&self) -> &str {
        "is_icao_airport_code"
    }

    fn evaluate(&self, value: &Value, args: &Value) -> bool {
        let known_only = args
            .get("known_only")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        validate_icao_airport_code(value, known_only)
    }
}

/// Validate an airport code as either IATA or ICAO.
///
/// Optional args:
/// - `system` ("ANY" | "IATA" | "ICAO"), default "ANY"
/// - `known_only` (bool), default false
struct IsAirportCodePredicate;

impl NamedPredicate for IsAirportCodePredicate {
    fn name(&self) -> &str {
        "is_airport_code"
    }

    fn evaluate(&self, value: &Value, args: &Value) -> bool {
        let system = args
            .get("system")
            .and_then(|v| v.as_str())
            .unwrap_or("ANY")
            .to_uppercase();
        let known_only = args
            .get("known_only")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        match system.as_str() {
            "IATA" => validate_iata_airport_code(value, known_only),
            "ICAO" => validate_icao_airport_code(value, known_only),
            "ANY" => {
                validate_iata_airport_code(value, known_only)
                    || validate_icao_airport_code(value, known_only)
            }
            _ => false,
        }
    }
}

/// Validate an IATA airline code (2 alphanumeric chars, e.g., UA, B6).
///
/// Optional args:
/// - `known_only` (bool): If true, require membership in a curated known-code set.
struct IsIataAirlineCodePredicate;

impl NamedPredicate for IsIataAirlineCodePredicate {
    fn name(&self) -> &str {
        "is_iata_airline_code"
    }

    fn evaluate(&self, value: &Value, args: &Value) -> bool {
        let known_only = args
            .get("known_only")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        validate_iata_airline_code(value, known_only)
    }
}

/// Validate an ICAO airline code (3 letters, e.g., UAL).
///
/// Optional args:
/// - `known_only` (bool): If true, require membership in a curated known-code set.
struct IsIcaoAirlineCodePredicate;

impl NamedPredicate for IsIcaoAirlineCodePredicate {
    fn name(&self) -> &str {
        "is_icao_airline_code"
    }

    fn evaluate(&self, value: &Value, args: &Value) -> bool {
        let known_only = args
            .get("known_only")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        validate_icao_airline_code(value, known_only)
    }
}

/// Validate an airline code as either IATA or ICAO.
///
/// Optional args:
/// - `system` ("ANY" | "IATA" | "ICAO"), default "ANY"
/// - `known_only` (bool), default false
struct IsAirlineCodePredicate;

impl NamedPredicate for IsAirlineCodePredicate {
    fn name(&self) -> &str {
        "is_airline_code"
    }

    fn evaluate(&self, value: &Value, args: &Value) -> bool {
        let system = args
            .get("system")
            .and_then(|v| v.as_str())
            .unwrap_or("ANY")
            .to_uppercase();
        let known_only = args
            .get("known_only")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        match system.as_str() {
            "IATA" => validate_iata_airline_code(value, known_only),
            "ICAO" => validate_icao_airline_code(value, known_only),
            "ANY" => {
                validate_iata_airline_code(value, known_only)
                    || validate_icao_airline_code(value, known_only)
            }
            _ => false,
        }
    }
}

/// Validate a flight number (e.g., UA123, UAL1234A).
///
/// Optional args:
/// - `carrier_format` ("ANY" | "IATA" | "ICAO"), default "ANY"
/// - `known_carrier` (bool), default false
/// - `allow_suffix` (bool), default true
struct IsFlightNumberPredicate;

impl NamedPredicate for IsFlightNumberPredicate {
    fn name(&self) -> &str {
        "is_flight_number"
    }

    fn evaluate(&self, value: &Value, args: &Value) -> bool {
        let s = match value.as_str() {
            Some(s) => s.trim().to_uppercase(),
            None => return false,
        };

        if s.is_empty() {
            return false;
        }

        let compact: String = s.chars().filter(|c| !c.is_ascii_whitespace()).collect();
        if compact.len() < 3 {
            return false;
        }

        let carrier_format = args
            .get("carrier_format")
            .and_then(|v| v.as_str())
            .unwrap_or("ANY")
            .to_uppercase();
        let known_carrier = args
            .get("known_carrier")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let allow_suffix = args
            .get("allow_suffix")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        match carrier_format.as_str() {
            "IATA" => validate_flight_number(&compact, 2, known_carrier, allow_suffix),
            "ICAO" => validate_flight_number(&compact, 3, known_carrier, allow_suffix),
            _ => {
                validate_flight_number(&compact, 2, known_carrier, allow_suffix)
                    || validate_flight_number(&compact, 3, known_carrier, allow_suffix)
            }
        }
    }
}

fn validate_flight_number(
    s: &str,
    carrier_len: usize,
    known_carrier: bool,
    allow_suffix: bool,
) -> bool {
    if s.len() <= carrier_len {
        return false;
    }

    let (carrier, rest) = s.split_at(carrier_len);
    let carrier_valid = if carrier_len == 2 {
        carrier
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
    } else {
        carrier.chars().all(|c| c.is_ascii_uppercase())
    };
    if !carrier_valid {
        return false;
    }

    if known_carrier {
        let known = if carrier_len == 2 {
            IATA_AIRLINE_CODES.contains(&carrier)
        } else {
            ICAO_AIRLINE_CODES.contains(&carrier)
        };
        if !known {
            return false;
        }
    }

    if rest.is_empty() {
        return false;
    }

    if allow_suffix
        && rest.len() >= 2
        && rest[..rest.len() - 1].chars().all(|c| c.is_ascii_digit())
        && rest.len() - 1 <= 4
        && rest.chars().last().is_some_and(|c| c.is_ascii_uppercase())
    {
        return true;
    }

    rest.len() <= 4 && rest.chars().all(|c| c.is_ascii_digit())
}

fn validate_iata_airport_code(value: &Value, known_only: bool) -> bool {
    let s = match value.as_str() {
        Some(s) => s.trim().to_uppercase(),
        None => return false,
    };

    if s.len() != 3 || !s.chars().all(|c| c.is_ascii_uppercase()) {
        return false;
    }

    !known_only || IATA_AIRPORT_CODES.contains(&s.as_str())
}

fn validate_icao_airport_code(value: &Value, known_only: bool) -> bool {
    let s = match value.as_str() {
        Some(s) => s.trim().to_uppercase(),
        None => return false,
    };

    if s.len() != 4 || !s.chars().all(|c| c.is_ascii_uppercase()) {
        return false;
    }

    !known_only || ICAO_AIRPORT_CODES.contains(&s.as_str())
}

fn validate_iata_airline_code(value: &Value, known_only: bool) -> bool {
    let s = match value.as_str() {
        Some(s) => s.trim().to_uppercase(),
        None => return false,
    };

    if s.len() != 2
        || !s
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
    {
        return false;
    }

    !known_only || IATA_AIRLINE_CODES.contains(&s.as_str())
}

fn validate_icao_airline_code(value: &Value, known_only: bool) -> bool {
    let s = match value.as_str() {
        Some(s) => s.trim().to_uppercase(),
        None => return false,
    };

    if s.len() != 3 || !s.chars().all(|c| c.is_ascii_uppercase()) {
        return false;
    }

    !known_only || ICAO_AIRLINE_CODES.contains(&s.as_str())
}

// Common airport and airline codes used for strict known-only mode.
const IATA_AIRPORT_CODES: &[&str] = &[
    "SFO", "LAX", "JFK", "ORD", "DFW", "DEN", "SEA", "BOS", "MIA", "ATL", "PHX", "IAH", "EWR",
    "LHR", "LGW", "CDG", "FRA", "AMS", "MAD", "BCN", "NRT", "HND", "KIX", "SYD", "MEL", "YYZ",
    "YVR", "DXB", "SIN", "HKG",
];

const ICAO_AIRPORT_CODES: &[&str] = &[
    "KSFO", "KLAX", "KJFK", "KORD", "KDFW", "KDEN", "KSEA", "KBOS", "KMIA", "KATL", "KPHX", "KIAH",
    "KEWR", "EGLL", "EGKK", "LFPG", "EDDF", "EHAM", "LEMD", "LEBL", "RJAA", "RJTT", "RJBB", "YSSY",
    "YMML", "CYYZ", "CYVR", "OMDB", "WSSS", "VHHH",
];

const IATA_AIRLINE_CODES: &[&str] = &[
    "UA", "AA", "DL", "WN", "B6", "AS", "NK", "F9", "AC", "BA", "LH", "AF", "KL", "EK", "SQ", "NH",
    "JL", "QF",
];

const ICAO_AIRLINE_CODES: &[&str] = &[
    "UAL", "AAL", "DAL", "SWA", "JBU", "ASA", "NKS", "FFT", "ACA", "BAW", "DLH", "AFR", "KLM",
    "UAE", "SIA", "ANA", "JAL", "QFA",
];

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_iata_airport_code() {
        let pred = IsIataAirportCodePredicate;
        assert!(pred.evaluate(&json!("SFO"), &json!(null)));
        assert!(pred.evaluate(&json!("sfo"), &json!(null)));
        assert!(!pred.evaluate(&json!("SF"), &json!(null)));
        assert!(!pred.evaluate(&json!("123"), &json!(null)));
        assert!(pred.evaluate(&json!("SFO"), &json!({"known_only": true})));
        assert!(!pred.evaluate(&json!("ZZZ"), &json!({"known_only": true})));
    }

    #[test]
    fn test_icao_airport_code() {
        let pred = IsIcaoAirportCodePredicate;
        assert!(pred.evaluate(&json!("KSFO"), &json!(null)));
        assert!(pred.evaluate(&json!("ksfo"), &json!(null)));
        assert!(!pred.evaluate(&json!("SFO"), &json!(null)));
        assert!(pred.evaluate(&json!("KSFO"), &json!({"known_only": true})));
        assert!(!pred.evaluate(&json!("ZZZZ"), &json!({"known_only": true})));
    }

    #[test]
    fn test_airline_codes() {
        let iata = IsIataAirlineCodePredicate;
        let icao = IsIcaoAirlineCodePredicate;

        assert!(iata.evaluate(&json!("UA"), &json!(null)));
        assert!(iata.evaluate(&json!("B6"), &json!(null)));
        assert!(!iata.evaluate(&json!("UAL"), &json!(null)));
        assert!(iata.evaluate(&json!("UA"), &json!({"known_only": true})));
        assert!(!iata.evaluate(&json!("ZZ"), &json!({"known_only": true})));

        assert!(icao.evaluate(&json!("UAL"), &json!(null)));
        assert!(!icao.evaluate(&json!("UA"), &json!(null)));
        assert!(icao.evaluate(&json!("UAL"), &json!({"known_only": true})));
        assert!(!icao.evaluate(&json!("ZZZ"), &json!({"known_only": true})));
    }

    #[test]
    fn test_flight_number() {
        let pred = IsFlightNumberPredicate;

        assert!(pred.evaluate(&json!("UA123"), &json!(null)));
        assert!(pred.evaluate(&json!("ual1234a"), &json!(null)));
        assert!(pred.evaluate(&json!("UA 123"), &json!(null)));
        assert!(!pred.evaluate(&json!("UA12345"), &json!(null))); // too many digits
        assert!(!pred.evaluate(&json!("U@123"), &json!(null)));

        // Force format
        assert!(pred.evaluate(&json!("UAL123"), &json!({"carrier_format": "ICAO"})));
        assert!(!pred.evaluate(&json!("UA123"), &json!({"carrier_format": "ICAO"})));

        // Known carrier mode
        assert!(pred.evaluate(
            &json!("UA123"),
            &json!({"known_carrier": true, "carrier_format": "IATA"})
        ));
        assert!(!pred.evaluate(
            &json!("ZZ123"),
            &json!({"known_carrier": true, "carrier_format": "IATA"})
        ));

        // Suffix toggle
        assert!(!pred.evaluate(
            &json!("UAL123A"),
            &json!({"allow_suffix": false, "carrier_format": "ICAO"})
        ));
    }

    #[test]
    fn test_airport_code_convenience() {
        let pred = IsAirportCodePredicate;
        assert!(pred.evaluate(&json!("SFO"), &json!(null)));
        assert!(pred.evaluate(&json!("KSFO"), &json!(null)));
        assert!(pred.evaluate(&json!("SFO"), &json!({"system": "IATA"})));
        assert!(!pred.evaluate(&json!("SFO"), &json!({"system": "ICAO"})));
        assert!(!pred.evaluate(
            &json!("ZZZ"),
            &json!({"system": "IATA", "known_only": true})
        ));
    }

    #[test]
    fn test_airline_code_convenience() {
        let pred = IsAirlineCodePredicate;
        assert!(pred.evaluate(&json!("UA"), &json!(null)));
        assert!(pred.evaluate(&json!("UAL"), &json!(null)));
        assert!(pred.evaluate(&json!("UAL"), &json!({"system": "ICAO"})));
        assert!(!pred.evaluate(&json!("UAL"), &json!({"system": "IATA"})));
        assert!(!pred.evaluate(&json!("ZZ"), &json!({"system": "IATA", "known_only": true})));
    }
}

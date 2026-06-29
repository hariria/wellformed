//! Date validation predicates.
//!
//! Validates dates, date ranges, and date comparisons for tax forms.

use super::registry::{NamedPredicate, PredicateRegistry};
use serde_json::Value;
use std::sync::Arc;

/// Register all date predicates.
pub fn register_date_predicates(registry: &mut PredicateRegistry) {
    registry.register(Arc::new(IsDatePredicate));
    registry.register(Arc::new(IsTimePredicate));
    registry.register(Arc::new(IsIsoDatetimePredicate));
    registry.register(Arc::new(IsTaxYearPredicate));
    registry.register(Arc::new(DateInRangePredicate));
    registry.register(Arc::new(DateBeforePredicate));
    registry.register(Arc::new(DateAfterPredicate));
    registry.register(Arc::new(TimeBeforePredicate));
    registry.register(Arc::new(TimeAfterPredicate));
    registry.register(Arc::new(TimeInRangePredicate));
}

// ============================================================================
// Date Parsing Helpers
// ============================================================================

/// Parsed date components (year, month, day).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct ParsedDate {
    year: u32,
    month: u32,
    day: u32,
}

impl ParsedDate {
    /// Parse a date string in various formats.
    ///
    /// Supported formats:
    /// - MM/DD/YYYY (US format)
    /// - YYYY-MM-DD (ISO 8601)
    /// - MM-DD-YYYY
    /// - MMDDYYYY (no separators)
    fn parse(s: &str) -> Option<Self> {
        let s = s.trim();

        // Try MM/DD/YYYY or MM-DD-YYYY
        if s.len() == 10 && (s.chars().nth(2) == Some('/') || s.chars().nth(2) == Some('-')) {
            let sep = s.chars().nth(2).unwrap();
            if s.chars().nth(5) == Some(sep) {
                let parts: Vec<&str> = s.split(sep).collect();
                if parts.len() == 3 {
                    let month: u32 = parts[0].parse().ok()?;
                    let day: u32 = parts[1].parse().ok()?;
                    let year: u32 = parts[2].parse().ok()?;
                    return Self::validate(year, month, day);
                }
            }
        }

        // Try YYYY-MM-DD (ISO 8601)
        if s.len() == 10 && s.chars().nth(4) == Some('-') && s.chars().nth(7) == Some('-') {
            let parts: Vec<&str> = s.split('-').collect();
            if parts.len() == 3 {
                let year: u32 = parts[0].parse().ok()?;
                let month: u32 = parts[1].parse().ok()?;
                let day: u32 = parts[2].parse().ok()?;
                return Self::validate(year, month, day);
            }
        }

        // Try MMDDYYYY (no separators)
        if s.len() == 8 && s.chars().all(|c| c.is_ascii_digit()) {
            let month: u32 = s[0..2].parse().ok()?;
            let day: u32 = s[2..4].parse().ok()?;
            let year: u32 = s[4..8].parse().ok()?;
            return Self::validate(year, month, day);
        }

        None
    }

    /// Validate and create a ParsedDate if the date is valid.
    fn validate(year: u32, month: u32, day: u32) -> Option<Self> {
        // Basic range checks
        if !(1..=12).contains(&month) {
            return None;
        }
        if day < 1 {
            return None;
        }

        // Days in month (accounting for leap years)
        let days_in_month = match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 => {
                if is_leap_year(year) {
                    29
                } else {
                    28
                }
            }
            _ => return None,
        };

        if day > days_in_month {
            return None;
        }

        Some(Self { year, month, day })
    }
}

fn is_leap_year(year: u32) -> bool {
    (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400)
}

// ============================================================================
// Is Date Predicate
// ============================================================================

/// Validate that a string is a valid date.
///
/// Supports formats: MM/DD/YYYY, YYYY-MM-DD, MM-DD-YYYY, MMDDYYYY
struct IsDatePredicate;

impl NamedPredicate for IsDatePredicate {
    fn name(&self) -> &str {
        "is_date"
    }

    fn evaluate(&self, value: &Value, _args: &Value) -> bool {
        let s = match value.as_str() {
            Some(s) => s,
            None => return false,
        };

        ParsedDate::parse(s).is_some()
    }
}

// ============================================================================
// Time Predicate
// ============================================================================

/// Validate a time string.
///
/// Supported formats:
/// - HH:MM (24-hour)
/// - HH:MM:SS (24-hour with seconds)
/// - h:MM AM/PM or hh:MM AM/PM (12-hour)
/// - h:MM:SS AM/PM or hh:MM:SS AM/PM (12-hour with seconds)
///
/// Args: `{ "format": "24h" | "12h" }` (optional, default accepts both)
struct IsTimePredicate;

impl NamedPredicate for IsTimePredicate {
    fn name(&self) -> &str {
        "is_time"
    }

    fn evaluate(&self, value: &Value, args: &Value) -> bool {
        let s = match value.as_str() {
            Some(s) => s.trim(),
            None => return false,
        };

        let format = args.get("format").and_then(|v| v.as_str());

        match format {
            Some("24h") => parse_time_24h(s),
            Some("12h") => parse_time_12h(s),
            _ => parse_time_24h(s) || parse_time_12h(s),
        }
    }
}

/// Parse 24-hour time: HH:MM or HH:MM:SS
fn parse_time_24h(s: &str) -> bool {
    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() != 2 && parts.len() != 3 {
        return false;
    }

    // All parts must be digits
    if !parts
        .iter()
        .all(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit()))
    {
        return false;
    }

    let hour: u32 = match parts[0].parse() {
        Ok(h) => h,
        Err(_) => return false,
    };
    let minute: u32 = match parts[1].parse() {
        Ok(m) => m,
        Err(_) => return false,
    };

    if hour > 23 || minute > 59 {
        return false;
    }

    // Hour and minute parts must be exactly 2 digits
    if parts[0].len() != 2 || parts[1].len() != 2 {
        return false;
    }

    if parts.len() == 3 {
        if parts[2].len() != 2 {
            return false;
        }
        let second: u32 = match parts[2].parse() {
            Ok(s) => s,
            Err(_) => return false,
        };
        if second > 59 {
            return false;
        }
    }

    true
}

/// Parse 12-hour time: h:MM AM/PM or hh:MM AM/PM (with optional :SS)
fn parse_time_12h(s: &str) -> bool {
    // Must end with AM or PM (case insensitive)
    let upper = s.to_uppercase();
    let (time_part, period) = if upper.ends_with("AM") || upper.ends_with("PM") {
        let period = &upper[upper.len() - 2..];
        let time_str = s[..s.len() - 2].trim();
        (time_str.to_string(), period.to_string())
    } else {
        return false;
    };

    let _ = period; // validated above

    let parts: Vec<&str> = time_part.split(':').collect();
    if parts.len() != 2 && parts.len() != 3 {
        return false;
    }

    if !parts
        .iter()
        .all(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit()))
    {
        return false;
    }

    let hour: u32 = match parts[0].parse() {
        Ok(h) => h,
        Err(_) => return false,
    };
    let minute: u32 = match parts[1].parse() {
        Ok(m) => m,
        Err(_) => return false,
    };

    // 12-hour: 1-12
    if !(1..=12).contains(&hour) {
        return false;
    }
    if minute > 59 {
        return false;
    }

    if parts.len() == 3 {
        let second: u32 = match parts[2].parse() {
            Ok(s) => s,
            Err(_) => return false,
        };
        if second > 59 {
            return false;
        }
    }

    true
}

// ============================================================================
// ISO 8601 Datetime Predicate
// ============================================================================

/// Validate an ISO 8601 datetime string.
///
/// Accepted formats:
/// - `YYYY-MM-DDTHH:MM:SSZ`
/// - `YYYY-MM-DDTHH:MM:SS+HH:MM` / `YYYY-MM-DDTHH:MM:SS-HH:MM`
/// - `YYYY-MM-DDTHH:MM:SS.sssZ` (with fractional seconds)
/// - `YYYY-MM-DDTHH:MM:SS` (no timezone — local time)
struct IsIsoDatetimePredicate;

impl NamedPredicate for IsIsoDatetimePredicate {
    fn name(&self) -> &str {
        "is_iso_datetime"
    }

    fn evaluate(&self, value: &Value, _args: &Value) -> bool {
        let s = match value.as_str() {
            Some(s) => s.trim(),
            None => return false,
        };

        // Must have a T separator
        let t_pos = match s.find('T') {
            Some(p) => p,
            None => return false,
        };

        // Date part
        let date_part = &s[..t_pos];
        if ParsedDate::parse(date_part).is_none() {
            // Try direct YYYY-MM-DD parse since ParsedDate::parse expects various formats
            let dparts: Vec<&str> = date_part.split('-').collect();
            if dparts.len() != 3 {
                return false;
            }
            let year: u32 = match dparts[0].parse() {
                Ok(y) => y,
                Err(_) => return false,
            };
            let month: u32 = match dparts[1].parse() {
                Ok(m) => m,
                Err(_) => return false,
            };
            let day: u32 = match dparts[2].parse() {
                Ok(d) => d,
                Err(_) => return false,
            };
            if ParsedDate::validate(year, month, day).is_none() {
                return false;
            }
        }

        // Time part (everything after T)
        let time_rest = &s[t_pos + 1..];

        // Separate timezone suffix
        let (time_part, tz_part) = if let Some(stripped) = time_rest.strip_suffix('Z') {
            (stripped, Some("Z"))
        } else if let Some(plus_pos) = time_rest.rfind('+') {
            // Make sure the + is in the timezone position (after time digits)
            if plus_pos >= 5 {
                (&time_rest[..plus_pos], Some(&time_rest[plus_pos..]))
            } else {
                (time_rest, None)
            }
        } else if let Some(minus_pos) = time_rest.rfind('-') {
            if minus_pos >= 5 {
                (&time_rest[..minus_pos], Some(&time_rest[minus_pos..]))
            } else {
                (time_rest, None)
            }
        } else {
            (time_rest, None)
        };

        // Parse time: HH:MM:SS or HH:MM:SS.sss
        let (time_hms, _frac) = if let Some(dot_pos) = time_part.find('.') {
            let frac = &time_part[dot_pos + 1..];
            if frac.is_empty() || !frac.chars().all(|c| c.is_ascii_digit()) {
                return false;
            }
            (&time_part[..dot_pos], Some(frac))
        } else {
            (time_part, None)
        };

        // Validate HH:MM:SS or HH:MM
        if !parse_time_24h(time_hms) {
            return false;
        }

        // Validate timezone offset if present
        if let Some(tz) = tz_part {
            if tz != "Z" {
                // +HH:MM or -HH:MM
                let tz_sign = &tz[..1];
                if tz_sign != "+" && tz_sign != "-" {
                    return false;
                }
                let tz_time = &tz[1..];
                let tz_parts: Vec<&str> = tz_time.split(':').collect();
                if tz_parts.len() != 2 {
                    return false;
                }
                let tz_hour: u32 = match tz_parts[0].parse() {
                    Ok(h) => h,
                    Err(_) => return false,
                };
                let tz_min: u32 = match tz_parts[1].parse() {
                    Ok(m) => m,
                    Err(_) => return false,
                };
                if tz_hour > 23 || tz_min > 59 {
                    return false;
                }
            }
        }

        true
    }
}

// ============================================================================
// Tax Year Predicate
// ============================================================================

/// Validate that a value is a valid tax year (2020-2100).
///
/// Accepts either a number or a string.
struct IsTaxYearPredicate;

impl NamedPredicate for IsTaxYearPredicate {
    fn name(&self) -> &str {
        "is_tax_year"
    }

    fn evaluate(&self, value: &Value, args: &Value) -> bool {
        let year = match value {
            Value::Number(n) => n.as_u64().map(|n| n as u32),
            Value::String(s) => s.trim().parse::<u32>().ok(),
            _ => return false,
        };

        let year = match year {
            Some(y) => y,
            None => return false,
        };

        let min_year = args.get("min").and_then(|v| v.as_u64()).unwrap_or(2020) as u32;

        let max_year = args.get("max").and_then(|v| v.as_u64()).unwrap_or(2100) as u32;

        year >= min_year && year <= max_year
    }
}

// ============================================================================
// Date In Range Predicate
// ============================================================================

/// Validate that a date falls within a specified range.
///
/// Args:
/// - min: Minimum date (inclusive), format matching input
/// - max: Maximum date (inclusive), format matching input
/// - min_year: Alternative - minimum year only
/// - max_year: Alternative - maximum year only
struct DateInRangePredicate;

impl NamedPredicate for DateInRangePredicate {
    fn name(&self) -> &str {
        "date_in_range"
    }

    fn evaluate(&self, value: &Value, args: &Value) -> bool {
        let s = match value.as_str() {
            Some(s) => s,
            None => return false,
        };

        let date = match ParsedDate::parse(s) {
            Some(d) => d,
            None => return false,
        };

        // Check year-only range first
        if let Some(min_year) = args.get("min_year").and_then(|v| v.as_u64()) {
            if date.year < min_year as u32 {
                return false;
            }
        }
        if let Some(max_year) = args.get("max_year").and_then(|v| v.as_u64()) {
            if date.year > max_year as u32 {
                return false;
            }
        }

        // Check full date range
        if let Some(min_str) = args.get("min").and_then(|v| v.as_str()) {
            if let Some(min_date) = ParsedDate::parse(min_str) {
                if date < min_date {
                    return false;
                }
            }
        }
        if let Some(max_str) = args.get("max").and_then(|v| v.as_str()) {
            if let Some(max_date) = ParsedDate::parse(max_str) {
                if date > max_date {
                    return false;
                }
            }
        }

        true
    }
}

// ============================================================================
// Date Before Predicate
// ============================================================================

/// Validate that the value date is before (or equal to) another date at a path.
///
/// Args:
/// - path: JSON Pointer path to the other date field
/// - allow_equal: If true (default), dates can be equal
///
/// Used for 1099-B: date acquired <= date sold
struct DateBeforePredicate;

impl NamedPredicate for DateBeforePredicate {
    fn name(&self) -> &str {
        "date_before"
    }

    fn evaluate(&self, value: &Value, args: &Value) -> bool {
        // This predicate compares the current value against a fixed date in args
        // For cross-field comparison, use the IR's Implies with path-based predicates

        let s = match value.as_str() {
            Some(s) => s,
            None => return false,
        };

        let date = match ParsedDate::parse(s) {
            Some(d) => d,
            None => return false,
        };

        let other_str = match args.get("date").and_then(|v| v.as_str()) {
            Some(s) => s,
            None => return true, // No comparison date specified, pass
        };

        let other_date = match ParsedDate::parse(other_str) {
            Some(d) => d,
            None => return true, // Can't parse comparison date, pass
        };

        let allow_equal = args
            .get("allow_equal")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        if allow_equal {
            date <= other_date
        } else {
            date < other_date
        }
    }
}

// ============================================================================
// Date After Predicate
// ============================================================================

/// Validate that the value date is after (or equal to) another date.
///
/// Args:
/// - date: The comparison date
/// - allow_equal: If true (default), dates can be equal
struct DateAfterPredicate;

impl NamedPredicate for DateAfterPredicate {
    fn name(&self) -> &str {
        "date_after"
    }

    fn evaluate(&self, value: &Value, args: &Value) -> bool {
        let s = match value.as_str() {
            Some(s) => s,
            None => return false,
        };

        let date = match ParsedDate::parse(s) {
            Some(d) => d,
            None => return false,
        };

        let other_str = match args.get("date").and_then(|v| v.as_str()) {
            Some(s) => s,
            None => return true,
        };

        let other_date = match ParsedDate::parse(other_str) {
            Some(d) => d,
            None => return true,
        };

        let allow_equal = args
            .get("allow_equal")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        if allow_equal {
            date >= other_date
        } else {
            date > other_date
        }
    }
}

// ============================================================================
// Time Parsing Helper (for comparisons)
// ============================================================================

/// Parsed time as total seconds from midnight, for comparison.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct ParsedTime {
    total_seconds: u32,
}

impl ParsedTime {
    /// Parse a time string (24h or 12h) into total seconds from midnight.
    fn parse(s: &str) -> Option<Self> {
        Self::from_24h(s).or_else(|| Self::from_12h(s))
    }

    fn from_24h(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() != 2 && parts.len() != 3 {
            return None;
        }
        if !parts
            .iter()
            .all(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit()))
        {
            return None;
        }
        if parts[0].len() != 2 || parts[1].len() != 2 {
            return None;
        }

        let hour: u32 = parts[0].parse().ok()?;
        let minute: u32 = parts[1].parse().ok()?;
        if hour > 23 || minute > 59 {
            return None;
        }

        let second = if parts.len() == 3 {
            if parts[2].len() != 2 {
                return None;
            }
            let s: u32 = parts[2].parse().ok()?;
            if s > 59 {
                return None;
            }
            s
        } else {
            0
        };

        Some(Self {
            total_seconds: hour * 3600 + minute * 60 + second,
        })
    }

    fn from_12h(s: &str) -> Option<Self> {
        let upper = s.to_uppercase();
        let (time_part, is_pm) = if upper.ends_with("AM") {
            (s[..s.len() - 2].trim(), false)
        } else if upper.ends_with("PM") {
            (s[..s.len() - 2].trim(), true)
        } else {
            return None;
        };

        let parts: Vec<&str> = time_part.split(':').collect();
        if parts.len() != 2 && parts.len() != 3 {
            return None;
        }
        if !parts
            .iter()
            .all(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit()))
        {
            return None;
        }

        let mut hour: u32 = parts[0].parse().ok()?;
        let minute: u32 = parts[1].parse().ok()?;
        if !(1..=12).contains(&hour) || minute > 59 {
            return None;
        }

        let second = if parts.len() == 3 {
            let s: u32 = parts[2].parse().ok()?;
            if s > 59 {
                return None;
            }
            s
        } else {
            0
        };

        // Convert to 24h
        if is_pm {
            if hour != 12 {
                hour += 12;
            }
        } else if hour == 12 {
            hour = 0;
        }

        Some(Self {
            total_seconds: hour * 3600 + minute * 60 + second,
        })
    }
}

// ============================================================================
// Time Before Predicate
// ============================================================================

/// Validate that a time is before a given time.
///
/// Args:
/// - time: The comparison time (24h or 12h format)
/// - allow_equal: If true (default), times can be equal
struct TimeBeforePredicate;

impl NamedPredicate for TimeBeforePredicate {
    fn name(&self) -> &str {
        "time_before"
    }

    fn evaluate(&self, value: &Value, args: &Value) -> bool {
        let s = match value.as_str() {
            Some(s) => s.trim(),
            None => return false,
        };

        let time = match ParsedTime::parse(s) {
            Some(t) => t,
            None => return false,
        };

        let other_str = match args.get("time").and_then(|v| v.as_str()) {
            Some(s) => s,
            None => return true,
        };

        let other = match ParsedTime::parse(other_str.trim()) {
            Some(t) => t,
            None => return true,
        };

        let allow_equal = args
            .get("allow_equal")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        if allow_equal {
            time <= other
        } else {
            time < other
        }
    }
}

// ============================================================================
// Time After Predicate
// ============================================================================

/// Validate that a time is after a given time.
///
/// Args:
/// - time: The comparison time (24h or 12h format)
/// - allow_equal: If true (default), times can be equal
struct TimeAfterPredicate;

impl NamedPredicate for TimeAfterPredicate {
    fn name(&self) -> &str {
        "time_after"
    }

    fn evaluate(&self, value: &Value, args: &Value) -> bool {
        let s = match value.as_str() {
            Some(s) => s.trim(),
            None => return false,
        };

        let time = match ParsedTime::parse(s) {
            Some(t) => t,
            None => return false,
        };

        let other_str = match args.get("time").and_then(|v| v.as_str()) {
            Some(s) => s,
            None => return true,
        };

        let other = match ParsedTime::parse(other_str.trim()) {
            Some(t) => t,
            None => return true,
        };

        let allow_equal = args
            .get("allow_equal")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        if allow_equal {
            time >= other
        } else {
            time > other
        }
    }
}

// ============================================================================
// Time In Range Predicate
// ============================================================================

/// Validate that a time falls within a range.
///
/// Args:
/// - min: Minimum time (inclusive), 24h or 12h format
/// - max: Maximum time (inclusive), 24h or 12h format
///
/// Supports overnight ranges: if min > max, the range wraps around midnight.
/// e.g., min=22:00, max=06:00 means "10 PM to 6 AM".
struct TimeInRangePredicate;

impl NamedPredicate for TimeInRangePredicate {
    fn name(&self) -> &str {
        "time_in_range"
    }

    fn evaluate(&self, value: &Value, args: &Value) -> bool {
        let s = match value.as_str() {
            Some(s) => s.trim(),
            None => return false,
        };

        let time = match ParsedTime::parse(s) {
            Some(t) => t,
            None => return false,
        };

        let min = args
            .get("min")
            .and_then(|v| v.as_str())
            .and_then(|s| ParsedTime::parse(s.trim()));
        let max = args
            .get("max")
            .and_then(|v| v.as_str())
            .and_then(|s| ParsedTime::parse(s.trim()));

        match (min, max) {
            (Some(min_t), Some(max_t)) => {
                if min_t <= max_t {
                    // Normal range: min <= time <= max
                    time >= min_t && time <= max_t
                } else {
                    // Overnight range: time >= min OR time <= max
                    time >= min_t || time <= max_t
                }
            }
            (Some(min_t), None) => time >= min_t,
            (None, Some(max_t)) => time <= max_t,
            (None, None) => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_date_us_format() {
        assert!(ParsedDate::parse("01/15/2024").is_some());
        assert!(ParsedDate::parse("12/31/2023").is_some());
        assert!(ParsedDate::parse("02/29/2024").is_some()); // Leap year
        assert!(ParsedDate::parse("02/29/2023").is_none()); // Not leap year
    }

    #[test]
    fn test_parse_date_iso_format() {
        assert!(ParsedDate::parse("2024-01-15").is_some());
        assert!(ParsedDate::parse("2023-12-31").is_some());
    }

    #[test]
    fn test_parse_date_no_separators() {
        assert!(ParsedDate::parse("01152024").is_some());
        assert!(ParsedDate::parse("12312023").is_some());
    }

    #[test]
    fn test_parse_date_invalid() {
        assert!(ParsedDate::parse("13/01/2024").is_none()); // Invalid month
        assert!(ParsedDate::parse("01/32/2024").is_none()); // Invalid day
        assert!(ParsedDate::parse("00/15/2024").is_none()); // Month 0
        assert!(ParsedDate::parse("01/00/2024").is_none()); // Day 0
        assert!(ParsedDate::parse("garbage").is_none());
    }

    #[test]
    fn test_is_date() {
        let pred = IsDatePredicate;

        assert!(pred.evaluate(&json!("01/15/2024"), &json!(null)));
        assert!(pred.evaluate(&json!("2024-01-15"), &json!(null)));
        assert!(pred.evaluate(&json!("01152024"), &json!(null)));

        assert!(!pred.evaluate(&json!("not a date"), &json!(null)));
        assert!(!pred.evaluate(&json!(12345), &json!(null)));
    }

    #[test]
    fn test_is_tax_year() {
        let pred = IsTaxYearPredicate;

        // Valid years
        assert!(pred.evaluate(&json!(2024), &json!(null)));
        assert!(pred.evaluate(&json!("2024"), &json!(null)));
        assert!(pred.evaluate(&json!(2020), &json!(null)));
        assert!(pred.evaluate(&json!(2100), &json!(null)));

        // Invalid years
        assert!(!pred.evaluate(&json!(2019), &json!(null)));
        assert!(!pred.evaluate(&json!(2101), &json!(null)));

        // Custom range
        assert!(pred.evaluate(&json!(2023), &json!({"min": 2023, "max": 2025})));
        assert!(!pred.evaluate(&json!(2022), &json!({"min": 2023, "max": 2025})));
    }

    #[test]
    fn test_date_in_range() {
        let pred = DateInRangePredicate;

        // Year-only range
        assert!(pred.evaluate(
            &json!("06/15/2024"),
            &json!({"min_year": 2020, "max_year": 2030})
        ));
        assert!(!pred.evaluate(
            &json!("06/15/2019"),
            &json!({"min_year": 2020, "max_year": 2030})
        ));

        // Full date range
        assert!(pred.evaluate(
            &json!("06/15/2024"),
            &json!({"min": "01/01/2024", "max": "12/31/2024"})
        ));
        assert!(!pred.evaluate(
            &json!("06/15/2023"),
            &json!({"min": "01/01/2024", "max": "12/31/2024"})
        ));
    }

    #[test]
    fn test_date_before() {
        let pred = DateBeforePredicate;

        // Date before
        assert!(pred.evaluate(&json!("01/01/2024"), &json!({"date": "12/31/2024"})));

        // Date equal (allowed by default)
        assert!(pred.evaluate(&json!("06/15/2024"), &json!({"date": "06/15/2024"})));

        // Date equal (not allowed)
        assert!(!pred.evaluate(
            &json!("06/15/2024"),
            &json!({"date": "06/15/2024", "allow_equal": false})
        ));

        // Date after (fail)
        assert!(!pred.evaluate(&json!("12/31/2024"), &json!({"date": "01/01/2024"})));
    }

    #[test]
    fn test_date_after() {
        let pred = DateAfterPredicate;

        // Date after
        assert!(pred.evaluate(&json!("12/31/2024"), &json!({"date": "01/01/2024"})));

        // Date equal (allowed by default)
        assert!(pred.evaluate(&json!("06/15/2024"), &json!({"date": "06/15/2024"})));

        // Date before (fail)
        assert!(!pred.evaluate(&json!("01/01/2024"), &json!({"date": "12/31/2024"})));
    }

    #[test]
    fn test_is_time() {
        let pred = IsTimePredicate;

        // Valid 24-hour times
        assert!(pred.evaluate(&json!("00:00"), &json!(null)));
        assert!(pred.evaluate(&json!("23:59"), &json!(null)));
        assert!(pred.evaluate(&json!("14:30"), &json!(null)));
        assert!(pred.evaluate(&json!("14:30:59"), &json!(null)));

        // Valid 12-hour times
        assert!(pred.evaluate(&json!("12:00 PM"), &json!(null)));
        assert!(pred.evaluate(&json!("1:30 AM"), &json!(null)));
        assert!(pred.evaluate(&json!("11:59:59 PM"), &json!(null)));
        assert!(pred.evaluate(&json!("12:00PM"), &json!(null)));

        // Format filter
        assert!(pred.evaluate(&json!("14:30"), &json!({"format": "24h"})));
        assert!(!pred.evaluate(&json!("2:30 PM"), &json!({"format": "24h"})));
        assert!(pred.evaluate(&json!("2:30 PM"), &json!({"format": "12h"})));
        assert!(!pred.evaluate(&json!("14:30"), &json!({"format": "12h"})));

        // Invalid
        assert!(!pred.evaluate(&json!("24:00"), &json!(null)));
        assert!(!pred.evaluate(&json!("12:60"), &json!(null)));
        assert!(!pred.evaluate(&json!("abc"), &json!(null)));
        assert!(!pred.evaluate(&json!("13:00 PM"), &json!(null))); // 13 not valid in 12h
        assert!(!pred.evaluate(&json!("0:00 AM"), &json!(null))); // 0 not valid in 12h

        // Non-string
        assert!(!pred.evaluate(&json!(1430), &json!(null)));
    }

    #[test]
    fn test_is_iso_datetime() {
        let pred = IsIsoDatetimePredicate;

        // Valid ISO 8601 datetimes
        assert!(pred.evaluate(&json!("2024-01-15T10:30:00Z"), &json!(null)));
        assert!(pred.evaluate(&json!("2024-01-15T10:30:00+05:30"), &json!(null)));
        assert!(pred.evaluate(&json!("2024-01-15T10:30:00-04:00"), &json!(null)));
        assert!(pred.evaluate(&json!("2024-01-15T10:30:00.123Z"), &json!(null)));
        assert!(pred.evaluate(&json!("2024-01-15T10:30:00"), &json!(null))); // No timezone
        assert!(pred.evaluate(&json!("2024-12-31T23:59:59Z"), &json!(null)));
        assert!(pred.evaluate(&json!("2024-01-15T10:30Z"), &json!(null))); // No seconds

        // Invalid
        assert!(!pred.evaluate(&json!("2024-01-15 10:30:00"), &json!(null))); // Space instead of T
        assert!(!pred.evaluate(&json!("2024-13-15T10:30:00Z"), &json!(null))); // Invalid month
        assert!(!pred.evaluate(&json!("2024-01-15T25:00:00Z"), &json!(null))); // Invalid hour
        assert!(!pred.evaluate(&json!("not-a-datetime"), &json!(null)));

        // Non-string
        assert!(!pred.evaluate(&json!(20240115), &json!(null)));
    }

    #[test]
    fn test_time_before() {
        let pred = TimeBeforePredicate;

        // Time before
        assert!(pred.evaluate(&json!("09:00"), &json!({"time": "12:00"})));
        assert!(pred.evaluate(&json!("08:30"), &json!({"time": "17:00"})));

        // Time equal (allowed by default)
        assert!(pred.evaluate(&json!("12:00"), &json!({"time": "12:00"})));

        // Time equal (not allowed)
        assert!(!pred.evaluate(
            &json!("12:00"),
            &json!({"time": "12:00", "allow_equal": false})
        ));

        // Time after (fail)
        assert!(!pred.evaluate(&json!("17:00"), &json!({"time": "12:00"})));

        // 12h format
        assert!(pred.evaluate(&json!("9:00 AM"), &json!({"time": "12:00"})));
        assert!(!pred.evaluate(&json!("1:00 PM"), &json!({"time": "12:00"})));

        // With seconds
        assert!(pred.evaluate(&json!("12:00:00"), &json!({"time": "12:00:01"})));
        assert!(!pred.evaluate(&json!("12:00:01"), &json!({"time": "12:00:00"})));

        // Non-string
        assert!(!pred.evaluate(&json!(1200), &json!({"time": "12:00"})));

        // No comparison time → pass
        assert!(pred.evaluate(&json!("12:00"), &json!(null)));
    }

    #[test]
    fn test_time_after() {
        let pred = TimeAfterPredicate;

        // Time after
        assert!(pred.evaluate(&json!("17:00"), &json!({"time": "12:00"})));
        assert!(pred.evaluate(&json!("14:30"), &json!({"time": "09:00"})));

        // Time equal (allowed by default)
        assert!(pred.evaluate(&json!("12:00"), &json!({"time": "12:00"})));

        // Time equal (not allowed)
        assert!(!pred.evaluate(
            &json!("12:00"),
            &json!({"time": "12:00", "allow_equal": false})
        ));

        // Time before (fail)
        assert!(!pred.evaluate(&json!("09:00"), &json!({"time": "12:00"})));

        // 12h format
        assert!(pred.evaluate(&json!("1:00 PM"), &json!({"time": "12:00"})));

        // Non-string
        assert!(!pred.evaluate(&json!(1700), &json!({"time": "12:00"})));
    }

    #[test]
    fn test_time_in_range() {
        let pred = TimeInRangePredicate;

        // Normal range: 09:00 - 17:00 (business hours)
        assert!(pred.evaluate(&json!("12:00"), &json!({"min": "09:00", "max": "17:00"})));
        assert!(pred.evaluate(&json!("09:00"), &json!({"min": "09:00", "max": "17:00"}))); // inclusive
        assert!(pred.evaluate(&json!("17:00"), &json!({"min": "09:00", "max": "17:00"}))); // inclusive
        assert!(!pred.evaluate(&json!("08:59"), &json!({"min": "09:00", "max": "17:00"})));
        assert!(!pred.evaluate(&json!("17:01"), &json!({"min": "09:00", "max": "17:00"})));

        // Overnight range: 22:00 - 06:00 (night shift)
        assert!(pred.evaluate(&json!("23:00"), &json!({"min": "22:00", "max": "06:00"})));
        assert!(pred.evaluate(&json!("02:00"), &json!({"min": "22:00", "max": "06:00"})));
        assert!(pred.evaluate(&json!("22:00"), &json!({"min": "22:00", "max": "06:00"}))); // inclusive
        assert!(pred.evaluate(&json!("06:00"), &json!({"min": "22:00", "max": "06:00"}))); // inclusive
        assert!(!pred.evaluate(&json!("12:00"), &json!({"min": "22:00", "max": "06:00"})));
        assert!(!pred.evaluate(&json!("21:59"), &json!({"min": "22:00", "max": "06:00"})));

        // Open-ended: min only
        assert!(pred.evaluate(&json!("14:00"), &json!({"min": "09:00"})));
        assert!(!pred.evaluate(&json!("08:00"), &json!({"min": "09:00"})));

        // Open-ended: max only
        assert!(pred.evaluate(&json!("08:00"), &json!({"max": "17:00"})));
        assert!(!pred.evaluate(&json!("18:00"), &json!({"max": "17:00"})));

        // 12h format input
        assert!(pred.evaluate(&json!("10:00 AM"), &json!({"min": "09:00", "max": "17:00"})));
        assert!(!pred.evaluate(&json!("8:00 AM"), &json!({"min": "09:00", "max": "17:00"})));

        // No args → pass
        assert!(pred.evaluate(&json!("12:00"), &json!(null)));

        // Non-string
        assert!(!pred.evaluate(&json!(1200), &json!({"min": "09:00", "max": "17:00"})));
    }
}

//! Vectorscan-based pattern matching for complex validation.
//!
//! This module provides high-performance regex matching using Vectorscan
//! (the open-source fork of Intel Hyperscan). Key features:
//!
//! - Match thousands of patterns simultaneously
//! - SIMD-accelerated on x86_64 (AVX2/AVX-512) and ARM (NEON)
//! - Stream-oriented for network-speed processing
//! - Zero-copy pattern matching
//!
//! ## Setup
//!
//! Vectorscan requires the system library to be installed:
//!
//! ```bash
//! # macOS
//! brew install vectorscan
//!
//! # Ubuntu/Debian
//! apt-get install libvectorscan-dev
//!
//! # From source
//! git clone https://github.com/VectorCamp/vectorscan
//! cd vectorscan && cmake -B build && cmake --build build
//! ```
//!
//! ## Usage
//!
//! ```rust
//! use wellformed_validate::patterns::{PatternDb, PatternId};
//!
//! // Create a pattern database with built-in patterns
//! let db = PatternDb::with_builtins().unwrap();
//!
//! // Match against input
//! let matches = db.scan("test@example.com");
//! assert!(matches.contains(PatternId::Email));
//! ```

use std::collections::HashSet;

/// Pattern identifier for matching results.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum PatternId {
    // TIN patterns
    SsnFormatted = 0,
    SsnUnformatted = 1,
    EinFormatted = 2,
    EinUnformatted = 3,

    // Contact patterns
    Email = 10,
    PhoneUs = 11,
    PhoneInternational = 12,
    Url = 13,

    // Address patterns
    ZipCode = 20,
    ZipPlus4 = 21,
    StateCode = 22,

    // Financial patterns
    Cusip = 30,
    AbaRouting = 31,
    AccountNumber = 32,

    // Date patterns
    DateMmDdYyyy = 40,
    DateYyyyMmDd = 41,
    DateIso8601 = 42,

    // Money patterns
    MoneyUsd = 50,
    MoneyDecimal = 51,
}

impl PatternId {
    /// Convert to u32 for Vectorscan.
    pub fn as_u32(self) -> u32 {
        self as u32
    }

    /// Try to convert from u32.
    pub fn from_u32(id: u32) -> Option<Self> {
        match id {
            0 => Some(Self::SsnFormatted),
            1 => Some(Self::SsnUnformatted),
            2 => Some(Self::EinFormatted),
            3 => Some(Self::EinUnformatted),
            10 => Some(Self::Email),
            11 => Some(Self::PhoneUs),
            12 => Some(Self::PhoneInternational),
            13 => Some(Self::Url),
            20 => Some(Self::ZipCode),
            21 => Some(Self::ZipPlus4),
            22 => Some(Self::StateCode),
            30 => Some(Self::Cusip),
            31 => Some(Self::AbaRouting),
            32 => Some(Self::AccountNumber),
            40 => Some(Self::DateMmDdYyyy),
            41 => Some(Self::DateYyyyMmDd),
            42 => Some(Self::DateIso8601),
            50 => Some(Self::MoneyUsd),
            51 => Some(Self::MoneyDecimal),
            _ => None,
        }
    }
}

/// Result of pattern matching.
#[derive(Debug, Default)]
pub struct MatchResult {
    /// Set of matched pattern IDs.
    pub matches: HashSet<PatternId>,
}

impl MatchResult {
    /// Create an empty result.
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if a pattern matched.
    pub fn contains(&self, id: PatternId) -> bool {
        self.matches.contains(&id)
    }

    /// Check if any pattern matched.
    pub fn has_matches(&self) -> bool {
        !self.matches.is_empty()
    }

    /// Get the number of matches.
    pub fn count(&self) -> usize {
        self.matches.len()
    }
}

/// Pre-compiled pattern definitions.
pub struct PatternDef {
    pub id: PatternId,
    pub pattern: &'static str,
    pub flags: u32,
}

/// All built-in patterns.
pub static BUILTIN_PATTERNS: &[PatternDef] = &[
    // SSN patterns
    PatternDef {
        id: PatternId::SsnFormatted,
        pattern: r"^\d{3}-\d{2}-\d{4}$",
        flags: 0,
    },
    PatternDef {
        id: PatternId::SsnUnformatted,
        pattern: r"^\d{9}$",
        flags: 0,
    },
    // EIN patterns
    PatternDef {
        id: PatternId::EinFormatted,
        pattern: r"^\d{2}-\d{7}$",
        flags: 0,
    },
    PatternDef {
        id: PatternId::EinUnformatted,
        pattern: r"^\d{9}$",
        flags: 0,
    },
    // Email
    PatternDef {
        id: PatternId::Email,
        pattern: r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$",
        flags: 0, // Case insensitive handled by pattern
    },
    // Phone
    PatternDef {
        id: PatternId::PhoneUs,
        pattern: r"^(\+1)?[-.\s]?\(?\d{3}\)?[-.\s]?\d{3}[-.\s]?\d{4}$",
        flags: 0,
    },
    PatternDef {
        id: PatternId::PhoneInternational,
        pattern: r"^\+\d{1,3}[-.\s]?\d{4,14}$",
        flags: 0,
    },
    // ZIP codes
    PatternDef {
        id: PatternId::ZipCode,
        pattern: r"^\d{5}$",
        flags: 0,
    },
    PatternDef {
        id: PatternId::ZipPlus4,
        pattern: r"^\d{5}-\d{4}$",
        flags: 0,
    },
    // Dates
    PatternDef {
        id: PatternId::DateMmDdYyyy,
        pattern: r"^(0[1-9]|1[0-2])/(0[1-9]|[12]\d|3[01])/\d{4}$",
        flags: 0,
    },
    PatternDef {
        id: PatternId::DateYyyyMmDd,
        pattern: r"^\d{4}-(0[1-9]|1[0-2])-(0[1-9]|[12]\d|3[01])$",
        flags: 0,
    },
    // Money
    PatternDef {
        id: PatternId::MoneyUsd,
        pattern: r"^\$?\d{1,3}(,\d{3})*(\.\d{2})?$",
        flags: 0,
    },
    PatternDef {
        id: PatternId::MoneyDecimal,
        pattern: r"^-?\d+(\.\d{1,2})?$",
        flags: 0,
    },
    // Financial identifiers
    PatternDef {
        id: PatternId::Cusip,
        pattern: r"^[A-Z0-9]{9}$",
        flags: 0,
    },
    PatternDef {
        id: PatternId::AbaRouting,
        pattern: r"^\d{9}$",
        flags: 0,
    },
];

// ============================================================================
// Vectorscan Implementation (feature-gated)
// ============================================================================

#[cfg(feature = "vectorscan")]
mod vectorscan_impl {
    use super::*;
    use vectorscan_rs::{BlockDatabase, BlockScanner, Error, Flag, Pattern, Scan};

    /// Pattern database backed by Vectorscan.
    ///
    /// Uses Intel/Vectorscan's SIMD-accelerated regex engine for
    /// matching thousands of patterns simultaneously.
    pub struct PatternDb {
        database: BlockDatabase,
    }

    impl PatternDb {
        /// Create a new pattern database with built-in patterns.
        pub fn with_builtins() -> Result<Self, Error> {
            let patterns: Vec<Pattern> = BUILTIN_PATTERNS
                .iter()
                .map(|def| {
                    Pattern::new(
                        def.pattern.as_bytes().to_vec(),
                        Flag::default(),
                        Some(def.id.as_u32()),
                    )
                })
                .collect();

            let database = BlockDatabase::new(patterns)?;

            Ok(Self { database })
        }

        /// Scan input for pattern matches.
        pub fn scan(&self, input: &str) -> MatchResult {
            let mut result = MatchResult::new();

            // Create a scanner for this scan operation
            let mut scanner = match BlockScanner::new(&self.database) {
                Ok(s) => s,
                Err(_) => return result,
            };

            let scan_result = scanner.scan(input.as_bytes(), |id, _from, _to, _flags| {
                if let Some(pattern_id) = PatternId::from_u32(id) {
                    result.matches.insert(pattern_id);
                }
                Scan::Continue
            });

            // Ignore errors - just return what we found
            let _ = scan_result;

            result
        }

        /// Scan multiple inputs in batch.
        pub fn scan_batch(&self, inputs: &[&str]) -> Vec<MatchResult> {
            inputs.iter().map(|s| self.scan(s)).collect()
        }
    }
}

#[cfg(feature = "vectorscan")]
pub use vectorscan_impl::PatternDb;

// ============================================================================
// Fallback Implementation (regex crate)
// ============================================================================

#[cfg(not(feature = "vectorscan"))]
mod fallback_impl {
    use super::*;
    use std::sync::LazyLock;

    /// Compiled regex patterns for fallback.
    struct CompiledPatterns {
        patterns: Vec<(PatternId, regex::Regex)>,
    }

    impl CompiledPatterns {
        fn new() -> Self {
            let patterns: Vec<_> = BUILTIN_PATTERNS
                .iter()
                .filter_map(|def| regex::Regex::new(def.pattern).ok().map(|r| (def.id, r)))
                .collect();

            Self { patterns }
        }
    }

    static COMPILED: LazyLock<CompiledPatterns> = LazyLock::new(CompiledPatterns::new);

    /// Pattern database using regex crate (fallback).
    pub struct PatternDb;

    impl PatternDb {
        /// Create a new pattern database.
        pub fn with_builtins() -> Result<Self, std::convert::Infallible> {
            // Force initialization
            let _ = &*COMPILED;
            Ok(Self)
        }

        /// Scan input for pattern matches.
        pub fn scan(&self, input: &str) -> MatchResult {
            let mut result = MatchResult::new();

            for (id, regex) in &COMPILED.patterns {
                if regex.is_match(input) {
                    result.matches.insert(*id);
                }
            }

            result
        }

        /// Scan multiple inputs in batch.
        pub fn scan_batch(&self, inputs: &[&str]) -> Vec<MatchResult> {
            inputs.iter().map(|s| self.scan(s)).collect()
        }
    }
}

#[cfg(not(feature = "vectorscan"))]
pub use fallback_impl::PatternDb;

// ============================================================================
// Convenience Functions
// ============================================================================

/// Check if a string matches the SSN format.
pub fn is_ssn_format(s: &str) -> bool {
    let bytes = s.as_bytes();

    // Formatted: XXX-XX-XXXX (11 chars)
    if bytes.len() == 11 {
        return bytes[3] == b'-'
            && bytes[6] == b'-'
            && bytes[..3].iter().all(|b| b.is_ascii_digit())
            && bytes[4..6].iter().all(|b| b.is_ascii_digit())
            && bytes[7..].iter().all(|b| b.is_ascii_digit());
    }

    // Unformatted: XXXXXXXXX (9 chars)
    if bytes.len() == 9 {
        return bytes.iter().all(|b| b.is_ascii_digit());
    }

    false
}

/// Check if a string matches the EIN format.
pub fn is_ein_format(s: &str) -> bool {
    let bytes = s.as_bytes();

    // Formatted: XX-XXXXXXX (10 chars)
    if bytes.len() == 10 {
        return bytes[2] == b'-'
            && bytes[..2].iter().all(|b| b.is_ascii_digit())
            && bytes[3..].iter().all(|b| b.is_ascii_digit());
    }

    // Unformatted: XXXXXXXXX (9 chars)
    if bytes.len() == 9 {
        return bytes.iter().all(|b| b.is_ascii_digit());
    }

    false
}

/// Check if a string matches the US ZIP code format.
pub fn is_zip_format(s: &str) -> bool {
    let bytes = s.as_bytes();

    // 5-digit: XXXXX
    if bytes.len() == 5 {
        return bytes.iter().all(|b| b.is_ascii_digit());
    }

    // ZIP+4: XXXXX-XXXX
    if bytes.len() == 10 {
        return bytes[5] == b'-'
            && bytes[..5].iter().all(|b| b.is_ascii_digit())
            && bytes[6..].iter().all(|b| b.is_ascii_digit());
    }

    false
}

/// Check if a string looks like an email address.
pub fn is_email_format(s: &str) -> bool {
    // Simple check: has @ with content before and after, domain has dot
    let at_pos = match s.find('@') {
        Some(pos) if pos > 0 && pos < s.len() - 1 => pos,
        _ => return false,
    };

    let domain = &s[at_pos + 1..];
    domain.contains('.') && !domain.starts_with('.') && !domain.ends_with('.')
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ssn_format() {
        assert!(is_ssn_format("123-45-6789"));
        assert!(is_ssn_format("123456789"));
        assert!(!is_ssn_format("12345678"));
        assert!(!is_ssn_format("1234567890"));
        assert!(!is_ssn_format("12-345-6789"));
    }

    #[test]
    fn test_ein_format() {
        assert!(is_ein_format("12-3456789"));
        assert!(is_ein_format("123456789"));
        assert!(!is_ein_format("123-456789"));
        assert!(!is_ein_format("12345678"));
    }

    #[test]
    fn test_zip_format() {
        assert!(is_zip_format("12345"));
        assert!(is_zip_format("12345-6789"));
        assert!(!is_zip_format("1234"));
        assert!(!is_zip_format("123456"));
        assert!(!is_zip_format("12345-678"));
    }

    #[test]
    fn test_email_format() {
        assert!(is_email_format("test@example.com"));
        assert!(is_email_format("a@b.c"));
        assert!(is_email_format("user.name+tag@example.org"));
        assert!(!is_email_format("invalid"));
        assert!(!is_email_format("@example.com"));
        assert!(!is_email_format("test@"));
        assert!(!is_email_format("test@.com"));
        assert!(!is_email_format("test@com."));
    }

    #[test]
    fn test_pattern_db() {
        let db = PatternDb::with_builtins().unwrap();

        let result = db.scan("123-45-6789");
        assert!(result.contains(PatternId::SsnFormatted));

        let result = db.scan("12345");
        assert!(result.contains(PatternId::ZipCode));

        let result = db.scan("test@example.com");
        assert!(result.contains(PatternId::Email));
    }

    #[test]
    fn test_batch_scan() {
        let db = PatternDb::with_builtins().unwrap();

        let inputs = vec!["123-45-6789", "test@example.com", "12345"];
        let results = db.scan_batch(&inputs);

        assert!(results[0].contains(PatternId::SsnFormatted));
        assert!(results[1].contains(PatternId::Email));
        assert!(results[2].contains(PatternId::ZipCode));
    }
}

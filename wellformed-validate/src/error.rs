//! Error types for validation.

use thiserror::Error;

/// Validation error types.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ValidationError {
    #[error("invalid TIN format: expected 9 digits")]
    InvalidTinFormat,

    #[error("invalid SSN: {0}")]
    InvalidSsn(SsnError),

    #[error("invalid EIN: {0}")]
    InvalidEin(EinError),

    #[error("invalid ITIN: must start with 9 and have 7 or 8 in position 4")]
    InvalidItin,

    #[error("pattern match failed: {0}")]
    PatternFailed(String),

    #[error("batch validation failed: {failed} of {total} items invalid")]
    BatchFailed { failed: usize, total: usize },
}

/// SSN-specific validation errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SsnError {
    /// Area number is 000
    AreaZero,
    /// Area number is 666
    Area666,
    /// Area number is 900-999 (reserved for ITINs)
    AreaReserved,
    /// Group number is 00
    GroupZero,
    /// Serial number is 0000
    SerialZero,
    /// All digits are the same
    AllSameDigits,
}

impl std::fmt::Display for SsnError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SsnError::AreaZero => write!(f, "area number cannot be 000"),
            SsnError::Area666 => write!(f, "area number cannot be 666"),
            SsnError::AreaReserved => write!(f, "area number 900-999 reserved for ITIN"),
            SsnError::GroupZero => write!(f, "group number cannot be 00"),
            SsnError::SerialZero => write!(f, "serial number cannot be 0000"),
            SsnError::AllSameDigits => write!(f, "all digits cannot be the same"),
        }
    }
}

/// EIN-specific validation errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EinError {
    /// Invalid campus/prefix code
    InvalidCampus,
    /// All zeros
    AllZeros,
}

impl std::fmt::Display for EinError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EinError::InvalidCampus => write!(f, "invalid IRS campus code"),
            EinError::AllZeros => write!(f, "EIN cannot be all zeros"),
        }
    }
}

/// Result type for validation operations.
pub type ValidationResult<T> = Result<T, ValidationError>;

/// Batch validation result with bit-packed valid flags.
#[derive(Debug, Clone)]
pub struct BatchResult {
    /// Bit vector of valid flags (1 = valid, 0 = invalid)
    valid_bits: Vec<u64>,
    /// Number of items validated
    count: usize,
    /// Number of valid items
    valid_count: usize,
}

impl BatchResult {
    /// Create a new batch result with given capacity.
    pub fn with_capacity(count: usize) -> Self {
        let num_words = count.div_ceil(64);
        Self {
            valid_bits: vec![0; num_words],
            count,
            valid_count: 0,
        }
    }

    /// Set the validation result for an index.
    #[inline]
    pub fn set(&mut self, index: usize, valid: bool) {
        let word = index / 64;
        let bit = index % 64;
        if valid {
            self.valid_bits[word] |= 1u64 << bit;
            self.valid_count += 1;
        }
    }

    /// Check if an index is valid.
    #[inline]
    pub fn is_valid(&self, index: usize) -> bool {
        let word = index / 64;
        let bit = index % 64;
        (self.valid_bits[word] >> bit) & 1 == 1
    }

    /// Get the number of valid items.
    #[inline]
    pub fn valid_count(&self) -> usize {
        self.valid_count
    }

    /// Get the number of invalid items.
    #[inline]
    pub fn invalid_count(&self) -> usize {
        self.count - self.valid_count
    }

    /// Check if all items are valid.
    #[inline]
    pub fn all_valid(&self) -> bool {
        self.valid_count == self.count
    }

    /// Iterate over indices of invalid items.
    pub fn invalid_indices(&self) -> impl Iterator<Item = usize> + '_ {
        (0..self.count).filter(|&i| !self.is_valid(i))
    }

    /// Iterate over indices of valid items.
    pub fn valid_indices(&self) -> impl Iterator<Item = usize> + '_ {
        (0..self.count).filter(|&i| self.is_valid(i))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_result() {
        let mut result = BatchResult::with_capacity(100);

        result.set(0, true);
        result.set(1, false);
        result.set(63, true);
        result.set(64, true);
        result.set(99, false);

        assert!(result.is_valid(0));
        assert!(!result.is_valid(1));
        assert!(result.is_valid(63));
        assert!(result.is_valid(64));
        assert!(!result.is_valid(99));

        assert_eq!(result.valid_count(), 3);
        assert_eq!(result.invalid_count(), 97);
    }
}

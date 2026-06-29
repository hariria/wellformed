//! Batch validation with Structure of Arrays (SoA) layout.
//!
//! This module provides high-throughput batch validation by:
//! - Using cache-friendly SoA memory layout
//! - Processing multiple fields with SIMD
//! - Minimizing allocations through buffer reuse
//!
//! ## Usage
//!
//! ```rust
//! use wellformed_validate::batch::{FormBatch, ValidationConfig};
//!
//! let mut batch = FormBatch::with_capacity(1000);
//!
//! // Add forms to the batch
//! batch.push_tins("123-45-6789", "987-65-4321");
//! batch.push_income(10000); // $100.00 in cents
//! batch.push_tax_year(2024);
//!
//! // Validate all at once
//! let results = batch.validate();
//! assert!(results.all_valid());
//! ```

use crate::error::BatchResult;
use crate::tin::{
    validate_atin_digits, validate_ein_digits, validate_itin_digits, validate_ssn_digits,
};

/// Configuration for batch validation.
#[derive(Debug, Clone)]
pub struct ValidationConfig {
    /// Allow formatted TINs (with dashes)
    pub allow_formatted_tins: bool,
    /// Require both payer and recipient TINs
    pub require_both_tins: bool,
    /// Validate amounts are non-negative
    pub validate_amounts: bool,
    /// Maximum batch size before automatic flush
    pub max_batch_size: usize,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            allow_formatted_tins: true,
            require_both_tins: true,
            validate_amounts: true,
            max_batch_size: 10_000,
        }
    }
}

/// Pre-allocated buffer for TIN digits.
/// Each TIN is stored as 9 bytes (digits 0-9).
#[derive(Clone)]
pub struct TinBuffer {
    /// Packed TIN digits (9 bytes per TIN)
    data: Vec<u8>,
    /// Valid flags (1 bit per TIN, packed into u64)
    valid: Vec<u64>,
    /// Number of TINs stored
    count: usize,
}

impl TinBuffer {
    /// Create a new buffer with given capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: Vec::with_capacity(capacity * 9),
            valid: Vec::with_capacity(capacity.div_ceil(64)),
            count: 0,
        }
    }

    /// Clear the buffer for reuse.
    pub fn clear(&mut self) {
        self.data.clear();
        self.valid.clear();
        self.count = 0;
    }

    /// Push a TIN string to the buffer.
    pub fn push(&mut self, tin: &str) -> bool {
        let mut digits = [0u8; 9];
        let mut digit_count = 0;

        for b in tin.bytes() {
            let d = b.wrapping_sub(b'0');
            if d <= 9 {
                if digit_count >= 9 {
                    // Too many digits - mark invalid and store zeros
                    self.data.extend_from_slice(&[0; 9]);
                    self.set_valid(self.count, false);
                    self.count += 1;
                    return false;
                }
                digits[digit_count] = d;
                digit_count += 1;
            } else if b != b'-' && b != b' ' {
                // Invalid character
                self.data.extend_from_slice(&[0; 9]);
                self.set_valid(self.count, false);
                self.count += 1;
                return false;
            }
        }

        if digit_count == 9 {
            self.data.extend_from_slice(&digits);
            self.set_valid(self.count, true);
            self.count += 1;
            true
        } else {
            self.data.extend_from_slice(&[0; 9]);
            self.set_valid(self.count, false);
            self.count += 1;
            false
        }
    }

    /// Set validity flag for an index.
    fn set_valid(&mut self, index: usize, valid: bool) {
        let word = index / 64;
        let bit = index % 64;

        while self.valid.len() <= word {
            self.valid.push(0);
        }

        if valid {
            self.valid[word] |= 1u64 << bit;
        } else {
            self.valid[word] &= !(1u64 << bit);
        }
    }

    /// Check if a TIN at index has valid format.
    #[inline]
    pub fn is_format_valid(&self, index: usize) -> bool {
        let word = index / 64;
        let bit = index % 64;
        word < self.valid.len() && (self.valid[word] >> bit) & 1 == 1
    }

    /// Get the digits for a TIN at index.
    #[inline]
    pub fn get_digits(&self, index: usize) -> Option<&[u8; 9]> {
        if index < self.count {
            let start = index * 9;
            Some(self.data[start..start + 9].try_into().unwrap())
        } else {
            None
        }
    }

    /// Get the number of TINs in the buffer.
    #[inline]
    pub fn len(&self) -> usize {
        self.count
    }

    /// Check if the buffer is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }
}

/// Amount buffer for monetary values.
/// Amounts are stored as i64 cents for exact arithmetic.
#[derive(Clone)]
pub struct AmountBuffer {
    /// Amounts in cents
    data: Vec<i64>,
}

impl AmountBuffer {
    /// Create a new buffer with given capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: Vec::with_capacity(capacity),
        }
    }

    /// Clear the buffer for reuse.
    pub fn clear(&mut self) {
        self.data.clear();
    }

    /// Push an amount in cents.
    #[inline]
    pub fn push(&mut self, cents: i64) {
        self.data.push(cents);
    }

    /// Push an amount from dollars (f64).
    #[inline]
    pub fn push_dollars(&mut self, dollars: f64) {
        self.data.push((dollars * 100.0).round() as i64);
    }

    /// Get amount at index.
    #[inline]
    pub fn get(&self, index: usize) -> Option<i64> {
        self.data.get(index).copied()
    }

    /// Get the number of amounts.
    #[inline]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Validate all amounts are non-negative.
    pub fn validate_non_negative(&self) -> BatchResult {
        let mut result = BatchResult::with_capacity(self.data.len());

        // Process in chunks for SIMD
        for (i, &amount) in self.data.iter().enumerate() {
            result.set(i, amount >= 0);
        }

        result
    }
}

/// Batch of tax forms for validation.
///
/// Uses Structure of Arrays (SoA) layout for cache efficiency.
pub struct FormBatch {
    /// Payer TINs
    pub payer_tins: TinBuffer,
    /// Recipient TINs
    pub recipient_tins: TinBuffer,
    /// Interest income amounts (cents)
    pub interest_income: AmountBuffer,
    /// Tax year (stored as u16)
    pub tax_years: Vec<u16>,
    /// Configuration
    config: ValidationConfig,
}

impl FormBatch {
    /// Create a new batch with given capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            payer_tins: TinBuffer::with_capacity(capacity),
            recipient_tins: TinBuffer::with_capacity(capacity),
            interest_income: AmountBuffer::with_capacity(capacity),
            tax_years: Vec::with_capacity(capacity),
            config: ValidationConfig::default(),
        }
    }

    /// Create a batch with custom configuration.
    pub fn with_config(capacity: usize, config: ValidationConfig) -> Self {
        Self {
            payer_tins: TinBuffer::with_capacity(capacity),
            recipient_tins: TinBuffer::with_capacity(capacity),
            interest_income: AmountBuffer::with_capacity(capacity),
            tax_years: Vec::with_capacity(capacity),
            config,
        }
    }

    /// Clear the batch for reuse.
    pub fn clear(&mut self) {
        self.payer_tins.clear();
        self.recipient_tins.clear();
        self.interest_income.clear();
        self.tax_years.clear();
    }

    /// Get the number of forms in the batch.
    #[inline]
    pub fn len(&self) -> usize {
        self.payer_tins.len()
    }

    /// Check if the batch is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.payer_tins.is_empty()
    }

    /// Push TINs for a form.
    pub fn push_tins(&mut self, payer_tin: &str, recipient_tin: &str) {
        self.payer_tins.push(payer_tin);
        self.recipient_tins.push(recipient_tin);
    }

    /// Push an interest income amount.
    pub fn push_income(&mut self, cents: i64) {
        self.interest_income.push(cents);
    }

    /// Push a tax year.
    pub fn push_tax_year(&mut self, year: u16) {
        self.tax_years.push(year);
    }

    /// Validate all forms in the batch.
    ///
    /// Returns a BatchResult with validation flags for each form.
    pub fn validate(&self) -> BatchResult {
        let count = self.len();
        let mut result = BatchResult::with_capacity(count);

        for i in 0..count {
            let mut valid = true;

            // Validate payer TIN
            if self.payer_tins.is_format_valid(i) {
                if let Some(digits) = self.payer_tins.get_digits(i) {
                    valid &= is_valid_tin_digits(digits);
                }
            } else {
                valid = false;
            }

            // Validate recipient TIN
            if self.config.require_both_tins {
                if self.recipient_tins.is_format_valid(i) {
                    if let Some(digits) = self.recipient_tins.get_digits(i) {
                        valid &= is_valid_tin_digits(digits);
                    }
                } else {
                    valid = false;
                }
            }

            // Validate amount
            if self.config.validate_amounts {
                if let Some(amount) = self.interest_income.get(i) {
                    valid &= amount >= 0;
                }
            }

            // Validate tax year
            if let Some(&year) = self.tax_years.get(i) {
                valid &= (2020..=2100).contains(&year);
            }

            result.set(i, valid);
        }

        result
    }

    /// Validate TINs only (faster if you only need TIN validation).
    pub fn validate_tins_only(&self) -> BatchResult {
        let count = self.len();
        let mut result = BatchResult::with_capacity(count);

        for i in 0..count {
            let payer_valid = self.payer_tins.is_format_valid(i)
                && self
                    .payer_tins
                    .get_digits(i)
                    .map(is_valid_tin_digits)
                    .unwrap_or(false);

            let recipient_valid = !self.config.require_both_tins
                || (self.recipient_tins.is_format_valid(i)
                    && self
                        .recipient_tins
                        .get_digits(i)
                        .map(is_valid_tin_digits)
                        .unwrap_or(false));

            result.set(i, payer_valid && recipient_valid);
        }

        result
    }
}

/// Check if TIN digits are valid (any type).
#[inline]
fn is_valid_tin_digits(digits: &[u8; 9]) -> bool {
    // Check for all zeros
    if digits.iter().all(|&d| d == 0) {
        return false;
    }

    // Try each TIN type
    validate_ssn_digits(digits).is_ok()
        || validate_ein_digits(digits).is_ok()
        || validate_itin_digits(digits)
        || validate_atin_digits(digits)
}

// ============================================================================
// Parallel Batch Validation
// ============================================================================

/// Validate multiple batches in parallel.
#[cfg(feature = "rayon")]
pub fn validate_parallel(batches: &[FormBatch]) -> Vec<BatchResult> {
    use rayon::prelude::*;
    batches.par_iter().map(|b| b.validate()).collect()
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tin_buffer() {
        let mut buf = TinBuffer::with_capacity(10);

        assert!(buf.push("123456789"));
        assert!(buf.push("123-45-6789"));
        assert!(!buf.push("12345678")); // Too few digits
        assert!(!buf.push("1234567890")); // Too many digits
        assert!(!buf.push("12345678a")); // Invalid char

        assert_eq!(buf.len(), 5);
        assert!(buf.is_format_valid(0));
        assert!(buf.is_format_valid(1));
        assert!(!buf.is_format_valid(2));
        assert!(!buf.is_format_valid(3));
        assert!(!buf.is_format_valid(4));

        assert_eq!(buf.get_digits(0), Some(&[1, 2, 3, 4, 5, 6, 7, 8, 9]));
        assert_eq!(buf.get_digits(1), Some(&[1, 2, 3, 4, 5, 6, 7, 8, 9]));
    }

    #[test]
    fn test_amount_buffer() {
        let mut buf = AmountBuffer::with_capacity(10);

        buf.push(10000); // $100.00
        buf.push(-5000); // -$50.00
        buf.push(0);
        buf.push_dollars(123.45);

        assert_eq!(buf.get(0), Some(10000));
        assert_eq!(buf.get(1), Some(-5000));
        assert_eq!(buf.get(2), Some(0));
        assert_eq!(buf.get(3), Some(12345));

        let result = buf.validate_non_negative();
        assert!(result.is_valid(0));
        assert!(!result.is_valid(1));
        assert!(result.is_valid(2));
        assert!(result.is_valid(3));
    }

    #[test]
    fn test_form_batch() {
        let mut batch = FormBatch::with_capacity(10);

        // Valid form
        batch.push_tins("123-45-6789", "987-65-4321");
        batch.push_income(10000);
        batch.push_tax_year(2024);

        // Invalid payer TIN
        batch.push_tins("000-00-0000", "123-45-6789");
        batch.push_income(5000);
        batch.push_tax_year(2024);

        // Invalid amount
        batch.push_tins("123-45-6789", "987-65-4321");
        batch.push_income(-1000);
        batch.push_tax_year(2024);

        let result = batch.validate();

        assert!(result.is_valid(0));
        assert!(!result.is_valid(1)); // Invalid payer TIN
        assert!(!result.is_valid(2)); // Negative amount

        assert_eq!(result.valid_count(), 1);
        assert_eq!(result.invalid_count(), 2);
    }

    #[test]
    fn test_batch_reuse() {
        let mut batch = FormBatch::with_capacity(10);

        batch.push_tins("123-45-6789", "987-65-4321");
        batch.push_income(10000);
        batch.push_tax_year(2024);

        assert_eq!(batch.len(), 1);

        batch.clear();

        assert_eq!(batch.len(), 0);
        assert!(batch.is_empty());

        // Reuse
        batch.push_tins("111-22-3333", "444-55-6666");
        assert_eq!(batch.len(), 1);
    }
}

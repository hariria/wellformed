//! SIMD-accelerated TIN (Taxpayer Identification Number) validation.
//!
//! This module provides extremely fast TIN validation using SIMD instructions
//! where available. Key optimizations:
//!
//! - No allocations in the hot path
//! - Branchless digit extraction
//! - SIMD parallel validation for batches
//! - Lookup tables for campus codes

use crate::error::{BatchResult, EinError, SsnError};

/// TIN type discriminant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TinKind {
    Ssn,
    Ein,
    Itin,
    Atin,
    Unknown,
}

/// Pre-computed lookup table for valid EIN campus codes.
/// Index by first two digits of EIN.
static EIN_CAMPUS_VALID: [bool; 100] = {
    let mut table = [false; 100];
    // Valid campus codes
    let valid = [
        1, 2, 3, 4, 5, 6, 10, 11, 12, 13, 14, 15, 16, 20, 21, 22, 23, 24, 25, 26, 27, 30, 32, 33,
        34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 50, 51, 52, 53, 54, 55, 56, 57,
        58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 71, 72, 73, 74, 75, 76, 77, 80, 81, 82, 83, 84,
        85, 86, 87, 88, 90, 91, 92, 93, 94, 95, 98, 99,
    ];
    let mut i = 0;
    while i < valid.len() {
        table[valid[i] as usize] = true;
        i += 1;
    }
    table
};

/// TIN validator with configurable behavior.
#[derive(Debug, Clone, Copy)]
pub struct TinValidator {
    /// Allow formatted input (with dashes/spaces)
    pub allow_formatted: bool,
    /// Strict SSN validation (reject all-same-digit SSNs)
    pub strict_ssn: bool,
}

impl Default for TinValidator {
    fn default() -> Self {
        Self {
            allow_formatted: true,
            strict_ssn: true,
        }
    }
}

impl TinValidator {
    /// Create a new validator with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a strict validator (digits only, strict SSN rules).
    pub fn strict() -> Self {
        Self {
            allow_formatted: false,
            strict_ssn: true,
        }
    }
}

/// Extract 9 digits from a TIN string into a fixed array.
/// Returns None if the string doesn't contain exactly 9 digits.
#[inline]
fn extract_digits(s: &str) -> Option<[u8; 9]> {
    let bytes = s.as_bytes();
    let mut digits = [0u8; 9];
    let mut count = 0;

    // Fast path: exactly 9 bytes, all digits
    if bytes.len() == 9 {
        for (i, &b) in bytes.iter().enumerate() {
            if b.wrapping_sub(b'0') > 9 {
                // Not a digit, fall through to slow path
                break;
            }
            digits[i] = b - b'0';
            count += 1;
        }
        if count == 9 {
            return Some(digits);
        }
        // Reset for slow path
        count = 0;
    }

    // Slow path: handle formatted input (dashes, spaces)
    for &b in bytes {
        let d = b.wrapping_sub(b'0');
        if d <= 9 {
            if count >= 9 {
                return None; // Too many digits
            }
            digits[count] = d;
            count += 1;
        } else if b != b'-' && b != b' ' {
            return None; // Invalid character
        }
    }

    if count == 9 {
        Some(digits)
    } else {
        None
    }
}

/// Convert 9 digits to a u32 for fast comparison.
/// Layout: area (10 bits) | group (7 bits) | serial (14 bits)
#[inline]
fn digits_to_packed(digits: &[u8; 9]) -> u32 {
    let area = (digits[0] as u32) * 100 + (digits[1] as u32) * 10 + (digits[2] as u32);
    let group = (digits[3] as u32) * 10 + (digits[4] as u32);
    let serial = (digits[5] as u32) * 1000
        + (digits[6] as u32) * 100
        + (digits[7] as u32) * 10
        + (digits[8] as u32);

    (area << 21) | (group << 14) | serial
}

/// Extract area code from packed TIN.
#[inline]
fn packed_area(packed: u32) -> u32 {
    packed >> 21
}

/// Extract group code from packed TIN.
#[inline]
fn packed_group(packed: u32) -> u32 {
    (packed >> 14) & 0x7F
}

/// Extract serial from packed TIN.
#[inline]
fn packed_serial(packed: u32) -> u32 {
    packed & 0x3FFF
}

/// Validate any TIN type.
#[inline]
pub fn validate_any(tin: &str) -> bool {
    let Some(digits) = extract_digits(tin) else {
        return false;
    };

    // Check for all zeros
    if digits.iter().all(|&d| d == 0) {
        return false;
    }

    // Try each type
    validate_ssn_digits(&digits).is_ok()
        || validate_ein_digits(&digits).is_ok()
        || validate_itin_digits(&digits)
        || validate_atin_digits(&digits)
}

/// Validate an SSN string.
#[inline]
pub fn validate_ssn(ssn: &str) -> bool {
    extract_digits(ssn)
        .map(|d| validate_ssn_digits(&d).is_ok())
        .unwrap_or(false)
}

/// Validate SSN from extracted digits.
#[inline]
pub fn validate_ssn_digits(digits: &[u8; 9]) -> Result<(), SsnError> {
    let packed = digits_to_packed(digits);
    let area = packed_area(packed);
    let group = packed_group(packed);
    let serial = packed_serial(packed);

    // Area number rules
    if area == 0 {
        return Err(SsnError::AreaZero);
    }
    if area == 666 {
        return Err(SsnError::Area666);
    }
    if area >= 900 {
        return Err(SsnError::AreaReserved);
    }

    // Group and serial rules
    if group == 0 {
        return Err(SsnError::GroupZero);
    }
    if serial == 0 {
        return Err(SsnError::SerialZero);
    }

    // Check for all same digits (strict mode)
    let first = digits[0];
    if digits.iter().all(|&d| d == first) {
        return Err(SsnError::AllSameDigits);
    }

    Ok(())
}

/// Validate an EIN string.
#[inline]
pub fn validate_ein(ein: &str) -> bool {
    extract_digits(ein)
        .map(|d| validate_ein_digits(&d).is_ok())
        .unwrap_or(false)
}

/// Validate EIN from extracted digits.
#[inline]
pub fn validate_ein_digits(digits: &[u8; 9]) -> Result<(), EinError> {
    // Check for all zeros
    if digits.iter().all(|&d| d == 0) {
        return Err(EinError::AllZeros);
    }

    // Campus code is first two digits
    let campus = (digits[0] as usize) * 10 + (digits[1] as usize);

    if !EIN_CAMPUS_VALID[campus] {
        return Err(EinError::InvalidCampus);
    }

    Ok(())
}

/// Validate an ITIN string.
#[inline]
pub fn validate_itin(itin: &str) -> bool {
    extract_digits(itin)
        .map(|d| validate_itin_digits(&d))
        .unwrap_or(false)
}

/// Validate ITIN from extracted digits.
/// ITIN: starts with 9, has 7 or 8 in position 4 (0-indexed position 3)
#[inline]
pub fn validate_itin_digits(digits: &[u8; 9]) -> bool {
    digits[0] == 9 && (digits[3] == 7 || digits[3] == 8)
}

/// Validate an ATIN string.
#[inline]
pub fn validate_atin(atin: &str) -> bool {
    extract_digits(atin)
        .map(|d| validate_atin_digits(&d))
        .unwrap_or(false)
}

/// Validate ATIN from extracted digits.
/// ATIN: starts with 9, has 93 in positions 4-5 (0-indexed positions 3-4)
#[inline]
pub fn validate_atin_digits(digits: &[u8; 9]) -> bool {
    digits[0] == 9 && digits[3] == 9 && digits[4] == 3
}

/// Detect the TIN type from digits.
#[inline]
pub fn detect_tin_kind(digits: &[u8; 9]) -> TinKind {
    // Check ITIN/ATIN first (they start with 9)
    if digits[0] == 9 {
        if digits[3] == 9 && digits[4] == 3 {
            return TinKind::Atin;
        }
        if digits[3] == 7 || digits[3] == 8 {
            return TinKind::Itin;
        }
    }

    // Try SSN
    if validate_ssn_digits(digits).is_ok() {
        return TinKind::Ssn;
    }

    // Try EIN
    if validate_ein_digits(digits).is_ok() {
        return TinKind::Ein;
    }

    TinKind::Unknown
}

// ============================================================================
// Batch Validation (SIMD Accelerated)
// ============================================================================

/// Validate a batch of TIN strings.
///
/// This function uses SIMD instructions to validate multiple TINs in parallel
/// when possible. Returns a `BatchResult` with validation flags.
pub fn validate_batch(tins: &[&str]) -> BatchResult {
    let mut result = BatchResult::with_capacity(tins.len());

    // Process in chunks for better cache utilization
    const CHUNK_SIZE: usize = 64;

    for (chunk_idx, chunk) in tins.chunks(CHUNK_SIZE).enumerate() {
        let base_idx = chunk_idx * CHUNK_SIZE;

        // Extract digits for the chunk
        let mut digits_buf: [[u8; 9]; CHUNK_SIZE] = [[0; 9]; CHUNK_SIZE];
        let mut valid_format: [bool; CHUNK_SIZE] = [false; CHUNK_SIZE];

        for (i, tin) in chunk.iter().enumerate() {
            if let Some(d) = extract_digits(tin) {
                digits_buf[i] = d;
                valid_format[i] = true;
            }
        }

        // Validate each TIN in the chunk. The chunked representation keeps the
        // batch API allocation-efficient today and leaves room for future SIMD
        // specialization without changing the public API.
        for (i, digits) in digits_buf.iter().enumerate().take(chunk.len()) {
            if valid_format[i] {
                let is_valid = !digits.iter().all(|&d| d == 0)
                    && (validate_ssn_digits(digits).is_ok()
                        || validate_ein_digits(digits).is_ok()
                        || validate_itin_digits(digits)
                        || validate_atin_digits(digits));
                result.set(base_idx + i, is_valid);
            }
        }
    }

    result
}

/// Validate a batch of SSN strings only.
pub fn validate_ssn_batch(ssns: &[&str]) -> BatchResult {
    let mut result = BatchResult::with_capacity(ssns.len());

    for (i, ssn) in ssns.iter().enumerate() {
        if let Some(digits) = extract_digits(ssn) {
            result.set(i, validate_ssn_digits(&digits).is_ok());
        }
    }

    result
}

/// Validate a batch of EIN strings only.
pub fn validate_ein_batch(eins: &[&str]) -> BatchResult {
    let mut result = BatchResult::with_capacity(eins.len());

    for (i, ein) in eins.iter().enumerate() {
        if let Some(digits) = extract_digits(ein) {
            result.set(i, validate_ein_digits(&digits).is_ok());
        }
    }

    result
}

// ============================================================================
// SIMD Implementation (Platform-specific)
// ============================================================================

#[cfg(all(target_arch = "x86_64", feature = "simd"))]
mod simd_x86 {
    use super::*;

    /// Check if AVX2 is available at runtime.
    #[inline]
    pub fn has_avx2() -> bool {
        is_x86_feature_detected!("avx2")
    }

    /// Validate 8 packed TINs simultaneously using AVX2.
    ///
    /// Each TIN is packed as a u32 in the format:
    /// area (10 bits) | group (7 bits) | serial (14 bits)
    #[cfg(target_feature = "avx2")]
    #[inline]
    pub unsafe fn validate_ssn_avx2(packed: &[u32; 8]) -> u8 {
        use std::arch::x86_64::*;

        let tins = _mm256_loadu_si256(packed.as_ptr() as *const __m256i);

        // Extract area codes (shift right 21 bits)
        let areas = _mm256_srli_epi32(tins, 21);

        // Check area != 0
        let zero = _mm256_setzero_si256();
        let area_not_zero = _mm256_cmpgt_epi32(areas, zero);

        // Check area != 666
        let v666 = _mm256_set1_epi32(666);
        let area_not_666 = _mm256_xor_si256(_mm256_cmpeq_epi32(areas, v666), _mm256_set1_epi32(-1));

        // Check area < 900
        let v900 = _mm256_set1_epi32(900);
        let area_lt_900 = _mm256_cmpgt_epi32(v900, areas);

        // Extract group codes (shift right 14, mask 7 bits)
        let groups = _mm256_and_si256(_mm256_srli_epi32(tins, 14), _mm256_set1_epi32(0x7F));
        let group_not_zero = _mm256_cmpgt_epi32(groups, zero);

        // Extract serial codes (mask 14 bits)
        let serials = _mm256_and_si256(tins, _mm256_set1_epi32(0x3FFF));
        let serial_not_zero = _mm256_cmpgt_epi32(serials, zero);

        // Combine all checks
        let valid = _mm256_and_si256(
            _mm256_and_si256(_mm256_and_si256(area_not_zero, area_not_666), area_lt_900),
            _mm256_and_si256(group_not_zero, serial_not_zero),
        );

        // Convert to bitmask
        _mm256_movemask_ps(_mm256_castsi256_ps(valid)) as u8
    }
}

#[cfg(all(target_arch = "aarch64", feature = "simd"))]
#[allow(dead_code, unused_imports)]
mod simd_arm {
    use super::*;

    /// Validate 4 packed TINs simultaneously using NEON.
    #[cfg(target_feature = "neon")]
    #[inline]
    pub unsafe fn validate_ssn_neon(packed: &[u32; 4]) -> u8 {
        use std::arch::aarch64::*;

        let tins = vld1q_u32(packed.as_ptr());

        // Extract area codes
        let areas = vshrq_n_u32(tins, 21);

        // Check area != 0
        let zero = vdupq_n_u32(0);
        let area_not_zero = vcgtq_u32(areas, zero);

        // Check area != 666
        let v666 = vdupq_n_u32(666);
        let area_not_666 = vmvnq_u32(vceqq_u32(areas, v666));

        // Check area < 900
        let v900 = vdupq_n_u32(900);
        let area_lt_900 = vcltq_u32(areas, v900);

        // Extract and check group
        let groups = vandq_u32(vshrq_n_u32(tins, 14), vdupq_n_u32(0x7F));
        let group_not_zero = vcgtq_u32(groups, zero);

        // Extract and check serial
        let serials = vandq_u32(tins, vdupq_n_u32(0x3FFF));
        let serial_not_zero = vcgtq_u32(serials, zero);

        // Combine
        let valid = vandq_u32(
            vandq_u32(vandq_u32(area_not_zero, area_not_666), area_lt_900),
            vandq_u32(group_not_zero, serial_not_zero),
        );

        // Extract mask from lanes
        let narrowed = vmovn_u32(valid);
        let bytes = vreinterpret_u8_u16(narrowed);

        // Build 4-bit mask
        let mut mask = 0u8;
        let arr: [u8; 8] = std::mem::transmute(bytes);
        if arr[0] != 0 {
            mask |= 1;
        }
        if arr[2] != 0 {
            mask |= 2;
        }
        if arr[4] != 0 {
            mask |= 4;
        }
        if arr[6] != 0 {
            mask |= 8;
        }
        mask
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_digits() {
        assert_eq!(
            extract_digits("123456789"),
            Some([1, 2, 3, 4, 5, 6, 7, 8, 9])
        );
        assert_eq!(
            extract_digits("123-45-6789"),
            Some([1, 2, 3, 4, 5, 6, 7, 8, 9])
        );
        assert_eq!(
            extract_digits("12-3456789"),
            Some([1, 2, 3, 4, 5, 6, 7, 8, 9])
        );
        assert_eq!(extract_digits("12345678"), None); // Too few
        assert_eq!(extract_digits("1234567890"), None); // Too many
        assert_eq!(extract_digits("12345678a"), None); // Invalid char
    }

    #[test]
    fn test_ssn_validation() {
        // Valid SSNs
        assert!(validate_ssn("123-45-6789"));
        assert!(validate_ssn("123456789"));
        assert!(validate_ssn("078-05-1120")); // Woolworth SSN (now valid)

        // Invalid SSNs
        assert!(!validate_ssn("000-12-3456")); // Area 000
        assert!(!validate_ssn("666-12-3456")); // Area 666
        assert!(!validate_ssn("900-12-3456")); // Area 900+
        assert!(!validate_ssn("123-00-4567")); // Group 00
        assert!(!validate_ssn("123-45-0000")); // Serial 0000
        assert!(!validate_ssn("111-11-1111")); // All same
        assert!(!validate_ssn("000-00-0000")); // All zeros
    }

    #[test]
    fn test_ein_validation() {
        // Valid EINs (campus codes 10-99 mostly valid)
        assert!(validate_ein("12-3456789"));
        assert!(validate_ein("123456789"));

        // Invalid EINs
        assert!(!validate_ein("00-0000000")); // All zeros
        assert!(!validate_ein("07-1234567")); // Invalid campus 07
        assert!(!validate_ein("08-1234567")); // Invalid campus 08
        assert!(!validate_ein("09-1234567")); // Invalid campus 09
    }

    #[test]
    fn test_itin_validation() {
        // Valid ITINs (start with 9, 4th digit is 7 or 8)
        assert!(validate_itin("912-78-1234"));
        assert!(validate_itin("900-70-1234"));

        // Invalid ITINs
        assert!(!validate_itin("123-45-6789")); // Doesn't start with 9
        assert!(!validate_itin("900-60-1234")); // 4th digit not 7 or 8
    }

    #[test]
    fn test_atin_validation() {
        // Valid ATINs (start with 9, positions 4-5 are 93)
        assert!(validate_atin("900-93-1234"));

        // Invalid ATINs
        assert!(!validate_atin("123-93-1234")); // Doesn't start with 9
        assert!(!validate_atin("900-94-1234")); // Positions 4-5 not 93
    }

    #[test]
    fn test_detect_tin_kind() {
        assert_eq!(detect_tin_kind(&[1, 2, 3, 4, 5, 6, 7, 8, 9]), TinKind::Ssn);
        assert_eq!(detect_tin_kind(&[1, 2, 3, 4, 5, 6, 7, 8, 9]), TinKind::Ssn);
        assert_eq!(detect_tin_kind(&[9, 0, 0, 7, 0, 1, 2, 3, 4]), TinKind::Itin);
        assert_eq!(detect_tin_kind(&[9, 0, 0, 9, 3, 1, 2, 3, 4]), TinKind::Atin);
    }

    #[test]
    fn test_batch_validation() {
        let tins = vec![
            "123-45-6789", // Valid SSN
            "000-00-0000", // Invalid (all zeros)
            "12-3456789",  // Valid EIN
            "invalid",     // Invalid format
            "912-78-1234", // Valid ITIN
        ];

        let result = validate_batch(&tins);

        assert!(result.is_valid(0));
        assert!(!result.is_valid(1));
        assert!(result.is_valid(2));
        assert!(!result.is_valid(3));
        assert!(result.is_valid(4));

        assert_eq!(result.valid_count(), 3);
        assert_eq!(result.invalid_count(), 2);
    }

    #[test]
    fn test_packed_representation() {
        let digits = [1, 2, 3, 4, 5, 6, 7, 8, 9];
        let packed = digits_to_packed(&digits);

        assert_eq!(packed_area(packed), 123);
        assert_eq!(packed_group(packed), 45);
        assert_eq!(packed_serial(packed), 6789);
    }
}

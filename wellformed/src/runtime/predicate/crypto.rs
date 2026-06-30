//! Encoding and cryptocurrency address validation predicates.
//!
//! Validates base58, base64, and crypto addresses (Bitcoin, Ethereum, Solana).

use super::registry::{NamedPredicate, PredicateRegistry};
use serde_json::Value;
use std::sync::Arc;

/// Register all encoding/crypto predicates.
pub fn register_crypto_predicates(registry: &mut PredicateRegistry) {
    registry.register(Arc::new(IsBase58Predicate));
    registry.register(Arc::new(IsBase64Predicate));
    registry.register(Arc::new(IsBitcoinAddressPredicate));
    registry.register(Arc::new(IsEthereumAddressPredicate));
    registry.register(Arc::new(IsSolanaAddressPredicate));
    registry.register(Arc::new(IsJwtPredicate));
    registry.register(Arc::new(IsHashPredicate));
}

// ============================================================================
// Base58
// ============================================================================

/// Base58 alphabet (Bitcoin variant): no 0, O, I, l.
const BASE58_CHARS: &str = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";

fn is_base58_str(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| BASE58_CHARS.contains(c))
}

/// Validate a base58-encoded string (Bitcoin alphabet).
struct IsBase58Predicate;

impl NamedPredicate for IsBase58Predicate {
    fn name(&self) -> &str {
        "is_base58"
    }

    fn evaluate(&self, value: &Value, _args: &Value) -> bool {
        let s = match value.as_str() {
            Some(s) => s.trim(),
            None => return false,
        };
        is_base58_str(s)
    }
}

// ============================================================================
// Base64
// ============================================================================

/// Validate a base64-encoded string.
///
/// Optional args:
/// - `url_safe` (bool): Accept URL-safe base64 (- and _ instead of + and /). Default: false.
struct IsBase64Predicate;

impl NamedPredicate for IsBase64Predicate {
    fn name(&self) -> &str {
        "is_base64"
    }

    fn evaluate(&self, value: &Value, args: &Value) -> bool {
        let s = match value.as_str() {
            Some(s) => s.trim(),
            None => return false,
        };

        if s.is_empty() {
            return false;
        }

        let url_safe = args
            .get("url_safe")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        // Strip padding
        let without_pad = s.trim_end_matches('=');

        // Check all chars are valid
        let valid = if url_safe {
            without_pad
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        } else {
            without_pad
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '/')
        };

        if !valid {
            return false;
        }

        // Check padding is correct: length (including padding) must be multiple of 4
        // and at most 2 padding chars
        let pad_count = s.len() - without_pad.len();
        if pad_count > 2 {
            return false;
        }
        if pad_count > 0 && s.len() % 4 != 0 {
            return false;
        }

        true
    }
}

// ============================================================================
// Bitcoin Address
// ============================================================================

/// Bech32 character set (lowercase, no 1, b, i, o).
const BECH32_CHARS: &str = "023456789acdefghjklmnpqrstuvwxyz";

/// Validate a Bitcoin address.
///
/// Supports:
/// - P2PKH: starts with `1`, 25-34 chars, base58
/// - P2SH: starts with `3`, 25-34 chars, base58
/// - Bech32 (SegWit): starts with `bc1q`, 42 chars
/// - Bech32m (Taproot): starts with `bc1p`, 62 chars
struct IsBitcoinAddressPredicate;

impl NamedPredicate for IsBitcoinAddressPredicate {
    fn name(&self) -> &str {
        "is_bitcoin_address"
    }

    fn evaluate(&self, value: &Value, _args: &Value) -> bool {
        let s = match value.as_str() {
            Some(s) => s.trim(),
            None => return false,
        };

        if s.is_empty() {
            return false;
        }

        // P2PKH (starts with 1) or P2SH (starts with 3)
        if s.starts_with('1') || s.starts_with('3') {
            return (25..=34).contains(&s.len()) && is_base58_str(s);
        }

        // Bech32/Bech32m (starts with bc1)
        if let Some(rest) = s.to_lowercase().strip_prefix("bc1") {
            // bc1q... (SegWit v0) = 42 chars total, bc1p... (Taproot) = 62 chars total
            if rest.starts_with('q') && s.len() == 42 {
                return rest.chars().all(|c| BECH32_CHARS.contains(c) || c == 'q');
            }
            if rest.starts_with('p') && s.len() == 62 {
                return rest.chars().all(|c| BECH32_CHARS.contains(c) || c == 'p');
            }
            return false;
        }

        false
    }
}

// ============================================================================
// Ethereum Address
// ============================================================================

/// Validate an Ethereum address.
///
/// Format: `0x` followed by 40 hex characters.
/// Accepts mixed case (EIP-55 checksummed) or all lowercase/uppercase.
struct IsEthereumAddressPredicate;

impl NamedPredicate for IsEthereumAddressPredicate {
    fn name(&self) -> &str {
        "is_ethereum_address"
    }

    fn evaluate(&self, value: &Value, _args: &Value) -> bool {
        let s = match value.as_str() {
            Some(s) => s.trim(),
            None => return false,
        };

        // Must start with 0x
        let hex = match s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
            Some(h) => h,
            None => return false,
        };

        // Must be exactly 40 hex chars
        hex.len() == 40 && hex.chars().all(|c| c.is_ascii_hexdigit())
    }
}

// ============================================================================
// Solana Address
// ============================================================================

/// Validate a Solana address.
///
/// Base58-encoded, 32-44 characters.
struct IsSolanaAddressPredicate;

impl NamedPredicate for IsSolanaAddressPredicate {
    fn name(&self) -> &str {
        "is_solana_address"
    }

    fn evaluate(&self, value: &Value, _args: &Value) -> bool {
        let s = match value.as_str() {
            Some(s) => s.trim(),
            None => return false,
        };

        (32..=44).contains(&s.len()) && is_base58_str(s)
    }
}

// ============================================================================
// JWT
// ============================================================================

/// Validate a compact JWT format (header.payload.signature).
///
/// All three segments must be non-empty base64url strings.
struct IsJwtPredicate;

impl NamedPredicate for IsJwtPredicate {
    fn name(&self) -> &str {
        "is_jwt"
    }

    fn evaluate(&self, value: &Value, _args: &Value) -> bool {
        let s = match value.as_str() {
            Some(s) => s.trim(),
            None => return false,
        };

        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() != 3 || parts.iter().any(|p| p.is_empty()) {
            return false;
        }

        parts.iter().all(|part| {
            part.chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        })
    }
}

// ============================================================================
// Hash
// ============================================================================

/// Validate a hexadecimal digest hash.
///
/// Optional args:
/// - `algorithm`: one of `md5`, `sha1`, `sha224`, `sha256`, `sha384`, `sha512`
struct IsHashPredicate;

impl NamedPredicate for IsHashPredicate {
    fn name(&self) -> &str {
        "is_hash"
    }

    fn evaluate(&self, value: &Value, args: &Value) -> bool {
        let s = match value.as_str() {
            Some(s) => s.trim(),
            None => return false,
        };

        if !s.chars().all(|c| c.is_ascii_hexdigit()) {
            return false;
        }

        let expected_len = match args
            .get("algorithm")
            .and_then(|v| v.as_str())
            .map(|v| v.to_lowercase())
            .as_deref()
        {
            Some("md5") => Some(32),
            Some("sha1") => Some(40),
            Some("sha224") => Some(56),
            Some("sha256") => Some(64),
            Some("sha384") => Some(96),
            Some("sha512") => Some(128),
            Some(_) => return false,
            None => None,
        };

        if let Some(expected) = expected_len {
            s.len() == expected
        } else {
            matches!(s.len(), 32 | 40 | 56 | 64 | 96 | 128)
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn eval(pred: &dyn NamedPredicate, value: &str, args: Value) -> bool {
        pred.evaluate(&json!(value), &args)
    }

    // --- Base58 ---

    #[test]
    fn test_base58_valid() {
        let p = IsBase58Predicate;
        assert!(eval(
            &p,
            "5HueCGU8rMjxEXxiPuD5BDku4MkFqeZyd4dZ1jvhTVqvbTLvyTJ",
            json!({})
        ));
        assert!(eval(
            &p,
            "ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnpqrstuvwxyz123456789",
            json!({})
        ));
    }

    #[test]
    fn test_base58_invalid() {
        let p = IsBase58Predicate;
        // Contains 0
        assert!(!eval(&p, "0ABC", json!({})));
        // Contains O
        assert!(!eval(&p, "OABC", json!({})));
        // Contains I
        assert!(!eval(&p, "IABC", json!({})));
        // Contains l
        assert!(!eval(&p, "lABC", json!({})));
        // Empty
        assert!(!eval(&p, "", json!({})));
    }

    // --- Base64 ---

    #[test]
    fn test_base64_valid() {
        let p = IsBase64Predicate;
        assert!(eval(&p, "SGVsbG8gV29ybGQ=", json!({})));
        assert!(eval(&p, "dGVzdA==", json!({})));
        assert!(eval(&p, "AQID", json!({})));
        // No padding needed for multiple of 3 input bytes
        assert!(eval(&p, "AQIDBA", json!({})));
    }

    #[test]
    fn test_base64_url_safe() {
        let p = IsBase64Predicate;
        assert!(eval(&p, "abc-def_ghi", json!({"url_safe": true})));
        // Standard base64 chars invalid in url-safe mode
        assert!(!eval(&p, "abc+def/ghi", json!({"url_safe": true})));
    }

    #[test]
    fn test_base64_invalid() {
        let p = IsBase64Predicate;
        assert!(!eval(&p, "", json!({})));
        // Invalid chars
        assert!(!eval(&p, "Hello World!", json!({})));
        // Too much padding
        assert!(!eval(&p, "A===", json!({})));
    }

    // --- Bitcoin Address ---

    #[test]
    fn test_bitcoin_p2pkh() {
        let p = IsBitcoinAddressPredicate;
        assert!(eval(&p, "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa", json!({})));
        assert!(eval(&p, "1BvBMSEYstWetqTFn5Au4m4GFg7xJaNVN2", json!({})));
    }

    #[test]
    fn test_bitcoin_p2sh() {
        let p = IsBitcoinAddressPredicate;
        assert!(eval(&p, "3J98t1WpEZ73CNmQviecrnyiWrnqRhWNLy", json!({})));
    }

    #[test]
    fn test_bitcoin_bech32() {
        let p = IsBitcoinAddressPredicate;
        // SegWit (bc1q...)
        assert!(eval(
            &p,
            "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4",
            json!({})
        ));
        // Taproot (bc1p...)
        assert!(eval(
            &p,
            "bc1p5d7rjq7g6rdk2yhzks9smlaqtedr4dekq08ge8ztwac72sfr9rusxg3s7a",
            json!({})
        ));
    }

    #[test]
    fn test_bitcoin_invalid() {
        let p = IsBitcoinAddressPredicate;
        // Too short
        assert!(!eval(&p, "1A1z", json!({})));
        // Contains invalid base58 char
        assert!(!eval(&p, "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfOa", json!({})));
        // Not a bitcoin address
        assert!(!eval(
            &p,
            "0x742d35Cc6634C0532925a3b844Bc9e7595f2bD18",
            json!({})
        ));
        // Empty
        assert!(!eval(&p, "", json!({})));
    }

    // --- Ethereum Address ---

    #[test]
    fn test_ethereum_valid() {
        let p = IsEthereumAddressPredicate;
        assert!(eval(
            &p,
            "0x742d35Cc6634C0532925a3b844Bc9e7595f2bD18",
            json!({})
        ));
        assert!(eval(
            &p,
            "0x0000000000000000000000000000000000000000",
            json!({})
        ));
        // All lowercase
        assert!(eval(
            &p,
            "0xde0b295669a9fd93d5f28d9ec85e40f4cb697bae",
            json!({})
        ));
        // All uppercase hex
        assert!(eval(
            &p,
            "0xDE0B295669A9FD93D5F28D9EC85E40F4CB697BAE",
            json!({})
        ));
    }

    #[test]
    fn test_ethereum_invalid() {
        let p = IsEthereumAddressPredicate;
        // Missing 0x
        assert!(!eval(
            &p,
            "742d35Cc6634C0532925a3b844Bc9e7595f2bD18",
            json!({})
        ));
        // Too short
        assert!(!eval(&p, "0x742d35Cc", json!({})));
        // Non-hex chars
        assert!(!eval(
            &p,
            "0xZZZd35Cc6634C0532925a3b844Bc9e7595f2bD18",
            json!({})
        ));
    }

    // --- Solana Address ---

    #[test]
    fn test_solana_valid() {
        let p = IsSolanaAddressPredicate;
        assert!(eval(
            &p,
            "7EcDhSYGxXyscszYEp35KHN8vvw3svAuLKTzXwCFLtV",
            json!({})
        ));
        assert!(eval(
            &p,
            "DRpbCBMxVnDK7maPGv5ZoMSGn3BbiTSx2CLEQE8VBUQP",
            json!({})
        ));
    }

    #[test]
    fn test_solana_invalid() {
        let p = IsSolanaAddressPredicate;
        // Too short
        assert!(!eval(&p, "7EcDhSYGxXyscszYEp35KHN8vvw", json!({})));
        // Contains invalid base58 chars
        assert!(!eval(
            &p,
            "0EcDhSYGxXyscszYEp35KHN8vvw3svAuLKTzXwCFLtV",
            json!({})
        ));
        // Ethereum-style address
        assert!(!eval(
            &p,
            "0x742d35Cc6634C0532925a3b844Bc9e7595f2bD18",
            json!({})
        ));
    }

    #[test]
    fn test_jwt_valid() {
        let p = IsJwtPredicate;
        assert!(eval(
            &p,
            "eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxMjMifQ.dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk",
            json!({})
        ));
    }

    #[test]
    fn test_jwt_invalid() {
        let p = IsJwtPredicate;
        assert!(!eval(&p, "abc.def", json!({})));
        assert!(!eval(&p, "abc.def.ghi.jkl", json!({})));
        assert!(!eval(&p, "abc.def.g$%", json!({})));
    }

    #[test]
    fn test_hash_valid() {
        let p = IsHashPredicate;
        assert!(eval(
            &p,
            "d41d8cd98f00b204e9800998ecf8427e",
            json!({"algorithm":"md5"})
        ));
        assert!(eval(
            &p,
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824",
            json!({"algorithm":"sha256"})
        ));
        assert!(eval(
            &p,
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824",
            json!({})
        ));
    }

    #[test]
    fn test_hash_invalid() {
        let p = IsHashPredicate;
        assert!(!eval(
            &p,
            "2cf24dba5fb0a30e26e83b2ac5b9e29e",
            json!({"algorithm":"sha256"})
        ));
        assert!(!eval(&p, "not-a-hash", json!({})));
        assert!(!eval(
            &p,
            "d41d8cd98f00b204e9800998ecf8427e",
            json!({"algorithm":"unknown"})
        ));
    }
}

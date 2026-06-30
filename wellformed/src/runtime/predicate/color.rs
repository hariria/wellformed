//! Color validation predicates.
//!
//! Validates hex colors, RGB, and HSL color strings.

use super::registry::{NamedPredicate, PredicateRegistry};
use serde_json::Value;
use std::sync::Arc;

/// Register all color predicates.
pub fn register_color_predicates(registry: &mut PredicateRegistry) {
    registry.register(Arc::new(IsHexColorPredicate));
    registry.register(Arc::new(IsRgbColorPredicate));
    registry.register(Arc::new(IsHslColorPredicate));
}

// ============================================================================
// Hex Color
// ============================================================================

/// Validate a hex color string.
///
/// Accepts:
/// - `#RGB` (3-digit shorthand)
/// - `#RRGGBB` (6-digit)
/// - `#RGBA` (4-digit shorthand with alpha)
/// - `#RRGGBBAA` (8-digit with alpha)
///
/// Optional args:
/// - `allow_alpha` (bool): If false, reject 4- and 8-digit forms. Default: true.
/// - `require_hash` (bool): If true, require leading `#`. Default: true.
struct IsHexColorPredicate;

impl NamedPredicate for IsHexColorPredicate {
    fn name(&self) -> &str {
        "is_hex_color"
    }

    fn evaluate(&self, value: &Value, args: &Value) -> bool {
        let s = match value.as_str() {
            Some(s) => s.trim(),
            None => return false,
        };

        let allow_alpha = args
            .get("allow_alpha")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let require_hash = args
            .get("require_hash")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let hex = if let Some(rest) = s.strip_prefix('#') {
            rest
        } else if require_hash {
            return false;
        } else {
            s
        };

        // Check length
        let valid_len = if allow_alpha {
            matches!(hex.len(), 3 | 4 | 6 | 8)
        } else {
            matches!(hex.len(), 3 | 6)
        };

        if !valid_len {
            return false;
        }

        // All chars must be hex digits
        hex.chars().all(|c| c.is_ascii_hexdigit())
    }
}

// ============================================================================
// RGB Color
// ============================================================================

/// Validate an RGB/RGBA color string.
///
/// Accepts:
/// - `rgb(R, G, B)` - comma-separated, values 0-255
/// - `rgb(R G B)` - space-separated (modern syntax)
/// - `rgba(R, G, B, A)` - with alpha 0-1 or 0%-100%
/// - `rgb(R G B / A)` - modern syntax with alpha
/// - Percentage values: `rgb(100%, 0%, 50%)`
struct IsRgbColorPredicate;

impl NamedPredicate for IsRgbColorPredicate {
    fn name(&self) -> &str {
        "is_rgb_color"
    }

    fn evaluate(&self, value: &Value, _args: &Value) -> bool {
        let s = match value.as_str() {
            Some(s) => s.trim(),
            None => return false,
        };

        // Must start with rgb( or rgba(
        let (inner, has_alpha_prefix) = if let Some(rest) = s.strip_prefix("rgba(") {
            (rest, true)
        } else if let Some(rest) = s.strip_prefix("rgb(") {
            (rest, false)
        } else {
            return false;
        };

        // Must end with )
        let inner = match inner.strip_suffix(')') {
            Some(i) => i.trim(),
            None => return false,
        };

        // Try to parse with slash separator for alpha: "R G B / A"
        if let Some((rgb_part, alpha_part)) = inner.split_once('/') {
            let parts: Vec<&str> = rgb_part.split_whitespace().collect();
            if parts.len() != 3 {
                return false;
            }
            let all_percent = parts.iter().all(|p| p.ends_with('%'));
            let all_number = parts.iter().all(|p| !p.ends_with('%'));
            if !all_percent && !all_number {
                return false;
            }
            for p in &parts {
                if !validate_rgb_component(p) {
                    return false;
                }
            }
            return validate_alpha_component(alpha_part.trim());
        }

        // Try comma-separated: "R, G, B" or "R, G, B, A"
        if inner.contains(',') {
            let parts: Vec<&str> = inner.split(',').map(|s| s.trim()).collect();
            match parts.len() {
                3 => {
                    let all_percent = parts.iter().all(|p| p.ends_with('%'));
                    let all_number = parts.iter().all(|p| !p.ends_with('%'));
                    if !all_percent && !all_number {
                        return false;
                    }
                    parts.iter().all(|p| validate_rgb_component(p))
                }
                4 => {
                    let rgb = &parts[..3];
                    let all_percent = rgb.iter().all(|p| p.ends_with('%'));
                    let all_number = rgb.iter().all(|p| !p.ends_with('%'));
                    if !all_percent && !all_number {
                        return false;
                    }
                    rgb.iter().all(|p| validate_rgb_component(p))
                        && validate_alpha_component(parts[3])
                }
                _ => false,
            }
        } else {
            // Space-separated: "R G B"
            let parts: Vec<&str> = inner.split_whitespace().collect();
            if parts.len() != 3 && !(has_alpha_prefix && parts.len() == 4) {
                return false;
            }
            let rgb = &parts[..3];
            let all_percent = rgb.iter().all(|p| p.ends_with('%'));
            let all_number = rgb.iter().all(|p| !p.ends_with('%'));
            if !all_percent && !all_number {
                return false;
            }
            let rgb_valid = rgb.iter().all(|p| validate_rgb_component(p));
            if parts.len() == 4 {
                rgb_valid && validate_alpha_component(parts[3])
            } else {
                rgb_valid
            }
        }
    }
}

/// Validate a single RGB component (0-255 or 0%-100%).
fn validate_rgb_component(s: &str) -> bool {
    if let Some(pct) = s.strip_suffix('%') {
        match pct.trim().parse::<f64>() {
            Ok(v) => (0.0..=100.0).contains(&v),
            Err(_) => false,
        }
    } else {
        match s.trim().parse::<f64>() {
            Ok(v) => (0.0..=255.0).contains(&v) && v.fract() == 0.0,
            Err(_) => false,
        }
    }
}

/// Validate an alpha component (0-1 or 0%-100%).
fn validate_alpha_component(s: &str) -> bool {
    if let Some(pct) = s.strip_suffix('%') {
        match pct.trim().parse::<f64>() {
            Ok(v) => (0.0..=100.0).contains(&v),
            Err(_) => false,
        }
    } else {
        match s.trim().parse::<f64>() {
            Ok(v) => (0.0..=1.0).contains(&v),
            Err(_) => false,
        }
    }
}

// ============================================================================
// HSL Color
// ============================================================================

/// Validate an HSL/HSLA color string.
///
/// Accepts:
/// - `hsl(H, S%, L%)` - comma-separated, hue 0-360, saturation/lightness 0%-100%
/// - `hsl(H S% L%)` - space-separated (modern syntax)
/// - `hsla(H, S%, L%, A)` - with alpha 0-1 or 0%-100%
/// - `hsl(H S% L% / A)` - modern syntax with alpha
struct IsHslColorPredicate;

impl NamedPredicate for IsHslColorPredicate {
    fn name(&self) -> &str {
        "is_hsl_color"
    }

    fn evaluate(&self, value: &Value, _args: &Value) -> bool {
        let s = match value.as_str() {
            Some(s) => s.trim(),
            None => return false,
        };

        // Must start with hsl( or hsla(
        let (inner, has_alpha_prefix) = if let Some(rest) = s.strip_prefix("hsla(") {
            (rest, true)
        } else if let Some(rest) = s.strip_prefix("hsl(") {
            (rest, false)
        } else {
            return false;
        };

        // Must end with )
        let inner = match inner.strip_suffix(')') {
            Some(i) => i.trim(),
            None => return false,
        };

        // Try to parse with slash separator for alpha: "H S% L% / A"
        if let Some((hsl_part, alpha_part)) = inner.split_once('/') {
            let parts: Vec<&str> = hsl_part.split_whitespace().collect();
            if parts.len() != 3 {
                return false;
            }
            return validate_hue(parts[0])
                && validate_percentage_component(parts[1])
                && validate_percentage_component(parts[2])
                && validate_alpha_component(alpha_part.trim());
        }

        // Try comma-separated: "H, S%, L%" or "H, S%, L%, A"
        if inner.contains(',') {
            let parts: Vec<&str> = inner.split(',').map(|s| s.trim()).collect();
            match parts.len() {
                3 => {
                    validate_hue(parts[0])
                        && validate_percentage_component(parts[1])
                        && validate_percentage_component(parts[2])
                }
                4 => {
                    validate_hue(parts[0])
                        && validate_percentage_component(parts[1])
                        && validate_percentage_component(parts[2])
                        && validate_alpha_component(parts[3])
                }
                _ => false,
            }
        } else {
            // Space-separated: "H S% L%"
            let parts: Vec<&str> = inner.split_whitespace().collect();
            if parts.len() != 3 && !(has_alpha_prefix && parts.len() == 4) {
                return false;
            }
            let hsl_valid = validate_hue(parts[0])
                && validate_percentage_component(parts[1])
                && validate_percentage_component(parts[2]);
            if parts.len() == 4 {
                hsl_valid && validate_alpha_component(parts[3])
            } else {
                hsl_valid
            }
        }
    }
}

/// Validate hue value (0-360, degrees).
fn validate_hue(s: &str) -> bool {
    // Strip optional "deg" suffix
    let s = s.strip_suffix("deg").unwrap_or(s);
    match s.trim().parse::<f64>() {
        Ok(v) => (0.0..=360.0).contains(&v),
        Err(_) => false,
    }
}

/// Validate a percentage component (must end with %, value 0-100).
fn validate_percentage_component(s: &str) -> bool {
    if let Some(pct) = s.strip_suffix('%') {
        match pct.trim().parse::<f64>() {
            Ok(v) => (0.0..=100.0).contains(&v),
            Err(_) => false,
        }
    } else {
        false
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

    // --- Hex Color ---

    #[test]
    fn test_hex_color_basic() {
        let p = IsHexColorPredicate;
        // 3-digit
        assert!(eval(&p, "#fff", json!({})));
        assert!(eval(&p, "#F0A", json!({})));
        // 6-digit
        assert!(eval(&p, "#FF00AA", json!({})));
        assert!(eval(&p, "#ff00aa", json!({})));
        assert!(eval(&p, "#000000", json!({})));
        // 4-digit (alpha)
        assert!(eval(&p, "#F0AF", json!({})));
        // 8-digit (alpha)
        assert!(eval(&p, "#FF00AAFF", json!({})));
    }

    #[test]
    fn test_hex_color_no_alpha() {
        let p = IsHexColorPredicate;
        assert!(eval(&p, "#FFF", json!({"allow_alpha": false})));
        assert!(eval(&p, "#FF00AA", json!({"allow_alpha": false})));
        assert!(!eval(&p, "#FFFF", json!({"allow_alpha": false})));
        assert!(!eval(&p, "#FF00AAFF", json!({"allow_alpha": false})));
    }

    #[test]
    fn test_hex_color_no_hash() {
        let p = IsHexColorPredicate;
        assert!(!eval(&p, "FF00AA", json!({})));
        assert!(eval(&p, "FF00AA", json!({"require_hash": false})));
        assert!(eval(&p, "#FF00AA", json!({"require_hash": false})));
    }

    #[test]
    fn test_hex_color_invalid() {
        let p = IsHexColorPredicate;
        assert!(!eval(&p, "#GGG", json!({})));
        assert!(!eval(&p, "#12345", json!({})));
        assert!(!eval(&p, "red", json!({})));
        assert!(!eval(&p, "", json!({})));
        assert!(!eval(&p, "#", json!({})));
    }

    // --- RGB Color ---

    #[test]
    fn test_rgb_comma_separated() {
        let p = IsRgbColorPredicate;
        assert!(eval(&p, "rgb(255, 0, 170)", json!({})));
        assert!(eval(&p, "rgb(0, 0, 0)", json!({})));
        assert!(eval(&p, "rgb(255, 255, 255)", json!({})));
    }

    #[test]
    fn test_rgb_space_separated() {
        let p = IsRgbColorPredicate;
        assert!(eval(&p, "rgb(255 0 170)", json!({})));
    }

    #[test]
    fn test_rgb_percentages() {
        let p = IsRgbColorPredicate;
        assert!(eval(&p, "rgb(100%, 0%, 50%)", json!({})));
        assert!(eval(&p, "rgb(100% 0% 50%)", json!({})));
    }

    #[test]
    fn test_rgba_comma() {
        let p = IsRgbColorPredicate;
        assert!(eval(&p, "rgba(255, 0, 170, 0.5)", json!({})));
        assert!(eval(&p, "rgba(255, 0, 170, 1)", json!({})));
        assert!(eval(&p, "rgba(255, 0, 170, 50%)", json!({})));
    }

    #[test]
    fn test_rgb_slash_alpha() {
        let p = IsRgbColorPredicate;
        assert!(eval(&p, "rgb(255 0 170 / 0.5)", json!({})));
        assert!(eval(&p, "rgb(255 0 170 / 50%)", json!({})));
    }

    #[test]
    fn test_rgb_invalid() {
        let p = IsRgbColorPredicate;
        // Out of range
        assert!(!eval(&p, "rgb(256, 0, 0)", json!({})));
        assert!(!eval(&p, "rgb(-1, 0, 0)", json!({})));
        assert!(!eval(&p, "rgb(0, 0, 101%)", json!({})));
        // Wrong format
        assert!(!eval(&p, "rgb(255, 0)", json!({})));
        assert!(!eval(&p, "rgb()", json!({})));
        assert!(!eval(&p, "hsl(0, 0%, 0%)", json!({})));
        // Mixed percent and number
        assert!(!eval(&p, "rgb(255, 0%, 0)", json!({})));
        // Alpha out of range
        assert!(!eval(&p, "rgba(255, 0, 0, 1.5)", json!({})));
    }

    // --- HSL Color ---

    #[test]
    fn test_hsl_comma_separated() {
        let p = IsHslColorPredicate;
        assert!(eval(&p, "hsl(360, 100%, 50%)", json!({})));
        assert!(eval(&p, "hsl(0, 0%, 0%)", json!({})));
        assert!(eval(&p, "hsl(180, 50%, 75%)", json!({})));
    }

    #[test]
    fn test_hsl_space_separated() {
        let p = IsHslColorPredicate;
        assert!(eval(&p, "hsl(360 100% 50%)", json!({})));
        assert!(eval(&p, "hsl(0 0% 0%)", json!({})));
    }

    #[test]
    fn test_hsla_comma() {
        let p = IsHslColorPredicate;
        assert!(eval(&p, "hsla(360, 100%, 50%, 0.5)", json!({})));
        assert!(eval(&p, "hsla(360, 100%, 50%, 50%)", json!({})));
    }

    #[test]
    fn test_hsl_slash_alpha() {
        let p = IsHslColorPredicate;
        assert!(eval(&p, "hsl(360 100% 50% / 0.5)", json!({})));
        assert!(eval(&p, "hsl(360 100% 50% / 50%)", json!({})));
    }

    #[test]
    fn test_hsl_deg_suffix() {
        let p = IsHslColorPredicate;
        assert!(eval(&p, "hsl(360deg, 100%, 50%)", json!({})));
        assert!(eval(&p, "hsl(180deg 50% 75%)", json!({})));
    }

    #[test]
    fn test_hsl_invalid() {
        let p = IsHslColorPredicate;
        // Hue out of range
        assert!(!eval(&p, "hsl(361, 100%, 50%)", json!({})));
        assert!(!eval(&p, "hsl(-1, 100%, 50%)", json!({})));
        // Saturation/lightness without %
        assert!(!eval(&p, "hsl(360, 100, 50)", json!({})));
        // Out of range percentages
        assert!(!eval(&p, "hsl(360, 101%, 50%)", json!({})));
        // Wrong format
        assert!(!eval(&p, "hsl()", json!({})));
        assert!(!eval(&p, "rgb(255, 0, 0)", json!({})));
        // Alpha out of range
        assert!(!eval(&p, "hsla(360, 100%, 50%, 1.5)", json!({})));
    }
}

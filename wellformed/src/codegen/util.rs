//! Naming utilities for code generation.

/// Rust reserved keywords that cannot be used as identifiers.
const RUST_KEYWORDS: &[&str] = &[
    // Strict keywords
    "as", "break", "const", "continue", "crate", "else", "enum", "extern", "false", "fn", "for",
    "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref", "return",
    "self", "Self", "static", "struct", "super", "trait", "true", "type", "unsafe", "use", "where",
    "while", // Async keywords (2018+)
    "async", "await", "dyn", // Reserved keywords
    "abstract", "become", "box", "do", "final", "macro", "override", "priv", "try", "typeof",
    "unsized", "virtual", "yield",
];

/// Escape a Rust identifier if it's a keyword or starts with a digit.
///
/// Uses raw identifier syntax for keywords: `r#keyword`
/// Prefixes with underscore for digit-starting identifiers: `_2nd`
pub fn escape_keyword(ident: &str) -> String {
    // Check if starts with a digit
    if ident
        .chars()
        .next()
        .map(|c| c.is_ascii_digit())
        .unwrap_or(false)
    {
        return format!("_{}", ident);
    }

    // Check if it's a keyword
    if RUST_KEYWORDS.contains(&ident) {
        format!("r#{}", ident)
    } else {
        ident.to_string()
    }
}

/// Convert a string to snake_case.
///
/// Handles:
/// - CamelCase -> camel_case
/// - kebab-case -> kebab_case
/// - spaces -> underscores
/// - slashes -> underscores (e.g., "Federal/State" -> "federal_state")
/// - periods -> underscores (e.g., "U.S." -> "u_s")
/// - Multiple underscores collapsed
/// - XMLParser -> xml_parser (consecutive uppercase followed by lowercase)
pub fn to_snake_case(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    let mut result = String::new();

    for (i, &c) in chars.iter().enumerate() {
        if c == '-' || c == ' ' || c == '_' || c == '.' || c == '/' {
            if !result.is_empty() && !result.ends_with('_') {
                result.push('_');
            }
        } else if c == '\'' || c == '"' || c == '`' || c == '(' || c == ')' || c == '[' || c == ']'
        {
            // Skip quotes, apostrophes, and brackets - they're not valid in identifiers
            continue;
        } else if c.is_uppercase() {
            let prev = i.checked_sub(1).map(|j| chars[j]);
            let prev_is_lower_or_digit =
                prev.is_some_and(|p| p.is_lowercase() || p.is_ascii_digit());
            let prev_is_upper = prev.is_some_and(|p| p.is_uppercase());
            let next_is_lower = i + 1 < chars.len() && chars[i + 1].is_lowercase();

            // Add underscore before uppercase if:
            // 1. Previous char was lowercase or digit (camelCase, Box1Amount)
            // 2. Previous was uppercase but next is lowercase (XMLParser -> xml_parser)
            if !result.is_empty()
                && !result.ends_with('_')
                && (prev_is_lower_or_digit || (prev_is_upper && next_is_lower))
            {
                result.push('_');
            }
            result.push(c.to_ascii_lowercase());
        } else {
            result.push(c);
        }
    }

    // Clean up: collapse multiple underscores, trim
    let mut final_result = String::new();
    let mut last_was_underscore = false;
    for c in result.chars() {
        if c == '_' {
            if !last_was_underscore && !final_result.is_empty() {
                final_result.push(c);
                last_was_underscore = true;
            }
        } else {
            final_result.push(c);
            last_was_underscore = false;
        }
    }

    final_result.trim_matches('_').to_string()
}

/// Convert a string to PascalCase.
///
/// Handles:
/// - snake_case -> SnakeCase
/// - kebab-case -> KebabCase
/// - already PascalCase -> unchanged
pub fn to_pascal_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = true;

    for c in s.chars() {
        if c == '-' || c == '_' || c == ' ' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_ascii_uppercase());
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }

    result
}

/// Derive a struct name from a schema ID.
///
/// Examples:
/// - "form_1099_int" -> "Form1099Int"
/// - "1099-int" -> "Form1099Int" (adds Form prefix if starts with digit)
pub fn derive_struct_name(id: &str) -> String {
    let pascal = to_pascal_case(id);

    // Rust identifiers can't start with digits
    if pascal
        .chars()
        .next()
        .map(|c| c.is_ascii_digit())
        .unwrap_or(false)
    {
        format!("Form{}", pascal)
    } else {
        pascal
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_snake_case() {
        assert_eq!(to_snake_case("PayerName"), "payer_name");
        assert_eq!(to_snake_case("payer-name"), "payer_name");
        assert_eq!(to_snake_case("payer_name"), "payer_name");
        assert_eq!(to_snake_case("PayerTIN"), "payer_tin");
        assert_eq!(to_snake_case("Box1Amount"), "box1_amount");
        assert_eq!(to_snake_case("already_snake"), "already_snake");
        assert_eq!(to_snake_case("XMLParser"), "xml_parser");
    }

    #[test]
    fn test_to_pascal_case() {
        assert_eq!(to_pascal_case("payer_name"), "PayerName");
        assert_eq!(to_pascal_case("payer-name"), "PayerName");
        assert_eq!(to_pascal_case("PayerName"), "PayerName");
        assert_eq!(to_pascal_case("form_1099_int"), "Form1099Int");
    }

    #[test]
    fn test_derive_struct_name() {
        assert_eq!(derive_struct_name("form_1099_int"), "Form1099Int");
        assert_eq!(derive_struct_name("1099-int"), "Form1099Int");
        assert_eq!(derive_struct_name("PayeeInfo"), "PayeeInfo");
    }
}

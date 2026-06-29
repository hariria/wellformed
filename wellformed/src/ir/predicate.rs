//! Predicate AST for validation constraints.
//!
//! Predicates are portable, declarative validation rules that can be
//! evaluated in any runtime (Rust, TypeScript, etc.).

use super::error::{ErrorMeta, ErrorSeverity};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

/// A validation constraint combining a predicate with error metadata.
///
/// Supports two JSON formats:
///
/// **wellformed format** (canonical):
/// ```json
/// {"pred": {"type": "regex", "pattern": "..."}, "error": {"code": "...", "message": "..."}}
/// ```
///
/// **Templates format** (simplified):
/// ```json
/// {"type": "pattern", "value": "...", "message": "...", "source": "iris"}
/// ```
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Constraint {
    /// Optional stable ID for tracking/referencing this constraint.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// The predicate to evaluate.
    pub pred: Predicate,

    /// Error metadata if the predicate fails.
    pub error: ErrorMeta,
}

impl<'de> Deserialize<'de> for Constraint {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // First, deserialize to a generic Value to inspect the format
        let value = Value::deserialize(deserializer)?;

        // Check if this is wellformed format (has "pred" field) or templates format (has "type" field)
        if value.get("pred").is_some() {
            // wellformed format
            #[derive(Deserialize)]
            struct WelConstraint {
                #[serde(default)]
                id: Option<String>,
                pred: Predicate,
                error: ErrorMeta,
            }

            let wel: WelConstraint =
                serde_json::from_value(value).map_err(serde::de::Error::custom)?;

            Ok(Constraint {
                id: wel.id,
                pred: wel.pred,
                error: wel.error,
            })
        } else if value.get("type").is_some() {
            // Templates format
            let template: TemplateConstraint =
                serde_json::from_value(value).map_err(serde::de::Error::custom)?;

            Ok(template.into_constraint())
        } else {
            Err(serde::de::Error::custom(
                "constraint must have either 'pred' (wellformed format) or 'type' (templates format)",
            ))
        }
    }
}

/// Templates-format constraint (simplified format from enrichment pipeline).
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
enum TemplateConstraint {
    /// Pattern constraint (regex).
    Pattern {
        value: String,
        #[serde(default)]
        message: Option<String>,
        #[serde(default)]
        source: Option<String>,
    },
    /// Maximum length constraint.
    MaxLength {
        value: usize,
        #[serde(default)]
        message: Option<String>,
        #[serde(default)]
        source: Option<String>,
    },
    /// Minimum length constraint.
    MinLength {
        value: usize,
        #[serde(default)]
        message: Option<String>,
        #[serde(default)]
        source: Option<String>,
    },
    /// Format constraint (e.g., "decimal-2").
    Format {
        value: String,
        #[serde(default)]
        message: Option<String>,
        #[serde(default)]
        source: Option<String>,
    },
    /// Enum constraint (value must be one of a set).
    Enum {
        #[serde(alias = "value")]
        values: Vec<Value>,
        #[serde(default)]
        message: Option<String>,
        #[serde(default)]
        source: Option<String>,
    },
}

impl TemplateConstraint {
    fn into_constraint(self) -> Constraint {
        match self {
            TemplateConstraint::Pattern {
                value,
                message,
                source,
            } => Constraint {
                id: None,
                pred: Predicate::Regex {
                    pattern: value.clone(),
                    flags: None,
                },
                error: ErrorMeta {
                    code: "PATTERN_MISMATCH".to_string(),
                    message: message.unwrap_or_else(|| format!("Must match pattern {}", value)),
                    path: None,
                    severity: ErrorSeverity::Error,
                    help: None,
                    source,
                },
            },
            TemplateConstraint::MaxLength {
                value,
                message,
                source,
            } => Constraint {
                id: None,
                pred: Predicate::MaxLen { len: value },
                error: ErrorMeta {
                    code: "MAX_LENGTH_EXCEEDED".to_string(),
                    message: message
                        .unwrap_or_else(|| format!("Must be at most {} characters", value)),
                    path: None,
                    severity: ErrorSeverity::Error,
                    help: None,
                    source,
                },
            },
            TemplateConstraint::MinLength {
                value,
                message,
                source,
            } => Constraint {
                id: None,
                pred: Predicate::MinLen { len: value },
                error: ErrorMeta {
                    code: "MIN_LENGTH_NOT_MET".to_string(),
                    message: message
                        .unwrap_or_else(|| format!("Must be at least {} characters", value)),
                    path: None,
                    severity: ErrorSeverity::Error,
                    help: None,
                    source,
                },
            },
            TemplateConstraint::Format {
                value,
                message,
                source,
            } => {
                // Format constraints are handled as named predicates
                Constraint {
                    id: None,
                    pred: Predicate::Call {
                        name: format!("format:{}", value),
                        args: Value::Null,
                    },
                    error: ErrorMeta {
                        code: "FORMAT_INVALID".to_string(),
                        message: message.unwrap_or_else(|| format!("Must be in {} format", value)),
                        path: None,
                        severity: ErrorSeverity::Error,
                        help: None,
                        source,
                    },
                }
            }
            TemplateConstraint::Enum {
                values,
                message,
                source,
            } => Constraint {
                id: None,
                pred: Predicate::In {
                    path: String::new(),
                    values: values.clone(),
                },
                error: ErrorMeta {
                    code: "INVALID_ENUM_VALUE".to_string(),
                    message: message.unwrap_or_else(|| format!("Must be one of: {:?}", values)),
                    path: None,
                    severity: ErrorSeverity::Error,
                    help: None,
                    source,
                },
            },
        }
    }
}

impl Constraint {
    /// Create a new constraint.
    pub fn new(pred: Predicate, error: ErrorMeta) -> Self {
        Self {
            id: None,
            pred,
            error,
        }
    }

    /// Create a constraint with an ID.
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }
}

/// A segment in a template literal predicate.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TemplateLiteralPart {
    /// Fixed literal text.
    Literal { value: String },
    /// ASCII digits (`0-9`).
    Digits {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        min: Option<usize>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        max: Option<usize>,
    },
    /// ASCII letters (`A-Za-z`).
    AsciiLetters {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        min: Option<usize>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        max: Option<usize>,
    },
    /// ASCII alphanumeric (`A-Za-z0-9`).
    AsciiAlphanumeric {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        min: Option<usize>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        max: Option<usize>,
    },
    /// ASCII uppercase letters (`A-Z`).
    Uppercase {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        min: Option<usize>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        max: Option<usize>,
    },
    /// ASCII lowercase letters (`a-z`).
    Lowercase {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        min: Option<usize>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        max: Option<usize>,
    },
    /// Hexadecimal (`0-9A-Fa-f`).
    Hex {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        min: Option<usize>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        max: Option<usize>,
    },
}

impl TemplateLiteralPart {
    /// Create a fixed literal segment.
    pub fn literal(value: impl Into<String>) -> Self {
        Self::Literal {
            value: value.into(),
        }
    }

    /// Create a digits segment.
    pub fn digits(min: Option<usize>, max: Option<usize>) -> Self {
        Self::Digits { min, max }
    }

    /// Create an ASCII letters segment.
    pub fn ascii_letters(min: Option<usize>, max: Option<usize>) -> Self {
        Self::AsciiLetters { min, max }
    }

    /// Create an ASCII alphanumeric segment.
    pub fn ascii_alphanumeric(min: Option<usize>, max: Option<usize>) -> Self {
        Self::AsciiAlphanumeric { min, max }
    }

    /// Create an uppercase ASCII segment.
    pub fn uppercase(min: Option<usize>, max: Option<usize>) -> Self {
        Self::Uppercase { min, max }
    }

    /// Create a lowercase ASCII segment.
    pub fn lowercase(min: Option<usize>, max: Option<usize>) -> Self {
        Self::Lowercase { min, max }
    }

    /// Create a hexadecimal segment.
    pub fn hex(min: Option<usize>, max: Option<usize>) -> Self {
        Self::Hex { min, max }
    }
}

/// A portable predicate expression.
///
/// Predicates form an AST that can be evaluated against JSON values.
/// All predicates are deterministic and can be serialized to JSON.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Predicate {
    // ========================================================================
    // String predicates
    // ========================================================================
    /// Match a regular expression pattern.
    Regex {
        /// The regex pattern.
        pattern: String,
        /// Optional flags (e.g., "i" for case-insensitive).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        flags: Option<String>,
    },

    /// Match a structured template literal pattern without regex.
    TemplateLiteral {
        /// Ordered template parts to match.
        parts: Vec<TemplateLiteralPart>,
    },

    /// Minimum string/array length.
    MinLen {
        /// Minimum length (inclusive).
        len: usize,
    },

    /// Maximum string/array length.
    MaxLen {
        /// Maximum length (inclusive).
        len: usize,
    },

    // ========================================================================
    // Numeric predicates
    // ========================================================================
    /// Numeric range check.
    Range {
        /// Minimum value (inclusive).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        min: Option<f64>,
        /// Maximum value (inclusive).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        max: Option<f64>,
    },

    // ========================================================================
    // Path-based predicates (relative JSON Pointer paths)
    // ========================================================================
    /// Check if a path exists and is not null.
    Exists {
        /// Relative JSON Pointer path.
        path: String,
    },

    /// Check if a path equals a specific value.
    Eq {
        /// Relative JSON Pointer path.
        path: String,
        /// Expected value.
        value: Value,
    },

    /// Check if a path's value is in a set of values.
    In {
        /// Relative JSON Pointer path.
        path: String,
        /// Set of allowed values.
        values: Vec<Value>,
    },

    /// Require a field when another field is present.
    ///
    /// Equivalent to: `!exists(with) || exists(field)`.
    RequiredWith {
        /// Field that becomes required.
        field: String,
        /// Field that triggers the requirement when present.
        with: String,
    },

    /// Require a field when another field is absent.
    ///
    /// Equivalent to: `exists(without) || exists(field)`.
    RequiredWithout {
        /// Field that becomes required.
        field: String,
        /// Field whose absence triggers the requirement.
        without: String,
    },

    /// Require exactly one of the given fields to be present.
    ExactlyOneOf {
        /// Field paths where exactly one must exist.
        paths: Vec<String>,
    },

    // ========================================================================
    // Cross-field predicates
    // ========================================================================
    /// Check if two fields have equal values.
    EqFields {
        /// Left field path.
        left: String,
        /// Right field path.
        right: String,
    },

    /// Check if left field > right field.
    GtField {
        /// Left field path.
        left: String,
        /// Right field path.
        right: String,
    },

    /// Check if left field >= right field.
    GteField {
        /// Left field path.
        left: String,
        /// Right field path.
        right: String,
    },

    /// Check if left field < right field.
    LtField {
        /// Left field path.
        left: String,
        /// Right field path.
        right: String,
    },

    /// Check if left field <= right field.
    LteField {
        /// Left field path.
        left: String,
        /// Right field path.
        right: String,
    },

    /// Check if sum of fields equals a target field.
    SumEquals {
        /// Fields to sum.
        paths: Vec<String>,
        /// Target field that should equal the sum.
        target: String,
    },

    /// Check if sum of fields equals a specific value.
    SumEqualsValue {
        /// Fields to sum.
        paths: Vec<String>,
        /// Value that the sum should equal.
        value: f64,
    },

    // ========================================================================
    // Boolean combinators
    // ========================================================================
    /// Logical AND of predicates.
    And {
        /// Predicates that must all be true.
        predicates: Vec<Predicate>,
    },

    /// Logical OR of predicates.
    Or {
        /// At least one predicate must be true.
        predicates: Vec<Predicate>,
    },

    /// Logical NOT of a predicate.
    Not {
        /// The predicate to negate.
        predicate: Box<Predicate>,
    },

    /// Logical implication: if antecedent is true, consequent must be true.
    Implies {
        /// The condition.
        #[serde(rename = "if")]
        antecedent: Box<Predicate>,
        /// The requirement when condition is true.
        #[serde(rename = "then")]
        consequent: Box<Predicate>,
    },

    // ========================================================================
    // Named predicates (extension hooks)
    // ========================================================================
    /// Call a named predicate function.
    ///
    /// Named predicates are implemented in each runtime and must have
    /// deterministic, identical behavior across languages.
    Call {
        /// Name of the predicate function.
        name: String,
        /// Arguments to pass to the function.
        #[serde(default, skip_serializing_if = "Value::is_null")]
        args: Value,
    },

    // ========================================================================
    // Constant predicates (useful for testing)
    // ========================================================================
    /// Always true.
    True,

    /// Always false.
    False,
}

impl Predicate {
    // ========================================================================
    // String predicate constructors
    // ========================================================================

    /// Create a regex predicate.
    pub fn regex(pattern: impl Into<String>) -> Self {
        Self::Regex {
            pattern: pattern.into(),
            flags: None,
        }
    }

    /// Create a regex predicate with flags.
    pub fn regex_with_flags(pattern: impl Into<String>, flags: impl Into<String>) -> Self {
        Self::Regex {
            pattern: pattern.into(),
            flags: Some(flags.into()),
        }
    }

    /// Create a template literal predicate.
    pub fn template_literal(parts: Vec<TemplateLiteralPart>) -> Self {
        Self::TemplateLiteral { parts }
    }

    /// Create a minimum length predicate.
    pub fn min_len(len: usize) -> Self {
        Self::MinLen { len }
    }

    /// Create a maximum length predicate.
    pub fn max_len(len: usize) -> Self {
        Self::MaxLen { len }
    }

    // ========================================================================
    // Numeric predicate constructors
    // ========================================================================

    /// Create a range predicate.
    pub fn range(min: Option<f64>, max: Option<f64>) -> Self {
        Self::Range { min, max }
    }

    /// Create a minimum value predicate.
    pub fn min(min: f64) -> Self {
        Self::Range {
            min: Some(min),
            max: None,
        }
    }

    /// Create a maximum value predicate.
    pub fn max(max: f64) -> Self {
        Self::Range {
            min: None,
            max: Some(max),
        }
    }

    // ========================================================================
    // Path-based predicate constructors
    // ========================================================================

    /// Create an exists predicate.
    pub fn exists(path: impl Into<String>) -> Self {
        Self::Exists { path: path.into() }
    }

    /// Create an equality predicate.
    pub fn eq(path: impl Into<String>, value: impl Into<Value>) -> Self {
        Self::Eq {
            path: path.into(),
            value: value.into(),
        }
    }

    /// Create an "in set" predicate.
    pub fn in_values(path: impl Into<String>, values: Vec<Value>) -> Self {
        Self::In {
            path: path.into(),
            values,
        }
    }

    /// Create a required-with predicate.
    pub fn required_with(field: impl Into<String>, with: impl Into<String>) -> Self {
        Self::RequiredWith {
            field: field.into(),
            with: with.into(),
        }
    }

    /// Create a required-without predicate.
    pub fn required_without(field: impl Into<String>, without: impl Into<String>) -> Self {
        Self::RequiredWithout {
            field: field.into(),
            without: without.into(),
        }
    }

    /// Create an exactly-one-of predicate.
    pub fn exactly_one_of(paths: Vec<String>) -> Self {
        Self::ExactlyOneOf { paths }
    }

    // ========================================================================
    // Boolean combinator constructors
    // ========================================================================

    /// Create an AND predicate.
    pub fn and(predicates: Vec<Predicate>) -> Self {
        Self::And { predicates }
    }

    /// Create an OR predicate.
    pub fn or(predicates: Vec<Predicate>) -> Self {
        Self::Or { predicates }
    }

    /// Create a NOT predicate.
    #[allow(clippy::should_implement_trait)]
    pub fn not(predicate: Predicate) -> Self {
        Self::Not {
            predicate: Box::new(predicate),
        }
    }

    /// Create an implies predicate.
    pub fn implies(antecedent: Predicate, consequent: Predicate) -> Self {
        Self::Implies {
            antecedent: Box::new(antecedent),
            consequent: Box::new(consequent),
        }
    }

    // ========================================================================
    // Named predicate constructors
    // ========================================================================

    /// Create a named predicate call.
    pub fn call(name: impl Into<String>, args: impl Into<Value>) -> Self {
        Self::Call {
            name: name.into(),
            args: args.into(),
        }
    }

    /// Create a named predicate call with no arguments.
    pub fn call_no_args(name: impl Into<String>) -> Self {
        Self::Call {
            name: name.into(),
            args: Value::Null,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_serde_regex() {
        let p = Predicate::regex(r"^\d{9}$");
        let json = serde_json::to_string(&p).unwrap();
        assert!(json.contains("regex"));
        // Backslash gets escaped in JSON: \d -> \\d
        assert!(json.contains(r"\\d{9}"));

        let parsed: Predicate = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, p);
    }

    #[test]
    fn test_serde_template_literal() {
        let p = Predicate::template_literal(vec![
            TemplateLiteralPart::literal("SFO-"),
            TemplateLiteralPart::digits(Some(3), Some(4)),
            TemplateLiteralPart::literal("-"),
            TemplateLiteralPart::uppercase(Some(2), Some(2)),
        ]);
        let json = serde_json::to_string(&p).unwrap();
        assert!(json.contains("template_literal"));
        assert!(json.contains("digits"));
        assert!(json.contains("uppercase"));

        let parsed: Predicate = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, p);
    }

    #[test]
    fn test_serde_range() {
        let p = Predicate::range(Some(0.0), Some(100.0));
        let json = serde_json::to_string(&p).unwrap();
        let parsed: Predicate = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, p);
    }

    #[test]
    fn test_serde_implies() {
        let p = Predicate::implies(
            Predicate::eq("/is_foreign", json!(false)),
            Predicate::exists("/zip"),
        );
        let json = serde_json::to_string_pretty(&p).unwrap();
        assert!(json.contains("implies"));
        assert!(json.contains("if"));
        assert!(json.contains("then"));

        let parsed: Predicate = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, p);
    }

    #[test]
    fn test_serde_call() {
        let p = Predicate::call("is_tin", json!({"kind": "ANY"}));
        let json = serde_json::to_string(&p).unwrap();
        assert!(json.contains("call"));
        assert!(json.contains("is_tin"));

        let parsed: Predicate = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, p);
    }

    #[test]
    fn test_serde_and() {
        let p = Predicate::and(vec![Predicate::min_len(1), Predicate::max_len(100)]);
        let json = serde_json::to_string(&p).unwrap();
        let parsed: Predicate = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, p);
    }

    #[test]
    fn test_serde_required_with() {
        let p = Predicate::required_with("/confirm_password", "/password");
        let json = serde_json::to_string(&p).unwrap();
        assert!(json.contains("\"required_with\""));
        let parsed: Predicate = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, p);
    }

    #[test]
    fn test_serde_required_without() {
        let p = Predicate::required_without("/tax_id", "/ssn");
        let json = serde_json::to_string(&p).unwrap();
        assert!(json.contains("\"required_without\""));
        let parsed: Predicate = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, p);
    }

    #[test]
    fn test_serde_exactly_one_of() {
        let p = Predicate::exactly_one_of(vec!["/ssn".to_string(), "/ein".to_string()]);
        let json = serde_json::to_string(&p).unwrap();
        assert!(json.contains("\"exactly_one_of\""));
        let parsed: Predicate = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, p);
    }

    #[test]
    fn test_constraint_with_id() {
        let c = Constraint::new(
            Predicate::regex(r"^\d{9}$"),
            ErrorMeta::new("TIN_INVALID", "TIN must be 9 digits"),
        )
        .with_id("tin-format-check");

        assert_eq!(c.id, Some("tin-format-check".to_string()));
    }

    #[test]
    fn test_template_enum_constraint_targets_current_value() {
        let c: Constraint =
            serde_json::from_str(r#"{"type":"enum","value":["A"],"message":"pick A"}"#).unwrap();

        assert_eq!(
            c.pred,
            Predicate::In {
                path: String::new(),
                values: vec![json!("A")]
            }
        );
    }
}

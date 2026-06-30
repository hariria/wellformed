//! Validation engine.
//!
//! This module implements the main validation logic that combines
//! transforms, type checking, and constraint evaluation.

use crate::error::Result;
use crate::ir::{
    ArraySchema, CatchSchema, Constraint, CurrencySchema, DateSchema, DecimalSchema, EnumSchema,
    FormError, Int32Schema, Int64Schema, IntegerSchema, IntersectionSchema, LiteralSchema,
    MoneySchema, NeverSchema, NumberSchema, ObjectSchema, PercentageFormat, PercentageSchema,
    Predicate, PreprocessSchema, RecordSchema, Schema, StringSchema, Transform, TupleSchema,
    TypeSchema, Uint32Schema, Uint64Schema, UnionSchema, UnknownKeysBehavior,
};
use crate::runtime::predicate::{evaluate as eval_predicate, EvalContext, PredicateRegistry};
use crate::runtime::transform::apply_transforms;
use serde_json::Value;

/// Result of validating a value against a schema.
#[derive(Debug, Clone, Default)]
pub struct ValidationResult {
    /// Hard errors that cause validation to fail.
    pub errors: Vec<FormError>,
    /// Warnings that don't cause validation to fail.
    pub warnings: Vec<FormError>,
}

impl ValidationResult {
    /// Create an empty result.
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if validation passed (no errors, warnings are OK).
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    /// Add a form error.
    pub fn add_error(&mut self, error: FormError) {
        if error.is_warning() {
            self.warnings.push(error);
        } else {
            self.errors.push(error);
        }
    }

    /// Merge another result into this one.
    pub fn merge(&mut self, other: ValidationResult) {
        self.errors.extend(other.errors);
        self.warnings.extend(other.warnings);
    }

    /// Get all issues (errors and warnings).
    pub fn all_issues(&self) -> impl Iterator<Item = &FormError> {
        self.errors.iter().chain(self.warnings.iter())
    }
}

/// Maximum schema/data recursion depth. Bounds both deeply nested input and
/// recursive `$ref` schemas so a cyclic or pathological schema returns a clean
/// error instead of overflowing the stack (a DoS vector). See conformance:
/// recursive-ref-schema.
const MAX_VALIDATION_DEPTH: usize = 128;

/// Validator for executing schema validation.
pub struct Validator<'a> {
    schema: &'a Schema,
    registry: &'a PredicateRegistry,
    depth: std::cell::Cell<usize>,
}

impl<'a> Validator<'a> {
    /// Create a new validator.
    pub fn new(schema: &'a Schema, registry: &'a PredicateRegistry) -> Self {
        Self {
            schema,
            registry,
            depth: std::cell::Cell::new(0),
        }
    }

    /// Validate a value against the schema's root type.
    ///
    /// The value is normalized in place (transforms are applied).
    pub fn validate(&self, value: &mut Value) -> Result<ValidationResult> {
        let mut ctx = EvalContext::new(self.registry);
        self.validate_type(&self.schema.root, value, "", &mut ctx)
    }

    /// Validate a value against a specific type schema.
    fn validate_type(
        &self,
        type_schema: &TypeSchema,
        value: &mut Value,
        path: &str,
        ctx: &mut EvalContext,
    ) -> Result<ValidationResult> {
        let depth = self.depth.get() + 1;
        if depth > MAX_VALIDATION_DEPTH {
            // In-band (valid:false), matching the TypeScript runtime's channel for
            // a cyclic/over-deep schema rather than aborting with an Err. The
            // depth has not been incremented yet, so there is nothing to restore.
            let mut result = ValidationResult::new();
            result.add_error(FormError::new(
                "MAX_DEPTH_EXCEEDED",
                format!("maximum validation depth ({MAX_VALIDATION_DEPTH}) exceeded"),
                path,
            ));
            return Ok(result);
        }
        self.depth.set(depth);
        let result = match type_schema {
            TypeSchema::String(schema) => self.validate_string(schema, value, path, ctx),
            TypeSchema::Number(schema) => self.validate_number(schema, value, path, ctx),
            TypeSchema::Integer(schema) => self.validate_integer(schema, value, path, ctx),
            TypeSchema::Int32(schema) => self.validate_int32(schema, value, path, ctx),
            TypeSchema::Int64(schema) => self.validate_int64(schema, value, path, ctx),
            TypeSchema::Uint32(schema) => self.validate_uint32(schema, value, path, ctx),
            TypeSchema::Uint64(schema) => self.validate_uint64(schema, value, path, ctx),
            TypeSchema::Boolean(_) => self.validate_boolean(value, path),
            TypeSchema::Money(schema) => self.validate_money(schema, value, path, ctx),
            TypeSchema::Currency(schema) => self.validate_currency(schema, value, path, ctx),
            TypeSchema::Decimal(schema) => self.validate_decimal(schema, value, path, ctx),
            TypeSchema::Percentage(schema) => self.validate_percentage(schema, value, path, ctx),
            TypeSchema::Date(schema) => self.validate_date(schema, value, path, ctx),
            TypeSchema::Object(schema) => self.validate_object(schema, value, path, ctx),
            TypeSchema::Array(schema) => self.validate_array(schema, value, path, ctx),
            TypeSchema::Tuple(schema) => self.validate_tuple(schema, value, path, ctx),
            TypeSchema::Enum(schema) => self.validate_enum(schema, value, path),
            TypeSchema::Literal(schema) => self.validate_literal(schema, value, path),
            TypeSchema::Never(schema) => self.validate_never(schema, value, path),
            TypeSchema::Union(schema) => self.validate_union(schema, value, path, ctx),
            TypeSchema::Intersection(schema) => {
                self.validate_intersection(schema, value, path, ctx)
            }
            TypeSchema::Record(schema) => self.validate_record(schema, value, path, ctx),
            TypeSchema::Preprocess(schema) => self.validate_preprocess(schema, value, path, ctx),
            TypeSchema::Catch(schema) => self.validate_catch(schema, value, path, ctx),
            TypeSchema::Ref { name } => self.validate_ref(name, value, path, ctx),
            TypeSchema::Any(_) => Ok(ValidationResult::new()), // Any type always passes
        };
        self.depth.set(depth - 1);
        result
    }

    fn validate_string(
        &self,
        schema: &StringSchema,
        value: &mut Value,
        path: &str,
        ctx: &mut EvalContext,
    ) -> Result<ValidationResult> {
        let mut result = ValidationResult::new();

        // Apply transforms
        apply_transforms(value, &schema.transforms, path)?;

        // Type check (null is OK if not required - handled at object level)
        if !value.is_string() && !value.is_null() {
            result.add_error(FormError::new(
                "TYPE_ERROR",
                format!("expected string, got {}", value_type_name(value)),
                path,
            ));
            return Ok(result);
        }

        // Evaluate constraints for any present string value, including empty string.
        if value.is_string() {
            self.evaluate_constraints(&schema.constraints, value, path, ctx, &mut result)?;
        }

        Ok(result)
    }

    fn validate_number(
        &self,
        schema: &NumberSchema,
        value: &mut Value,
        path: &str,
        ctx: &mut EvalContext,
    ) -> Result<ValidationResult> {
        let mut result = ValidationResult::new();

        apply_transforms(value, &schema.transforms, path)?;

        if !value.is_number() && !value.is_null() {
            result.add_error(FormError::new(
                "TYPE_ERROR",
                format!("expected number, got {}", value_type_name(value)),
                path,
            ));
            return Ok(result);
        }

        if value.is_number() {
            self.evaluate_constraints(&schema.constraints, value, path, ctx, &mut result)?;
        }

        Ok(result)
    }

    fn validate_integer(
        &self,
        schema: &IntegerSchema,
        value: &mut Value,
        path: &str,
        ctx: &mut EvalContext,
    ) -> Result<ValidationResult> {
        let mut result = ValidationResult::new();

        apply_transforms(value, &schema.transforms, path)?;

        if value.is_null() {
            return Ok(result);
        }

        // Check if it's a valid integer
        let is_integer = match value {
            Value::Number(n) => {
                n.is_i64() || n.is_u64() || n.as_f64().is_some_and(|f| f.fract() == 0.0)
            }
            _ => false,
        };

        if !is_integer {
            result.add_error(FormError::new(
                "TYPE_ERROR",
                format!("expected integer, got {}", value_type_name(value)),
                path,
            ));
            return Ok(result);
        }

        self.evaluate_constraints(&schema.constraints, value, path, ctx, &mut result)?;

        Ok(result)
    }

    fn validate_int32(
        &self,
        schema: &Int32Schema,
        value: &mut Value,
        path: &str,
        ctx: &mut EvalContext,
    ) -> Result<ValidationResult> {
        let mut result = ValidationResult::new();

        apply_transforms(value, &schema.transforms, path)?;

        if value.is_null() {
            return Ok(result);
        }

        // Check if it's a valid i32
        match value.as_i64() {
            Some(n) if n >= i32::MIN as i64 && n <= i32::MAX as i64 => {
                self.evaluate_constraints(&schema.constraints, value, path, ctx, &mut result)?;
            }
            _ => {
                result.add_error(FormError::new(
                    "TYPE_ERROR",
                    format!(
                        "expected int32 ({} to {}), got {}",
                        i32::MIN,
                        i32::MAX,
                        value_type_name(value)
                    ),
                    path,
                ));
            }
        }

        Ok(result)
    }

    fn validate_int64(
        &self,
        schema: &Int64Schema,
        value: &mut Value,
        path: &str,
        ctx: &mut EvalContext,
    ) -> Result<ValidationResult> {
        let mut result = ValidationResult::new();

        apply_transforms(value, &schema.transforms, path)?;

        if value.is_null() {
            return Ok(result);
        }

        // Check if it's a valid i64
        match value.as_i64() {
            Some(_) => {
                self.evaluate_constraints(&schema.constraints, value, path, ctx, &mut result)?;
            }
            None => {
                result.add_error(FormError::new(
                    "TYPE_ERROR",
                    format!("expected int64, got {}", value_type_name(value)),
                    path,
                ));
            }
        }

        Ok(result)
    }

    fn validate_uint32(
        &self,
        schema: &Uint32Schema,
        value: &mut Value,
        path: &str,
        ctx: &mut EvalContext,
    ) -> Result<ValidationResult> {
        let mut result = ValidationResult::new();

        apply_transforms(value, &schema.transforms, path)?;

        if value.is_null() {
            return Ok(result);
        }

        // Check if it's a valid u32
        match value.as_u64() {
            Some(n) if n <= u32::MAX as u64 => {
                self.evaluate_constraints(&schema.constraints, value, path, ctx, &mut result)?;
            }
            _ => {
                result.add_error(FormError::new(
                    "TYPE_ERROR",
                    format!(
                        "expected uint32 (0 to {}), got {}",
                        u32::MAX,
                        value_type_name(value)
                    ),
                    path,
                ));
            }
        }

        Ok(result)
    }

    fn validate_uint64(
        &self,
        schema: &Uint64Schema,
        value: &mut Value,
        path: &str,
        ctx: &mut EvalContext,
    ) -> Result<ValidationResult> {
        let mut result = ValidationResult::new();

        apply_transforms(value, &schema.transforms, path)?;

        if value.is_null() {
            return Ok(result);
        }

        // Check if it's a valid u64
        match value.as_u64() {
            Some(_) => {
                self.evaluate_constraints(&schema.constraints, value, path, ctx, &mut result)?;
            }
            None => {
                result.add_error(FormError::new(
                    "TYPE_ERROR",
                    format!(
                        "expected uint64 (non-negative integer), got {}",
                        value_type_name(value)
                    ),
                    path,
                ));
            }
        }

        Ok(result)
    }

    fn validate_boolean(&self, value: &Value, path: &str) -> Result<ValidationResult> {
        let mut result = ValidationResult::new();

        if !value.is_boolean() && !value.is_null() {
            result.add_error(FormError::new(
                "TYPE_ERROR",
                format!("expected boolean, got {}", value_type_name(value)),
                path,
            ));
        }

        Ok(result)
    }

    fn validate_money(
        &self,
        schema: &MoneySchema,
        value: &mut Value,
        path: &str,
        ctx: &mut EvalContext,
    ) -> Result<ValidationResult> {
        let mut result = ValidationResult::new();

        apply_transforms(value, &schema.transforms, path)?;

        // Money should be a number (cents) after transforms
        if !value.is_number() && !value.is_null() {
            result.add_error(FormError::new(
                "TYPE_ERROR",
                format!("expected money (number), got {}", value_type_name(value)),
                path,
            ));
            return Ok(result);
        }

        if value.is_number() {
            self.evaluate_constraints(&schema.constraints, value, path, ctx, &mut result)?;
        }

        Ok(result)
    }

    fn validate_currency(
        &self,
        schema: &CurrencySchema,
        value: &mut Value,
        path: &str,
        ctx: &mut EvalContext,
    ) -> Result<ValidationResult> {
        let mut result = ValidationResult::new();

        apply_transforms(value, &schema.transforms, path)?;

        // Currency should be a number (amount)
        if !value.is_number() && !value.is_null() {
            result.add_error(FormError::new(
                "TYPE_ERROR",
                format!("expected currency (number), got {}", value_type_name(value)),
                path,
            ));
            return Ok(result);
        }

        if let Some(n) = value.as_f64() {
            // Validate scale (decimal places)
            let scale = schema.scale;
            let scaled = n * 10f64.powi(scale as i32);
            if (scaled - scaled.round()).abs() > 1e-10 {
                result.add_error(FormError::new(
                    "CURRENCY_SCALE_EXCEEDED",
                    format!("currency value has more than {} decimal places", scale),
                    path,
                ));
            }

            self.evaluate_constraints(&schema.constraints, value, path, ctx, &mut result)?;
        }

        Ok(result)
    }

    fn validate_decimal(
        &self,
        schema: &DecimalSchema,
        value: &mut Value,
        path: &str,
        ctx: &mut EvalContext,
    ) -> Result<ValidationResult> {
        let mut result = ValidationResult::new();

        apply_transforms(value, &schema.transforms, path)?;

        // Decimal should be a number
        if !value.is_number() && !value.is_null() {
            result.add_error(FormError::new(
                "TYPE_ERROR",
                format!("expected decimal (number), got {}", value_type_name(value)),
                path,
            ));
            return Ok(result);
        }

        if let Some(n) = value.as_f64() {
            // Validate scale (decimal places)
            if let Some(scale) = schema.scale {
                let scaled = n * 10f64.powi(scale as i32);
                if (scaled - scaled.round()).abs() > 1e-10 {
                    result.add_error(FormError::new(
                        "DECIMAL_SCALE_EXCEEDED",
                        format!("value has more than {} decimal places", scale),
                        path,
                    ));
                }
            }

            // Validate precision (total digits)
            if let Some(precision) = schema.precision {
                let abs_n = n.abs();
                let digit_count = if abs_n == 0.0 {
                    1
                } else {
                    // Count digits by converting to string (simple approach)
                    let s = format!("{:.10}", abs_n);
                    let s = s.trim_end_matches('0').trim_end_matches('.');
                    s.chars().filter(|c| c.is_ascii_digit()).count()
                };
                if digit_count > precision as usize {
                    result.add_error(FormError::new(
                        "DECIMAL_PRECISION_EXCEEDED",
                        format!("value exceeds {} total digits", precision),
                        path,
                    ));
                }
            }

            self.evaluate_constraints(&schema.constraints, value, path, ctx, &mut result)?;
        }

        Ok(result)
    }

    fn validate_percentage(
        &self,
        schema: &PercentageSchema,
        value: &mut Value,
        path: &str,
        ctx: &mut EvalContext,
    ) -> Result<ValidationResult> {
        let mut result = ValidationResult::new();

        apply_transforms(value, &schema.transforms, path)?;

        // Percentage should be a number
        if !value.is_number() && !value.is_null() {
            result.add_error(FormError::new(
                "TYPE_ERROR",
                format!(
                    "expected percentage (number), got {}",
                    value_type_name(value)
                ),
                path,
            ));
            return Ok(result);
        }

        if let Some(n) = value.as_f64() {
            // Validate range based on format
            let (min, max) = match schema.format {
                PercentageFormat::Decimal => {
                    (0.0, if schema.allow_over_100 { f64::MAX } else { 1.0 })
                }
                PercentageFormat::Whole => (
                    0.0,
                    if schema.allow_over_100 {
                        f64::MAX
                    } else {
                        100.0
                    },
                ),
            };

            if n < min {
                result.add_error(FormError::new(
                    "PERCENTAGE_NEGATIVE",
                    "percentage cannot be negative",
                    path,
                ));
            } else if n > max && !schema.allow_over_100 {
                let max_display = match schema.format {
                    PercentageFormat::Decimal => "1.0 (100%)",
                    PercentageFormat::Whole => "100",
                };
                result.add_error(FormError::new(
                    "PERCENTAGE_TOO_HIGH",
                    format!("percentage cannot exceed {}", max_display),
                    path,
                ));
            }

            // Validate scale (decimal places)
            if let Some(scale) = schema.scale {
                let scaled = n * 10f64.powi(scale as i32);
                if (scaled - scaled.round()).abs() > 1e-10 {
                    result.add_error(FormError::new(
                        "PERCENTAGE_SCALE_EXCEEDED",
                        format!("percentage has more than {} decimal places", scale),
                        path,
                    ));
                }
            }

            self.evaluate_constraints(&schema.constraints, value, path, ctx, &mut result)?;
        }

        Ok(result)
    }

    fn validate_date(
        &self,
        schema: &DateSchema,
        value: &mut Value,
        path: &str,
        ctx: &mut EvalContext,
    ) -> Result<ValidationResult> {
        let mut result = ValidationResult::new();

        apply_transforms(value, &schema.transforms, path)?;

        if !value.is_string() && !value.is_null() {
            result.add_error(FormError::new(
                "TYPE_ERROR",
                format!("expected date string, got {}", value_type_name(value)),
                path,
            ));
            return Ok(result);
        }

        if value.is_string() {
            self.evaluate_constraints(&schema.constraints, value, path, ctx, &mut result)?;
        }

        Ok(result)
    }

    fn validate_object(
        &self,
        schema: &ObjectSchema,
        value: &mut Value,
        path: &str,
        ctx: &mut EvalContext,
    ) -> Result<ValidationResult> {
        let mut result = ValidationResult::new();

        if value.is_null() {
            return Ok(result);
        }

        let obj = match value.as_object_mut() {
            Some(obj) => obj,
            None => {
                result.add_error(FormError::new(
                    "TYPE_ERROR",
                    format!("expected object, got {}", value_type_name(value)),
                    path,
                ));
                return Ok(result);
            }
        };

        // Validate each field
        for (prop_name, prop_schema) in &schema.fields {
            let prop_path = if path.is_empty() {
                format!("/{}", escape_pointer_segment(prop_name))
            } else {
                format!("{}/{}", path, escape_pointer_segment(prop_name))
            };

            match obj.get_mut(prop_name) {
                Some(prop_value) => {
                    if prop_schema.required
                        && prop_value.is_null()
                        && !self.schema_allows_null(&prop_schema.schema)
                    {
                        result.add_error(FormError::new(
                            "REQUIRED",
                            format!("required field '{}' cannot be null", prop_name),
                            &prop_path,
                        ));
                        continue;
                    }

                    let prop_result =
                        self.validate_type(&prop_schema.schema, prop_value, &prop_path, ctx)?;
                    result.merge(prop_result);
                }
                None => {
                    if self.schema_fills_missing(&prop_schema.schema) {
                        let mut prop_value = Value::Null;
                        let prop_result = self.validate_type(
                            &prop_schema.schema,
                            &mut prop_value,
                            &prop_path,
                            ctx,
                        )?;
                        result.merge(prop_result);
                        obj.insert(prop_name.clone(), prop_value);
                        continue;
                    }

                    if prop_schema.required {
                        result.add_error(FormError::new(
                            "REQUIRED",
                            format!("required field '{}' is missing", prop_name),
                            &prop_path,
                        ));
                    }
                }
            }
        }

        // Resolve unknown-key behavior. `unknown_keys` overrides legacy flag.
        let unknown_behavior = schema
            .unknown_keys
            .unwrap_or(if schema.additional_properties {
                UnknownKeysBehavior::Passthrough
            } else {
                UnknownKeysBehavior::Strict
            });

        // Validate/reject/strip unknown properties.
        let unknown_keys = obj
            .keys()
            .filter(|key| !schema.fields.contains_key(*key))
            .cloned()
            .collect::<Vec<_>>();

        for key in unknown_keys {
            let prop_path = if path.is_empty() {
                format!("/{}", escape_pointer_segment(&key))
            } else {
                format!("{}/{}", path, escape_pointer_segment(&key))
            };

            if let Some(catchall_schema) = &schema.catchall {
                if let Some(prop_value) = obj.get_mut(&key) {
                    let prop_result =
                        self.validate_type(catchall_schema, prop_value, &prop_path, ctx)?;
                    result.merge(prop_result);
                }
                continue;
            }

            match unknown_behavior {
                UnknownKeysBehavior::Strict => {
                    result.add_error(FormError::new(
                        "ADDITIONAL_PROPERTY_NOT_ALLOWED",
                        format!("additional property '{}' is not allowed", key),
                        &prop_path,
                    ));
                }
                UnknownKeysBehavior::Passthrough => {}
                UnknownKeysBehavior::Strip => {
                    obj.remove(&key);
                }
            }
        }

        // Evaluate cross-field rules
        // Rules are evaluated against the object as a whole
        self.evaluate_constraints(&schema.rules, value, path, ctx, &mut result)?;

        Ok(result)
    }

    fn validate_array(
        &self,
        schema: &ArraySchema,
        value: &mut Value,
        path: &str,
        ctx: &mut EvalContext,
    ) -> Result<ValidationResult> {
        let mut result = ValidationResult::new();

        if value.is_null() {
            return Ok(result);
        }

        let arr = match value.as_array_mut() {
            Some(arr) => arr,
            None => {
                result.add_error(FormError::new(
                    "TYPE_ERROR",
                    format!("expected array, got {}", value_type_name(value)),
                    path,
                ));
                return Ok(result);
            }
        };

        // Check array length constraints
        if let Some(min) = schema.min_items {
            if arr.len() < min && !has_min_len_constraint(&schema.constraints, min) {
                result.add_error(FormError::new(
                    "ARRAY_TOO_SHORT",
                    format!("array must have at least {} items", min),
                    path,
                ));
            }
        }

        if let Some(max) = schema.max_items {
            if arr.len() > max && !has_max_len_constraint(&schema.constraints, max) {
                result.add_error(FormError::new(
                    "ARRAY_TOO_LONG",
                    format!("array must have at most {} items", max),
                    path,
                ));
            }
        }

        // Validate each item
        for (i, item) in arr.iter_mut().enumerate() {
            let item_path = format!("{}/{}", path, i);
            let item_result = self.validate_type(&schema.items, item, &item_path, ctx)?;
            result.merge(item_result);
        }

        // Evaluate array-level constraints
        self.evaluate_constraints(&schema.constraints, value, path, ctx, &mut result)?;

        Ok(result)
    }

    fn validate_enum(
        &self,
        schema: &EnumSchema,
        value: &Value,
        path: &str,
    ) -> Result<ValidationResult> {
        let mut result = ValidationResult::new();

        if value.is_null() {
            return Ok(result);
        }

        // Skip validation for empty strings (optional fields default to "")
        if let Some(s) = value.as_str() {
            if s.is_empty() {
                return Ok(result);
            }
        }

        if !schema.values.iter().any(|v| json_value_eq(v, value)) {
            result.add_error(FormError::new(
                "INVALID_ENUM",
                format!(
                    "value must be one of: {}",
                    schema
                        .values
                        .iter()
                        .map(|v| format!("{}", v))
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
                path,
            ));
        }

        Ok(result)
    }

    fn validate_literal(
        &self,
        schema: &LiteralSchema,
        value: &Value,
        path: &str,
    ) -> Result<ValidationResult> {
        let mut result = ValidationResult::new();

        if !json_value_eq(value, &schema.value) {
            result.add_error(FormError::new(
                "INVALID_LITERAL",
                format!("expected literal {}, got {}", schema.value, value),
                path,
            ));
        }

        Ok(result)
    }

    fn validate_never(
        &self,
        _schema: &NeverSchema,
        value: &Value,
        path: &str,
    ) -> Result<ValidationResult> {
        let mut result = ValidationResult::new();
        result.add_error(FormError::new(
            "TYPE_ERROR",
            format!("expected never, got {}", value_type_name(value)),
            path,
        ));
        Ok(result)
    }

    fn validate_tuple(
        &self,
        schema: &TupleSchema,
        value: &mut Value,
        path: &str,
        ctx: &mut EvalContext,
    ) -> Result<ValidationResult> {
        let mut result = ValidationResult::new();

        if value.is_null() {
            return Ok(result);
        }

        let arr = match value.as_array_mut() {
            Some(arr) => arr,
            None => {
                result.add_error(FormError::new(
                    "TYPE_ERROR",
                    format!("expected tuple (array), got {}", value_type_name(value)),
                    path,
                ));
                return Ok(result);
            }
        };

        if arr.len() != schema.items.len() {
            result.add_error(FormError::new(
                "INVALID_TUPLE",
                format!(
                    "tuple must have exactly {} items, got {}",
                    schema.items.len(),
                    arr.len()
                ),
                path,
            ));
        }

        for (idx, (item, item_schema)) in arr.iter_mut().zip(&schema.items).enumerate() {
            let item_path = format!("{}/{}", path, idx);
            let item_result = self.validate_type(item_schema, item, &item_path, ctx)?;
            result.merge(item_result);
        }

        Ok(result)
    }

    fn validate_union(
        &self,
        schema: &UnionSchema,
        value: &mut Value,
        path: &str,
        ctx: &mut EvalContext,
    ) -> Result<ValidationResult> {
        if value.is_null() {
            return Ok(ValidationResult::new());
        }

        // Try each variant until one succeeds
        for variant in &schema.variants {
            // Clone the value for non-destructive validation attempt
            let mut test_value = value.clone();
            let result = self.validate_type(variant, &mut test_value, path, ctx)?;
            if result.is_valid() {
                // This variant matched - apply transforms to original value
                *value = test_value;
                return Ok(result);
            }
        }

        // No variant matched
        let mut result = ValidationResult::new();
        result.add_error(FormError::new(
            "INVALID_UNION",
            "value does not match any variant in union",
            path,
        ));
        Ok(result)
    }

    fn validate_intersection(
        &self,
        schema: &IntersectionSchema,
        value: &mut Value,
        path: &str,
        ctx: &mut EvalContext,
    ) -> Result<ValidationResult> {
        let mut result = ValidationResult::new();
        let mut current = value.clone();

        for variant in &schema.variants {
            let variant_result = self.validate_type(variant, &mut current, path, ctx)?;
            result.merge(variant_result.clone());
            if !variant_result.is_valid() {
                return Ok(result);
            }
        }

        *value = current;
        Ok(result)
    }

    fn validate_record(
        &self,
        schema: &RecordSchema,
        value: &mut Value,
        path: &str,
        ctx: &mut EvalContext,
    ) -> Result<ValidationResult> {
        let mut result = ValidationResult::new();

        if value.is_null() {
            return Ok(result);
        }

        let obj = match value.as_object_mut() {
            Some(obj) => obj,
            None => {
                result.add_error(FormError::new(
                    "TYPE_ERROR",
                    format!("expected object (record), got {}", value_type_name(value)),
                    path,
                ));
                return Ok(result);
            }
        };

        for (key, item_value) in obj.iter_mut() {
            let item_path = if path.is_empty() {
                format!("/{}", escape_pointer_segment(key))
            } else {
                format!("{}/{}", path, escape_pointer_segment(key))
            };

            if let Some(key_schema) = &schema.key {
                let mut key_value = Value::String(key.clone());
                let key_path = format!("{}/$key", item_path);
                let key_result = self.validate_type(key_schema, &mut key_value, &key_path, ctx)?;
                result.merge(key_result);
            }

            let value_result = self.validate_type(&schema.value, item_value, &item_path, ctx)?;
            result.merge(value_result);
        }

        Ok(result)
    }

    fn validate_preprocess(
        &self,
        schema: &PreprocessSchema,
        value: &mut Value,
        path: &str,
        ctx: &mut EvalContext,
    ) -> Result<ValidationResult> {
        apply_transforms(value, &schema.transforms, path)?;
        self.validate_type(&schema.schema, value, path, ctx)
    }

    fn validate_catch(
        &self,
        schema: &CatchSchema,
        value: &mut Value,
        path: &str,
        ctx: &mut EvalContext,
    ) -> Result<ValidationResult> {
        let mut test_value = value.clone();
        let result = self.validate_type(&schema.schema, &mut test_value, path, ctx)?;
        if result.is_valid() {
            *value = test_value;
            return Ok(result);
        }

        *value = schema.value.clone();
        Ok(ValidationResult::new())
    }

    fn validate_ref(
        &self,
        name: &str,
        value: &mut Value,
        path: &str,
        ctx: &mut EvalContext,
    ) -> Result<ValidationResult> {
        self.validate_ref_inner(name, value, path, ctx, &mut Vec::new())
    }

    fn validate_ref_inner(
        &self,
        name: &str,
        value: &mut Value,
        path: &str,
        ctx: &mut EvalContext,
        seen_refs: &mut Vec<String>,
    ) -> Result<ValidationResult> {
        if seen_refs.iter().any(|seen| seen == name) {
            let mut cycle = seen_refs.clone();
            cycle.push(name.to_string());
            let mut result = ValidationResult::new();
            result.add_error(FormError::new(
                "REF_CYCLE",
                format!("schema reference cycle detected: {}", cycle.join(" -> ")),
                path,
            ));
            return Ok(result);
        }

        match self.schema.resolve_ref(name) {
            Some(TypeSchema::Ref { name: next }) => {
                seen_refs.push(name.to_string());
                let result = self.validate_ref_inner(next, value, path, ctx, seen_refs);
                seen_refs.pop();
                result
            }
            Some(type_schema) => self.validate_type(type_schema, value, path, ctx),
            None => {
                // Match the TypeScript runtime: an unresolvable $ref is an
                // in-band validation error, not a hard Err that aborts the call.
                let mut result = ValidationResult::new();
                result.add_error(FormError::new(
                    "REF_NOT_FOUND",
                    format!("schema reference not found: {name}"),
                    path,
                ));
                Ok(result)
            }
        }
    }

    fn schema_fills_missing(&self, schema: &TypeSchema) -> bool {
        self.schema_fills_missing_inner(schema, &mut Vec::new())
    }

    fn schema_fills_missing_inner(&self, schema: &TypeSchema, seen_refs: &mut Vec<String>) -> bool {
        if schema_has_default_transform(schema) {
            return true;
        }

        match schema {
            TypeSchema::Preprocess(preprocess) => {
                self.schema_fills_missing_inner(&preprocess.schema, seen_refs)
            }
            TypeSchema::Catch(catch) => self.schema_fills_missing_inner(&catch.schema, seen_refs),
            TypeSchema::Ref { name } => {
                if seen_refs.iter().any(|seen| seen == name) {
                    return false;
                }

                let Some(type_schema) = self.schema.resolve_ref(name) else {
                    return false;
                };

                seen_refs.push(name.clone());
                let fills = self.schema_fills_missing_inner(type_schema, seen_refs);
                seen_refs.pop();
                fills
            }
            _ => false,
        }
    }

    fn schema_allows_null(&self, schema: &TypeSchema) -> bool {
        self.schema_allows_null_inner(schema, &mut Vec::new())
    }

    fn schema_allows_null_inner(&self, schema: &TypeSchema, seen_refs: &mut Vec<String>) -> bool {
        if schema_has_default_transform(schema) {
            return true;
        }

        match schema {
            TypeSchema::Literal(literal) => literal.value.is_null(),
            TypeSchema::Enum(enumeration) => enumeration.values.iter().any(Value::is_null),
            TypeSchema::Union(union) => union
                .variants
                .iter()
                .any(|variant| self.schema_allows_null_inner(variant, seen_refs)),
            TypeSchema::Preprocess(preprocess) => {
                self.schema_allows_null_inner(&preprocess.schema, seen_refs)
            }
            TypeSchema::Catch(catch) => self.schema_allows_null_inner(&catch.schema, seen_refs),
            TypeSchema::Ref { name } => {
                if seen_refs.iter().any(|seen| seen == name) {
                    return false;
                }

                let Some(type_schema) = self.schema.resolve_ref(name) else {
                    return false;
                };

                seen_refs.push(name.clone());
                let allows = self.schema_allows_null_inner(type_schema, seen_refs);
                seen_refs.pop();
                allows
            }
            TypeSchema::Any(_) => true,
            _ => false,
        }
    }

    fn evaluate_constraints(
        &self,
        constraints: &[Constraint],
        value: &Value,
        path: &str,
        ctx: &mut EvalContext,
        result: &mut ValidationResult,
    ) -> Result<()> {
        for constraint in constraints {
            let passed = eval_predicate(&constraint.pred, value, ctx)?;
            if !passed {
                let error = FormError::from_meta(&constraint.error, path);
                result.add_error(error);
            }
        }
        Ok(())
    }
}

fn schema_has_default_transform(schema: &TypeSchema) -> bool {
    match schema {
        TypeSchema::String(schema) => has_default_transform(&schema.transforms),
        TypeSchema::Number(schema) => has_default_transform(&schema.transforms),
        TypeSchema::Integer(schema) => has_default_transform(&schema.transforms),
        TypeSchema::Int32(schema) => has_default_transform(&schema.transforms),
        TypeSchema::Int64(schema) => has_default_transform(&schema.transforms),
        TypeSchema::Uint32(schema) => has_default_transform(&schema.transforms),
        TypeSchema::Uint64(schema) => has_default_transform(&schema.transforms),
        TypeSchema::Money(schema) => has_default_transform(&schema.transforms),
        TypeSchema::Currency(schema) => has_default_transform(&schema.transforms),
        TypeSchema::Decimal(schema) => has_default_transform(&schema.transforms),
        TypeSchema::Percentage(schema) => has_default_transform(&schema.transforms),
        TypeSchema::Date(schema) => has_default_transform(&schema.transforms),
        TypeSchema::Preprocess(schema) => has_default_transform(&schema.transforms),
        _ => false,
    }
}

fn has_default_transform(transforms: &[Transform]) -> bool {
    transforms
        .iter()
        .any(|transform| matches!(transform, Transform::Default { .. }))
}

fn has_min_len_constraint(constraints: &[Constraint], len: usize) -> bool {
    constraints
        .iter()
        .any(|constraint| matches!(&constraint.pred, Predicate::MinLen { len: n } if *n == len))
}

fn has_max_len_constraint(constraints: &[Constraint], len: usize) -> bool {
    constraints
        .iter()
        .any(|constraint| matches!(&constraint.pred, Predicate::MaxLen { len: n } if *n == len))
}

/// Get a human-readable name for a JSON value type.
fn value_type_name(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

/// Escape a JSON Pointer segment.
fn escape_pointer_segment(s: &str) -> String {
    s.replace('~', "~0").replace('/', "~1")
}

/// Value equality matching the TypeScript runtime's `isEqualValue`: numbers
/// compare by numeric value (so `1 == 1.0`), everything else is deep structural
/// equality. serde_json's derived `PartialEq` is variant-sensitive for numbers
/// (`Number(Int(1)) != Number(Float(1.0))`), which diverges from JS `===`; this
/// restores cross-runtime parity for enum/literal matching.
/// See conformance: enum-number-int-vs-float, literal-number-int-vs-float.
pub(crate) fn json_value_eq(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Number(x), Value::Number(y)) => match (x.as_f64(), y.as_f64()) {
            (Some(xf), Some(yf)) => xf == yf,
            _ => x == y,
        },
        (Value::Array(xs), Value::Array(ys)) => {
            xs.len() == ys.len() && xs.iter().zip(ys).all(|(x, y)| json_value_eq(x, y))
        }
        (Value::Object(xo), Value::Object(yo)) => {
            xo.len() == yo.len()
                && xo
                    .iter()
                    .all(|(k, x)| yo.get(k).is_some_and(|y| json_value_eq(x, y)))
        }
        _ => a == b,
    }
}

/// Convenience function to validate a value against a schema.
///
/// Uses the global static registry for zero-allocation predicate lookups.
pub fn validate(schema: &Schema, value: &mut Value) -> Result<ValidationResult> {
    // Use the static registry (initialized once via LazyLock)
    use super::predicate::REGISTRY;
    let validator = Validator::new(schema, &REGISTRY);
    validator.validate(value)
}

/// Convenience function to validate with a custom registry.
pub fn validate_with_registry(
    schema: &Schema,
    value: &mut Value,
    registry: &PredicateRegistry,
) -> Result<ValidationResult> {
    let validator = Validator::new(schema, registry);
    validator.validate(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{ErrorMeta, ErrorSeverity, ObjectSchema, Predicate, StringSchema, Transform};
    use serde_json::json;

    #[test]
    fn test_validate_string() {
        let schema = Schema::new(
            "1.0.0",
            TypeSchema::String(StringSchema::new().transform(Transform::trim()).constraint(
                Constraint::new(
                    Predicate::min_len(1),
                    ErrorMeta::new("REQUIRED", "value is required"),
                ),
            )),
        );

        let mut value = json!("  hello  ");
        let result = validate(&schema, &mut value).unwrap();
        assert!(result.is_valid());
        assert_eq!(value, json!("hello")); // Trimmed

        let mut value = json!("   ");
        let result = validate(&schema, &mut value).unwrap();
        assert!(!result.is_valid());
        assert_eq!(result.errors[0].code, "REQUIRED");
    }

    #[test]
    fn test_validate_object() {
        let schema = Schema::new(
            "1.0.0",
            TypeSchema::Object(
                ObjectSchema::new()
                    .property(
                        "name",
                        TypeSchema::String(StringSchema::new().transform(Transform::trim())),
                    )
                    .optional_property("age", TypeSchema::integer()),
            ),
        );

        let mut value = json!({"name": "  Alice  ", "age": 30});
        let result = validate(&schema, &mut value).unwrap();
        assert!(result.is_valid());
        assert_eq!(value["name"], json!("Alice"));

        let mut value = json!({"age": 30});
        let result = validate(&schema, &mut value).unwrap();
        assert!(!result.is_valid());
        assert_eq!(result.errors[0].code, "REQUIRED");
    }

    #[test]
    fn test_validate_object_required_nullability() {
        let non_nullable = Schema::new(
            "1.0.0",
            TypeSchema::Object(ObjectSchema::new().property("name", TypeSchema::string())),
        );

        let mut value = json!({"name": null});
        let result = validate(&non_nullable, &mut value).unwrap();
        assert!(!result.is_valid());
        assert_eq!(result.errors[0].code, "REQUIRED");
        assert_eq!(result.errors[0].path, "/name");

        let nullable = Schema::new(
            "1.0.0",
            TypeSchema::Object(
                ObjectSchema::new().property("bio", TypeSchema::nullable(TypeSchema::string())),
            ),
        );

        let mut value = json!({"bio": null});
        let result = validate(&nullable, &mut value).unwrap();
        assert!(result.is_valid());
        assert_eq!(value, json!({"bio": null}));
    }

    #[test]
    fn test_validate_object_materializes_default_for_missing_field() {
        let schema = Schema::new(
            "1.0.0",
            TypeSchema::Object(ObjectSchema::new().property(
                "name",
                TypeSchema::String(
                    StringSchema::new().transform(Transform::default_value(json!("Anonymous"))),
                ),
            )),
        );

        let mut value = json!({});
        let result = validate(&schema, &mut value).unwrap();
        assert!(result.is_valid());
        assert_eq!(value, json!({"name": "Anonymous"}));
    }

    #[test]
    fn test_validate_object_unknown_keys_modes() {
        let strict_schema = Schema::new(
            "1.0.0",
            TypeSchema::Object(ObjectSchema::new().property("name", TypeSchema::string())),
        );

        let mut strict_value = json!({"name": "Alice", "extra": "x"});
        let strict_result = validate(&strict_schema, &mut strict_value).unwrap();
        assert!(!strict_result.is_valid());
        assert_eq!(
            strict_result.errors[0].code,
            "ADDITIONAL_PROPERTY_NOT_ALLOWED"
        );

        let passthrough_schema = Schema::new(
            "1.0.0",
            TypeSchema::Object(
                ObjectSchema::new()
                    .property("name", TypeSchema::string())
                    .passthrough(),
            ),
        );

        let mut passthrough_value = json!({"name": "Alice", "extra": "x"});
        let passthrough_result = validate(&passthrough_schema, &mut passthrough_value).unwrap();
        assert!(passthrough_result.is_valid());
        assert_eq!(passthrough_value["extra"], json!("x"));

        let strip_schema = Schema::new(
            "1.0.0",
            TypeSchema::Object(
                ObjectSchema::new()
                    .property("name", TypeSchema::string())
                    .strip(),
            ),
        );

        let mut strip_value = json!({"name": "Alice", "extra": "x"});
        let strip_result = validate(&strip_schema, &mut strip_value).unwrap();
        assert!(strip_result.is_valid());
        assert!(strip_value.get("extra").is_none());
    }

    #[test]
    fn test_validate_object_catchall() {
        let schema = Schema::new(
            "1.0.0",
            TypeSchema::Object(
                ObjectSchema::new()
                    .property("name", TypeSchema::string())
                    .catchall(TypeSchema::integer()),
            ),
        );

        let mut valid = json!({"name": "Alice", "age": 30});
        let result = validate(&schema, &mut valid).unwrap();
        assert!(result.is_valid());

        let mut invalid = json!({"name": "Alice", "age": "thirty"});
        let result = validate(&schema, &mut invalid).unwrap();
        assert!(!result.is_valid());
        assert_eq!(result.errors[0].path, "/age");
    }

    #[test]
    fn test_validate_array() {
        let schema = Schema::new(
            "1.0.0",
            TypeSchema::Array(
                ArraySchema::new(TypeSchema::string())
                    .min_items(1)
                    .max_items(3),
            ),
        );

        let mut value = json!(["a", "b"]);
        let result = validate(&schema, &mut value).unwrap();
        assert!(result.is_valid());

        let mut value = json!([]);
        let result = validate(&schema, &mut value).unwrap();
        assert!(!result.is_valid());
        assert_eq!(result.errors[0].code, "ARRAY_TOO_SHORT");

        let mut value = json!(["a", "b", "c", "d"]);
        let result = validate(&schema, &mut value).unwrap();
        assert!(!result.is_valid());
        assert_eq!(result.errors[0].code, "ARRAY_TOO_LONG");
    }

    #[test]
    fn test_validate_array_deduplicates_first_class_length_constraints() {
        let mut array = ArraySchema::new(TypeSchema::string())
            .min_items(1)
            .max_items(3);
        array.constraints.push(Constraint::new(
            Predicate::min_len(1),
            ErrorMeta::new("TOO_FEW_ITEMS", "must have at least 1 item"),
        ));
        array.constraints.push(Constraint::new(
            Predicate::max_len(3),
            ErrorMeta::new("TOO_MANY_ITEMS", "must have at most 3 items"),
        ));

        let schema = Schema::new("1.0.0", TypeSchema::Array(array));

        let mut value = json!([]);
        let result = validate(&schema, &mut value).unwrap();
        assert!(!result.is_valid());
        assert_eq!(result.errors.len(), 1);
        assert_eq!(result.errors[0].code, "TOO_FEW_ITEMS");

        let mut value = json!(["a", "b", "c", "d"]);
        let result = validate(&schema, &mut value).unwrap();
        assert!(!result.is_valid());
        assert_eq!(result.errors.len(), 1);
        assert_eq!(result.errors[0].code, "TOO_MANY_ITEMS");
    }

    #[test]
    fn test_validate_enum() {
        let schema = Schema::new("1.0.0", TypeSchema::enum_values(&["red", "green", "blue"]));

        let mut value = json!("red");
        let result = validate(&schema, &mut value).unwrap();
        assert!(result.is_valid());

        let mut value = json!("yellow");
        let result = validate(&schema, &mut value).unwrap();
        assert!(!result.is_valid());
        assert_eq!(result.errors[0].code, "INVALID_ENUM");
    }

    #[test]
    fn test_validate_literal() {
        let schema = Schema::new("1.0.0", TypeSchema::literal(json!("active")));

        let mut value = json!("active");
        let result = validate(&schema, &mut value).unwrap();
        assert!(result.is_valid());

        let mut value = json!("inactive");
        let result = validate(&schema, &mut value).unwrap();
        assert!(!result.is_valid());
        assert_eq!(result.errors[0].code, "INVALID_LITERAL");
    }

    #[test]
    fn test_validate_never() {
        let schema = Schema::new("1.0.0", TypeSchema::never());

        let mut value = json!("anything");
        let result = validate(&schema, &mut value).unwrap();
        assert!(!result.is_valid());
        assert_eq!(result.errors[0].code, "TYPE_ERROR");
    }

    #[test]
    fn test_validate_tuple() {
        let schema = Schema::new(
            "1.0.0",
            TypeSchema::tuple(vec![TypeSchema::string(), TypeSchema::integer()]),
        );

        let mut value = json!(["name", 10]);
        let result = validate(&schema, &mut value).unwrap();
        assert!(result.is_valid());

        let mut value = json!(["name"]);
        let result = validate(&schema, &mut value).unwrap();
        assert!(!result.is_valid());
        assert_eq!(result.errors[0].code, "INVALID_TUPLE");

        let mut value = json!(["name", "bad"]);
        let result = validate(&schema, &mut value).unwrap();
        assert!(!result.is_valid());
        assert_eq!(result.errors[0].path, "/1");
    }

    #[test]
    fn test_validate_intersection() {
        let schema = Schema::new(
            "1.0.0",
            TypeSchema::intersection(vec![
                TypeSchema::String(StringSchema::new().constraint(Constraint::new(
                    Predicate::min_len(2),
                    ErrorMeta::new("TOO_SHORT", "must be at least 2 chars"),
                ))),
                TypeSchema::String(StringSchema::new().constraint(Constraint::new(
                    Predicate::max_len(5),
                    ErrorMeta::new("TOO_LONG", "must be at most 5 chars"),
                ))),
            ]),
        );

        let mut valid = json!("abcd");
        let result = validate(&schema, &mut valid).unwrap();
        assert!(result.is_valid());

        let mut invalid = json!("a");
        let result = validate(&schema, &mut invalid).unwrap();
        assert!(!result.is_valid());
        assert_eq!(result.errors[0].code, "TOO_SHORT");
    }

    #[test]
    fn test_validate_record() {
        let schema = Schema::new(
            "1.0.0",
            TypeSchema::Record(
                RecordSchema::new(TypeSchema::integer()).with_key(TypeSchema::string()),
            ),
        );

        let mut valid = json!({"a": 1, "b": 2});
        let result = validate(&schema, &mut valid).unwrap();
        assert!(result.is_valid());

        let mut invalid = json!({"a": 1, "b": "bad"});
        let result = validate(&schema, &mut invalid).unwrap();
        assert!(!result.is_valid());
        assert_eq!(result.errors[0].path, "/b");
    }

    #[test]
    fn test_validate_preprocess_and_catch() {
        let preprocess_schema = Schema::new(
            "1.0.0",
            TypeSchema::preprocess(
                TypeSchema::string(),
                vec![Transform::trim(), Transform::upper()],
            ),
        );

        let mut value = json!("  abc ");
        let result = validate(&preprocess_schema, &mut value).unwrap();
        assert!(result.is_valid());
        assert_eq!(value, json!("ABC"));

        let catch_schema = Schema::new("1.0.0", TypeSchema::catch(TypeSchema::integer(), json!(0)));
        let mut invalid = json!("nope");
        let result = validate(&catch_schema, &mut invalid).unwrap();
        assert!(result.is_valid());
        assert_eq!(invalid, json!(0));
    }

    #[test]
    fn test_validate_cross_field_rule() {
        let schema = Schema::new(
            "1.0.0",
            TypeSchema::Object(
                ObjectSchema::new()
                    .property("is_foreign", TypeSchema::boolean())
                    .optional_property("zip", TypeSchema::string())
                    .rule(Constraint::new(
                        Predicate::implies(
                            Predicate::eq("/is_foreign", json!(false)),
                            Predicate::exists("/zip"),
                        ),
                        ErrorMeta::new("ZIP_REQUIRED", "ZIP is required for US addresses")
                            .with_path("/zip"),
                    )),
            ),
        );

        // US address with ZIP - valid
        let mut value = json!({"is_foreign": false, "zip": "12345"});
        let result = validate(&schema, &mut value).unwrap();
        assert!(result.is_valid());

        // US address without ZIP - invalid
        let mut value = json!({"is_foreign": false});
        let result = validate(&schema, &mut value).unwrap();
        assert!(!result.is_valid());
        assert_eq!(result.errors[0].code, "ZIP_REQUIRED");
        assert_eq!(result.errors[0].path, "/zip");

        // Foreign address without ZIP - valid
        let mut value = json!({"is_foreign": true});
        let result = validate(&schema, &mut value).unwrap();
        assert!(result.is_valid());
    }

    #[test]
    fn test_validate_with_ref() {
        let schema = Schema::new("1.0.0", TypeSchema::ref_to("Name")).define(
            "Name",
            TypeSchema::String(StringSchema::new().constraint(Constraint::new(
                Predicate::min_len(1),
                ErrorMeta::new("REQUIRED", "name is required"),
            ))),
        );

        let mut value = json!("Alice");
        let result = validate(&schema, &mut value).unwrap();
        assert!(result.is_valid());

        let mut value = json!("");
        let result = validate(&schema, &mut value).unwrap();
        assert!(!result.is_valid());
    }

    #[test]
    fn test_validate_named_predicate() {
        let schema = Schema::new(
            "1.0.0",
            TypeSchema::String(StringSchema::new().constraint(Constraint::new(
                Predicate::call("is_ssn", json!(null)),
                ErrorMeta::new("INVALID_SSN", "must be a valid SSN"),
            ))),
        );

        let mut value = json!("123-45-6789");
        let result = validate(&schema, &mut value).unwrap();
        assert!(result.is_valid());

        let mut value = json!("000-00-0000");
        let result = validate(&schema, &mut value).unwrap();
        assert!(!result.is_valid());
        assert_eq!(result.errors[0].code, "INVALID_SSN");
    }

    #[test]
    fn test_validation_result_warnings() {
        let schema = Schema::new(
            "1.0.0",
            TypeSchema::String(StringSchema::new().constraint(Constraint::new(
                Predicate::max_len(10),
                ErrorMeta::new("TOO_LONG", "value is quite long").warning(),
            ))),
        );

        let mut value = json!("this is a very long string");
        let result = validate(&schema, &mut value).unwrap();
        assert!(result.is_valid()); // Warnings don't fail validation
        assert_eq!(result.warnings.len(), 1);
        assert_eq!(result.warnings[0].code, "TOO_LONG");
        assert_eq!(result.warnings[0].severity, ErrorSeverity::Warning);
    }
}

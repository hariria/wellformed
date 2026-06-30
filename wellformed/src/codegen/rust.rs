//! Rust code generation from wellformed schemas.

use crate::ir::{ErrorMeta, ObjectSchema, Predicate, Schema, Transform, TypeSchema};

use super::api::{generate_api, generate_openapi_spec};
use super::api_types::generate_shared_types;
use super::util::{derive_struct_name, escape_keyword, to_pascal_case, to_snake_case};

/// Options for code generation.
#[derive(Debug, Clone)]
pub struct CodegenOptions {
    /// Generate Axum API handlers, DTOs, repository traits, and OpenAPI constants.
    ///
    /// This is disabled by default so type generation only requires serde and
    /// wellformed in the consuming crate. Enable it when the consuming app also
    /// provides the web/runtime dependencies used by generated handlers.
    pub generate_api: bool,
    /// Generate PDF render handlers when API generation is enabled and the schema has PDF metadata.
    ///
    /// This is disabled by default because PDF handlers reference application
    /// rendering modules and PDF-filler dependencies that are not part of the
    /// core `wellformed` crates.
    pub generate_pdf_handlers: bool,
    /// Route path prefix for registering handlers (relative to nested router).
    /// Default: "/forms" - routes are registered at /forms/1099/int etc.
    pub route_prefix: String,
    /// OpenAPI spec path prefix (full external path as seen by API clients).
    /// Default: "/api/v1/forms" - spec shows /api/v1/forms/1099/int etc.
    pub openapi_prefix: String,
    /// Base path for PDF templates (used to resolve include_bytes paths).
    /// If set, PDF filenames will be prefixed with this path.
    pub pdf_base_path: Option<String>,
}

impl Default for CodegenOptions {
    fn default() -> Self {
        Self {
            generate_api: false,
            generate_pdf_handlers: false,
            // Route prefix is relative to nested router
            // If using .nest("/api/v1", ...), routes go at /forms/...
            route_prefix: "/forms".to_string(),
            // OpenAPI prefix shows the full external path
            openapi_prefix: "/api/v1/forms".to_string(),
            pdf_base_path: None,
        }
    }
}

/// Generated Rust code output.
pub struct GeneratedCode {
    /// The complete generated Rust code as a string.
    pub code: String,
}

/// Generate all Rust code from a wellformed schema.
///
/// This includes:
/// - Struct definitions for all types
/// - validate() method implementation
/// - API handlers, DTOs, and a clean OpenAPI spec when `options.generate_api` is true
pub fn generate_all(schema: &Schema, schema_json: &str, options: &CodegenOptions) -> GeneratedCode {
    let mut code = String::new();

    // Header
    code.push_str("// Generated from wellformed schema - DO NOT EDIT\n\n");

    // Imports
    code.push_str("use serde::{Deserialize, Serialize};\n");
    code.push_str("use wellformed::{Schema, ValidationResult};\n");

    // Additional imports for API generation
    if options.generate_api {
        code.push_str("\n// API imports\n");
        code.push_str("use std::future::Future;\n");
        code.push_str("use axum::{\n");
        code.push_str("    extract::{Json, Path, Query, State},\n");
        code.push_str("    http::StatusCode,\n");
        code.push_str("    response::{IntoResponse, Response},\n");
        code.push_str("    routing::{delete, get, patch, post},\n");
        code.push_str("    Router,\n");
        code.push_str("};\n");
        code.push_str("use std::sync::Arc;\n");
    }

    code.push('\n');

    // Collect all struct definitions we need to generate
    let mut structs = Vec::new();

    // Generate structs for definitions
    for (name, type_schema) in &schema.definitions {
        if let TypeSchema::Object(obj) = type_schema {
            let struct_name = to_pascal_case(name);
            structs.push((struct_name.clone(), obj.clone(), false));
        }
    }

    // Generate main struct from root
    let root_name = schema
        .id
        .as_ref()
        .map(|id| derive_struct_name(id))
        .unwrap_or_else(|| "Root".to_string());

    if let TypeSchema::Object(obj) = &schema.root {
        structs.push((root_name.clone(), obj.clone(), true));
    }

    // Generate all collected structs
    for (name, obj, is_root) in &structs {
        code.push_str(&generate_struct(name, obj));
        code.push('\n');

        if *is_root {
            // Generate module-level schema constant for reuse by validate() and render_pdf()
            code.push_str(&generate_schema_constant(name, schema_json));
            code.push('\n');
            code.push_str(&generate_validate_impl(name, obj));
            code.push('\n');
            // Generate field name map for transforming validation error paths
            code.push_str(&generate_field_name_map(schema, name));
            code.push('\n');
        }
    }

    // Generate API code if enabled
    if options.generate_api {
        code.push('\n');
        code.push_str(&generate_shared_types());
        code.push('\n');
        code.push_str(&generate_api(schema, &options.route_prefix, options));
        code.push('\n');
        // Generate clean OpenAPI spec as a constant (uses full external path)
        let mut openapi_schema = schema.clone();
        if !options.generate_pdf_handlers {
            openapi_schema.pdf_template = None;
        }
        code.push_str(&generate_openapi_constant(
            &openapi_schema,
            &options.openapi_prefix,
        ));
    }

    GeneratedCode { code }
}

/// Generate a namespaced form module from a wellformed schema.
///
/// Instead of expanding generated structs as free-floating items, callers get
/// one stable module containing typed values, form errors, field metadata,
/// schema JSON, and validation helpers.
pub fn generate_form_module(
    schema: &Schema,
    schema_json: &str,
    module_name: &str,
    visibility: &str,
    runtime_path: Option<&str>,
    client_path: Option<&str>,
    options: &CodegenOptions,
) -> GeneratedCode {
    let root_name = schema
        .id
        .as_ref()
        .map(|id| derive_struct_name(id))
        .unwrap_or_else(|| "Root".to_string());
    let schema_const = format!("{}_SCHEMA_JSON", to_snake_case(&root_name).to_uppercase());
    let id = schema.id.as_deref().unwrap_or(module_name);
    let vis = visibility.trim();
    let vis_prefix = if vis.is_empty() {
        String::new()
    } else {
        format!("{vis} ")
    };

    let mut inner = generate_all(schema, schema_json, options).code;
    if let Some(runtime_path) = runtime_path {
        inner = format!(
            "use {runtime_path}::__private::regex;\nuse {runtime_path}::__private::serde;\nuse {runtime_path}::__private::serde_json;\nuse {runtime_path}::__private::wellformed;\nuse {runtime_path}::__private::wellformed_validate;\n\n{inner}",
        );
    }
    inner.push('\n');
    inner.push_str(&generate_form_field_metadata(schema, id));
    inner.push('\n');
    let client_module_name = format!("{}_client", to_snake_case(id));
    let client_module_const = if client_path.is_some() {
        format!("Some({})", rust_string_lit(&client_module_name))
    } else {
        "None".to_string()
    };
    inner.push_str(&format!(
        r#"
pub type Values = {root_name};
pub type Errors = wellformed::FormErrors;
pub type State = wellformed::FormState<Values>;

pub const ID: &str = {id_lit};
pub const TITLE: Option<&str> = {title_lit};
pub const DESCRIPTION: Option<&str> = {description_lit};
pub const SCHEMA_JSON: &str = {schema_const};

pub const CLIENT: wellformed::ClientFormSpec = wellformed::ClientFormSpec {{
    id: ID,
    title: TITLE,
    description: DESCRIPTION,
    schema_json: SCHEMA_JSON,
    fields: FIELDS,
    client_module: CLIENT_MODULE,
}};

pub const CLIENT_MODULE: Option<&str> = {client_module_const};

pub fn schema() -> wellformed::Schema {{
    serde_json::from_str(SCHEMA_JSON).expect("generated wellformed schema JSON should parse")
}}

pub fn validate(values: &Values) -> wellformed::ValidationResult {{
    values.validate()
}}

pub fn validate_value(mut value: serde_json::Value) -> Result<Values, Errors> {{
    let schema = schema();
    match wellformed::validate(&schema, &mut value) {{
        Ok(result) if result.is_valid() => serde_json::from_value(value.clone()).map_err(|err| {{
            Errors::from_message(value, "DESERIALIZE_FAILED", err.to_string())
        }}),
        Ok(result) => Err(Errors::from_validation(value, result)),
        Err(err) => Err(Errors::from_message(
            value,
            "VALIDATION_RUNTIME_ERROR",
            err.to_string(),
        )),
    }}
}}

pub fn validate_json(json: &str) -> Result<Values, Errors> {{
    match serde_json::from_str::<serde_json::Value>(json) {{
        Ok(value) => validate_value(value),
        Err(err) => Err(Errors::from_message(
            serde_json::Value::Null,
            "INVALID_JSON",
            err.to_string(),
        )),
    }}
}}

pub fn validate_form(value: serde_json::Value) -> Result<Values, Errors> {{
    validate_value(value)
}}

pub fn state(values: Values) -> State {{
    let submitted_values =
        serde_json::to_value(&values).unwrap_or_else(|_| serde_json::Value::Object(Default::default()));
    wellformed::FormState::new(values).with_submitted_values(submitted_values)
}}

pub fn state_with_errors(errors: Errors) -> State {{
    let submitted_values = errors.values.clone();
    let values = serde_json::from_value(errors.values.clone()).unwrap_or_default();
    wellformed::FormState::new(values)
        .with_submitted_values(submitted_values)
        .with_errors(errors)
}}
"#,
        root_name = root_name,
        id_lit = rust_string_lit(id),
        title_lit = rust_option_string_lit(schema.title.as_deref()),
        description_lit = rust_option_string_lit(schema.description.as_deref()),
        schema_const = schema_const,
        client_module_const = client_module_const,
    ));

    if let Some(client_path) = client_path {
        if let Some(client_module) =
            generate_client_helpers_module(schema, &client_module_name, client_path)
        {
            inner.push('\n');
            inner.push_str(&client_module);
        }
    }

    let code = format!(
        "// Generated from wellformed form schema - DO NOT EDIT\n\n{vis_prefix}mod {module_name} {{\n{inner}\n}}\n",
        inner = indent(&inner, 4),
    );

    GeneratedCode { code }
}

/// Generate the OpenAPI spec as a Rust constant.
fn generate_openapi_constant(schema: &Schema, api_prefix: &str) -> String {
    let root_name = schema
        .id
        .as_ref()
        .map(|id| derive_struct_name(id))
        .unwrap_or_else(|| "Root".to_string());
    let snake_name = to_snake_case(&root_name).to_uppercase();

    let openapi_json = generate_openapi_spec(schema, api_prefix);

    // Determine how many hash marks we need for the raw string literal
    let max_hashes = count_max_consecutive_hashes(&openapi_json);
    let delimiter_hashes = "#".repeat(max_hashes + 1);

    format!(
        r#"
// ============================================================================
// OpenAPI Spec
// ============================================================================

/// Clean OpenAPI spec for this form.
pub const {snake_name}_OPENAPI_JSON: &str = r{delim}"{json}"{delim};

/// Get the OpenAPI spec as a JSON value.
pub fn openapi_spec() -> serde_json::Value {{
    serde_json::from_str({snake_name}_OPENAPI_JSON).expect("valid OpenAPI JSON")
}}
"#,
        snake_name = snake_name,
        delim = delimiter_hashes,
        json = openapi_json
    )
}

/// Generate a Rust struct definition from an ObjectSchema.
fn generate_struct(name: &str, schema: &ObjectSchema) -> String {
    let mut code = String::new();

    // Doc comment
    if let Some(ref desc) = schema.description {
        code.push_str(&format!("/// {}\n", desc));
    }

    // Derives - simple serde only, no utoipa
    code.push_str("#[derive(Debug, Clone, Default, Serialize, Deserialize)]\n");
    code.push_str("#[serde(default)]\n");
    code.push_str(&format!("pub struct {} {{\n", name));

    for (field_name, prop) in &schema.fields {
        let rust_field = rust_field_name(field_name);
        let rust_type = type_to_rust(&prop.schema, prop.required);

        if rust_field != *field_name {
            code.push_str(&format!("    #[serde(rename = \"{}\")]\n", field_name));
        }
        code.push_str(&format!("    pub {}: {},\n", rust_field, rust_type));
    }

    code.push_str("}\n");
    code
}

/// Generate a module-level constant for the schema JSON.
fn generate_schema_constant(name: &str, schema_json: &str) -> String {
    let snake_name = to_snake_case(name).to_uppercase();

    // Count the maximum consecutive # characters in the JSON to determine
    // how many hash marks we need for the raw string literal
    let max_hashes = count_max_consecutive_hashes(schema_json);
    let delimiter_hashes = "#".repeat(max_hashes + 1);

    format!(
        "/// Embedded schema JSON for {}.\npub const {}_SCHEMA_JSON: &str = r{}\"{}\"{};",
        name, snake_name, delimiter_hashes, schema_json, delimiter_hashes
    )
}

/// Generate the validate() impl for the root struct using direct field access.
///
/// This generates type-safe validation that directly accesses struct fields
/// instead of serializing to JSON and walking the tree. This is 100-1000x faster.
fn generate_validate_impl(name: &str, schema: &ObjectSchema) -> String {
    let mut validations = Vec::new();

    for (field_key, prop) in &schema.fields {
        let rust_field = rust_field_name(field_key);
        let json_path = format!("/{}", field_key);

        // Generate validation for this field based on its type
        if let TypeSchema::String(string_schema) = &prop.schema {
            // For string fields, generate constraint checks
            for constraint in &string_schema.constraints {
                if let Some(code) = generate_constraint_check(
                    &rust_field,
                    &json_path,
                    &constraint.pred,
                    &constraint.error,
                ) {
                    validations.push(code);
                }
            }
        }
        // Direct codegen currently emits field-level checks for string constraints only.
        // Other schema types still appear in generated structs, but their validation
        // should be enforced by the runtime validator when full coverage is required.
    }

    let validation_code = validations.join("\n\n");

    format!(
        r##"impl {name} {{
    /// Validate this struct using direct field access.
    ///
    /// This is generated at compile time and directly accesses struct fields
    /// without serializing to JSON. ~100x faster than JSON-based validation.
    pub fn validate(&self) -> ValidationResult {{
        use wellformed::FormError;
        let mut result = ValidationResult::new();

{validation_code}

        result
    }}
}}
"##,
        name = name,
        validation_code = indent(&validation_code, 8),
    )
}

/// Generate code for a single constraint check.
fn generate_constraint_check(
    rust_field: &str,
    json_path: &str,
    pred: &Predicate,
    error: &ErrorMeta,
) -> Option<String> {
    let error_code = &error.code;
    let error_msg = &error.message;

    match pred {
        Predicate::MaxLen { len } => Some(format!(
            r#"// MaxLen constraint for {field}
if !self.{field}.is_empty() && self.{field}.chars().count() > {len} {{
    result.add_error(FormError::new("{code}", "{msg}", "{path}"));
}}"#,
            field = rust_field,
            len = len,
            code = error_code,
            msg = escape_string(error_msg),
            path = json_path,
        )),

        Predicate::MinLen { len } => Some(format!(
            r#"// MinLen constraint for {field}
if !self.{field}.is_empty() && self.{field}.chars().count() < {len} {{
    result.add_error(FormError::new("{code}", "{msg}", "{path}"));
}}"#,
            field = rust_field,
            len = len,
            code = error_code,
            msg = escape_string(error_msg),
            path = json_path,
        )),

        Predicate::Regex { pattern, flags } => {
            // Use static lazy regex for zero-allocation matching
            let flags_str = flags.as_deref().unwrap_or("");
            let case_insensitive = flags_str.contains('i');
            // Escape the pattern for use in a Rust string literal
            let escaped_pattern = pattern.replace('\\', "\\\\").replace('"', "\\\"");

            Some(format!(
                r#"// Regex constraint for {field}
if !self.{field}.is_empty() {{
    static RE: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {{
        regex::RegexBuilder::new("{pattern}")
            .case_insensitive({case_insensitive})
            .build()
            .expect("invalid regex pattern")
    }});
    if !RE.is_match(&self.{field}) {{
        result.add_error(FormError::new("{code}", "{msg}", "{path}"));
    }}
}}"#,
                field = rust_field,
                pattern = escaped_pattern,
                case_insensitive = case_insensitive,
                code = error_code,
                msg = escape_string(error_msg),
                path = json_path,
            ))
        }

        Predicate::Call { name, args: _ } => {
            // Map named predicates to wellformed_validate functions
            match name.as_str() {
                "is_tin" | "is_ssn" => Some(format!(
                    r#"// TIN validation for {field}
if !self.{field}.is_empty() && !wellformed_validate::validate_tin(&self.{field}) {{
    result.add_error(FormError::new("{code}", "{msg}", "{path}"));
}}"#,
                    field = rust_field,
                    code = error_code,
                    msg = escape_string(error_msg),
                    path = json_path,
                )),
                "is_ein" => Some(format!(
                    r#"// EIN validation for {field}
if !self.{field}.is_empty() && !wellformed_validate::validate_ein(&self.{field}) {{
    result.add_error(FormError::new("{code}", "{msg}", "{path}"));
}}"#,
                    field = rust_field,
                    code = error_code,
                    msg = escape_string(error_msg),
                    path = json_path,
                )),
                "is_us_zip" => Some(format!(
                    r#"// ZIP validation for {field}
if !self.{field}.is_empty() && !wellformed_validate::patterns::is_zip_format(&self.{field}) {{
    result.add_error(FormError::new("{code}", "{msg}", "{path}"));
}}"#,
                    field = rust_field,
                    code = error_code,
                    msg = escape_string(error_msg),
                    path = json_path,
                )),
                "is_email" => Some(format!(
                    r#"// Email validation for {field}
if !self.{field}.is_empty() && !wellformed_validate::patterns::is_email_format(&self.{field}) {{
    result.add_error(FormError::new("{code}", "{msg}", "{path}"));
}}"#,
                    field = rust_field,
                    code = error_code,
                    msg = escape_string(error_msg),
                    path = json_path,
                )),
                _ => None,
            }
        }

        // Skip predicates we can't generate direct code for yet
        _ => None,
    }
}

/// Escape a string for use in generated Rust code.
fn escape_string(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

/// Indent each line of a string by the given number of spaces.
fn indent(s: &str, spaces: usize) -> String {
    let prefix = " ".repeat(spaces);
    s.lines()
        .map(|line| {
            if line.is_empty() {
                String::new()
            } else {
                format!("{}{}", prefix, line)
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Generate the field name map constant that maps schema keys to Rust fields.
fn generate_field_name_map(schema: &Schema, name: &str) -> String {
    let snake_name = to_snake_case(name).to_uppercase();

    let mut mappings = Vec::new();
    if let TypeSchema::Object(obj) = &schema.root {
        for field_name in obj.fields.keys() {
            let rust_field = rust_field_name(field_name);

            if *field_name != rust_field {
                mappings.push(format!("        (\"{}\", \"{}\"),", field_name, rust_field));
            }
        }
    }

    let mappings_str = mappings.join("\n");

    format!(
        r##"
/// Field name mapping from schema keys to generated Rust field names.
/// Used to transform validation error paths for API responses.
pub static {snake_name}_FIELD_MAP: &[(&str, &str)] = &[
{mappings_str}
];

/// Get the field name map as a HashMap for efficient lookups.
pub fn field_name_map() -> std::collections::HashMap<&'static str, &'static str> {{
    {snake_name}_FIELD_MAP.iter().copied().collect()
}}
"##,
        snake_name = snake_name,
        mappings_str = mappings_str
    )
}

fn generate_form_field_metadata(schema: &Schema, form_id: &str) -> String {
    let mut code = String::new();
    let mut field_const_names = Vec::new();
    let mut field_accessors = String::new();

    if let TypeSchema::Object(obj) = &schema.root {
        for (field_name, prop) in &obj.fields {
            let const_name = field_const_name(field_name);
            let rust_name = rust_field_name(field_name);
            let error_id = format!("{form_id}-{field_name}-error");
            let accessor_name = format!("field_{rust_name}");

            code.push_str(&format!(
                r#"pub const {const_name}: wellformed::FieldSpec = wellformed::FieldSpec {{
    name: {name},
    rust_name: {rust_name},
    label: {label},
    description: {description},
    required: {required},
    kind: wellformed::FieldKind::{kind},
    section: {section},
    error_id: {error_id},
}};

"#,
                const_name = const_name,
                name = rust_string_lit(field_name),
                rust_name = rust_string_lit(&rust_name),
                label = rust_option_string_lit(prop.label.as_deref()),
                description = rust_option_string_lit(prop.description.as_deref()),
                required = prop.required,
                kind = field_kind_variant(&prop.schema),
                section = rust_option_string_lit(prop.section.as_deref()),
                error_id = rust_string_lit(&error_id),
            ));
            field_accessors.push_str(&format!(
                r#"pub fn {accessor_name}(state: &State) -> wellformed::FieldState<'_> {{
    state.field(&{const_name})
}}

"#,
                accessor_name = accessor_name,
                const_name = const_name,
            ));

            field_const_names.push(const_name);
        }
    }

    code.push_str("pub const FIELDS: &[wellformed::FieldSpec] = &[\n");
    for field_const_name in field_const_names {
        code.push_str("    ");
        code.push_str(&field_const_name);
        code.push_str(",\n");
    }
    code.push_str("];\n");
    code.push('\n');
    code.push_str(&field_accessors);
    code
}

fn generate_client_helpers_module(
    schema: &Schema,
    module_path: &str,
    client_path: &str,
) -> Option<String> {
    let TypeSchema::Object(obj) = &schema.root else {
        return None;
    };

    let mut helpers = String::new();
    for (field_name, prop) in &obj.fields {
        let TypeSchema::String(string_schema) = &prop.schema else {
            continue;
        };

        let rust_field = rust_field_name(field_name);
        let normalize_name = format!("normalize_{rust_field}");
        let error_name = format!("error_{rust_field}");
        let valid_name = format!("valid_{rust_field}");
        let normalized_expr = client_normalize_expr("value", &string_schema.transforms);
        let checks = client_error_checks("normalized", prop.required, string_schema, field_name);
        let valid_checks = client_valid_checks("normalized", prop.required, string_schema);

        helpers.push_str(&format!(
            r#"
    #[{client_path}::client]
    pub fn {normalize_name}(value: String) -> String {{
        {normalized_expr}
    }}

    #[{client_path}::client]
    pub fn {error_name}(value: String) -> &'static str {{
        let normalized = {normalize_name}(value);
{checks}
        ""
    }}

    #[{client_path}::client]
    pub fn {valid_name}(value: String) -> bool {{
        let normalized = {normalize_name}(value);
{valid_checks}
        true
    }}
"#,
            client_path = client_path,
            normalize_name = normalize_name,
            error_name = error_name,
            valid_name = valid_name,
            normalized_expr = normalized_expr,
            checks = indent(&checks, 8),
            valid_checks = indent(&valid_checks, 8),
        ));
    }

    if helpers.is_empty() {
        return None;
    }

    Some(format!(
        r#"
#[{client_path}::client_module(path = {module_path_lit})]
pub mod client {{
{helpers}
}}
"#,
        client_path = client_path,
        module_path_lit = rust_string_lit(module_path),
        helpers = helpers.trim_start(),
    ))
}

fn client_normalize_expr(value_ident: &str, transforms: &[Transform]) -> String {
    let mut expr = value_ident.to_string();
    for transform in transforms {
        match transform {
            Transform::Trim => expr = format!("{expr}.trim()"),
            Transform::Lower => expr = format!("{expr}.to_lowercase()"),
            Transform::Upper => expr = format!("{expr}.to_uppercase()"),
            _ => {}
        }
    }
    expr
}

fn client_error_checks(
    value_ident: &str,
    required: bool,
    schema: &crate::ir::StringSchema,
    field_name: &str,
) -> String {
    let mut checks = String::new();

    if required {
        checks.push_str(&format!(
            r#"if {value_ident}.is_empty() {{
    return {message};
}}
"#,
            value_ident = value_ident,
            message = rust_string_lit(&format!("required field '{}' is missing", field_name)),
        ));
    }

    for constraint in &schema.constraints {
        let message = rust_string_lit(&constraint.error.message);
        match &constraint.pred {
            Predicate::MinLen { len } => {
                checks.push_str(&format!(
                    r#"if {value_ident}.len() < {len} {{
    return {message};
}}
"#,
                    value_ident = value_ident,
                ));
            }
            Predicate::MaxLen { len } => {
                checks.push_str(&format!(
                    r#"if {value_ident}.len() > {len} {{
    return {message};
}}
"#,
                    value_ident = value_ident,
                ));
            }
            Predicate::Call { name, .. } if name == "is_email" => {
                checks.push_str(&format!(
                    r#"if !{value_ident}.contains("@") {{
    return {message};
}}
"#,
                    value_ident = value_ident,
                ));
            }
            _ => {}
        }
    }

    checks
}

fn client_valid_checks(
    value_ident: &str,
    required: bool,
    schema: &crate::ir::StringSchema,
) -> String {
    let mut checks = String::new();

    if required {
        checks.push_str(&format!(
            r#"if {value_ident}.is_empty() {{
    return false;
}}
"#,
            value_ident = value_ident,
        ));
    }

    for constraint in &schema.constraints {
        match &constraint.pred {
            Predicate::MinLen { len } => {
                checks.push_str(&format!(
                    r#"if {value_ident}.len() < {len} {{
    return false;
}}
"#,
                    value_ident = value_ident,
                ));
            }
            Predicate::MaxLen { len } => {
                checks.push_str(&format!(
                    r#"if {value_ident}.len() > {len} {{
    return false;
}}
"#,
                    value_ident = value_ident,
                ));
            }
            Predicate::Call { name, .. } if name == "is_email" => {
                checks.push_str(&format!(
                    r#"if !{value_ident}.contains("@") {{
    return false;
}}
"#,
                    value_ident = value_ident,
                ));
            }
            _ => {}
        }
    }

    checks
}

fn field_const_name(field_name: &str) -> String {
    let mut out = String::from("FIELD_");
    for ch in to_snake_case(field_name).chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_uppercase());
        } else {
            out.push('_');
        }
    }
    out
}

fn rust_field_name(field_name: &str) -> String {
    escape_keyword(&to_snake_case(field_name))
}

fn field_kind_variant(schema: &TypeSchema) -> &'static str {
    match schema {
        TypeSchema::String(_) => "String",
        TypeSchema::Number(_) => "Number",
        TypeSchema::Integer(_) | TypeSchema::Int32(_) | TypeSchema::Int64(_) => "Integer",
        TypeSchema::Uint32(_) | TypeSchema::Uint64(_) => "Integer",
        TypeSchema::Boolean(_) => "Boolean",
        TypeSchema::Money(_) => "Money",
        TypeSchema::Currency(_) => "Currency",
        TypeSchema::Decimal(_) => "Decimal",
        TypeSchema::Percentage(_) => "Percentage",
        TypeSchema::Date(_) => "Date",
        TypeSchema::Object(_) => "Object",
        TypeSchema::Array(_) => "Array",
        TypeSchema::Tuple(_) => "Tuple",
        TypeSchema::Enum(_) => "Enum",
        TypeSchema::Literal(_) => "Literal",
        TypeSchema::Ref { .. } => "Object",
        TypeSchema::Never(_)
        | TypeSchema::Union(_)
        | TypeSchema::Intersection(_)
        | TypeSchema::Record(_)
        | TypeSchema::Preprocess(_)
        | TypeSchema::Catch(_)
        | TypeSchema::Any(_) => "Json",
    }
}

fn rust_string_lit(value: &str) -> String {
    format!("{value:?}")
}

fn rust_option_string_lit(value: Option<&str>) -> String {
    match value {
        Some(value) => format!("Some({})", rust_string_lit(value)),
        None => "None".to_string(),
    }
}

/// Count the maximum number of consecutive '#' characters in a string.
fn count_max_consecutive_hashes(s: &str) -> usize {
    let mut max = 0;
    let mut current = 0;
    for c in s.chars() {
        if c == '#' {
            current += 1;
            max = max.max(current);
        } else {
            current = 0;
        }
    }
    max
}

/// Convert a TypeSchema to a Rust type string.
fn type_to_rust(schema: &TypeSchema, required: bool) -> String {
    let base_type = match schema {
        TypeSchema::String(_) => "String".to_string(),
        TypeSchema::Number(_) => "f64".to_string(),
        TypeSchema::Integer(_) => "i64".to_string(),
        TypeSchema::Int32(_) => "i32".to_string(),
        TypeSchema::Int64(_) => "i64".to_string(),
        TypeSchema::Uint32(_) => "u32".to_string(),
        TypeSchema::Uint64(_) => "u64".to_string(),
        TypeSchema::Boolean(_) => "bool".to_string(),
        TypeSchema::Money(_) => "rust_decimal::Decimal".to_string(),
        TypeSchema::Currency(_) => "rust_decimal::Decimal".to_string(),
        TypeSchema::Decimal(_) => "rust_decimal::Decimal".to_string(),
        TypeSchema::Percentage(_) => "f64".to_string(),
        TypeSchema::Date(_) => "chrono::NaiveDate".to_string(),
        TypeSchema::Array(arr) => {
            let item_type = type_to_rust(&arr.items, true);
            format!("Vec<{}>", item_type)
        }
        TypeSchema::Tuple(_) => "serde_json::Value".to_string(),
        TypeSchema::Object(_) => {
            // Named object definitions generate structs. Inline anonymous objects
            // intentionally stay as serde_json::Value until the schema gives them
            // stable names that can become Rust type identifiers.
            "serde_json::Value".to_string()
        }
        TypeSchema::Enum(_) => "String".to_string(),
        TypeSchema::Literal(_) => "serde_json::Value".to_string(),
        TypeSchema::Never(_) => "serde_json::Value".to_string(),
        TypeSchema::Union(_) => "serde_json::Value".to_string(),
        TypeSchema::Intersection(_) => "serde_json::Value".to_string(),
        TypeSchema::Record(_) => "serde_json::Value".to_string(),
        TypeSchema::Preprocess(_) => "serde_json::Value".to_string(),
        TypeSchema::Catch(_) => "serde_json::Value".to_string(),
        TypeSchema::Ref { name } => {
            // Reference to a named type
            to_pascal_case(name)
        }
        TypeSchema::Any(_) => "serde_json::Value".to_string(),
    };

    if required {
        base_type
    } else {
        format!("Option<{}>", base_type)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{ObjectSchema, PdfTemplate, PropertySchema, StringSchema};
    use indexmap::IndexMap;

    #[test]
    fn test_type_to_rust_primitives() {
        assert_eq!(
            type_to_rust(&TypeSchema::String(StringSchema::default()), true),
            "String"
        );
        assert_eq!(
            type_to_rust(&TypeSchema::String(StringSchema::default()), false),
            "Option<String>"
        );
    }

    #[test]
    fn test_generate_simple_struct() {
        let mut props = IndexMap::new();
        props.insert(
            "name".to_string(),
            PropertySchema {
                schema: TypeSchema::String(StringSchema::default()),
                required: true,
                description: Some("The name".to_string()),
                label: None,
                render: None,
                acroform: None,
                section: None,
            },
        );
        props.insert(
            "age".to_string(),
            PropertySchema {
                schema: TypeSchema::Integer(Default::default()),
                required: false,
                description: None,
                label: None,
                render: None,
                acroform: None,
                section: None,
            },
        );
        props.insert(
            "tin".to_string(),
            PropertySchema {
                schema: TypeSchema::String(StringSchema::default()),
                required: true,
                description: Some("Tax identifier".to_string()),
                label: Some("Tax ID".to_string()),
                render: None,
                acroform: None,
                section: None,
            },
        );

        let schema = ObjectSchema {
            fields: props,
            pages: IndexMap::new(),
            acroform_mappings: Vec::new(),
            additional_properties: false,
            unknown_keys: None,
            catchall: None,
            rules: Vec::new(),
            description: Some("A person".to_string()),
        };

        let code = generate_struct("Person", &schema);
        assert!(code.contains("pub struct Person"));
        assert!(code.contains("pub name: String"));
        assert!(code.contains("pub age: Option<i64>"));
        assert!(code.contains("pub tin: String"));
        assert!(!code.contains("pub tax_id: String"));
        assert!(code.contains("/// A person"));
        // No utoipa - using clean OpenAPI spec instead
        assert!(!code.contains("ToSchema"));
    }

    #[test]
    fn test_form_metadata_keeps_label_out_of_rust_name() {
        let mut props = IndexMap::new();
        props.insert(
            "tin".to_string(),
            PropertySchema {
                schema: TypeSchema::String(StringSchema::default()),
                required: true,
                description: Some("Tax identifier".to_string()),
                label: Some("Tax ID".to_string()),
                render: None,
                acroform: None,
                section: None,
            },
        );

        let schema = Schema::new(
            "1.0.0",
            TypeSchema::Object(ObjectSchema {
                fields: props,
                pages: IndexMap::new(),
                acroform_mappings: Vec::new(),
                additional_properties: false,
                unknown_keys: None,
                catchall: None,
                rules: Vec::new(),
                description: None,
            }),
        );

        let code = generate_form_field_metadata(&schema, "taxpayer");

        assert!(code.contains("name: \"tin\""));
        assert!(code.contains("rust_name: \"tin\""));
        assert!(code.contains("label: Some(\"Tax ID\")"));
        assert!(!code.contains("rust_name: \"tax_id\""));
    }

    fn test_schema_with_pdf_template() -> Schema {
        let mut props = IndexMap::new();
        props.insert(
            "name".to_string(),
            PropertySchema {
                schema: TypeSchema::String(StringSchema::default()),
                required: true,
                description: Some("The name".to_string()),
                label: None,
                render: None,
                acroform: None,
                section: None,
            },
        );

        let mut schema = Schema::new(
            "1.0.0",
            TypeSchema::Object(ObjectSchema {
                fields: props,
                pages: IndexMap::new(),
                acroform_mappings: Vec::new(),
                additional_properties: false,
                unknown_keys: None,
                catchall: None,
                rules: Vec::new(),
                description: Some("A person".to_string()),
            }),
        )
        .with_id("test_form");

        schema.pdf_template = Some(PdfTemplate {
            hash: None,
            filename: Some("test.pdf".to_string()),
            path: None,
            source_uri: None,
        });

        schema
    }

    #[test]
    fn test_generate_all_defaults_to_types_only() {
        let schema = test_schema_with_pdf_template();
        let code = generate_all(&schema, "{}", &CodegenOptions::default()).code;

        assert!(code.contains("pub struct TestForm"));
        assert!(code.contains("pub fn validate(&self)"));
        assert!(!code.contains("Router"));
        assert!(!code.contains("ApiError"));
        assert!(!code.contains("OPENAPI_JSON"));
    }

    #[test]
    fn test_generate_api_omits_pdf_handlers_by_default() {
        let schema = test_schema_with_pdf_template();
        let options = CodegenOptions {
            generate_api: true,
            ..CodegenOptions::default()
        };
        let code = generate_all(&schema, "{}", &options).code;

        assert!(code.contains("Router"));
        assert!(!code.contains("wireform_acroform"));
        assert!(!code.contains("render_test_form_pdf"));
    }

    #[test]
    fn test_generate_pdf_handlers_are_explicit() {
        let schema = test_schema_with_pdf_template();
        let options = CodegenOptions {
            generate_api: true,
            generate_pdf_handlers: true,
            ..CodegenOptions::default()
        };
        let code = generate_all(&schema, "{}", &options).code;

        assert!(code.contains("wireform_acroform"));
        assert!(code.contains("render_test_form_pdf"));
    }
}

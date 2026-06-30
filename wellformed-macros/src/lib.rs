//! Proc macros for generating Rust types from wellformed schemas.
//!
//! # Example
//!
//! ```ignore
//! use wellformed_macros::wellformed;
//!
//! // Load from a local path relative to CARGO_MANIFEST_DIR:
//! let signup = wellformed!("schemas/signup.json");
//! ```

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{quote, ToTokens};
use std::path::{Path, PathBuf};
use syn::parse::{Parse, ParseStream};
use syn::{parse_macro_input, Ident, LitStr, Token, Visibility};
use wellformed::codegen::{generate_all, generate_form_module, CodegenOptions, GeneratedCode};
use wellformed::Schema;

/// Input to the wel_schema! macro.
///
/// Supports two forms:
/// - `wel_schema!("path/to/schema.json")` - loads relative to CARGO_MANIFEST_DIR
/// - `wel_schema!(templates::form_id)` - loads from templates/src/form_id/schema.json
enum WelSchemaInput {
    /// templates::<form_id> or templates::<form_id>::SCHEMA_JSON
    Templates { form_id: Ident },
    /// Literal path string
    Path { path: LitStr },
}

/// Input to the form_schema! macro.
///
/// Syntax:
/// `form_schema!(pub mod signup = "schemas/signup.json");`
struct FormSchemaInput {
    visibility: Visibility,
    module_name: Ident,
    path: LitStr,
    runtime_path: Option<syn::Path>,
    client_path: Option<syn::Path>,
}

/// Input to the wellformed! macro.
///
/// - `wellformed!("path/to/schema.json")` embeds a schema value.
/// - `wellformed!()` embeds `schema.json` relative to CARGO_MANIFEST_DIR.
/// - `wellformed!(pub mod signup = "schemas/signup.json")` is accepted as a
///   compatibility alias for `form_schema!`.
enum WellformedInput {
    DefaultPath,
    Schema(WelSchemaInput),
    Module(FormSchemaInput),
}

impl Parse for WelSchemaInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Try to parse `templates::form_id` syntax
        if input.peek(Ident) {
            let first: Ident = input.parse()?;

            if first == "templates" {
                // Expect ::
                let _: Token![::] = input.parse()?;

                // Get form_id
                let form_id: Ident = input.parse()?;

                // Optionally consume ::SCHEMA_JSON if present
                if input.peek(Token![::]) {
                    let _: Token![::] = input.parse()?;
                    let _const_name: Ident = input.parse()?;
                    // We ignore the constant name - always use schema.json
                }

                return Ok(WelSchemaInput::Templates { form_id });
            } else {
                return Err(syn::Error::new(
                    first.span(),
                    "expected 'templates' or a string literal",
                ));
            }
        }

        // Otherwise parse as a path string
        let path: LitStr = input.parse()?;
        Ok(WelSchemaInput::Path { path })
    }
}

impl Parse for FormSchemaInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let visibility: Visibility = input.parse()?;
        let _: Token![mod] = input.parse()?;
        let module_name: Ident = input.parse()?;
        let _: Token![=] = input.parse()?;
        let path: LitStr = input.parse()?;
        let mut runtime_path = None;
        let mut client_path = None;
        while input.peek(Token![,]) {
            let _: Token![,] = input.parse()?;
            if input.is_empty() {
                break;
            }
            let key: Ident = input.parse()?;
            let _: Token![=] = input.parse()?;
            if key == "runtime" {
                runtime_path = Some(input.parse()?);
            } else if key == "client" {
                client_path = Some(input.parse()?);
            } else {
                return Err(syn::Error::new(
                    key.span(),
                    "expected `runtime = path` or `client = path`",
                ));
            }
        }

        if input.peek(Token![;]) {
            let _: Token![;] = input.parse()?;
        }

        Ok(Self {
            visibility,
            module_name,
            path,
            runtime_path,
            client_path,
        })
    }
}

impl Parse for WellformedInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.is_empty() {
            return Ok(Self::DefaultPath);
        }

        let fork = input.fork();
        let _visibility: Visibility = fork.parse()?;
        if fork.peek(Token![mod]) {
            return Ok(Self::Module(input.parse()?));
        }

        Ok(Self::Schema(input.parse()?))
    }
}

/// Generate Rust types from a wellformed schema file.
///
/// The macro reads the schema file at compile time and generates:
/// - Struct definitions with serde derives
/// - A `validate()` method that runs wellformed validation
///
/// For Axum handlers, repository traits, and OpenAPI constants, use the
/// lower-level `wellformed::codegen::generate_all` API with
/// `CodegenOptions { generate_api: true, .. }` so the consuming application can
/// explicitly provide the generated web dependencies. PDF render handlers also
/// require `generate_pdf_handlers: true` and app-level rendering dependencies.
///
/// # Examples
///
/// Load from a local path:
/// ```ignore
/// wel_schema!("schemas/signup.json");
/// ```
///
/// Load from a templates workspace:
/// ```ignore
/// wel_schema!(templates::signup);
/// // Or with explicit constant:
/// wel_schema!(templates::signup::SCHEMA_JSON);
/// ```
///
/// Use the generated types:
/// ```ignore
/// let form = Signup {
///     email: "ada@example.com".to_string(),
///     ..Default::default()
/// };
///
/// let result = form.validate();
/// ```
#[proc_macro]
pub fn wel_schema(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as WelSchemaInput);

    // Track the form directory for PDF path resolution (absolute path)
    let (full_path, form_dir, span): (PathBuf, Option<PathBuf>, Span) = match &input {
        WelSchemaInput::Templates { form_id } => {
            // Resolve path to templates/src/{form_id}/schema.json
            let manifest_dir =
                std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
            let manifest_path = PathBuf::from(&manifest_dir);

            // Walk up to find workspace root (look for templates directory)
            let templates_dir = find_templates_dir(&manifest_path);
            let (schema_path, form_directory) = match templates_dir {
                Some(dir) => {
                    let form_dir = dir.join("src").join(form_id.to_string());
                    let schema_path = form_dir.join("schema.json");
                    (schema_path, Some(form_dir))
                }
                None => {
                    let msg = format!(
                        "Could not find templates directory. Searched from '{}'",
                        manifest_path.display()
                    );
                    return syn::Error::new(form_id.span(), msg)
                        .to_compile_error()
                        .into();
                }
            };

            (schema_path, form_directory, form_id.span())
        }
        WelSchemaInput::Path { path } => {
            // Resolve path relative to CARGO_MANIFEST_DIR
            let manifest_dir =
                std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
            let full_path = PathBuf::from(&manifest_dir).join(path.value());
            // For path-based schemas, the form directory is the parent of the schema file
            let form_dir = full_path.parent().map(|p| p.to_path_buf());
            (full_path, form_dir, path.span())
        }
    };

    // Read and parse schema
    let content = match std::fs::read_to_string(&full_path) {
        Ok(c) => c,
        Err(e) => {
            let msg = format!(
                "Failed to read schema file '{}': {}",
                full_path.display(),
                e
            );
            return syn::Error::new(span, msg).to_compile_error().into();
        }
    };

    let schema: Schema = match serde_json::from_str(&content) {
        Ok(s) => s,
        Err(e) => {
            let msg = format!("Failed to parse schema JSON: {}", e);
            return syn::Error::new(span, msg).to_compile_error().into();
        }
    };

    // Generate code with options including PDF base path
    let options = CodegenOptions {
        pdf_base_path: form_dir.map(|p| p.to_string_lossy().to_string()),
        ..Default::default()
    };
    let GeneratedCode { code } = generate_all(&schema, &content, &options);

    // Parse generated code as TokenStream
    match code.parse() {
        Ok(ts) => ts,
        Err(e) => {
            let msg = format!("Failed to parse generated code: {}", e);
            syn::Error::new(span, msg).to_compile_error().into()
        }
    }
}

/// Embed a wellformed schema file as a value.
///
/// This is the preferred public macro for Rust applications that want a schema
/// value they can assign to a local, static, or const binding. Use
/// `form_schema!` when you want generated Rust types and a namespaced module
/// facade.
///
/// ```ignore
/// use wellformed_macros::wellformed;
///
/// let signup = wellformed!("schemas/signup.json");
///
/// let (result, value) = signup.validate_json(r#"{"email":"ada@example.com"}"#)?;
/// assert!(result.is_valid());
/// ```
#[proc_macro]
pub fn wellformed(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as WellformedInput);

    match input {
        WellformedInput::DefaultPath => expand_embedded_schema(None),
        WellformedInput::Schema(input) => expand_embedded_schema(Some(input)),
        WellformedInput::Module(input) => expand_form_schema(input),
    }
}

/// Generate a namespaced form module from a wellformed schema file.
///
/// Unlike `wel_schema!`, this macro does not emit free-floating generated
/// items. It creates a module with typed values, validation helpers, field
/// metadata, schema constants, and framework-neutral form state aliases.
///
/// ```ignore
/// use wellformed_macros::form_schema;
///
/// form_schema!(pub mod signup = "schemas/signup.json");
///
/// let values = signup::validate_json(r#"{"email":"ada@example.com"}"#)?;
/// let first_field = &signup::FIELDS[0];
/// ```
#[proc_macro]
pub fn form_schema(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as FormSchemaInput);
    expand_form_schema(input)
}

fn expand_embedded_schema(input: Option<WelSchemaInput>) -> TokenStream {
    let (full_path, span) = match input {
        None => {
            let manifest_dir =
                std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
            (
                PathBuf::from(manifest_dir).join("schema.json"),
                Span::call_site(),
            )
        }
        Some(WelSchemaInput::Path { path }) => {
            let manifest_dir =
                std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
            (PathBuf::from(manifest_dir).join(path.value()), path.span())
        }
        Some(WelSchemaInput::Templates { form_id }) => {
            let manifest_dir =
                std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
            let manifest_path = PathBuf::from(&manifest_dir);
            let templates_dir = match find_templates_dir(&manifest_path) {
                Some(dir) => dir,
                None => {
                    let msg = format!(
                        "Could not find templates directory. Searched from '{}'",
                        manifest_path.display()
                    );
                    return syn::Error::new(form_id.span(), msg)
                        .to_compile_error()
                        .into();
                }
            };
            (
                templates_dir
                    .join("src")
                    .join(form_id.to_string())
                    .join("schema.json"),
                form_id.span(),
            )
        }
    };

    let content = match std::fs::read_to_string(&full_path) {
        Ok(c) => c,
        Err(e) => {
            let msg = format!(
                "Failed to read schema file '{}': {}",
                full_path.display(),
                e
            );
            return syn::Error::new(span, msg).to_compile_error().into();
        }
    };

    if let Err(e) = serde_json::from_str::<Schema>(&content) {
        let msg = format!("Failed to parse schema JSON: {}", e);
        return syn::Error::new(span, msg).to_compile_error().into();
    }

    let schema_json = LitStr::new(&content, span);
    quote!(::wellformed::EmbeddedSchema::new(#schema_json)).into()
}

fn expand_form_schema(input: FormSchemaInput) -> TokenStream {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let full_path = PathBuf::from(&manifest_dir).join(input.path.value());

    let content = match std::fs::read_to_string(&full_path) {
        Ok(c) => c,
        Err(e) => {
            let msg = format!(
                "Failed to read schema file '{}': {}",
                full_path.display(),
                e
            );
            return syn::Error::new(input.path.span(), msg)
                .to_compile_error()
                .into();
        }
    };

    let schema: Schema = match serde_json::from_str(&content) {
        Ok(s) => s,
        Err(e) => {
            let msg = format!("Failed to parse schema JSON: {}", e);
            return syn::Error::new(input.path.span(), msg)
                .to_compile_error()
                .into();
        }
    };

    let options = CodegenOptions {
        pdf_base_path: full_path
            .parent()
            .map(|path| path.to_string_lossy().to_string()),
        ..Default::default()
    };
    let visibility = input.visibility.to_token_stream().to_string();
    let runtime_path = input
        .runtime_path
        .as_ref()
        .map(|path| path.to_token_stream().to_string().replace(' ', ""));
    let client_path = input
        .client_path
        .as_ref()
        .map(|path| path.to_token_stream().to_string().replace(' ', ""));
    let GeneratedCode { code } = generate_form_module(
        &schema,
        &content,
        &input.module_name.to_string(),
        &visibility,
        runtime_path.as_deref(),
        client_path.as_deref(),
        &options,
    );

    match code.parse() {
        Ok(ts) => ts,
        Err(e) => {
            let msg = format!("Failed to parse generated form module: {}", e);
            syn::Error::new(input.module_name.span(), msg)
                .to_compile_error()
                .into()
        }
    }
}

/// Find the templates directory by walking up from the given path.
fn find_templates_dir(start: &Path) -> Option<PathBuf> {
    let mut current = start.to_path_buf();

    // Walk up the directory tree
    for _ in 0..10 {
        // Check if templates directory exists here
        let templates = current.join("templates");
        if templates.is_dir() && templates.join("src").is_dir() {
            return Some(templates);
        }

        // Move up one directory
        if !current.pop() {
            break;
        }
    }

    None
}

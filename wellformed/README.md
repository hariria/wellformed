# wellformed

Rust runtime, IR types, form helpers, and code generation utilities for
wellformed schemas.

`wellformed` evaluates the same portable JSON intermediate representation (IR)
produced by the TypeScript builder package. Use it when schemas are authored in
TypeScript but enforcement needs to happen in a Rust service, worker, CLI, or
batch process.

## Install

After the crate is published:

```bash
cargo add wellformed serde_json
```

For local workspace development:

```toml
[dependencies]
wellformed = { path = "../wellformed" }
serde_json = "1"
```

The crate declares Rust 1.93.0 as its minimum supported Rust version.

## Validate JSON IR

```rust
use serde_json::json;
use wellformed::ir::{ObjectSchema, StringSchema};
use wellformed::{validate, Constraint, ErrorMeta, Predicate, Schema, Transform, TypeSchema};

fn main() -> wellformed::Result<()> {
    let schema = Schema::new(
        "1.0",
        TypeSchema::Object(
            ObjectSchema::new()
                .property(
                    "email",
                    TypeSchema::String(
                        StringSchema::new()
                            .transform(Transform::trim())
                            .constraint(Constraint::new(
                                Predicate::call("is_email", serde_json::Value::Null),
                                ErrorMeta::new("INVALID_EMAIL", "Enter a valid email address"),
                            )),
                    ),
                ),
        ),
    );

    let mut value = json!({ "email": " ada@example.com " });
    let result = validate(&schema, &mut value)?;

    assert!(result.is_valid());
    assert_eq!(value["email"], "ada@example.com");
    Ok(())
}
```

Validation mutates the input `serde_json::Value` in place when transforms run.
Keep a copy of the raw input if your application needs both raw and normalized
values.

## Custom Predicates

The default `validate` function uses the built-in predicate registry. Use
`validate_with_registry` when schemas contain organization-specific named
predicates. Custom predicates serialize by name and arguments only, so every
runtime that evaluates a schema must register equivalent implementations.

## Form Facades

The companion `wellformed-macros` crate provides `wellformed!` for embedding a
schema as a value, `form_schema!` for a namespaced form facade with typed
values, field metadata, validation helpers, and framework-neutral form state,
and `wel_schema!` when you prefer free-floating generated Rust types.

## Codegen Defaults

Rust codegen defaults to generated data types and validation helpers. API
handlers, repository traits, OpenAPI constants, and PDF render handlers are
explicit opt-ins because they require application-level dependencies and
integration choices.

## Optional Features

| Feature | Description |
|---------|-------------|
| `address` | Enables libpostal-backed address parsing predicates through the `postal` crate. Requires native libpostal headers, libraries, and parser data. |

The default feature set does not require native address-parsing libraries.

Some libpostal installs, including Homebrew on Apple Silicon, expose headers
through `pkg-config` but not through Clang's default include path. In that case,
build with:

```bash
export BINDGEN_EXTRA_CLANG_ARGS="$(pkg-config --cflags libpostal)"
export LIBRARY_PATH="$(pkg-config --variable=libdir libpostal):${LIBRARY_PATH:-}"
cargo check -p wellformed --features address
```

## License

MIT.

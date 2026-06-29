# wellformed-macros

Proc macros for generating Rust code from wellformed schema JSON.

This crate is intended for applications that want compile-time schema
embedding, generated Rust types, validation helpers, or a namespaced form facade
from a schema file. It builds on the `wellformed` runtime crate and expects
schema JSON in the wellformed IR format.

## Macros

### `wellformed!`

Embeds a schema JSON file as a `wellformed::EmbeddedSchema` value.

```rust
use wellformed_macros::wellformed;

const SIGNUP: wellformed::EmbeddedSchema = wellformed!("schemas/signup.json");

fn main() -> wellformed::Result<()> {
    let signup = wellformed!("schemas/signup.json");
    let (result, _value) = signup.validate_json(r#"{"email":"ada@example.com"}"#)?;

    assert!(result.is_valid());

    Ok(())
}
```

The path is resolved relative to `CARGO_MANIFEST_DIR`. `wellformed!()` defaults
to `schema.json`.

### `wel_schema!`

Generates Rust structs and validation methods from a schema file at compile
time.

```rust
use wellformed_macros::wel_schema;

wel_schema!("schemas/signup.json");
```

Use this when you prefer free-floating generated structs instead of a
namespaced module.

### `form_schema!`

Generates a namespaced module facade with typed values, field metadata, schema
constants, validation helpers, and framework-neutral form state types.

```rust
use wellformed_macros::form_schema;

form_schema!(pub mod signup = "schemas/signup.json");

let values = signup::validate_json(r#"{"email":"ada@example.com"}"#)?;
let state = signup::state(values);
let first_field = &signup::FIELDS[0];
```

The generated module exposes `Values`, `Errors`, `State`, `ID`, `TITLE`,
`DESCRIPTION`, `SCHEMA_JSON`, `FIELDS`, `CLIENT`, `schema()`, `validate()`,
`validate_value()`, `validate_json()`, `validate_form()`, `state()`, and
`state_with_errors()`.

## Codegen Defaults

Generated API handlers, repository traits, OpenAPI constants, and PDF render
handlers are not emitted by these macros by default. Use the lower-level
`wellformed::codegen::generate_all` API with explicit `CodegenOptions` when an
application wants those scaffolds and can provide the necessary dependencies.

## Requirements

- Rust 1.82 or newer
- `serde` / `serde_json` in the consuming crate when using generated serde
  types from `form_schema!` or `wel_schema!`

## License

MIT.

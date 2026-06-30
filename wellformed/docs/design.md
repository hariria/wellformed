# wellformed Design Notes

`wellformed` is a portable validation system for forms and structured data. The
core design goal is to make validation rules inspectable data instead of opaque
runtime code, so the same schema can be authored in TypeScript, stored as JSON,
reviewed in pull requests, and evaluated by TypeScript or Rust.

## Goals

- Keep schema authoring ergonomic with a TypeScript builder API.
- Serialize schemas into a deterministic JSON intermediate representation (IR).
- Validate the same IR in multiple runtimes without rewriting business rules.
- Preserve structured errors with stable codes, paths, severity, messages, and
  help text.
- Keep transforms, predicates, and cross-field rules portable and reviewable.
- Generate Rust types and form facades without forcing web-framework
  dependencies into applications that do not need them.

## Non-Goals

- Running arbitrary JavaScript validation closures in Rust.
- Treating display labels as stable schema keys or generated code identifiers.
- Generating application-specific API, persistence, or PDF infrastructure by
  default.
- Replacing application-level authorization, encryption, logging, retention, or
  compliance controls.

## Architecture

### Authoring Layer

Application code usually authors schemas with the TypeScript builder:

```ts
import { w } from "wellformed-ts/builder";

export const signup = w.object({
  email: w.string().trim().email(),
  name: w.string().trim().minLen(1),
}).strict();

export const signupSchema = signup.toSchema("1.0");
```

The builder records the type structure, transforms, constraints, and error
metadata directly as IR. Portable schemas do not capture arbitrary closures.

### Intermediate Representation

The IR is JSON with a small set of concepts:

- type schemas: strings, numbers, booleans, objects, arrays, tuples, records,
  enums, literals, unions, intersections, references, and domain primitives
- transforms: deterministic normalization steps such as `trim`, `digits_only`,
  `upper`, `date_parse`, and formatting/masking helpers
- predicates: declarative checks such as length, range, regex, boolean
  composition, field existence, field comparisons, sums, and named calls
- errors: stable structured metadata attached to constraints
- definitions: named schemas for reuse and reference resolution

The IR is the cross-runtime contract. Applications that persist schemas should
store their own schema id and migration version alongside the wellformed IR
version.

### Runtime Validation

Runtimes evaluate schemas by:

1. Walking the schema tree.
2. Applying transforms before constraints.
3. Evaluating built-in and registered named predicates.
4. Resolving `ref` schemas through top-level `definitions`.
5. Returning normalized output and structured validation errors.

Objects should choose unknown-key behavior explicitly with `strict`, `strip`,
`passthrough`, or `catchall`. External input should usually use `strict` or
`strip`.

### Named Predicates

Named predicates are extension points:

```json
{ "type": "call", "name": "is_internal_account" }
```

The name and arguments are serialized, not the implementation. Every runtime
that evaluates a schema with custom predicates must register equivalent
semantics. Built-in predicates should be preferred for common identifiers and
formats so schemas stay portable.

### Code Generation

Rust codegen defaults to generated data types and validation helpers. API
handlers, repository traits, OpenAPI constants, and PDF render handlers are
explicit opt-ins because those outputs require application-level dependencies
and integration choices.

`wellformed!` is for embedding a schema as a value. `form_schema!` is for a
namespaced form facade with typed values, field metadata, validation helpers,
serialized form state, and optional client helper hooks. `wel_schema!` is
available when callers prefer free-floating generated structs.

Generated Rust field identifiers are derived from schema keys. Labels remain
presentation metadata and must not rename Rust fields or API request members.

## Package Boundaries

- `wellformed`: Rust IR types, runtime validation, transforms, predicates,
  form helpers, and codegen.
- `wellformed-macros`: proc macros for schema-driven Rust code generation.
- `wellformed-validate`: lower-level validation primitives and batch helpers.
- `typescript/packages/wellformed`: TypeScript builder, IR types, runtime,
  serializer, form helpers, and npm package docs.
- `typescript/apps/docs`: public documentation, playground, and LLM text routes.

## Production Guidance

- Pin exact package and crate versions before 1.0.
- Treat error `code` values as application API; treat `message` as display copy.
- Store immutable schema versions for historical submissions.
- Validate loaded schemas at startup and fail closed on unresolved references or
  reference cycles.
- Keep compatibility tests that run representative serialized IR in every
  runtime deployed by the application.
- Avoid logging real sensitive values in tests, examples, issues, or diagnostic
  output.

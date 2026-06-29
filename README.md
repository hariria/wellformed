<div align="center">

<img src="assets/logo.svg" width="88" height="88" alt="wellformed" />

# wellformed

**Validation logic should be data.**

Author schemas once in TypeScript, compile them to a portable JSON IR, and run
the exact same rules in TypeScript and Rust.

[![npm](https://img.shields.io/npm/v/wellformed-ts?style=flat-square&logo=npm&color=76CE54&label=wellformed-ts)](https://www.npmjs.com/package/wellformed-ts)
[![crates.io](https://img.shields.io/crates/v/wellformed?style=flat-square&logo=rust&color=76CE54&label=wellformed)](https://crates.io/crates/wellformed)
[![CI](https://img.shields.io/github/actions/workflow/status/hariria/wellformed/ci.yml?style=flat-square&logo=github&label=CI)](https://github.com/hariria/wellformed/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-76CE54?style=flat-square)](LICENSE)
[![X](https://img.shields.io/badge/X-76CE54?style=flat-square&logo=x&logoColor=white)](https://x.com/drewhariri)

[**Docs**](https://wellformed.dev) · [**Playground**](https://wellformed.dev/playground) · [Getting Started](https://wellformed.dev/docs/getting-started) · [Comparison](https://wellformed.dev/docs/comparison)

</div>

---

## Why wellformed

- **One schema, every runtime.** Validation rules serialize to JSON instead of
  living in opaque JavaScript closures. The same IR validates identically in
  JavaScript, TypeScript, and Rust. No backend rewrite, no drift.
- **Zod-like authoring.** A familiar chained builder:
  `w.string().trim().minLen(1).email()`, with full type inference via
  `Infer<typeof schema>`.
- **60+ domain predicates.** SSN, EIN, ITIN, CUSIP, ABA routing, IBAN, ICD-10,
  dates, phones, and more. Purpose-built validators, not generic regex.
- **Cross-field rules as data.** Conditional requirements, mutual exclusion, and
  comparisons: `.when("kind").equals("business").require("ein")`.
- **Transform pipeline.** Normalize before you validate (trim, digits-only,
  money-to-cents, date parsing). Transforms travel inside the schema, so every
  runtime cleans data the same way.
- **Fast.** Built-in predicates validate in tens of nanoseconds, roughly 10 to
  40x faster than Zod in the same V8 runtime, with the Rust runtime faster
  still. [See the benchmark](https://wellformed.dev/docs/performance).

## Install

```bash
npm install wellformed-ts        # TypeScript / JavaScript
cargo add wellformed serde_json  # Rust
```

## Quick start

```ts
import { validate, w, type Infer } from "wellformed-ts";

const taxpayer = w
  .object({
    kind: w.enum(["individual", "business"] as const),
    name: w.string().trim().minLen(1),
    ssn: w.string().digitsOnly().ssn().optional(),
    ein: w.string().digitsOnly().ein().optional(),
  })
  .when("kind").equals("individual").require("ssn")
  .when("kind").equals("business").require("ein")
  .mutuallyExclusive("ssn", "ein");

type Taxpayer = Infer<typeof taxpayer>;

const result = validate(taxpayer.toSchema("1.0"), {
  kind: "individual",
  name: "  Ada Lovelace  ",
  ssn: "123-45-6789",
});

if (result.valid) {
  console.log((result.value as Taxpayer).name); // "Ada Lovelace" (trimmed)
}
```

## How it works

```
TypeScript builder  ──►  JSON IR  ──┬──►  TypeScript validate()
                                    └──►  Rust validate()
```

Schemas compile to a portable JSON intermediate representation. Store it in a
database, diff it in code review, send it over the wire, and evaluate it in any
runtime. The same JSON your frontend authored deserializes into the Rust
`wellformed` crate and validates with identical results.
[Read the IR reference](https://wellformed.dev/docs/ir-schema).

## Workspace layout

- `wellformed/` is the Rust core crate: IR types, runtime validation,
  transforms, predicates, and codegen.
- `wellformed-macros/` provides Rust proc macros for schema-driven code
  generation.
- `wellformed-validate/` holds low-level validation primitives and benchmark
  helpers.
- `typescript/packages/wellformed/` is the TypeScript builder API, IR types,
  runtime, forms helpers, and tests.
- `typescript/apps/docs/` is the documentation site and playground.

## Status

`wellformed` is preparing for its first public production users. Treat it as
pre-1.0: pin exact versions, read the changelog before upgrading, and expect
compatibility notes for public API changes. Do not put sensitive personal data
in public issues or examples; report vulnerabilities privately per
[`SECURITY.md`](SECURITY.md).

## Development

Prerequisites:

- Rust stable toolchain (published crates support a minimum of Rust 1.93.0)
- `cargo-audit` (`cargo install cargo-audit --locked`)
- Node.js 22 LTS (`nvm use`)
- pnpm 9+

Run the core checks:

```bash
bash scripts/release-preflight.sh
```

Run the docs site and playground locally:

```bash
cd typescript
pnpm --filter @wellformed/docs dev
```

## Production notes

- Prefer stable machine-readable error `code` values over parsing
  human-readable `message` text.
- Decide explicitly how objects handle unknown keys: `.strict()`, `.strip()`,
  `.passthrough()`, or `.catchall(...)`.
- Keep custom predicates registered consistently in every runtime that
  evaluates the same IR.
- Store serialized schemas with a version and migration metadata.
- In Rust, use `wellformed!` to embed a schema as a value for native
  validation, or `form_schema!` for a namespaced facade with typed values,
  field metadata, and framework-neutral `FormState`.

## AI-assisted integration

The TypeScript package ships `skills.md`, a compact agent guide for integrating
wellformed into applications. The docs site exposes
[`/llms-full.txt`](https://wellformed.dev/llms-full.txt) for full-documentation
context and `/docs/<path>.md` markdown twins for individual pages.

## Contributing

See [`CONTRIBUTING.md`](CONTRIBUTING.md) for local setup, pull request
expectations, and release note requirements.

## License

MIT. See [`LICENSE`](LICENSE).

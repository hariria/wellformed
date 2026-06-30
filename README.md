<div align="center">

<img src="assets/logo.svg" width="88" height="88" alt="wellformed" />

# wellformed

**Validation logic, as data.**

Author schemas once in TypeScript, compile them to a portable JSON IR, and run
the exact same rules in TypeScript and Rust.

[![npm](https://img.shields.io/npm/v/wellformed-ts?style=flat-square&logo=npm&color=76CE54&label=wellformed-ts)](https://www.npmjs.com/package/wellformed-ts)
[![crates.io](https://img.shields.io/crates/v/wellformed?style=flat-square&logo=rust&color=76CE54&label=wellformed)](https://crates.io/crates/wellformed)
[![License: MIT](https://img.shields.io/badge/license-MIT-76CE54?style=flat-square)](LICENSE)
[![X](https://img.shields.io/badge/X-76CE54?style=flat-square&logo=x&logoColor=white)](https://x.com/drewhariri)

[**Docs**](https://wellformed.net) · [**Playground**](https://wellformed.net/playground) · [Getting Started](https://wellformed.net/docs/getting-started) · [Comparison](https://wellformed.net/docs/comparison)

</div>

---

## Your form isn't always a web page

Validation rules are usually written as code, locked inside one app. But the
form they validate often isn't. It's defined in a form builder, configured per
customer, stored in a database, submitted as JSON, and re-validated by a backend
in another language. Code can't go where the form goes, so every surface
reimplements the rules by hand and the copies drift out of sync.

The same rules have to hold everywhere the form travels:

- **Form builders** define fields at runtime; the rules live with the form, not
  hardcoded in your app.
- **Web & mobile** show inline field errors as the user types.
- **PDFs & documents** need fields validated before you fill or accept them.
- **Any backend** validates submitted JSON in Node, Rust, or a worker before
  trusting the shape.
- **AI agents** generate and inspect rules as data, not opaque closures.

wellformed compiles your validation to one portable JSON definition. Store it
with the form, load it at runtime, and run the exact same contract in TypeScript
and Rust.

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
- **Fast.** Built-in predicates validate in tens of nanoseconds and stay
  competitive with the fastest validation libraries, while keeping schemas
  portable across TypeScript and Rust.

## Performance snapshot

Steady-state validation, lower is better. The Rust numbers are dynamic JSON
validation with `wellformed::validate(&schema, &mut serde_json::Value)`, not
generated typed Rust.

| Case | Rust dynamic JSON | TypeScript | Valibot | Zod |
| --- | ---: | ---: | ---: | ---: |
| Email | ~58 ns | ~61 ns | ~53 ns | ~2.73 us |
| HTTP URL | ~58 ns | ~106 ns | ~1.31 us | ~4.41 us |
| UUID | ~76 ns | ~77 ns | ~82 ns | ~2.82 us |
| Object | ~544 ns | ~411 ns | ~517 ns | ~4.35 us |

[See the full methodology and benchmark table](https://wellformed.net/docs/performance).

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
[Read the IR reference](https://wellformed.net/docs/ir-schema).

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
[`/llms-full.txt`](https://wellformed.net/llms-full.txt) for full-documentation
context and `/docs/<path>.md` markdown twins for individual pages.

## Contributing

See [`CONTRIBUTING.md`](CONTRIBUTING.md) for local setup, pull request
expectations, and release note requirements.

## License

MIT. See [`LICENSE`](LICENSE).

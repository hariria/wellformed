# wellformed

Portable, declarative validation schemas for forms and structured data.

`wellformed` gives you a TypeScript builder API that compiles to a JSON intermediate representation (IR). That IR can be stored, reviewed, versioned, and evaluated outside JavaScript, including by the Rust runtime in this workspace.

## Why Use It

- **Portable schemas**: validation rules serialize to JSON instead of living inside opaque JavaScript closures.
- **TypeScript-first authoring**: use a familiar chained API such as `w.string().trim().minLen(1).email()`.
- **Runtime validation**: validate and normalize data in TypeScript, or deserialize the same IR in Rust.
- **Domain predicates**: built-ins for TINs, SSNs, EINs, CUSIPs, ABA routing numbers, card fields, dates, contact fields, codes, text classes, and more.
- **Cross-field rules**: encode requirements such as "if `kind` is `individual`, require `ssn`" as structured predicates.
- **Type inference**: `Infer<typeof schema>` produces the corresponding TypeScript type.

## Status

`wellformed` is preparing for its first public production users. Treat the project as pre-1.0: pin exact versions, read the changelog before upgrading, and expect compatibility notes for public API changes.

Do not put sensitive personal data in public issues or examples. Use the private security reporting process in `SECURITY.md` for vulnerabilities.

## Quick Start

```npm
npm install wellformed-ts
```

```ts
import { validate, w, type Infer } from "wellformed-ts";

const taxpayer = w.object({
  kind: w.enum(["individual", "business"] as const),
  name: w.string().trim().minLen(1),
  ssn: w.string().digitsOnly().ssn().optional(),
  ein: w.string().digitsOnly().ein().optional(),
})
  .when("kind").equals("individual").require("ssn")
  .when("kind").equals("business").require("ein")
  .mutuallyExclusive("ssn", "ein");

type Taxpayer = Infer<typeof taxpayer>;

const schema = taxpayer.toSchema("1.0");
const result = validate(schema, {
  kind: "individual",
  name: "  Ada Lovelace  ",
  ssn: "123-45-6789",
});

if (result.valid) {
  const value = result.value as Taxpayer;
  console.log(value.name); // "Ada Lovelace"
}
```

## Workspace Layout

- `wellformed/`: Rust core crate for IR types, runtime validation, transforms, predicates, and codegen.
- `wellformed-macros/`: Rust proc macros for schema-driven code generation.
- `wellformed-validate/`: low-level validation primitives and benchmark-focused helpers.
- `typescript/packages/wellformed/`: TypeScript builder API, IR types, runtime, forms helpers, and tests.
- `typescript/apps/docs/`: documentation site and playground.

## Development

Prerequisites:

- Rust 1.93.0+ stable toolchain
- `cargo-audit` (`cargo install cargo-audit --locked --version 0.22.1`)
- Node.js 22 LTS (`nvm use`)
- pnpm 9+

Core checks:

```bash
bash scripts/release-preflight.sh
```

Optional address-predicate feature check, on machines with native libpostal:

```bash
bash scripts/release-preflight.sh --address
```

Docs:

```bash
cd typescript
pnpm --filter @wellformed/docs dev
```

Docs deployment:

The docs site deploys to Cloudflare Pages through `.github/workflows/docs-pages.yml`.
Set these GitHub repository secrets:

- `CLOUDFLARE_API_TOKEN`
- `CLOUDFLARE_ACCOUNT_ID`

The API token should grant `Account` -> `Cloudflare Pages` -> `Edit`.

Create a Cloudflare Pages project named `wellformed`, or override the default
with the repository variable `CLOUDFLARE_PAGES_PROJECT_NAME` if the project uses
a different name. Pushes to `main` deploy production; same-repository pull
requests get preview deployments and a PR comment with the preview URL.

## Production Notes

- Prefer stable machine-readable error `code` values over parsing human-readable `message` text.
- Decide explicitly how objects should handle unknown keys: `.strict()`, `.strip()`, `.passthrough()`, or `.catchall(...)`.
- Keep custom predicates registered consistently in every runtime that evaluates the same IR.
- Store serialized schemas with a version and application-level migration metadata.
- Use Rust `wellformed!` when an application wants to embed a schema as a value for native validation.
- Use Rust `form_schema!` when an application needs a namespaced form facade with typed values, field metadata, schema JSON, validation helpers, submitted-value field state, optional client helper functions, and framework-neutral `FormState`.
- Rust `wel_schema!` is available when you prefer free-floating generated structs and validation methods. Opt into generated Axum/API code explicitly with `CodegenOptions { generate_api: true, .. }`; PDF handlers require the separate `generate_pdf_handlers` option and app-provided rendering dependencies.
- Pin package/crate versions until the project reaches 1.0.

## AI-Assisted Integration

The TypeScript package ships `skills.md`, a compact AI-agent guide for integrating wellformed into applications. The docs site exposes `/llms-full.txt` for full-documentation context and `/docs/<path>.md` markdown twins for individual docs pages.

## Contributing

See `CONTRIBUTING.md` for local setup, pull request expectations, and release note requirements.

## Security

See `SECURITY.md` for vulnerability reporting instructions.

## License

MIT, see `LICENSE`.

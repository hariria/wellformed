# wellformed

Portable, declarative validation schemas for forms and structured data.

`wellformed` provides a TypeScript builder API that compiles to a serializable JSON intermediate representation (IR). The same IR can be validated in TypeScript or deserialized and evaluated by the Rust runtime.

## Install

```npm
npm install wellformed-ts
```

The published package supports Node.js 18 or newer. This repository uses Node
22 for development and release checks.

## Import Paths

The top-level import is convenient while authoring schemas:

```ts
import { validate, w, type Infer } from "wellformed-ts";
```

For smaller production bundles, import runtime-only pieces from subpaths:

```ts
import { w } from "wellformed-ts/builder";
import type { Schema } from "wellformed-ts/ir";
import { validate } from "wellformed-ts/runtime";
import { parseSchema, schemaToJSON } from "wellformed-ts/serialize";
```

## AI Integration

This package ships `skills.md`, a compact AI-agent integration guide covering schema authoring, runtime validation, React Hook Form resolver patterns, serialized IR, Rust boundaries, and common mistakes. Use it as context when asking an AI coding tool to add `wellformed` to an application.

## Example

```ts
import { validate, w, type Infer } from "wellformed-ts";

const schemaBuilder = w.object({
  kind: w.enum(["individual", "business"] as const),
  name: w.string().trim().minLen(1),
  ssn: w.string().digitsOnly().ssn().optional(),
  ein: w.string().digitsOnly().ein().optional(),
})
  .when("kind").equals("individual").require("ssn")
  .when("kind").equals("business").require("ein")
  .mutuallyExclusive("ssn", "ein");

type Taxpayer = Infer<typeof schemaBuilder>;

const result = validate(schemaBuilder.toSchema("1.0"), {
  kind: "individual",
  name: "  Ada Lovelace  ",
  ssn: "123-45-6789",
});

if (result.valid) {
  const value = result.value as Taxpayer;
  console.log(value.name); // "Ada Lovelace"
}
```

## What It Includes

- Zod-like TypeScript builder API
- Runtime validation and transform pipeline
- Serializable JSON IR
- Cross-field rules
- Type inference
- Built-in predicates for taxpayer identifiers, financial identifiers, dates, contact fields, healthcare codes, aviation codes, colors, text classes, and more

## Production Notes

- Pin exact versions before 1.0.
- Use machine-readable error `code` values for application logic.
- Choose object unknown-key behavior explicitly with `.strict()`, `.strip()`, `.passthrough()`, or `.catchall(...)`.
- Register equivalent custom predicates in every runtime that evaluates the same IR.
- Do not put sensitive personal data in public issues or examples.

## License

MIT.

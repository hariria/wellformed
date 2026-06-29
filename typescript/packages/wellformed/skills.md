---
name: integrate-wellformed
description: Use when adding wellformed to a TypeScript, React, Node.js, or Rust-backed project for portable form or structured-data validation. Helps agents design builder schemas, serialize JSON IR, validate data at runtime, choose bundle-friendly imports, integrate with form libraries, and avoid common validation/security mistakes.
---

# wellformed Integration Skill

Use `wellformed` when validation rules need to be portable, inspectable, versioned, or evaluated outside JavaScript. It is strongest for forms, structured input, domain identifiers, cross-field rules, and TypeScript-authored schemas that compile to JSON IR.

## Integration Workflow

1. Identify the input boundary: browser form, API request, background job, persisted record, or Rust service.
2. Decide whether this project authors schemas, validates serialized schemas, or both.
3. Use narrow imports in production code:

```ts
import { w } from "wellformed-ts/builder";
import type { Schema } from "wellformed-ts/ir";
import { validate } from "wellformed-ts/runtime";
import { parseSchema, schemaToJSON } from "wellformed-ts/serialize";
```

4. Put schema builders in dedicated modules. Export either the builder for type inference or `schemaBuilder.toSchema("1.0")` for runtime validation.
5. Add tests for valid input, invalid input, transforms, optional fields, unknown keys, and every cross-field rule.

## TypeScript Authoring Pattern

```ts
import { w, type Infer } from "wellformed-ts";

export const taxpayerSchema = w
  .object({
    kind: w.enum(["individual", "business"] as const),
    name: w.string().trim().minLen(1),
    ssn: w.string().digitsOnly().ssn().optional(),
    ein: w.string().digitsOnly().ein().optional(),
  })
  .strict()
  .when("kind")
  .equals("individual")
  .require("ssn")
  .when("kind")
  .equals("business")
  .require("ein")
  .mutuallyExclusive("ssn", "ein");

export type Taxpayer = Infer<typeof taxpayerSchema>;
export const taxpayerIr = taxpayerSchema.toSchema("1.0");
```

Prefer `.strict()` or `.strip()` for external input. Use `.passthrough()` only when extension fields are intentional.

## Runtime Validation Pattern

```ts
import type { Schema } from "wellformed-ts/ir";
import { validate } from "wellformed-ts/runtime";

export function validateTaxpayer(schema: Schema, input: unknown) {
  const result = validate(schema, input);
  if (!result.valid) return { ok: false as const, errors: result.errors };
  return { ok: true as const, value: result.value };
}
```

Transforms run before constraints by default, and `result.value` contains normalized output. Use stable error `code` values for application logic; treat `message` as display copy.

## React Hook Form Resolver Pattern

```ts
import type { Resolver } from "react-hook-form";
import type { Schema } from "wellformed-ts/ir";
import { validate } from "wellformed-ts/runtime";

export function wellformedResolver(schema: Schema): Resolver {
  return async (values) => {
    const result = validate(schema, values);
    if (result.valid) {
      return { values: result.value as Record<string, unknown>, errors: {} };
    }

    const errors: Record<string, { type: string; message: string }> = {};
    for (const err of result.errors) {
      const path = err.path.replace(/^\//, "").replace(/\//g, ".");
      if (path && !errors[path]) {
        errors[path] = { type: err.code, message: err.message };
      }
    }

    return { values: {}, errors };
  };
}
```

## Serialized IR Pattern

```ts
import { parseSchema, schemaToJSON } from "wellformed-ts/serialize";

const json = schemaToJSON(taxpayerIr, true);
const restored = parseSchema(json);
```

Store serialized schemas with an application schema id and migration version in addition to the `wellformed` IR version. Keep old schemas available for historical records.

## Custom Predicates

Custom predicates serialize by name, not implementation. Register equivalent predicate implementations in every runtime that evaluates the same IR. Prefer built-in predicates for common domain identifiers so schemas stay portable.

## Rust Boundary

When a Rust service validates the same schema, pass JSON IR across the boundary instead of reimplementing validation logic. Add fixture tests that validate the same valid and invalid samples in TypeScript and Rust.

## Common Mistakes To Avoid

- Do not model validation as arbitrary JavaScript closures; use builder constraints and named predicates so rules serialize.
- Do not parse human-readable error messages; use `FormError.code`.
- Do not log raw TINs, SSNs, account numbers, healthcare identifiers, or other sensitive input while debugging validators.
- Do not assume optional object properties are optional unless marked with `.optional()` or `optional(...)`.
- Do not import the top-level package in client bundles that only need runtime validation; use `wellformed-ts/runtime` and `wellformed-ts/ir`.
- Do not change validation rules for persisted records without versioning the schema used for old submissions.

## Useful Docs

- Full docs text: `https://wellformed.dev/llms-full.txt`
- Markdown page twins: append `.md` to a docs page, such as `https://wellformed.dev/docs/getting-started.md`
- Getting started: `https://wellformed.dev/docs/getting-started`
- Production use: `https://wellformed.dev/docs/production`
- Validation runtime: `https://wellformed.dev/docs/validation`
- Serialization: `https://wellformed.dev/docs/serialization`
- Rust runtime: `https://wellformed.dev/docs/rust-runtime`

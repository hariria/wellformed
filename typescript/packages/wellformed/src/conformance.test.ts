// Cross-runtime conformance suite (TypeScript side).
// Runs the shared fixtures in /conformance/cases through the TS runtime and
// asserts behavior per the fixture `status`. The Rust side runs the same files
// (wellformed/tests/conformance.rs). See /conformance/README.md.

import { readdirSync, readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";
import type { Schema } from "./ir/types.js";
import { validate } from "./runtime/validate.js";

const CASES_DIR = join(
  dirname(fileURLToPath(import.meta.url)),
  "../../../../conformance/cases",
);

interface Outcome {
  valid: boolean;
  value?: unknown;
  /** If present, at least one emitted error must carry this stable code. */
  code?: string;
}

interface Fixture {
  name: string;
  note?: string;
  schema: Schema;
  input: unknown;
  expect: Outcome;
  status: "agree" | "known_divergence";
  current?: { ts: Outcome; rust: Outcome };
  skip_ts?: boolean;
  skip_rust?: boolean;
}

function loadFixtures(): Fixture[] {
  return readdirSync(CASES_DIR)
    .filter((f) => f.endsWith(".json") && !f.startsWith("_"))
    .sort()
    .map(
      (f) => JSON.parse(readFileSync(join(CASES_DIR, f), "utf8")) as Fixture,
    );
}

describe("cross-runtime conformance (TypeScript)", () => {
  for (const fx of loadFixtures()) {
    const run = fx.skip_ts ? it.skip : it;
    run(fx.name, () => {
      // The target the TS runtime must match: `expect` when the runtimes agree,
      // or the documented current TS behavior for a known divergence.
      const target =
        fx.status === "agree" ? fx.expect : (fx.current?.ts ?? fx.expect);

      const result = validate(fx.schema, fx.input);
      expect(result.valid, fx.note ?? fx.name).toBe(target.valid);
      if ("value" in target) {
        expect(result.value).toEqual(target.value);
      }
      if (target.code) {
        expect(result.errors.map((e) => e.code)).toContain(target.code);
      }
    });
  }
});

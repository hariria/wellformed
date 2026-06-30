// Differential fuzzer for the wellformed IR.
//
// Generates random *valid* schemas and inputs from a seed, runs each through
// both runtimes (TypeScript in-process, Rust via the fuzz_driver example), and
// reports any case where they disagree on validity, stable issue fields, or
// normalized value. On a disagreement it shrinks to a minimal reproducer and
// prints it as a ready-made conformance fixture.
//
// The generator only builds inputs where both runtimes are supposed to match
// (shallow nesting, portable numbers, known predicates, no raw regex, no
// pathological ref depth), so any disagreement is a real bug, not a documented
// limit. See fuzz/README.md.
//
// Usage:
//   node fuzz/fuzz.mjs [--seed N] [--cases M] [--batches B] [--verbose]

import { execFileSync } from "node:child_process";
import { existsSync, mkdtempSync, readdirSync, readFileSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const HERE = dirname(fileURLToPath(import.meta.url));
const ROOT = join(HERE, "..");
const CONFORMANCE_CASES = join(ROOT, "conformance", "cases");
const DRIVER = join(ROOT, "target", "release", "examples", "fuzz_driver");
const DIST = join(ROOT, "typescript", "packages", "wellformed", "dist", "index.js");

const {
  createEvalContext,
  parseSchema,
  PredicateRegistry,
  schemaToJSON,
  validate,
} = await import(DIST);

const customRegistry = PredicateRegistry.withBuiltins();
customRegistry.register("custom_is_even", (value) => {
  if (typeof value === "number") return Number.isInteger(value) && value % 2 === 0;
  if (typeof value === "string" && /^-?\d+$/.test(value)) return BigInt(value) % 2n === 0n;
  return false;
});
const customEvalContext = createEvalContext(customRegistry);

// ---------------------------------------------------------------------------
// CLI
// ---------------------------------------------------------------------------

function arg(name, def) {
  const i = process.argv.indexOf(`--${name}`);
  if (i === -1) return def;
  const v = process.argv[i + 1];
  return v && !v.startsWith("--") ? v : true;
}
const SEED = Number(arg("seed", 1)) >>> 0;
const CASES = Number(arg("cases", 2000));
const BATCHES = Number(arg("batches", 25));
const VERBOSE = arg("verbose", false) === true;

// ---------------------------------------------------------------------------
// Seeded RNG (mulberry32): deterministic, so a seed reproduces a run exactly.
// ---------------------------------------------------------------------------

function mulberry32(a) {
  return () => {
    a |= 0;
    a = (a + 0x6d2b79f5) | 0;
    let t = Math.imul(a ^ (a >>> 15), 1 | a);
    t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}
const mk = (rng) => ({
  f: rng,
  int: (n) => Math.floor(rng() * n),
  pick: (arr) => arr[Math.floor(rng() * arr.length)],
  chance: (p) => rng() < p,
});

// ---------------------------------------------------------------------------
// Generators: produce raw IR JSON (plain objects) and targeted inputs.
// ---------------------------------------------------------------------------

function genNumber(r) {
  const kind = r.int(6);
  if (kind === 0) return 0;
  if (kind === 1) return r.int(1000) - 500; // small int, +/-
  if (kind === 2) return (r.int(20000) - 10000) / 100; // 2-dp decimal
  if (kind === 3) return (r.int(2000) - 1000) / 8; // odd fractions
  if (kind === 4) return r.pick([0.5, -0.5, 2.5, -2.5, 1.005, 2.675, 0.045]);
  return r.int(1_000_000); // mid integer, still safe
}

function genInteger(r) {
  return r.int(2000) - 1000;
}

function genUint(r) {
  return r.int(2000);
}

function genDateString(r) {
  return r.pick([
    "2024-12-25",
    "12/25/2024",
    "25/12/2024",
    "12252024",
    "02/29/2024",
    "02/31/2024",
  ]);
}

function genString(r) {
  const kind = r.int(9);
  if (kind === 0) return "";
  if (kind === 1) return ["a", "ab", "abc", "hello", "  pad  ", "MiXeD"][r.int(6)];
  if (kind === 2) return String(genNumber(r)); // numeric string (feeds transforms)
  if (kind === 3) return ["123-45-6789", "12-3456789", "4111 1111 1111 1111", "1,234.5"][r.int(4)];
  if (kind === 4)
    return ["(415) 555-2671", "4155552671", "+1 (415) 555-2671", "14155552671"][r.int(4)];
  if (kind === 5) return ["GB29NWBK60161331926819", "gb29 nwbk 6016 1331 9268 19"][r.int(2)];
  if (kind === 6) return ["S72 001A", "A1234", "J1234", "1234-5678-90"][r.int(4)];
  if (kind === 7) return genDateString(r);
  return "x".repeat(r.int(8));
}

function genScalar(r) {
  return r.pick([
    () => genNumber(r),
    () => genString(r),
    () => r.chance(0.5),
    () => null,
  ])();
}

function isJsonValue(value) {
  if (value === null) return true;
  if (typeof value === "string" || typeof value === "boolean") return true;
  if (typeof value === "number") return Number.isFinite(value);
  if (Array.isArray(value)) return value.every(isJsonValue);
  if (value && typeof value === "object")
    return Object.values(value).every(isJsonValue);
  return false;
}

const STRING_TRANSFORMS = [
  { fn: "trim" },
  { fn: "lower" },
  { fn: "upper" },
  { fn: "collapse_whitespace" },
  { fn: "digits_only" },
  { fn: "date_parse", format: "MM/DD/YYYY" },
  { fn: "replace", pattern: "-", replacement: "" },
  { fn: "phone_us" },
  { fn: "phone_e164" },
  { fn: "card_mask_last4" },
  { fn: "format_ssn" },
  { fn: "format_ein" },
  { fn: "mask_ssn" },
  { fn: "mask_ein" },
  { fn: "format_iban" },
  { fn: "format_credit_card" },
  { fn: "normalize_flight_number" },
  { fn: "normalize_icd10" },
  { fn: "normalize_cpt" },
  { fn: "normalize_hcpcs" },
  { fn: "normalize_ndc11" },
];

const STRING_TRANSFORM_CHAINS = [
  [{ fn: "trim" }, { fn: "lower" }],
  [{ fn: "trim" }, { fn: "upper" }],
  [{ fn: "digits_only" }, { fn: "format_ssn" }],
  [{ fn: "digits_only" }, { fn: "format_ein" }],
  [{ fn: "trim" }, { fn: "phone_us" }],
  [{ fn: "replace", pattern: "-", replacement: "" }, { fn: "upper" }],
  [{ fn: "money_to_cents", scale: 2 }, { fn: "format_decimal", places: 0 }],
];

function genTransforms(r) {
  if (r.chance(0.25)) return r.pick(STRING_TRANSFORM_CHAINS);
  return [r.pick(STRING_TRANSFORMS)];
}

function errorMeta(r, code, message) {
  const e = { code, message };
  if (r.chance(0.15)) e.severity = "warning";
  if (r.chance(0.1)) e.path = "/_schema";
  return e;
}

function maybeStringFields(r) {
  const f = {};
  if (r.chance(0.45)) f.transforms = genTransforms(r);
  if (r.chance(0.4)) {
    const c = [];
    if (r.chance(0.6))
      c.push({ pred: { type: "min_len", len: r.int(5) }, error: errorMeta(r, "MIN", "min") });
    if (r.chance(0.6))
      c.push({ pred: { type: "max_len", len: r.int(8) + 1 }, error: errorMeta(r, "MAX", "max") });
    if (c.length) f.constraints = c;
  }
  return f;
}

// Numeric-string transforms: the historically bug-prone money/decimal area.
function genTransformString(r) {
  const transforms = r.chance(0.25) ? r.pick(STRING_TRANSFORM_CHAINS) : [r.pick([
    { fn: "money_to_cents", scale: r.int(4) },
    { fn: "format_decimal", places: r.int(4) },
    { fn: "format_thousands" },
  ])];
  return { schema: { type: "string", transforms }, numeric: true };
}

const OBJECT_KEYS = ["f0", "f1", "a/b", "a~b", "", "sp ace"];

function genObjectKey(r, used, fallbackIndex) {
  const candidates = OBJECT_KEYS.filter((k) => !used.has(k));
  const key = candidates.length ? r.pick(candidates) : `f${fallbackIndex}`;
  used.add(key);
  return key;
}

function objectProperties(schema) {
  return schema.properties ?? schema.fields ?? {};
}

function genNumericConstraint(r) {
  const min = r.int(20) - 10;
  const max = min + r.int(30) + 1;
  return { pred: { type: "range", min, max }, error: errorMeta(r, "R", "r") };
}

function maybeNumericFields(r) {
  return r.chance(0.35) ? { constraints: [genNumericConstraint(r)] } : {};
}

function hardError(code, message) {
  return { code, message, severity: "error" };
}

function hardConstraint(pred, code, message) {
  return { pred, error: hardError(code, message) };
}

function genObjectSchema(r, depth) {
  const n = r.int(3) + 1;
  const properties = {};
  const used = new Set();
  for (let i = 0; i < n; i++) {
    const child = genType(r, depth - 1);
    if (r.chance(0.4)) child.required = false;
    properties[genObjectKey(r, used, i)] = child;
  }

  const s = { type: "object" };
  if (r.chance(0.12)) s.fields = properties;
  else s.properties = properties;

  const mode = r.int(5);
  if (mode === 0) s.unknown_keys = "strict";
  else if (mode === 1) s.unknown_keys = "passthrough";
  else if (mode === 2) s.unknown_keys = "strip";
  else if (mode === 3) s.additional_properties = true;
  else if (mode === 4) s.additional_properties = false;

  if (r.chance(0.25)) {
    s.catchall = r.chance(0.5)
      ? { type: "string", transforms: [{ fn: "trim" }] }
      : { type: "number", ...maybeNumericFields(r) };
  }
  return s;
}

const TEMPLATE_CASES = [
  {
    parts: [
      { kind: "literal", value: "INV-" },
      { kind: "digits", min: 2, max: 4 },
      { kind: "literal", value: "-" },
      { kind: "uppercase", min: 2, max: 2 },
    ],
    valid: "INV-123-AB",
    invalid: "INV-A-AB",
  },
  {
    parts: [
      { kind: "ascii_letters", min: 2, max: 3 },
      { kind: "literal", value: ":" },
      { kind: "hex", min: 4, max: 4 },
    ],
    valid: "ab:09Af",
    invalid: "ab:09xz",
  },
];

function genTemplateLiteralCase(r, forceValid = false) {
  const c = r.pick(TEMPLATE_CASES);
  const valid = forceValid || r.chance(0.55);
  return {
    schema: {
      version: "1.0",
      root: {
        type: "string",
        constraints: [
          hardConstraint({ type: "template_literal", parts: c.parts }, "TPL", "template"),
        ],
      },
    },
    input: valid ? c.valid : c.invalid,
    expectedRuleCode: valid ? undefined : "TPL",
  };
}

const CALL_SCENARIOS = [
  { pred: { type: "call", name: "is_email" }, input: "ada@example.com", bad: "not-an-email" },
  { pred: { type: "call", name: "is_ssn" }, input: "123-45-6789", bad: "000-00-0000" },
  { pred: { type: "call", name: "is_iban" }, input: "GB29NWBK60161331926819", bad: "GB00NWBK60161331926819" },
  { pred: { type: "call", name: "is_npi" }, input: "1234567893", bad: "1234567890" },
  {
    pred: { type: "call", name: "is_uuid", args: { version: 4 } },
    input: "550e8400-e29b-41d4-a716-446655440000",
    bad: "550e8400-e29b-11d4-a716-446655440000",
  },
  {
    pred: { type: "call", name: "is_url", args: { require_https: true } },
    input: "https://example.com",
    bad: "http://example.com",
  },
  { pred: { type: "call", name: "starts_with", args: { value: "wf_" } }, input: "wf_schema", bad: "schema_wf" },
  { pred: { type: "call", name: "contains", args: { value: "abc" } }, input: "xxabcxx", bad: "xxabxx" },
  {
    pred: { type: "call", name: "is_decimal_places", args: { places: 2 } },
    input: 12.34,
    bad: 12.345,
    schemaType: "number",
  },
  {
    pred: { type: "call", name: "custom_is_even" },
    input: 24,
    bad: 25,
    schemaType: "number",
    registry: "custom",
  },
  {
    pred: { type: "call", name: "is_i64" },
    input: "9223372036854775807",
    bad: "9223372036854775808",
  },
  {
    pred: { type: "call", name: "is_u64" },
    input: "18446744073709551615",
    bad: "18446744073709551616",
  },
];

function genCallPredicateCase(r, forceValid = false) {
  const c = r.pick(CALL_SCENARIOS);
  const valid = forceValid || r.chance(0.55);
  return {
    schema: {
      version: "1.0",
      root: {
        type: c.schemaType ?? "string",
        constraints: [hardConstraint(c.pred, "CALL", "call")],
      },
    },
    input: valid ? c.input : c.bad,
    registry: c.registry,
    expectedRuleCode: valid ? undefined : "CALL",
  };
}

function genUnknownCallCase() {
  return {
    schema: {
      version: "1.0",
      root: {
        type: "string",
        constraints: [
          hardConstraint({ type: "call", name: "not_registered_in_any_runtime" }, "UNKNOWN", "unknown"),
        ],
      },
    },
    input: "x",
    expectedOutOfBand: "unknown_predicate",
  };
}

function genValuePredicateCase(r, forceValid = false) {
  const family = r.pick(["eq", "in", "exists", "or", "not", "implies"]);
  let pred;
  let input;
  let validInput;
  let invalidInput;
  let code = family.toUpperCase();

  if (family === "eq") {
    pred = { type: "eq", path: "/kind", value: "business" };
    validInput = { kind: "business" };
    invalidInput = { kind: "individual" };
  } else if (family === "in") {
    pred = { type: "in", path: "/status", values: ["draft", "sent", "paid"] };
    validInput = { status: "sent" };
    invalidInput = { status: "void" };
  } else if (family === "exists") {
    pred = { type: "exists", path: "/email" };
    validInput = { email: "ada@example.com" };
    invalidInput = {};
  } else if (family === "or") {
    pred = {
      type: "or",
      predicates: [
        { type: "eq", path: "/kind", value: "business" },
        { type: "eq", path: "/status", value: "paid" },
      ],
    };
    validInput = { kind: "individual", status: "paid" };
    invalidInput = { kind: "individual", status: "draft" };
  } else if (family === "not") {
    pred = { type: "not", predicate: { type: "eq", path: "/status", value: "blocked" } };
    validInput = { status: "active" };
    invalidInput = { status: "blocked" };
  } else {
    pred = {
      type: "implies",
      if: { type: "eq", path: "/kind", value: "business" },
      then: { type: "exists", path: "/ein" },
    };
    validInput = r.chance(0.5) ? { kind: "business", ein: "12-3456789" } : { kind: "individual" };
    invalidInput = { kind: "business" };
    code = "IMPLIES";
  }

  const valid = forceValid || r.chance(0.55);
  input = valid ? validInput : invalidInput;

  return {
    schema: {
      version: "1.0",
      root: {
        type: "object",
        properties: {
          kind: { type: "string", required: false },
          status: { type: "string", required: false },
          email: { type: "string", required: false },
          ein: { type: "string", required: false },
        },
        rules: [hardConstraint(pred, code, "rule")],
      },
    },
    input,
    expectedRuleCode: valid ? undefined : code,
  };
}

function genDefaultCase(r) {
  const input = r.pick([
    {},
    { name: null, count: null, code: null, prefs: null, tags: null },
    { name: "Ada" },
  ]);
  return {
    schema: {
      version: "1.0",
      root: {
        type: "object",
        properties: {
          name: { type: "string", transforms: [{ fn: "default", value: "Anonymous" }] },
          count: { type: "number", transforms: [{ fn: "default", value: 3 }] },
          code: { type: "string", required: false, transforms: [{ fn: "default", value: "WF" }] },
          prefs: {
            type: "object",
            required: false,
            transforms: [{ fn: "default", value: { theme: "system", compact: false } }],
            properties: {
              theme: { type: "string" },
              compact: { type: "boolean" },
            },
          },
          tags: {
            type: "array",
            required: false,
            transforms: [{ fn: "default", value: ["new", "portable"] }],
            items: { type: "string" },
          },
        },
      },
    },
    input,
  };
}

function genObjectInteractionCase(r) {
  const family = r.pick(["rules_after_transform", "rule_after_default", "strict_plus_invalid"]);
  if (family === "rules_after_transform") {
    return {
      schema: {
        version: "1.0",
        root: {
          type: "object",
          properties: {
            country: { type: "string", transforms: [{ fn: "trim" }, { fn: "upper" }] },
          },
          rules: [hardConstraint({ type: "eq", path: "/country", value: "US" }, "COUNTRY", "country")],
        },
      },
      input: { country: " us " },
    };
  }
  if (family === "rule_after_default") {
    return {
      schema: {
        version: "1.0",
        root: {
          type: "object",
          properties: {
            country: { type: "string", transforms: [{ fn: "default", value: "US" }] },
          },
          rules: [hardConstraint({ type: "eq", path: "/country", value: "US" }, "COUNTRY", "country")],
        },
      },
      input: {},
    };
  }
  return {
    schema: {
      version: "1.0",
      root: {
        type: "object",
        unknown_keys: "strict",
        properties: {
          age: { type: "number" },
          name: { type: "string" },
        },
      },
    },
    input: { age: "old", name: 123, extra: true },
  };
}

function genWrapperScenarioCase(r) {
  const family = r.pick(["deep_preprocess", "catch_fallback", "union_output", "intersection_sequence"]);
  if (family === "deep_preprocess") {
    return {
      schema: {
        version: "1.0",
        root: {
          type: "preprocess",
          transforms: [{ fn: "trim" }],
          schema: {
            type: "preprocess",
            transforms: [{ fn: "upper" }],
            schema: {
              type: "string",
              constraints: [hardConstraint({ type: "eq", path: "", value: "OK" }, "OK", "ok")],
            },
          },
        },
      },
      input: " ok ",
    };
  }
  if (family === "catch_fallback") {
    return {
      schema: {
        version: "1.0",
        root: {
          type: "catch",
          schema: { type: "object", properties: { n: { type: "number" } } },
          value: { n: 0 },
        },
      },
      input: { n: "bad" },
    };
  }
  if (family === "union_output") {
    return {
      schema: {
        version: "1.0",
        root: {
          type: "union",
          oneOf: [
            { type: "string", transforms: [{ fn: "trim" }], constraints: [hardConstraint({ type: "min_len", len: 1 }, "MIN", "min")] },
            { type: "number" },
          ],
        },
      },
      input: " x ",
    };
  }
  return {
    schema: {
      version: "1.0",
      root: {
        type: "intersection",
        allOf: [
          { type: "string", transforms: [{ fn: "trim" }] },
          { type: "string", transforms: [{ fn: "replace", pattern: "-", replacement: "" }] },
          { type: "string", transforms: [{ fn: "upper" }] },
          { type: "string", constraints: [hardConstraint({ type: "eq", path: "", value: "AB" }, "AB", "ab")] },
        ],
      },
    },
    input: " a-b ",
  };
}

function linkedList(depth) {
  let node = { value: depth };
  for (let i = depth - 1; i >= 0; i--) node = { value: i, next: node };
  return node;
}

function genRefScenarioCase(r) {
  const family = r.pick(["missing", "cycle", "recursive"]);
  if (family === "missing") {
    return {
      schema: { version: "1.0", root: { type: "ref", $ref: "Missing" } },
      input: 1,
    };
  }
  if (family === "cycle") {
    return {
      schema: {
        version: "1.0",
        definitions: { A: { type: "ref", $ref: "A" } },
        root: { type: "ref", $ref: "A" },
      },
      input: 1,
    };
  }
  return {
    schema: {
      version: "1.0",
      definitions: {
        Node: {
          type: "object",
          properties: {
            value: { type: "number" },
            next: { type: "ref", $ref: "Node", required: false },
          },
        },
      },
      root: { type: "ref", $ref: "Node" },
    },
    input: linkedList(r.int(5) + 1),
  };
}

function genNumericBoundaryCase(r) {
  const cases = [
    { schema: { type: "int32" }, input: 2147483647 },
    { schema: { type: "int32" }, input: -2147483648 },
    { schema: { type: "int32" }, input: 2147483648 },
    { schema: { type: "uint32" }, input: 4294967295 },
    { schema: { type: "uint32" }, input: -1 },
    { schema: { type: "decimal", precision: 5, scale: 2 }, input: 123.45 },
    { schema: { type: "decimal", precision: 5, scale: 2 }, input: 1234.56 },
    { schema: { type: "string", transforms: [{ fn: "format_decimal", places: 2 }] }, input: -0.5 },
    { schema: { type: "string", transforms: [{ fn: "format_decimal", places: 2 }] }, input: 2.675 },
  ];
  const c = r.pick(cases);
  return { schema: { version: "1.0", root: c.schema }, input: c.input };
}

function genTargetedCase(r) {
  return r.pick([
    () => genTemplateLiteralCase(r),
    () => genCallPredicateCase(r),
    () => genUnknownCallCase(r),
    () => genValuePredicateCase(r),
    () => genDefaultCase(r),
    () => genObjectInteractionCase(r),
    () => genWrapperScenarioCase(r),
    () => genRefScenarioCase(r),
    () => genNumericBoundaryCase(r),
  ])();
}

function genWrapperType(r) {
  const kind = r.pick(["preprocess", "catch", "intersection"]);
  if (kind === "preprocess") {
    return {
      type: "preprocess",
      transforms: r.pick([
        [{ fn: "trim" }],
        [{ fn: "digits_only" }],
        [{ fn: "money_to_cents", scale: 2 }],
        [{ fn: "date_parse", format: "MM/DD/YYYY" }],
      ]),
      schema: r.pick([
        { type: "string" },
        { type: "integer" },
        { type: "money", scale: 2 },
        { type: "date" },
      ]),
    };
  }
  if (kind === "catch") {
    const schema = r.pick([
      { type: "integer" },
      {
        type: "string",
        constraints: [{ pred: { type: "min_len", len: 2 }, error: { code: "MIN", message: "m" } }],
      },
      { type: "object", properties: { f0: { type: "number" } } },
    ]);
    return { type: "catch", schema, value: r.pick([0, "", {}, null]) };
  }
  return {
    type: "intersection",
    allOf: r.chance(0.5)
      ? [
          { type: "string", transforms: [{ fn: "trim" }] },
          { type: "string", constraints: [{ pred: { type: "min_len", len: 1 }, error: { code: "MIN", message: "m" } }] },
        ]
      : [
          { type: "number", constraints: [{ pred: { type: "range", min: -1000, max: 1000 }, error: { code: "R1", message: "r" } }] },
          { type: "number", constraints: [{ pred: { type: "range", min: -500, max: 500 }, error: { code: "R2", message: "r" } }] },
        ],
  };
}

function genType(r, depth) {
  // At depth budget, only leaf types.
  const composite = depth > 0;
  const choices = [
    "string",
    "stringT",
    "number",
    "integer",
    "int32",
    "int64",
    "uint32",
    "uint64",
    "boolean",
    "money",
    "currency",
    "decimal",
    "percentage",
    "date",
    "enum",
    "literal",
    "never",
    "any",
  ];
  if (composite) choices.push("object", "array", "tuple", "union", "record", "wrapper", "ref");

  const t = r.pick(choices);
  switch (t) {
    case "string":
      return { type: "string", ...maybeStringFields(r) };
    case "stringT":
      return genTransformString(r).schema;
    case "number":
    case "integer":
    case "int32":
    case "int64":
    case "uint32":
    case "uint64": {
      const s = { type: t, ...maybeNumericFields(r) };
      if (r.chance(0.4))
        s.constraints = [genNumericConstraint(r)];
      return s;
    }
    case "boolean":
      return { type: "boolean" };
    case "money":
      return { type: "money", scale: r.int(4) };
    case "currency":
      return { type: "currency", code: r.pick(["USD", "EUR", "JPY"]), scale: r.int(4) };
    case "decimal":
      return {
        type: "decimal",
        scale: r.int(4),
        ...(r.chance(0.4) ? { precision: r.int(6) + 2 } : {}),
      };
    case "percentage":
      return {
        type: "percentage",
        ...(r.chance(0.4) ? { format: "whole" } : {}),
        ...(r.chance(0.25) ? { allow_over_100: true } : {}),
      };
    case "date":
      return {
        type: "date",
        ...(r.chance(0.35) ? { transforms: [{ fn: "date_parse", format: "MM/DD/YYYY" }] } : {}),
      };
    case "enum":
      return { type: "enum", values: Array.from({ length: r.int(4) + 1 }, () => genScalar(r)) };
    case "literal":
      return { type: "literal", value: genScalar(r) };
    case "never":
      return { type: "never" };
    case "any":
      return { type: "any" };
    case "object":
      return genObjectSchema(r, depth);
    case "array": {
      const s = { type: "array", items: genType(r, depth - 1) };
      if (r.chance(0.5)) s.min_items = r.int(3);
      if (r.chance(0.5)) s.max_items = r.int(3) + 1;
      return s;
    }
    case "tuple":
      return { type: "tuple", items: Array.from({ length: r.int(3) + 1 }, () => genType(r, depth - 1)) };
    case "union":
      return { type: "union", oneOf: Array.from({ length: r.int(2) + 2 }, () => genType(r, depth - 1)) };
    case "record":
      return {
        type: "record",
        value: genType(r, depth - 1),
        ...(r.chance(0.4)
          ? {
              key: {
                type: "string",
                constraints: [{ pred: { type: "min_len", len: 1 }, error: { code: "KEY", message: "k" } }],
              },
            }
          : {}),
      };
    case "wrapper":
      return genWrapperType(r);
    case "ref":
      return { type: "ref", $ref: "Def" };
    default:
      return { type: "any" };
  }
}

// Generate an input targeted at a schema: conforming most of the time, random
// otherwise, so both the valid and invalid paths get exercised.
function genInput(r, schema, depth, definitions) {
  if (r.chance(0.2)) return genScalar(r); // perturbation: random scalar

  switch (schema.type) {
    case "string":
      return genString(r);
    case "number":
    case "integer":
    case "int32":
    case "int64":
      return r.chance(0.65) ? genInteger(r) : String(genInteger(r));
    case "uint32":
    case "uint64":
      return r.chance(0.65) ? genUint(r) : String(genUint(r));
    case "money":
    case "decimal":
    case "percentage":
    case "currency":
      return r.chance(0.5) ? genNumber(r) : String(genNumber(r));
    case "date":
      return r.chance(0.75) ? genDateString(r) : genScalar(r);
    case "boolean":
      return r.chance(0.85);
    case "enum":
      return r.chance(0.7) && schema.values.length ? r.pick(schema.values) : genScalar(r);
    case "literal":
      return r.chance(0.7) ? schema.value : genScalar(r);
    case "any":
      return genScalar(r);
    case "object": {
      if (depth <= 0) return {};
      const o = {};
      for (const [k, child] of Object.entries(objectProperties(schema))) {
        if (child.required === false && r.chance(0.4)) continue;
        o[k] = genInput(r, child, depth - 1, definitions);
      }
      if (r.chance(0.3)) {
        const extraSchema = schema.catchall ?? { type: "string" };
        o.extra = genInput(r, extraSchema, depth - 1, definitions);
      }
      return o;
    }
    case "array": {
      if (depth <= 0) return [];
      const n = r.int(4);
      return Array.from({ length: n }, () => genInput(r, schema.items, depth - 1, definitions));
    }
    case "tuple":
      return (schema.items ?? []).map((it) => genInput(r, it, depth - 1, definitions));
    case "union":
      return genInput(r, r.pick(schema.oneOf), depth - 1, definitions);
    case "intersection":
      return genInput(r, schema.allOf[0], depth - 1, definitions);
    case "record": {
      if (depth <= 0) return {};
      const n = r.int(3);
      const o = {};
      for (let i = 0; i < n; i++) {
        const key = r.chance(0.15) ? "" : `k${i}`;
        o[key] = genInput(r, schema.value, depth - 1, definitions);
      }
      return o;
    }
    case "preprocess": {
      const fn = schema.transforms?.[0]?.fn;
      if (fn === "money_to_cents") return r.chance(0.6) ? "12.34" : 12.34;
      if (fn === "digits_only") return r.chance(0.6) ? "123-45" : genString(r);
      if (fn === "date_parse") return r.chance(0.6) ? "12/25/2024" : genString(r);
      return genInput(r, schema.schema, depth - 1, definitions);
    }
    case "catch":
      return r.chance(0.5) ? genInput(r, schema.schema, depth - 1, definitions) : genScalar(r);
    case "ref":
      return definitions?.Def ? genInput(r, definitions.Def, depth - 1, definitions) : genScalar(r);
    default:
      return genScalar(r);
  }
}

// ---------------------------------------------------------------------------
// Cross-field rule generation (solve, then perturb).
//
// Builds an object of numeric fields, lays down a chain of cross-field rules
// over them (ordering / equality / sum / presence / exactly-one), solves a
// satisfying assignment in which every field is present, then emits either the
// satisfier (valid path) or a perturbed input (violation path). All paths are
// JSON pointers (/f0), matching the runtime's resolution. The differential
// oracle does the checking; the solver just keeps generation in the interesting
// region instead of producing vacuous, never-referenced rules.
// ---------------------------------------------------------------------------

function genRulesCase(r, forceValid = false) {
  const n = r.int(3) + 2; // 2..4 fields
  const fields = Array.from({ length: n }, (_, i) => `f${i}`);
  const ptr = (f) => `/${f}`;
  const properties = {};
  for (const f of fields) properties[f] = { type: "number", required: false };

  const rules = [];
  const solution = {}; // a fully-present satisfying assignment
  const family = r.pick(["order", "equal", "sum", "presence", "exactly_one"]);

  if (family === "order") {
    const op = r.pick(["gt_field", "gte_field", "lt_field", "lte_field"]);
    for (let i = 0; i + 1 < n; i++)
      rules.push({
        pred: { type: op, left: ptr(fields[i]), right: ptr(fields[i + 1]) },
        error: { code: "ORD", message: "o" },
      });
    const start = r.int(50);
    for (let i = 0; i < n; i++)
      solution[fields[i]] = op.startsWith("gt") ? start + (n - 1 - i) * 3 : start + i * 3;
  } else if (family === "equal") {
    for (let i = 0; i + 1 < n; i++)
      rules.push({
        pred: { type: "eq_fields", left: ptr(fields[i]), right: ptr(fields[i + 1]) },
        error: { code: "EQ", message: "e" },
      });
    const v = r.int(100);
    for (const f of fields) solution[f] = v;
  } else if (family === "sum") {
    const target = fields[0];
    const parts = fields.slice(1).length ? fields.slice(1) : [fields[0]];
    if (r.chance(0.5)) {
      rules.push({
        pred: { type: "sum_equals", paths: parts.map(ptr), target: ptr(target) },
        error: { code: "SUM", message: "s" },
      });
      let sum = 0;
      for (const f of parts) {
        const v = r.int(40);
        solution[f] = v;
        sum += v;
      }
      solution[target] = sum;
    } else {
      const value = parts.length * 10;
      rules.push({
        pred: { type: "sum_equals_value", paths: parts.map(ptr), value },
        error: { code: "SUMV", message: "s" },
      });
      let remaining = value;
      for (let i = 0; i < parts.length; i++) {
        const v = i === parts.length - 1 ? remaining : r.int(remaining + 1);
        solution[parts[i]] = v;
        remaining -= v;
      }
      if (!(target in solution)) solution[target] = r.int(100);
    }
  } else if (family === "presence") {
    const op = r.chance(0.5) ? "required_with" : "required_without";
    const key = op === "required_with" ? "with" : "without";
    for (let i = 0; i + 1 < n; i++)
      rules.push({
        pred: { type: op, field: ptr(fields[i]), [key]: ptr(fields[i + 1]) },
        error: { code: "PRES", message: "p" },
      });
    for (const f of fields) solution[f] = r.int(100); // all present satisfies both forms
  } else {
    rules.push({
      pred: { type: "exactly_one_of", paths: fields.map(ptr) },
      error: { code: "XOR", message: "x" },
    });
    solution[r.pick(fields)] = r.int(100); // exactly one present
  }

  // Optionally fold all rule predicates into one `and` (exercises combinators
  // without changing the satisfying assignment).
  let finalRules = rules;
  if (rules.length > 1 && r.chance(0.3))
    finalRules = [
      { pred: { type: "and", predicates: rules.map((x) => x.pred) }, error: { code: "AND", message: "a" } },
    ];

  const schema = { version: "1.0", root: { type: "object", properties, rules: finalRules } };

  const input = { ...solution };
  if (!forceValid) {
    const mode = r.int(4); // 0: satisfier; 1: nudge; 2: drop; 3: add
    const keys = Object.keys(input);
    if (mode === 1 && keys.length) {
      const f = r.pick(keys);
      input[f] = input[f] + (r.chance(0.5) ? 1000 : -1000);
    } else if (mode === 2 && keys.length) {
      delete input[r.pick(keys)];
    } else if (mode === 3) {
      const absent = fields.filter((f) => !(f in input));
      if (absent.length) input[r.pick(absent)] = r.int(100);
    }
  }
  return { schema, input };
}

function genCase(r) {
  // Mix generic structural cases with targeted scenarios for historically
  // under-generated behavior: call predicates, defaults, refs, wrapper output,
  // parse-sensitive rules, and numeric boundaries.
  const roll = r.int(10);
  if (roll < 3) return genRulesCase(r);
  if (roll < 5) return genTargetedCase(r);
  // A simple, non-cyclic ref definition is available to any generated ref.
  const definitions = { Def: genType(r, 0) };
  const root = genType(r, 2);
  const schema = { version: "1.0", definitions, root };
  const input = genInput(r, root, 3, definitions);
  return { schema, input };
}

function genSerdeCase(r) {
  const family = r.pick(["generated", "legacy_fields", "template_constraint", "defaults", "metadata"]);
  if (family === "legacy_fields") {
    return {
      mode: "serde",
      schema: {
        version: "1.0",
        root: {
          type: "object",
          fields: {
            name: { type: "string", required: true },
            age: { type: "integer", required: false },
          },
        },
      },
    };
  }
  if (family === "template_constraint") {
    return {
      mode: "serde",
      schema: {
        version: "1.0",
        root: {
          type: "string",
          constraints: [
            { type: "minLength", value: 2, source: "iris" },
            { type: "format", value: "decimal-2" },
            { type: "enum", value: ["A", "B"] },
          ],
        },
      },
    };
  }
  if (family === "defaults") {
    return {
      mode: "serde",
      schema: {
        version: "1.0",
        root: {
          type: "object",
          additional_properties: false,
          properties: {
            amount: { type: "money" },
            pct: { type: "percentage" },
            code: {
              type: "string",
              constraints: [
                {
                  pred: { type: "min_len", len: 1 },
                  error: { code: "MIN", message: "min" },
                },
              ],
            },
          },
        },
      },
    };
  }
  if (family === "metadata") {
    return {
      mode: "serde",
      schema: {
        version: "1.0",
        id: "contact",
        title: "Contact",
        description: "Example",
        sections: { main: { title: "Main", order: 1 } },
        root: {
          type: "object",
          description: "Contact fields",
          properties: {
            email: {
              type: "string",
              label: "Email",
              section: "main",
              render: { type: "text", page: 0, x: 1, y: 2 },
            },
          },
        },
      },
    };
  }
  const definitions = { Def: genType(r, 0) };
  const root = genType(r, 2);
  return { mode: "serde", schema: { version: "1.0", definitions, root } };
}

// ---------------------------------------------------------------------------
// Runtimes
// ---------------------------------------------------------------------------

function runTs(c) {
  try {
    const options = c.registry === "custom" ? { context: customEvalContext } : undefined;
    const res = validate(c.schema, structuredClone(c.input), options);
    return {
      valid: res.valid,
      value: res.value,
      errors: res.errors,
      warnings: res.warnings,
    };
  } catch (e) {
    return { error: String(e && e.message ? e.message : e) };
  }
}

function runRustBatch(cases) {
  const file = join(mkdtempSync(join(tmpdir(), "wf-fuzz-")), "cases.ndjson");
  writeFileSync(file, cases.map((c) => JSON.stringify(c)).join("\n"));
  const out = execFileSync(DRIVER, [file], { encoding: "utf8", maxBuffer: 1 << 28 });
  return out.split("\n").filter(Boolean).map((l) => JSON.parse(l));
}

function runTsSerde(c) {
  try {
    const parsed = parseSchema(schemaToJSON(c.schema));
    const reparsed = parseSchema(schemaToJSON(parsed));
    return { ok: true, schema: canonicalizeSchema(reparsed) };
  } catch (e) {
    return { parse_error: String(e && e.message ? e.message : e) };
  }
}

function runRustSerdeBatch(cases) {
  return runRustBatch(cases).map((result) => {
    if (result.ok && result.schema) return { ok: true, schema: canonicalizeSchema(result.schema) };
    return result;
  });
}

// ---------------------------------------------------------------------------
// Comparison (numeric-aware deep equality, mirroring json_value_eq / isEqualValue)
// ---------------------------------------------------------------------------

function normalizeIssues(issues) {
  return [...(issues ?? [])]
    .map((issue) => ({
      code: String(issue.code),
      path: String(issue.path ?? "/"),
      severity: String(issue.severity ?? "error"),
    }))
    .sort((a, b) =>
      `${a.path}\0${a.code}\0${a.severity}`.localeCompare(
        `${b.path}\0${b.code}\0${b.severity}`,
      ),
    );
}

function normalizeRuntimeResult(result) {
  return {
    valid: result.valid,
    value: result.value,
    errors: normalizeIssues(result.errors),
    warnings: normalizeIssues(result.warnings),
  };
}

function canonicalizeSchema(value) {
  const cloned = structuredClone(value);
  canonicalizeValue(cloned);
  return cloned;
}

function canonicalizeValue(value) {
  if (Array.isArray(value)) {
    for (const item of value) canonicalizeValue(item);
    return;
  }
  if (!value || typeof value !== "object") return;

  for (const item of Object.values(value)) canonicalizeValue(item);

  if (value.fields && !value.properties) {
    value.properties = value.fields;
    delete value.fields;
  }
  if (value.required === true) delete value.required;
  if (value.severity === "error") delete value.severity;
  if (value.additional_properties === false) delete value.additional_properties;
  if (value.allow_over_100 === false) delete value.allow_over_100;
  if ((value.type === "money" || value.type === "currency") && value.scale === 2) delete value.scale;
  if (value.type === "percentage" && value.format === "decimal") delete value.format;
  if (value.fn === "money_to_cents" && value.scale === 2) delete value.scale;
  if (value.fn === "format_thousands" && value.separator === ",") delete value.separator;

  for (const [key, child] of Object.entries(value)) {
    if (Array.isArray(child) && child.length === 0) delete value[key];
    else if (child && typeof child === "object" && !Array.isArray(child) && Object.keys(child).length === 0)
      delete value[key];
  }
}

function hasIssueCode(result, code) {
  return [...(result.errors ?? []), ...(result.warnings ?? [])].some((issue) => issue.code === code);
}

function genOracleCase(r) {
  const generators = [
    () => genTemplateLiteralCase(r),
    () => genCallPredicateCase(r),
    () => genValuePredicateCase(r),
  ];
  for (let i = 0; i < 100; i++) {
    const c = r.pick(generators)();
    if (c.expectedRuleCode) return c;
  }
  return {
    schema: {
      version: "1.0",
      root: {
        type: "object",
        properties: { status: { type: "string", required: false } },
        rules: [hardConstraint({ type: "eq", path: "/status", value: "paid" }, "EQ", "eq")],
      },
    },
    input: { status: "draft" },
    expectedRuleCode: "EQ",
  };
}

function valueEq(a, b) {
  if (typeof a === "number" && typeof b === "number") return a === b || (Number.isNaN(a) && Number.isNaN(b));
  if (Array.isArray(a) && Array.isArray(b))
    return a.length === b.length && a.every((x, i) => valueEq(x, b[i]));
  if (a && b && typeof a === "object" && typeof b === "object") {
    const ka = Object.keys(a), kb = Object.keys(b);
    return ka.length === kb.length && ka.every((k) => k in b && valueEq(a[k], b[k]));
  }
  return a === b;
}

// Returns null if the runtimes agree, or a short reason string if they diverge.
function disagreement(ts, rust) {
  const tsOutOfBand = outOfBandKindTs(ts);
  const rustOutOfBand = outOfBandKindRust(rust);
  if (tsOutOfBand || rustOutOfBand) {
    if (tsOutOfBand && tsOutOfBand === rustOutOfBand) return null;
    return `out-of-band mismatch (ts ${tsOutOfBand ?? "none"}, rust ${rustOutOfBand ?? "none"})`;
  }

  const outOfBand = [];
  if ("error" in ts) outOfBand.push(`ts threw: ${ts.error}`);
  if ("line_error" in rust) outOfBand.push(`rust line error: ${rust.line_error}`);
  if ("parse_error" in rust) outOfBand.push(`rust parse error: ${rust.parse_error}`);
  if ("err" in rust) outOfBand.push(`rust validate error: ${rust.err}`);
  if ("panic" in rust) outOfBand.push("rust panicked");
  if (outOfBand.length) return `out-of-band failure (${outOfBand.join("; ")})`;

  const t = normalizeRuntimeResult(ts);
  const r = normalizeRuntimeResult(rust);
  if (t.valid !== r.valid) return `valid mismatch (ts ${t.valid}, rust ${r.valid})`;
  if (!valueEq(t.errors, r.errors))
    return `errors mismatch (ts ${JSON.stringify(t.errors)}, rust ${JSON.stringify(r.errors)})`;
  if (!valueEq(t.warnings, r.warnings))
    return `warnings mismatch (ts ${JSON.stringify(t.warnings)}, rust ${JSON.stringify(r.warnings)})`;
  if (!valueEq(t.value, r.value))
    return `value mismatch (ts ${JSON.stringify(ts.value)}, rust ${JSON.stringify(rust.value)})`;
  return null;
}

function serdeDisagreement(ts, rust) {
  if ("panic" in rust) return "rust panicked";
  if ("line_error" in rust) return `rust line error: ${rust.line_error}`;

  const tsOk = ts.ok === true;
  const rustOk = rust.ok === true;
  if (tsOk !== rustOk)
    return `parse acceptance mismatch (ts ${tsOk ? "ok" : "err"}, rust ${rustOk ? "ok" : "err"})`;
  if (!tsOk && !rustOk) return null;
  if (!valueEq(ts.schema, rust.schema))
    return `schema mismatch (ts ${JSON.stringify(ts.schema)}, rust ${JSON.stringify(rust.schema)})`;
  return null;
}

function loadKnownDivergenceCases() {
  return readdirSync(CONFORMANCE_CASES)
    .filter((name) => name.endsWith(".json") && !name.startsWith("_"))
    .map((name) => {
      const fixture = JSON.parse(readFileSync(join(CONFORMANCE_CASES, name), "utf8"));
      return { file: name, fixture };
    })
    .filter(({ fixture }) => fixture.status === "known_divergence");
}

function matchesPinnedCurrent(actual, expected) {
  if (!expected || typeof expected !== "object") return false;
  const pinnedFields = ["valid", "value", "errors", "warnings"];
  if (!pinnedFields.some((field) => Object.prototype.hasOwnProperty.call(expected, field)))
    return false;
  if (expected.valid !== undefined && actual.valid !== expected.valid) return false;
  if (expected.value !== undefined && !valueEq(actual.value, expected.value)) return false;
  if (expected.errors !== undefined && !valueEq(normalizeIssues(actual.errors), normalizeIssues(expected.errors)))
    return false;
  if (expected.warnings !== undefined && !valueEq(normalizeIssues(actual.warnings), normalizeIssues(expected.warnings)))
    return false;
  return true;
}

function outOfBandKindTs(result) {
  if (!("error" in result)) return null;
  if (/Unknown predicate/.test(result.error)) return "unknown_predicate";
  return null;
}

function outOfBandKindRust(result) {
  if (!("err" in result)) return null;
  if (/UnknownPredicate/.test(result.err)) return "unknown_predicate";
  return null;
}

// ---------------------------------------------------------------------------
// Shrinking: reduce a failing case while it still reproduces.
// ---------------------------------------------------------------------------

function reproduces(c) {
  if (!isJsonValue(c.input)) return false;
  const ts = runTs(c);
  const rust = runRustBatch([c])[0];
  return disagreement(ts, rust) !== null;
}

function* candidates(c) {
  const s = c.schema.root;
  // Drop transforms / constraints.
  if (s.transforms?.length) {
    const root = { ...s };
    delete root.transforms;
    yield { ...c, schema: { ...c.schema, root } };
  }
  if (s.constraints?.length) {
    const root = { ...s };
    delete root.constraints;
    yield { ...c, schema: { ...c.schema, root } };
  }
  // Replace root with a child schema + corresponding input slice.
  if (s.type === "object" && c.input && typeof c.input === "object" && !Array.isArray(c.input))
    for (const [k, child] of Object.entries(objectProperties(s)))
      if (Object.prototype.hasOwnProperty.call(c.input, k))
        yield { schema: { ...c.schema, root: child }, input: c.input[k] };
  if (s.type === "array" && s.items && Array.isArray(c.input) && c.input.length)
    yield { schema: { ...c.schema, root: s.items }, input: c.input[0] };
  if (s.type === "union" && s.oneOf)
    for (const variant of s.oneOf) yield { schema: { ...c.schema, root: variant }, input: c.input };
  if (s.type === "tuple" && s.items && Array.isArray(c.input))
    for (let i = 0; i < s.items.length; i++)
      yield { schema: { ...c.schema, root: s.items[i] }, input: c.input[i] };
  if (s.type === "record" && s.value && c.input && typeof c.input === "object" && !Array.isArray(c.input)) {
    const values = Object.values(c.input);
    if (values.length) yield { schema: { ...c.schema, root: s.value }, input: values[0] };
  }
  if ((s.type === "preprocess" || s.type === "catch") && s.schema)
    yield { schema: { ...c.schema, root: s.schema }, input: c.input };
  if (s.type === "intersection" && Array.isArray(s.allOf))
    for (const member of s.allOf)
      yield { schema: { ...c.schema, root: member }, input: c.input };
  // Drop object cross-field rules to isolate structure from rule logic.
  if (s.type === "object" && s.rules?.length) {
    const root = { ...s };
    delete root.rules;
    yield { ...c, schema: { ...c.schema, root } };
  }
}

function shrink(c) {
  let best = c;
  let improved = true;
  let guard = 0;
  while (improved && guard++ < 200) {
    improved = false;
    for (const cand of candidates(best)) {
      try {
        if (reproduces(cand)) {
          best = cand;
          improved = true;
          break;
        }
      } catch {
        /* candidate invalid (e.g. undefined input), skip */
      }
    }
  }
  return best;
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

if (!existsSync(DRIVER)) {
  console.error(`Rust driver not found at ${DRIVER}`);
  console.error("Build it first:  cargo build --example fuzz_driver --release");
  process.exit(2);
}

// --dump N: print N sample cases with both runtimes' results, then a coverage
// histogram of root types. Used to confirm the generator is diverse.
const DUMP = Number(arg("dump", 0));
if (DUMP > 0) {
  const r = mk(mulberry32(SEED >>> 0));
  const hist = {};
  for (let i = 0; i < DUMP; i++) {
    const c = genCase(r);
    hist[c.schema.root.type] = (hist[c.schema.root.type] ?? 0) + 1;
    if (i < 12) {
      const ts = runTs(c);
      const rust = runRustBatch([c])[0];
      const tr = (c.schema.root.transforms ?? []).map((t) => t.fn).join(",");
      console.log(
        `${c.schema.root.type.padEnd(10)}${tr ? `[${tr}]` : ""}\tinput=${JSON.stringify(c.input)?.slice(0, 40)}\tts=${ts.valid ?? "x"} rust=${rust.valid ?? "x"}`,
      );
    }
  }
  console.log("\nroot-type histogram:", JSON.stringify(hist));
  process.exit(0);
}

// --rulescheck N: prove the cross-field rule generator is not vacuous. Solved
// satisfiers should validate (in TS, the reference) and random cases should
// often fail, and both runtimes must agree throughout. A satisfier valid-rate
// near 100% means the rules genuinely reference present fields at their
// boundary, rather than no-op rules over absent paths.
const RULESCHECK = Number(arg("rulescheck", 0));
if (RULESCHECK > 0) {
  const r = mk(mulberry32(SEED >>> 0));
  const sats = Array.from({ length: RULESCHECK }, () => genRulesCase(r, true));
  const vios = Array.from({ length: RULESCHECK }, () => genRulesCase(r, false));
  const satTs = sats.map(runTs);
  const satRust = runRustBatch(sats);
  const vioTs = vios.map(runTs);
  const vioRust = runRustBatch(vios);

  const fams = {};
  for (const s of sats) {
    const f = s.schema.root.rules[0]?.pred?.type ?? "?";
    fams[f] = (fams[f] ?? 0) + 1;
  }
  const satValid = satTs.filter((t) => t.valid).length;
  const satDisagree = satTs.filter((t, i) => disagreement(t, satRust[i])).length;
  const vioInvalid = vioTs.filter((t) => t.valid === false).length;
  const vioDisagree = vioTs.filter((t, i) => disagreement(t, vioRust[i])).length;

  console.log(`rules generator self-check (${RULESCHECK} satisfiers + ${RULESCHECK} random):`);
  console.log(`  satisfiers valid in TS: ${satValid}/${RULESCHECK} (want ~all)   disagreements: ${satDisagree}`);
  console.log(`  random cases invalid in TS: ${vioInvalid}/${RULESCHECK}           disagreements: ${vioDisagree}`);
  console.log("  rule predicate seen:", JSON.stringify(fams));
  process.exit(satDisagree + vioDisagree === 0 ? 0 : 1);
}

// --oraclecheck N: generate targeted cases that intentionally break exactly
// one known predicate/constraint family and assert the expected stable code is
// emitted by both runtimes. This complements differential comparison: parity
// says the runtimes agree; this says the generator actually hit the rule it
// meant to hit.
const ORACLECHECK = Number(arg("oraclecheck", 0));
if (ORACLECHECK > 0) {
  const r = mk(mulberry32(SEED >>> 0));
  const cases = Array.from({ length: ORACLECHECK }, () => genOracleCase(r));
  const tsResults = cases.map(runTs);
  const rustResults = runRustBatch(cases);

  let disagreements = 0;
  let missingExpected = 0;
  let firstFailure = null;
  const families = {};
  for (let i = 0; i < cases.length; i++) {
    const reason = disagreement(tsResults[i], rustResults[i]);
    if (reason) disagreements++;
    const expected = cases[i].expectedRuleCode;
    families[expected] = (families[expected] ?? 0) + 1;
    if (!hasIssueCode(tsResults[i], expected) || !hasIssueCode(rustResults[i], expected)) {
      missingExpected++;
      firstFailure ??= { index: i, reason, case: cases[i], ts: tsResults[i], rust: rustResults[i] };
    } else if (reason) {
      firstFailure ??= { index: i, reason, case: cases[i], ts: tsResults[i], rust: rustResults[i] };
    }
  }

  console.log(`oracle self-check (${ORACLECHECK} targeted invalid cases):`);
  console.log(`  disagreements: ${disagreements}`);
  console.log(`  missing expected code: ${missingExpected}`);
  console.log("  expected code histogram:", JSON.stringify(families));
  if (firstFailure) {
    console.log(`  first: #${firstFailure.index} ${firstFailure.reason ?? "missing expected code"}`);
    console.log(JSON.stringify({ case: firstFailure.case, ts: firstFailure.ts, rust: firstFailure.rust }, null, 2));
  }
  process.exit(disagreements + missingExpected === 0 ? 0 : 1);
}

// --serdecheck N: compare schema parse/serialize behavior. TS parses the raw
// schema, serializes it, reparses it, then canonicalizes. Rust deserializes the
// same schema through serde and serializes it back. This catches parser/serde
// drift separately from validation semantics.
const SERDECHECK = Number(arg("serdecheck", 0));
if (SERDECHECK > 0) {
  const r = mk(mulberry32(SEED >>> 0));
  const cases = Array.from({ length: SERDECHECK }, () => genSerdeCase(r));
  const tsResults = cases.map(runTsSerde);
  const rustResults = runRustSerdeBatch(cases);

  const failures = [];
  for (let i = 0; i < cases.length; i++) {
    const reason = serdeDisagreement(tsResults[i], rustResults[i]);
    if (reason) failures.push({ index: i, reason, case: cases[i], ts: tsResults[i], rust: rustResults[i] });
  }

  console.log(`serde self-check (${SERDECHECK} schema cases):`);
  console.log(`  disagreements: ${failures.length}`);
  if (failures.length) {
    const f = failures[0];
    console.log(`  first: #${f.index} ${f.reason}`);
    console.log(JSON.stringify({ schema: f.case.schema, ts: f.ts, rust: f.rust }, null, 2));
  }
  process.exit(failures.length === 0 ? 0 : 1);
}

// --divergencecheck: assert documented known-divergence fixtures still match
// their pinned current outputs. This is for non-portable cases that the normal
// random generator deliberately avoids.
const DIVERGENCECHECK = arg("divergencecheck", false) === true;
if (DIVERGENCECHECK) {
  const fixtures = loadKnownDivergenceCases();
  const cases = fixtures.map(({ fixture }) => ({ schema: fixture.schema, input: fixture.input }));
  const tsResults = cases.map(runTs);
  const rustResults = runRustBatch(cases);
  const failures = [];

  for (let i = 0; i < fixtures.length; i++) {
    const current = fixtures[i].fixture.current ?? {};
    if (!matchesPinnedCurrent(tsResults[i], current.ts))
      failures.push({ file: fixtures[i].file, runtime: "ts", expected: current.ts, actual: tsResults[i] });
    if (!matchesPinnedCurrent(rustResults[i], current.rust))
      failures.push({ file: fixtures[i].file, runtime: "rust", expected: current.rust, actual: rustResults[i] });
  }

  console.log(`known-divergence check (${fixtures.length} fixtures):`);
  console.log(`  mismatches: ${failures.length}`);
  if (failures.length) console.log(JSON.stringify(failures[0], null, 2));
  process.exit(failures.length === 0 ? 0 : 1);
}

console.log(
  `differential fuzz: seed=${SEED} batches=${BATCHES} cases/batch=${CASES} (${BATCHES * CASES} total)`,
);

let total = 0;
const failures = [];

for (let b = 0; b < BATCHES; b++) {
  const r = mk(mulberry32((SEED * 2654435761 + b * 40503) >>> 0));
  const cases = Array.from({ length: CASES }, () => genCase(r));
  const tsResults = cases.map(runTs);
  const rustResults = runRustBatch(cases);
  total += cases.length;

  for (let i = 0; i < cases.length; i++) {
    const reason = disagreement(tsResults[i], rustResults[i]);
    if (reason) {
      failures.push({ batch: b, index: i, reason, case: cases[i] });
      if (VERBOSE) console.log(`  [b${b} #${i}] ${reason}`);
    }
  }
  process.stdout.write(`\r  ${total} cases, ${failures.length} disagreements`);
}
process.stdout.write("\n");

if (failures.length === 0) {
  console.log(`\nNo disagreements across ${total} cases. Both runtimes agree.`);
  process.exit(0);
}

console.log(`\n${failures.length} disagreement(s). Shrinking the first...\n`);
const f = failures[0];
const minimal = shrink(f.case);
const ts = runTs(minimal);
const rust = runRustBatch([minimal])[0];

console.log(`reason: ${disagreement(ts, rust)}`);
console.log(`ts:   ${JSON.stringify(ts)}`);
console.log(`rust: ${JSON.stringify(rust)}`);
console.log("\nminimal reproducer (drop into conformance/cases/ to pin):");
console.log(
  JSON.stringify(
    {
      name: "fuzz-found",
      note: `differential fuzzer, seed ${SEED} batch ${f.batch}: ${f.reason}`,
      schema: minimal.schema,
      input: minimal.input,
      expect: { valid: ts.valid ?? false },
      status: "known_divergence",
      current: { ts, rust },
    },
    null,
    2,
  ),
);
process.exit(1);

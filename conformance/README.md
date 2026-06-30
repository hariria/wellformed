# Cross-runtime conformance suite

The load-bearing promise of wellformed is that the same JSON IR validates
**identically** in the TypeScript and Rust runtimes. These fixtures are the
executable proof. Each case is run through both runtimes by:

- TypeScript: `typescript/packages/wellformed/src/conformance.test.ts` (vitest)
- Rust: `wellformed/tests/conformance.rs` (`cargo test -p wellformed --test conformance`)

## Fixture format

One JSON file per case in `cases/`:

```jsonc
{
  "name": "human-readable id",
  "note": "what this pins and why",
  "schema": { "version": "1.0", "root": { /* IR */ } },
  "input": <any JSON value>,
  "expect": { "valid": true, "value": <optional normalized value>, "code": <optional error code> },
  "status": "agree" | "known_divergence",
  // required only when status === "known_divergence":
  "current": { "ts": { "valid": false }, "rust": { "valid": true } },
  // optional guards for cases that crash or are rejected at parse in a runtime:
  "skip_ts": false,
  "skip_rust": false
}
```

Files whose names start with `_` are ignored by both runners. Use that prefix
for local probes or generated stress cases that are not part of CI conformance.

## Status semantics

- **`agree`**: both runtimes must produce `expect`. A mismatch is a hard test
  failure. This is the target state for every case.
- **`known_divergence`**: the runtimes currently disagree. Each runner asserts
  its runtime matches `current.<ts|rust>`, so the suite stays green and the
  divergence is documented as a living test. When the underlying bug is fixed,
  that runtime no longer matches `current`, the test goes red, and you promote
  the case to `status: "agree"` (deleting `current`). `expect` records the
  agreed-correct behavior the fix should converge on. If the divergence is about
  normalized output, include `current.<rt>.value`; otherwise the fixture only
  pins the `valid` flag.

This lets the suite encode every known divergence today without breaking CI,
while guaranteeing the moment a divergence is closed, the case is upgraded.

Assertions compare `valid` always, `value` (the normalized output) when
`expect.value` / `current.<rt>.value` is present, and `code` when present.

## Number semantics

wellformed targets portable JSON behavior across TypeScript and Rust. JSON
numbers used by enum and literal checks are compared with JavaScript-compatible
number semantics, not exact Rust `serde_json::Number` identity. That means
`1` and `1.0` match. It also means integers beyond JavaScript's safe integer
range follow JS behavior and may collapse to the same numeric value. Prefer
strings for identifiers or exact 64-bit values.

Numeric transforms are guaranteed only inside the portable numeric domain:
finite values where the scaled integer stays within JavaScript's safe integer
range (`abs(value * 10^scale) <= Number.MAX_SAFE_INTEGER`). Extreme values can
hit language-specific rendering or overflow behavior. Record those as
`known_divergence` when useful, but do not treat them as normal conformance
targets.

## Known limits

The runtime suite exercises validation of realistic data, and that class is
closed: there is no input in the portable domain that validates differently
between the runtimes. The following are deliberately-unreconciled limits, not
defects. They do not produce wrong answers on realistic data; each is recorded
here (and, where it has a concrete input, as a `known_divergence` fixture) so it
is tracked rather than forgotten.

- **Numeric extremes (`money-to-cents-overflow`, `format-decimal-huge-magnitude`).**
  Outside the portable numeric domain above, integer overflow and number-to-string
  rendering differ (Rust saturates `i64` / prints full decimals; JS produces large
  floats / switches to exponential notation at `>= 1e21`). Use strings for exact
  64-bit values. See the FAQ.
- **`version` is not enforced.** Both runtimes ignore the top-level schema
  `version`; it is advisory metadata you version and gate on in your application,
  not a compatibility check the runtime performs.
- **`parseSchema` is not guaranteed pure.** The TypeScript `parseSchema` may
  normalize legacy constraint shapes in place, so it is not a lossless,
  side-effect-free round-trip. Treat its input as consumed.
- **Deeply nested data is outside the guaranteed domain.** Recursive schemas
  over realistic data validate identically, but very deep nesting (roughly 100+
  levels) is a ragged edge: TypeScript bounds `$ref` recursion at depth 128,
  Rust bounds total validation depth, and `serde_json` independently rejects
  input nested deeper than 128 at *deserialization* time (before validation
  runs). So a pathologically deep document can deserialize and validate in
  TypeScript while Rust refuses to parse it. Keep documents shallow; this is a
  resource bound, not a validation result.
- **Parse-layer parity lives in the fuzzer.** This suite runs `validate` on an
  already-parsed schema. Parser/serde drift is covered separately by
  `node fuzz/fuzz.mjs --serdecheck`, which compares TS `parseSchema` /
  `schemaToJSON` against Rust serde on generated and legacy schema forms. Truly
  malformed or forward-compatible schema JSON still depends on the chosen
  unknown-field policy (reject vs ignore).

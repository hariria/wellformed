# Differential fuzzer

Two runtimes. One IR. They have to agree, or "write the rules once, validate
anywhere" is a lie.

A test suite checks the cases you thought of. A fuzzer checks the ones you
didn't, which is where the bug you ship always lives. So: take a seed, generate a
random schema and a random input, run both through TypeScript and Rust, and fail
the instant they disagree. Fifty thousand cases a run, in under a second.

## How it works

```
fuzz.mjs (Node)                         examples/fuzz_driver.rs (Rust)
  seed -> random {schema, input}
  run TS validate in-process
  write batch as NDJSON  ───────────▶   read NDJSON, run Rust validate
  read Rust results     ◀───────────    write {valid, value, errors} per line
  diff TS vs Rust (validity, issue codes/paths, normalized value)
  on mismatch: shrink, print fixture
```

One seeded RNG drives everything, so a run is reproducible: same `--seed`, same
cases, and every failure replays exactly. The Rust driver runs each case behind a
panic guard, because a crash is a finding, not a stop. Parse failures and crashes
come back labeled, not counted as passes. A TypeScript throw, a Rust parse error,
a Rust `Err`, a Rust panic: all fail the run.

The generator only builds inputs where both runtimes are supposed to match:
shallow nesting, portable numbers, known predicates, no raw regex, no
pathological ref depth. Outside that, they're allowed to differ, and those cases
are listed in `../conformance/README.md#known-limits`. Generate out there and
every run just rediscovers a known limit and tells you nothing. Stay inside it
and any disagreement is a real bug.

When a value is invalid, it compares the stable fields: `code`, `path`,
`severity`. It ignores the human-readable message. Wording isn't behavior.

## Running

```sh
# one-time: build the Rust driver and the TS package
cargo build --example fuzz_driver --release
pnpm --filter wellformed-ts build

# fuzz (defaults: seed 1, 25 batches x 2000 = 50k cases)
node fuzz/fuzz.mjs
node fuzz/fuzz.mjs --seed 42 --cases 5000 --batches 40   # 200k cases
node fuzz/fuzz.mjs --dump 2000                            # sample coverage, no diff

# focused self-checks
node fuzz/fuzz.mjs --rulescheck 500       # cross-field rules are not no-ops
node fuzz/fuzz.mjs --oraclecheck 500      # intentionally invalid cases emit expected codes
node fuzz/fuzz.mjs --serdecheck 500       # TS parse/schemaToJSON vs Rust serde
node fuzz/fuzz.mjs --divergencecheck      # pinned known-divergence fixtures still match
```

Exit `0` is agreement. Exit `1` is a divergence. Exit `2` means you forgot to
build the driver.

There are also Rust-only cargo-fuzz targets for parser and validator robustness:

```sh
cargo install cargo-fuzz
rustup toolchain install nightly
cargo +nightly fuzz run serde_schema -- -runs=0
cargo +nightly fuzz run validate_json -- -runs=0
```

Those targets do not compare against TypeScript. They feed arbitrary bytes into
Rust serde/schema validation paths and assert no panics plus stable schema
round-trips where parsing succeeds.

## When it finds something

It shrinks the failure to the smallest case that still breaks, then prints it as
a `known_divergence` fixture. Paste that into `../conformance/cases/`, fix the
runtime until it flips to `agree`, re-run with the printed seed. A fuzz finding
becomes a permanent regression test in one copy-paste.

## What it covers, and what it does not

Read a green run carefully. It means "everything generated agrees," not
"everything possible agrees." Here is exactly what that covers.

**Covered by the deterministic differential fuzzer.**

- Generated type shapes: `string`, `number`, `integer`, `int32`, `int64`,
  `uint32`, `uint64`, `boolean`, `money`, `currency`, `decimal`, `percentage`,
  `date`, `enum`, `literal`, `never`, `any`, `object`, `array`, `tuple`,
  `union`, `record`, `preprocess`, `catch`, `intersection`, and bounded `ref`
  scenarios.
- String and numeric transforms, including transform chains, `default`,
  `date_parse`, `replace`, phone, SSN, EIN, IBAN, credit-card, decimal/money
  formatting, and `normalize_*` transforms.
- Constraints and predicates: `min_len`, `max_len`, `range`, `template_literal`,
  `eq`, `in`, `exists`, `and`, `or`, `not`, `implies`, representative built-in
  `call` predicates with args, an unknown `call` predicate, and one custom
  registry predicate in both runtimes.
- Object behavior: `properties`, legacy `fields`, `required`, `unknown_keys`,
  `additional_properties`, `catchall`, scalar/object/array defaults on
  missing/null fields, escaped keys, empty-string keys, invalid known fields plus
  unknown keys, and rules evaluated after property transforms.
- Wrapper behavior: nested `preprocess`, `catch` fallback output, union branch
  output selection, and intersection transform sequencing.
- References: missing refs, direct ref cycles, and finite recursive schemas over
  shallow data.
- Numeric boundaries in the portable domain: safe integer edges, `int64` /
  `uint64` string edges, decimal precision/scale, and rounding cases like
  `-0.5`.
- Parse/serde parity through `--serdecheck`: TS `parseSchema` / `schemaToJSON`
  compared with Rust serde for generated schemas, legacy `fields`, legacy
  template constraints, defaulted fields, aliases, and metadata.
- Expected divergences through `--divergencecheck`: known non-portable outputs
  must keep matching their pinned `current.ts` and `current.rust` values.

A second layer, proptest in `wellformed/tests/properties.rs`, checks things a
parity test can't: properties that should hold inside a single runtime (no
panics, the same answer every time, transforms that keep their promises, and
schemas that survive a save-and-reload).

**Still intentionally limited:**

- Domain `call` coverage is representative, not exhaustive. It does not yet
  enumerate every built-in predicate, every argument shape, or address predicates
  that require optional libpostal behavior.
- Custom registry coverage uses one simple custom predicate. That proves the
  registry path works in both runtimes, not every possible custom predicate
  behavior.
- Error oracles are targeted. `--oraclecheck` knows which code should fail for
  selected predicate families, but the general fuzzer still compares emitted
  errors rather than predicting every failure in multi-error cases.
- Pathologically deep data is outside cross-runtime parity. Rust `serde_json`
  can reject deeply nested JSON before validation runs, while TypeScript can
  parse it and then hit runtime depth guards. That is tracked as a known limit,
  not randomized in the portable generator.
- The Rust cargo-fuzz targets are scaffolded. They need a local
  `cargo-fuzz`/nightly run, and eventually coverage reports, before they become
  a coverage claim.

The cross-field rules were the part that mattered most: that's exactly where the
earlier cross-field equality bug lived, and it's covered now. The list above is
the remaining backlog for future generator work, not stale bookkeeping from this
pass.

Want proof the rules actually do something? `node fuzz/fuzz.mjs --rulescheck N`.
It builds rules, solves inputs that should pass them, and checks they do. Should
be ~all. If that number drops, the rules are no-ops and a green run means
nothing.

## Roadmap

Two layers matter: **parity** (TS == Rust) and **invariants** (things that should
be true of one runtime on its own). Cheapest first.

**1. Property tests. Done.** Some bugs aren't divergences. A panic, a result that
changes between runs, a transform that breaks its own promise: both runtimes can
be wrong the same way, so a parity check can't see them. Each one is checkable
inside a single runtime. `wellformed/tests/properties.rs` uses `proptest` to
assert that `validate` never panics, always gives the same answer, that each
transform keeps its promise (`money_to_cents` gives an integer, `format_decimal(n)`
gives `n` decimals, `trim` leaves no edge whitespace), and that a schema survives
serialize-then-parse unchanged. It runs in `cargo test --workspace`, so it's
already in CI, and shrinks failures for free. One thing it skips on purpose:
re-validating a normalized value isn't expected to be stable, because
`money_to_cents` would just run a second time.

**2. Predicate-rule generation. Done.** `genRulesCase` in `fuzz.mjs` builds
cross-field rules in chains (`a > b`, `b > c`, ...). The trick that keeps it
honest is *solve, then perturb*: build a chain over a few number fields, work out
values that satisfy all of it, then feed either those values (should pass) or a
broken version (nudge a value, drop a field, add a field). Targeted generators
also hit `or`, `not`, `implies`, `eq`, `in`, `exists`, `template_literal`,
representative `call` predicates, unknown calls, and a custom registry
predicate. `--rulescheck` confirms rules are not no-ops; `--oraclecheck`
confirms selected intentionally-invalid cases emit the expected stable codes.

**3. Parser and serde fuzzing. Done, deterministic pass.** `--serdecheck`
compares `schema JSON -> TS parse -> JSON` against
`schema JSON -> Rust serde -> JSON`. It includes generated schemas plus legacy
forms (`fields`, template constraints, defaulted fields, omitted severity) and
metadata. This catches what `validate()` can't: schema compatibility, defaults,
metadata, and parse/serde drift.

**4. Wrapper, object, and transform expansion. Done, first pass.** The generator
now reaches `catchall`, `record`, `preprocess`, `catch`, `intersection`, `date`,
`currency`, `int64` / `uint64`, `never`, transform chains, warning/path metadata,
legacy `fields`, `additional_properties`, and escaped or empty object keys. Keep
expanding here with default transforms, deeper refs, and more targeted wrapper
cases before adding heavier machinery.

**5. Coverage-guided generation. Scaffolded for Rust.** `fuzz/Cargo.toml` adds
two cargo-fuzz targets: `serde_schema` for Rust schema parse/round-trip behavior
and `validate_json` for arbitrary schema/input validation. They are Rust-only
robustness harnesses, not TS/Rust parity checks. A cross-runtime
coverage-guided fuzzer would need a JS oracle bridge; save that until the
deterministic generator stops finding useful bugs.

Coverage-guidance amplifies a good generator. It doesn't replace one. Build the
generator first; only add the engine if the nightly runs stop finding bugs.

## Extending

Add a node type, transform, or predicate to `genType` / `genInput` in `fuzz.mjs`.
Keep new generators in the portable domain unless you mean to pin a new
`known_divergence`. To make a finding permanent, copy its shrunk reproducer into
the conformance suite. Then it runs in CI forever.

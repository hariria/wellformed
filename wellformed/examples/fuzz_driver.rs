//! Differential-fuzzing driver (Rust side).
//!
//! Reads newline-delimited JSON cases from the file given as argv[1]. Each line
//! is `{"schema": <IR>, "input": <value>}`. For each case it parses the schema,
//! runs `validate`, and writes one JSON result line to stdout, in order:
//!
//! - `{"valid": bool, "value": <normalized input>, "errors": [...], "warnings": [...]}` on success
//! - `{"parse_error": "..."}` if the schema does not deserialize
//! - `{"err": "..."}` if `validate` returns an out-of-band error
//! - `{"panic": true}` if validation panics (itself a finding)
//!
//! The TypeScript side (`fuzz/fuzz.mjs`) generates the cases, runs the TS
//! runtime in-process, invokes this binary, and diffs the two result streams.
//! See `fuzz/README.md`.

use std::fs;
use std::io::{self, BufWriter, Write};
use std::panic::{self, AssertUnwindSafe};

use serde_json::{json, Value};
use std::sync::Arc;
use wellformed::runtime::{NamedPredicate, PredicateRegistry};
use wellformed::{validate, validate_with_registry, Schema};

fn main() {
    // Swallow the default panic output; a panic is reported as a result line.
    panic::set_hook(Box::new(|_| {}));

    let path = std::env::args()
        .nth(1)
        .expect("usage: fuzz_driver <cases.ndjson>");
    let data = fs::read_to_string(&path).expect("read cases file");

    let stdout = io::stdout();
    let mut out = BufWriter::new(stdout.lock());

    for line in data.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let result = panic::catch_unwind(AssertUnwindSafe(|| run_case(line)));
        let value = match result {
            Ok(v) => v,
            Err(_) => json!({ "panic": true }),
        };
        writeln!(out, "{value}").expect("write result");
    }
}

fn run_case(line: &str) -> Value {
    let case: Value = match serde_json::from_str(line) {
        Ok(v) => v,
        Err(e) => return json!({ "line_error": e.to_string() }),
    };

    let schema: Schema = match serde_json::from_value(case["schema"].clone()) {
        Ok(s) => s,
        Err(e) => return json!({ "parse_error": e.to_string() }),
    };

    if case.get("mode").and_then(Value::as_str) == Some("serde") {
        return match serde_json::to_value(&schema) {
            Ok(schema) => json!({ "ok": true, "schema": schema }),
            Err(e) => json!({ "err": e.to_string() }),
        };
    }

    let mut input = case["input"].clone();
    let result = if case.get("registry").and_then(Value::as_str) == Some("custom") {
        let mut registry = PredicateRegistry::with_builtins();
        registry.register(Arc::new(CustomEvenPredicate));
        validate_with_registry(&schema, &mut input, &registry)
    } else {
        validate(&schema, &mut input)
    };

    match result {
        Ok(r) => {
            let valid = r.is_valid();
            json!({
                "valid": valid,
                "value": input,
                "errors": r.errors,
                "warnings": r.warnings,
            })
        }
        Err(e) => json!({ "err": format!("{e:?}") }),
    }
}

struct CustomEvenPredicate;

impl NamedPredicate for CustomEvenPredicate {
    fn name(&self) -> &str {
        "custom_is_even"
    }

    fn evaluate(&self, value: &Value, _args: &Value) -> bool {
        match value {
            Value::Number(n) => n.as_i64().is_some_and(|x| x % 2 == 0),
            Value::String(s) => s.parse::<i64>().is_ok_and(|x| x % 2 == 0),
            _ => false,
        }
    }
}

//! Single-runtime property tests (the proptest layer).
//!
//! The conformance suite and the differential fuzzer check cross-runtime
//! *parity*. These check invariants that must hold *within* the Rust runtime
//! alone, which a differential oracle structurally cannot see (both runtimes
//! could be wrong identically): that `validate` never panics, is deterministic,
//! that each transform holds its postcondition, and that a schema survives a
//! serialize/parse round-trip unchanged.
//!
//! Run with `cargo test -p wellformed --test properties` (included in
//! `cargo test --workspace`, so it gates CI). Failures shrink automatically.

use proptest::prelude::*;
use serde_json::{json, Map, Value};
use wellformed::{validate, Schema};

// ---------------------------------------------------------------------------
// Strategies: build valid schema JSON and arbitrary inputs.
// ---------------------------------------------------------------------------

fn scalar() -> impl Strategy<Value = Value> {
    prop_oneof![
        Just(Value::Null),
        any::<bool>().prop_map(Value::Bool),
        (-1000i64..1000).prop_map(|n| json!(n)),
        (-100000i64..100000).prop_map(|n| json!(n as f64 / 100.0)),
        "[a-zA-Z0-9 .-]{0,10}".prop_map(Value::String),
    ]
}

/// A schema's `root`/node as JSON, valid by construction.
fn type_schema() -> impl Strategy<Value = Value> {
    let leaf = prop_oneof![
        Just(json!({ "type": "string" })),
        Just(json!({ "type": "number" })),
        Just(json!({ "type": "integer" })),
        Just(json!({ "type": "int32" })),
        Just(json!({ "type": "uint32" })),
        Just(json!({ "type": "boolean" })),
        Just(json!({ "type": "money", "scale": 2 })),
        Just(json!({ "type": "decimal", "scale": 2 })),
        Just(json!({ "type": "percentage" })),
        Just(json!({ "type": "any" })),
        // string with a single transform
        prop_oneof![
            Just(json!({ "type": "string", "transforms": [{ "fn": "trim" }] })),
            Just(json!({ "type": "string", "transforms": [{ "fn": "lower" }] })),
            Just(json!({ "type": "string", "transforms": [{ "fn": "upper" }] })),
            Just(json!({ "type": "string", "transforms": [{ "fn": "money_to_cents", "scale": 2 }] })),
            Just(json!({ "type": "string", "transforms": [{ "fn": "format_decimal", "places": 2 }] })),
        ],
        // string with a length constraint
        (0u64..6).prop_map(|n| json!({
            "type": "string",
            "constraints": [{ "pred": { "type": "min_len", "len": n }, "error": { "code": "MIN", "message": "m" } }]
        })),
        prop::collection::vec(scalar(), 1..4).prop_map(|vs| json!({ "type": "enum", "values": vs })),
        scalar().prop_map(|v| json!({ "type": "literal", "value": v })),
    ];

    leaf.prop_recursive(3, 32, 4, |inner| {
        prop_oneof![
            prop::collection::vec(inner.clone(), 1..4).prop_map(|children| {
                let mut props = Map::new();
                for (i, c) in children.into_iter().enumerate() {
                    props.insert(format!("f{i}"), c);
                }
                json!({ "type": "object", "properties": props })
            }),
            inner
                .clone()
                .prop_map(|items| json!({ "type": "array", "items": items })),
            prop::collection::vec(inner.clone(), 1..4)
                .prop_map(|items| json!({ "type": "tuple", "items": items })),
            prop::collection::vec(inner.clone(), 2..4)
                .prop_map(|variants| json!({ "type": "union", "oneOf": variants })),
        ]
    })
}

fn schema_json() -> impl Strategy<Value = Value> {
    type_schema().prop_map(|root| json!({ "version": "1.0", "root": root }))
}

/// An arbitrary input value (independent of the schema, so both valid and
/// invalid paths are exercised).
fn input() -> impl Strategy<Value = Value> {
    let leaf = scalar();
    leaf.prop_recursive(3, 24, 4, |inner| {
        prop_oneof![
            prop::collection::vec(inner.clone(), 0..4).prop_map(Value::Array),
            prop::collection::vec(("[a-z]{1,3}", inner.clone()), 0..4)
                .prop_map(|kvs| Value::Object(kvs.into_iter().collect())),
        ]
    })
}

fn parse(schema_json: &Value) -> Option<Schema> {
    serde_json::from_value(schema_json.clone()).ok()
}

// ---------------------------------------------------------------------------
// Properties
// ---------------------------------------------------------------------------

proptest! {
    // validate must never panic on any well-formed schema and any input.
    #[test]
    fn validate_never_panics(s in schema_json(), v in input()) {
        if let Some(schema) = parse(&s) {
            let mut value = v;
            let _ = validate(&schema, &mut value);
        }
    }

    // validate is a pure function: same input twice yields the same outcome and
    // the same normalized value.
    #[test]
    fn validate_is_deterministic(s in schema_json(), v in input()) {
        if let Some(schema) = parse(&s) {
            let mut a = v.clone();
            let mut b = v;
            let ra = validate(&schema, &mut a);
            let rb = validate(&schema, &mut b);
            match (ra, rb) {
                (Ok(ra), Ok(rb)) => {
                    prop_assert_eq!(ra.is_valid(), rb.is_valid());
                    prop_assert_eq!(a, b);
                }
                (Err(_), Err(_)) => {}
                _ => prop_assert!(false, "validate returned Ok on one call and Err on another"),
            }
        }
    }

    // A parsed schema survives serialize -> parse unchanged: re-serializing the
    // reparsed schema is a fixed point. Catches lossy/non-idempotent
    // (de)serialization of the IR.
    #[test]
    fn schema_serialization_round_trips(s in schema_json()) {
        if let Some(schema) = parse(&s) {
            let once = serde_json::to_value(&schema).expect("serialize schema");
            let reparsed: Schema = serde_json::from_value(once.clone()).expect("reparse serialized schema");
            let twice = serde_json::to_value(&reparsed).expect("serialize reparsed schema");
            prop_assert_eq!(once, twice);
        }
    }

    // money_to_cents postcondition: a parseable input becomes an integer; an
    // unparseable one is left unchanged. Never a non-integer number.
    #[test]
    fn money_to_cents_yields_integer_or_passthrough(n in -1_000_000i64..1_000_000) {
        let schema: Schema = serde_json::from_value(json!({
            "version": "1.0",
            "root": { "type": "money", "transforms": [{ "fn": "money_to_cents", "scale": 2 }] }
        })).unwrap();
        let original = json!(n as f64 / 100.0);
        let mut value = original.clone();
        let _ = validate(&schema, &mut value);
        let ok = value.is_i64() || value == original;
        prop_assert!(ok, "money_to_cents produced {value} from {original}");
    }

    // format_decimal postcondition: numeric input is rendered with exactly
    // `places` digits after the decimal point.
    #[test]
    fn format_decimal_has_exact_places(n in -1_000_000i64..1_000_000, places in 0u64..5) {
        let schema: Schema = serde_json::from_value(json!({
            "version": "1.0",
            "root": { "type": "string", "transforms": [{ "fn": "format_decimal", "places": places }] }
        })).unwrap();
        let mut value = json!(n as f64 / 1000.0);
        let _ = validate(&schema, &mut value);
        let s = value.as_str().expect("format_decimal yields a string");
        let decimals = s.split_once('.').map_or(0, |(_, frac)| frac.len());
        prop_assert_eq!(decimals as u64, places, "format_decimal({}) gave {:?}", places, s);
    }

    // trim postcondition: output has no leading or trailing whitespace.
    #[test]
    fn trim_strips_edge_whitespace(s in "[ \t]*[a-zA-Z0-9]*[ \t]*") {
        let schema: Schema = serde_json::from_value(json!({
            "version": "1.0",
            "root": { "type": "string", "transforms": [{ "fn": "trim" }] }
        })).unwrap();
        let mut value = json!(s);
        let _ = validate(&schema, &mut value);
        let out = value.as_str().expect("trim yields a string");
        prop_assert_eq!(out, out.trim());
    }
}

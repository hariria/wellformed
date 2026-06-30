#![no_main]

use libfuzzer_sys::fuzz_target;
use serde_json::Value;
use wellformed::{validate, Schema};

fuzz_target!(|data: &[u8]| {
    let Ok(case) = serde_json::from_slice::<Value>(data) else {
        return;
    };
    let Some(schema_value) = case.get("schema") else {
        return;
    };
    let Ok(schema) = serde_json::from_value::<Schema>(schema_value.clone()) else {
        return;
    };

    let mut input = case.get("input").cloned().unwrap_or(Value::Null);
    let _ = validate(&schema, &mut input);
});

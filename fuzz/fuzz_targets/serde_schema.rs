#![no_main]

use libfuzzer_sys::fuzz_target;
use wellformed::Schema;

fuzz_target!(|data: &[u8]| {
    let Ok(schema) = serde_json::from_slice::<Schema>(data) else {
        return;
    };

    let Ok(value) = serde_json::to_value(&schema) else {
        return;
    };
    let Ok(reparsed) = serde_json::from_value::<Schema>(value.clone()) else {
        panic!("serialized schema did not reparse");
    };
    let Ok(round_tripped) = serde_json::to_value(&reparsed) else {
        panic!("reparsed schema did not serialize");
    };

    assert_eq!(value, round_tripped);
});

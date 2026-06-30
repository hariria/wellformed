//! Cross-runtime conformance suite (Rust side).
//!
//! Runs the shared fixtures in `/conformance/cases` through the Rust runtime and
//! asserts behavior per the fixture `status`. The TypeScript side runs the same
//! files (`typescript/packages/wellformed/src/conformance.test.ts`).
//! See `/conformance/README.md`.

use std::fs;
use std::path::PathBuf;

use serde_json::Value;
use wellformed::{validate, Schema};

#[test]
fn cross_runtime_conformance() {
    let cases_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../conformance/cases");

    let mut paths: Vec<PathBuf> = fs::read_dir(&cases_dir)
        .unwrap_or_else(|e| panic!("read {}: {e}", cases_dir.display()))
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| {
            p.extension().is_some_and(|x| x == "json")
                && p.file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| !name.starts_with('_'))
        })
        .collect();
    paths.sort();
    assert!(
        !paths.is_empty(),
        "no conformance fixtures found in {}",
        cases_dir.display()
    );

    let mut failures: Vec<String> = Vec::new();

    for path in paths {
        let raw = fs::read_to_string(&path).unwrap();
        let fx: Value = serde_json::from_str(&raw)
            .unwrap_or_else(|e| panic!("parse fixture {}: {e}", path.display()));
        let name = fx["name"].as_str().unwrap_or("<unnamed>").to_string();

        if fx
            .get("skip_rust")
            .and_then(Value::as_bool)
            .unwrap_or(false)
        {
            continue;
        }

        // Target the Rust runtime must match: `expect` when the runtimes agree,
        // or the documented current Rust behavior for a known divergence.
        let status = fx["status"].as_str().unwrap_or("agree");
        let target = if status == "agree" {
            &fx["expect"]
        } else {
            &fx["current"]["rust"]
        };
        let want_valid = target["valid"]
            .as_bool()
            .unwrap_or_else(|| panic!("{name}: missing target.valid"));

        let schema: Schema = match serde_json::from_value(fx["schema"].clone()) {
            Ok(s) => s,
            Err(e) => {
                failures.push(format!("{name}: schema failed to parse in Rust: {e}"));
                continue;
            }
        };

        let mut input = fx["input"].clone();
        let result = match validate(&schema, &mut input) {
            Ok(r) => r,
            Err(e) => {
                failures.push(format!(
                    "{name}: validate() returned Err in Rust: {e:?} (target valid={want_valid})"
                ));
                continue;
            }
        };

        let got_valid = result.is_valid();
        if got_valid != want_valid {
            failures.push(format!(
                "{name}: valid mismatch (rust got {got_valid}, target {want_valid})"
            ));
            continue;
        }

        if let Some(want_code) = target.get("code").and_then(Value::as_str) {
            if !result.errors.iter().any(|e| e.code == want_code) {
                let got: Vec<&str> = result.errors.iter().map(|e| e.code.as_str()).collect();
                failures.push(format!(
                    "{name}: code mismatch (rust codes {got:?}, expected to contain {want_code})"
                ));
            }
        }

        if let Some(want_value) = target.get("value") {
            if &input != want_value {
                failures.push(format!(
                    "{name}: value mismatch (rust got {input}, target {want_value})"
                ));
            }
        }
    }

    assert!(
        failures.is_empty(),
        "Rust conformance failures:\n{}",
        failures.join("\n")
    );
}

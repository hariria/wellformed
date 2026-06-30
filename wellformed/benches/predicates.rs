//! Simple benchmarks for predicates and transforms.
//!
//! Run with: cargo bench -p wellformed --bench predicates

use std::hint::black_box;
use std::time::Instant;

use serde_json::{json, Value};
use wellformed::ir::{
    Constraint, ErrorMeta, IntegerSchema, ObjectSchema, Predicate, Schema, StringSchema,
    TemplateLiteralPart, Transform, TypeSchema,
};
use wellformed::runtime::predicate::{EvalContext, REGISTRY};
use wellformed::runtime::transform::apply_transform;

const ITERS: u64 = 100_000;

fn bench<F: FnMut()>(name: &str, mut f: F) {
    // warmup
    for _ in 0..1_000 {
        f();
    }

    let start = Instant::now();
    for _ in 0..ITERS {
        f();
    }
    let elapsed = start.elapsed();
    let ns = elapsed.as_nanos() as u64 / ITERS;
    println!("{name:>30}  {ns:>6} ns/op  ({ITERS} iters in {elapsed:.2?})");
}

fn bench_predicate(name: &str, pred: Predicate, val: Value) {
    let mut ctx = EvalContext::new(&REGISTRY);
    bench(name, || {
        black_box(
            wellformed::runtime::predicate::evaluate(black_box(&pred), black_box(&val), &mut ctx)
                .unwrap(),
        );
    });
}

fn named_constraint(name: &str) -> Constraint {
    Constraint::new(
        Predicate::call_no_args(name),
        ErrorMeta::new("INVALID", "Invalid value"),
    )
}

fn string_call_schema(name: &str) -> TypeSchema {
    TypeSchema::String(StringSchema::new().constraint(named_constraint(name)))
}

fn integer_range_schema(min: f64, max: f64) -> TypeSchema {
    TypeSchema::Integer(IntegerSchema {
        constraints: vec![Constraint::new(
            Predicate::range(Some(min), Some(max)),
            ErrorMeta::new("OUT_OF_RANGE", "Out of range"),
        )],
        ..IntegerSchema::new()
    })
}

fn benchmark_object_schema() -> Schema {
    Schema::new(
        "1.0.0",
        TypeSchema::Object(
            ObjectSchema::new()
                .property("email", string_call_schema("is_email"))
                .property("age", integer_range_schema(18.0, 99.0))
                .property("website", string_call_schema("is_url"))
                .property("ssn", string_call_schema("is_ssn")),
        ),
    )
}

fn schema_with_call(name: &str) -> Schema {
    Schema::new("1.0.0", string_call_schema(name))
}

fn bench_validate_inputs(name: &str, schema: Schema, mut values: Vec<Value>) {
    let mut index = 0usize;
    let len = values.len();
    bench(name, || {
        // These validation cases do not transform/default values, so reusing
        // samples keeps the comparison focused on validation instead of cloning.
        let value = &mut values[index];
        index = (index + 1) % len;
        black_box(
            wellformed::runtime::validate::validate(black_box(&schema), black_box(value))
                .unwrap()
                .is_valid(),
        );
    });
}

fn main() {
    println!("--- predicates ---");

    bench_predicate(
        "phone_number (US)",
        Predicate::call_no_args("phone_number"),
        json!("(555) 234-5678"),
    );

    bench_predicate(
        "phone_number (intl)",
        Predicate::call_no_args("phone_number"),
        json!("+44 20 7946 0958"),
    );

    bench_predicate(
        "phone_number_us",
        Predicate::call_no_args("phone_number_us"),
        json!("5552345678"),
    );

    bench_predicate(
        "is_email",
        Predicate::call_no_args("is_email"),
        json!("user@example.com"),
    );

    bench_predicate(
        "is_url",
        Predicate::call_no_args("is_url"),
        json!("https://sub.example.com/path?q=1"),
    );

    bench_predicate(
        "is_uuid",
        Predicate::call_no_args("is_uuid"),
        json!("550e8400-e29b-41d4-a716-446655440000"),
    );

    bench_predicate(
        "is_ssn",
        Predicate::call_no_args("is_ssn"),
        json!("123-45-6789"),
    );

    bench_predicate(
        "is_credit_card",
        Predicate::call_no_args("is_credit_card"),
        json!("4111 1111 1111 1111"),
    );

    bench_predicate(
        "is_iban",
        Predicate::call_no_args("is_iban"),
        json!("GB29 NWBK 6016 1331 9268 19"),
    );

    bench_predicate(
        "luhn",
        Predicate::call_no_args("luhn"),
        json!("4111111111111111"),
    );

    bench_predicate(
        "is_date",
        Predicate::call_no_args("is_date"),
        json!("12/31/2024"),
    );

    bench_predicate(
        "is_aba_routing",
        Predicate::call_no_args("is_aba_routing"),
        json!("021000021"),
    );

    bench_predicate(
        "is_country_code",
        Predicate::call_no_args("is_country_code"),
        json!("US"),
    );

    bench_predicate(
        "regex (digits)",
        Predicate::regex(r"^\d{9}$"),
        json!("123456789"),
    );

    bench_predicate(
        "regex (template eqv)",
        Predicate::regex(r"^SFO-\d{3,4}-[A-Z]{2}$"),
        json!("SFO-1234-AB"),
    );

    bench_predicate(
        "template_literal",
        Predicate::template_literal(vec![
            TemplateLiteralPart::literal("SFO-"),
            TemplateLiteralPart::digits(Some(3), Some(4)),
            TemplateLiteralPart::literal("-"),
            TemplateLiteralPart::uppercase(Some(2), Some(2)),
        ]),
        json!("SFO-1234-AB"),
    );

    println!();
    println!("--- validation ---");

    bench_validate_inputs(
        "validate email",
        schema_with_call("is_email"),
        vec![
            json!("ada@example.com"),
            json!("user.name+tag@example.org"),
            json!("a@b.co"),
            json!("not-an-email"),
            json!("@example.com"),
            json!("bad@.com"),
        ],
    );

    bench_validate_inputs(
        "validate url",
        schema_with_call("is_url"),
        vec![
            json!("https://example.com"),
            json!("http://sub.example.com/path?q=1"),
            json!("https://docs.example.org/benchmarks"),
            json!("ftp://example.com"),
            json!("not a url"),
            json!("https://"),
        ],
    );

    bench_validate_inputs(
        "validate uuid",
        schema_with_call("is_uuid"),
        vec![
            json!("550e8400-e29b-41d4-a716-446655440000"),
            json!("6fa459ea-ee8a-3ca4-894e-db77e160355e"),
            json!("00000000-0000-0000-0000-000000000000"),
            json!("550e8400-e29b-41d4-a716-44665544000"),
            json!("not-a-uuid"),
            json!("550e8400e29b41d4a716446655440000"),
        ],
    );

    bench_validate_inputs(
        "validate ssn",
        schema_with_call("is_ssn"),
        vec![
            json!("123-45-6789"),
            json!("078-05-1120"),
            json!("219099999"),
            json!("000-12-3456"),
            json!("666-12-3456"),
            json!("987-65-4321"),
        ],
    );

    bench_validate_inputs(
        "validate credit_card",
        schema_with_call("is_credit_card"),
        vec![
            json!("4111 1111 1111 1111"),
            json!("5555-5555-5555-4444"),
            json!("378282246310005"),
            json!("4111 1111 1111 1112"),
            json!("1234567890123456"),
            json!("not-a-card"),
        ],
    );

    bench_validate_inputs(
        "validate iban",
        schema_with_call("is_iban"),
        vec![
            json!("GB29 NWBK 6016 1331 9268 19"),
            json!("DE89370400440532013000"),
            json!("FR7630006000011234567890189"),
            json!("GB29NWBK60161331926818"),
            json!("DE123"),
            json!("not-an-iban"),
        ],
    );

    bench_validate_inputs(
        "validate object",
        benchmark_object_schema(),
        vec![
            json!({
                "email": "ada@example.com",
                "age": 37,
                "website": "https://example.com",
                "ssn": "123-45-6789",
            }),
            json!({
                "email": "grace@example.org",
                "age": 58,
                "website": "http://research.example.org/profile",
                "ssn": "078-05-1120",
            }),
            json!({
                "email": "bad",
                "age": 37,
                "website": "https://example.com",
                "ssn": "123-45-6789",
            }),
            json!({
                "email": "ada@example.com",
                "age": 12,
                "website": "https://example.com",
                "ssn": "123-45-6789",
            }),
            json!({
                "email": "ada@example.com",
                "age": 37,
                "website": "ftp://example.com",
                "ssn": "123-45-6789",
            }),
            json!({
                "email": "ada@example.com",
                "age": 37,
                "website": "https://example.com",
                "ssn": "000-12-3456",
            }),
        ],
    );

    println!();
    println!("--- transforms ---");

    bench("trim", || {
        let mut val = json!("  hello world  ");
        apply_transform(&mut val, &Transform::Trim, "").unwrap();
        black_box(&val);
    });

    bench("digits_only", || {
        let mut val = json!("(650) 123-4567");
        apply_transform(&mut val, &Transform::DigitsOnly, "").unwrap();
        black_box(&val);
    });

    bench("phone_us (10 digits)", || {
        let mut val = json!("6501234567");
        apply_transform(&mut val, &Transform::PhoneUs, "").unwrap();
        black_box(&val);
    });

    bench("phone_us (formatted)", || {
        let mut val = json!("(650) 123-4567");
        apply_transform(&mut val, &Transform::PhoneUs, "").unwrap();
        black_box(&val);
    });

    bench("phone_us (11 digits)", || {
        let mut val = json!("16501234567");
        apply_transform(&mut val, &Transform::PhoneUs, "").unwrap();
        black_box(&val);
    });

    bench("upper", || {
        let mut val = json!("hello world");
        apply_transform(&mut val, &Transform::Upper, "").unwrap();
        black_box(&val);
    });

    bench("lower", || {
        let mut val = json!("HELLO WORLD");
        apply_transform(&mut val, &Transform::Lower, "").unwrap();
        black_box(&val);
    });

    bench("money_to_cents", || {
        let mut val = json!("$1,234.56");
        apply_transform(&mut val, &Transform::MoneyToCents { scale: 2 }, "").unwrap();
        black_box(&val);
    });

    bench("replace", || {
        let mut val = json!("123-45-6789");
        apply_transform(
            &mut val,
            &Transform::Replace {
                pattern: "-".to_string(),
                replacement: "".to_string(),
            },
            "",
        )
        .unwrap();
        black_box(&val);
    });
}

//! Benchmarks for pattern matching (vectorscan vs regex fallback).

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use wellformed_validate::patterns::{
    is_ein_format, is_email_format, is_ssn_format, is_zip_format, PatternDb,
};

fn generate_test_data(count: usize) -> Vec<String> {
    (0..count)
        .map(|i| match i % 5 {
            0 => format!(
                "{:03}-{:02}-{:04}",
                100 + (i % 800),
                10 + (i % 89),
                1000 + (i % 8999)
            ),
            1 => format!("user{}@example.com", i),
            2 => format!("{:05}", 10000 + (i % 89999)),
            3 => format!("{:02}-{:07}", 10 + (i % 90), 1000000 + i),
            _ => format!("random text {}", i),
        })
        .collect()
}

fn bench_pattern_db(c: &mut Criterion) {
    let mut group = c.benchmark_group("pattern_db");

    let db = PatternDb::with_builtins().unwrap();

    // Single pattern scan
    group.bench_function("scan_ssn", |b| b.iter(|| db.scan(black_box("123-45-6789"))));

    group.bench_function("scan_email", |b| {
        b.iter(|| db.scan(black_box("test@example.com")))
    });

    group.bench_function("scan_zip", |b| b.iter(|| db.scan(black_box("12345"))));

    // Batch scanning
    for size in [100, 1000, 10000] {
        let data = generate_test_data(size);
        let refs: Vec<&str> = data.iter().map(|s| s.as_str()).collect();

        group.throughput(Throughput::Elements(size as u64));

        group.bench_function(format!("scan_batch_{}", size), |b| {
            b.iter(|| db.scan_batch(black_box(&refs)))
        });
    }

    group.finish();
}

fn bench_inline_checks(c: &mut Criterion) {
    let mut group = c.benchmark_group("inline_format_checks");

    // Compare inline format checks (no regex)
    group.bench_function("is_ssn_format", |b| {
        b.iter(|| is_ssn_format(black_box("123-45-6789")))
    });

    group.bench_function("is_ein_format", |b| {
        b.iter(|| is_ein_format(black_box("12-3456789")))
    });

    group.bench_function("is_email_format", |b| {
        b.iter(|| is_email_format(black_box("test@example.com")))
    });

    group.bench_function("is_zip_format", |b| {
        b.iter(|| is_zip_format(black_box("12345")))
    });

    group.finish();
}

fn bench_mixed_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("mixed_patterns");

    let db = PatternDb::with_builtins().unwrap();

    // Mix of different pattern types
    let inputs = vec![
        "123-45-6789",      // SSN
        "test@example.com", // Email
        "12345",            // ZIP
        "12-3456789",       // EIN
        "12/25/2024",       // Date
        "$1,234.56",        // Money
        "random text",      // No match
    ];

    group.bench_function("scan_mixed_7", |b| {
        b.iter(|| {
            for input in &inputs {
                black_box(db.scan(input));
            }
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_pattern_db,
    bench_inline_checks,
    bench_mixed_patterns
);
criterion_main!(benches);

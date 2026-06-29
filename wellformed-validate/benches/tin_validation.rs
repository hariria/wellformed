//! Benchmarks for TIN validation.

use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use std::hint::black_box;
use wellformed_validate::tin::{validate_any, validate_batch, validate_ein, validate_ssn};

fn generate_ssns(count: usize) -> Vec<String> {
    (0..count)
        .map(|i| {
            let area = 100 + (i % 800);
            let group = 10 + (i % 89);
            let serial = 1000 + (i % 8999);
            format!("{:03}-{:02}-{:04}", area, group, serial)
        })
        .collect()
}

fn generate_eins(count: usize) -> Vec<String> {
    (0..count)
        .map(|i| {
            let campus = 10 + (i % 90);
            let serial = 1000000 + (i % 8999999);
            format!("{:02}-{:07}", campus, serial)
        })
        .collect()
}

fn bench_single_ssn(c: &mut Criterion) {
    let mut group = c.benchmark_group("single_tin");

    group.bench_function("validate_ssn_formatted", |b| {
        b.iter(|| validate_ssn(black_box("123-45-6789")))
    });

    group.bench_function("validate_ssn_unformatted", |b| {
        b.iter(|| validate_ssn(black_box("123456789")))
    });

    group.bench_function("validate_ein_formatted", |b| {
        b.iter(|| validate_ein(black_box("12-3456789")))
    });

    group.bench_function("validate_any", |b| {
        b.iter(|| validate_any(black_box("123-45-6789")))
    });

    group.finish();
}

fn bench_batch_validation(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_tin");

    for size in [100, 1000, 10000] {
        let ssns = generate_ssns(size);
        let ssn_refs: Vec<&str> = ssns.iter().map(|s| s.as_str()).collect();

        group.throughput(Throughput::Elements(size as u64));

        group.bench_function(format!("validate_batch_{}", size), |b| {
            b.iter(|| validate_batch(black_box(&ssn_refs)))
        });
    }

    group.finish();
}

fn bench_mixed_tins(c: &mut Criterion) {
    let mut group = c.benchmark_group("mixed_tin");

    let ssns = generate_ssns(500);
    let eins = generate_eins(500);

    let mut mixed: Vec<&str> = Vec::with_capacity(1000);
    for i in 0..500 {
        mixed.push(&ssns[i]);
        mixed.push(&eins[i]);
    }

    group.throughput(Throughput::Elements(1000));

    group.bench_function("mixed_batch_1000", |b| {
        b.iter(|| validate_batch(black_box(&mixed)))
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_single_ssn,
    bench_batch_validation,
    bench_mixed_tins
);
criterion_main!(benches);

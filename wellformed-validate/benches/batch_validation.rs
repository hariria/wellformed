//! Benchmarks for batch form validation.

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use wellformed_validate::batch::FormBatch;

fn generate_form_data(count: usize) -> Vec<(String, String, i64, u16)> {
    (0..count)
        .map(|i| {
            let payer_tin = format!(
                "{:03}-{:02}-{:04}",
                100 + (i % 800),
                10 + (i % 89),
                1000 + (i % 8999)
            );
            let recipient_tin = format!(
                "{:03}-{:02}-{:04}",
                200 + (i % 700),
                20 + (i % 79),
                2000 + (i % 7999)
            );
            let amount = (i as i64 + 1) * 10000; // $100.00 increments
            let year = 2020 + ((i % 5) as u16);
            (payer_tin, recipient_tin, amount, year)
        })
        .collect()
}

fn bench_form_batch(c: &mut Criterion) {
    let mut group = c.benchmark_group("form_batch");

    for size in [100, 1000, 10000] {
        let data = generate_form_data(size);

        group.throughput(Throughput::Elements(size as u64));

        group.bench_function(format!("build_batch_{}", size), |b| {
            b.iter(|| {
                let mut batch = FormBatch::with_capacity(size);
                for (payer, recipient, amount, year) in &data {
                    batch.push_tins(payer, recipient);
                    batch.push_income(*amount);
                    batch.push_tax_year(*year);
                }
                black_box(batch.len())
            })
        });

        // Pre-build batch for validation benchmark
        let mut batch = FormBatch::with_capacity(size);
        for (payer, recipient, amount, year) in &data {
            batch.push_tins(payer, recipient);
            batch.push_income(*amount);
            batch.push_tax_year(*year);
        }

        group.bench_function(format!("validate_batch_{}", size), |b| {
            b.iter(|| batch.validate())
        });

        group.bench_function(format!("validate_tins_only_{}", size), |b| {
            b.iter(|| batch.validate_tins_only())
        });
    }

    group.finish();
}

fn bench_batch_reuse(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_reuse");

    let data = generate_form_data(1000);
    let mut batch = FormBatch::with_capacity(1000);

    group.throughput(Throughput::Elements(1000));

    group.bench_function("fill_validate_clear_cycle", |b| {
        b.iter(|| {
            batch.clear();
            for (payer, recipient, amount, year) in &data {
                batch.push_tins(payer, recipient);
                batch.push_income(*amount);
                batch.push_tax_year(*year);
            }
            let result = batch.validate();
            black_box(result.valid_count())
        })
    });

    group.finish();
}

fn bench_invalid_forms(c: &mut Criterion) {
    let mut group = c.benchmark_group("invalid_forms");

    // Generate data with some invalid entries
    let count = 1000;
    let mut batch = FormBatch::with_capacity(count);

    for i in 0..count {
        if i % 10 == 0 {
            // Invalid SSN (area 000)
            batch.push_tins("000-12-3456", "123-45-6789");
        } else if i % 7 == 0 {
            // Invalid amount (negative)
            batch.push_tins("123-45-6789", "987-65-4321");
            batch.push_income(-1000);
            batch.push_tax_year(2024);
            continue;
        } else {
            batch.push_tins(
                &format!(
                    "{:03}-{:02}-{:04}",
                    100 + (i % 800),
                    10 + (i % 89),
                    1000 + (i % 8999)
                ),
                &format!(
                    "{:03}-{:02}-{:04}",
                    200 + (i % 700),
                    20 + (i % 79),
                    2000 + (i % 7999)
                ),
            );
        }
        batch.push_income((i as i64 + 1) * 100);
        batch.push_tax_year(2024);
    }

    group.throughput(Throughput::Elements(count as u64));

    group.bench_function("validate_with_invalid", |b| {
        b.iter(|| {
            let result = batch.validate();
            black_box((result.valid_count(), result.invalid_count()))
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_form_batch,
    bench_batch_reuse,
    bench_invalid_forms
);
criterion_main!(benches);

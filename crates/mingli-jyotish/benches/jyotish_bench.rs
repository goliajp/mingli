#![allow(missing_docs, clippy::unreadable_literal)]

use criterion::{criterion_group, criterion_main, Criterion};
use mingli_jyotish::{compute, Ayanamsa, BirthInput};

fn bench_compute(c: &mut Criterion) {
    c.bench_function("jyotish_compute_1990", |b| {
        b.iter(|| {
            let _ = compute(
                BirthInput { year: 1990, month: 6, day: 15, hour: 14, minute: 30, tz: 8.0 },
                None,
                Ayanamsa::Lahiri,
            );
        });
    });
}

criterion_group!(benches, bench_compute);
criterion_main!(benches);

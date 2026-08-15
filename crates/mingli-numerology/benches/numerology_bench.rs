//! mingli-numerology 换算基准。
#![allow(missing_docs, reason = "criterion 宏生成的 harness 函数无需文档")]

use std::hint::black_box;
use criterion::{criterion_group, criterion_main, Criterion};
use mingli_numerology::{compute_named, System, expression};
use mingli_astro::Moment;

fn bench(c: &mut Criterion) {
    let m = Moment::new(1990, 6, 15, 12, 0, 8.0);
    c.bench_function("numerology_named", |b| {
        b.iter(|| compute_named(black_box(&m), black_box("Ada Lovelace")));
    });
    c.bench_function("numerology_expression", |b| {
        b.iter(|| expression(black_box("Ada Lovelace"), System::Chaldean));
    });
}

criterion_group!(benches, bench);
criterion_main!(benches);

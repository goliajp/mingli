//! mingli-tibetan 年要素基准。
#![allow(missing_docs, reason = "criterion 宏生成的 harness 函数无需文档")]

use std::hint::black_box;
use criterion::{criterion_group, criterion_main, Criterion};
use mingli_tibetan::compute_year;

fn bench(c: &mut Criterion) {
    c.bench_function("tibetan_compute", |b| {
        b.iter(|| compute_year(black_box(2024)));
    });
}

criterion_group!(benches, bench);
criterion_main!(benches);

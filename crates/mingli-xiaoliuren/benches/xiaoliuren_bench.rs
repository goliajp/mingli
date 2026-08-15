//! mingli-xiaoliuren 掐指基准。
#![allow(missing_docs, reason = "criterion 宏生成的 harness 函数无需文档")]

use std::hint::black_box;
use criterion::{criterion_group, criterion_main, Criterion};
use mingli_xiaoliuren::compute;

fn bench(c: &mut Criterion) {
    c.bench_function("xiaoliuren_compute", |b| {
        b.iter(|| compute(black_box(2024), 6, 15, 14, 30, 8.0));
    });
}

criterion_group!(benches, bench);
criterion_main!(benches);

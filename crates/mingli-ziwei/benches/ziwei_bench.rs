//! mingli-ziwei 排盘基准。
#![allow(missing_docs, reason = "criterion 宏生成的 harness 函数无需文档")]

use std::hint::black_box;
use criterion::{criterion_group, criterion_main, Criterion};
use mingli_ziwei::{compute, BirthInput, Gender};

fn bench_compute(c: &mut Criterion) {
    let input = BirthInput {
        year: 1990,
        month: 6,
        day: 15,
        hour: 14,
        minute: 30,
        tz: 8.0,
        gender: Some(Gender::Male),
    };
    c.bench_function("ziwei_compute", |b| b.iter(|| compute(black_box(input))));
}

criterion_group!(benches, bench_compute);
criterion_main!(benches);

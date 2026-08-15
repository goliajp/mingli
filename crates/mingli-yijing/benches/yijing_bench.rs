//! mingli-yijing 起卦基准。
#![allow(missing_docs, reason = "criterion 宏生成的 harness 函数无需文档")]

use std::hint::black_box;
use criterion::{criterion_group, criterion_main, Criterion};
use mingli_yijing::{cast, Method};

fn bench(c: &mut Criterion) {
    c.bench_function("cast_three_coins", |b| {
        b.iter(|| cast(Method::ThreeCoins, black_box(2024)));
    });
    c.bench_function("cast_yarrow", |b| {
        b.iter(|| cast(Method::YarrowStalks, black_box(2024)));
    });
}

criterion_group!(benches, bench);
criterion_main!(benches);

//! mingli-astrology 排盘基准。
#![allow(missing_docs, reason = "criterion 宏生成的 harness 函数无需文档")]

use std::hint::black_box;
use criterion::{criterion_group, criterion_main, Criterion};
use mingli_astrology::{compute, GeoLocation};

fn bench(c: &mut Criterion) {
    c.bench_function("natal_compute", |b| {
        b.iter(|| compute(black_box(1990), 6, 15, 14, 30, 8.0, None));
    });
    let geo = GeoLocation { latitude: 52.833, longitude: 0.500 };
    c.bench_function("natal_compute_geo", |b| {
        b.iter(|| compute(black_box(1990), 6, 15, 14, 30, 8.0, Some(geo)));
    });
}

criterion_group!(benches, bench);
criterion_main!(benches);

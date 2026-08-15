//! mingli-ganzhi 热路径基准。
#![allow(missing_docs, reason = "criterion 宏生成的 harness 函数无需文档")]

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use mingli_ganzhi::{day_ganzhi, month_pillar_stem, year_ganzhi};

fn bench_ganzhi(c: &mut Criterion) {
    c.bench_function("day_ganzhi", |b| {
        b.iter(|| day_ganzhi(black_box(2_460_311)));
    });
    c.bench_function("year_ganzhi", |b| {
        b.iter(|| year_ganzhi(black_box(1990)));
    });
    c.bench_function("month_pillar_stem", |b| {
        b.iter(|| month_pillar_stem(black_box(6), black_box(6)));
    });
}

criterion_group!(benches, bench_ganzhi);
criterion_main!(benches);

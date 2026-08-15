//! mingli-core 热路径基准（criterion）。
#![allow(missing_docs, reason = "criterion 宏生成的 harness 函数无需文档")]

use std::hint::black_box;
use criterion::{criterion_group, criterion_main, Criterion};
use mingli_core::{cyclic, gf2, sampler};

fn bench_crt(c: &mut Criterion) {
    c.bench_function("crt_combine ganzhi", |b| {
        b.iter(|| cyclic::crt_combine(black_box(&[(7, 10), (11, 12)])));
    });
}

fn bench_geomancy(c: &mut Criterion) {
    c.bench_function("geomancy_shield", |b| {
        b.iter(|| gf2::geomancy_shield(black_box([0b1010, 0b0110, 0b1111, 0b0001])));
    });
}

fn bench_shuffle(c: &mut Criterion) {
    c.bench_function("fisher_yates tarot78", |b| {
        b.iter(|| sampler::shuffle(black_box(78), black_box(0x00C0_FFEE)));
    });
}

criterion_group!(benches, bench_crt, bench_geomancy, bench_shuffle);
criterion_main!(benches);

//! mingli-ifa 起卦基准。
#![allow(missing_docs, reason = "criterion 宏生成的 harness 函数无需文档")]

use std::hint::black_box;
use criterion::{criterion_group, criterion_main, Criterion};
use mingli_ifa::cast;

fn bench(c: &mut Criterion) {
    c.bench_function("ifa_cast", |b| b.iter(|| cast(black_box(2024))));
}

criterion_group!(benches, bench);
criterion_main!(benches);

//! mingli-gematria 求和基准。
#![allow(missing_docs, reason = "criterion 宏生成的 harness 函数无需文档")]

use std::hint::black_box;
use criterion::{criterion_group, criterion_main, Criterion};
use mingli_gematria::compute;

fn bench(c: &mut Criterion) {
    c.bench_function("gematria_compute", |b| {
        b.iter(|| compute(black_box("בראשית ברא אלהים")));
    });
}

criterion_group!(benches, bench);
criterion_main!(benches);

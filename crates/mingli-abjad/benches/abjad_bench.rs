//! mingli-abjad 求和基准。
#![allow(missing_docs, reason = "criterion 宏生成的 harness 函数无需文档")]

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use mingli_abjad::compute;

fn bench(c: &mut Criterion) {
    c.bench_function("abjad_compute", |b| {
        b.iter(|| compute(black_box("بسم الله الرحمن الرحيم")));
    });
}

criterion_group!(benches, bench);
criterion_main!(benches);

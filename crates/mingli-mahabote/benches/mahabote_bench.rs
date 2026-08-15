//! mingli-mahabote 本命换算基准。
#![allow(missing_docs, reason = "criterion 宏生成的 harness 函数无需文档")]

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use mingli_mahabote::compute;

fn bench(c: &mut Criterion) {
    c.bench_function("mahabote_compute", |b| {
        b.iter(|| compute(black_box(2000), 1, 1, 9, 0, 6.5));
    });
}

criterion_group!(benches, bench);
criterion_main!(benches);

//! mingli-maya 历日换算基准。
#![allow(missing_docs, reason = "criterion 宏生成的 harness 函数无需文档")]

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use mingli_maya::compute_from_jdn;

fn bench(c: &mut Criterion) {
    c.bench_function("maya_compute", |b| {
        b.iter(|| compute_from_jdn(black_box(2_456_283)));
    });
}

criterion_group!(benches, bench);
criterion_main!(benches);

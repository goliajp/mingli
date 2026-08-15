//! mingli-qimen 排盘基准。
#![allow(missing_docs, reason = "criterion 宏生成的 harness 函数无需文档")]

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use mingli_qimen::compute;

fn bench(c: &mut Criterion) {
    c.bench_function("qimen_compute", |b| {
        b.iter(|| compute(black_box(2024), 6, 15, 14, 30, 8.0));
    });
}

criterion_group!(benches, bench);
criterion_main!(benches);

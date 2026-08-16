//! mingli-analysis 跨叶 NMI 基准。
#![allow(missing_docs, reason = "criterion 宏生成的 harness 函数无需文档")]

use std::hint::black_box;
use criterion::{criterion_group, criterion_main, Criterion};
use mingli_analysis::{cross_leaf, sample_grid};
use mingli_registry::registry;

fn bench(c: &mut Criterion) {
    let reg = registry();
    let q = sample_grid(2000, 2004); // 5 年 × 24 = 120 样本
    c.bench_function("cross_leaf_120", |b| {
        b.iter(|| cross_leaf(&reg, black_box(&q)));
    });
}

criterion_group!(benches, bench);
criterion_main!(benches);

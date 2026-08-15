//! mingli-sikidy 起盘基准。
#![allow(missing_docs, reason = "criterion 宏生成的 harness 函数无需文档")]

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use mingli_sikidy::cast;

fn bench(c: &mut Criterion) {
    c.bench_function("sikidy_cast", |b| b.iter(|| cast(black_box(2024))));
}

criterion_group!(benches, bench);
criterion_main!(benches);

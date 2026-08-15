//! mingli-luoshu 热路径基准。
#![allow(missing_docs, reason = "criterion 宏生成的 harness 函数无需文档")]

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use mingli_luoshu::fly;

fn bench_fly(c: &mut Criterion) {
    c.bench_function("fly", |b| {
        b.iter(|| fly(black_box(8), black_box(true)));
    });
}

criterion_group!(benches, bench_fly);
criterion_main!(benches);

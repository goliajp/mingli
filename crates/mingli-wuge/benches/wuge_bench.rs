//! mingli-wuge 五格基准。
#![allow(missing_docs, reason = "criterion 宏生成的 harness 函数无需文档")]

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use mingli_wuge::five_grids;

fn bench(c: &mut Criterion) {
    c.bench_function("wuge_five_grids", |b| {
        b.iter(|| five_grids(black_box(&[7]), black_box(&[16, 9])));
    });
}

criterion_group!(benches, bench);
criterion_main!(benches);

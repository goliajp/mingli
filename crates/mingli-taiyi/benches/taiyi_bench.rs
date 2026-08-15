//! mingli-taiyi 起局基准。
#![allow(missing_docs, reason = "criterion 宏生成的 harness 函数无需文档")]

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use mingli_taiyi::compute;

fn bench(c: &mut Criterion) {
    c.bench_function("taiyi_compute", |b| {
        b.iter(|| compute(black_box(2024), 6, 15, 8.0));
    });
}

criterion_group!(benches, bench);
criterion_main!(benches);

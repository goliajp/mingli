//! mingli-pawukon 週序换算基准。
#![allow(missing_docs, reason = "criterion 宏生成的 harness 函数无需文档")]

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use mingli_pawukon::compute_from_day;

fn bench(c: &mut Criterion) {
    c.bench_function("pawukon_compute", |b| {
        b.iter(|| compute_from_day(black_box(73)));
    });
}

criterion_group!(benches, bench);
criterion_main!(benches);

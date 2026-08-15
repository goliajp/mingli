#![allow(missing_docs, clippy::unreadable_literal)]

use criterion::{criterion_group, criterion_main, Criterion};
use mingli_astro::Moment;
use mingli_qizhengsiyu::compute_at;

fn moment_at() -> Moment {
    Moment::new(2024, 6, 15, 14, 30, 8.0)
}

fn bench_qizhengsiyu(c: &mut Criterion) {
    let m = moment_at();
    c.bench_function("qizhengsiyu_compute_at", |b| {
        b.iter(|| compute_at(&m));
    });
}

criterion_group!(benches, bench_qizhengsiyu);
criterion_main!(benches);

//! mingli-gua 热路径基准。
#![allow(missing_docs, reason = "criterion 宏生成的 harness 函数无需文档")]
#![allow(clippy::unreadable_literal, reason = "卦的二进制位型连写更直观")]

use std::hint::black_box;
use criterion::{criterion_group, criterion_main, Criterion};
use mingli_gua::Hexagram;

fn bench_gua(c: &mut Criterion) {
    c.bench_function("hexagram_transforms", |b| {
        b.iter(|| {
            let h = Hexagram(black_box(0b010110));
            (h.opposite(), h.reversed(), h.mutual(), h.changed(0b000101))
        });
    });
}

criterion_group!(benches, bench_gua);
criterion_main!(benches);

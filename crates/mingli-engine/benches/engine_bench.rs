//! mingli-engine fan-out 基准。
#![allow(missing_docs, reason = "criterion 宏生成的 harness 函数无需文档")]

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use mingli_engine::{cast_all, cast_one, Gender, Query};
use std::collections::BTreeMap;

fn q() -> Query {
    Query {
        year: 1990,
        month: 6,
        day: 15,
        hour: 14,
        minute: 30,
        tz: 8.0,
        gender: Some(Gender::Male),
        latitude: Some(31.23),
        longitude: Some(121.47),
        seed: Some(2024),
        name: Some("Ada Lovelace".to_string()),
        schools: BTreeMap::new(),
    }
}

fn bench(c: &mut Criterion) {
    let query = q();
    c.bench_function("cast_all_parallel", |b| b.iter(|| cast_all(black_box(&query))));
    // 单叶：占星（含 VSOP87，最重） vs 八字（纯历法）——量化 cast_one 相对全盘的省算。
    c.bench_function("cast_one_astrology", |b| b.iter(|| cast_one(black_box("astrology"), black_box(&query))));
    c.bench_function("cast_one_bazi", |b| b.iter(|| cast_one(black_box("bazi"), black_box(&query))));
}

criterion_group!(benches, bench);
criterion_main!(benches);

//! mingli-interpret 提示词组装基准。
#![allow(missing_docs, reason = "criterion 宏生成的 harness 函数无需文档")]

use std::hint::black_box;
use criterion::{criterion_group, criterion_main, Criterion};
use mingli_contract::{Gender, Query};
use mingli_engine::cast_all_detailed;
use mingli_registry::registry;
use mingli_interpret::build_prompt;

fn bench(c: &mut Criterion) {
    let q = Query {
        year: 1990, month: 6, day: 15, hour: 14, minute: 30, tz: 8.0,
        gender: Some(Gender::Male), latitude: Some(31.23), longitude: Some(121.47),
        seed: None, name: Some("Ada".to_string()),
        schools: std::collections::BTreeMap::new(),
    };
    let leaf = cast_all_detailed(&registry(), &q).into_iter().find(|l| l.id == "liuren").unwrap();
    c.bench_function("build_prompt", |b| b.iter(|| build_prompt(black_box(&leaf))));
}

criterion_group!(benches, bench);
criterion_main!(benches);

//! mingli-astrology 排盘基准。
#![allow(missing_docs, reason = "criterion 宏生成的 harness 函数无需文档")]

use std::hint::black_box;
use criterion::{criterion_group, criterion_main, Criterion};
use mingli_astrology::{compute, longitudes_at, GeoLocation};
use mingli_astro::Moment;

fn bench(c: &mut Criterion) {
    c.bench_function("natal_compute", |b| {
        b.iter(|| compute(black_box(1990), 6, 15, 14, 30, 8.0, None));
    });
    // 消融：整张盘减去只算位置，剩下的就是相位 + 整宫 + 分宫 + 落座那些活的价钱。
    // 浏览器那一侧拿我们跟 astronomy-engine 比时，对手只出九个黄经——
    // 不先把这两段分开，1301 µs 对 40 µs 比的就不是同一件事。
    let m = Moment::new(1990, 6, 15, 14, 30, 8.0);
    c.bench_function("longitudes_only", |b| b.iter(|| longitudes_at(black_box(&m))));
    let geo = GeoLocation { latitude: 52.833, longitude: 0.500 };
    c.bench_function("natal_compute_geo", |b| {
        b.iter(|| compute(black_box(1990), 6, 15, 14, 30, 8.0, Some(geo)));
    });
}

criterion_group!(benches, bench);
criterion_main!(benches);

//! mingli-ephemeris 基准。
#![allow(missing_docs, reason = "criterion 宏生成的 harness 函数无需文档")]

use std::hint::black_box;
use criterion::{criterion_group, criterion_main, Criterion};
use mingli_ephemeris::{geocentric_ecliptic_longitude, Body};

fn bench_eph(c: &mut Criterion) {
    let jde = 2_460_476.0;
    c.bench_function("geocentric_longitude_mars", |b| {
        b.iter(|| geocentric_ecliptic_longitude(black_box(Body::Mars), black_box(jde)));
    });
    c.bench_function("geocentric_longitude_sun", |b| {
        b.iter(|| geocentric_ecliptic_longitude(black_box(Body::Sun), black_box(jde)));
    });
    c.bench_function("geocentric_longitude_moon", |b| {
        b.iter(|| geocentric_ecliptic_longitude(black_box(Body::Moon), black_box(jde)));
    });
}

criterion_group!(benches, bench_eph);
criterion_main!(benches);

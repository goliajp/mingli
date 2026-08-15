//! 太阳视黄经与二十四节气时刻（Meeus 简化模型）。

use crate::{jd_ut_to_jde, julian_day, norm180};

/// 太阳视黄经（度，`[0,360)`），输入为力学时 JDE。Meeus ch.25 低精度（~0.01°）。
#[must_use]
pub fn sun_apparent_longitude(jde: f64) -> f64 {
    let t = (jde - 2451545.0) / 36525.0;
    let l0 = 280.46646 + 36000.76983 * t + 0.0003032 * t * t;
    let m = 357.52911 + 35999.05029 * t - 0.0001537 * t * t;
    let mr = m.to_radians();
    let c = (1.914602 - 0.004817 * t - 0.000014 * t * t) * mr.sin()
        + (0.019993 - 0.000101 * t) * (2.0 * mr).sin()
        + 0.000289 * (3.0 * mr).sin();
    let true_long = l0 + c;
    let omega = 125.04 - 1934.136 * t;
    let lambda = true_long - 0.00569 - 0.00478 * omega.to_radians().sin();
    lambda.rem_euclid(360.0)
}

/// 求给定公历年内，太阳视黄经达到 `target`（度）的唯一时刻 JD(UT)。
/// 例：`solar_term_jd(2024, 315.0)` → 2024 立春；`solar_term_jd(2024, 270.0)` → 2024 冬至。
#[must_use]
pub fn solar_term_jd(year: i32, target: f64) -> f64 {
    // 初值：从该年 1/1 的太阳黄经出发，按平黄经速度 ~0.98565°/日 推进到 target，
    // 保证落在请求的公历年内（每个节气在一个公历年内恰好出现一次）。
    let jan1 = julian_day(year, 1, 1.0);
    let l0 = sun_apparent_longitude(jd_ut_to_jde(jan1));
    let adv = (target - l0).rem_euclid(360.0);
    let jd = jan1 + adv / 0.98565;
    solar_term_time_near(jd, target)
}

/// 从给定 JD(UT) 附近收敛到「太阳视黄经 = target」的时刻。用于大运起运（找前/后一个「节」）。
#[must_use]
pub fn solar_term_time_near(jd_guess: f64, target: f64) -> f64 {
    let mut jd = jd_guess;
    for _ in 0..10 {
        let lam = sun_apparent_longitude(jd_ut_to_jde(jd));
        let d = norm180(lam - target);
        jd -= d / 0.98565;
    }
    jd
}

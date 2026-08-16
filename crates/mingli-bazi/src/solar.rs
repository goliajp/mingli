//! 真太阳时校正：经度差 + 简化均时差，以及所需的民用日历算术。

use super::*;

/// 是否闰年（公历）。
pub(crate) const fn is_leap_year(y: i32) -> bool {
    y % 4 == 0 && (y % 100 != 0 || y % 400 == 0)
}

pub(crate) const DAYS_IN_MONTH: [u32; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

pub(crate) const fn days_in_month(y: i32, m: u32) -> u32 {
    if m == 2 && is_leap_year(y) { 29 } else { DAYS_IN_MONTH[(m - 1) as usize] }
}

/// 公历日期 +Δ 天（可正可负，跨月/跨年自动处理）。
pub(crate) fn add_days_civil(y: i32, m: u32, d: u32, delta: i32) -> (i32, u32, u32) {
    let mut y = y;
    let mut m = m;
    let mut d = d as i32 + delta;
    while d < 1 {
        m = if m == 1 { y -= 1; 12 } else { m - 1 };
        d += days_in_month(y, m) as i32;
    }
    while d > days_in_month(y, m) as i32 {
        d -= days_in_month(y, m) as i32;
        m = if m == 12 { y += 1; 1 } else { m + 1 };
    }
    (y, m, d as u32)
}

/// 公历年内日序 1..366。
pub(crate) fn day_of_year(y: i32, m: u32, d: u32) -> u32 {
    const CUMUL: [u32; 12] = [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334];
    let mut n = CUMUL[(m - 1) as usize] + d;
    if is_leap_year(y) && m > 2 {
        n += 1;
    }
    n
}

/// 均时差（分钟，Spencer/Iqbal 简化公式，精度 ±0.5 min，足够时辰判定）。
///
/// 真太阳时与平太阳时的差(EoT) = 9.87 sin(2B) − 7.53 cos(B) − 1.5 sin(B)，
/// 其中 B = 2π(N−81)/365、N = 年内日序(1..366)。
#[must_use]
pub fn equation_of_time_minutes(year: i32, month: u32, day: u32) -> f64 {
    let n = f64::from(day_of_year(year, month, day));
    let b = 2.0 * std::f64::consts::PI * (n - 81.0) / 365.0;
    9.87 * (2.0 * b).sin() - 7.53 * b.cos() - 1.5 * b.sin()
}

/// 真太阳时相对钟表时的总偏移（分钟，正=真太阳时较钟表晚）。
///
/// 由两部分组成：① 经度差（出生地经度 − 时区标准经线）× 4 分钟/度；② [`equation_of_time_minutes`] 均时差。
#[must_use]
pub fn true_solar_offset_minutes(
    longitude: f64,
    tz_hours: f64,
    year: i32,
    month: u32,
    day: u32,
) -> f64 {
    let std_longitude = tz_hours * 15.0;
    let geo_correction = (longitude - std_longitude) * 4.0;
    geo_correction + equation_of_time_minutes(year, month, day)
}

/// 真太阳时排盘：按出生地经度 + EoT 校正钟表时，再排八字。
///
/// 钟表时 → 真太阳时差（±约 30 分钟内典型）；跨时辰边界时，时柱与钟表版排盘不同。
#[must_use]
pub fn compute_with_true_solar(input: BirthInput, longitude: f64) -> BaziChart {
    let offset = true_solar_offset_minutes(
        longitude, input.tz, input.year, input.month, input.day,
    );
    let offset_min = offset.round() as i32;
    let total = input.hour as i32 * 60 + input.minute as i32 + offset_min;
    let (day_delta, in_day_min) = if total < 0 {
        (-1, total + 24 * 60)
    } else if total >= 24 * 60 {
        (1, total - 24 * 60)
    } else {
        (0, total)
    };
    let (ny, nm, nd) = if day_delta == 0 {
        (input.year, input.month, input.day)
    } else {
        add_days_civil(input.year, input.month, input.day, day_delta)
    };
    let nh = (in_day_min / 60) as u32;
    let nmin = (in_day_min % 60) as u32;
    let moment = Moment::new(ny, nm, nd, nh, nmin, input.tz);
    compute_at(&moment, input.gender)
}

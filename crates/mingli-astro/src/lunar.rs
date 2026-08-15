//! 农历（阴阳合历）：基于朔与中气的「无中气置闰」算法，确定性、不查表。
//!
//! 规则（通行的中国农历定朔定气法）：
//! - 朔（新月）所在民用日为农历月初一。
//! - 含冬至(λ=270°)之月为「十一月」（子月）。
//! - 一岁（冬至到次冬至）内若有 13 个月，则该岁置闰；闰月 = 该岁内第一个「不含中气」之月，
//!   其月名沿用前一个月（如闰二月）。中气 = 太阳黄经为 30° 整数倍之节气。

use crate::sun::{solar_term_jd, sun_apparent_longitude};
use crate::{
    civil_day_number, jd_ut_to_jde, local_civil_day_of,
    moon::{new_moon_jd_ut, new_moon_k_near},
};
use serde::Serialize;

/// 阴阳合历日期（公历换算所得）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct LunarDate {
    /// 农历年（以正月初一为界；十一、十二月归本岁起始年）。
    pub year: i32,
    /// 月序 1..12（闰月沿用同序号，配合 `leap`）。
    pub month: u32,
    /// 是否为闰月。
    pub leap: bool,
    /// 日 1..30。
    pub day: u32,
}

/// 返回「本地民用日序号 <= cdn」的最大朔所在民用日 cdn，及其朔序号 k。
fn new_moon_on_or_before(cdn: i64, tz: f64) -> (i64, i64) {
    let mut k = new_moon_k_near(cdn as f64 - 0.5);
    loop {
        let c = local_civil_day_of(new_moon_jd_ut(k), tz);
        let c_next = local_civil_day_of(new_moon_jd_ut(k + 1), tz);
        if c <= cdn && cdn < c_next {
            return (c, k);
        } else if c > cdn {
            k -= 1;
        } else {
            k += 1;
        }
    }
}

/// 含某年冬至之月的初一（子月初一）的民用日 cdn 与朔序号。
fn month11(year: i32, tz: f64) -> (i64, i64) {
    let ws_cdn = local_civil_day_of(solar_term_jd(year, 270.0), tz);
    new_moon_on_or_before(ws_cdn, tz)
}

/// 公历 → 农历。`tz` 为时区偏移小时（中国 +8，日本 +9）。
#[must_use]
pub fn solar_to_lunar(year: i32, month: u32, day: u32, tz: f64) -> LunarDate {
    let cdn = civil_day_number(year, month, day);

    let (m11_y_cdn, _) = month11(year, tz);
    let start_year = if cdn >= m11_y_cdn { year } else { year - 1 };
    let (_start_cdn, start_k) = month11(start_year, tz);
    let (next11_cdn, _) = month11(start_year + 1, tz);

    // 枚举本岁内每个朔（农历月初一），记录其所在民用日 cdn
    let mut nm_cdn: Vec<i64> = Vec::new();
    let mut k = start_k;
    loop {
        let c = local_civil_day_of(new_moon_jd_ut(k), tz);
        nm_cdn.push(c);
        if c >= next11_cdn {
            break;
        }
        k += 1;
    }
    let n_months = nm_cdn.len() - 1; // 12（平） 或 13（闰）
    let is_leap_year = n_months == 13;

    // 某月 [i， i+1) 是否含中气（λ 为 30° 整数倍）。
    // 关键：中国农历定气按「民用日」判定，故取该月初一与下月初一【当地 0 时】的太阳黄经，
    // 看其间是否跨过 30° 整数倍（而非用朔的瞬时——否则 2020 夏至这类刀刃 case 会误判）。
    let lam_at_day = |cdn: i64| -> f64 {
        let jd_ut = (cdn as f64 - 0.5) - tz / 24.0; // 该民用日当地 0 时，转 UT
        sun_apparent_longitude(jd_ut_to_jde(jd_ut))
    };
    let has_zhongqi = |i: usize| -> bool {
        let a = lam_at_day(nm_cdn[i]);
        let b = lam_at_day(nm_cdn[i + 1]);
        let na = (a / 30.0).floor() as i64;
        let mut nb = (b / 30.0).floor() as i64;
        if b < a {
            nb += 12;
        }
        nb > na
    };

    // 编号
    let mut num: u32 = 11;
    let mut last: u32 = 11;
    let mut leap_done = false;
    let mut info: Vec<(u32, bool)> = Vec::with_capacity(n_months);
    for i in 0..n_months {
        let is_leap = is_leap_year && !leap_done && i >= 1 && !has_zhongqi(i);
        if is_leap {
            info.push((last, true));
            leap_done = true;
        } else {
            info.push((num, false));
            last = num;
            num = if num == 12 { 1 } else { num + 1 };
        }
    }

    // 定位 cdn 所在月
    let mut idx = 0usize;
    for i in 0..n_months {
        if nm_cdn[i] <= cdn && cdn < nm_cdn[i + 1] {
            idx = i;
            break;
        }
    }
    let (mnum, leap) = info[idx];
    let lday = (cdn - nm_cdn[idx] + 1) as u32;
    let lyear = if mnum >= 11 { start_year } else { start_year + 1 };
    LunarDate {
        year: lyear,
        month: mnum,
        leap,
        day: lday,
    }
}

//! L1 物理石：计算天文学与历法学。
//!
//! 给定一个时刻，确定性地计算：太阳视黄经与二十四节气时刻、朔（新月）时刻、
//! 阴阳合历（含定气置闰）、以及六十干支循环。算法采用 Meeus《Astronomical Algorithms》
//! 的截断模型，精度约：太阳黄经 ~0.01°（≈15 分钟），朔 ~数分钟——足以可靠判定
//! 节气/朔落在哪一个民用日。所有公开函数对相同输入恒返回相同结果。
//!
//! 角度归一等纯角度工具下沉至 [`mingli_core::quantizer`]（L0）。
//!
//! 应用域：本层为各类历法型与天文型术数（八字、紫微、择日、占星、七政四余等）
//! 提供共同的时间→天文量基底；它本身只做天文/历法计算，不含任何释义。

#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap,
    clippy::cast_lossless,
    reason = "计算天文学：f64 与整数（儒略日、角度分段、民用日序）间的换算固有且取值范围受控"
)]
#![allow(
    clippy::unreadable_literal,
    reason = "天文/历法系数沿用 Meeus 等权威文献的原始字面，加数字分隔符反而失真、难对照"
)]

mod lunar;
mod moon;
mod sun;

pub use lunar::{solar_to_lunar, LunarDate};
pub use mingli_core::quantizer::{norm180, norm360};
pub use moon::new_moon_jd_ut;
pub use sun::{solar_term_jd, solar_term_time_near, sun_apparent_longitude};

/// 儒略日（JD），输入为「世界时 UT」的格里历日期+小数日。
/// 对 1582 年后的格里历有效（含本项目目标年代 1900–2100）。
#[must_use]
pub fn julian_day(year: i32, month: u32, day: f64) -> f64 {
    let (y, m) = if month <= 2 {
        (year - 1, month as i32 + 12)
    } else {
        (year, month as i32)
    };
    let a = (y as f64 / 100.0).floor();
    let b = 2.0 - a + (a / 4.0).floor();
    (365.25 * (y as f64 + 4716.0)).floor()
        + (30.6001 * (m as f64 + 1.0)).floor()
        + day
        + b
        - 1524.5
}

/// 把本地民用时刻（含时区偏移，单位小时，如日本 +9、中国 +8）转为 JD(UT)。
#[must_use]
pub fn jd_from_local(
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
    second: f64,
    tz_hours: f64,
) -> f64 {
    let day_frac = (hour as f64 + minute as f64 / 60.0 + second / 3600.0) / 24.0;
    let jd_local = julian_day(year, month, day as f64 + day_frac);
    jd_local - tz_hours / 24.0
}

/// 民用日序号（整数 JDN），用于「落在哪一天」的判定与日柱递推。
/// 输入为本地日期（0 时）。
#[must_use]
pub fn civil_day_number(year: i32, month: u32, day: u32) -> i64 {
    (julian_day(year, month, day as f64) + 0.5).floor() as i64
}

/// 给定 JD(UT) 与时区，返回该时刻所在的本地民用日序号（整数 JDN）。
#[must_use]
pub fn local_civil_day_of(jd_ut: f64, tz_hours: f64) -> i64 {
    (jd_ut + tz_hours / 24.0 + 0.5).floor() as i64
}

/// ΔT（TT − UT，单位秒），Espenak–Meeus 分段多项式。
/// 各段按 NASA Espenak–Meeus 给定的起算历元与系数；覆盖 1900–2150（含本项目 1900–2100）。
/// 用于把「UT 的 JD」换成天文算法所需的「力学时 JDE」。
#[must_use]
pub fn delta_t_seconds(year: f64) -> f64 {
    if year < 1920.0 {
        let t = year - 1900.0;
        -2.79 + 1.494119 * t - 0.0598939 * t.powi(2) + 0.0061966 * t.powi(3) - 0.000197 * t.powi(4)
    } else if year < 1941.0 {
        let t = year - 1920.0;
        21.20 + 0.84493 * t - 0.076100 * t.powi(2) + 0.0020936 * t.powi(3)
    } else if year < 1961.0 {
        let t = year - 1950.0;
        29.07 + 0.407 * t - t.powi(2) / 233.0 + t.powi(3) / 2547.0
    } else if year < 1986.0 {
        let t = year - 1975.0;
        45.45 + 1.067 * t - t.powi(2) / 260.0 - t.powi(3) / 718.0
    } else if year < 2005.0 {
        let t = year - 2000.0;
        63.86 + 0.3345 * t - 0.060374 * t.powi(2)
            + 0.0017275 * t.powi(3)
            + 0.000651814 * t.powi(4)
            + 0.00002373599 * t.powi(5)
    } else if year < 2050.0 {
        let t = year - 2000.0;
        62.92 + 0.32217 * t + 0.005589 * t.powi(2)
    } else {
        // 2050–2150 段。
        let u = (year - 1820.0) / 100.0;
        -20.0 + 32.0 * u * u - 0.5628 * (2150.0 - year)
    }
}

/// 由 JD(UT) 估算所在公历年（用于 ΔT 取值）。
fn year_of_jd(jd_ut: f64) -> f64 {
    2000.0 + (jd_ut - 2451545.0) / 365.25
}

/// 格林尼治**平**恒星时（Greenwich mean sidereal time），单位度 `[0,360)`。
///
/// Meeus《Astronomical Algorithms》式 12.4：对任意瞬时（不限 0ʰ）的 GMST。恒星时由
/// 地球自转决定，故以世界时 JD(UT) 为自变量（非力学时）。本地恒星时 = GMST + 东经经度。
///
/// 用途：B 族（定位天文）算上升点/中天需本地恒星时 RAMC。
#[must_use]
pub fn mean_sidereal_time(jd_ut: f64) -> f64 {
    let d = jd_ut - 2451545.0;
    let t = d / 36525.0;
    (280.46061837 + 360.98564736629 * d + 0.000387933 * t * t - t * t * t / 38_710_000.0)
        .rem_euclid(360.0)
}

/// 黄道的**平**交角 ε₀（mean obliquity of the ecliptic），单位度。
///
/// Meeus 式 22.2（低精度档，±2000 年内有效，覆盖本项目年代）：
/// ε₀ = 23°26′21.448″ − 46.8150″·T − 0.00059″·T² + 0.001813″·T³，T 为自 J2000 的儒略世纪
/// （力学时）。不含章动；占星上升点/中天用平交角即足（与真交角差 ≤ ~9″，对 Asc 影响 < 1′）。
#[must_use]
pub fn mean_obliquity(jde: f64) -> f64 {
    let t = (jde - 2451545.0) / 36525.0;
    // ε₀ 以角秒表达：23°26′21.448″ = 84381.448″。
    let arcsec = 84_381.448 - 46.8150 * t - 0.00059 * t * t + 0.001813 * t * t * t;
    arcsec / 3600.0
}

/// JD(UT) → JDE（力学时）。
#[must_use]
pub fn jd_ut_to_jde(jd_ut: f64) -> f64 {
    jd_ut + delta_t_seconds(year_of_jd(jd_ut)) / 86400.0
}

/// 时刻的**共享天文/历法上下文**：对一个出生/问事时刻，把所有「时间→天文量」的公共子计算
/// （儒略日、力学时、太阳视黄经、民用日序、农历）一次性算出并缓存。
///
/// 这是「树即记忆化计算 DAG」的共享层：多片叶子（八字、紫微、择日…）共用同一个 `Moment`，
/// 避免各自重复昂贵的日月历法计算。各叶以 `compute_at(&Moment)` 复用它。
#[derive(Debug, Clone, Copy)]
pub struct Moment {
    /// 公历年。
    pub year: i32,
    /// 公历月 1..12。
    pub month: u32,
    /// 公历日 1..31。
    pub day: u32,
    /// 时 0..23。
    pub hour: u32,
    /// 分 0..59。
    pub minute: u32,
    /// 时区偏移小时。
    pub tz: f64,
    /// 世界时儒略日。
    pub jd_ut: f64,
    /// 力学时儒略日（含 ΔT）。
    pub jde: f64,
    /// 太阳视黄经（度）。
    pub sun_longitude: f64,
    /// 格林尼治平恒星时（度）——B 族算上升点/中天的共享量。
    pub sidereal_time: f64,
    /// 黄道平交角 ε₀（度）——B 族算上升点/中天的共享量。
    pub obliquity: f64,
    /// 民用日序（JDN）。
    pub civil_day: i64,
    /// 农历日期。
    pub lunar: LunarDate,
}

impl Moment {
    /// 由本地民用时刻构造，**一次性**算出全部共享天文/历法量。
    #[must_use]
    pub fn new(year: i32, month: u32, day: u32, hour: u32, minute: u32, tz: f64) -> Self {
        let jd_ut = jd_from_local(year, month, day, hour, minute, 0.0, tz);
        let jde = jd_ut_to_jde(jd_ut);
        Moment {
            year,
            month,
            day,
            hour,
            minute,
            tz,
            jd_ut,
            jde,
            sun_longitude: sun_apparent_longitude(jde),
            sidereal_time: mean_sidereal_time(jd_ut),
            obliquity: mean_obliquity(jde),
            civil_day: civil_day_number(year, month, day),
            lunar: solar_to_lunar(year, month, day, tz),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // —— ΔT 五个年代分段全覆盖（值与已知 ΔT 量级吻合，且不抛 NaN）——
    #[test]
    fn delta_t_all_branches() {
        for &(y, lo, hi) in &[
            (1910.0, -5.0, 12.0),  // <1920 段
            (1930.0, 22.0, 26.0),  // 1920–1941 段
            (1950.0, 28.0, 32.0),  // 1941–1961
            (1975.0, 44.0, 48.0),  // 1961–1986
            (1995.0, 60.0, 64.0),  // 1986–2005
            (2024.0, 68.0, 76.0),  // 2005–2050
            (2100.0, 90.0, 220.0), // >2050 外推
        ] {
            let dt = delta_t_seconds(y);
            assert!(dt > lo && dt < hi, "ΔT({y})={dt} 不在 [{lo}，{hi}]");
        }
    }

    // —— 儒略日/民用日序/JDE 的自洽 ——
    #[test]
    fn jd_and_civil_day() {
        // 2000-01-01 12:00 UT = JD 2451545.0
        assert!((julian_day(2000, 1, 1.5) - 2451545.0).abs() < 1e-6);
        // jd_from_local 在东八区 00：00 = 前一日 16：00 UT
        let jd = jd_from_local(2024, 1, 1, 0, 0, 0.0, 8.0);
        assert!((jd - (julian_day(2024, 1, 1.0) - 8.0 / 24.0)).abs() < 1e-9);
        // 民用日序连续
        assert_eq!(
            civil_day_number(2024, 1, 2) - civil_day_number(2024, 1, 1),
            1
        );
        // local_civil_day_of：东八区某 UT 时刻落在对应本地日
        let c = local_civil_day_of(julian_day(2024, 1, 1.0), 8.0);
        assert_eq!(c, civil_day_number(2024, 1, 1));
        // JDE = JD + ΔT；2024 年 ΔT≈70s
        assert!(jd_ut_to_jde(2451545.0) > 2451545.0);
    }

    // —— 共享上下文 Moment 一次算齐 ——
    #[test]
    fn moment_precomputes() {
        let m = Moment::new(1990, 6, 15, 14, 30, 8.0);
        assert_eq!((m.lunar.year, m.lunar.month, m.lunar.day), (1990, 5, 23));
        assert_eq!(m.civil_day, civil_day_number(1990, 6, 15));
        assert!((0.0..360.0).contains(&m.sun_longitude));
        assert!(m.jde > m.jd_ut); // JDE = JD + ΔT
    }

    // —— GMST 对 Meeus 12.4 教科书算例（1987-04-10 0ʰ UT， JD 2446895.5）——
    #[test]
    fn gmst_matches_meeus_example() {
        // Meeus AA 例 12.a：θ₀ = 13ʰ10ᵐ46.3668ˢ = 197.693195°。
        let g = mean_sidereal_time(2446895.5);
        assert!((g - 197.693195).abs() < 1e-4, "GMST={g}，应 ≈197.693195°");
        // 例 12.b（同日 19ʰ21ᵐ00ˢ UT）：8ʰ34ᵐ57.0896ˢ = 128.737873°。
        let g2 = mean_sidereal_time(2446895.5 + (19.0 + 21.0 / 60.0) / 24.0);
        assert!((g2 - 128.737873).abs() < 1e-4, "GMST={g2}，应 ≈128.737873°");
    }

    // —— 平黄赤交角对 Meeus 22.2 算例（例 22.a，1987-04-10）——
    #[test]
    fn obliquity_matches_meeus_example() {
        // ε₀ = 23°26′27.407″ = 23.440946°。
        let e = mean_obliquity(2446895.5);
        assert!((e - 23.440946).abs() < 1e-5, "ε₀={e}，应 ≈23.440946°");
    }

    // —— 从 L0 重导出的角度工具可用 ——
    #[test]
    fn reexported_angle_utils() {
        assert!((norm360(370.0) - 10.0).abs() < 1e-9);
        assert!((norm180(190.0) + 170.0).abs() < 1e-9);
    }

    // 日柱干支锚点测试已随 ganzhi 迁出至 mingli-ganzhi crate（其 day_pillar_anchors 测试）。

    /// 2024 节气精确时刻（北京时间 UTC+8）。节气定月柱边界，而月柱定格局与用神，
    /// 故这里日期必须精确，时刻只能差在模型已知的量级内。
    ///
    /// 参照两源：
    ///
    /// - 搜狐转发的天文科普稿《北京时间 2024 年 2 月 4 日 16 时 27 分迎来立春节气》
    ///   <https://www.sohu.com/a/756272634_121106902>
    /// - bmcx 节气表 <https://jieqi.bmcx.com/2024__jieqi/>，立春给到秒：`16:26:53`，
    ///   其余四个的日期同为 03-20 / 06-21 / 09-22 / 12-21
    ///
    /// **本算相对参照恒偏早**：实测 −7 / −3 / −3 / −8 / −7 分钟，五个同号。
    /// 这不是随机误差而是低精度太阳视黄经模型的系统偏置（λ 差 ~0.005° ≈ 7 分钟）。
    /// 容差取 12 分钟——比实测最大的 8 分钟留了半倍余量，又比原先的 15 分钟紧，
    /// 且**把「同号」这件事一并钉住**：偏置若翻号或翻倍，说明模型换了，该重新对源。
    #[test]
    fn solar_terms_2024_bjt() {
        // （公历月，日，时，分， 目标黄经）
        let cases = [
            (2, 4, 16, 27, 315.0),  // 立春
            (3, 20, 11, 6, 0.0),    // 春分
            (6, 21, 4, 51, 90.0),   // 夏至
            (9, 22, 20, 44, 180.0), // 秋分
            (12, 21, 17, 20, 270.0),// 冬至
        ];
        for (mo, d, hh, mm, lambda) in cases {
            let jd = solar_term_jd(2024, lambda);
            // 转回北京时间民用时刻
            let (ry, rmo, rd, rhh, rmm) = jd_ut_to_local_ymdhm(jd, 8.0);
            let got = format!("{ry:04}-{rmo:02}-{rd:02} {rhh:02}:{rmm:02}");
            let want = format!("2024-{mo:02}-{d:02} {hh:02}:{mm:02}");
            // 允许 ±2 分钟（低精度 Meeus + ΔT 误差）
            let want_min = (d as i64) * 1440 + hh as i64 * 60 + mm as i64;
            let got_min = (rd as i64) * 1440 + rhh as i64 * 60 + rmm as i64;
            // 日期必须精确——这是定柱的关键，差一天就是另一个月柱
            assert_eq!(rmo, mo, "节气 λ={lambda} 月份不符：got {got} want {want}");
            assert_eq!(rd, d, "节气 λ={lambda} 日期不符：got {got} want {want}");
            let diff = got_min - want_min;
            assert!(
                (-12..=0).contains(&diff),
                "节气 λ={lambda}：got {got} want {want}，差 {diff} 分钟——\
                 本算应恒偏早 0–12 分钟（低精度太阳黄经的系统偏置）。\
                 偏出这个区间说明模型变了，参照值要重新对源，不是把容差放宽",
            );
        }
    }

    // —— 春节（农历正月初一）公历日期，三源一致 ——
    #[test]
    fn spring_festivals() {
        let cases = [
            (2020, 1, 25),
            (2021, 2, 12),
            (2022, 2, 1),
            (2023, 1, 22),
            (2024, 2, 10),
            (2025, 1, 29),
        ];
        for (y, mo, d) in cases {
            let ld = solar_to_lunar(y, mo, d, 8.0);
            assert!(
                ld.year == y && ld.month == 1 && !ld.leap && ld.day == 1,
                "{y}-{mo:02}-{d:02} 应为农历 {y} 正月初一，实得 {ld:?}"
            );
        }
    }

    // —— 闰月：2023 闰二月（03-22 起）、2020 闰四月（05-23 起），HKO 权威表 ——
    #[test]
    fn leap_months() {
        let a = solar_to_lunar(2023, 3, 22, 8.0);
        assert!(
            a.month == 2 && a.leap && a.day == 1,
            "2023-03-22 应为农历闰二月初一，实得 {a:?}"
        );
        // 闰二月末日 4/19，4/20 应为三月初一
        let b = solar_to_lunar(2023, 4, 20, 8.0);
        assert!(
            b.month == 3 && !b.leap && b.day == 1,
            "2023-04-20 应为农历三月初一，实得 {b:?}"
        );
        let c = solar_to_lunar(2020, 5, 23, 8.0);
        assert!(
            c.month == 4 && c.leap && c.day == 1,
            "2020-05-23 应为农历闰四月初一，实得 {c:?}"
        );
    }

    // —— 十一/十二月（子月/丑月）归本岁起始年：late-Dec 日期农历年仍为当年 ——
    #[test]
    fn lunar_winter_month_year() {
        let ld = solar_to_lunar(2023, 12, 25, 8.0);
        assert_eq!(ld.year, 2023);
        assert!(ld.month == 11 || ld.month == 12, "实得 {ld:?}");
    }

    // —— 完整农历样例：1990-06-15 CST = 庚午年 五月廿三 ——
    #[test]
    fn lunar_sample_1990() {
        let ld = solar_to_lunar(1990, 6, 15, 8.0);
        assert_eq!(
            (ld.year, ld.month, ld.leap, ld.day),
            (1990, 5, false, 23),
            "1990-06-15 应为农历庚午年五月廿三"
        );
    }

    // 测试辅助：JD(UT) → 本地 （年，月，日，时，分）
    pub(super) fn jd_ut_to_local_ymdhm(jd_ut: f64, tz: f64) -> (i32, u32, u32, u32, u32) {
        let jd = jd_ut + tz / 24.0 + 0.5;
        let z = jd.floor();
        let f = jd - z;
        let mut a = z;
        if z >= 2299161.0 {
            let alpha = ((z - 1867216.25) / 36524.25).floor();
            a = z + 1.0 + alpha - (alpha / 4.0).floor();
        }
        let b = a + 1524.0;
        let c = ((b - 122.1) / 365.25).floor();
        let d = (365.25 * c).floor();
        let e = ((b - d) / 30.6001).floor();
        let day = b - d - (30.6001 * e).floor();
        let month = if e < 14.0 { e - 1.0 } else { e - 13.0 };
        let year = if month > 2.0 { c - 4716.0 } else { c - 4715.0 };
        let total_min = (f * 1440.0).round() as i64;
        let hh = (total_min / 60) as u32;
        let mm = (total_min % 60) as u32;
        (year as i32, month as u32, day as u32, hh, mm)
    }

    use proptest::prelude::*;
    /// 农历序列的结构性质——穷举 1900–2100 每一天。
    ///
    /// 现有几条农历测试钉的都是具体日期（1990 样本、历年春节、闰月表、子月）。
    /// 那种测试能确认「这一天算对了」，确认不了「不会在某处断开」。
    /// 下面五条是历法本身必须成立的东西，任何一条破了，上面所有吃农历的叶都跟着错，
    /// 而错法多半是某个月的边界上少一天或多一天——按日期抽查恰好最难发现。
    ///
    /// 实测（2026-08-23）：73 414 天，日恒在 1..=30、完整月恒为 29 或 30 天、
    /// 一农历年至多一个闰月、逐日无缝、201 个正月初一落在 1-21 至 2-20 之间。
    #[test]
    fn the_lunar_sequence_has_no_seams_across_two_centuries() {
        use std::collections::{BTreeMap, BTreeSet};
        let tz = 8.0;
        let days_in = |y: i32, m: u32| -> u32 {
            match m {
                1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
                4 | 6 | 9 | 11 => 30,
                _ => u32::from((y % 4 == 0 && y % 100 != 0) || y % 400 == 0) + 28,
            }
        };

        let mut prev: Option<(crate::LunarDate, i64)> = None;
        let mut month_days: BTreeMap<(i32, u32, bool), u32> = BTreeMap::new();
        let mut new_year_at: Vec<(u32, u32)> = Vec::new();
        let mut n = 0u32;

        for y in 1900..=2100 {
            for m in 1..=12u32 {
                for d in 1..=days_in(y, m) {
                    let l = crate::solar_to_lunar(y, m, d, tz);
                    let cdn = civil_day_number(y, m, d);
                    n += 1;

                    // ① 日恒在 1..=30
                    assert!((1..=30).contains(&l.day), "{y}-{m:02}-{d:02} 得农历日 {}", l.day);
                    // ② 月序恒在 1..=12
                    assert!((1..=12).contains(&l.month), "{y}-{m:02}-{d:02} 得农历月 {}", l.month);
                    *month_days.entry((l.year, l.month, l.leap)).or_insert(0) += 1;
                    if l.month == 1 && l.day == 1 && !l.leap {
                        new_year_at.push((m, d));
                    }

                    // ③ 逐日无缝：公历相邻两日，农历要么日 +1，要么翻月落初一
                    if let Some((p, pc)) = prev
                        && cdn == pc + 1
                    {
                        let same_month = l.month == p.month && l.leap == p.leap;
                        let ok = if same_month {
                            l.day == p.day + 1
                        } else {
                            l.day == 1 && (29..=30).contains(&p.day)
                        };
                        assert!(
                            ok,
                            "{y}-{m:02}-{d:02} 处农历断开：前一日 {}年{}{}月{}日，本日 {}年{}{}月{}日",
                            p.year, if p.leap { "闰" } else { "" }, p.month, p.day,
                            l.year, if l.leap { "闰" } else { "" }, l.month, l.day,
                        );
                    }
                    prev = Some((l, cdn));
                }
            }
        }
        assert_eq!(n, 73_414, "扫描规模变了，下面几条实测结论要跟着重验");

        // ④ 完整月只能是 29 或 30 天（首末两月被扫描区间截断，不计）
        let full: BTreeSet<u32> = month_days.values().copied().filter(|&v| v >= 20).collect();
        assert_eq!(full, BTreeSet::from([29, 30]), "完整农历月的天数集合应恰为 {{29,30}}，实得 {full:?}");

        // ⑤ 一个农历年至多一个闰月
        let mut leaps: BTreeMap<i32, u32> = BTreeMap::new();
        for (yy, _, is_leap) in month_days.keys() {
            if *is_leap {
                *leaps.entry(*yy).or_insert(0) += 1;
            }
        }
        assert!(
            leaps.values().all(|&c| c == 1),
            "有农历年出现不止一个闰月：{:?}",
            leaps.iter().filter(|&(_, &c)| c != 1).collect::<Vec<_>>()
        );

        // ⑥ 正月初一恒落在 1-21 至 2-20 之间（201 年实测的真实区间）
        assert_eq!(new_year_at.len(), 201, "1900–2100 应有 201 个正月初一");
        let earliest = new_year_at.iter().min().expect("非空");
        let latest = new_year_at.iter().max().expect("非空");
        assert_eq!(*earliest, (1, 21), "最早的正月初一应是 1 月 21 日");
        assert_eq!(*latest, (2, 20), "最晚的正月初一应是 2 月 20 日");
    }

    proptest! {
        #[test]
        fn prop_sidereal_time_in_range(jd in 2_400_000.0f64..2_500_000.0) {
            prop_assert!((0.0..360.0).contains(&mean_sidereal_time(jd)));
        }
        #[test]
        fn prop_obliquity_modern_range(jde in 2_400_000.0f64..2_500_000.0) {
            // 现代纪元黄赤交角 ε₀ ≈ 23.4°。
            let e = mean_obliquity(jde);
            prop_assert!(e > 23.0 && e < 24.0);
        }
        #[test]
        fn prop_sun_longitude_in_range(jde in 2_400_000.0f64..2_500_000.0) {
            prop_assert!((0.0..360.0).contains(&sun_apparent_longitude(jde)));
        }
        #[test]
        fn prop_solar_term_longitude_roundtrip(year in 1950i32..2050, lambda in 1.0f64..359.0) {
            // solar_term_jd 求出的时刻，其太阳视黄经应≈请求的 λ（求解器往返自洽）。
            let l = sun_apparent_longitude(jd_ut_to_jde(solar_term_jd(year, lambda)));
            let diff = (l - lambda).rem_euclid(360.0);
            prop_assert!(diff.min(360.0 - diff) < 0.05, "λ={} got {}", lambda, l);
        }
        #[test]
        fn prop_civil_day_consecutive(d in 1u32..28) {
            prop_assert_eq!(civil_day_number(2000, 1, d + 1), civil_day_number(2000, 1, d) + 1);
        }
    }
}

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

    // —— ΔT 七个年代分段，逐值钉住 ——
    //
    // 从前这里给的是每段一个 ±2 秒的区间。区间对「量级没错」有用，对「系数有没有被改过」
    // 没用：十六个改系数的变异体全部落在带内活着。系数出自 Espenak–Meeus 且已在
    // `delta_t_seconds` 的注释里引用，那么它的输出就是确定的，该逐值钉。
    //
    // 这些是公式的**推算值**，不是观测值。2005 年以后那两段是外推，与实测 ΔT 的偏离
    // 本处没有核对——本环境里找不到第二个独立实现或权威表（`astro` crate 把 ΔT 当参数
    // 收，不自己算）。按铁律三，查不到就不写，而不是编一个看起来合理的数。
    // 物理上这也不吃紧：ΔT 差一秒，月亮动约 0.5″、行星远小于此，而公开盘只到角分。
    #[test]
    fn delta_t_all_branches() {
        for &(y, want) in &[
            // 每段取一个**不在该段起算历元上**的年份。第一版给 1941–1961 取 1950、
            // 给 1961–1986 取 1975，而那两段的 t = year − 1950 / year − 1975 在那两年
            // 恰好为零，多项式塌成常数项，系数怎么改都看不出来——同一个反模式，
            // 我在修它的时候又犯了一次。
            (1910.0, 10.388_400),   // <1920，t=10
            (1930.0, 24.132_900),   // 1920–1941，t=10
            (1955.0, 31.046_781),   // 1941–1961，t=5
            (1980.0, 50.514_751),   // 1961–1986，t=5
            (1995.0, 60.795_421),   // 1986–2005，t=−5
            (2024.0, 73.871_344),   // 2005–2050，t=24
            (2100.0, 202.740_000),  // >2050 外推
            // 分段边界本身。`<` 写成 `<=` 只在年份正好等于边界时分叉，
            // 段内任何取样都看不见——六个边界各取一次。
            (1920.0, 21.200_000),
            (1941.0, 24.773_141),
            (1961.0, 33.579_881),
            (1986.0, 54.877_738),
            (2005.0, 64.670_575),
            (2050.0, 93.000_000),
        ] {
            let dt = delta_t_seconds(y);
            assert!(
                (dt - want).abs() < 1e-5,
                "ΔT({y}) = {dt}，应为 {want}——系数动过了"
            );
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
        // 上面那一行取的是时分秒全零，而日内小数正是 `(时 + 分/60 + 秒/3600) / 24`——
        // 全零时整条算式归零，里面的加号除号怎么改都看不出来（十一个变异体因此活着）。
        // 时分秒都给上非零值，每个运算符才各自可见。
        for &(h, mi, sec, tz, want_frac) in &[
            (6_u32, 0_u32, 0.0_f64, 0.0_f64, 6.0 / 24.0),
            (0, 30, 0.0, 0.0, 0.5 / 24.0),
            (0, 0, 45.0, 0.0, (45.0 / 3600.0) / 24.0),
            (13, 47, 23.5, 8.0, (13.0 + 47.0 / 60.0 + 23.5 / 3600.0) / 24.0 - 8.0 / 24.0),
        ] {
            let got = jd_from_local(2024, 3, 15, h, mi, sec, tz);
            let want = julian_day(2024, 3, 15.0) + want_frac;
            assert!(
                // 1e-9 天 ≈ 86 µs。放到这里不是迁就错误：期望值这一侧的求和次序与实现
                // 不同，末位差约 2e-10；而任何一个运算符被改动造成的偏移都在小时量级。
                (got - want).abs() < 1e-9,
                "jd_from_local(2024-03-15 {h}:{mi}:{sec} tz{tz}) = {got}，应为 {want}"
            );
        }
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

    /// GMST 的高阶项与 `year_of_jd`，在拉开的历元上钉住。
    ///
    /// 与 [`obliquity_higher_order_terms_are_pinned_at_far_epochs`] 同一类问题：
    /// `0.000387933·t²` 在 |t| ≤ 1 内只有 1.4″，`t³/38710000` 更只有 2.6e-8 度——
    /// 1900–2100 内取样看不见，变异扫描下留了五个存活。t = ±10 上后者涨到
    /// 2.6e-5 度，落在这里的 1e-7 容差之上，于是可测。
    ///
    /// `year_of_jd` 是私有的，只把 JD 折成公历年喂给 [`delta_t_seconds`] 选分段。
    /// 它错到几十年才会换段，而换段处 ΔT 本来就连续——所以经由 ΔT 的间接路径
    /// 观测不到它，只能直接钉。这是「私有不等于不用守」：它的输出是 ΔT 的自变量，
    /// 错了整条月相与节气都跟着偏。
    ///
    /// 两组期望值都由本文件当前的算式逐项算出，钉的是转写不是权威值。
    #[test]
    fn sidereal_and_year_helpers_are_pinned_across_epochs() {
        for &(jd, want) in &[
            (2_086_302.50_f64, 100.191_253_752_f64), // t = −10
            (2_415_020.50, 100.183_776_259),         // t = −1
            (2_451_545.00, 280.460_618_370),         // t = 0，J2000.0 正午
            (2_451_545.25, 10.707_030_212),          // 同日 18ʰ，验非整日
            (2_488_070.50, 101.723_883_713),         // t = +1
            (2_816_788.50, 101.793_213_993),         // t = +10
        ] {
            let got = mean_sidereal_time(jd);
            assert!(
                (got - want).abs() < 1e-7,
                "GMST(JD {jd}) = {got}，应为 {want}——系数动过了"
            );
        }

        for &(jd, want) in &[
            (2_086_302.5_f64, 1_000.020_533_881_f64),
            (2_415_020.5, 1_900.001_368_925),
            (2_451_545.0, 2_000.0),
            (2_488_070.5, 2_100.001_368_925),
            (2_816_788.5, 2_999.982_203_970),
        ] {
            let got = year_of_jd(jd);
            assert!(
                (got - want).abs() < 1e-9,
                "year_of_jd({jd}) = {got}，应为 {want}"
            );
        }
    }

    /// 太阳视黄经的各项系数，在拉开的历元上钉住。
    ///
    /// 现有的守卫有两条，都够不着这些系数：proptest 只验值域落在 0–360，
    /// 而「节气往返」是**自指的**——`solar_term_jd` 反解的正是这个函数，
    /// 函数变了往返照样自洽。节气那条对公开值的 oracle 够得着主系数
    /// （改动 280.46646 会红），够不着一阶以上的小项。
    ///
    /// 这里钉的是转写：期望值由本函数当前的系数逐项算出，不是新的权威值，
    /// 也不冒充观测。它能答的问题只有一个——「有没有人改过这些数字」。
    /// 正确性的判据仍是节气那条对公开值的比对。
    ///
    /// 取样拉到 t = ±10：`0.0003032·t²` 这类项在 |t| ≤ 1 内只有角秒量级，
    /// 段内取样看不见；而 `mingli-astro` 是独立发布的 crate，用它的人不受
    /// 本项目 1900–2100 那个范围约束。
    ///
    /// JDE 2448908.5 是 Meeus 例 25.b 的日期。本环境里查不到那本书，
    /// 所以这里不写它印的那个值，只钉我们自己算出来的——按铁律三，
    /// 查不到就不写，而不是凭记忆填一个看着对的数。
    #[test]
    fn sun_longitude_coefficients_are_pinned_across_epochs() {
        for &(jde, want) in &[
            (2_086_302.5_f64, 280.681_233_688_f64), // 约公元 1000，t = −10
            (2_415_020.5, 280.153_570_628),         // 1900，t = −1
            (2_448_908.5, 199.908_941_860),         // 1992-10-13，Meeus 例 25.b 的日期
            (2_451_545.0, 280.372_554_879),         // 2000，t = 0
            (2_488_070.5, 281.624_859_618),         // 2100，t = +1
            (2_816_788.5, 281.186_020_342),         // 约公元 3000，t = +10
        ] {
            let got = sun_apparent_longitude(jde);
            assert!(
                (got - want).abs() < 1e-8,
                "λ☉(JDE {jde}) = {got}，应为 {want}——系数动过了"
            );
        }
    }

    /// ε₀ 的高阶项，在远历元上钉住。
    ///
    /// 上面那条算例落在 t≈−0.13 上，而 t² 与 t³ 项在本项目支持的 1900–2100
    /// （|t| ≤ 1）内最大只贡献 0.00059″ 与 0.00181″——远在那条 1e-5 度容差之下，
    /// 段内任何取样都看不见它们。变异扫描下这两个系数因此留了十一个存活。
    ///
    /// 但 `mingli-astro` 是独立发布的 crate：单独取用它的人可以在任何历元上调
    /// [`mean_obliquity`]，而那时高阶项就是主角（t=±10 时 t³ 项到 1.8″）。
    /// 钉的是这个函数的契约，不只是本项目当前的用法——与
    /// `mingli-gua` 那条「只认这十六个字」同理。
    ///
    /// 期望值由 `84381.448 − 46.8150·t − 0.00059·t² + 0.001813·t³`（Meeus 22.2，
    /// 已在函数注释里引用）逐项算出，不是新的权威值。
    #[test]
    fn obliquity_higher_order_terms_are_pinned_at_far_epochs() {
        for &(jde, want) in &[
            (2_086_302.5_f64, 23.568_810_139_f64), // 约公元 1000，t = −10
            (2_415_020.5, 23.452_294_432),         // 1900，t = −1
            (2_451_545.0, 23.439_291_111),         // 2000，t = 0
            (2_488_070.5, 23.426_287_106),         // 2100，t = +1
            (2_816_788.5, 23.309_738_955),         // 约公元 3000，t = +10
        ] {
            let got = mean_obliquity(jde);
            assert!(
                (got - want).abs() < 1e-9,
                "ε₀(JDE {jde}) = {got}，应为 {want}——系数动过了"
            );
        }
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
        // （公历月，日，时，分，目标黄经，本算相对公开值的偏差·分钟）
        //
        // 末一列是实测记录，逐个节气各有各的值——不是一个共用的容差窗口。见下方说明。
        let cases = [
            (2, 4, 16, 27, 315.0, -7),  // 立春
            (3, 20, 11, 6, 0.0, -3),    // 春分
            (6, 21, 4, 51, 90.0, -3),   // 夏至
            (9, 22, 20, 44, 180.0, -8), // 秋分
            (12, 21, 17, 20, 270.0, -7),// 冬至
        ];
        for (mo, d, hh, mm, lambda, expected) in cases {
            let jd = solar_term_jd(2024, lambda);
            // 转回北京时间民用时刻
            let (ry, rmo, rd, rhh, rmm) = jd_ut_to_local_ymdhm(jd, 8.0);
            let got = format!("{ry:04}-{rmo:02}-{rd:02} {rhh:02}:{rmm:02}");
            let want = format!("2024-{mo:02}-{d:02} {hh:02}:{mm:02}");
            let want_min = (d as i64) * 1440 + hh as i64 * 60 + mm as i64;
            let got_min = (rd as i64) * 1440 + rhh as i64 * 60 + rmm as i64;
            // 日期必须精确——这是定柱的关键，差一天就是另一个月柱
            assert_eq!(rmo, mo, "节气 λ={lambda} 月份不符：got {got} want {want}");
            assert_eq!(rd, d, "节气 λ={lambda} 日期不符：got {got} want {want}");
            // 逐个节气钉住各自的偏差，而不是给一个共用的窗口。
            //
            // 从前这里是 `(-12..=0)`：十二分钟宽，而实测偏差只在 3–8 分钟之间
            // （低精度太阳黄经的系统性偏早）。那点余量足够藏下真错——把
            // `1.914602 - 0.004817·t` 的减号改成加号，五个节气的偏差变成
            // −8/−6/−4/−5/−6，与正确的 −7/−3/−3/−8/−7 在**总区间上几乎完全重叠**，
            // 一个窗口分不开，逐个就分得开（λ=0 差 3 分钟，λ=180 差 3 分钟）。
            //
            // 旁边那句注释当时写着「允许 ±2 分钟」，而代码给的是 12 分钟——
            // 说明与实现对不上的注释比没有注释更糟。
            let diff = got_min - want_min;
            assert!(
                (diff - expected).abs() <= 1,
                "节气 λ={lambda}：got {got} want {want}，差 {diff} 分钟，实测记录是 {expected}——\
                 偏出这一分钟说明太阳黄经那一路变了，参照值要重新对源，不是把容差放宽",
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
    /// 朔的**时刻**对不对，此前没有任何一处在看。整个仓库只断言朔落在哪一民用日，
    /// 而那一步把瞬时量化掉了：只要扰动没把朔推过午夜，就一律看不见。实测把七处
    /// 算术逐个改坏、每次跑全量套件，主项（约 0.4 天）与次项（约 4 小时）都有测试
    /// 拦住，而朔望月长度末位改一（两百年累计约 13 秒）、ΔT 1961–1986 段三次项翻号
    /// （约 8 秒）一条都不红——它们改的是秒，午夜离得远。
    ///
    /// 所以这里改成直接对时刻。取值两源相合，两源各自独立推算，不是互抄：
    ///
    /// 1. 美国海军天文台历书处 <https://aa.usno.navy.mil/calculated/moon/phases>
    /// 2. Fred Espenak《Six Millennium Catalog of Phases of the Moon》
    ///    <https://www.astropixels.com/ephemeris/phasescat/phases1901.html>
    ///
    /// 七个朔横跨 1901–2050，两源逐条一致，只有 1950-01-18 差一分钟（07:59 与 08:00），
    /// 取海军天文台的那个。实测最大偏差 0.547 分钟，容差按实测取 2 分钟。
    ///
    /// 说清它拦得住什么、拦不住什么：秒级的扰动它一样看不见（模型自身与两源就差半分钟，
    /// 再紧就是在钉噪声）。它补上的是另一件事——此前拿去跟外界比对的只有春节日期与闰月
    /// 这些**日**粒度的锚，整条模型均匀平移几分钟，序列依旧自洽、月长依旧 29 或 30，
    /// 没有一处会红。现在时刻本身有了七个外部锚点。
    ///
    /// `k` 直接写死而不是由日期反推——反推要用平朔公式，那正是被测对象之一。写死之后，
    /// 朔望月长度只要动一位，同一个 `k` 指向的就是几周之外的另一个朔。
    #[test]
    fn the_new_moon_instants_match_two_published_ephemerides() {
        // (k, 年, 月, 日, 时, 分) —— 时刻为世界时。
        const PUBLISHED: [(i64, i32, u32, u32, u32, u32); 7] = [
            (-1224, 1901, 1, 20, 14, 36),
            (-618, 1950, 1, 18, 7, 59),
            (300, 2024, 4, 8, 18, 21),
            (301, 2024, 5, 8, 3, 22),
            (304, 2024, 8, 4, 11, 13),
            (309, 2024, 12, 30, 22, 27),
            (619, 2050, 1, 23, 4, 57),
        ];
        const TOLERANCE_MINUTES: f64 = 2.0;

        let mut worst = 0.0f64;
        for (k, y, m, d, hour, minute) in PUBLISHED {
            let published =
                julian_day(y, m, f64::from(d) + (f64::from(hour) + f64::from(minute) / 60.0) / 24.0);
            let computed = new_moon_jd_ut(k);
            let off_minutes = (computed - published) * 24.0 * 60.0;
            assert!(
                off_minutes.abs() < TOLERANCE_MINUTES,
                "第 {k} 个朔：算出 JD {computed:.5}，两源作 {y}-{m:02}-{d:02} {hour:02}:{minute:02} UT \
                 (JD {published:.5})，差 {off_minutes:.2} 分钟"
            );
            worst = worst.max(off_minutes.abs());
        }
        // 真实误差应远小于容差；一旦逼近，说明模型已经变了而不只是抖动。
        // 实测（2026-08-23）最大偏差 0.547 分钟。留出四倍余量，但远紧于模块自述的
        // 「约数分钟」——文档那句是保守说法，实际吻合度高得多，容差按实测定。
        assert!(worst < 2.0, "最大偏差 {worst:.3} 分钟，模型已经变了");
    }

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

    /// 朔的时刻，与另一份转写逐个比。
    ///
    /// 上面那条拿的是两源公开朔时刻，容差 2 分钟——公开值只给到分，钉不了更紧，
    /// 于是 `new_moon_jd_ut` 里那些小周期项无人过问：变异扫描下它一个函数就留了
    /// 193 个存活，全是改动幅度低于分钟级分辨率的系数。
    ///
    /// 这一条换个方向：`astro` crate 是同一套 Meeus 第 49 章公式的**另一份转写**。
    /// 算法不独立（所以它不能当正确性权威，那仍是上面那条公开值的事），
    /// 但转写独立——我们这边抄错任何一个系数，两边立刻分岔。实测它们逐位相同。
    #[test]
    fn our_new_moons_match_an_independent_transcription() {
        use astro::{lunar, time};
        let (mut worst, mut matched) = (0.0_f64, 0_u32);
        for k in -5000..=20_000_i64 {
            let ours = crate::new_moon_jd_ut(k);
            let z = (ours + 0.5).floor();
            let f = ours + 0.5 - z;
            let a = if z < 2_299_161.0 {
                z
            } else {
                let al = ((z - 1_867_216.25) / 36524.25).floor();
                z + 1.0 + al - (al / 4.0).floor()
            };
            let b = a + 1524.0;
            let cc = ((b - 122.1) / 365.25).floor();
            let d = (365.25 * cc).floor();
            let e = ((b - d) / 30.6001).floor();
            let day = b - d - (30.6001 * e).floor() + f;
            let month = if e < 14.0 { e - 1.0 } else { e - 13.0 };
            let year = if month > 2.0 { cc - 4716.0 } else { cc - 4715.0 };
            #[allow(clippy::cast_possible_truncation, reason = "年月落在 1500–2100，窄化安全")]
            let date = time::Date {
                year: year as i16,
                month: month as u8,
                decimal_day: day,
                cal_type: time::CalType::Gregorian,
            };
            let theirs_jde = lunar::time_of_phase(&date, &lunar::Phase::New);
            let theirs_ut = theirs_jde - delta_t_seconds(year_of_jd(theirs_jde)) / 86400.0;
            let dm = (ours - theirs_ut) * 24.0 * 60.0;
            // 差超过半个朔望月，说明 astro 按日期反推时落到了相邻的朔——那不是分歧。
            if dm.abs() > 15.0 * 24.0 * 60.0 {
                continue;
            }
            worst = worst.max(dm.abs());
            matched += 1;
        }
        assert!(matched > 19_000, "只对上 {matched} 个朔，取样太少");
        // 容差与取样范围都按**实测**定，不按「看起来够」定。
        //
        // 实测（2026-09-01）：k 从 −5000 到 20000（约公元 1596–3618）共 20001 个朔，
        // 最大差 5.364e-6 分钟。此前这里是 0.001 的容差配 ±1200 的范围，两处都留得太宽：
        // 容差宽出 250 倍，范围又让 |t| 始终小于 1（t = k/1236.85）。角度式里的 t³、t⁴ 项
        // 在那里贡献不到 1e-8 分钟，改了没人看得见——变异扫描下 `new_moon_jd_ut`
        // 一个函数留了十七个漏网，全在这两层余量里。
        //
        // 把范围拉到 |t| ≈ 16，那些项涨到 1e-4～1e-1 分钟，于是可测；而两份实现在这段
        // 上仍然吻合到 5e-6，说明拉开的是**取样**不是分歧。范围不是在声称模型到 3618 年
        // 还准（模块自述是「约数分钟」），只是在说：同一组公式的两次转写，在这么长一段上
        // 逐个对得上。
        assert!(
            worst < 1e-5,
            "与另一份转写最大差 {worst:.3e} 分钟，实测记录是 5.364e-6——有一侧的系数动过了"
        );
    }
}

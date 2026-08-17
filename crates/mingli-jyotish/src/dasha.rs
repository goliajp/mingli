//! Vimśottarī daśā：以月宿主星起算的 120 年大运周期。

use super::*;

/// Vimshottari mahadasha 主星序列（9 步循环，周期 120 年）。第 i 个 nakshatra 由
/// `VIMSHOTTARI_LORDS[i % 9]` 主管；循环顺序固定为 Ketu/Venus/Sun/Moon/Mars/Rahu/Jupiter/Saturn/Mercury。
pub const VIMSHOTTARI_LORDS: [&str; 9] = [
    "Ketu", "Venus", "Sun", "Moon", "Mars", "Rahu", "Jupiter", "Saturn", "Mercury",
];

/// Vimshottari mahadasha 主星与对应总年数（120 年周期）。9 步固定顺序循环：
/// Ketu 7 / Venus 20 / Sun 6 / Moon 10 / Mars 7 / Rahu 18 / Jupiter 16 / Saturn 19 / Mercury 17。
/// 总和 = 120。
pub const VIMSHOTTARI_YEARS: [(&str, f64); 9] = [
    ("Ketu", 7.0), ("Venus", 20.0), ("Sun", 6.0), ("Moon", 10.0),
    ("Mars", 7.0), ("Rahu", 18.0), ("Jupiter", 16.0), ("Saturn", 19.0), ("Mercury", 17.0),
];

/// 一段 mahadasha（主星 + 起止儒略日 + 持续年数）。
#[derive(Debug, Clone, Serialize)]
pub struct Mahadasha {
    /// 主星名（IAST，9 之一）。
    pub lord: &'static str,
    /// 主星总持续年数（Vimshottari 固定）。
    pub years: f64,
    /// 本段实际持续年数（birth dasha 可能 < years，其后 = years）。
    pub effective_years: f64,
    /// 起儒略日(UT)。
    pub start_jd: f64,
    /// 止儒略日(UT)。
    pub end_jd: f64,
    /// 起公历近似年（自 birth 起算的年龄，可负 = 出生前残段；通常 birth dasha 起 = 0 之负值）。
    pub start_age_years: f64,
    /// 止公历近似年龄。
    pub end_age_years: f64,
}

/// 从月亮恒星黄经 + 出生 jd_ut 派生 Vimshottari mahadasha 9 段 timeline（共 120 年）。
///
/// 算法：
/// 1. birth dasha 主星 = 月亮 nakshatra 的 Vimshottari 主星；
/// 2. 月亮在该 nakshatra 已过比例 `elapsed = (lon % 13°20') / 13°20'`；
/// 3. birth dasha **剩余** 年数 = `(1 − elapsed) × lord_years`；
/// 4. birth dasha 名义起始 = `birth − elapsed × lord_years`（出生前的"残段"）；
/// 5. 之后顺序循环 Vimshottari 9 步，各占固定年数。
///
/// 转儒略日：1 平年 = 365.25 d（传统印度占星 Vimshottari 用儒略年）。
#[must_use]
pub fn vimshottari_timeline(moon_sidereal_lon: f64, birth_jd_ut: f64) -> Vec<Mahadasha> {
    const NAKSHATRA_SPAN: f64 = 360.0 / 27.0; // 13°20'
    const DAYS_PER_YEAR: f64 = 365.25;
    let lon = moon_sidereal_lon.rem_euclid(360.0);
    let naks = (lon / NAKSHATRA_SPAN).floor() as usize % 27;
    let elapsed_frac = (lon / NAKSHATRA_SPAN).fract();
    // birth dasha 在 Vimshottari 9 步中的索引
    let start_step = naks % 9;
    let (start_lord, start_years) = VIMSHOTTARI_YEARS[start_step];
    let birth_dasha_age_start = -elapsed_frac * start_years;
    let birth_dasha_age_end = (1.0 - elapsed_frac) * start_years;

    let mut out: Vec<Mahadasha> = Vec::with_capacity(9);
    let mut age = birth_dasha_age_start;
    for i in 0..9 {
        let (lord, years) = VIMSHOTTARI_YEARS[(start_step + i) % 9];
        let effective = if i == 0 { start_years } else { years };
        let next_age = age + effective;
        out.push(Mahadasha {
            lord,
            years: effective,
            effective_years: if i == 0 { birth_dasha_age_end - birth_dasha_age_start } else { effective },
            start_jd: birth_jd_ut + age * DAYS_PER_YEAR,
            end_jd: birth_jd_ut + next_age * DAYS_PER_YEAR,
            start_age_years: age,
            end_age_years: next_age,
        });
        age = next_age;
    }
    debug_assert_eq!(out[0].lord, start_lord);
    out
}

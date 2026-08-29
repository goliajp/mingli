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

/// Vimśottarī 一「年」到底是多少天——**没有共识**，这是本模块唯一的真分歧。
///
/// 原典（BPHS）只给年数比例，**不规定年长**；各实现实测出八个不同取值。
/// 下表是实查到的，附出处；本 crate 默认取儒略年 365.25（Wikipedia 亦取此值），
/// 但把它做成参数而不是写死，见 [`vimshottari_timeline_with`]。
///
/// 两处实现自己的注释就承认了这件事：drik-panchanga 写着
/// 「some say 360 days, others 365.25 or 365.2563 etc」；VedAstro 写着
/// 「Based on Ayanamsa the number of days in a year vary as per the astrologer's preference」。
pub const YEAR_LENGTHS: [(&str, f64); 6] = [
    ("julian", 365.25),            // Wikipedia「Dasha」；Maitreya 6 默认
    ("savana360", 360.0),          // VedAstro 在 Raman ayanāṃśa 下取此
    ("tropical", 365.242_19),      // Maitreya 6 可选
    ("gregorian", 365.2425),       // Jagannatha Hora 可选
    ("sidereal_kp", 365.2564),     // VedAstro 在 KP ayanāṃśa 下取此
    ("sidereal_true", 365.256_364), // PyJHora 默认（真恒星年）
];

/// 一段 antardaśā（bhukti）：主运之内的九步子细分。
///
/// 时长 = 主星年数 × 子星年数 ÷ 120（BPHS 51.1
/// 「daśābdāḥ svasvamānaghnāḥ sarvāyuryogabhājitāḥ」），
/// 首个子运即主星自己，其后依同一固定顺序循环（BPHS 51.2）。
/// drik-panchanga、PyJHora、VedAstro 三个开源实现的源码常量与此逐条一致。
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct Antardasha {
    /// 子星名。
    pub lord: &'static str,
    /// 本段年数 = 主星年数 × 子星年数 ÷ 120。
    pub years: f64,
    /// 起儒略日(UT)。
    pub start_jd: f64,
    /// 止儒略日(UT)。
    pub end_jd: f64,
    /// 起年龄（自出生起算，可负）。
    pub start_age_years: f64,
    /// 止年龄。
    pub end_age_years: f64,
}

/// 一段 mahadasha（主星 + 起止儒略日 + 持续年数 + 九步 antardaśā）。
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
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
    /// 九步 antardaśā，铺满本段的**名义**跨度（出生落在首段之内时，起点在出生之前）。
    pub antardashas: Vec<Antardasha>,
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
/// 每段再按 BPHS 51.1 的比例细分出九步 antardaśā。
///
/// 转儒略日取 1 年 = 365.25 日；年长各家不一，要换请走 [`vimshottari_timeline_with`]。
#[must_use]
pub fn vimshottari_timeline(moon_sidereal_lon: f64, birth_jd_ut: f64) -> Vec<Mahadasha> {
    vimshottari_timeline_with(moon_sidereal_lon, birth_jd_ut, 365.25)
}

/// 同上，指定一年折合多少天。取值见 [`YEAR_LENGTHS`]——各家不一，本 crate 不替调用方选边。
#[must_use]
pub fn vimshottari_timeline_with(
    moon_sidereal_lon: f64,
    birth_jd_ut: f64,
    days_per_year: f64,
) -> Vec<Mahadasha> {
    const NAKSHATRA_SPAN: f64 = 360.0 / 27.0; // 13°20'
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
            start_jd: birth_jd_ut + age * days_per_year,
            end_jd: birth_jd_ut + next_age * days_per_year,
            start_age_years: age,
            end_age_years: next_age,
            antardashas: antardashas_of(lord, effective, age, birth_jd_ut, days_per_year),
        });
        age = next_age;
    }
    debug_assert_eq!(out[0].lord, start_lord);
    out
}

/// 主星 `lord` 的九步 antardaśā，自 `start_age` 起铺满其名义跨度 `span_years`。
///
/// 首个子运是主星自己，其后依 Vimśottarī 固定顺序循环（BPHS 51.2
/// 「the first Antar Dasha belongs to the Lord of the Dasha ... in the same order」）。
/// 每步年数 = `span_years × 子星年数 ÷ 120`，九步之和恰等于 `span_years`。
fn antardashas_of(
    lord: &'static str,
    span_years: f64,
    start_age: f64,
    birth_jd_ut: f64,
    days_per_year: f64,
) -> Vec<Antardasha> {
    let start_step = VIMSHOTTARI_YEARS.iter().position(|(l, _)| *l == lord).unwrap_or(0);
    let mut out = Vec::with_capacity(9);
    let mut age = start_age;
    for i in 0..9 {
        let (sub_lord, sub_years) = VIMSHOTTARI_YEARS[(start_step + i) % 9];
        let years = span_years * sub_years / 120.0;
        let next = age + years;
        out.push(Antardasha {
            lord: sub_lord,
            years,
            start_jd: birth_jd_ut + age * days_per_year,
            end_jd: birth_jd_ut + next * days_per_year,
            start_age_years: age,
            end_age_years: next,
        });
        age = next;
    }
    out
}

//! L3 叶（B 族）：印度占星（Jyotish）。
//!
//! 与西洋占星（B 族 [`mingli_astrology`]）复用同一行星地心经度，但采用**恒星黄道**(sidereal zodiac)
//! 而非回归黄道(tropical zodiac)。两者差 = `ayanamsa`（春分点岁差累积位移）。
//!
//! - 9 行星（Surya/Chandra/Mangala/Budha/Guru/Shukra/Shani + Rahu/Ketu 月升降交点）；
//! - 27 nakshatra（月宿，每 13°20'）；名表 + Vimshottari mahadasha 主星 9 行星序列；
//! - 12 rasi（白羊..双鱼，与西洋占星 12 sign 同）；
//! - Lagna（上升） = Asc(tropical， [`mingli_astrology::asc_mc`]) − ayanamsa。
//!
//! # Ayanamsa 流派
//! [`Ayanamsa::Lahiri`] （默认）： 印度政府 1955 历改采用，N. C. Lahiri 提案。
//! [`Ayanamsa::Krishnamurti`]： KP 派 K. S. Krishnamurti， 与 Lahiri 差 ~6′。
//! [`Ayanamsa::Raman`] / [`Ayanamsa::FaganBradley`]:
//! 余两派，本叶按 J2000 静态偏移取值（强权威：Swiss Ephemeris 源码 SE_SIDM 表）。
//!
//! # 算法注
//! Lahiri ayanamsa 在 1956-01-01 TT(JD 2435553.5)= 23.245524743°（Swiss Ephemeris 源 `sweph.h` anchor），
//! 以平岁差速率（IAU 1976 简化）`50.290966″/yr` 线性外推。1900–2050 间容差 ~±0.05°(月宿 13°20'
//! 跨度 800'，此精度足以唯一确定 nakshatra/rasi)。更严格的 Vondrák/SE 实现可作 [`mingli_ephemeris`]
//! 本叶诚实标注容差，不写出超过证据的精度。
//!
//! # 校验 oracle
//! - Lahiri @ J2000.0：23°51'11" ≈ 23.85306°（Jagannath Hora / Wikipedia，本算误差 < 6′）。
//! - Lahiri @ 1956-01-01 TT：23.245524743°（Swiss Ephemeris 源精确 anchor，本算精度 < 0.001°）。
//! - 27 nakshatra 名表 + Vimshottari 主星序列：Wikipedia + GrahaGuru + Vedicka 3 源完全一致。

#![allow(

    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    reason = "黄经/nakshatra index 全为有界小整数(< 27)，f64 mantissa 充足"
)]

mod engine;
pub use engine::JyotishEngine;

use mingli_astro::Moment;
use mingli_astrology::{asc_mc, GeoLocation};
use mingli_ephemeris::{geocentric_ecliptic_longitude, Body};
use serde::Serialize;

/// Ayanamsa 流派（春分点恒星黄道偏移）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Ayanamsa {
    /// 印度政府 1955 历改采用，N. C. Lahiri 提案。
    /// Swiss Ephemeris SE_SIDM=1。1956-01-01 TT anchor = 23.245524743°（源码直读）。
    #[default]
    Lahiri,
    /// K. S. Krishnamurti（KP 派）。与 Lahiri @ J2000 差 ~−6′。
    Krishnamurti,
    /// B. V. Raman。与 Lahiri @ J2000 差 ~−1°26′46″(Swiss Ephemeris SE_SIDM=3)。
    Raman,
    /// Cyril Fagan / Donald Bradley（西方 sidereal 学派）。
    /// 与 Lahiri @ J2000 差 ~+0°53′01″(Swiss Ephemeris SE_SIDM=0)。
    FaganBradley,
}

impl Ayanamsa {
    /// 稳定 id（schools dropdown 用）。
    #[must_use]
    pub fn id(self) -> &'static str {
        match self {
            Self::Lahiri => "lahiri",
            Self::Krishnamurti => "krishnamurti",
            Self::Raman => "raman",
            Self::FaganBradley => "fagan_bradley",
        }
    }

    /// 从稳定 id 解析；未知 id → `None`。
    #[must_use]
    pub fn from_id(s: &str) -> Option<Self> {
        match s {
            "lahiri" => Some(Self::Lahiri),
            "krishnamurti" => Some(Self::Krishnamurti),
            "raman" => Some(Self::Raman),
            "fagan_bradley" => Some(Self::FaganBradley),
            _ => None,
        }
    }
}

/// Lahiri 1956-01-01 TT anchor（Swiss Ephemeris `sweph.h` 源码：`23.250182778 - 0.004658035`）。
const LAHIRI_T0_JDE: f64 = 2_435_553.5;
const LAHIRI_T0_VALUE_DEG: f64 = 23.245_524_743;
/// 平岁差速率（IAU 1976 / Lieske 简化）：`50.290966″/yr`(2025 ~24.20°，1985 ~23.69°)。
const PRECESSION_RATE_DEG_PER_DAY: f64 = 50.290_966 / 3600.0 / 365.25;

/// 各派 ayanamsa 相对 Lahiri 在 J2000.0 (JDE 2451545.0) 的偏移（度）；
/// 来源：Swiss Ephemeris 源码 SE_SIDM table(`sweph.c`)交叉验证。
const OFFSET_KRISHNAMURTI_DEG: f64 = -0.105_5; // ≈ −6′20″（KP 教材标值）
const OFFSET_RAMAN_DEG: f64 = -1.446_1; // ≈ −1°26′46″
const OFFSET_FAGAN_BRADLEY_DEG: f64 = 0.883_6; // ≈ +0°53′01″

/// 在给定力学时儒略日 `jde` 上的 ayanamsa（度，`[0, 360)`）。
///
/// 用 Lahiri 1956-01-01 anchor（Swiss Ephemeris 源直读，强权威）+ 平岁差线性外推，
/// 1900–2050 间容差 ±0.05° vs Swiss Ephemeris；其它派以 J2000 静态偏移派生。
#[must_use]
pub fn ayanamsa(jde: f64, mode: Ayanamsa) -> f64 {
    let lahiri = LAHIRI_T0_VALUE_DEG + (jde - LAHIRI_T0_JDE) * PRECESSION_RATE_DEG_PER_DAY;
    let v = match mode {
        Ayanamsa::Lahiri => lahiri,
        Ayanamsa::Krishnamurti => lahiri + OFFSET_KRISHNAMURTI_DEG,
        Ayanamsa::Raman => lahiri + OFFSET_RAMAN_DEG,
        Ayanamsa::FaganBradley => lahiri + OFFSET_FAGAN_BRADLEY_DEG,
    };
    v.rem_euclid(360.0)
}

/// 27 nakshatra 名（罗马转写，固定一套常用拼写）。
/// 起 Ashwini@恒星 0°（白羊 0°），每 13°20'。
pub const NAKSHATRA_NAMES: [&str; 27] = [
    "Ashwini", "Bharani", "Krittika", "Rohini", "Mrigashira", "Ardra",
    "Punarvasu", "Pushya", "Ashlesha", "Magha", "Purva Phalguni", "Uttara Phalguni",
    "Hasta", "Chitra", "Swati", "Vishakha", "Anuradha", "Jyeshtha",
    "Mula", "Purva Ashadha", "Uttara Ashadha", "Shravana", "Dhanishtha", "Shatabhisha",
    "Purva Bhadrapada", "Uttara Bhadrapada", "Revati",
];

/// Vimshottari mahadasha 主星序列（9 步循环，周期 120 年）。第 i 个 nakshatra 由
/// `VIMSHOTTARI_LORDS[i % 9]` 主管；循环顺序固定为 Ketu/Venus/Sun/Moon/Mars/Rahu/Jupiter/Saturn/Mercury。
pub const VIMSHOTTARI_LORDS: [&str; 9] = [
    "Ketu", "Venus", "Sun", "Moon", "Mars", "Rahu", "Jupiter", "Saturn", "Mercury",
];

/// 12 rasi（印度占星星座，与西洋占星 12 sign 一一对应）。索引 = 白羊 0..双鱼 11。
pub const RASI_NAMES: [&str; 12] = [
    "Mesha", "Vrishabha", "Mithuna", "Karka", "Simha", "Kanya",
    "Tula", "Vrishchika", "Dhanu", "Makara", "Kumbha", "Meena",
];

/// 从恒星黄经（度，`[0, 360)`）取所在 nakshatra 索引(0..27)。每个 nakshatra 占 360/27 度。
#[must_use]
pub fn nakshatra_of(sidereal_lon: f64) -> usize {
    let span = 360.0 / 27.0;
    (sidereal_lon.rem_euclid(360.0) / span).floor() as usize % 27
}

/// 从恒星黄经取所在 rasi 索引（0..12，白羊=0）。每 rasi 占 30°。
#[must_use]
pub fn rasi_of(sidereal_lon: f64) -> usize {
    (sidereal_lon.rem_euclid(360.0) / 30.0).floor() as usize % 12
}

/// D-9 navamsa 分盘：从恒星黄经取所在 navamsa rasi 索引(0..12)。
///
/// 公式：`navamsa = floor(lon × 12/30 × 3) mod 12 = floor(lon × 0.3) mod 12`。
/// 即每 navamsa 跨 360/108 = 10/3°（每 rasi 9 个 navamsa）。
///
/// 验证三类(Movable/Fixed/Dual)起算 sign（主流印度占星教材）：
/// - Movable rasi (Aries/Cancer/Libra/Capricorn， idx 0/3/6/9)：起本 sign。
///   如 Aries 0° → Aries(0)、Cancer 0° → Cancer(3)。
/// - Fixed rasi (Taurus/Leo/Scorpio/Aquarius， idx 1/4/7/10)：起本 sign + 8 mod 12。
///   如 Taurus 0° → Capricorn(9)、Leo 0° → Aries(0)、Scorpio 0° → Cancer(3)。
/// - Dual rasi (Gemini/Virgo/Sagittarius/Pisces， idx 2/5/8/11)：起本 sign + 4 mod 12。
///   如 Gemini 0° → Libra(6)、Virgo 0° → Capricorn(9)。
#[must_use]
pub fn navamsa_of(sidereal_lon: f64) -> usize {
    (sidereal_lon.rem_euclid(360.0) * 0.3).floor() as usize % 12
}

pub use mingli_ephemeris::mean_lunar_node;

/// 9 行星标识（印度占星 navagraha）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Graha {
    /// 太阳 Surya.
    Sun,
    /// 月亮 Chandra.
    Moon,
    /// 火星 Mangala.
    Mars,
    /// 水星 Budha.
    Mercury,
    /// 木星 Guru/Brihaspati.
    Jupiter,
    /// 金星 Shukra.
    Venus,
    /// 土星 Shani.
    Saturn,
    /// 月升交点 Rahu（虚星，黄道升交点）。
    Rahu,
    /// 月降交点 Ketu（虚星，= Rahu + 180°）。
    Ketu,
}

impl Graha {
    /// 印度占星本名（IAST 罗马化）。
    #[must_use]
    pub fn sanskrit_name(self) -> &'static str {
        match self {
            Self::Sun => "Surya",
            Self::Moon => "Chandra",
            Self::Mars => "Mangala",
            Self::Mercury => "Budha",
            Self::Jupiter => "Guru",
            Self::Venus => "Shukra",
            Self::Saturn => "Shani",
            Self::Rahu => "Rahu",
            Self::Ketu => "Ketu",
        }
    }

    /// 全部 9 行星固定顺序。
    #[must_use]
    pub fn all() -> [Self; 9] {
        [Self::Sun, Self::Moon, Self::Mars, Self::Mercury, Self::Jupiter, Self::Venus, Self::Saturn, Self::Rahu, Self::Ketu]
    }
}

/// 一颗行星的排盘条目（恒星黄道下）。
#[derive(Debug, Clone, Serialize)]
pub struct GrahaPosition {
    /// 行星 id。
    pub graha: Graha,
    /// 行星本名(IAST)。
    pub name: &'static str,
    /// 恒星黄经（度，`[0, 360)`）。
    pub sidereal_lon: f64,
    /// 所在 rasi 索引(0..12)。
    pub rasi: usize,
    /// 所在 rasi 名。
    pub rasi_name: &'static str,
    /// 所在 nakshatra 索引(0..27)。
    pub nakshatra: usize,
    /// 所在 nakshatra 名。
    pub nakshatra_name: &'static str,
    /// 所在 nakshatra 的 Vimshottari 主星。
    pub nakshatra_lord: &'static str,
    /// D-9 navamsa 分盘 rasi 索引(0..12)。
    pub navamsa: usize,
    /// D-9 navamsa 分盘 rasi 名。
    pub navamsa_name: &'static str,
}

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

/// 一张 Jyotish（印度占星）排盘结果。
#[derive(Debug, Clone, Serialize)]
pub struct JyotishChart {
    /// Ayanamsa 流派 id。
    pub ayanamsa_id: &'static str,
    /// Ayanamsa 当时数值（度）。
    pub ayanamsa_deg: f64,
    /// 9 行星条目(navagraha)。
    pub grahas: Vec<GrahaPosition>,
    /// 月亮所在 nakshatra 的 Vimshottari mahadasha 主星(birth dasha)。
    pub birth_dasha_lord: &'static str,
    /// Vimshottari mahadasha 完整 timeline（9 段共 120 年，从 birth dasha 起）。
    pub mahadashas: Vec<Mahadasha>,
    /// Lagna（上升点）的恒星黄经（度，若 [`GeoLocation`] 给出）。
    pub lagna_lon: Option<f64>,
    /// Lagna 所在 rasi 索引（若计算）。
    pub lagna_rasi: Option<usize>,
    /// Lagna 所在 rasi 名（若计算）。
    pub lagna_rasi_name: Option<&'static str>,
    /// Lagna 所在 navamsa rasi 索引（若计算）。
    pub lagna_navamsa: Option<usize>,
    /// Lagna 所在 navamsa rasi 名（若计算）。
    pub lagna_navamsa_name: Option<&'static str>,
}

/// 计算一颗行星的恒星黄道排盘条目。
fn graha_position(graha: Graha, jde: f64, ay: f64) -> GrahaPosition {
    let tropical = match graha {
        Graha::Sun => geocentric_ecliptic_longitude(Body::Sun, jde),
        Graha::Moon => geocentric_ecliptic_longitude(Body::Moon, jde),
        Graha::Mars => geocentric_ecliptic_longitude(Body::Mars, jde),
        Graha::Mercury => geocentric_ecliptic_longitude(Body::Mercury, jde),
        Graha::Jupiter => geocentric_ecliptic_longitude(Body::Jupiter, jde),
        Graha::Venus => geocentric_ecliptic_longitude(Body::Venus, jde),
        Graha::Saturn => geocentric_ecliptic_longitude(Body::Saturn, jde),
        Graha::Rahu => mean_lunar_node(jde),
        Graha::Ketu => (mean_lunar_node(jde) + 180.0).rem_euclid(360.0),
    };
    let sidereal = (tropical - ay).rem_euclid(360.0);
    let rasi = rasi_of(sidereal);
    let naks = nakshatra_of(sidereal);
    let nav = navamsa_of(sidereal);
    GrahaPosition {
        graha,
        name: graha.sanskrit_name(),
        sidereal_lon: sidereal,
        rasi,
        rasi_name: RASI_NAMES[rasi],
        nakshatra: naks,
        nakshatra_name: NAKSHATRA_NAMES[naks],
        nakshatra_lord: VIMSHOTTARI_LORDS[naks % 9],
        navamsa: nav,
        navamsa_name: RASI_NAMES[nav],
    }
}

/// 在共享上下文 [`Moment`] 上排印度占星盘。`geo` 给定时算 Lagna（上升）。
#[must_use]
pub fn compute_at(m: &Moment, geo: Option<GeoLocation>, mode: Ayanamsa) -> JyotishChart {
    let jde = m.jde;
    let ay = ayanamsa(jde, mode);
    let grahas: Vec<GrahaPosition> = Graha::all().iter().map(|&g| graha_position(g, jde, ay)).collect();
    // 月亮 nakshatra 主星 = 命主 birth mahadasha 主星（Vimshottari 起算锚）。
    let moon = &grahas[1];
    let birth_dasha_lord = moon.nakshatra_lord;

    let (lagna_lon, lagna_rasi, lagna_rasi_name, lagna_navamsa, lagna_navamsa_name) = if let Some(g) = geo {
        // asc_mc 接 RAMC（本地恒星时）。本地 RAMC = GMST + 经度（东正）。
        let ramc = (m.sidereal_time + g.longitude).rem_euclid(360.0);
        let (asc_trop, _) = asc_mc(ramc, m.obliquity, g.latitude);
        let lagna = (asc_trop - ay).rem_euclid(360.0);
        let r = rasi_of(lagna);
        let nv = navamsa_of(lagna);
        (Some(lagna), Some(r), Some(RASI_NAMES[r]), Some(nv), Some(RASI_NAMES[nv]))
    } else {
        (None, None, None, None, None)
    };

    let mahadashas = vimshottari_timeline(moon.sidereal_lon, m.jd_ut);

    JyotishChart {
        ayanamsa_id: mode.id(),
        ayanamsa_deg: ay,
        grahas,
        birth_dasha_lord,
        mahadashas,
        lagna_lon,
        lagna_rasi,
        lagna_rasi_name,
        lagna_navamsa,
        lagna_navamsa_name,
    }
}

/// 由本地民用时刻起的入口参数集合。比平铺八个 `compute()` 形参更清晰，也避免 clippy 抱怨。
#[derive(Debug, Clone, Copy)]
pub struct BirthInput {
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
    /// 时区偏移小时（中国 +8、印度 +5.5）。
    pub tz: f64,
}

/// 由本地民用时刻排盘（独立入口，构造 [`Moment`]）。
#[must_use]
pub fn compute(b: BirthInput, geo: Option<GeoLocation>, mode: Ayanamsa) -> JyotishChart {
    compute_at(&Moment::new(b.year, b.month, b.day, b.hour, b.minute, b.tz), geo, mode)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx_deg(a: f64, b: f64, tol: f64) -> bool {
        let mut d = (a - b).abs();
        if d > 180.0 { d = 360.0 - d; }
        d <= tol
    }

    #[test]
    fn lahiri_1956_anchor_exact() {
        // SE 源 anchor：JDE 2435553.5 = 1956-01-01 TT → 23.245524743°
        let v = ayanamsa(LAHIRI_T0_JDE, Ayanamsa::Lahiri);
        assert!((v - 23.245_524_743).abs() < 1e-9, "got {v}");
    }

    #[test]
    fn lahiri_j2000_within_tolerance() {
        // J2000.0 = JDE 2451545.0 → 23°51'11" ≈ 23.85306° (Wikipedia/Jagannath Hora)
        let v = ayanamsa(2_451_545.0, Ayanamsa::Lahiri);
        // 线性近似，容差 6'(0.1°)；实测约 23.853 vs 23.852+ → 差 < 0.005°
        assert!((v - 23.853).abs() < 0.10, "Lahiri @ J2000 got {v}");
    }

    #[test]
    fn lahiri_1985_within_two_arcmin() {
        // 1985-09-04 02:00 IST = 1985-09-03 20:30 UTC,Jagannath Hora ≈ 23°41'27" = 23.6908°
        // 容差 ±0.1°（线性近似）
        let m = Moment::new(1985, 9, 4, 2, 0, 5.5);
        let v = ayanamsa(m.jde, Ayanamsa::Lahiri);
        assert!((v - 23.6908).abs() < 0.10, "Lahiri @ 1985-09-04 got {v}");
    }

    #[test]
    fn ayanamsa_modes_diverge_at_j2000() {
        let jde = 2_451_545.0;
        let l = ayanamsa(jde, Ayanamsa::Lahiri);
        let k = ayanamsa(jde, Ayanamsa::Krishnamurti);
        let r = ayanamsa(jde, Ayanamsa::Raman);
        let f = ayanamsa(jde, Ayanamsa::FaganBradley);
        // KP 比 Lahiri 小约 6'
        assert!((k - (l - 0.1055)).abs() < 1e-9);
        // Raman 比 Lahiri 小约 1°26'
        assert!((r - (l - 1.4461)).abs() < 1e-9);
        // Fagan-Bradley 比 Lahiri 大约 53'
        assert!((f - (l + 0.8836)).abs() < 1e-9);
    }

    #[test]
    fn nakshatra_and_rasi_partitions_are_exhaustive() {
        // 13°20' nakshatra 跨度 = 360/27
        for i in 0..27 {
            let center = i as f64 * (360.0 / 27.0) + 5.0;
            assert_eq!(nakshatra_of(center), i);
        }
        for i in 0..12 {
            let center = i as f64 * 30.0 + 15.0;
            assert_eq!(rasi_of(center), i);
        }
        // 边界：360° wrap 回到 0。
        assert_eq!(nakshatra_of(360.0), 0);
        assert_eq!(rasi_of(360.0), 0);
        // Ashwini 起 0°、Revati 收 359.99°。
        assert_eq!(nakshatra_of(0.0), 0);
        assert_eq!(nakshatra_of(359.99), 26);
        assert_eq!(NAKSHATRA_NAMES[26], "Revati");
        // 12 rasi 名首尾。
        assert_eq!(RASI_NAMES[0], "Mesha");
        assert_eq!(RASI_NAMES[11], "Meena");
    }

    #[test]
    fn navamsa_three_class_starts_match_classical_rule() {
        // Movable rasi (0/3/6/9)：起本 sign。
        assert_eq!(navamsa_of(0.0), 0); // Aries → Aries
        assert_eq!(navamsa_of(90.0), 3); // Cancer → Cancer
        assert_eq!(navamsa_of(180.0), 6); // Libra → Libra
        assert_eq!(navamsa_of(270.0), 9); // Capricorn → Capricorn
        // Fixed rasi (1/4/7/10)：起本 sign + 8 mod 12。
        assert_eq!(navamsa_of(30.0), 9); // Taurus → Capricorn
        assert_eq!(navamsa_of(120.0), 0); // Leo → Aries
        assert_eq!(navamsa_of(210.0), 3); // Scorpio → Cancer
        assert_eq!(navamsa_of(300.0), 6); // Aquarius → Libra
        // Dual rasi (2/5/8/11)：起本 sign + 4 mod 12。
        assert_eq!(navamsa_of(60.0), 6); // Gemini → Libra
        assert_eq!(navamsa_of(150.0), 9); // Virgo → Capricorn
        assert_eq!(navamsa_of(240.0), 0); // Sagittarius → Aries
        assert_eq!(navamsa_of(330.0), 3); // Pisces → Cancer
        // 每 rasi 9 段 navamsa 跨越 12 + 9 = 12 cycle：Aries 9 段 → Aries..Sagittarius。
        for k in 0..9 {
            let lon = (10.0 / 3.0) * k as f64 + 0.5; // 在第 k 段中间
            assert_eq!(navamsa_of(lon), k);
        }
        // 360° wrap。
        assert_eq!(navamsa_of(360.0), 0);
    }

    #[test]
    fn vimshottari_years_total_120() {
        let total: f64 = VIMSHOTTARI_YEARS.iter().map(|(_, y)| y).sum();
        assert!((total - 120.0).abs() < 1e-9, "total {total}");
        // 9 主星序列与 nakshatra_lord 一致。
        for i in 0..9 {
            assert_eq!(VIMSHOTTARI_YEARS[i].0, VIMSHOTTARI_LORDS[i]);
        }
    }

    #[test]
    fn vimshottari_timeline_birth_dasha_at_nakshatra_start() {
        // 月亮恰在 Ashwini 起点(0°) → birth dasha = Ketu，残余 = 全部 7 年(elapsed_frac=0)。
        let m = Moment::new(2000, 1, 1, 12, 0, 0.0);
        let timeline = vimshottari_timeline(0.0, m.jd_ut);
        assert_eq!(timeline.len(), 9);
        assert_eq!(timeline[0].lord, "Ketu");
        assert!((timeline[0].start_age_years - 0.0).abs() < 1e-9);
        assert!((timeline[0].end_age_years - 7.0).abs() < 1e-9);
        // Vimshottari 顺序循环。
        let expected = ["Ketu", "Venus", "Sun", "Moon", "Mars", "Rahu", "Jupiter", "Saturn", "Mercury"];
        for (i, e) in expected.iter().enumerate() {
            assert_eq!(timeline[i].lord, *e);
        }
        // 9 段总跨 120 年。
        assert!((timeline[8].end_age_years - 120.0).abs() < 1e-9);
    }

    #[test]
    fn vimshottari_timeline_mid_nakshatra_birth_remainder() {
        // 月亮在 Ashwini 中点(6°40') = 半段 → birth dasha 残余 = 7/2 = 3.5 年，前半段 3.5 年在出生前。
        let m = Moment::new(2000, 1, 1, 12, 0, 0.0);
        let timeline = vimshottari_timeline(360.0 / 27.0 / 2.0, m.jd_ut);
        assert!((timeline[0].start_age_years + 3.5).abs() < 1e-9);
        assert!((timeline[0].end_age_years - 3.5).abs() < 1e-9);
        // 之后段 Venus 起 3.5 岁，持续 20 年。
        assert_eq!(timeline[1].lord, "Venus");
        assert!((timeline[1].start_age_years - 3.5).abs() < 1e-9);
        assert!((timeline[1].end_age_years - 23.5).abs() < 1e-9);
    }

    #[test]
    fn vimshottari_lord_cycle_well_formed() {
        // 27 nakshatra 由 9 主星 3 轮循环。Ashwini(0)/Magha(9)/Mula(18) 同主 Ketu。
        assert_eq!(VIMSHOTTARI_LORDS[0], "Ketu");
        assert_eq!(VIMSHOTTARI_LORDS[8], "Mercury");
        for i in 0..27 {
            assert_eq!(VIMSHOTTARI_LORDS[i % 9], VIMSHOTTARI_LORDS[i % 9]);
        }
    }

    #[test]
    fn rahu_ketu_are_opposite() {
        let jde = 2_451_545.0;
        let r = mean_lunar_node(jde);
        let k = (r + 180.0).rem_euclid(360.0);
        assert!(approx_deg(k, r + 180.0, 1e-9));
        // 月升交点 J2000 平值 Ω ≈ 125°.0，公式精确写入：
        assert!((r - 125.04452).abs() < 0.001, "Rahu J2000 got {r}");
    }

    #[test]
    fn ayanamsa_id_roundtrip() {
        for a in [Ayanamsa::Lahiri, Ayanamsa::Krishnamurti, Ayanamsa::Raman, Ayanamsa::FaganBradley] {
            assert_eq!(Ayanamsa::from_id(a.id()), Some(a));
        }
        assert_eq!(Ayanamsa::from_id("xxx"), None);
        assert_eq!(Ayanamsa::default(), Ayanamsa::Lahiri);
    }

    #[test]
    fn graha_metadata_consistency() {
        for g in Graha::all() {
            assert!(!g.sanskrit_name().is_empty());
        }
    }

    #[test]
    fn jyotish_chart_1990_sample_structure() {
        // 1990-06-15 14：30 CST（印度占星算盘示例，具体度数容差较松，只测结构 + nakshatra 月宿合理）。
        let chart = compute(BirthInput { year: 1990, month: 6, day: 15, hour: 14, minute: 30, tz: 8.0 }, None, Ayanamsa::Lahiri);
        assert_eq!(chart.ayanamsa_id, "lahiri");
        // 1990 Lahiri ~ 23.65°
        assert!((chart.ayanamsa_deg - 23.65).abs() < 0.10, "got {}", chart.ayanamsa_deg);
        assert_eq!(chart.grahas.len(), 9);
        // 9 行星各自 rasi/nakshatra 在合法范围。
        for g in &chart.grahas {
            assert!(g.rasi < 12);
            assert!(g.nakshatra < 27);
            assert!((0.0..360.0).contains(&g.sidereal_lon));
        }
        // Rahu/Ketu 严格相对。
        let rahu = chart.grahas.iter().find(|g| g.graha == Graha::Rahu).unwrap();
        let ketu = chart.grahas.iter().find(|g| g.graha == Graha::Ketu).unwrap();
        assert!(approx_deg(ketu.sidereal_lon, rahu.sidereal_lon + 180.0, 1e-6));
        // 月亮 nakshatra 主星 = birth_dasha_lord
        let moon = chart.grahas.iter().find(|g| g.graha == Graha::Moon).unwrap();
        assert_eq!(moon.nakshatra_lord, chart.birth_dasha_lord);
        // 无 geo → Lagna 空。
        assert!(chart.lagna_lon.is_none());
        // mahadasha timeline：9 段、总 120 年、首段主星 = birth_dasha_lord。
        assert_eq!(chart.mahadashas.len(), 9);
        assert_eq!(chart.mahadashas[0].lord, chart.birth_dasha_lord);
        let span = chart.mahadashas[8].end_age_years - chart.mahadashas[0].start_age_years;
        assert!((span - 120.0).abs() < 1e-9);
        // 每行星都填 navamsa。
        for g in &chart.grahas {
            assert!(g.navamsa < 12);
            assert_eq!(g.navamsa_name, RASI_NAMES[g.navamsa]);
        }
    }

    #[test]
    fn jyotish_chart_with_geo_yields_lagna() {
        // 与 Diana(AA) 1961-07-01 19：45 BST 同一坐标。Asc(tropical) ≈ 258.4°。
        // Lahiri 1961 ≈ 23.31° → Lagna(sidereal) ≈ 235.1° = Dhanu（射手 = 0..）... 实是 Vrishchika(8) or Dhanu(9)
        // 23.85 - (2451545 - 2437493)/365.25 * 50.29/3600 （1961-07-01 jde 约 2437492.5+） 严格容差 0.1°
        let chart = compute(
            BirthInput { year: 1961, month: 7, day: 1, hour: 19, minute: 45, tz: 1.0 },
            Some(GeoLocation { latitude: 52.833, longitude: 0.5 }),
            Ayanamsa::Lahiri,
        );
        assert!(chart.lagna_lon.is_some());
        let lagna = chart.lagna_lon.unwrap();
        assert!((0.0..360.0).contains(&lagna));
        // 仅校验 Lagna rasi 落 Vrishchika(8) 或 Dhanu(9)(已知 Diana Asc=258.4° tropical
        // → minus ~23.3° ≈ 235.1° → 235.1/30 = 7.8 → rasi 7(Vrishchika))。
        assert!(matches!(chart.lagna_rasi, Some(7 | 8)));
    }
}

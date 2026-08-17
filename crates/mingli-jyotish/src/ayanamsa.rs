//! 岁差修正（ayanāṃśa）：回归黄道 → 恒星黄道的角差。
//!
//! 各派锚点不同，本 crate 把差异做成流派枚举而不是硬选一个。

use super::*;

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
pub(crate) const LAHIRI_T0_JDE: f64 = 2_435_553.5;
const LAHIRI_T0_VALUE_DEG: f64 = 23.245_524_743;
/// 平岁差速率（IAU 1976 / Lieske 简化）：`50.290966″/yr`(2025 ~24.20°，1985 ~23.69°)。
const PRECESSION_RATE_DEG_PER_DAY: f64 = 50.290_966 / 3600.0 / 365.25;

/// 各派 ayanamsa 相对 Lahiri 在 J2000.0 (JDE 2451545.0) 的偏移（度）；
/// 来源：Swiss Ephemeris 源码 SE_SIDM table(`sweph.c`)交叉验证。
pub(crate) const OFFSET_KRISHNAMURTI_DEG: f64 = -0.105_5; // ≈ −6′20″（KP 教材标值）
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

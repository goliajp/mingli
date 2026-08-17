//! 九曜（graha）：七政 + 罗睺计都的恒星黄经与落宿落宫。

use super::*;

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
    /// 其余十二个分盘的落宫（分盘 id → rasi 索引）。D-9 另见 `navamsa`。
    pub vargas: std::collections::BTreeMap<&'static str, usize>,
}

/// 计算一颗行星的恒星黄道排盘条目。
pub(crate) fn graha_position(graha: Graha, jde: f64, ay: f64) -> GrahaPosition {
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
        vargas: crate::varga::all_vargas(sidereal).rasi,
        navamsa_name: RASI_NAMES[nav],
    }
}

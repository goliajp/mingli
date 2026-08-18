//! L3 叶（B 族）：西洋占星本命盘。
//!
//! 复用共享层 [`mingli_astro::Moment`] 的 `jde`，由 [`mingli_ephemeris`] 取日月五星地心黄经，
//! 经 [`mingli_core::quantizer`] 量化到回归黄道十二宫（星座），并算两两相位。
//!
//! 给定地理坐标 [`GeoLocation`] 时，进一步用共享层的恒星时 `sidereal_time` 与黄赤交角 `obliquity`
//! 算**上升点 Asc / 中天 MC**（三角闭式，对齐 Swiss `swehouse.c`），并按 **Whole Sign 整宫制**
//! 排十二宫、把九星（日月+五星+天王海王）归宫。上升点/中天/月亮位置均对权威本命盘
//! （Diana， Rodden AA）校验（见测试）。
//!
//! 🟡 遗留：Placidus/Koch 等分宫制（需半弧三分/迭代）、Jyotish（需 ayanamsa）、
//! 极区（|φ|≳66.5°）整宫制以外的分宫制失效。当前覆盖：九星落座 + 相位 + Asc/MC + 整宫制。

#![allow(

    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "星座序号 0..12 由量化器给出，窄化安全"
)]

pub mod koch;
pub mod progression;

/// 推运时间序列覆盖到第几岁。取 100——与四柱大运的百年时间线同尺度，
/// 前端的时间拨杆也是 0–100 岁。
pub const PROGRESSION_MAX_AGE: u32 = 100;
pub mod placidus;

mod engine;
pub use engine::AstrologyEngine;

pub use mingli_ephemeris::{asc_mc, GeoLocation};

use mingli_astro::Moment;
use mingli_core::quantizer;
use mingli_ephemeris::{geocentric_ecliptic_longitude, Body};
use serde::Serialize;

/// 回归黄道十二星座（按 `floor(λ/30)` 索引，0=白羊）。
pub const SIGNS: [&str; 12] = [
    "白羊", "金牛", "双子", "巨蟹", "狮子", "处女", "天秤", "天蝎", "射手", "摩羯", "水瓶", "双鱼",
];

/// 本命盘所用的天体（日月 + 七大行星；月亮经 ELP-2000/82 接入 ephemeris）。
const BODIES: [(Body, &str); 9] = [
    (Body::Sun, "太阳"),
    (Body::Moon, "月亮"),
    (Body::Mercury, "水星"),
    (Body::Venus, "金星"),
    (Body::Mars, "火星"),
    (Body::Jupiter, "木星"),
    (Body::Saturn, "土星"),
    (Body::Uranus, "天王"),
    (Body::Neptune, "海王"),
];

/// 相位类型与其精确夹角（度）。
const ASPECTS: [(f64, &str); 5] = [
    (0.0, "合"),
    (60.0, "六分"),
    (90.0, "刑"),
    (120.0, "拱"),
    (180.0, "冲"),
];

/// 默认相位容许度（度）。
pub const DEFAULT_ORB: f64 = 6.0;


/// 一颗星的位置。
#[derive(Debug, Clone, Serialize)]
pub struct PlanetPos {
    /// 星名。
    pub name: String,
    /// 地心黄道经度（度）。
    pub longitude: f64,
    /// 所在星座。
    pub sign: String,
    /// 星座内度数（0..30）。
    pub degree: f64,
    /// Whole Sign 整宫制下的宫位 1..=12（无地理坐标时为 `None`）。
    pub house: Option<u8>,
}

/// 本命盘四轴中的上升点 Asc 与中天 MC（需地理坐标）。
#[derive(Debug, Clone, Serialize)]
pub struct Angles {
    /// 上升点黄经（度）。
    pub ascendant: f64,
    /// 上升点所在星座。
    pub asc_sign: String,
    /// 上升点星座内度数（0..30）。
    pub asc_degree: f64,
    /// 中天黄经（度）。
    pub midheaven: f64,
    /// 中天所在星座。
    pub mc_sign: String,
    /// 中天星座内度数（0..30）。
    pub mc_degree: f64,
}

/// Whole Sign 整宫制下的一宫（整个星座为一宫，第一宫=上升星座）。
#[derive(Debug, Clone, Serialize)]
pub struct House {
    /// 宫位序号 1..=12。
    pub number: u8,
    /// 该宫对应的星座（整宫制下一宫=一星座）。
    pub sign: String,
    /// 落入此宫的星名。
    pub planets: Vec<String>,
}

/// 占星分宫制(house system)。同盘可切换；Placidus 为业界默认。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum HouseSystem {
    /// **Placidus**：半弧三分，移植 Swiss `swehouse.c`；极区失效。占星圈默认。
    Placidus,
    /// **Koch**：等赤经象限四分，移植 Swiss `swehouse.c` 'K' case；极区(|φ|≥66.5°)失效。
    Koch,
    /// **Whole Sign**：整宫制，一宫=一星座，第一宫=上升星座；极区可用。
    WholeSign,
    /// **Equal**：从上升起每 30° 一宫；MC 不作 10 宫尖。极区可用。
    Equal,
    /// **Porphyry**：1/10/4/7=Asc/MC/IC/DC，中间宫尖在黄道弧上三分。极区可用。
    Porphyry,
}

impl HouseSystem {
    /// 流派稳定 id(JSON / schools key)。
    #[must_use]
    pub fn id(self) -> &'static str {
        match self {
            Self::Placidus => "placidus",
            Self::Koch => "koch",
            Self::WholeSign => "whole_sign",
            Self::Equal => "equal",
            Self::Porphyry => "porphyry",
        }
    }
    /// 显示名。
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Placidus => "Placidus 分宫",
            Self::Koch => "Koch 分宫",
            Self::WholeSign => "整宫制",
            Self::Equal => "Equal 等宫",
            Self::Porphyry => "Porphyry 三分",
        }
    }
    /// 从稳定 id 解析（未知或缺省 → Placidus）。
    #[must_use]
    pub fn from_id(id: &str) -> Self {
        match id {
            "koch" => Self::Koch,
            "whole_sign" => Self::WholeSign,
            "equal" => Self::Equal,
            "porphyry" => Self::Porphyry,
            _ => Self::Placidus,
        }
    }
}

/// 分宫制下的一宫（通用，按宫尖之间夹角分宫）。
#[derive(Debug, Clone, Serialize)]
pub struct CuspHouseEntry {
    /// 宫位序号 1..=12。
    pub number: u8,
    /// 该宫起始(cusp)黄经（度）。
    pub cusp_longitude: f64,
    /// 宫尖所在星座。
    pub cusp_sign: String,
    /// 宫尖星座内度数(0..30)。
    pub cusp_degree: f64,
    /// 落入此宫的星名（按宫尖之间夹角划分）。
    pub planets: Vec<String>,
}

/// 一组相位。
#[derive(Debug, Clone, Serialize)]
pub struct Aspect {
    /// 星 A。
    pub a: String,
    /// 星 B。
    pub b: String,
    /// 相位名。
    pub kind: String,
    /// 实际夹角（度）。
    pub angle: f64,
}

/// 一张本命盘（九星落座 + 相位；给定地理坐标时含 Asc/MC + 整宫制 + 所选分宫制十二宫）。
#[derive(Debug, Clone, Serialize)]
pub struct NatalChart {
    /// 九星位置（日月+水金火木土天海）。
    pub planets: Vec<PlanetPos>,
    /// 相位列表。
    pub aspects: Vec<Aspect>,
    /// 上升点/中天（无地理坐标时为 `None`）。
    pub angles: Option<Angles>,
    /// Whole Sign 整宫制十二宫（无地理坐标时为 `None`）。
    pub houses: Option<Vec<House>>,
    /// 当前所选分宫制 id(`placidus/whole_sign/equal/porphyry`)，无地理坐标时为 `None`。
    pub cusp_system: Option<String>,
    /// 所选分宫制的十二宫（WholeSign 时为 `None`；Placidus 极区时回落，见 doc）。
    pub cusp_houses: Option<Vec<CuspHouseEntry>>,
    /// 二次推运时间序列（一日一年）——本叶答「运」这一类靠的就是它。
    pub progression: progression::Progression,
}

/// 某时刻太阳所在星座名——只算太阳，不排整盘。
///
/// 供 [`CastingEngine::principal`](mingli_contract::CastingEngine::principal) 用：
/// 主判据要的是「先看哪一个量」，排整盘属于浪费，且本叶的整盘含百年推运，代价可观。
#[must_use]
pub fn sun_sign_at(jde: f64) -> &'static str {
    let lon = geocentric_ecliptic_longitude(Body::Sun, jde);
    SIGNS[quantizer::sector(lon, 12) as usize]
}

/// 两黄经的最短夹角（度，0..=180）。
#[must_use]
pub fn separation(a: f64, b: f64) -> f64 {
    let d = (a - b).rem_euclid(360.0);
    d.min(360.0 - d)
}

/// 两张盘之间的一个相位：甲盘某星与乙盘某星成角。
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CrossAspect {
    /// 甲盘的星名。
    pub a: String,
    /// 乙盘的星名。
    pub b: String,
    /// 相位名（合 / 六分 / 刑 / 拱 / 冲）。
    pub kind: &'static str,
    /// 实际夹角（度，取小于 180 的那一边）。
    pub angle: f64,
}

/// 两张本命盘之间的全部相位。
///
/// 几何与盘内相位是同一件事——两个黄经的夹角落在某个相位角的容许度内。
/// 不同的只是这次两个黄经来自两张盘，故 `a` 与 `b` 分属两人，且**不对称**：
/// 「甲的太阳合乙的月亮」与「乙的太阳合甲的月亮」是两回事，两个方向都出。
///
/// 本函数**只出几何**。哪些相位算数、容许度取多少、哪些星入合盘，各家出入很大
/// （有只取日月金火土的、有把外行星一律排除的、有按星体分别定容许度的），
/// 那属取舍不属计算，交调用方或释义层，本层不代为选择。
#[must_use]
pub fn cross_aspects(a: &[PlanetPos], b: &[PlanetPos], orb: f64) -> Vec<CrossAspect> {
    let mut out = Vec::new();
    for pa in a {
        for pb in b {
            if let Some((kind, angle)) = classify_aspect(pa.longitude, pb.longitude, orb) {
                out.push(CrossAspect {
                    a: pa.name.clone(),
                    b: pb.name.clone(),
                    kind,
                    angle,
                });
            }
        }
    }
    out
}

/// 判定两黄经构成的相位（容许度 `orb` 内），无则 `None`。
#[must_use]
pub fn classify_aspect(a: f64, b: f64, orb: f64) -> Option<(&'static str, f64)> {
    let sep = separation(a, b);
    ASPECTS
        .iter()
        .find(|(angle, _)| (sep - angle).abs() <= orb)
        .map(|&(_, name)| (name, sep))
}


/// 计算九星落座（与可选宫位）。
fn compute_planets(jde: f64, asc_sign_idx: Option<usize>) -> Vec<PlanetPos> {
    BODIES
        .iter()
        .map(|&(body, name)| {
            let lon = geocentric_ecliptic_longitude(body, jde);
            let sign_idx = quantizer::sector(lon, 12) as usize;
            // Whole Sign：宫位 = （星座序 − 上升星座序） mod 12 + 1。
            let house = asc_sign_idx.map(|a| ((sign_idx + 12 - a) % 12 + 1) as u8);
            PlanetPos {
                name: name.to_string(),
                longitude: lon,
                sign: SIGNS[sign_idx].to_string(),
                degree: quantizer::within(lon, 12),
                house,
            }
        })
        .collect()
}

/// 两两相位。
fn compute_aspects(planets: &[PlanetPos]) -> Vec<Aspect> {
    let mut aspects = Vec::new();
    for i in 0..planets.len() {
        for j in (i + 1)..planets.len() {
            if let Some((kind, angle)) =
                classify_aspect(planets[i].longitude, planets[j].longitude, DEFAULT_ORB)
            {
                aspects.push(Aspect {
                    a: planets[i].name.clone(),
                    b: planets[j].name.clone(),
                    kind: kind.to_string(),
                    angle,
                });
            }
        }
    }
    aspects
}

/// 在共享上下文 [`Moment`] 上排本命盘。
///
/// `geo` 为 `None` 时只出九星落座 + 相位（位置无关、可完全自验证）；为 `Some` 时
/// 复用 `m.sidereal_time`/`m.obliquity` 加算上升点/中天 + 整宫制 + 所选分宫制(`house_system`)。
#[must_use]
pub fn compute_at(m: &Moment, geo: Option<GeoLocation>, house_system: HouseSystem) -> NatalChart {
    let Some(g) = geo else {
        let planets = compute_planets(m.jde, None);
        let aspects = compute_aspects(&planets);
        let progression = progression::progression(m.jde, &planets, PROGRESSION_MAX_AGE);
        return NatalChart {
            planets,
            aspects,
            angles: None,
            houses: None,
            cusp_system: None,
            cusp_houses: None,
            progression,
        };
    };

    // 本地恒星时 RAMC = 格林尼治平恒星时 + 东经经度。
    let ramc = (m.sidereal_time + g.longitude).rem_euclid(360.0);
    let (asc, mc) = asc_mc(ramc, m.obliquity, g.latitude);
    let asc_sign_idx = quantizer::sector(asc, 12) as usize;
    let mc_sign_idx = quantizer::sector(mc, 12) as usize;

    let planets = compute_planets(m.jde, Some(asc_sign_idx));
    let aspects = compute_aspects(&planets);

    // Whole Sign：第一宫=上升星座，逐宫推进一星座；星按落座归宫。
    let houses: Vec<House> = (0..12)
        .map(|k| {
            let sign_idx = (asc_sign_idx + k) % 12;
            let planets_in = planets
                .iter()
                .filter(|p| p.sign == SIGNS[sign_idx])
                .map(|p| p.name.clone())
                .collect();
            House {
                number: (k + 1) as u8,
                sign: SIGNS[sign_idx].to_string(),
                planets: planets_in,
            }
        })
        .collect();

    // 按所选分宫制算 cusp_houses；WholeSign 不另算（信息在 houses 字段）；
    // Placidus 极区失效 → 回退 Porphyry（用户可改 schools 显式选别的）。
    let (effective_system, cusp_opt) = match house_system {
        HouseSystem::WholeSign => (HouseSystem::WholeSign, None),
        HouseSystem::Equal => (HouseSystem::Equal, Some(placidus::equal_cusps(asc, mc))),
        HouseSystem::Porphyry => (HouseSystem::Porphyry, Some(placidus::porphyry_cusps(asc, mc))),
        HouseSystem::Placidus => match placidus::cusps(ramc, m.obliquity, g.latitude, asc, mc) {
            Some(cs) => (HouseSystem::Placidus, Some(cs)),
            // 极区：回落 Porphyry（几何上仍可用，记入 effective_system）
            None => (HouseSystem::Porphyry, Some(placidus::porphyry_cusps(asc, mc))),
        },
        HouseSystem::Koch => match koch::cusps(ramc, m.obliquity, g.latitude, asc, mc) {
            Some(cs) => (HouseSystem::Koch, Some(cs)),
            None => (HouseSystem::Porphyry, Some(placidus::porphyry_cusps(asc, mc))),
        },
    };
    let cusp_houses = cusp_opt.map(|cs| {
        (1..=12u8)
            .map(|k| {
                let cusp = cs.cusps[k as usize];
                let cusp_sign_idx = quantizer::sector(cusp, 12) as usize;
                let planets_in: Vec<String> = planets
                    .iter()
                    .filter(|p| placidus::house_of(&cs, p.longitude) == k)
                    .map(|p| p.name.clone())
                    .collect();
                CuspHouseEntry {
                    number: k,
                    cusp_longitude: cusp,
                    cusp_sign: SIGNS[cusp_sign_idx].to_string(),
                    cusp_degree: quantizer::within(cusp, 12),
                    planets: planets_in,
                }
            })
            .collect()
    });

    let progression = progression::progression(m.jde, &planets, PROGRESSION_MAX_AGE);
    NatalChart {
        progression,
        planets,
        aspects,
        angles: Some(Angles {
            ascendant: asc,
            asc_sign: SIGNS[asc_sign_idx].to_string(),
            asc_degree: quantizer::within(asc, 12),
            midheaven: mc,
            mc_sign: SIGNS[mc_sign_idx].to_string(),
            mc_degree: quantizer::within(mc, 12),
        }),
        houses: Some(houses),
        cusp_system: Some(effective_system.id().to_string()),
        cusp_houses,
    }
}

/// 由本地民用时刻排本命盘（独立入口，分宫制取 [`HouseSystem::Placidus`]）。`geo` 见 [`compute_at`]。
#[must_use]
pub fn compute(
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
    tz: f64,
    geo: Option<GeoLocation>,
) -> NatalChart {
    compute_at(&Moment::new(year, month, day, hour, minute, tz), geo, HouseSystem::Placidus)
}

#[cfg(test)]
mod tests;

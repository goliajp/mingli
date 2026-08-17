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
}

/// 两黄经的最短夹角（度，0..=180）。
#[must_use]
pub fn separation(a: f64, b: f64) -> f64 {
    let d = (a - b).rem_euclid(360.0);
    d.min(360.0 - d)
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
        return NatalChart {
            planets,
            aspects,
            angles: None,
            houses: None,
            cusp_system: None,
            cusp_houses: None,
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

    NatalChart {
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
mod tests {
    use super::*;

    #[test]
    fn sun_sign_is_verifiable() {
        // 1990-06-15 太阳黄经 ~84° → 双子(sign 2)。太阳经度已校验 Meeus，故此可验证。
        let chart = compute(1990, 6, 15, 14, 30, 8.0, None);
        let sun = chart.planets.iter().find(|p| p.name == "太阳").unwrap();
        assert_eq!(sun.sign, "双子", "实得 {} @ {:.2}°", sun.sign, sun.longitude);
        assert!(sun.house.is_none() && chart.angles.is_none() && chart.houses.is_none());
        // 2024-03-25 太阳在白羊
        let c2 = compute(2024, 3, 25, 12, 0, 8.0, None);
        assert_eq!(c2.planets[0].sign, "白羊");
    }

    #[test]
    fn nine_planets_each_have_a_sign() {
        let chart = compute(2000, 1, 1, 12, 0, 8.0, None);
        assert_eq!(chart.planets.len(), 9);
        for p in &chart.planets {
            assert!(SIGNS.contains(&p.sign.as_str()));
            assert!((0.0..30.0).contains(&p.degree));
        }
        // 月亮也在（校验已接入 ephemeris ELP）。
        assert!(chart.planets.iter().any(|p| p.name == "月亮"));
    }

    #[test]
    fn aspect_geometry() {
        assert!((separation(10.0, 350.0) - 20.0).abs() < 1e-9); // 跨 0° 最短 20
        assert_eq!(classify_aspect(0.0, 90.0, 6.0), Some(("刑", 90.0)));
        assert_eq!(classify_aspect(0.0, 120.0, 6.0), Some(("拱", 120.0)));
        assert_eq!(classify_aspect(0.0, 5.0, 6.0), Some(("合", 5.0)));
        assert_eq!(classify_aspect(0.0, 45.0, 6.0), None); // 半刑不在五大相位
    }

    // —— 上升点/中天校验权威本命盘：Diana， Princess of Wales（Rodden AA）——
    // 1961-07-01 19:45 GMT+1（=UT 18:45），Sandringham 52°50′N 0°30′E。
    // astrotheme/astro.com(Placidus)：Asc=射手18°24′=258.40°、MC=天秤23°03′=203.05°、
    // Sun=巨蟹9°40′=99.667°（Sun 经度由 VSOP87 独立给出，三方交叉验证整条管线）。
    #[test]
    fn ascendant_midheaven_matches_diana() {
        let geo = GeoLocation { latitude: 52.833, longitude: 0.500 };
        let chart = compute(1961, 7, 1, 19, 45, 1.0, Some(geo));
        let a = chart.angles.as_ref().expect("有地理坐标应出 Asc/MC");
        assert_eq!(a.asc_sign, "射手", "Asc 实得 {} @ {:.2}°", a.asc_sign, a.ascendant);
        assert_eq!(a.mc_sign, "天秤", "MC 实得 {} @ {:.2}°", a.mc_sign, a.midheaven);
        // 角分级容差（oracle 为 arcmin、本算用平恒星时/平交角，无章动）。
        assert!((a.ascendant - 258.40).abs() < 0.5, "Asc={:.3}°，应 ≈258.40°", a.ascendant);
        assert!((a.midheaven - 203.05).abs() < 0.5, "MC={:.3}°，应 ≈203.05°", a.midheaven);
        // Sun 落座经度独立交叉验证（VSOP87）。
        let sun = chart.planets.iter().find(|p| p.name == "太阳").unwrap();
        assert!((sun.longitude - 99.667).abs() < 0.2, "Sun={:.3}°，应 ≈99.667°", sun.longitude);
        // 月亮落座经度独立交叉验证（ELP-2000/82， ephemeris）。
        // Astrodienst Placidus 给出 Moon @ Aquarius 25°02' ≈ 325.033°。
        let moon = chart.planets.iter().find(|p| p.name == "月亮").unwrap();
        assert_eq!(moon.sign, "水瓶", "Moon 实得 {} @ {:.2}°", moon.sign, moon.longitude);
        assert!(
            (moon.longitude - 325.033).abs() < 0.2,
            "Moon={:.3}°，应 ≈325.033°（水瓶 25°02'）",
            moon.longitude
        );
    }

    // —— Whole Sign 整宫制结构 ——
    #[test]
    fn whole_sign_houses_structure() {
        let geo = GeoLocation { latitude: 52.833, longitude: 0.500 };
        let chart = compute(1961, 7, 1, 19, 45, 1.0, Some(geo));
        let houses = chart.houses.as_ref().unwrap();
        assert_eq!(houses.len(), 12);
        // 第一宫=上升星座；逐宫推进一星座。
        assert_eq!(houses[0].sign, "射手");
        for k in 0..12 {
            assert_eq!(houses[k].number, (k + 1) as u8);
            let want = SIGNS[(8 + k) % 12]; // 射手=8
            assert_eq!(houses[k].sign, want);
        }
        // 每颗星都被归入唯一一宫，且与其 house 字段一致。
        for p in &chart.planets {
            let h = p.house.expect("有坐标时星应有宫位");
            assert!((1..=12).contains(&h));
            assert!(houses[(h - 1) as usize].planets.contains(&p.name));
        }
    }

    #[test]
    fn house_system_id_name_roundtrip() {
        let all = [
            HouseSystem::Placidus,
            HouseSystem::Koch,
            HouseSystem::WholeSign,
            HouseSystem::Equal,
            HouseSystem::Porphyry,
        ];
        // id 与 from_id 互反；id/name 非空且唯一。
        let mut ids = std::collections::HashSet::new();
        let mut names = std::collections::HashSet::new();
        for hs in all {
            assert_eq!(HouseSystem::from_id(hs.id()), hs);
            assert!(ids.insert(hs.id()));
            assert!(names.insert(hs.name()));
            assert!(!hs.name().is_empty());
        }
        // 未知 id 退到 Placidus。
        assert_eq!(HouseSystem::from_id("nonexistent"), HouseSystem::Placidus);
        assert_eq!(HouseSystem::from_id(""), HouseSystem::Placidus);
    }

    #[test]
    fn koch_house_system_routes_to_koch_cusps() {
        // 显式选 Koch → cusp_system == "koch"，cusp_houses Some。
        let geo = GeoLocation { latitude: 52.833, longitude: 0.5 };
        let m = Moment::new(1961, 7, 1, 19, 45, 1.0);
        let chart = compute_at(&m, Some(geo), HouseSystem::Koch);
        assert_eq!(chart.cusp_system.as_deref(), Some("koch"));
        assert!(chart.cusp_houses.is_some());
        // Whole Sign → 不出 cusp_houses。
        let chart_w = compute_at(&m, Some(geo), HouseSystem::WholeSign);
        assert_eq!(chart_w.cusp_system.as_deref(), Some("whole_sign"));
        assert!(chart_w.cusp_houses.is_none());
        // Equal / Porphyry 也走 cusp_houses。
        for hs in [HouseSystem::Equal, HouseSystem::Porphyry] {
            let c = compute_at(&m, Some(geo), hs);
            assert_eq!(c.cusp_system.as_deref(), Some(hs.id()));
            assert!(c.cusp_houses.is_some());
        }
    }

    /// 极区：Placidus 与 Koch 的分宫方程在 |φ| > 66.5° 附近无解，
    /// 此时回落 Porphyry 并**如实记进 `cusp_system`**——盘照出，但不假装用的是 Placidus。
    #[test]
    fn beyond_the_polar_circle_the_house_system_falls_back_and_says_so() {
        // 特罗姆瑟 69°39′N 18°57′E，极夜期。
        let geo = GeoLocation { latitude: 69.65, longitude: 18.95 };
        let m = Moment::new(2026, 12, 21, 12, 0, 1.0);
        for requested in [HouseSystem::Placidus, HouseSystem::Koch] {
            let chart = compute_at(&m, Some(geo), requested);
            assert_eq!(
                chart.cusp_system.as_deref(),
                Some("porphyry"),
                "{requested:?} 在极区应回落 Porphyry 并如实记录"
            );
            let cusps = chart.cusp_houses.as_ref().expect("回落后仍应出 12 宫");
            assert_eq!(cusps.len(), 12);
        }
        // 同一坐标在中纬度不回落：Placidus 解得出来就该用 Placidus。
        let mid = GeoLocation { latitude: 52.833, longitude: 0.500 };
        let m2 = Moment::new(1961, 7, 1, 19, 45, 1.0);
        assert_eq!(
            compute_at(&m2, Some(mid), HouseSystem::Placidus).cusp_system.as_deref(),
            Some("placidus")
        );
        assert_eq!(
            compute_at(&m2, Some(mid), HouseSystem::Koch).cusp_system.as_deref(),
            Some("koch")
        );
    }

    // —— Asc/MC 闭式：赤道(φ=0)上 MC 与 Asc 应正交于子午圈几何 ——
    #[test]
    fn asc_mc_closed_form_sanity() {
        // RAMC=0（春分点上中天）、ε=23.44°、φ=0：MC=0°（白羊0°）、Asc=90°（巨蟹0°，东地平）。
        let (asc, mc) = asc_mc(0.0, 23.44, 0.0);
        assert!(mc.abs() < 1e-9 || (mc - 360.0).abs() < 1e-9, "MC={mc}");
        assert!((asc - 90.0).abs() < 1e-9, "Asc={asc}");
    }
}

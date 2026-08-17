//! 西洋占星的校验：宫位制、行星落座与相位。

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

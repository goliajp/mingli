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
    // 容差 0.05°（3′）：oracle 只给到角分（±0.008°），本算不含章动（≤0.005°），
    // 实测 Asc 差 +0.0072°、MC +0.0047°，余量约七倍。同一对值在 `mingli-ephemeris`
    // 的 `ascendant_and_midheaven_match_diana` 里也验一遍——那里验的是球面几何本身。
    assert!((a.ascendant - 258.40).abs() < 0.05, "Asc={:.3}°，应 ≈258.40°", a.ascendant);
    assert!((a.midheaven - 203.05).abs() < 0.05, "MC={:.3}°，应 ≈203.05°", a.midheaven);
    // Sun 落座经度独立交叉验证（VSOP87）。
    let sun = chart.planets.iter().find(|p| p.name == "太阳").unwrap();
    // 容差 0.03°：Meeus 低精度太阳黄经本身约 0.01° 量级，实测差 +0.0039°。
    assert!((sun.longitude - 99.667).abs() < 0.03, "Sun={:.3}°，应 ≈99.667°", sun.longitude);
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

// ── 两盘比对 ────────────────────────────────────────────────────────────
mod cross {
    use super::*;

    fn pos(name: &str, lon: f64) -> PlanetPos {
        PlanetPos {
            name: name.to_string(),
            longitude: lon,
            sign: String::new(),
            degree: 0.0,
            house: None,
        }
    }

    /// 相位是几何，与盘内同一套判定——夹角落在相位角的容许度内就算。
    #[test]
    fn a_cross_aspect_is_the_same_geometry_as_an_in_chart_one() {
        let a = [pos("太阳", 10.0)];
        let b = [pos("月亮", 190.0), pos("火星", 100.5), pos("木星", 45.0)];
        let got = cross_aspects(&a, &b, DEFAULT_ORB);
        // 10 与 190 差 180 → 冲；10 与 100.5 差 90.5 → 刑（容许 6°）；10 与 45 差 35 → 无
        let kinds: Vec<&str> = got.iter().map(|x| x.kind).collect();
        assert_eq!(kinds, ["冲", "刑"], "实得 {got:?}");
        assert!((got[0].angle - 180.0).abs() < 1e-9);
    }

    /// 夹角是对称的，**归属不是**——每一条都要说清是谁的星对谁的星。
    ///
    /// 这是合盘与盘内相位最要紧的差别：盘内是一张表的上三角（同一个人的两颗星），
    /// 合盘是两张表的全矩阵，且每格都有主宾。丢掉主宾，「甲的太阳合乙的月亮」
    /// 与「乙的太阳合甲的月亮」就成了同一句话，而它们说的是两回事。
    #[test]
    fn each_pair_says_whose_planet_is_whose() {
        let a = [pos("甲太阳", 0.0), pos("甲月亮", 90.0)];
        let b = [pos("乙太阳", 90.0), pos("乙月亮", 180.0)];
        let ab = cross_aspects(&a, &b, DEFAULT_ORB);
        assert!(!ab.is_empty());
        for x in &ab {
            assert!(x.a.starts_with('甲') && x.b.starts_with('乙'), "主宾错位：{x:?}");
        }
        // 反向跑一遍，主宾整体调转，而夹角一一对应
        let ba = cross_aspects(&b, &a, DEFAULT_ORB);
        assert_eq!(ab.len(), ba.len());
        for x in &ba {
            assert!(x.a.starts_with('乙') && x.b.starts_with('甲'), "反向主宾错位：{x:?}");
        }
        for x in &ab {
            assert!(
                ba.iter().any(|y| y.a == x.b && y.b == x.a && y.kind == x.kind
                    && (y.angle - x.angle).abs() < 1e-9),
                "每条都该在反向里有一条镜像：{x:?}"
            );
        }
    }

    /// 容许度放宽只会多出相位，不会少——单调性，防止边界写反。
    #[test]
    fn a_wider_orb_never_loses_an_aspect() {
        let a = [pos("太阳", 12.3), pos("金星", 200.0)];
        let b = [pos("月亮", 100.0), pos("土星", 15.0), pos("火星", 272.7)];
        let narrow = cross_aspects(&a, &b, 2.0);
        let wide = cross_aspects(&a, &b, 8.0);
        assert!(wide.len() >= narrow.len());
        for n in &narrow {
            assert!(wide.iter().any(|w| w.a == n.a && w.b == n.b && w.kind == n.kind));
        }
    }
}

/// 二次推运：**用两条各家都点名的性质当 oracle**，而不是拿本算的输出当参照。
///
/// 「一日一年」两源同述（Cafe Astrology《Secondary Progressions》、
/// Kepler College《An Introduction to Secondary Progressions》），
/// 而它们同时给出了两个可独立核对的量：推运太阳约 **1°/年**、推运月亮约 **13°/年**
/// （后者故每两三年换一座）。这两个数不是从本实现导出的，
/// 所以拿它们做判据能真正验到「一日一年」这条换算有没有落对——
/// 若误写成一日一月、或把日数当成年数乘错，两条立刻都不成立。
///
/// **推运不在本命盘上**（每格一次完整星历，而问本命盘的人没有要一生的运），
/// 故这里直接调 `progression::progression`——用例层的「运」那条路调的也是它。
#[test]
fn secondary_progression_moves_the_sun_a_degree_and_the_moon_thirteen_per_year() {
    let m = Moment::new(1990, 6, 15, 14, 30, 8.0);
    let chart = compute_at(&m, None, HouseSystem::WholeSign);
    let p = progression::progression(m.jde, &chart.planets, 100, 1);
    assert_eq!(p.method, "secondary");
    assert_eq!(p.step, 1);
    assert_eq!(u32::try_from(p.years.len()).expect("格数"), p.max_age + 1, "step = 1 时每岁一格");
    assert_eq!(p.years[0].age, 0);

    // 第 0 岁的推运盘就是本命盘本身（出生后第 0 日）
    for (a, b) in p.years[0].planets.iter().zip(&chart.planets) {
        assert!((a.longitude - b.longitude).abs() < 1e-9, "第 0 岁应与本命同盘：{} ", a.name);
    }

    let lon = |age: usize, name: &str| {
        p.years[age].planets.iter().find(|x| x.name == name).expect("该星应在盘上").longitude
    };

    // 判据一：推运太阳约 1°/年。百年净移不足 100°，不绕圈，无须补圈记账
    let sun_rate = (lon(100, "太阳") - lon(0, "太阳")).rem_euclid(360.0) / 100.0;
    assert!(
        (0.9..=1.05).contains(&sun_rate),
        "推运太阳应约 1°/年（两源同述），实测 {sun_rate:.4}°/年",
    );

    // 判据二：**推运月亮每两三年换一座**——两源都是这么写的，且它不必补圈，
    // 比「13°/年」更适合做判据：月行度在 11.8–15.4°/日间摆，跨度取多长都留几分残差，
    // 而「几年换一座」是把那点残差吸收掉之后仍成立的说法
    let sign_of = |age: usize| (lon(age, "月亮") / 30.0).floor() as i32;
    let changes = (1..=100).filter(|&a| sign_of(a) != sign_of(a - 1)).count();
    let years_per_sign = 100.0 / f64::from(u32::try_from(changes).expect("百年内换座次数不会溢出"));
    assert!(
        (2.0..=3.0).contains(&years_per_sign),
        "推运月亮应每 2–3 年换一座，实测每 {years_per_sign:.2} 年一次（百年内换了 {changes} 次）",
    );

    // 「运」的着力处：推运星与本命星成角。一生百年里不该一条都没有
    let total: usize = p.years.iter().map(|y| y.to_natal.len()).sum();
    assert!(total > 0, "百年推运里竟无一处与本命成相位，说明比对那一步没接上");

    // 粒度可调：十年一格恰 11 格，且与逐年那份的同岁数取值一致
    let d = progression::progression(m.jde, &chart.planets, 100, 10);
    assert_eq!(d.step, 10);
    assert_eq!(d.years.len(), 11);
    for (k, y) in d.years.iter().enumerate() {
        assert_eq!(y.age, u32::try_from(k).expect("格序") * 10);
        // 比位模式而非近似：同一岁数两种粒度算的是同一个 jde，**必须逐位相同**。
        // 若哪天不同了，说明粒度参数漏进了计算本身，那是缺陷不是精度问题
        assert_eq!(
            y.planets[0].longitude.to_bits(),
            p.years[k * 10].planets[0].longitude.to_bits(),
            "同岁数两种粒度应给逐位相同的值",
        );
    }
}

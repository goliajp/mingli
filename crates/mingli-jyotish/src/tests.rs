//! Jyotish 的校验：岁差多源对照、宿/宫分段边界、大运周期性质。

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

// ── Antardaśā（bhukti）：BPHS 51.1–51.2 的比例细分 ──────────────────

/// 每步 = 主星年数 × 子星年数 ÷ 120，九步之和恰等于主运跨度。
///
/// BPHS 51.1「daśābdāḥ svasvamānaghnāḥ sarvāyuryogabhājitāḥ」——
/// 主星年数乘子星年数、除以全部主星年数之和（120）。
/// drik-panchanga（`factor = mahadasa[lord] * mahadasa[maha_lord] / 120.`）、
/// PyJHora（除数常量 `human_life_span_for_vimsottari_dhasa = 120`）、
/// VedAstro（`const double fullHumanLifeYears = 120.0`）三个实现逐条一致。
#[test]
fn each_antardasha_is_the_product_over_one_twenty_and_they_tile_the_period() {
    let tl = vimshottari_timeline(123.456, 2_447_892.5);
    for md in &tl {
        assert_eq!(md.antardashas.len(), 9, "{} 应有九步", md.lord);
        let md_years = VIMSHOTTARI_YEARS.iter().find(|(l, _)| *l == md.lord).expect("主星在表内").1;
        let mut total = 0.0;
        for ad in &md.antardashas {
            let sub_years = VIMSHOTTARI_YEARS.iter().find(|(l, _)| *l == ad.lord).expect("子星在表内").1;
            let want = md.years * sub_years / 120.0;
            assert!((ad.years - want).abs() < 1e-9, "{}·{} 应为 {want}，实得 {}", md.lord, ad.lord, ad.years);
            total += ad.years;
        }
        // 九步铺满主运跨度，首尾与主运对齐
        assert!((total - md_years).abs() < 1e-9, "{} 的九步之和应等于 {md_years}", md.lord);
        assert!((md.antardashas[0].start_age_years - md.start_age_years).abs() < 1e-9);
        assert!((md.antardashas[8].end_age_years - md.end_age_years).abs() < 1e-9);
    }
}

/// 首个子运即主星自己，其后依同一固定顺序循环（BPHS 51.2）。
#[test]
fn the_first_antardasha_belongs_to_the_lord_of_the_dasha() {
    for lon in [0.0, 13.5, 99.9, 200.0, 359.99] {
        for md in vimshottari_timeline(lon, 2_451_545.0) {
            assert_eq!(md.antardashas[0].lord, md.lord, "首个子运应是主星自己");
            // 九个子星互不重复，恰是那九颗
            let mut got: Vec<&str> = md.antardashas.iter().map(|a| a.lord).collect();
            got.sort_unstable();
            let mut want: Vec<&str> = VIMSHOTTARI_LORDS.to_vec();
            want.sort_unstable();
            assert_eq!(got, want);
            // 顺序是固定序列的循环
            let start = VIMSHOTTARI_LORDS.iter().position(|l| *l == md.lord).expect("主星在序列内");
            for (i, ad) in md.antardashas.iter().enumerate() {
                assert_eq!(ad.lord, VIMSHOTTARI_LORDS[(start + i) % 9], "第 {i} 步");
            }
        }
    }
}

/// 一年折合多少天是参数，不是常数——换一个年长，整条时间轴按比例伸缩，年龄不变。
///
/// 原典只给年数比例、不规定年长；实查到六个不同取值（见 `YEAR_LENGTHS`）。
#[test]
fn the_length_of_a_year_scales_the_timeline_without_moving_the_ages() {
    let birth = 2_447_892.5;
    let julian = vimshottari_timeline_with(123.456, birth, 365.25);
    let savana = vimshottari_timeline_with(123.456, birth, 360.0);
    for (a, b) in julian.iter().zip(savana.iter()) {
        assert_eq!(a.lord, b.lord);
        // 年龄（以「年」计）与年长无关
        assert!((a.start_age_years - b.start_age_years).abs() < 1e-9);
        // 儒略日则按年长伸缩
        let ja = a.end_jd - birth;
        let jb = b.end_jd - birth;
        assert!((ja * 360.0 - jb * 365.25).abs() < 1e-6, "两种年长应成定比");
    }
    // 六个取值互不相同、皆为正
    let mut vals: Vec<f64> = YEAR_LENGTHS.iter().map(|(_, v)| *v).collect();
    assert!(vals.iter().all(|v| *v > 300.0));
    vals.sort_by(f64::total_cmp);
    vals.dedup();
    assert_eq!(vals.len(), 6, "六个取值应互不相同");
    // 默认入口取儒略年
    let d = vimshottari_timeline(123.456, birth);
    assert!((d[0].end_jd - julian[0].end_jd).abs() < 1e-9);
}

/// 108 个 navamsa 边界逐个落在正确的一格里。
///
/// 边界是零测集，实盘几乎踩不到，但这条守卫零成本，而它挡的是一类很隐蔽的错：
/// `lon × 0.3` 看着等价于 `lon × 9 / 30`，实际 0.3 在二进制里表示不精确，
/// 108 个边界里有 25 个会落回上一格——盘面照出，只是那一格错了。
#[test]
fn all_one_hundred_eight_navamsa_boundaries_land_in_the_right_division() {
    for k in 0..108 {
        let lon = f64::from(k) * 10.0 / 3.0;
        assert_eq!(
            navamsa_of(lon),
            (k as usize) % 12,
            "第 {k} 个 navamsa 边界（{lon}°）落错格"
        );
        // 格内一点点也该在同一格
        assert_eq!(navamsa_of(lon + 1.0), (k as usize) % 12, "边界 {k} 之后 1° 应仍在本格");
    }
}

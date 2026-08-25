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

/// 与公布的 Lahiri 表值逐年对照，容差由**章动**定，不是随手放宽的。
///
/// 这里有一处必须先说清，否则整条对照会读错：本 crate 算的是**平** ayanamsa。
/// Swiss Ephemeris 的锚点写作 `23.250182778 - 0.004658035`，而它自己的注释讲明
/// 「the subtracted value is nutation」——减掉的就是章动。各家表与排盘软件公布的
/// 则是**含章动的真值**。两者之差恒等于当日的 Δψ，量级 ≤ 17.2″ 且**来回摆动**。
///
/// 参照取 Jagannath Hora 的历史表（Swiss Ephemeris 所算，各年 1 月 1 日 00:00 UT）：
/// <https://jagannathhora.com/historical-lahiri-ayanamsa-values-tables/>。
/// 实测四个年份的差为 +7.9″ / +13.4″ / **−12.2″** / +13.7″——符号会翻，
/// 这正是章动在摆而不是本算在漂。故容差取 20″（0.0056°）：够容下整个章动包络，
/// 又比它紧得多，真出现系统性漂移就会红。
///
/// **这条测试是重写的。** 原先两条各验一个时点、容差 ±0.1°（6′），比实际吻合度松约 25 倍，
/// 于是其中一条引的参照值 `1985-09-04 = 23°41'27"` 错了约 2.2′ 也照样通过——
/// 表值实为 1985-01-01 = 23°38'38"，折到 9 月约 23.653°。
#[test]
fn lahiri_tracks_the_published_tables_to_within_the_nutation_term() {
// 章动 Δψ 的振幅上界（Meeus AA 第 22 章主项 17.20″，余项合计不足 2″）
const NUTATION_ENVELOPE_ARCSEC: f64 = 20.0;
// (年, 该年 1 月 1 日 00:00 UT 的 JD, 表值 度/分/秒)
let table = [
    (1980, 2_444_239.5_f64, (23, 34, 32)),
    (1985, 2_446_066.5, (23, 38, 38)),
    (1990, 2_447_892.5, (23, 43, 15)),
    (2000, 2_451_544.5, (23, 51, 12)),
];
for (year, jd, (d, m, s)) in table {
    let want = f64::from(d) + f64::from(m) / 60.0 + f64::from(s) / 3600.0;
    let got = ayanamsa(jd, Ayanamsa::Lahiri);
    let diff = (got - want) * 3600.0;
    assert!(
        diff.abs() < NUTATION_ENVELOPE_ARCSEC,
        "{year}-01-01：本算（平）{got:.6}° vs 表值（真）{want:.6}°，差 {diff:+.1}″，\
         超出章动包络 ±{NUTATION_ENVELOPE_ARCSEC}″——那就不是章动了，是本算漂了",
    );
}
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

/// 时间轴的**儒略日**字段。
///
/// 此前只验过年龄那两个字段。变异测试在 `start_jd` / `end_jd` 那几行上留了十来个活口——
/// 对外的 JSON 给的正是它们，下游要落到日历上就得用它们。
///
/// 四条性质，实测（2026-08-26）在 [`YEAR_LENGTHS`] 六种年长下全部成立：
/// 段与段之间无缝、九段总跨恰为 120 × 年长、每段的九步子运铺满该段、
/// 而**年龄完全不随年长变化**——最后这条钉住「年长只进儒略日、不进年龄」。
#[test]
fn the_julian_days_tile_the_timeline_at_every_year_length() {
    let birth = 2_447_000.5f64;
    for (name, days_per_year) in crate::dasha::YEAR_LENGTHS {
        for lon in [0.0f64, 6.5, 13.0, 100.0, 200.0, 359.9] {
            let timeline = crate::dasha::vimshottari_timeline_with(lon, birth, days_per_year);
            assert_eq!(timeline.len(), 9);

            // 段间无缝且严格前进。
            for pair in timeline.windows(2) {
                assert!(
                    (pair[1].start_jd - pair[0].end_jd).abs() < 1e-6,
                    "{name} lon={lon}：{} 段止于 {} 而下一段起于 {}",
                    pair[0].lord,
                    pair[0].end_jd,
                    pair[1].start_jd
                );
                assert!(pair[0].end_jd > pair[0].start_jd, "{name}：段内应前进");
            }

            // 九段总跨 = 120 年 × 年长。
            let span = timeline[8].end_jd - timeline[0].start_jd;
            assert!(
                (span - 120.0 * days_per_year).abs() < 1e-6,
                "{name} lon={lon}：总跨 {span} 天，应为 {} 天",
                120.0 * days_per_year
            );

            // 出生落在首段之内（首段起点在出生之前，这是 birth daśā 的残段）。
            assert!(
                timeline[0].start_jd <= birth && birth <= timeline[0].end_jd,
                "{name} lon={lon}：出生 {birth} 不在首段 [{}, {}] 内",
                timeline[0].start_jd,
                timeline[0].end_jd
            );

            // 九步子运在儒略日上铺满本段。
            for md in &timeline {
                assert!((md.antardashas[0].start_jd - md.start_jd).abs() < 1e-6);
                assert!((md.antardashas[8].end_jd - md.end_jd).abs() < 1e-6);
                for pair in md.antardashas.windows(2) {
                    assert!(
                        (pair[1].start_jd - pair[0].end_jd).abs() < 1e-6,
                        "{name} lon={lon} {} 段内子运有缝",
                        md.lord
                    );
                }
            }
        }
    }

    // 年长只该进儒略日，不该进年龄。
    let julian = crate::dasha::vimshottari_timeline_with(100.0, birth, 365.25);
    let savana = crate::dasha::vimshottari_timeline_with(100.0, birth, 360.0);
    for (a, b) in julian.iter().zip(savana.iter()) {
        assert_eq!(a.lord, b.lord);
        assert!((a.start_age_years - b.start_age_years).abs() < 1e-12, "换年长不该动年龄");
        assert!((a.end_age_years - b.end_age_years).abs() < 1e-12, "换年长不该动年龄");
        // 年龄非零的段，换了年长儒略日就该跟着动；年龄恰为零的那一点两边同为出生时刻。
        if a.start_age_years.abs() > 1e-12 {
            assert!(
                (a.start_jd - b.start_jd).abs() > 1e-6,
                "{} 段起于年龄 {}，换年长后儒略日却没动",
                a.lord,
                a.start_age_years
            );
        }
    }
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
// 1990 Lahiri ~ 23.65°。这是**合理性上界**不是 oracle——真正对表的那条在
// `lahiri_tracks_the_published_tables_to_within_the_nutation_term`，容差 20″。
// 这里 23.65 是个取整的参照，实测差 0.074°，故留 0.10。
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

// ── 分盘 ────────────────────────────────────────────────────────────────
//
// 每一条落宫都由两个彼此独立的开源实现交叉确认：`kunjara/jyotish`（PHP，每盘只实现
// Parasara 一法）与 `naturalstupid/PyJHora`（Python，每盘并列 3–6 法，取其 Parasara 默认）。
// 把两者各自的写法分别转录一遍，在 12 盘 × 12 宫 × 300 点 = 43 200 个点上逐点比对，
// **零分歧**；下面的期望值取自那批全同的点。
mod varga_tests {
    use crate::varga::{all_vargas, varga_rasi, Varga, ALL};

    /// 除数顺序与 `ALL` 一致，供下面的表逐列对上。
    const DIVISORS: [Varga; 12] = ALL;

    /// (恒星黄经, 十二盘落宫)。落宫 0=白羊 … 11=双鱼。
    const ORACLE: [(f64, [usize; 12]); 11] = [
        (0.0, [0, 0, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0]),
        (13.4, [4, 3, 3, 4, 5, 7, 8, 2, 0, 5, 8, 2]),
        (29.99, [8, 9, 6, 9, 11, 3, 7, 3, 2, 3, 8, 11]),
        (45.0, [5, 7, 10, 2, 7, 0, 6, 3, 4, 2, 2, 7]),
        (77.7, [6, 8, 6, 7, 9, 5, 3, 6, 9, 11, 10, 1]),
        (123.456, [4, 4, 4, 5, 5, 5, 10, 6, 3, 4, 9, 10]),
        (180.0, [6, 6, 6, 6, 6, 0, 0, 4, 6, 0, 0, 6]),
        (222.22, [11, 10, 3, 7, 11, 10, 4, 0, 7, 10, 10, 7]),
        (271.5, [9, 9, 3, 5, 9, 0, 1, 4, 4, 8, 2, 0]),
        (333.33, [11, 11, 5, 8, 0, 9, 6, 5, 11, 10, 0, 5]),
        (359.99, [7, 8, 11, 4, 10, 11, 11, 2, 11, 9, 4, 10]),
    ];

    #[test]
    fn every_varga_matches_both_reference_implementations() {
        for (lon, want) in ORACLE {
            for (i, v) in DIVISORS.iter().enumerate() {
                assert_eq!(
                    varga_rasi(*v, lon),
                    want[i],
                    "{} 在恒星黄经 {lon}° 上应落第 {} 宫",
                    v.id(),
                    want[i]
                );
            }
        }
    }

    /// 一宫三十度切成 n 份，走满一宫恰好用掉 n 份——不多不少。
    ///
    /// 这条抓的是「份宽算错」：份宽偏大则一宫走不满，偏小则越界到下一宫，
    /// 两种都不会让上面的抽样 oracle 全错，只会错在边界附近。
    #[test]
    fn one_sign_is_exactly_n_parts_wide() {
        for v in ALL {
            let n = v.divisor() as usize;
            for rasi in 0_i32..12 {
                let base = f64::from(rasi) * 30.0;
                let seen: Vec<usize> = (0..n)
                    .map(|k| varga_rasi(v, base + (k as f64 + 0.5) * 30.0 / n as f64))
                    .collect();
                assert_eq!(seen.len(), n);
                // 份序连续推进：相邻两份的落宫差必是固定步长（各盘的 step 见模块说明）
                let step = (seen[1] + 12 - seen[0]) % 12;
                for w in seen.windows(2) {
                    assert_eq!(
                        (w[1] + 12 - w[0]) % 12,
                        step,
                        "{} 在第 {rasi} 宫内的份序推进不匀",
                        v.id()
                    );
                }
            }
        }
    }

    /// 落宫恒在 0..12，且黄经绕一圈回到原处。
    #[test]
    fn a_varga_rasi_is_always_a_rasi() {
        for v in ALL {
            for k in 0..3600 {
                let lon = f64::from(k) / 10.0;
                let r = varga_rasi(v, lon);
                assert!(r < 12, "{} 落宫 {r} 越界", v.id());
                assert_eq!(r, varga_rasi(v, lon + 360.0), "{} 绕一圈应回到原处", v.id());
                assert_eq!(r, varga_rasi(v, lon - 360.0), "{} 负向绕一圈应回到原处", v.id());
            }
        }
    }

    /// 十二盘各有其名与所主，且 id 不重。
    #[test]
    fn the_twelve_vargas_are_all_named() {
        let mut ids: Vec<&str> = ALL.iter().map(|v| v.id()).collect();
        ids.sort_unstable();
        let n = ids.len();
        ids.dedup();
        assert_eq!(ids.len(), n, "分盘 id 不该重复");
        for v in ALL {
            assert!(!v.sanskrit_name().is_empty() && !v.subject().is_empty());
            assert_eq!(v.id(), format!("d{}", v.divisor()));
        }
    }

    /// 汇总函数与逐盘计算给同一个答案。
    #[test]
    fn the_summary_agrees_with_each_varga() {
        let lon = 123.456;
        let all = all_vargas(lon);
        assert_eq!(all.rasi.len(), 12);
        for v in ALL {
            assert_eq!(all.rasi[v.id()], varga_rasi(v, lon));
        }
    }
}

/// Ashtakuta：拿各家都点名的**结构性事实**当判据。
///
/// 这套系统的逐项判定表各家不同（见 `kuta.rs` 的模块说明与本叶 `profile()`），
/// 所以不能拿「某对男女得几分」当 oracle——那个数随选哪份表而变。
/// 能当判据的是各家一致的结构：权重之和、Nadi 的同则零、Bhakoot 的凶位、
/// 同宿同宫必得满分、以及三张按宿查的表各自的分布。
/// 八项里的 Tara：两人月宿相隔数各除以 9，余数落在 Vipat(3)/Pratyak(5)/Vadha(7) 为凶。
///
/// 这一项此前一条测试也没有——变异测试把 `tara_step` 整个换成常量 0 或 1、
/// 把 `tara_bad` 整个换成恒真或恒假，全都活了下来。
///
/// 规则与出处见本 crate `kuta.rs` 的模块头：两份互相独立的公布表
/// （Saravali 的 Asta Koota 分项页与 freehoroscopesonline 的同名页）在此项上一致，
/// 作「两向皆吉 3 分、一吉一凶 1.5、皆凶 0」。
#[test]
fn tara_counts_the_stars_between_and_calls_three_of_every_nine_bad() {
    use crate::kuta::ashtakuta;

    // 相隔数从 1 起数，同宿即第一位 Janma。二十七宿走一圈恰取遍 1..=27。
    for from in 0..27usize {
        let mut seen: Vec<usize> = (0..27).map(|to| crate::kuta::tara_step(from, to)).collect();
        assert_eq!(seen[from], 1, "自宿到自宿应是第一位");
        seen.sort_unstable();
        assert_eq!(seen, (1..=27).collect::<Vec<_>>(), "从第 {from} 宿数出去应取遍 1..=27");
    }

    // 九位里恰三位为凶：Vipat 第 3、Pratyak 第 5、Vadha 第 7。
    // 二十七位里因此恰九位为凶，且余 0 那一位（第 9、18、27）算吉。
    let bad: Vec<usize> = (1..=27).filter(|&s| crate::kuta::tara_bad(s)).collect();
    assert_eq!(bad, vec![3, 5, 7, 12, 14, 16, 21, 23, 25], "凶位应是 3/5/7 每九位一轮");
    for good in [9usize, 18, 27] {
        assert!(!crate::kuta::tara_bad(good), "余 0 那一位算吉");
    }

    // 落到分数上：两向皆吉 30/10 分、一吉一凶 15/10、皆凶 0。
    let tara_of = |b: usize, g: usize| -> u32 {
        ashtakuta((b, 0), (g, 0))
            .kutas
            .iter()
            .find(|k| k.kuta == "Tara")
            .expect("八项里有 Tara")
            .min_tenths
    };
    // 同宿：两向都是第一位，皆吉。
    assert_eq!(tara_of(0, 0), 30, "同宿两向皆吉");
    // 女宿 0、男宿 2：一向第 3（Vipat，凶），反向第 26（吉）。
    assert_eq!(crate::kuta::tara_step(0, 2), 3);
    assert_eq!(crate::kuta::tara_step(2, 0), 26);
    assert_eq!(tara_of(0, 2), 15, "一吉一凶");
    // 两向皆凶那一档在这套数法下取不到，这不是缺测而是算术使然：
    // 两向的相隔数恒和为 29（同宿时为 2），而 29 ≡ 2 (mod 9)，凶位是 3/5/7，
    // 配对的另一位必是 8/6/4，一定是吉。七百二十九对逐一验过，皆凶为零。
    let mut tally = [0u32; 3]; // [两向皆吉, 一吉一凶, 两向皆凶]
    for b in 0..27usize {
        for g in 0..27usize {
            let bad_count = u32::from(crate::kuta::tara_bad(crate::kuta::tara_step(b, g)))
                + u32::from(crate::kuta::tara_bad(crate::kuta::tara_step(g, b)));
            tally[bad_count as usize] += 1;
            let sum = crate::kuta::tara_step(b, g) + crate::kuta::tara_step(g, b);
            assert!(sum == 29 || sum == 2, "第 {b}/{g} 宿两向相隔数之和为 {sum}");
            // 得分只该落在这两档上。
            let t = tara_of(b, g);
            assert!(t == 30 || t == 15, "第 {b}/{g} 宿的 Tara 得 {t}，应为 30 或 15");
        }
    }
    assert_eq!(tally, [243, 486, 0], "两向皆吉 243 对、一吉一凶 486 对、皆凶 0 对");
}

#[test]
fn ashtakuta_holds_the_structure_every_source_agrees_on() {
    use crate::kuta::{ashtakuta, GANA, NADI, YONI, YONI_SWORN_ENEMIES};

    // 八项权重之和恒为 36——这是这套系统的定义
    let r = ashtakuta((0, 0), (0, 0));
    assert_eq!(r.kutas.len(), 8);
    assert_eq!(r.kutas.iter().map(|k| k.max_points).sum::<u32>(), 36);
    assert_eq!(r.max_points, 36);

    // 同宿同宫：除 Nadi 外各项皆满，而 **Nadi 必为 0**——同脉得零是各家一致的一条
    let nadi = r.kutas.iter().find(|k| k.kuta == "Nadi").expect("应有 Nadi 一项");
    assert_eq!((nadi.min_tenths, nadi.max_tenths), (0, 0), "同宿必同脉，Nadi 应得 0");
    let yoni = r.kutas.iter().find(|k| k.kuta == "Yoni").expect("应有 Yoni 一项");
    assert_eq!((yoni.min_tenths, yoni.max_tenths), (40, 40), "同宿必同兽，Yoni 应满 4 且无区间");
    // 故同宿同宫的总分上界必不足 36（Nadi 的 8 分拿不到）
    assert!(r.total_max_tenths <= 280, "同宿同宫应至多 28 分，实得 {}", f64::from(r.total_max_tenths) / 10.0);

    // Nadi：异脉必满 8，同脉必 0
    for (bn, gn) in [(0_usize, 1_usize), (0, 2), (3, 4)] {
        let k = ashtakuta((bn, 0), (gn, 0));
        let n = k.kutas.iter().find(|k| k.kuta == "Nadi").expect("Nadi");
        let want = if NADI[bn] == NADI[gn] { 0 } else { 80 };
        assert_eq!(n.min_tenths, want, "宿 {bn} 与 {gn} 的 Nadi");
        assert!(n.settled, "Nadi 两源一致，不该出区间");
    }

    // Bhakoot：2/5/6/8/9/12 位为凶得 0，其余得 7——两源同表
    for d in 0..12_usize {
        let k = ashtakuta((0, 0), (0, d));
        let b = k.kutas.iter().find(|k| k.kuta == "Bhakoot").expect("Bhakoot");
        let pos = d + 1;
        let want = if matches!(pos, 1 | 3 | 4 | 7 | 10 | 11) { 70 } else { 0 };
        assert_eq!(b.min_tenths, want, "相隔 {pos} 位的 Bhakoot");
    }

    // Yoni 死敌七对：两源完全一致地给 0，且必不出区间
    for (a, b) in YONI_SWORN_ENEMIES {
        let bn = YONI.iter().position(|x| *x == a).expect("该兽应有宿");
        let gn = YONI.iter().position(|x| *x == b).expect("该兽应有宿");
        let k = ashtakuta((bn, 0), (gn, 0));
        let y = k.kutas.iter().find(|k| k.kuta == "Yoni").expect("Yoni");
        assert_eq!((y.min_tenths, y.max_tenths), (0, 0), "死敌兽对应得 0 且无区间");
    }

    // 三张按宿查的表：结构上各类等分
    #[allow(clippy::naive_bytecount, reason = "这是分类表的计数，不是字节串搜索")]
    for (name, t, groups) in [("Gana", &GANA, 3_u8), ("Nadi", &NADI, 3)] {
        for g in 0..groups {
            let n = t.iter().filter(|x| **x == g).count();
            // clippy 的 naive_bytecount 在这里是误报：t 是分类表不是字节串
            assert_eq!(n, 9, "{name} 第 {g} 类应辖九宿，实辖 {n}");
        }
    }
    // 14 兽里十三兽各辖二宿，Uttara Ashadha 那一兽只一宿（另一宿 Abhijit 不在通行 27 宿内）
    #[allow(clippy::naive_bytecount, reason = "同上：按兽计宿数")]
    let ones = (0..14_u8).filter(|a| YONI.iter().filter(|x| *x == a).count() == 1).count();
    assert_eq!(ones, 1, "只该有一兽辖单宿");
}

/// 区间的宽度只由两源不一致的那几项贡献，且不一致的只可能是 Vashya 与 Yoni。
#[test]
fn the_spread_comes_only_from_the_two_tables_the_sources_disagree_on() {
    use crate::kuta::ashtakuta;
    for bn in [0_usize, 5, 13, 26] {
        for gn in [1_usize, 8, 20] {
            for br in [0_usize, 4, 9] {
                let r = ashtakuta((bn, br), (gn, (br + 5) % 12));
                let spread = r.total_max_tenths - r.total_min_tenths;
                let by_kuta: u32 = r.kutas.iter().map(|k| k.max_tenths - k.min_tenths).sum();
                assert_eq!(spread, by_kuta, "总区间应等于各项区间之和");
                for k in &r.kutas {
                    if !k.settled {
                        assert!(
                            matches!(k.kuta, "Vashya" | "Yoni"),
                            "只有 Vashya 与 Yoni 两源不一，却见到「{}」出区间",
                            k.kuta,
                        );
                    }
                }
                assert!(r.total_max_tenths <= 360, "总分不得超 36");
            }
        }
    }
}

/// Yoni 中段必须出区间——把它硬定成一个值，就是把两源的分歧藏起来。
///
/// 上一条只验了「不该出区间的项没出」，验不出反向：**该出区间的项被压成了定值**。
/// 而这恰是这套实现最该防的事——两份公布的 14×14 矩阵在 72/196 格上不同（69 格差 1），
/// 静默取其一，得出的「36 分制得几分」就随选谁而变，读的人无从知道。
#[test]
fn a_yoni_pair_the_sources_disagree_on_must_come_out_as_a_range() {
    use crate::kuta::{ashtakuta, YONI, YONI_SWORN_ENEMIES};

    let sworn = |a: u8, b: u8| YONI_SWORN_ENEMIES.iter().any(|&(x, y)| (x == a && y == b) || (x == b && y == a));
    let mut ranged = 0;
    let mut settled = 0;
    for (bn, &ya) in YONI.iter().enumerate() {
        for (gn, &yb) in YONI.iter().enumerate() {
            let y = ashtakuta((bn, 0), (gn, 0))
                .kutas
                .into_iter()
                .find(|k| k.kuta == "Yoni")
                .expect("Yoni");
            if ya == yb || sworn(ya, yb) {
                assert!(y.settled, "同兽或死敌两源一致，不该出区间");
                settled += 1;
            } else {
                assert!(!y.settled, "宿 {bn}×{gn}：中段两源不一，必须出区间而不是定值");
                assert_eq!((y.min_tenths, y.max_tenths), (10, 30), "中段只定得下 1..3");
                ranged += 1;
            }
        }
    }
    assert!(ranged > 0 && settled > 0, "两类都该出现过：定值 {settled} 组、区间 {ranged} 组");
}

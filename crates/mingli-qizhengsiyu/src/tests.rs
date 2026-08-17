//! 七政四余的校验：日月五星过宫、四余、28 宿值日。

use super::*;

/// 三套宿度表的 oracle：拿各书**自载的四方小计与周天**对账。
///
/// 这是最硬的一种校验——表若在传抄中崩了一位，小计立刻对不上。
/// 考据过程里就靠这条抓到两处讹误：维基文库《淮南子》作「東井三十」，
/// 整表和只有 362¼ 而非 365¼，判为脱字；《宋史》崇天历作「氐十七度」，
/// 与它自己的「東方七十五度」矛盾，改回十五才合。
#[test]
fn every_lodge_table_closes_on_the_totals_its_own_source_reports() {
    use xiudu::{table, Epoch};
    // (纪元, 东, 北, 西, 南, 周天)  单位：万分之一度
    const ORACLE: [(Epoch, u32, u32, u32, u32, u32); 3] = [
        // 《汉书·律历志下》自载「東七十五度／北九十八度／西八十度／南百一十二度」
        (Epoch::Han, 750_000, 980_000, 800_000, 1_120_000, 3_650_000),
        // 《新唐书》卷 028 上大衍历赤道；西 81 南 111（毕觜参鬼四宿改，净值为零）
        (Epoch::Dayan, 750_000, 980_000, 810_000, 1_110_000, 3_650_000),
        // 《元史》卷 054 自载「七十九度二十分／九十三度八十分太／八十三度八十五分／一百八度四十分」
        (Epoch::Shoushi, 792_000, 938_075, 838_500, 1_084_000, 3_652_575),
    ];
    for (epoch, e, n, w, s, total) in ORACLE {
        let t = table(epoch);
        let quad = |k: usize| t[k * 7..k * 7 + 7].iter().sum::<u32>();
        assert_eq!(quad(0), e, "{epoch:?} 东方七宿");
        assert_eq!(quad(1), n, "{epoch:?} 北方七宿");
        assert_eq!(quad(2), w, "{epoch:?} 西方七宿");
        assert_eq!(quad(3), s, "{epoch:?} 南方七宿");
        assert_eq!(t.iter().sum::<u32>(), total, "{epoch:?} 周天");
    }
}

/// 汉→唐只动了四宿，且原典自己点了名；汉→元则一宿不剩。
#[test]
fn the_tang_table_changes_exactly_the_four_lodges_its_source_names() {
    use xiudu::{degrees_of, table, Epoch, NAMES};
    let (han, dayan) = (table(Epoch::Han), table(Epoch::Dayan));
    let changed: Vec<&str> = (0..28).filter(|&i| han[i] != dayan[i]).map(|i| NAMES[i]).collect();
    assert_eq!(changed, ["毕", "觜", "参", "鬼"], "《新唐书》「其畢、觜觿、參、輿鬼四宿度數，與古不同」");
    // 净值为零，故周天整数部分不变
    assert_eq!(han.iter().sum::<u32>(), dayan.iter().sum::<u32>());
    // 到授时已无一宿与汉制相同
    let same = (0..28).filter(|&i| han[i] == table(Epoch::Shoushi)[i]).count();
    assert_eq!(same, 0, "授时历应无一宿与汉制相同");
    // 觜宿的塌缩：2 → 1 → 0.05
    assert_eq!(degrees_of(Epoch::Han, "觜"), Some(2.0));
    assert_eq!(degrees_of(Epoch::Dayan, "觜"), Some(1.0));
    assert_eq!(degrees_of(Epoch::Shoushi, "觜"), Some(0.05));
}

/// 汉代那 ¼ 度余分的两种归属，各自都让整表收在 365¼。
#[test]
fn either_home_for_the_quarter_degree_closes_the_circle() {
    use xiudu::{table, Epoch, QUARTER_REMAINDER, NAMES};
    for (label, idx, with_remainder) in QUARTER_REMAINDER {
        let mut t = *table(Epoch::Han);
        assert_eq!(with_remainder - t[idx], 2_500, "{label}：加的应恰是 ¼ 度");
        t[idx] = with_remainder;
        assert_eq!(t.iter().sum::<u32>(), 3_652_500, "{label}：整表应收在 365¼");
    }
    assert_eq!(NAMES[QUARTER_REMAINDER[0].1], "箕");
    assert_eq!(NAMES[QUARTER_REMAINDER[1].1], "斗");
}

/// 距度表与本叶的二十八宿名同序，取用接口自洽。
#[test]
fn the_lodge_table_lines_up_with_the_mansion_names() {
    use xiudu::{degrees, degrees_of, Epoch, NAMES};
    assert_eq!(NAMES, MANSIONS);
    for (i, name) in NAMES.iter().enumerate() {
        assert_eq!(degrees(Epoch::Han, i), degrees_of(Epoch::Han, name), "{name}");
        assert!(degrees(Epoch::Han, i).is_some_and(|d| d > 0.0), "{name} 的汉制距度应为正");
    }
    assert!(degrees(Epoch::Han, 28).is_none());
    assert!(degrees_of(Epoch::Han, "不存在之宿").is_none());
    assert_eq!(Epoch::default(), Epoch::Han);
}

/// 十二次对照表的 oracle：三层各有各的来源，逐层钉。
#[test]
fn the_twelve_ci_table_holds_on_all_three_layers() {
    use erci::{ci_of_mansion, TWELVE_CI};
    // 一、名与顺序：五部正史一致，星纪起、析木末
    let names: Vec<&str> = TWELVE_CI.iter().map(|c| c.name).collect();
    assert_eq!(
        names,
        ["星纪", "玄枵", "娵訾", "降娄", "大梁", "实沈", "鹑首", "鹑火", "鹑尾", "寿星", "大火", "析木"]
    );

    // 二、次 ↔ 辰：星纪丑起，**次序与辰序逆行**（次往前一格、辰往后一格）
    assert_eq!(TWELVE_CI[0].branch, 1, "星纪於辰在丑");
    for pair in TWELVE_CI.windows(2) {
        let (a, b) = (pair[0].branch, pair[1].branch);
        assert_eq!(b, (a + 11) % 12, "{} → {} 的辰应逆行一格", pair[0].name, pair[1].name);
    }
    // 十二辰各用一次，无重无漏
    let mut branches: Vec<u8> = TWELVE_CI.iter().map(|c| c.branch).collect();
    branches.sort_unstable();
    assert_eq!(branches, (0..12).collect::<Vec<u8>>());

    // 三、整宿归属：二十八宿恰好被十二次瓜分，不重不漏
    let mut all: Vec<&str> = TWELVE_CI.iter().flat_map(|c| c.mansions.iter().copied()).collect();
    assert_eq!(all.len(), 28, "整宿归次应覆盖全部二十八宿");
    all.sort_unstable();
    let mut uniq = all.clone();
    uniq.dedup();
    assert_eq!(uniq.len(), 28, "不该有宿被归进两个次");
    let mut canonical = MANSIONS.to_vec();
    canonical.sort_unstable();
    assert_eq!(all, canonical, "归次用的宿名应与本叶的二十八宿表一致");

    // 反查
    assert_eq!(ci_of_mansion("斗").map(|c| c.name), Some("星纪"));
    assert_eq!(ci_of_mansion("柳").map(|c| c.name), Some("鹑火"));
    assert_eq!(ci_of_mansion("觜").map(|c| c.name), Some("实沈"));
    assert!(ci_of_mansion("不存在之宿").is_none());
}

/// 《尔雅·释天》给的是标志宿而非次界，且**连十二个都没给全**——
/// 实沈 / 鹑首 / 鹑尾三个次名在《尔雅》全书零出现，玄枵 / 大梁 / 鹑火只给单宿标志。
/// 这条测试把「不能拿《尔雅》补全十二次」这个判断固定下来，防止日后有人照它改表。
#[test]
fn the_erya_marker_stars_are_a_subset_and_cannot_fill_the_table() {
    use erci::ci_of_mansion;
    // 《尔雅》能对上的（标志宿落在通行整宿表的同一个次里）
    for (mansion, ci) in [("角", "寿星"), ("斗", "星纪"), ("虚", "玄枵"), ("室", "娵訾"),
                          ("奎", "降娄"), ("昴", "大梁"), ("柳", "鹑火"), ("箕", "析木")] {
        assert_eq!(ci_of_mansion(mansion).map(|c| c.name), Some(ci), "《尔雅》{mansion} → {ci}");
    }
    // 《尔雅》「大辰 = 房心尾」与通行整宿表「大火 = 氐房心」不合：尾归析木
    assert_eq!(ci_of_mansion("尾").map(|c| c.name), Some("析木"), "尾在通行表归析木，非大火");
}

fn moment(y: i32, mo: u32, d: u32, h: u32, mi: u32, tz: f64) -> Moment {
    Moment::new(y, mo, d, h, mi, tz)
}

/// 十体顺序固定：7 七政在前 + 3 四余在后（紫炁 🟡 不入）。
#[test]
fn star_list_well_formed() {
    assert_eq!(STARS.len(), 10);
    assert!(STARS[..7].iter().all(|s| s.is_qizheng()));
    assert!(STARS[7..].iter().all(|s| !s.is_qizheng()));
    // 不含紫炁（诚实标 🟡）。Star enum 也无 Ziqi 变体。
}

/// 名表长度与 const。
#[test]
fn name_tables_len() {
    assert_eq!(SIGNS.len(), 12);
    assert_eq!(MANSIONS.len(), 28);
    assert_eq!(MANSION_OFFSET, 11);
}

/// `sign_of` 边界：0° = 白羊、30° = 金牛、359.999° = 双鱼，wrap 360 → 白羊。
#[test]
fn sign_of_boundaries() {
    assert_eq!(sign_of(0.0), 0);
    assert_eq!(sign_of(29.999), 0);
    assert_eq!(sign_of(30.0), 1);
    assert_eq!(sign_of(359.99), 11);
    assert_eq!(sign_of(360.0), 0);
    assert_eq!(sign_of(720.0), 0);
    assert_eq!(sign_of(-30.0), 11);
}

/// 28 宿值日：2026-06-14 = 昴（idx 17，与 zeri 校验值一致）。
#[test]
fn mansion_2026_06_14() {
    let jdn = mingli_astro::civil_day_number(2026, 6, 14);
    let i = mansion_for_jdn(jdn);
    assert_eq!(MANSIONS[i], "昴");
}

/// 性质：罗㬋/计都恒 180° 对宫。
#[test]
fn luohou_opposite_jidu() {
    for (y, mo, d) in [(2024, 1, 1), (1990, 6, 15), (2000, 1, 1)] {
        let m = moment(y, mo, d, 12, 0, 0.0);
        let c = compute_at(&m);
        let lo = c.stars.iter().find(|s| s.star == Star::Luohou).unwrap();
        let ji = c.stars.iter().find(|s| s.star == Star::Jidu).unwrap();
        let diff = ((ji.longitude - lo.longitude - 180.0 + 540.0).rem_euclid(360.0) - 180.0)
            .abs();
        assert!(diff < 1e-9, "{y}-{mo}-{d}： 罗/计非 180° 对宫 (diff={diff})");
    }
}

/// 性质：月孛黄经在 [0， 360) 且与月平近地点恒差 180°（已在 ephemeris 测过，这里走完整路径）。
#[test]
fn yuebo_in_range() {
    let m = moment(2024, 6, 15, 14, 30, 8.0);
    let c = compute_at(&m);
    let y = c.stars.iter().find(|s| s.star == Star::Yuebo).unwrap();
    assert!((0.0..360.0).contains(&y.longitude), "月孛越界： {}", y.longitude);
}

/// 1990-06-15 14：30 CST 七政四余完整排盘 oracle（全字段类型 + 值范围 + 关键值）。
///
/// - 太阳：在双子座（已由 astrology 校验）
/// - 日柱：辛亥（与 bazi/zeri 同源）
/// - 罗㬋 ≈ Ω(1990-06-15) ≈ 月升交点位置（逆行约 250°/yr 自 J2000 起 ~10 年）
/// - 月孛 ≈ Π(1990-06-15)+180° ≈ 月远地点位置（顺行约 40°/yr）
#[test]
fn sample_1990_06_15_full() {
    let m = moment(1990, 6, 15, 14, 30, 8.0);
    let c = compute_at(&m);

    assert_eq!(c.stars.len(), 10);
    for sp in &c.stars {
        assert!((0.0..360.0).contains(&sp.longitude));
        assert!(sp.sign < 12);
        assert!((0.0..30.0).contains(&sp.degree_in_sign));
        assert_eq!(sp.sign_name, SIGNS[sp.sign]);
    }

    // 太阳在双子（与 astrology 校验值一致）。
    let sun = c.stars.iter().find(|s| s.star == Star::Sun).unwrap();
    assert_eq!(sun.sign_name, "双子", "太阳座 {} @ {:.2}°", sun.sign_name, sun.longitude);

    // 日柱辛亥（与 bazi 校验一致）。
    assert_eq!(c.day_ganzhi, "辛亥");
    // 28 宿值日落在某一宿。
    assert!(MANSIONS.contains(&c.mansion_name));
}

/// `compute` 入口与 `compute_at` 等价。
#[test]
fn compute_equals_compute_at() {
    let a = compute(1990, 6, 15, 14, 30, 8.0);
    let m = moment(1990, 6, 15, 14, 30, 8.0);
    let b = compute_at(&m);
    assert_eq!(a.stars.len(), b.stars.len());
    for (x, y) in a.stars.iter().zip(b.stars.iter()) {
        assert!((x.longitude - y.longitude).abs() < 1e-12);
    }
    assert_eq!(a.mansion, b.mansion);
    assert_eq!(a.day_ganzhi, b.day_ganzhi);
}

/// `Star::chinese_name` 全 10 变体唯一且非空。
#[test]
fn chinese_names_unique() {
    let mut names: Vec<&str> = STARS.iter().map(|s| s.chinese_name()).collect();
    names.sort_unstable();
    let n = names.len();
    names.dedup();
    assert_eq!(names.len(), n, "中文名有重复");
    assert!(names.iter().all(|n| !n.is_empty()));
}

/// `is_qizheng` 与 STARS 切片划分一致。
#[test]
fn is_qizheng_partition() {
    for &s in &STARS[..7] {
        assert!(s.is_qizheng(), "{s:?} 应属七政");
    }
    for &s in &STARS[7..] {
        assert!(!s.is_qizheng(), "{s:?} 应属四余");
    }
}

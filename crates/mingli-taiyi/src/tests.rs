//! 太乙神数的校验：积年、行宫、二目与诸将。

use super::*;

#[test]
fn jinian_epoch_anchor() {
    // 《太乙金镜式经》：724 CE = 积年 1_937_281。
    assert_eq!(accumulated_years(724), 1_937_281);
    // 线性外推：相邻年差 1。
    assert_eq!(accumulated_years(725), 1_937_282);
    assert_eq!(accumulated_years(723), 1_937_280);
}

#[test]
fn palaces_skip_center_and_are_eight() {
    assert_eq!(PALACES_8.len(), 8);
    assert!(!PALACES_8.contains(&5), "太乙不入中五");
    let set: std::collections::HashSet<u8> = PALACES_8.iter().copied().collect();
    assert_eq!(set.len(), 8);
    // 顺行序：一宫起，1→2→3→4→6→7→8→9。
    assert_eq!(PALACES_8[0], 1);
    assert_eq!(PALACES_8[7], 9);
}

#[test]
fn taiyi_dwells_three_years_per_palace_cycle_24() {
    // 三年居一宫、廿四年转一周：同一阳遁下，每 3 年宫序 +1，24 年回到原宫。
    for base in 0..24i64 {
        let p0 = taiyi_palace(base, true);
        // 入宫年数 1..3，三才随之。
        assert!((1..=3).contains(&p0.year_in_palace));
        assert_eq!(p0.sancai, SANCAI[(p0.year_in_palace - 1) as usize]);
        // 同宫连居三年。
        let same = taiyi_palace(base - (base % 3), true);
        let same2 = taiyi_palace(base - (base % 3) + 2, true);
        assert_eq!(same.palace, same2.palace, "同宫三年应同宫");
        // 24 年后复位。
        assert_eq!(taiyi_palace(base + 24, true).palace, p0.palace);
    }
    // 八宫在 24 年内恰好各被走到一次。
    let set: std::collections::HashSet<u8> = (0..24).step_by(3).map(|r| taiyi_palace(r, true).palace).collect();
    assert_eq!(set.len(), 8);
}

#[test]
fn yin_dun_is_mirror_of_yang() {
    // 阴遁逆行：九宫起，与阳遁同步序号镜像。
    assert_eq!(taiyi_palace(0, false).palace, 9); // 阴遁一步起九宫
    for r in 0..24i64 {
        let y = taiyi_palace(r, true);
        let n = taiyi_palace(r, false);
        assert_eq!(n.palace, PALACES_8[7 - y.step as usize]);
        // 太乙恒不入中五。
        assert_ne!(y.palace, 5);
        assert_ne!(n.palace, 5);
    }
}

#[test]
fn yang_yin_dun_by_solar_term() {
    // 冬至(λ=270)后阳遁、夏至(λ=90)后阴遁。
    assert!(is_yang_dun(270.0)); // 冬至
    assert!(is_yang_dun(0.0)); // 春分仍阳遁段
    assert!(!is_yang_dun(90.0)); // 夏至
    assert!(!is_yang_dun(180.0)); // 秋分阴遁段
    // 全 360° 各半。
    let yang = (0..360).filter(|&d| is_yang_dun(f64::from(d))).count();
    assert_eq!(yang, 180);
}

#[test]
fn sixteen_gods_framework() {
    assert_eq!(SIXTEEN_DIRECTIONS.len(), 16);
    assert_eq!(SIXTEEN_GODS.len(), 16);
    // 十六方位含十二地支 + 四维卦。
    for s in ["子", "午", "卯", "酉", "艮", "巽", "坤", "乾"] {
        assert!(SIXTEEN_DIRECTIONS.contains(&s), "缺方位 {s}");
    }
    assert_eq!(SIXTEEN_GODS[0], "地主"); // 子神
}

#[test]
fn compute_is_deterministic_and_palace_valid() {
    let c = compute(2024, 6, 15, 8.0);
    assert_eq!(c.jinian, accumulated_years(2024));
    assert!((1..=9).contains(&c.taiyi.palace) && c.taiyi.palace != 5);
    // 卦名走太乙自家的九宫配法，不是洛书——见 `taiyi_numbers_its_palaces_differently_from_the_luoshu`
    assert_eq!(c.taiyi.gua, PALACE_GUA[c.taiyi.palace as usize]);
    let c2 = compute(2024, 6, 15, 8.0);
    assert_eq!(c.taiyi.palace, c2.taiyi.palace);
    assert_eq!(c.taiyi.sancai, c2.taiyi.sancai);
}

mod generals {
    use super::*;

    /// 两则原典纪年实例，相隔一千一百年、出自两部不同的书，全字段吻合。
    ///
    /// 《太乙统宗宝鉴》卷四：「唐昭宗天復二年壬戍嵗……壬子元第十一局，太乙在四宫，
    /// 文昌在髙叢……始擊在陽徳，客筭得四」；
    /// 「秦二世二年甲午歲……庚子元第五十五局，太乙在三宫，始擊臨和徳……文昌在武徳」。
    /// 两者与《太乙金镜式经》卷三的立成表逐字吻合，是两部书之间的交叉见证。
    ///
    /// 这一条同时钉住：积年锚点、入局数、太乙行宫、文昌周法十八、计神逆行十二辰、
    /// 「计神加和德看文昌」的方向、以及主客算的三条边界。任一处错都会红。
    #[test]
    fn two_dated_examples_from_the_sources_agree_on_every_field() {
        /// 一则纪年实例：年 / 局 / 太乙宫 / 文昌 / 计神 / 始击 / 客算（原文未载则 `None`）。
        struct Example {
            year: i64,
            ju: i64,
            palace: u8,
            wenchang: &'static str,
            jishen: &'static str,
            shiji: &'static str,
            ke_suan: Option<u32>,
        }
        const ORACLE: [Example; 2] = [
            Example { year: 902, ju: 11, palace: 4, wenchang: "高丛", jishen: "辰", shiji: "阳德", ke_suan: Some(4) },
            Example { year: -206, ju: 55, palace: 3, wenchang: "武德", jishen: "申", shiji: "和德", ke_suan: None },
        ];
        for Example { year, ju: want_ju, palace: want_palace, wenchang: want_wc, jishen: want_js, shiji: want_sj, ke_suan: want_ke } in ORACLE {
            let j = accumulated_years(year);
            // 用库里那份，不再在测试里抄一遍——抄一遍就等于不验它。
            let ju = crate::ju_of(j);
            assert_eq!(ju, want_ju, "{year} 年应是第 {want_ju} 局");
            let t = taiyi_palace(j, true);
            assert_eq!(t.palace, want_palace, "{year} 年太乙应在 {want_palace} 宫");
            let wc = wenchang(j, true);
            assert_eq!(SIXTEEN_GODS[wc], want_wc, "{year} 年文昌");
            let js = jishen(ju, true);
            assert_eq!(SIXTEEN_DIRECTIONS[js], want_js, "{year} 年计神");
            let sj = shiji(wc, js);
            assert_eq!(SIXTEEN_GODS[sj], want_sj, "{year} 年始击");
            if let Some(k) = want_ke {
                assert_eq!(suan(sj, t.palace), k, "{year} 年客算");
            }
        }
    }

    /// 三个纪年锚点落在同一个 72 局环上、互相自洽。
    ///
    /// 《金镜式经》「開元十二年甲子＝局 49」、《统宗》「天復二年＝局 11」「秦二世二年＝局 55」。
    /// 724 → 902 相隔 178 年，49 + 178 ≡ 11 (mod 72)。
    #[test]
    fn the_three_anchors_land_on_one_consistent_cycle() {
        let ju = |y: i64| {
            let r = accumulated_years(y).rem_euclid(72);
            if r == 0 { 72 } else { r }
        };
        assert_eq!(ju(724), 49, "开元十二年应是第 49 局");
        assert_eq!(ju(902), 11);
        assert_eq!(ju(-206), 55);
        assert_eq!((49 + 178) % 72, 11, "724→902 相隔 178 年，局序自洽");
    }

    /// 文昌周法十八：两个双计位各占两算，绕一圈恰 18 算。
    #[test]
    fn the_wenchang_cycle_is_eighteen_because_two_positions_count_twice() {
        for yang in [true, false] {
            let seen: Vec<usize> = (1..=18).map(|j| wenchang(j, yang)).collect();
            // 18 算走遍 16 神位，其中两位各占两算
            let mut uniq = seen.clone();
            uniq.sort_unstable();
            uniq.dedup();
            assert_eq!(uniq.len(), 16, "十八算应走遍十六神位");
            // 第 19 算回到起点
            assert_eq!(wenchang(19, yang), wenchang(1, yang), "周法十八，第十九算回起点");
            let doubled: Vec<usize> = (0..16)
                .filter(|k| seen.iter().filter(|s| *s == k).count() == 2)
                .collect();
            let want = if yang { vec![10, 14] } else { vec![2, 6] };
            assert_eq!(doubled, want, "阳遁双计大武阴德，阴遁双计和德大炅");
        }
        // 阳遁起武德、阴遁起吕申
        assert_eq!(SIXTEEN_GODS[wenchang(1, true)], "武德");
        assert_eq!(SIXTEEN_GODS[wenchang(1, false)], "吕申");
    }

    /// 计神逆行十二辰，不入四维；阳起寅、阴起申。
    #[test]
    fn the_reckoning_spirit_runs_backwards_through_the_twelve_branches() {
        assert_eq!(SIXTEEN_DIRECTIONS[jishen(1, true)], "寅");
        assert_eq!(SIXTEEN_DIRECTIONS[jishen(1, false)], "申");
        // 逆行：第二局退到丑
        assert_eq!(SIXTEEN_DIRECTIONS[jishen(2, true)], "丑");
        // 十二局走遍十二辰，绝不落四维
        for yang in [true, false] {
            let mut seen: Vec<&str> =
                (1..=12).map(|j| SIXTEEN_DIRECTIONS[jishen(j, yang)]).collect();
            for d in &seen {
                assert!(!["艮", "巽", "坤", "乾"].contains(d), "计神不入四维，实得 {d}");
            }
            seen.sort_unstable();
            seen.dedup();
            assert_eq!(seen.len(), 12);
            assert_eq!(jishen(13, yang), jishen(1, yang), "十二局一周");
        }
    }

    /// 主客算的三条边界：起点为间神计 1、间神不累加、终点太乙宫不计入。
    #[test]
    fn the_reckoning_counts_only_proper_palaces_and_stops_before_taiyi() {
        // 起点即压太乙宫 → 径取该宫数
        assert_eq!(suan(4, 4), 4, "高丛正宫 4，恰压太乙 4 宫");
        // 局 11 的客算：阳德(间神,计1) → 和德(宫3) → 吕申(间神,不计) → 高丛=太乙宫，止
        assert_eq!(suan(1, 4), 4);
        // 无论起点与太乙宫如何，算数恒为正且有限
        for from in 0..16 {
            for t in [1, 2, 3, 4, 6, 7, 8, 9] {
                assert!(suan(from, t) > 0);
            }
        }
    }

    /// 去十用零与三因取参将。整十者以九去之，是《统宗》明写的特例。
    #[test]
    fn dropping_the_tens_treats_exact_tens_by_nine() {
        assert_eq!(qu_shi(4), 4);
        assert_eq!(qu_shi(16), 6);
        assert_eq!(qu_shi(25), 5);
        assert_eq!(qu_shi(10), 1, "整十以九去之：10 % 9 = 1");
        assert_eq!(qu_shi(20), 2);
        assert_eq!(qu_shi(30), 3);
        assert_eq!(qu_shi(90), 9, "90 % 9 = 0 → 归 9");
        // 参将 = 大将三因后再去十
        assert_eq!(can_jiang(4), 2, "4×3=12 → 2");
        assert_eq!(can_jiang(5), 5, "5×3=15 → 5，中五特例：诸将可落中五");
        for d in 1..=9u8 {
            assert!((1..=9).contains(&can_jiang(d)));
        }
    }

    /// 太乙的九宫配法与洛书不同——宫 1 是乾不是坎。
    ///
    /// 这不是命名口味问题：主客算沿环累加宫数，配法错了算数就错。
    /// 局 11 客算得 4 只在太乙配法下成立（阳德计 1 + 和德艮 3 = 4，止于高丛卯 = 4 宫）。
    #[test]
    fn taiyi_numbers_its_palaces_differently_from_the_luoshu() {
        assert_eq!(PALACE_GUA[1], "乾");
        assert_eq!(mingli_luoshu::PALACE_NAME[1], "坎", "洛书的一宫是坎，两套不可混用");
        assert_eq!(PALACE_GUA[4], "震");
        assert_eq!(PALACE_GUA[9], "巽");
        assert_eq!(PALACE_GUA[5], "中");
        // 十六神环上八正宫恰配八卦宫数，八间神无宫
        let with_palace: Vec<u8> = RING_PALACE.iter().flatten().copied().collect();
        let mut sorted = with_palace.clone();
        sorted.sort_unstable();
        assert_eq!(sorted, vec![1, 2, 3, 4, 6, 7, 8, 9], "八正宫恰是不含中五的八宫");
        for (k, p) in RING_PALACE.iter().enumerate() {
            assert_eq!(p.is_some(), k.is_multiple_of(2), "偶数位正宫、奇数位间神");
        }
    }
}

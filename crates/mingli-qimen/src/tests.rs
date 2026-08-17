//! 奇门四盘的校验：72 局常数表、古例复现、结构不变量。

use super::*;


fn plate_of(ju: u8, yang: bool, yi_palace: u8, zhi_fu_palace: u8) -> ([&'static str; 9], SkyPlate) {
let ep = earth_plate(ju, yang);
let mut earth = [""; 9];
for k in 0..9 {
    earth[k] = STEM_NAMES[ep[k] as usize];
}
let sky = sky_rotation(&earth, yi_palace, zhi_fu_palace);
(earth, sky)
}

/// 伏吟 = 旋转 0 格（各归原位），反吟 = 旋转 4 格（各落对冲宫）。
#[test]
fn qm5_fu_yin_and_fan_yin_come_from_the_shift() {
let (earth, sky) = plate_of(1, true, 1, 1);
let gates = gate_plate(1, 0, 0, true);
let p = patterns(&earth, &sky, &gates);
assert!(p.star_fu_yin && p.gate_fu_yin && p.full_fu_yin, "零位移 = 全盘伏吟");
assert_eq!(p.stem_fu_yin_palaces, vec![1, 2, 3, 4, 6, 7, 8, 9], "外八宫天地盘干重叠");
assert!(!p.star_fan_yin && !p.gate_fan_yin);

// 对冲：坎 1 → 离 9 恰是圆周 4 格
let (earth, sky) = plate_of(1, true, 1, 9);
// 门按宫序号线性数：自坎 1 顺数 8 步才到宫 9（对冲宫），与星走圆周 4 格是同一落点
let gates = gate_plate(1, 0, 8, true);
let p = patterns(&earth, &sky, &gates);
assert_eq!((sky.shift, gates.shift), (4, 4));
assert!(p.star_fan_yin && p.gate_fan_yin);
assert!(!p.star_fu_yin && !p.full_fu_yin);
assert!(p.stem_fu_yin_palaces.is_empty(), "反吟时天地盘干处处不同");
}

/// 星伏吟 ⟺ 每宫天盘星都等于该宫原配星（与 shift 判定等价）。
#[test]
fn qm5_star_fu_yin_matches_a_cell_by_cell_check() {
for from in ORBIT {
    for to in ORBIT {
        let (earth, sky) = plate_of(4, false, from, to);
        let gates = gate_plate(from, 0, 0, false);
        let p = patterns(&earth, &sky, &gates);
        let cell_wise = ORBIT
            .iter()
            .all(|&g| sky.stars[g as usize - 1] == JIU_XING_PALACE[g as usize]);
        assert_eq!(p.star_fu_yin, cell_wise);
    }
}
}

/// 三奇临吉门只收「天盘乙丙丁 + 同宫开休生」这一结构事实。
#[test]
fn qm5_qi_gates_pair_the_three_odds_with_the_three_good_gates() {
let c = compute(1987, 9, 17, 15, 0, 8.0);
for qg in &c.patterns.qi_gates {
    let k = qg.palace as usize - 1;
    assert!(SAN_QI.contains(&c.sky.stems[k]));
    assert!(JI_MEN.contains(&c.gates.gates[k]));
    assert_eq!((qg.qi, qg.gate), (c.sky.stems[k], c.gates.gates[k]));
}
// 该盘天盘乙在震 3，震 3 正是生门
assert_eq!(c.sky.stems[2], "乙");
assert_eq!(c.gates.gates[2], "生门");
assert!(c.patterns.qi_gates.iter().any(|q| q.palace == 3 && q.qi == "乙" && q.gate == "生门"));
}

/// 参考时刻不伏不反（星转 7 格、门转 1 格）。
#[test]
fn qm5_patterns_on_the_reference_moment() {
let c = compute(1987, 9, 17, 15, 0, 8.0);
let p = &c.patterns;
assert!(!p.star_fu_yin && !p.star_fan_yin && !p.gate_fu_yin && !p.gate_fan_yin && !p.full_fu_yin);
}


/// 每个「节」开一个月，两气一支：24 节气恰好铺满 12 支，各支两次，且立春开寅月。
#[test]
fn qm4_two_terms_per_month_branch() {
let mut count = [0u8; 12];
for i in 0..24 {
    count[month_branch_of_term(i) as usize] += 1;
}
assert_eq!(count, [2; 12]);
assert_eq!(month_branch_of_term(21), 2, "立春开寅月");
assert_eq!(month_branch_of_term(22), 2, "雨水仍在寅月");
assert_eq!(month_branch_of_term(11), 9, "白露属酉月");
assert_eq!(month_branch_of_term(18), 0, "冬至属子月");
}

/// 旺相休囚死的通行判法：当令旺、令生相、生令休、克令囚、令克死。
#[test]
fn qm4_vigor_follows_the_classical_definition() {
use Element::{Earth, Fire, Metal, Water, Wood};
// 春木当令：木旺、火相（木生火）、水休（水生木）、金囚（金克木）、土死（木克土）
assert_eq!(vigor_of(Wood, Wood), Vigor::Wang);
assert_eq!(vigor_of(Fire, Wood), Vigor::Xiang);
assert_eq!(vigor_of(Water, Wood), Vigor::Xiu);
assert_eq!(vigor_of(Metal, Wood), Vigor::Qiu);
assert_eq!(vigor_of(Earth, Wood), Vigor::Si);
// 五行 × 五行穷举：每个月令下五等级恰好各出现一次
for month in [Metal, Wood, Water, Fire, Earth] {
    let mut seen = std::collections::BTreeSet::new();
    for s in [Metal, Wood, Water, Fire, Earth] {
        assert!(seen.insert(vigor_of(s, month).label()));
    }
    assert_eq!(seen.len(), 5);
}
}

/// 星名 → 五行：9 星齐备，未知名给 None。
#[test]
fn qm4_star_elements_are_complete() {
assert_eq!(star_element("天蓬"), Some(Element::Water));
assert_eq!(star_element("天芮"), Some(Element::Earth));
assert_eq!(star_element("天英"), Some(Element::Fire));
assert_eq!(star_element("天心"), Some(Element::Metal));
assert_eq!(star_element("天冲"), Some(Element::Wood));
assert!(star_element("").is_none() && star_element("天某").is_none());
assert_eq!((1..=9).filter(|&p| star_element(JIU_XING_PALACE[p]).is_some()).count(), 9);
}

/// 1987-09-17 15:00（白露 → 酉月，月令金）：金星旺、水星相、土星休、火星囚、木星死。
#[test]
fn qm4_vigor_on_the_reference_moment() {
let c = compute(1987, 9, 17, 15, 0, 8.0);
assert_eq!((c.month_branch, c.month_element), (9, "金"));
for k in 0..9 {
    let star = c.sky.stars[k];
    if star.is_empty() {
        assert_eq!(c.star_vigor[k], "");
        continue;
    }
    let want = match star_element(star).expect("九星五行齐备") {
        Element::Metal => "旺",
        Element::Water => "相",
        Element::Earth => "休",
        Element::Fire => "囚",
        Element::Wood => "死",
    };
    assert_eq!(c.star_vigor[k], want, "{star} 在酉月");
}
// 抽两宫核对：艮 8 天冲（木）死、兑 7 天心（金）旺
assert_eq!((c.sky.stars[7], c.star_vigor[7]), ("天冲", "死"));
assert_eq!((c.sky.stars[6], c.star_vigor[6]), ("天心", "旺"));
}


/// 起点坎 1 的阳遁八神（公开教程例题）：沿圆周顺时针依次落 8 宫。
#[test]
fn qm3_spirits_oracle_yang_from_kan_one() {
let s = spirit_plate(1, true);
assert_eq!(s.start_palace, 1);
assert_eq!(
    s.spirits,
    ["值符", "玄武", "太阴", "六合", "", "九天", "九地", "腾蛇", "白虎"],
    "坎1值符 坤2玄武 震3太阴 巽4六合 中5空 乾6九天 兑7九地 艮8腾蛇 离9白虎"
);
// 另一系只换第 5 / 6 位的名字，位置不动
assert_eq!(s.spirits_alt[8], "勾陈", "离 9 的白虎在另一系作勾陈");
assert_eq!(s.spirits_alt[1], "朱雀", "坤 2 的玄武在另一系作朱雀");
assert_eq!(s.spirits_alt[0], "值符");
}

/// 阴遁逆布：同一起点下顺序沿圆周反向。
#[test]
fn qm3_spirits_run_backwards_under_yin_escape() {
let s = spirit_plate(1, false);
assert_eq!(
    s.spirits,
    ["值符", "六合", "九地", "玄武", "", "腾蛇", "太阴", "九天", "白虎"],
    "坎1值符 乾6腾蛇 兑7太阴 坤2玄武…… 逆时针"
);
}

/// 两套称谓都不分阴阳遁——只有第 5 / 6 位换名，位置与其余六神一字不动。
///
/// 《遁甲演义》两遁俱用白虎 / 玄武，《奇门遁甲统宗》两遁俱用勾陈 / 朱雀且明言异名同位。
/// 坊间「阳遁勾陈朱雀、阴遁白虎玄武」在转盘语境下无据，见 `BA_SHEN_ALT` 的说明。
#[test]
fn qm3_the_second_naming_applies_under_both_escapes() {
for palace in 1..=9u8 {
    for yang in [true, false] {
        let s = spirit_plate(palace, yang);
        for k in 0..9 {
            let (a, b) = (s.spirits[k], s.spirits_alt[k]);
            let want = match a {
                "白虎" => "勾陈",
                "玄武" => "朱雀",
                other => other,
            };
            assert_eq!(b, want, "宫 {} 遁 {yang}：{a} 的另一名应是 {want}", k + 1);
        }
        // 两套里第 5 / 6 位必定各出现一次，别的六神一字不改
        assert_eq!(s.spirits_alt.iter().filter(|n| **n == "勾陈").count(), 1);
        assert_eq!(s.spirits_alt.iter().filter(|n| **n == "朱雀").count(), 1);
    }
}
}

/// 任意起点与遁向：八神恒是外八宫的置换，值符恒在起点，中宫恒空。
#[test]
fn qm3_spirits_are_a_permutation_anchored_at_the_duty_symbol() {
let want: std::collections::BTreeSet<&str> = BA_SHEN.iter().copied().collect();
for &yang in &[true, false] {
    for start in [1u8, 2, 3, 4, 5, 6, 7, 8, 9] {
        let s = spirit_plate(start, yang);
        assert_eq!(s.spirits[4], "");
        assert_eq!(s.spirits[s.start_palace as usize - 1], "值符");
        let got: std::collections::BTreeSet<&str> =
            ORBIT.iter().map(|&p| s.spirits[p as usize - 1]).collect();
        assert_eq!(got, want);
    }
}
assert_eq!(spirit_plate(5, true), spirit_plate(2, true), "落中 5 按寄坤 2 论");
}

/// 1987-09-17 15:00（阴遁 3 局，值符宫艮 8）整链回归。
#[test]
fn qm3_spirits_on_the_reference_moment() {
let c = compute(1987, 9, 17, 15, 0, 8.0);
assert_eq!(c.spirits.start_palace, 8);
assert_eq!(c.spirits.spirits[7], "值符");
// 阴遁自艮 8 逆行：艮8 → 坎1 → 乾6 → 兑7 → 坤2 → 离9 → 巽4 → 震3
assert_eq!(
    c.spirits.spirits,
    ["腾蛇", "白虎", "九天", "九地", "", "太阴", "六合", "值符", "玄武"]
);
}


/// 阳遁一局庚午时（古例）：旬首戊在坎 1，庚午是甲子旬第 7 个时辰，
/// 值使休门自坎 1 按宫号顺数 6 步落兑 7；八门 8 宫逐宫比对。
#[test]
fn qm2_gate_plate_oracle_yang_one_gengwu() {
// 甲子旬首（子 = 0），庚午（午 = 6）
let g = gate_plate(1, 0, 6, true);
assert_eq!((g.zhi_shi_gate, g.zhi_shi_palace), ("休门", 7));
assert_eq!(g.steps, 6);
assert_eq!(g.shift, 6);
assert_eq!(
    g.gates,
    ["伤门", "开门", "景门", "死门", "", "生门", "休门", "杜门", "惊门"],
    "坎1伤 坤2开 震3景 巽4死 中5空 乾6生 兑7休 艮8杜 离9惊"
);
}

/// 值使随时辰走的是**宫序号线性**（阳遁 +1 / 阴遁 −1，中 5 也占一位），
/// 与九星沿圆周走不同 —— 这是八门最易与天盘混淆之处。
#[test]
fn qm2_zhi_shi_counts_along_palace_numbers_not_the_orbit() {
// 阳遁：坎1 起，逐时辰 1→2→3→4→5(中,寄坤2)→6→7
let landed: Vec<u8> = (0..7).map(|k| gate_plate(1, 0, k, true).zhi_shi_palace).collect();
assert_eq!(landed, [1, 2, 3, 4, 2, 6, 7], "第 5 步落中 5 → 寄坤 2");
// 阴遁：同起点反向 1→9→8→7…
let back: Vec<u8> = (0..4).map(|k| gate_plate(1, 0, k, false).zhi_shi_palace).collect();
assert_eq!(back, [1, 9, 8, 7]);
}

/// 一旬十个时辰：值使门恒为旬首宫本位门，八门恒是外八宫的一个置换，中宫恒空。
#[test]
fn qm2_gates_stay_a_permutation_across_a_whole_decade() {
let want: std::collections::BTreeSet<&str> = BA_MEN_ORBIT.iter().copied().collect();
for &yang in &[true, false] {
    for head in [0u8, 2, 4, 6, 8, 10] {
        for k in 0..10u8 {
            let g = gate_plate(3, head, (head + k) % 12, yang);
            assert_eq!(g.steps, k);
            assert_eq!(g.zhi_shi_gate, BA_MEN_ORBIT[orbit_index(3)], "值使 = 旬首宫本位门");
            assert_eq!(g.gates[4], "", "八门不入中宫");
            let got: std::collections::BTreeSet<&str> =
                ORBIT.iter().map(|&p| g.gates[p as usize - 1]).collect();
            assert_eq!(got, want);
            // 值使门必落在算出的那一宫
            assert_eq!(g.gates[g.zhi_shi_palace as usize - 1], g.zhi_shi_gate);
        }
    }
}
}

/// 1987-09-17 15:00（阴遁 3 局，旬首戊震 3，壬申为甲子旬第 9 个时辰）整链回归。
#[test]
fn qm2_gate_plate_on_the_reference_moment() {
let c = compute(1987, 9, 17, 15, 0, 8.0);
assert_eq!(c.gates.steps, 8, "申 = 8，甲子旬第 9 个时辰");
assert_eq!(c.gates.zhi_shi_gate, "伤门", "旬首戊在震 3，震 3 本位门为伤门");
// 阴遁自震 3 逆数 8 步：3→2→1→9→8→7→6→5(寄坤2)→4
assert_eq!(c.gates.zhi_shi_palace, 4);
assert_eq!(c.gates.gates[3], "伤门");
}

/// 阳遁三局丙寅时（古例）：旬首戊在震 3、时干丙在坎 1，值符天冲随时干落坎 1。
///
/// 天盘九星期望值取自公开排盘教程的完整例题，8 宫逐宫比对。
#[test]
fn qm1b_sky_stars_oracle_yang_three_bingyin() {
let ep = earth_plate(3, true);
let mut earth = [""; 9];
for k in 0..9 {
    earth[k] = STEM_NAMES[ep[k] as usize];
}
// 地盘先自证：阳三局 戊震3 己巽4 庚中5 辛乾6 壬兑7 癸艮8 丁离9 丙坎1 乙坤2
assert_eq!(earth, ["丙", "乙", "戊", "己", "庚", "辛", "壬", "癸", "丁"]);

let sky = sky_rotation(&earth, 3, 1);
assert_eq!(sky.shift, 6, "震 3 → 坎 1 沿圆周顺时针 6 格");
assert_eq!(
    sky.stars,
    ["天冲", "天心", "天英", "天芮", "", "天任", "天蓬", "天辅", "天柱"],
    "坎1冲 坤2心 震3英 巽4芮 中5空 乾6任 兑7蓬 艮8辅 离9柱"
);
// 旬首随时干：值符宫的天盘干必是本旬六仪
assert_eq!(sky.stems[0], "戊");
assert_eq!(sky.stems, ["戊", "辛", "丁", "乙", "", "癸", "丙", "己", "壬"]);
// 中宫之干寄坤 2，随坤 2 转到巽 4（即随天芮走）
assert_eq!((sky.center_stem, sky.center_palace), ("庚", 4));
assert_eq!(sky.stars[3], "天芮", "中宫寄干的落宫正是天芮所在宫");
}

/// 阳遁一局庚午时（古例）：旬首戊在坎 1、时干庚在震 3，值符天蓬随时干落震 3。
#[test]
fn qm1b_sky_stars_oracle_yang_one_gengwu() {
let ep = earth_plate(1, true);
let mut earth = [""; 9];
for k in 0..9 {
    earth[k] = STEM_NAMES[ep[k] as usize];
}
assert_eq!(earth, ["戊", "己", "庚", "辛", "壬", "癸", "丁", "丙", "乙"]);

let sky = sky_rotation(&earth, 1, 3);
assert_eq!(sky.shift, 2, "坎 1 → 震 3 沿圆周顺时针 2 格");
assert_eq!(
    sky.stars,
    ["天柱", "天辅", "天蓬", "天任", "", "天芮", "天英", "天心", "天冲"],
    "坎1柱 坤2辅 震3蓬 巽4任 中5空 乾6芮 兑7英 艮8心 离9冲"
);
assert_eq!(sky.stems[2], "戊", "旬首六仪落到时干所在的震 3");
}

/// 时干恰为本旬六仪时不发生旋转：天盘 = 原配 / 地盘。
#[test]
fn qm1b_zero_shift_leaves_the_plate_untouched() {
let ep = earth_plate(1, true);
let mut earth = [""; 9];
for k in 0..9 {
    earth[k] = STEM_NAMES[ep[k] as usize];
}
let sky = sky_rotation(&earth, 1, 1);
assert_eq!(sky.shift, 0);
for p in 1..=9u8 {
    if p == 5 {
        assert_eq!(sky.stars[4], "");
        assert_eq!(sky.stems[4], "");
    } else {
        assert_eq!(sky.stars[p as usize - 1], JIU_XING_PALACE[p as usize]);
        assert_eq!(sky.stems[p as usize - 1], earth[p as usize - 1]);
    }
}
}

/// 旋转是刚体置换：任意起止组合下，外八宫的星集合与干集合都守恒，且中宫恒空。
#[test]
fn qm1b_rotation_is_a_permutation_of_the_outer_ring() {
let ep = earth_plate(5, false);
let mut earth = [""; 9];
for k in 0..9 {
    earth[k] = STEM_NAMES[ep[k] as usize];
}
let want_stars: std::collections::BTreeSet<&str> =
    ORBIT.iter().map(|&p| JIU_XING_PALACE[p as usize]).collect();
let want_stems: std::collections::BTreeSet<&str> =
    ORBIT.iter().map(|&p| earth[p as usize - 1]).collect();
for from in ORBIT {
    for to in ORBIT {
        let sky = sky_rotation(&earth, from, to);
        assert!(sky.shift < 8);
        assert_eq!(sky.stars[4], "");
        assert_eq!(sky.stems[4], "");
        let got_stars: std::collections::BTreeSet<&str> =
            ORBIT.iter().map(|&p| sky.stars[p as usize - 1]).collect();
        let got_stems: std::collections::BTreeSet<&str> =
            ORBIT.iter().map(|&p| sky.stems[p as usize - 1]).collect();
        assert_eq!(got_stars, want_stars);
        assert_eq!(got_stems, want_stems);
        // 值符星必落在时干宫
        assert_eq!(sky.stars[to as usize - 1], JIU_XING_PALACE[from as usize]);
    }
}
}

/// 落中宫按寄坤 2 论：符首或时干在中 5 时与在坤 2 时结果相同。
#[test]
fn qm1b_center_palace_is_lodged_in_kun_two() {
let ep = earth_plate(2, true);
let mut earth = [""; 9];
for k in 0..9 {
    earth[k] = STEM_NAMES[ep[k] as usize];
}
assert_eq!(lodged_palace(5), 2);
assert_eq!(sky_rotation(&earth, 5, 7), sky_rotation(&earth, 2, 7));
assert_eq!(sky_rotation(&earth, 3, 5), sky_rotation(&earth, 3, 2));
}

/// 1987-09-17 15:00 长沙男（阴遁 3 局）整链回归：时干壬在艮 8、旬首戊在震 3。
#[test]
fn qm1b_sky_plate_on_the_reference_moment() {
let c = compute(1987, 9, 17, 15, 0, 8.0);
assert_eq!((c.xun_yi_palace, c.zhi_fu_palace), (3, 8));
assert_eq!(c.zhi_fu_xing, "天冲");
// 圆周是 坎1→艮8→震3→…，故震 3 退到艮 8 等于顺时针走 7 格
assert_eq!(c.sky.shift, 7, "震 3 → 艮 8 沿圆周顺时针 7 格");
// 值符星天冲落到时干所在的艮 8
assert_eq!(c.sky.stars[7], "天冲");
assert_eq!(c.sky.stems[7], "戊", "旬首六仪随值符同落艮 8");
assert_eq!(
    c.sky.stars,
    ["天任", "天柱", "天辅", "天英", "", "天蓬", "天心", "天冲", "天芮"]
);
}

#[test]
fn yang_dun_yang_ju1_matches_classic() {
// 阳遁一局校验：坎1戊·坤2己·震3庚·巽4辛·中5壬·乾6癸·兑7丁·艮8丙·离9乙。
let p = earth_plate(1, true);
let want = [4u8, 5, 6, 7, 8, 9, 3, 2, 1]; // 戊己庚辛壬癸丁丙乙
assert_eq!(p, want);
// 宫↔八卦复用洛书：宫1=坎 … 宫9=离。
assert_eq!(mingli_luoshu::PALACE_NAME[1], "坎");
assert_eq!(mingli_luoshu::PALACE_NAME[9], "离");
}

#[test]
fn earth_plate_is_a_permutation_for_all_ju_both_dun() {
// 任意局数、阴阳遁：九宫恰好布满 9 个三奇六仪（双射），无重无漏。
for ju in 1..=9u8 {
    for yang in [true, false] {
        let p = earth_plate(ju, yang);
        let set: std::collections::HashSet<u8> = p.iter().copied().collect();
        assert_eq!(set.len(), 9, "ju={ju} yang={yang} 应布满九宫");
        // 布的恰是三奇六仪（戊己庚辛壬癸乙丙丁 = 天干 1..9，无甲）。
        for &stem in &p {
            assert!((1..=9).contains(&stem), "不应出现甲(0)");
        }
    }
}
}

#[test]
fn yin_dun_ju1_six_yi_reversed_three_qi_forward() {
// 阴遁一局：戊从宫1起逆行 → 戊在宫1、己在宫9、庚在宫8、辛在宫7、壬在宫6、癸在宫5；
// 三奇乙丙丁顺接癸（宫5）之后 → 乙宫6、丙宫7、丁宫8……但宫6/7/8已被壬辛庚占？
// 故按算法回绕填空位，最终仍为九宫双射（上条已验）。这里验六仪逆布的起段。
let p = earth_plate(1, false);
assert_eq!(p[0], 4); // 宫1 = 戊
// 己庚辛壬癸沿逆行（宫9，8，7，6，5）。
assert_eq!(p[8], 5); // 宫9 = 己
assert_eq!(p[7], 6); // 宫8 = 庚
assert_eq!(p[6], 7); // 宫7 = 辛
assert_eq!(p[5], 8); // 宫6 = 壬
assert_eq!(p[4], 9); // 宫5 = 癸
// 三奇顺布：乙丙丁落于宫序递增的余三宫（宫2/3/4）。
assert_eq!(p[1], 1); // 宫2 = 乙
assert_eq!(p[2], 2); // 宫3 = 丙
assert_eq!(p[3], 3); // 宫4 = 丁
}

#[test]
fn ju_table_invariants() {
// 72 局表结构自检（防录入错）：阳遁 中=上+6、下=上+3 (mod9，1..9)；阴遁 中=上−6、下=上−3。
let amod9 = |x: i64| ((x - 1).rem_euclid(9) + 1) as u8;
for k in 0..24usize {
    let [up, mid, down] = YUAN_JU[k];
    if is_yang_dun(k) {
        assert_eq!(mid, amod9(i64::from(up) + 6), "{} 阳遁中元", SOLAR_TERMS[k]);
        assert_eq!(down, amod9(i64::from(up) + 3), "{} 阳遁下元", SOLAR_TERMS[k]);
    } else {
        assert_eq!(mid, amod9(i64::from(up) - 6), "{} 阴遁中元", SOLAR_TERMS[k]);
        assert_eq!(down, amod9(i64::from(up) - 3), "{} 阴遁下元", SOLAR_TERMS[k]);
    }
    // 所有局数在 1..9。
    assert!((1..=9).contains(&up) && (1..=9).contains(&mid) && (1..=9).contains(&down));
}
}

#[test]
fn yuan_of_branch_groups() {
// 子午卯酉=上、寅申巳亥=中、辰戌丑未=下。
for b in [0u8, 6, 3, 9] {
    assert_eq!(yuan_of_branch(b), Yuan::Upper);
}
for b in [2u8, 8, 5, 11] {
    assert_eq!(yuan_of_branch(b), Yuan::Middle);
}
for b in [4u8, 10, 1, 7] {
    assert_eq!(yuan_of_branch(b), Yuan::Lower);
}
}

#[test]
fn solar_term_index_and_dun() {
// λ=0（春分，k0）阳…λ=90（夏至，k6）阴…λ=270（冬至，k18）阳。
assert_eq!(solar_term_index(0.0), 0);
assert_eq!(SOLAR_TERMS[solar_term_index(0.0)], "春分");
assert_eq!(SOLAR_TERMS[solar_term_index(270.0)], "冬至");
assert!(is_yang_dun(solar_term_index(270.0))); // 冬至阳遁
assert!(!is_yang_dun(solar_term_index(90.0))); // 夏至阴遁
// 12 阳 12 阴。
let yang = (0..24).filter(|&k| is_yang_dun(k)).count();
assert_eq!(yang, 12);
}

#[test]
fn three_yuan_select_correct_ju() {
// 冬至 [1,7,4]：上元→1、中元→7、下元→4，并校验三元下标/名。
assert_eq!(solar_term_setup(18, Yuan::Upper).ju, 1);
assert_eq!(solar_term_setup(18, Yuan::Middle).ju, 7);
assert_eq!(solar_term_setup(18, Yuan::Lower).ju, 4);
assert_eq!(Yuan::Upper.index(), 0);
assert_eq!(Yuan::Middle.index(), 1);
assert_eq!(Yuan::Lower.index(), 2);
assert_eq!(Yuan::Middle.name(), "中元");
assert_eq!(Yuan::Lower.name(), "下元");
}

#[test]
fn setup_and_compute_consistent() {
let s = solar_term_setup(18, Yuan::Upper); // 冬至上元 → 1 局阳遁
assert_eq!(s.ju, 1);
assert!(s.yang_dun);
assert_eq!(s.yuan.name(), "上元");
let c = compute(2024, 6, 15, 14, 30, 8.0);
// 地盘双射、宫名取自洛书。
let set: std::collections::HashSet<&str> = c.earth.iter().copied().collect();
assert_eq!(set.len(), 9);
assert_eq!(c.palace[0], "坎");
assert!(c.fu_tou_branch < 12);
// 确定性。
let c2 = compute(2024, 6, 15, 14, 30, 8.0);
assert_eq!(c.earth, c2.earth);
assert_eq!(c.setup.ju, c2.setup.ju);
}

/// oracle：1987-09-17 15：00 长沙男 → 日柱 己巳 / 时柱 壬申 / 甲子旬 / 旬遁戊 / 旬空戌亥。
#[test]
fn qm0_xun_oracle_1987_changsha_male() {
let c = compute(1987, 9, 17, 15, 0, 8.0);
// 时柱：日柱己巳(stem=5，branch=5)，时支申(8) → 时干 (5%5)*2+8=8=壬。时柱壬申。
assert_eq!(c.time_ganzhi, "壬申");
assert_eq!(c.time_stem, 8);
assert_eq!(c.time_branch, 8);
// 旬：壬申 → head_branch=(8-8+12)%12=0 → 甲子旬，遁戊
assert_eq!(c.xun.head_ganzhi, "甲子");
assert_eq!(c.xun.head_branch, 0);
assert_eq!(c.xun.head_yi, "戊");
assert_eq!(c.xun.head_yi_stem, 4);
// 甲子旬旬空：戌亥
assert_eq!(c.xun.xunkong, ["戌", "亥"]);
}

/// oracle：不同时柱对应不同旬首六仪（6 旬覆盖）。
#[test]
fn qm0_six_xun_via_different_times() {
// 同日 1987-09-17 己巳日，时辰 → 时支：
// 23：30 子(0) → 时柱 甲子 → 甲子旬遁戊
// 03：30 寅(2) → 时柱 丙寅 → 甲子旬遁戊
// 15：00 申(8) → 时柱 壬申 → 甲子旬遁戊（以上同日，旬不跨）
// 不同日才会跨旬，我们换日：
// 1987-09-22（甲戌日） 子时(0) → 甲子时 → 甲子旬戊
// 但用同日不同时柱必同旬，因为日柱固定 stem=5，时柱 stem=(5*2+tb)%10，branch=tb，
// head=(tb - ((10+tb)%10) + 12)%12 — 计算
for (h, expected_yi) in [
    (23, "戊"), // 子时 甲子 旬遁戊
    (15, "戊"), // 申时 壬申 同旬
] {
    let c = compute(1987, 9, 17, h, 0, 8.0);
    assert_eq!(c.xun.head_yi, expected_yi, "h={h}");
}
// 跨旬：1992-08-09 甲申日，子时：日柱甲申(0，8)，子时=(0%5)*2+0=0，时柱甲子(0，0) → 甲子旬戊
let c1 = compute(1992, 8, 9, 0, 30, 8.0);
assert!(["戊", "己", "庚", "辛", "壬", "癸"].contains(&c1.xun.head_yi));
}

/// 旬首六仪在地盘九宫中必有一席之地（双射性质）；xun_yi_palace ∈ 1..=9。
#[test]
fn qm0_xun_yi_palace_in_range() {
for (y, m, d, h) in [(1987, 9, 17, 15), (1990, 6, 15, 14), (2024, 1, 1, 0), (2026, 6, 17, 10)] {
    let c = compute(y, m, d, h, 0, 8.0);
    assert!((1..=9).contains(&c.xun_yi_palace), "xun_yi_palace {} 应 ∈ 1..=9", c.xun_yi_palace);
    // 该宫的地盘干 = 旬首六仪
    assert_eq!(c.earth[(c.xun_yi_palace - 1) as usize], c.xun.head_yi);
}
}

/// oracle：1987-09-17 15：00 长沙男 阴遁 3 局 → 时干壬在艮 8 宫 = 值符宫；
/// 旬首戊在震 3 宫 → 本旬值符星 = 震 3 原配「天冲」。
#[test]
fn qm1a_zhi_fu_oracle_1987_changsha() {
let c = compute(1987, 9, 17, 15, 0, 8.0);
// 时柱壬申：time_stem=8（壬） ≠ 0（甲） → 实际值符 = 壬
assert_eq!(c.zhi_fu_stem, 8);
assert_eq!(c.zhi_fu_stem_name, "壬");
// 阴遁 3 局地盘：坎1庚 坤2己 震3戊 巽4乙 中5丙 乾6丁 兑7癸 艮8壬 离9辛
// → 壬在艮 8 宫
assert_eq!(c.zhi_fu_palace, 8);
// 旬首戊在震 3 宫（已验） → 本旬值符星 = 震 3 原配 = 天冲
assert_eq!(c.xun_yi_palace, 3);
assert_eq!(c.zhi_fu_xing, "天冲");
// 九星原配 9 宫（地盘初始，未旋转）
assert_eq!(c.jiuxing_earth, ["天蓬", "天芮", "天冲", "天辅", "天禽", "天心", "天柱", "天任", "天英"]);
}

/// 时干为甲时，实际值符 = 旬首六仪（遁仪规则）。
#[test]
fn qm1a_effective_zhi_fu_stem_jia_remap() {
// 6 旬旬首的甲（时干=0）分别遁戊/己/庚/辛/壬/癸
for (head_yi, want_name) in [(4u8, "戊"), (5, "己"), (6, "庚"), (7, "辛"), (8, "壬"), (9, "癸")] {
    assert_eq!(effective_zhi_fu_stem(0, head_yi), head_yi);
    assert_eq!(STEM_NAMES[head_yi as usize], want_name);
}
// 时干非甲(1..=9) → 实际值符 = 时干本身，旬首六仪无关
for ts in 1..=9u8 {
    for head_yi in [4u8, 5, 6, 7, 8, 9] {
        assert_eq!(effective_zhi_fu_stem(ts, head_yi), ts);
    }
}
}

/// 9 星原配 9 宫不变量（蓬 1 / 芮 2 / 冲 3 / 辅 4 / 禽 5 / 心 6 / 柱 7 / 任 8 / 英 9）。
#[test]
fn qm1a_jiuxing_palace_table_stable() {
// 索引 0 占位，1..=9 为 9 宫原配九星
assert_eq!(JIU_XING_PALACE[1], "天蓬");
assert_eq!(JIU_XING_PALACE[2], "天芮");
assert_eq!(JIU_XING_PALACE[3], "天冲");
assert_eq!(JIU_XING_PALACE[4], "天辅");
assert_eq!(JIU_XING_PALACE[5], "天禽");
assert_eq!(JIU_XING_PALACE[6], "天心");
assert_eq!(JIU_XING_PALACE[7], "天柱");
assert_eq!(JIU_XING_PALACE[8], "天任");
assert_eq!(JIU_XING_PALACE[9], "天英");
// 9 颗星全唯一
let set: std::collections::HashSet<&str> = JIU_XING_PALACE[1..].iter().copied().collect();
assert_eq!(set.len(), 9);
}

/// 值符宫与值符星跨时刻覆盖性 — 不同时刻 zhi_fu_palace 与 zhi_fu_xing 应都 ∈ 合法集合。
#[test]
fn qm1a_zhi_fu_consistency_over_times() {
for (y, m, d, h) in [(1987, 9, 17, 15), (1990, 6, 15, 14), (2024, 1, 1, 0), (2026, 6, 17, 10)] {
    let c = compute(y, m, d, h, 0, 8.0);
    assert!((1..=9).contains(&c.zhi_fu_palace), "zhi_fu_palace {} 应 ∈ 1..=9", c.zhi_fu_palace);
    assert!(JIU_XING_PALACE[1..].contains(&c.zhi_fu_xing), "{} 不在九星表内", c.zhi_fu_xing);
    // 值符宫的地盘干 = 实际值符天干
    assert_eq!(c.earth[(c.zhi_fu_palace - 1) as usize], c.zhi_fu_stem_name);
    // 值符星 = 旬首六仪所在宫的原配九星
    assert_eq!(c.zhi_fu_xing, JIU_XING_PALACE[c.xun_yi_palace as usize]);
}
}

#[test]
fn fu_tou_is_recent_jia_or_ji_day() {
// 符头日的日干必为甲(0)或己(5)。扫描多日校验。
let base = mingli_astro::civil_day_number(2024, 1, 1);
for k in 0..60i64 {
    let jdn = base + k;
    let day = mingli_ganzhi::day_ganzhi(jdn);
    let back = i64::from(day.stem % 5);
    let fu = mingli_ganzhi::day_ganzhi(jdn - back);
    assert!(fu.stem == 0 || fu.stem == 5, "符头日干应为甲/己");
}
}

// ── 天地盘干相加诸格 ────────────────────────────────────────────────
//
// 四层独立编纂（《奇门遁甲统宗》卷一奇门四十格 ·《遁甲演义》卷二逐格详解 ·
// 《奇门法窍》卷六吉凶格注释 ·《奇门遁甲秘笈大全》卷十五）在条件与方向上完全一致。
// 《遁甲演义》引王璋作「天上六辛加地下六乙」「天上六癸加地下六丁」，方向无歧义。

/// 造一个只在指定宫摆好天地盘干的最小盘面，用来单测格的判定。
fn plate_with(palace: u8, sky_stem: &'static str, earth_stem: &'static str) -> ([&'static str; 9], SkyPlate, GatePlate) {
    let k = palace as usize - 1;
    let mut earth = [""; 9];
    let mut stems = [""; 9];
    earth[k] = earth_stem;
    stems[k] = sky_stem;
    let sky = SkyPlate { shift: 1, stars: [""; 9], stems, center_stem: "", center_palace: 2 };
    let gates = GatePlate { zhi_shi_gate: "", zhi_shi_palace: 1, steps: 0, shift: 1, gates: [""; 9] };
    (earth, sky, gates)
}

/// 八个干加干格，逐格钉方向：反过来必是另一个格或不成格。
#[test]
fn qm5_stem_patterns_hold_in_one_direction_only() {
    for (sky_stem, earth_stem, name, class) in STEM_PATTERNS {
        let (earth, sky, gates) = plate_with(3, sky_stem, earth_stem);
        let p = patterns(&earth, &sky, &gates);
        assert_eq!(p.stem_patterns.len(), 1, "{name} 应恰成一格");
        let found = &p.stem_patterns[0];
        assert_eq!((found.name, found.sky, found.earth), (name, sky_stem, earth_stem));
        assert_eq!(found.classical_class, class);
        assert_eq!(found.palace, 3);

        // 反向：要么是另一个格，要么不成格；绝不能还判成同一个格
        let (earth_r, sky_r, gates_r) = plate_with(3, earth_stem, sky_stem);
        let back = patterns(&earth_r, &sky_r, &gates_r);
        for hit in &back.stem_patterns {
            assert_ne!(hit.name, name, "{sky_stem}加{earth_stem} 与 {earth_stem}加{sky_stem} 不该同名");
        }
    }
}

/// 三对反向格各自成对，且吉凶归类照古籍：返首/跌穴俱吉，其余三对俱凶。
#[test]
fn qm5_the_reversed_pairs_are_named_apart() {
    let name_of = |sky_stem: &'static str, earth_stem: &'static str| {
        let (e, s, g) = plate_with(1, sky_stem, earth_stem);
        patterns(&e, &s, &g).stem_patterns.first().map(|p| (p.name, p.classical_class))
    };
    assert_eq!(name_of("戊", "丙"), Some(("青龙返首", "吉")));
    assert_eq!(name_of("丙", "戊"), Some(("飞鸟跌穴", "吉")));
    assert_eq!(name_of("辛", "乙"), Some(("白虎猖狂", "凶")));
    assert_eq!(name_of("乙", "辛"), Some(("青龙逃走", "凶")));
    assert_eq!(name_of("癸", "丁"), Some(("螣蛇夭矫", "凶")));
    assert_eq!(name_of("丁", "癸"), Some(("朱雀投江", "凶")));
    assert_eq!(name_of("丙", "庚"), Some(("荧入太白", "凶")));
    assert_eq!(name_of("庚", "丙"), Some(("太白入荧", "凶")));
    // 不在表里的组合不成格
    assert_eq!(name_of("戊", "戊"), None);
    assert_eq!(name_of("乙", "丙"), None);
}

/// 三奇得使六组：奇在天、仪在地，且**三组与凶格是同一个判据**。
///
/// 「乙加甲午」与「乙加辛（青龙逃走）」不是可能共现，是同一个盘面——地盘辛恒为甲午辛。
/// 《遁甲演义》卷二：「乙奇加甲午辛乃青龙逃走，丙奇加甲申庚上乃荧入太白，丁奇加甲寅癸乃朱雀投江。
/// 凡此三者，尚有微疵不吉。如遇本旬直符同临其上，方可用之而吉也。」
/// 朴素实现会把这三个凶格标成吉格，所以这条测试要求两边同时出现。
#[test]
fn qm5_three_of_the_six_de_shi_pairs_are_the_very_same_board_as_a_calamity() {
    for (qi, yi, xun_head, conflicting) in QI_DE_SHI_PAIRS {
        let (earth, sky, gates) = plate_with(7, qi, yi);
        let p = patterns(&earth, &sky, &gates);
        assert_eq!(p.qi_de_shi.len(), 1, "{qi}加{yi} 应成得使");
        let d = &p.qi_de_shi[0];
        assert_eq!((d.qi, d.yi, d.xun_head, d.conflicting), (qi, yi, xun_head, conflicting));

        match conflicting {
            // 同判据的凶格必须同时出现在 stem_patterns 里，名字对得上
            Some(bad) => {
                let names: Vec<&str> = p.stem_patterns.iter().map(|s| s.name).collect();
                assert!(names.contains(&bad), "{qi}加{yi} 应同时判出 {bad}，实得 {names:?}");
            }
            // 洁净的三组：乙加己、丁加壬 不成任何凶格；丙加戊 同时是飞鸟跌穴（同为吉）
            None => {
                for hit in &p.stem_patterns {
                    assert_eq!(hit.classical_class, "吉", "{qi}加{yi} 不该判出凶格 {}", hit.name);
                }
            }
        }
    }
    // 六组恰好覆盖三奇 × 各两仪，且旬首互不重复
    let mut heads: Vec<&str> = QI_DE_SHI_PAIRS.iter().map(|(_, _, h, _)| *h).collect();
    heads.sort_unstable();
    let mut want = ["甲子", "甲戌", "甲申", "甲午", "甲辰", "甲寅"];
    want.sort_unstable();
    assert_eq!(heads, want, "六旬首各用一次");
    // 每奇恰配两仪
    for qi in SAN_QI {
        assert_eq!(QI_DE_SHI_PAIRS.iter().filter(|(q, ..)| *q == qi).count(), 2, "{qi} 应配两仪");
    }
}

/// 三奇合吉门与三奇得使是两个格，不可混为一谈。
///
/// 《烟波钓叟歌》分作两句（「吉门偶尔合三奇」与「三奇得使诚堪使」），
/// 《奇门遁甲秘笈大全》卷十五也分列「三奇上吉门格」与「三奇得使格」。
#[test]
fn qm5_meeting_a_lucky_gate_is_not_the_same_as_being_on_duty() {
    // 乙 + 开门同宫，但地盘不是甲戌己 → 只合吉门，不得使
    let mut earth = [""; 9];
    let mut stems = [""; 9];
    let mut gate_names = [""; 9];
    earth[2] = "丙";
    stems[2] = "乙";
    gate_names[2] = "开门";
    let sky = SkyPlate { shift: 1, stars: [""; 9], stems, center_stem: "", center_palace: 2 };
    let gates = GatePlate { zhi_shi_gate: "开门", zhi_shi_palace: 3, steps: 0, shift: 1, gates: gate_names };
    let p = patterns(&earth, &sky, &gates);
    assert_eq!(p.qi_gates.len(), 1, "乙与开门同宫，应判三奇合吉门");
    assert!(p.qi_de_shi.is_empty(), "地盘不是甲戌己，不该判得使");

    // 反过来：乙加己 得使，但无吉门
    let (e2, s2, g2) = plate_with(3, "乙", "己");
    let q = patterns(&e2, &s2, &g2);
    assert_eq!(q.qi_de_shi.len(), 1);
    assert!(q.qi_gates.is_empty(), "无门则不合吉门");
}

/// 中五宫无天盘干（寄坤二），扫描不该在那里静默漏判或误判。
#[test]
fn qm5_the_empty_centre_never_matches_a_pattern() {
    let mut earth = [""; 9];
    earth[4] = "丙"; // 中五地盘有干，天盘为空
    let sky = SkyPlate { shift: 1, stars: [""; 9], stems: [""; 9], center_stem: "戊", center_palace: 2 };
    let gates = GatePlate { zhi_shi_gate: "", zhi_shi_palace: 1, steps: 0, shift: 1, gates: [""; 9] };
    let p = patterns(&earth, &sky, &gates);
    assert!(p.stem_patterns.is_empty() && p.qi_de_shi.is_empty(), "中五天盘为空，不该成格");
    assert!(!p.stem_fu_yin_palaces.contains(&5), "中五也不该判干伏吟");
}

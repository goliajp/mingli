//! 干支各层的校验：六十甲子的对角子群结构、口诀多源对照、性质测试。

use super::*;

#[test]
fn hidden_stems_benqi_matches_branch_element() {
// 性质校验：每支「本气」藏干五行必须 = 地支本气五行；录入放错支会被抓。
for b in 0..12u8 {
    assert_eq!(stem_element(hidden_stems(b)[0]), branch_element(b), "支 {b} 本气不符");
}
let total: usize = (0..12u8).map(|b| hidden_stems(b).len()).sum();
assert_eq!(total, 28); // 藏干总数
}

#[test]
fn twelve_stage_lin_guan_equals_lu() {
// 强自校验：各天干「临官」(stage 3)必须落在其禄位（禄位独立 oracle，极标准），
// 10 干全验 = 把长生起点表完整交叉验证；再验长生位 stage=0。
let lu = [2u8, 3, 5, 6, 5, 6, 8, 9, 11, 0]; // 甲寅乙卯丙巳丁午戊巳己午庚申辛酉壬亥癸子
for s in 0..10u8 {
    assert_eq!(twelve_stage(s, lu[s as usize]), 3, "干{s}临官应在禄位");
    assert_eq!(twelve_stage(s, CHANGSHENG_START[s as usize]), 0, "干{s}长生位");
}
}

#[test]
fn cycle_matches_core() {
// 周期 60 = lcm(10，12)；且干支是对角子群（异阴阳组合不可达）。
assert_eq!(i64::from(CYCLE), mingli_core::cyclic::cycle_period(&[10, 12]));
assert!(mingli_core::cyclic::crt_combine(&[(0, 10), (1, 12)]).is_none()); // 甲+丑 异阴阳
assert_eq!(mingli_core::cyclic::crt_combine(&[(0, 10), (0, 12)]), Some(0)); // 甲子
}

#[test]
fn index_roundtrip() {
for n in 0..60u8 {
    assert_eq!(GanZhi::from_index(n).index(), n);
}
}

/// 日柱锚点。**这是全树放大半径最大的一个常数**——四柱、择日、大六壬、奇门、
/// 七政四余的日柱全从它推，错一天就整片错，且错得很像对的。
///
/// 三源交叉确认，且第三源不是同类互抄：
///
/// 1. 万年历 <https://wannianrili.bmcx.com/2024-01-01__wannianrili/>：
///    「癸卯年 甲子月 **甲子日**」，农历十一月二十
/// 2. 不亦居老黄历 <https://www.buyiju.com/lhl/2024-1-1.html>：同日同作甲子
/// 3. **另一条路**：Yuk Tung Liu《Sexagenary Cycle》给的算法
///    <https://ytliu0.github.io/ChineseCalendar/sexagenary.html> 作
///    `S = 1 + mod(JDnoon − 11, 60)`，甲子为 `S = 1`，其自身锚在
///    **2019-01-27（JD 2458511）**——与本仓库的锚点是不同的一天。
///    代入 `JDN 2460311` 得 `S = 1 + (2460300 mod 60) = 1` → 甲子，与前两源相合。
///
/// JDN 本身也自核过：Fliegel–Van Flandern 式算 2024-01-01 得 2460311。
#[test]
fn day_pillar_anchors() {
assert_eq!(day_ganzhi(DAY_ANCHOR_JDN).to_string(), "甲子"); // 2024-01-01
assert_eq!(day_ganzhi_index(DAY_ANCHOR_JDN + 1), 1); // 乙丑
assert_eq!(day_ganzhi_index(2_451_545), 54); // 2000-01-01 = 戊午#54

// 把第三源的公式原样跑一遍：它与本仓库各自独立地由自己的锚点推出同一个答案。
// 只要两边的推法有一处系统性偏差，这里就会分道扬镳。
for jdn in [DAY_ANCHOR_JDN, 2_451_545, 2_458_511, 2_460_311 + 12_345, 2_400_000] {
    let s = 1 + (jdn - 11).rem_euclid(60); // ytliu：甲子 = 1
    assert_eq!(
        day_ganzhi_index(jdn),
        u8::try_from(s - 1).expect("S∈1..=60"),
        "JDN {jdn}：本仓库与 ytliu 的公式给出不同的干支序",
    );
}
}

#[test]
fn year_pillar() {
assert_eq!(year_ganzhi(1984).to_string(), "甲子");
assert_eq!(year_ganzhi(1990).to_string(), "庚午");
assert_eq!(year_ganzhi(2024).to_string(), "甲辰");
}

#[test]
fn wuhu_dun() {
assert_eq!(month_pillar_stem(6, 2), 4); // 庚年 寅=戊
assert_eq!(month_pillar_stem(6, 6), 8); // 午=壬
assert_eq!(month_pillar_stem(6, 11), 3); // 亥=丁
}

#[test]
fn hour_branches() {
assert_eq!(hour_branch(14, 30), 7); // 未
assert_eq!(hour_branch(23, 30), 0); // 子
assert_eq!(hour_branch(0, 30), 0); // 子
assert_eq!(hour_branch(1, 0), 1); // 丑
}

#[test]
fn nayin() {
assert_eq!(nayin_element(GanZhi { stem: 3, branch: 11 }), Element::Earth); // 丁亥 屋上土
assert_eq!(nayin_element(GanZhi { stem: 0, branch: 0 }), Element::Metal); // 甲子 海中金
assert_eq!(nayin_element(GanZhi { stem: 4, branch: 4 }), Element::Wood); // 戊辰 大林木
assert_eq!(nayin_element(GanZhi { stem: 2, branch: 0 }), Element::Water); // 丙子 涧下水
assert_eq!(nayin_element(GanZhi { stem: 2, branch: 2 }), Element::Fire); // 丙寅 炉中火
}

#[test]
fn elements_and_cycles() {
assert_eq!(stem_element(0), Element::Wood); // 甲
assert_eq!(stem_element(7), Element::Metal); // 辛
assert_eq!(branch_element(0), Element::Water); // 子
assert_eq!(branch_element(2), Element::Wood); // 寅
assert_eq!(branch_element(5), Element::Fire); // 巳
assert_eq!(Element::Wood.name(), "木");
assert_eq!(Element::Fire.name(), "火");
assert_eq!(Element::Earth.name(), "土");
assert_eq!(Element::Metal.name(), "金");
assert_eq!(Element::Water.name(), "水");
assert_eq!(Element::Wood.generates(), Element::Fire);
assert_eq!(Element::Wood.controls(), Element::Earth);
// 五行各自生克闭环
for e in [
    Element::Wood,
    Element::Fire,
    Element::Earth,
    Element::Metal,
    Element::Water,
] {
    assert_ne!(e.generates(), e);
    assert_ne!(e.controls(), e);
}
}

/// 神煞 mapping 性质校验：与十二长生的派生关系。
/// 禄=临官、文昌=食神临官、学堂=日干长生、词馆 ≈ 食神临官（一致与文昌）。
#[test]
fn shensha_derivation_properties() {
for s in 0..10u8 {
    // 禄 = 十二长生临官位 (stage 3)
    assert_eq!(twelve_stage(s, LU[s as usize]), 3, "禄=临官 干{s}");
    // 学堂 = 日干长生(stage 0)— 与 CHANGSHENG_START 一致
    assert_eq!(twelve_stage(s, XUETANG[s as usize]), 0, "学堂=长生 干{s}");
    // 阳干羊刃在帝旺(stage 4)，阴干为 12 sentinel
    if s.is_multiple_of(2) {
        assert_eq!(twelve_stage(s, YANGREN[s as usize]), 4, "羊刃=帝旺 阳干{s}");
    } else {
        assert_eq!(YANGREN[s as usize], 12, "阴干无羊刃 干{s}");
    }
    // 词馆地支 ≈ 禄之地支（只看支位 — 词馆严格用法需配干，见 doc）
    // 实际不少干位词馆与禄同 — 这是巧合，非严格相等；仅校验 ∈ 12 范围
    assert!(CIGUAN[s as usize] < 12);
    assert!(WENCHANG[s as usize] < 12);
    assert!(HONGYAN[s as usize] < 12);
}
}

/// 三合神煞 mapping：寅午戌组 (group 0) 的桃花=卯/驿马=申/华盖=戌/将星=午。
#[test]
fn sanhe_shensha_oracle() {
// 寅午戌组 → 0
for b in [2u8, 6, 10] { assert_eq!(sanhe_group_index(b), 0); }
for b in [8u8, 0, 4] { assert_eq!(sanhe_group_index(b), 1); }
for b in [5u8, 9, 1] { assert_eq!(sanhe_group_index(b), 2); }
for b in [11u8, 3, 7] { assert_eq!(sanhe_group_index(b), 3); }

// 桃花 = 沐浴（三合长生顺数 1 步）
assert_eq!(TAOHUA[0], 3);  // 寅午戌见卯
assert_eq!(TAOHUA[1], 9);  // 申子辰见酉
assert_eq!(TAOHUA[2], 6);  // 巳酉丑见午
assert_eq!(TAOHUA[3], 0);  // 亥卯未见子

// 驿马 = 三合首字对冲(+6 mod 12)
for i in 0..4 {
    let first = [2u8, 8, 5, 11][i];
    assert_eq!(YIMA[i], (first + 6) % 12, "驿马 = 三合首字对冲");
}

// 华盖 = 三合末字（三合首+8 = 库）
for i in 0..4 {
    let first = [2u8, 8, 5, 11][i];
    assert_eq!(HUAGAI[i], (first + 8) % 12, "华盖 = 三合末字");
}

// 将星 = 三合中字（三合首+4 = 帝旺）
for i in 0..4 {
    let first = [2u8, 8, 5, 11][i];
    assert_eq!(JIANGXING[i], (first + 4) % 12, "将星 = 三合中字");
}
}

/// 魁罡四日柱固定。
#[test]
fn kuigang_four_days_oracle() {
// 庚辰(6，4) / 庚戌(6，10) / 壬辰(8，4) / 戊戌(4，10)
assert!(is_kuigang_day(GanZhi { stem: 6, branch: 4 }));
assert!(is_kuigang_day(GanZhi { stem: 6, branch: 10 }));
assert!(is_kuigang_day(GanZhi { stem: 8, branch: 4 }));
assert!(is_kuigang_day(GanZhi { stem: 4, branch: 10 }));
// 非魁罡示例
assert!(!is_kuigang_day(GanZhi { stem: 0, branch: 0 })); // 甲子
assert!(!is_kuigang_day(GanZhi { stem: 6, branch: 0 })); // 庚子（辰戌之外）
}

/// 1987-09-17 男 → 日柱 己巳(stem=5)、日支 巳(5)、年支 卯(3)。
/// 神煞 oracle：日干己土锚 → 月支酉 = 学堂（己长生在酉）+ 词馆/禄（均午，不在酉）；
/// 时支申 = 红艳（癸申？不是，己干红艳=辰）；看几柱地支落点。
#[test]
fn shensha_lookup_1987_oracle() {
// 日主 己(5)
// 学堂（己）= 酉(9) ← XUETANG[5] = 9
assert_eq!(XUETANG[5], 9);
// 禄（己）= 午(6)
assert_eq!(LU[5], 6);
// 文昌（己）= 酉(9)
assert_eq!(WENCHANG[5], 9);
// 红艳（己）= 辰(4)
assert_eq!(HONGYAN[5], 4);

// 月支酉(9) + 日干己 → 命中 学堂 + 文昌（同位 9）
let v = shensha_by_day_stem(5, 9);
assert!(v.contains(&"学堂"));
assert!(v.contains(&"文昌"));
assert!(!v.contains(&"禄"));

// 年支卯(3) anchor → 亥卯未组 → 桃花=子(0)、驿马=巳(5)、华盖=未(7)、将星=卯(3)
// 日支巳(5) 对年支卯(3) anchor → 命中 驿马！
let v2 = shensha_by_branch_anchor(3, 5);
assert!(v2.contains(&"驿马"));
assert!(!v2.contains(&"桃花"));
}

#[test]
fn parse_ganzhi_round_trip() {
for n in 0..60u8 {
    let g = GanZhi::from_index(n);
    assert_eq!(parse_ganzhi(&g.to_string()), Some(g));
}
assert_eq!(parse_ganzhi("甲子"), Some(GanZhi { stem: 0, branch: 0 }));
assert_eq!(parse_ganzhi("癸亥"), Some(GanZhi { stem: 9, branch: 11 }));
// 异阴阳组合可解析（语义上不入六十甲子，但符号上仍是 （干，支））
assert_eq!(parse_ganzhi("甲丑"), Some(GanZhi { stem: 0, branch: 1 }));
assert!(parse_ganzhi("").is_none());
assert!(parse_ganzhi("甲").is_none());
assert!(parse_ganzhi("甲子丑").is_none());
assert!(parse_ganzhi("XY").is_none());
// 天干过关、地支不在表内——两个位置各自都要挡住
assert!(parse_ganzhi("甲X").is_none());
}

#[test]
fn element_index_round_trip() {
// 五个五行索引互不相同、且与 ten_gods 划分（比劫=同党）兼容
let all = [
    Element::Wood, Element::Fire, Element::Earth, Element::Metal, Element::Water,
];
let mut seen = [false; 5];
for e in all {
    let i = e.index();
    assert!(i < 5);
    assert!(!seen[i]);
    seen[i] = true;
}
assert!(seen.iter().all(|&b| b));
}

#[test]
fn friendly_to_day_master_matches_ten_gods() {
// 同党 = 十神为 比肩/劫财/偏印/正印。穷举 10 干 × 10 干对照 `ten_god`。
for dm in 0..10u8 {
    for x in 0..10u8 {
        let tg = ten_god(dm, x);
        let want = matches!(tg, "比肩" | "劫财" | "偏印" | "正印");
        assert_eq!(
            is_friendly_to_day_master(dm, x), want,
            "dm={dm} other={x} ten_god={tg}"
        );
    }
}
}

#[test]
fn ten_gods() {
// 日主 辛（7， 金阴）
assert_eq!(ten_god(7, 6), "劫财"); // 辛 vs 庚（金阳）
assert_eq!(ten_god(7, 7), "比肩");
assert_eq!(ten_god(7, 8), "伤官"); // 辛 vs 壬（水阳） 我生异性
assert_eq!(ten_god(7, 4), "正印"); // 辛 vs 戊（土阳） 生我异性
assert_eq!(ten_god(7, 0), "正财"); // 辛 vs 甲（木阳） 我克异性
assert_eq!(ten_god(7, 2), "正官"); // 辛 vs 丙（火阳） 克我异性
assert_eq!(ten_god(7, 5), "偏印"); // 辛 vs 己（土阴） 生我同性
assert_eq!(ten_god(7, 3), "七杀"); // 辛 vs 丁（火阴） 克我同性
assert_eq!(ten_god(7, 1), "偏财"); // 辛 vs 乙（木阴） 我克同性
assert_eq!(ten_god(7, 9), "食神"); // 辛 vs 癸（水阴） 我生同性
}

#[test]
fn xun_head_branch_six_xun_anchors() {
// 60 甲子 6 旬，每旬 10 干支，旬首支 ∈ {子，戌，申，午，辰，寅}。
let xuns: [(u8, &str); 6] = [(0, "子"), (10, "戌"), (8, "申"), (6, "午"), (4, "辰"), (2, "寅")];
for (i, (head, name)) in xuns.iter().enumerate() {
    // 该旬第 1 个干支（stem=0/甲） 的 head = head
    assert_eq!(
        xun_head_branch(GanZhi { stem: 0, branch: *head }),
        *head,
        "旬首 甲{name}",
    );
    // 该旬第 10 个干支（stem=9/癸） 的 head 也 = head（同旬）
    let last_b = (*head + 9) % 12;
    assert_eq!(
        xun_head_branch(GanZhi { stem: 9, branch: last_b }),
        *head,
        "末位 癸{} 应同旬",
        BRANCHES[last_b as usize],
    );
    // 旬内任一干支都应 → 该旬首
    for k in 0..10u8 {
        let b = (*head + k) % 12;
        assert_eq!(
            xun_head_branch(GanZhi { stem: k, branch: b }),
            *head,
            "旬 {i} 第 {k} 位 应归该旬",
        );
    }
}
}

#[test]
fn xun_yi_six_yi_for_six_xun() {
// 6 旬 → 6 仪：甲子→戊 / 甲戌→己 / 甲申→庚 / 甲午→辛 / 甲辰→壬 / 甲寅→癸
let cases: [(u8, u8, &str); 6] = [
    (0,  4, "戊"),  // 甲子旬遁戊
    (10, 5, "己"),  // 甲戌旬遁己
    (8,  6, "庚"),  // 甲申旬遁庚
    (6,  7, "辛"),  // 甲午旬遁辛
    (4,  8, "壬"),  // 甲辰旬遁壬
    (2,  9, "癸"),  // 甲寅旬遁癸
];
for (head, yi, name) in cases {
    assert_eq!(
        xun_yi(GanZhi { stem: 0, branch: head }),
        yi,
        "旬首甲{} → {name}",
        BRANCHES[head as usize],
    );
    assert_eq!(STEMS[yi as usize], name);
}
// 六仪 ∈ {戊己庚辛壬癸}，值落 4..=9。
for i in 0..60u8 {
    let gz = GanZhi { stem: i % 10, branch: i % 12 };
    let y = xun_yi(gz);
    assert!((4..=9).contains(&y), "六仪应 ∈ 4..=9， got {y} for gz {i}");
}
}

#[test]
fn xunkong_six_xun_oracles() {
// 经典 6 旬旬空 oracle（三命通会通行版）。
// 甲子旬空 戌亥(10，11)、甲戌旬空 申酉(8，9)、甲申旬空 午未(6，7)、
// 甲午旬空 辰巳(4，5)、甲辰旬空 寅卯(2，3)、甲寅旬空 子丑(0，1)。
let oracle: [(u8, [u8; 2], &str); 6] = [
    (0,  [10, 11], "甲子旬空戌亥"),
    (10, [8, 9],   "甲戌旬空申酉"),
    (8,  [6, 7],   "甲申旬空午未"),
    (6,  [4, 5],   "甲午旬空辰巳"),
    (4,  [2, 3],   "甲辰旬空寅卯"),
    (2,  [0, 1],   "甲寅旬空子丑"),
];
for (head, want, desc) in oracle {
    assert_eq!(xunkong(GanZhi { stem: 0, branch: head }), want, "{desc}");
}
// 1987-09-17 15：00 时柱壬申 (stem=8， branch=8) → 甲子旬 → 旬空戌亥
assert_eq!(xunkong(GanZhi { stem: 8, branch: 8 }), [10, 11]);
// 性质：60 甲子（stem 与 branch 奇偶同性）旬空 2 支恒不在本旬 10 个地支内。
for idx in 0..60u8 {
    let gz = GanZhi { stem: idx % 10, branch: idx % 12 };
    let head = xun_head_branch(gz);
    let kong = xunkong(gz);
    // 本旬 10 支 = (head..head+9) mod 12，旬空 2 支 = (head+10， head+11) mod 12，不交。
    for k in 0..10u8 {
        let in_xun = (head + k) % 12;
        assert_ne!(in_xun, kong[0]);
        assert_ne!(in_xun, kong[1]);
    }
}
}

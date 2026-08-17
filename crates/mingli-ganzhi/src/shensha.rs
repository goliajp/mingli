//! 神煞：11 项通行口诀，每项经 3 个以上中文源校验（含《三命通会》《渊海子平》原文）。
//!
//! 流派分歧（羊刃 / 红艳 / 学堂）显式取通行版，分歧写在各叶的 profile 备注里。

use super::*;

// ─── 神煞 ─── 11 神煞通行口诀，经多源校验（每项 3+ 中文源，含《三命通会》/《渊海子平》古籍原文）。
// 流派分歧（羊刃/红艳/学堂）显式取通行版，profile 备注。

/// 羊刃（阳刃）— 古典派（《三命通会》）：仅 5 阳干立刃，阴干无刃。
/// 甲卯 / 丙午 / 戊午 / 庚酉 / 壬子。阴干索引返回 12 表「无」。
/// 🟡 退一位通俗派（阴干乙寅丁巳己巳辛申癸亥）未入码；阴干索引返回 12 sentinel。
pub const YANGREN: [u8; 10] = [
    3, 12, 6, 12, 6, 12, 9, 12, 0, 12,
];

/// 禄神（建禄）= 十干临官位。各家完全一致。
/// 甲寅乙卯丙戊巳丁己午庚申辛酉壬亥癸子。
pub const LU: [u8; 10] = [2, 3, 5, 6, 5, 6, 8, 9, 11, 0];

/// 文昌贵人 = 食神之临官位。各家一致。
/// 甲巳乙午丙戊申丁己酉庚亥辛子壬寅癸卯。
pub const WENCHANG: [u8; 10] = [5, 6, 8, 9, 8, 9, 11, 0, 2, 3];

/// 红艳煞（A 派 / 三命通会）：甲乙午、丙寅、丁未、戊己辰、庚戌、辛酉、壬子、癸申。
/// 🟡 B 派（乙申 / 戊午 / 庚酉 / 辛戌）未入码。
pub const HONGYAN: [u8; 10] = [6, 6, 2, 7, 4, 4, 10, 9, 0, 8];

/// 学堂（子平派 / 日干长生位）— 与 `CHANGSHENG_START`（crate 私有）一致。
/// 🟡 三命通会派（纳音长生）未入码。
pub const XUETANG: [u8; 10] = [11, 6, 2, 9, 2, 9, 5, 0, 8, 3];

/// 词馆（地支位，不含干）— 各家一致。
/// 甲寅乙卯丙戊巳丁己午庚申辛酉壬亥癸戌。
/// 严格用法需「干支组合」匹配，本算法仅返回地支（满足常用查表）。
pub const CIGUAN: [u8; 10] = [2, 3, 5, 6, 5, 6, 8, 9, 11, 10];

/// 三合局首字 → 神煞落点 mapping。
/// 三合： 寅午戌(2，6，10)/ 申子辰(8，0，4)/ 巳酉丑(5，9，1)/ 亥卯未(11，3，7)。
/// 桃花（三合沐浴位）/驿马（对冲）/华盖（墓库）/将星（中神帝旺）
/// 按年支或日支 anchor 查；返回 12 sentinel = anchor 不在三合首字之四种之列（即非寅/申/巳/亥 → 走对应三合组）。
/// 通用方式：对任意地支查其三合组归属，然后返回神煞落点。
#[must_use]
#[allow(
    clippy::match_same_arms,
    reason = "寅午戌（组 0） 与 wildcard 体相同；wildcard 在 % 12 后理论不可达，保留以保 match 完整性与四组对称可读"
)]
pub fn sanhe_group_index(anchor_branch: u8) -> u8 {
    // 寅午戌→0， 申子辰→1， 巳酉丑→2， 亥卯未→3
    match anchor_branch % 12 {
        2 | 6 | 10 => 0,
        8 | 0 | 4 => 1,
        5 | 9 | 1 => 2,
        11 | 3 | 7 => 3,
        _ => 0,
    }
}

/// 桃花（咸池）= 三合沐浴位：寅午戌见卯、申子辰见酉、巳酉丑见午、亥卯未见子。
pub const TAOHUA: [u8; 4] = [3, 9, 6, 0];
/// 驿马 = 三合长生对冲：寅午戌见申、申子辰见寅、巳酉丑见亥、亥卯未见巳。
pub const YIMA: [u8; 4] = [8, 2, 11, 5];
/// 华盖 = 三合墓库：寅午戌见戌、申子辰见辰、巳酉丑见丑、亥卯未见未。
pub const HUAGAI: [u8; 4] = [10, 4, 1, 7];
/// 将星 = 三合中神（帝旺）：寅午戌见午、申子辰见子、巳酉丑见酉、亥卯未见卯。
pub const JIANGXING: [u8; 4] = [6, 0, 9, 3];

/// 魁罡四日柱（《三命通会》经典严格派，仅日柱入格）：庚辰/庚戌/壬辰/戊戌。
/// 返回 (stem， branch) 4 元组。
pub const KUIGANG_DAYS: [(u8, u8); 4] = [(6, 4), (6, 10), (8, 4), (4, 10)];

/// 是否为魁罡日（日柱 ∈ KUIGANG_DAYS）。
#[must_use]
pub fn is_kuigang_day(day_gz: GanZhi) -> bool {
    KUIGANG_DAYS.iter().any(|&(s, b)| s == day_gz.stem && b == day_gz.branch)
}

/// 给定锚干（日干）+ 目标支，返回命中的「日干锚」神煞名列表（羊刃/禄/文昌/红艳/学堂/词馆）。
#[must_use]
pub fn shensha_by_day_stem(day_stem: u8, branch: u8) -> Vec<&'static str> {
    let mut v = Vec::new();
    if YANGREN[day_stem as usize] == branch { v.push("羊刃"); }
    if LU[day_stem as usize] == branch { v.push("禄"); }
    if WENCHANG[day_stem as usize] == branch { v.push("文昌"); }
    if HONGYAN[day_stem as usize] == branch { v.push("红艳"); }
    if XUETANG[day_stem as usize] == branch { v.push("学堂"); }
    if CIGUAN[day_stem as usize] == branch { v.push("词馆"); }
    v
}

/// 给定锚支（年支或日支）+ 目标支，返回命中的「年/日支锚」神煞名列表（桃花/驿马/华盖/将星）。
#[must_use]
pub fn shensha_by_branch_anchor(anchor: u8, branch: u8) -> Vec<&'static str> {
    let g = sanhe_group_index(anchor) as usize;
    let mut v = Vec::new();
    if TAOHUA[g] == branch { v.push("桃花"); }
    if YIMA[g] == branch { v.push("驿马"); }
    if HUAGAI[g] == branch { v.push("华盖"); }
    if JIANGXING[g] == branch { v.push("将星"); }
    v
}

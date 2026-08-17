//! L2 主干：六十干支（sexagenary cycle）符号系统。
//!
//! 天干（10）与地支（12）并行推进，因 `gcd(10,12)=2`，其联合不是完整乘积 `Z₁₀×Z₁₂`
//! 而是阶为 `lcm(10,12)=60` 的对角子群——即六十甲子恰有 60 个组合（同阴阳配对），而非 120。
//! 这一结构由 [`mingli_core::cyclic`] 提供；本 crate 在其上构建干支的领域语义
//! （五行、纳音、五虎遁、时辰、日柱递推）。对天文/历法零依赖：日柱以民用日序（JDN）为输入。

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "小循环群上的模运算：结果恒被约束在 [0，n) 内（n≤60），整数窄化安全"
)]

use serde::Serialize;

/// 六十干支的循环周期（= `mingli_core::cyclic::cycle_period(&[10,12])`）。
pub const CYCLE: u8 = 60;

/// 日柱锚点：民用日序（JDN）2_460_311 = 公历 2024-01-01 = 甲子(#0)。
pub const DAY_ANCHOR_JDN: i64 = 2_460_311;

/// 十天干字面（甲=0 … 癸=9）。
pub const STEMS: [&str; 10] = ["甲", "乙", "丙", "丁", "戊", "己", "庚", "辛", "壬", "癸"];
/// 十二地支字面（子=0 … 亥=11）。
pub const BRANCHES: [&str; 12] = [
    "子", "丑", "寅", "卯", "辰", "巳", "午", "未", "申", "酉", "戌", "亥",
];

/// 一个干支组合：`stem` 天干 0..9（甲=0），`branch` 地支 0..11（子=0）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct GanZhi {
    /// 天干序号 0..9（甲=0）。
    pub stem: u8,
    /// 地支序号 0..11（子=0）。
    pub branch: u8,
}

impl GanZhi {
    /// 60 甲子序号 0..59（甲子=0）。
    #[must_use]
    pub fn index(&self) -> u8 {
        let mut n = i32::from(self.stem);
        while n % 12 != i32::from(self.branch) {
            n += 10;
        }
        (n % 60) as u8
    }
    /// 由 60 甲子序号 `n`（甲子=0）构造（对 `n` 取模，越界安全）。
    #[must_use]
    pub fn from_index(n: u8) -> Self {
        GanZhi {
            stem: n % 10,
            branch: n % 12,
        }
    }
    /// 天干字面。
    #[must_use]
    pub fn stem_str(&self) -> &'static str {
        STEMS[self.stem as usize]
    }
    /// 地支字面。
    #[must_use]
    pub fn branch_str(&self) -> &'static str {
        BRANCHES[self.branch as usize]
    }
}

impl std::fmt::Display for GanZhi {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}{}", self.stem_str(), self.branch_str())
    }
}

/// 日柱干支序号 0..59，输入为民用日序（JDN）。以 [`DAY_ANCHOR_JDN`] 为锚线性递推。
///
/// 注：八字「晚子时换日」传统不在此处处理；调用方按需在传入的 JDN 上 ±1。
#[must_use]
pub fn day_ganzhi_index(civil_day_jdn: i64) -> u8 {
    (civil_day_jdn - DAY_ANCHOR_JDN).rem_euclid(i64::from(CYCLE)) as u8
}

/// 日柱干支，输入为民用日序（JDN）。
#[must_use]
pub fn day_ganzhi(civil_day_jdn: i64) -> GanZhi {
    GanZhi::from_index(day_ganzhi_index(civil_day_jdn))
}

/// 年柱干支。`solar_year` 须为已按立春调整后的年份（八字）或农历年（紫微）。
#[must_use]
pub fn year_ganzhi(solar_year: i32) -> GanZhi {
    GanZhi {
        stem: (solar_year - 4).rem_euclid(10) as u8,
        branch: (solar_year - 4).rem_euclid(12) as u8,
    }
}

/// 五虎遁：给定年干，返回某地支宫位对应的天干（0..9）。寅(2) 为正月起点。
/// 用于月柱天干，以及紫微「命宫天干」。
#[must_use]
pub fn month_pillar_stem(year_stem: u8, branch: u8) -> u8 {
    let base = ((year_stem % 5) * 2 + 2) % 10; // 寅之干（甲己→丙…）
    let pos = (i32::from(branch) - 2).rem_euclid(12) as u8; // 距寅步数
    (base + pos) % 10
}

/// 时辰地支 0..11（子=0）。23：00–01：00 为子时。
#[must_use]
pub fn hour_branch(hour: u32, _minute: u32) -> u8 {
    (((hour + 1) % 24) / 2) as u8
}

/// 五行（金木水火土）。既用于纳音，也用于天干地支本气。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Element {
    /// 金。
    Metal,
    /// 木。
    Wood,
    /// 水。
    Water,
    /// 火。
    Fire,
    /// 土。
    Earth,
}

impl Element {
    /// 五行字面（木火土金水之一）。
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Element::Wood => "木",
            Element::Fire => "火",
            Element::Earth => "土",
            Element::Metal => "金",
            Element::Water => "水",
        }
    }
    /// 我生（所生之五行）：木→火→土→金→水→木。
    #[must_use]
    pub fn generates(self) -> Element {
        match self {
            Element::Wood => Element::Fire,
            Element::Fire => Element::Earth,
            Element::Earth => Element::Metal,
            Element::Metal => Element::Water,
            Element::Water => Element::Wood,
        }
    }
    /// 我克（所克之五行）：木克土、火克金、土克水、金克木、水克火。
    #[must_use]
    pub fn controls(self) -> Element {
        match self {
            Element::Wood => Element::Earth,
            Element::Fire => Element::Metal,
            Element::Earth => Element::Water,
            Element::Metal => Element::Wood,
            Element::Water => Element::Fire,
        }
    }
    /// 五行索引 0..5（木=0、火=1、土=2、金=3、水=4）——向量化分布用。
    #[must_use]
    pub fn index(self) -> usize {
        match self {
            Element::Wood => 0,
            Element::Fire => 1,
            Element::Earth => 2,
            Element::Metal => 3,
            Element::Water => 4,
        }
    }
}

/// 天干本气五行（甲乙木、丙丁火、戊己土、庚辛金、壬癸水）。
#[must_use]
pub fn stem_element(stem: u8) -> Element {
    match stem / 2 {
        0 => Element::Wood,
        1 => Element::Fire,
        2 => Element::Earth,
        3 => Element::Metal,
        _ => Element::Water,
    }
}

/// 地支本气五行（子亥水、寅卯木、巳午火、申酉金、辰戌丑未土）。
#[must_use]
pub fn branch_element(branch: u8) -> Element {
    const T: [Element; 12] = [
        Element::Water, // 子
        Element::Earth, // 丑
        Element::Wood,  // 寅
        Element::Wood,  // 卯
        Element::Earth, // 辰
        Element::Fire,  // 巳
        Element::Fire,  // 午
        Element::Earth, // 未
        Element::Metal, // 申
        Element::Metal, // 酉
        Element::Earth, // 戌
        Element::Water, // 亥
    ];
    T[(branch % 12) as usize]
}

/// 十神：`other_stem` 相对日主 `day_master` 的关系。
#[must_use]
pub fn ten_god(day_master: u8, other_stem: u8) -> &'static str {
    let dm = stem_element(day_master);
    let x = stem_element(other_stem);
    let same = (day_master % 2) == (other_stem % 2);
    // （同性， 异性） 名
    let (yang, yin) = if x == dm {
        ("比肩", "劫财")
    } else if dm.generates() == x {
        ("食神", "伤官") // 我生
    } else if dm.controls() == x {
        ("偏财", "正财") // 我克
    } else if x.controls() == dm {
        ("七杀", "正官") // 克我
    } else {
        ("偏印", "正印") // 生我
    };
    if same { yang } else { yin }
}

/// 由字符串（"甲子"/"癸亥"等）解析干支。两字异常或不在表内返回 None。
#[must_use]
pub fn parse_ganzhi(s: &str) -> Option<GanZhi> {
    let mut it = s.chars();
    let st = it.next()?;
    let br = it.next()?;
    if it.next().is_some() {
        return None;
    }
    let stem = STEMS.iter().position(|&v| v.starts_with(st))?;
    let branch = BRANCHES.iter().position(|&v| v.starts_with(br))?;
    Some(GanZhi { stem: stem as u8, branch: branch as u8 })
}

/// 旬首支：一个干支（天干 s，地支 b）所在六十甲子旬的旬首（甲）所在地支。
///
/// 60 甲子分 6 旬：甲子 / 甲戌 / 甲申 / 甲午 / 甲辰 / 甲寅，每旬 10 个干支。
/// 旬首支 ∈ {子， 戌， 申， 午， 辰， 寅}（6 个偶数支）。**算法**：
/// `head_branch = (branch − stem + 12) mod 12`。
#[must_use]
pub fn xun_head_branch(gz: GanZhi) -> u8 {
    ((u32::from(gz.branch) + 12 - u32::from(gz.stem)) % 12) as u8
}

/// 旬首六仪：一个干支所在旬的「遁仪」天干 — 奇门遁甲三奇六仪的根。
///
/// 6 旬 → 6 仪映射：
/// - 甲子旬（旬首支 = 子 = 0） → **戊** (stem = 4)
/// - 甲戌旬（旬首支 = 戌 = 10） → **己** (stem = 5)
/// - 甲申旬（旬首支 = 申 = 8） → **庚** (stem = 6)
/// - 甲午旬（旬首支 = 午 = 6） → **辛** (stem = 7)
/// - 甲辰旬（旬首支 = 辰 = 4） → **壬** (stem = 8)
/// - 甲寅旬（旬首支 = 寅 = 2） → **癸** (stem = 9)
///
/// 这是奇门「**值符**所遁之仪」的根：旬首六仪在地盘所在的宫 = 值符宫。
#[must_use]
pub fn xun_yi(gz: GanZhi) -> u8 {
    let head = xun_head_branch(gz);
    // head ∈ {0, 10, 8, 6, 4, 2} → yi ∈ {4, 5, 6, 7, 8, 9}
    4 + (12 - u32::from(head)) as u8 / 2 % 6
}

/// 旬空（空亡）：一个干支所在旬的 10 个干 12 支，余下未配上的 2 个地支。
///
/// **算法**：旬首支前 1 位与前 2 位（即旬首支 +10、+11 mod 12）。
/// 例：甲子旬（10 干 配 子丑寅卯辰巳午未申酉）→ 旬空 = 戌(10)/亥(11)。
///
/// 八字看「本命旬空」用日柱；奇门看「时柱旬空」用占事此刻的时柱。
#[must_use]
pub fn xunkong(gz: GanZhi) -> [u8; 2] {
    let head = xun_head_branch(gz);
    [(head + 10) % 12, (head + 11) % 12]
}

/// 「同党」判定：他干是否帮扶日主——即十神为比劫（同五行）或印星（生我）。
/// 其余（食伤/财/官杀）为「耗身」。
///
/// 这是旺衰量化（得地/得势）与用神扶抑的基础。
#[must_use]
pub fn is_friendly_to_day_master(day_master: u8, other_stem: u8) -> bool {
    let dm = stem_element(day_master);
    let x = stem_element(other_stem);
    x == dm || x.generates() == dm
}

/// 由干支求纳音五行（天干分组 + 地支分组求和定五行）。
#[must_use]
pub fn nayin_element(gz: GanZhi) -> Element {
    let s = (gz.stem / 2) + 1; // 甲乙=1…壬癸=5
    let b = ((gz.branch / 2) % 3) + 1; // 子丑/午未=1， 寅卯/申酉=2， 辰巳/戌亥=3
    let mut n = s + b;
    if n > 5 {
        n -= 5;
    }
    match n {
        1 => Element::Wood,
        2 => Element::Metal,
        3 => Element::Water,
        4 => Element::Fire,
        _ => Element::Earth,
    }
}

/// 地支藏干（人元）：每支所藏天干，**本气→中气→余气**顺序，值为天干 index（甲0…癸9）。
///
/// 通行「不分日固定表」。多源交叉验证(en/zh Wikipedia「地支/Earthly Branches」+ 百度百科
/// 「地支藏干」+ 子平典籍《渊海子平/子平真诠/滴天髓》体系)：11 支四源一致。
/// 🟡 流派异说：巳取通行「丙庚戊」（另有「丙戊庚」）、申取「庚壬戊」（另有「庚戊壬」）——
/// 取主流（巳为庚金长生→中气庚；申为壬水长生→中气壬）。「月令分日用事」日数分配分歧大，未入码。
pub const BRANCH_HIDDEN_STEMS: [&[u8]; 12] = [
    &[9],       // 子 癸
    &[5, 9, 7], // 丑 己癸辛
    &[0, 2, 4], // 寅 甲丙戊
    &[1],       // 卯 乙
    &[4, 1, 9], // 辰 戊乙癸
    &[2, 6, 4], // 巳 丙庚戊（🟡 异说 丙戊庚）
    &[3, 5],    // 午 丁己
    &[5, 1, 3], // 未 己乙丁
    &[6, 8, 4], // 申 庚壬戊（🟡 异说 庚戊壬）
    &[7],       // 酉 辛
    &[4, 7, 3], // 戌 戊辛丁
    &[8, 0],    // 亥 壬甲
];

/// 取某地支所藏天干（本→中→余），值为天干 index。
#[must_use]
pub fn hidden_stems(branch: u8) -> &'static [u8] {
    BRANCH_HIDDEN_STEMS[(branch % 12) as usize]
}

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

/// 十二长生阶段名（0=长生…11=养）。
pub const TWELVE_STAGES: [&str; 12] = [
    "长生", "沐浴", "冠带", "临官", "帝旺", "衰", "病", "死", "墓", "绝", "胎", "养",
];

/// 各天干「长生」起始地支(branch index)。阳干长生=本五行生地；阴干长生=同五行阳干「死」位。
const CHANGSHENG_START: [u8; 10] = [11, 6, 2, 9, 2, 9, 5, 0, 8, 3];

/// 日主天干在某地支的十二长生阶段 index(0..12)。阳干顺行、阴干逆行（传统派）。🟡 新派阴干顺行未实现。
#[must_use]
pub fn twelve_stage(stem: u8, branch: u8) -> u8 {
    let start = CHANGSHENG_START[(stem % 10) as usize];
    if stem.is_multiple_of(2) {
        (branch + 12 - start) % 12 // 阳干顺行
    } else {
        (start + 12 - branch) % 12 // 阴干逆行
    }
}

#[cfg(test)]
mod tests {
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

    #[test]
    fn day_pillar_anchors() {
        assert_eq!(day_ganzhi(DAY_ANCHOR_JDN).to_string(), "甲子"); // 2024-01-01
        assert_eq!(day_ganzhi_index(DAY_ANCHOR_JDN + 1), 1); // 乙丑
        assert_eq!(day_ganzhi_index(2_451_545), 54); // 2000-01-01 = 戊午#54
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
}

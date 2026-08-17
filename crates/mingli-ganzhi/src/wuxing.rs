//! 五行语义：干支各自的五行、藏干、纳音、十神与十二长生。

use super::*;

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

/// 十二长生阶段名（0=长生…11=养）。
pub const TWELVE_STAGES: [&str; 12] = [
    "长生", "沐浴", "冠带", "临官", "帝旺", "衰", "病", "死", "墓", "绝", "胎", "养",
];

/// 各天干「长生」起始地支(branch index)。阳干长生=本五行生地；阴干长生=同五行阳干「死」位。
pub(crate) const CHANGSHENG_START: [u8; 10] = [11, 6, 2, 9, 2, 9, 5, 0, 8, 3];

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

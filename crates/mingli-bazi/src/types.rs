//! 本叶的领域类型：出生输入、四柱、大运、农历与整张命盘。

use super::*;

/// 性别（用于定大运顺逆）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Gender {
    /// 男。
    Male,
    /// 女。
    Female,
}

/// 出生信息（排盘输入）。
#[derive(Debug, Clone, Copy, Serialize)]
pub struct BirthInput {
    /// 公历年。
    pub year: i32,
    /// 公历月 1..12。
    pub month: u32,
    /// 公历日 1..31。
    pub day: u32,
    /// 时 0..23。
    pub hour: u32,
    /// 分 0..59。
    pub minute: u32,
    /// 时区偏移小时（中国 +8，日本 +9）。
    pub tz: f64,
    /// 性别；`None` 则不排大运。
    pub gender: Option<Gender>,
}

/// 一柱（年/月/日/时）。
#[derive(Debug, Clone, Serialize)]
pub struct Pillar {
    /// 干支（如「庚午」）。
    pub ganzhi: String,
    /// 天干。
    pub stem: String,
    /// 地支。
    pub branch: String,
    /// 天干本气五行。
    pub stem_wuxing: String,
    /// 地支本气五行。
    pub branch_wuxing: String,
    /// 纳音五行（DET；30 名表「海中金」等属 🟡 后续）。
    pub nayin: String,
    /// 该柱天干相对日主的十神（日柱为「日主」）。
    pub ten_god: String,
    /// 地支藏干（人元）及各藏干对日主的十神（本气→中气→余气）。
    pub hidden: Vec<HiddenStem>,
    /// 日主在本柱地支的十二长生阶段（长生/帝旺/墓/绝…）。
    pub day_twelve: String,
    /// 该柱地支命中的神煞名：日干锚（羊刃/禄/文昌/红艳/学堂/词馆）+ 年支锚（桃花/驿马/华盖/将星）+ 日柱魁罡。
    ///
    /// 🟡 流派分歧已按通行版固化（羊刃古典 5 阳干、红艳 A 派/三命通会、学堂子平派、魁罡严格四日柱）。
    pub shensha: Vec<String>,
}

/// 一个支藏天干 + 其对日主的十神（支藏十神）。
#[derive(Debug, Clone, Serialize)]
pub struct HiddenStem {
    /// 藏干（天干）。
    pub stem: String,
    /// 该藏干相对日主的十神。
    pub ten_god: String,
}

/// 一步大运。
#[derive(Debug, Clone, Serialize)]
pub struct LuckPillar {
    /// 起运虚岁。
    pub start_age: u32,
    /// 该步大运干支。
    pub ganzhi: String,
}

/// 大运。
#[derive(Debug, Clone, Serialize)]
pub struct DaYun {
    /// 是否顺行（阳男阴女顺、阴男阳女逆）。
    pub forward: bool,
    /// 起运年龄（3 日折 1 年）。
    pub start_age_years: f64,
    /// 十步大运。
    pub pillars: Vec<LuckPillar>,
}

/// 农历日期。
#[derive(Debug, Clone, Serialize)]
pub struct LunarChart {
    /// 农历年。
    pub year: i32,
    /// 月序 1..12。
    pub month: u32,
    /// 是否闰月。
    pub leap: bool,
    /// 日 1..30。
    pub day: u32,
}

/// 一张完整八字盘。
#[derive(Debug, Clone, Serialize)]
pub struct BaziChart {
    /// 排盘输入。
    pub input: BirthInput,
    /// 农历日期。
    pub lunar: LunarChart,
    /// 年柱。
    pub year: Pillar,
    /// 月柱。
    pub month: Pillar,
    /// 日柱。
    pub day: Pillar,
    /// 时柱。
    pub hour: Pillar,
    /// 日主（日干）。
    pub day_master: String,
    /// 日主五行。
    pub day_master_wuxing: String,
    /// 日柱旬空（空亡）两支（DET）。
    pub xunkong: [String; 2],
    /// 日主旺衰量化（得令+得地+得势 → 0–100 强弱条 + 五行力量分布）。🟡 权重为显式默认，流派分歧未开关。
    pub strength: Strength,
    /// 命格：月令藏干透干定型 — 八正格/建禄月刃/暗格。🟡 从/化/专旺格留 INT。
    pub pattern: Pattern,
    /// 用神/喜忌：扶抑+调候出主用神五行+副用神+忌神。用神所属五行被供给 = 吉，忌神被供给 = 凶。🟡 仅本算法显式默认，流派分歧未开关。
    pub yongshen: YongShen,
    /// 三宫：命宫/身宫/胎元 DET 公式（子平节气月支版）；🟡 紫微版基于农历月。
    pub three_houses: ThreeHouses,
    /// 大运（输入含性别时）。
    pub dayun: Option<DaYun>,
}

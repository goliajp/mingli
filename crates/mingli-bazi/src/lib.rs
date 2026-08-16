//! L3 叶：四柱推命（八字）排盘。
//!
//! 确定性「排盘」：用 `mingli-astro` 的天文/历法 + `mingli-ganzhi` 的干支符号，
//! 算出年/月/日/时四柱、十神、五行、农历、大运。年柱以立春为界，月柱以「节」换月，
//! 日柱由民用日序递推，时柱五鼠遁。不含「释义/文案」（那是表达层/LLM 的事）。

#![allow(

    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap,
    reason = "干支/大运均为小循环群上的有界模运算与天数折算，整数窄化安全"
)]

mod engine;
pub use engine::BaziEngine;

use mingli_astro::{solar_term_jd, solar_term_time_near, Moment};
use mingli_ganzhi::{
    branch_element, day_ganzhi, hidden_stems, hour_branch, is_friendly_to_day_master,
    is_kuigang_day, month_pillar_stem, nayin_element, shensha_by_branch_anchor,
    shensha_by_day_stem, stem_element, ten_god, twelve_stage, year_ganzhi, Element, GanZhi,
    BRANCHES, STEMS, TWELVE_STAGES,
};
pub use mingli_ganzhi::parse_ganzhi;
use serde::Serialize;

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

fn pillar(gz: GanZhi, day_master: u8, is_day: bool, year_branch: u8, day_gz: GanZhi) -> Pillar {
    // 神煞落到该柱（日干锚 + 年支锚 + 日柱魁罡）
    let mut shensha: Vec<String> = Vec::new();
    for &name in &shensha_by_day_stem(day_master, gz.branch) {
        shensha.push(name.to_string());
    }
    for &name in &shensha_by_branch_anchor(year_branch, gz.branch) {
        // 避免日干锚和年支锚同支重复
        let s = name.to_string();
        if !shensha.contains(&s) {
            shensha.push(s);
        }
    }
    if is_day && is_kuigang_day(day_gz) {
        shensha.push("魁罡".to_string());
    }
    Pillar {
        ganzhi: gz.to_string(),
        stem: STEMS[gz.stem as usize].to_string(),
        branch: BRANCHES[gz.branch as usize].to_string(),
        stem_wuxing: stem_element(gz.stem).name().to_string(),
        branch_wuxing: branch_element(gz.branch).name().to_string(),
        nayin: nayin_element(gz).name().to_string(),
        ten_god: if is_day {
            "日主".to_string()
        } else {
            ten_god(day_master, gz.stem).to_string()
        },
        hidden: hidden_stems(gz.branch)
            .iter()
            .map(|&hs| HiddenStem {
                stem: STEMS[hs as usize].to_string(),
                ten_god: ten_god(day_master, hs).to_string(),
            })
            .collect(),
        day_twelve: TWELVE_STAGES[twelve_stage(day_master, gz.branch) as usize].to_string(),
        shensha,
    }
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

/// 用神 / 喜忌：把旺衰 + 五行分布合起来给出「该补什么 / 该忌什么」。
///
/// **算法（扶抑学派 + 调候辅助，显式权重）**：
/// - **身强(score ≥ 60)** → 取耗身五行（官杀/财/食伤）；三类候选取**当前盘中分布最弱**者
///   为主用神（补缺最有效），次弱者为副用神；忌神 = 印（生身）+ 比劫（帮身）。
/// - **身弱(score ≤ 40)** → 取助身五行（印星/比劫）；**印星优先**（双重作用 — 生身 + 化杀生官），
///   比劫副选；忌神 = 官杀（克身） + 财（损印）。
/// - **中和(40 < score < 60)** → 走调候为主：寒月（亥子丑）取火、燥月（巳午未）取水、
///   春木月（寅卯）取金修剪、秋金月（申酉）取火炼；辰戌杂气取日主同行扶身。
///
/// 「同党」+「耗身」分类沿用 [`is_friendly_to_day_master`]。
///
/// 用神是命格 + 旺衰的**自然推论** — 命局所喜五行 = 用神 = 补之则吉；
/// 命局所忌五行 = 忌神 = 加强则凶。这是命理体系给出的吉凶判断方向。
///
/// **🟡 流派分歧**：取用神有扶抑/调候/通关/病药/格局用神五法，各家先后顺序不同；
/// 「从格 / 化格」反扶抑（扶其太过、抑其不及）本算法不覆盖。
#[derive(Debug, Clone, Serialize)]
pub struct YongShen {
    /// 取用法（扶抑·身强 / 扶抑·身弱 / 调候为主）。
    pub method: String,
    /// 主用神五行（命局最该补的）。
    pub primary_wuxing: String,
    /// 主用神对日主的角色（印星/比劫/官杀/财/食伤/调候）。
    pub primary_role: String,
    /// 副用神五行（次要扶抑）；调候法无副用神。
    pub secondary_wuxing: Option<String>,
    /// 副用神对日主的角色。
    pub secondary_role: Option<String>,
    /// 忌神五行（命局最该避的；调候法暂留空）。
    pub avoid_wuxing: Vec<String>,
    /// 推理链（短句解释为什么取这个用神）。
    pub reasoning: String,
}

/// 命格：月令藏干透干定的「命主类型」。
///
/// **主流通行规则（子平真诠/三命通会一致）**：
/// 1. **建禄 / 月刃**：月令本气 = 日主同五行（同阴阳=建禄、异阴阳=月刃/阳刃），不取八正格。
/// 2. **八正格**（正官/七杀/正财/偏财/正印/偏印/食神/伤官）：月令本气、中气、余气 **按序** 查
///    年/月/时三干头是否「透出」，先透先定；透干所对日主的十神 = 格局名。
/// 3. **暗格**：月令藏干都不透 → 取本气暗藏立格（传统称「八字纯藏」）。
///
/// 格局是命主的**结构性属性**（命理传统的「类型分类」），命主的吉凶倾向 = 格局类型 × 用神扶抑。
///
/// **🟡 流派分歧**：从格/化格/专旺格成立条件分歧大（身极弱+无救助+顺势依附，各家定义不一），
/// 杂气格（辰戌丑未）透干优先级、月令分日用事均有派别，本算法不机械化，留 INT 释义层。
#[derive(Debug, Clone, Serialize)]
pub struct Pattern {
    /// 格局名（正官格/七杀格/正财格/偏财格/正印格/偏印格/食神格/伤官格/建禄格/月刃格/暗 X 格）。
    pub name: String,
    /// 取格依据（中文短句：如「月令本气透干 月柱」「月令暗藏」「月令本气=日主同五行」）。
    pub source: String,
    /// 取格的藏干（月令本/中/余气）字面；禄刃情况为月令本气。
    pub qi_stem: String,
    /// 该藏干所属的气位（本气/中气/余气）。
    pub qi_kind: String,
    /// 透干所在柱（年柱/月柱/时柱）；未透为 None。
    pub revealed_in: Option<String>,
    /// 该藏干相对日主的十神。
    pub ten_god: String,
    /// 是否透干（true=明格、false=暗格或禄刃）。
    pub revealed: bool,
    /// 是否走「禄刃」特殊分支（比肩/劫财不入八正格）。
    pub is_lu_ren: bool,
}

/// 月令定格：按主流通行规则取八正格 / 建禄月刃 / 暗格。
///
/// 入参为本命四柱；不依赖出生时刻——纯符号层运算。
#[must_use]
pub fn determine_pattern(
    year_gz: GanZhi,
    month_gz: GanZhi,
    day_gz: GanZhi,
    hour_gz: GanZhi,
) -> Pattern {
    let dm = day_gz.stem;
    let mb = month_gz.branch;
    let hidden = hidden_stems(mb);
    let main_qi = hidden[0];

    // 特殊分支：月令本气 = 日主同五行 → 建禄（同阴阳） / 月刃（异阴阳）
    if stem_element(main_qi) == stem_element(dm) {
        let same_polarity = (main_qi % 2) == (dm % 2);
        let name = if same_polarity { "建禄格" } else { "月刃格" };
        return Pattern {
            name: name.to_string(),
            source: "月令本气=日主同五行（比劫不入八正格）".to_string(),
            qi_stem: STEMS[main_qi as usize].to_string(),
            qi_kind: "本气".to_string(),
            revealed_in: None,
            ten_god: ten_god(dm, main_qi).to_string(),
            revealed: false,
            is_lu_ren: true,
        };
    }

    // 八正格：按本→中→余 查年/月/时透干。日主自身不算「透」。
    let pillars: [(&str, u8); 3] = [
        ("年柱", year_gz.stem),
        ("月柱", month_gz.stem),
        ("时柱", hour_gz.stem),
    ];
    let qi_names = ["本气", "中气", "余气"];

    for (i, &qi) in hidden.iter().enumerate() {
        if let Some(&(pname, _)) = pillars.iter().find(|&&(_, st)| st == qi) {
            let god = ten_god(dm, qi);
            return Pattern {
                name: format!("{god}格"),
                source: format!("月令{}透干 {}", qi_names[i.min(2)], pname),
                qi_stem: STEMS[qi as usize].to_string(),
                qi_kind: qi_names[i.min(2)].to_string(),
                revealed_in: Some(pname.to_string()),
                ten_god: god.to_string(),
                revealed: true,
                is_lu_ren: false,
            };
        }
    }

    // 全不透：取本气暗藏立格
    let god = ten_god(dm, main_qi);
    Pattern {
        name: format!("暗{god}格"),
        source: "月令本气暗藏（年/月/时三干头无月令藏干）".to_string(),
        qi_stem: STEMS[main_qi as usize].to_string(),
        qi_kind: "本气".to_string(),
        revealed_in: None,
        ten_god: god.to_string(),
        revealed: false,
        is_lu_ren: false,
    }
}

/// 团队五行画像：N 个本命盘五行向量平均 → 团队整体五行分布(%)。
/// 用于「合盘 / 团队」：看整支队伍五行结构是否均衡。
#[must_use]
pub fn team_wuxing_average(members: &[BaziChart]) -> WuxingPower {
    if members.is_empty() {
        return WuxingPower { wood: 0, fire: 0, earth: 0, metal: 0, water: 0 };
    }
    let n = members.len() as u32;
    let sum_w = members.iter().map(|c| c.strength.wuxing.wood).sum::<u32>();
    let sum_f = members.iter().map(|c| c.strength.wuxing.fire).sum::<u32>();
    let sum_e = members.iter().map(|c| c.strength.wuxing.earth).sum::<u32>();
    let sum_m = members.iter().map(|c| c.strength.wuxing.metal).sum::<u32>();
    let sum_wa = members.iter().map(|c| c.strength.wuxing.water).sum::<u32>();
    WuxingPower {
        wood: sum_w / n,
        fire: sum_f / n,
        earth: sum_e / n,
        metal: sum_m / n,
        water: sum_wa / n,
    }
}

/// 互补度：成员 j 的盘中，成员 i 主用神五行的占比(%)。值越高 = j 越能补 i 的缺。
///
/// 用于团队互补矩阵 N×N 的每一格。
#[must_use]
pub fn complement_score(i_yongshen_wuxing: &str, j_wuxing: &WuxingPower) -> u32 {
    match i_yongshen_wuxing {
        "木" => j_wuxing.wood,
        "火" => j_wuxing.fire,
        "土" => j_wuxing.earth,
        "金" => j_wuxing.metal,
        "水" => j_wuxing.water,
        _ => 0,
    }
}

/// 五行索引 0..5 → 中文（用于按团队画像取最弱/最旺项的字面）。
const WX_NAMES: [&str; 5] = ["木", "火", "土", "金", "水"];

/// 取团队五行最弱的项 (name， pct)。
#[must_use]
pub fn team_weakest(wx: &WuxingPower) -> (String, u32) {
    let arr = [wx.wood, wx.fire, wx.earth, wx.metal, wx.water];
    let (idx, &v) = arr.iter().enumerate().min_by_key(|&(_, &v)| v).unwrap_or((0, &0));
    (WX_NAMES[idx].to_string(), v)
}

/// 取团队五行最旺的项 (name， pct)。
#[must_use]
pub fn team_strongest(wx: &WuxingPower) -> (String, u32) {
    let arr = [wx.wood, wx.fire, wx.earth, wx.metal, wx.water];
    let (idx, &v) = arr.iter().enumerate().max_by_key(|&(_, &v)| v).unwrap_or((0, &0));
    (WX_NAMES[idx].to_string(), v)
}

/// 三宫：命宫 / 身宫 / 胎元。子平传统 DET 公式。
///
/// **算法（各家通行）**：
/// - **命宫** = （月支 − 时支） mod 12 → 月支起子时**逆数**生时；天干由五虎遁（年干 → 命宫支）。
/// - **身宫** = （月支 + 时支） mod 12 → 月支起子时**顺数**生时；天干同样由五虎遁定。
/// - **胎元** = 月柱干 +1，月柱支 +3（怀胎十月范畴的「受气宫」）。
///
/// 注：命宫流派分歧——子平用**节气月支**（本算法），紫微/三命通会用**农历月**（数字 1..12）。
/// 本 crate 站在子平视角，与 [`mingli_ziwei`](crate::compute_with_true_solar) 的农历月命宫
/// 在「月支 ≠ 农历月对应支」的节气切换日会差一格。
#[derive(Debug, Clone, Serialize)]
pub struct ThreeHouses {
    /// 命宫干支（如「丙寅」）。
    pub ming_gong: String,
    /// 身宫干支（如「庚辰」）。
    pub shen_gong: String,
    /// 胎元干支（如「庚子」）。
    pub tai_yuan: String,
}

/// 取三宫：命宫/身宫/胎元（通行 DET 公式）。
#[must_use]
pub fn determine_three_houses(year_gz: GanZhi, month_gz: GanZhi, hour_b: u8) -> ThreeHouses {
    let ming_b = (month_gz.branch + 12 - hour_b) % 12;
    let ming_s = month_pillar_stem(year_gz.stem, ming_b);
    let shen_b = (month_gz.branch + hour_b) % 12;
    let shen_s = month_pillar_stem(year_gz.stem, shen_b);
    let tai_s = (month_gz.stem + 1) % 10;
    let tai_b = (month_gz.branch + 3) % 12;
    let fmt = |s: u8, b: u8| format!("{}{}", STEMS[s as usize], BRANCHES[b as usize]);
    ThreeHouses {
        ming_gong: fmt(ming_s, ming_b),
        shen_gong: fmt(shen_s, shen_b),
        tai_yuan: fmt(tai_s, tai_b),
    }
}

/// 是否闰年（公历）。
const fn is_leap_year(y: i32) -> bool {
    y % 4 == 0 && (y % 100 != 0 || y % 400 == 0)
}

const DAYS_IN_MONTH: [u32; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

const fn days_in_month(y: i32, m: u32) -> u32 {
    if m == 2 && is_leap_year(y) { 29 } else { DAYS_IN_MONTH[(m - 1) as usize] }
}

/// 公历日期 +Δ 天（可正可负，跨月/跨年自动处理）。
fn add_days_civil(y: i32, m: u32, d: u32, delta: i32) -> (i32, u32, u32) {
    let mut y = y;
    let mut m = m;
    let mut d = d as i32 + delta;
    while d < 1 {
        m = if m == 1 { y -= 1; 12 } else { m - 1 };
        d += days_in_month(y, m) as i32;
    }
    while d > days_in_month(y, m) as i32 {
        d -= days_in_month(y, m) as i32;
        m = if m == 12 { y += 1; 1 } else { m + 1 };
    }
    (y, m, d as u32)
}

/// 公历年内日序 1..366。
fn day_of_year(y: i32, m: u32, d: u32) -> u32 {
    const CUMUL: [u32; 12] = [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334];
    let mut n = CUMUL[(m - 1) as usize] + d;
    if is_leap_year(y) && m > 2 {
        n += 1;
    }
    n
}

/// 均时差（分钟，Spencer/Iqbal 简化公式，精度 ±0.5 min，足够时辰判定）。
///
/// 真太阳时与平太阳时的差(EoT) = 9.87 sin(2B) − 7.53 cos(B) − 1.5 sin(B)，
/// 其中 B = 2π(N−81)/365、N = 年内日序(1..366)。
#[must_use]
pub fn equation_of_time_minutes(year: i32, month: u32, day: u32) -> f64 {
    let n = f64::from(day_of_year(year, month, day));
    let b = 2.0 * std::f64::consts::PI * (n - 81.0) / 365.0;
    9.87 * (2.0 * b).sin() - 7.53 * b.cos() - 1.5 * b.sin()
}

/// 真太阳时相对钟表时的总偏移（分钟，正=真太阳时较钟表晚）。
///
/// 由两部分组成：① 经度差（出生地经度 − 时区标准经线）× 4 分钟/度；② [`equation_of_time_minutes`] 均时差。
#[must_use]
pub fn true_solar_offset_minutes(
    longitude: f64,
    tz_hours: f64,
    year: i32,
    month: u32,
    day: u32,
) -> f64 {
    let std_longitude = tz_hours * 15.0;
    let geo_correction = (longitude - std_longitude) * 4.0;
    geo_correction + equation_of_time_minutes(year, month, day)
}

/// 真太阳时排盘：按出生地经度 + EoT 校正钟表时，再排八字。
///
/// 钟表时 → 真太阳时差（±约 30 分钟内典型）；跨时辰边界时，时柱与钟表版排盘不同。
#[must_use]
pub fn compute_with_true_solar(input: BirthInput, longitude: f64) -> BaziChart {
    let offset = true_solar_offset_minutes(
        longitude, input.tz, input.year, input.month, input.day,
    );
    let offset_min = offset.round() as i32;
    let total = input.hour as i32 * 60 + input.minute as i32 + offset_min;
    let (day_delta, in_day_min) = if total < 0 {
        (-1, total + 24 * 60)
    } else if total >= 24 * 60 {
        (1, total - 24 * 60)
    } else {
        (0, total)
    };
    let (ny, nm, nd) = if day_delta == 0 {
        (input.year, input.month, input.day)
    } else {
        add_days_civil(input.year, input.month, input.day, day_delta)
    };
    let nh = (in_day_min / 60) as u32;
    let nmin = (in_day_min % 60) as u32;
    let moment = Moment::new(ny, nm, nd, nh, nmin, input.tz);
    compute_at(&moment, input.gender)
}

/// 反查「生我」的五行（印星五行 = X 满足 X.generates() == dm）。
const fn yin_xing_of(dm: Element) -> Element {
    match dm {
        Element::Wood => Element::Water,
        Element::Fire => Element::Wood,
        Element::Earth => Element::Fire,
        Element::Metal => Element::Earth,
        Element::Water => Element::Metal,
    }
}

/// 反查「克我」的五行（官杀五行 = X 满足 X.controls() == dm）。
const fn guan_sha_of(dm: Element) -> Element {
    match dm {
        Element::Wood => Element::Metal,
        Element::Fire => Element::Water,
        Element::Earth => Element::Wood,
        Element::Metal => Element::Fire,
        Element::Water => Element::Earth,
    }
}

/// 在 [`WuxingPower`] 上按 [`Element`] 取百分比。
fn wx_pct(wx: &WuxingPower, e: Element) -> u32 {
    match e {
        Element::Wood => wx.wood,
        Element::Fire => wx.fire,
        Element::Earth => wx.earth,
        Element::Metal => wx.metal,
        Element::Water => wx.water,
    }
}

/// 取用神：旺衰 + 五行分布 → 主/副用神五行 + 忌神。
///
/// 见 [`YongShen`] 文档说明算法与流派分歧。
#[must_use]
pub fn determine_yongshen(
    day_master_stem: u8,
    month_branch: u8,
    strength: &Strength,
) -> YongShen {
    let dm_e = stem_element(day_master_stem);
    let bijie = dm_e;
    let yin = yin_xing_of(dm_e);
    let guan = guan_sha_of(dm_e);
    let cai = dm_e.controls();
    let shishang = dm_e.generates();
    let score = strength.score;
    let dm_name = STEMS[day_master_stem as usize];

    if score >= 60 {
        // 身强：取耗身。三候选按盘中分布升序排，最弱者为主用（补缺最有效）。
        let mut candidates: [(Element, &'static str); 3] = [
            (guan, "官杀"),
            (cai, "财"),
            (shishang, "食伤"),
        ];
        candidates.sort_by_key(|&(e, _)| wx_pct(&strength.wuxing, e));
        let (p_e, p_r) = candidates[0];
        let (s_e, s_r) = candidates[1];
        YongShen {
            method: "扶抑 · 身强宜耗".to_string(),
            primary_wuxing: p_e.name().to_string(),
            primary_role: p_r.to_string(),
            secondary_wuxing: Some(s_e.name().to_string()),
            secondary_role: Some(s_r.to_string()),
            avoid_wuxing: vec![yin.name().to_string(), bijie.name().to_string()],
            reasoning: format!(
                "日主{}{}（综合 {}），宜以耗身五行抑之；三候选（官杀{}/财{}/食伤{}）中{}{}最缺({}%)，补之最有效。忌印星{}生身、比劫{}帮身。",
                dm_name, strength.level, score,
                guan.name(), cai.name(), shishang.name(),
                p_r, p_e.name(), wx_pct(&strength.wuxing, p_e),
                yin.name(), bijie.name(),
            ),
        }
    } else if score <= 40 {
        // 身弱：取助身。印星优先（生身+化杀），比劫副选。
        YongShen {
            method: "扶抑 · 身弱宜扶".to_string(),
            primary_wuxing: yin.name().to_string(),
            primary_role: "印星".to_string(),
            secondary_wuxing: Some(bijie.name().to_string()),
            secondary_role: Some("比劫".to_string()),
            avoid_wuxing: vec![guan.name().to_string(), cai.name().to_string()],
            reasoning: format!(
                "日主{}{}（综合 {}），宜以助身五行扶之；印星{}双重作用（生身+化杀）优先，比劫{}副选。忌官杀{}克身、财{}损印。",
                dm_name, strength.level, score,
                yin.name(), bijie.name(), guan.name(), cai.name(),
            ),
        }
    } else {
        // 中和：调候为主，按月支寒燥取。
        let (target, note) = match month_branch {
            0..=1 | 11 => (Element::Fire, "亥子丑寒月 — 取火暖局"),
            5..=7 => (Element::Water, "巳午未燥月 — 取水润局"),
            2..=3 => (Element::Metal, "寅卯春木月 — 取金修剪"),
            8..=9 => (Element::Fire, "申酉秋金月 — 取火炼金"),
            _ => (dm_e, "辰戌杂气月 — 取日主同行扶身"),
        };
        YongShen {
            method: "调候为主".to_string(),
            primary_wuxing: target.name().to_string(),
            primary_role: "调候".to_string(),
            secondary_wuxing: None,
            secondary_role: None,
            avoid_wuxing: vec![],
            reasoning: format!(
                "日主{dm_name}中和（综合 {score}），扶抑余地小 → 看调候。{note}。",
            ),
        }
    }
}

/// 日主旺衰量化（扶抑学派范式）。
///
/// **算法（显式声明权重，🟡 权重值无统一标准）**：
/// - **得令(0–30)** = 日主在月支的十二长生分(`LING_TABLE`)+ 月支藏干同党加成（本气+5/中气+3/余气+1）
/// - **得地(0–30)** = 年/日/时三支藏干中同党（比劫/印）按本气×9+中气×5+余气×3 加权，封顶 30
/// - **得势(0–30)** = 年/月/时三干头（非日主）中每个同党 +10
/// - **总分** = 三栏之和(0..90)× 100 ÷ 90，封顶 100。
///
/// 「同党」= 比劫（同五行）+ 印星（生我五行），见 [`is_friendly_to_day_master`]。
/// 「耗身」= 食伤/财/官杀，本算法仅做正向加分，不做反向扣分(三栏直观可读；
///  扣分制等同净值，但黑盒化)。
///
/// **🟡 流派分歧**：权重值传统命理书未给统一表(《滴天髓》《子平真诠》《盲派》各家月令
///  权重 30%–60% 不一)。用神/扶抑会以本结果为输入。
///
/// **诚实**：量化是辅助判断，非定论；阈值（强/偏强/中和/偏弱/弱）亦为常见区分。
#[derive(Debug, Clone, Serialize)]
pub struct Strength {
    /// 综合强弱 0–100。
    pub score: u32,
    /// 等级（强/偏强/中和/偏弱/弱）。
    pub level: String,
    /// 得令（月支）0–30。
    pub got_ling: u32,
    /// 得地（通根）0–30。
    pub got_di: u32,
    /// 得势（干头）0–30。
    pub got_shi: u32,
    /// 五行力量分布（百分比，合 100）。
    pub wuxing: WuxingPower,
}

/// 五行力量分布（百分比，合 100）。
///
/// 权重：天干 10、地支本气 12、中气 6、余气 3；月支×1.5（得令加成）。
#[derive(Debug, Clone, Copy, Serialize)]
pub struct WuxingPower {
    /// 木 %。
    pub wood: u32,
    /// 火 %。
    pub fire: u32,
    /// 土 %。
    pub earth: u32,
    /// 金 %。
    pub metal: u32,
    /// 水 %。
    pub water: u32,
}

/// 日主在月支的十二长生阶段「得令」分(0..30)。
/// 帝旺/临官最旺，死/绝最弱。索引同 [`mingli_ganzhi::TWELVE_STAGES`]。
const LING_TABLE: [u32; 12] = [
    20, // 0 长生
    8,  // 1 沐浴
    18, // 2 冠带
    27, // 3 临官
    30, // 4 帝旺
    10, // 5 衰
    6,  // 6 病
    3,  // 7 死
    8,  // 8 墓
    1,  // 9 绝
    10, // 10 胎
    15, // 11 养
];

/// 在本命四柱基础上叠加额外干支（岁运叠加旺衰：大运柱、流年柱等）算 t 时刻的实际旺衰。
///
/// 「得令」固定取本命月支（月令是出生即定的天时，大运流年不改）；「得地」「得势」「五行分布」
/// 把 extras 拼入加权；封顶 30 不变（外力把得地/得势推得更接近满分，但不超）。
/// 综合分仍以 90 为基线归一到 100，可解释「外力让本命旺衰偏向更强还是更弱」。
///
/// **🟡 流派分歧**：岁运叠加权重无统一标准，本算法把 extras 当作与本命三干/三支同质量级的「外援」（每干10、每支按本/中/余 9/5/3）；
/// 盲派/新派对岁运折扣不一（有给 ½ 的、有完全等同的）。
#[must_use]
pub fn compute_strength_with_extras(
    year_gz: GanZhi,
    month_gz: GanZhi,
    day_gz: GanZhi,
    hour_gz: GanZhi,
    extras: &[GanZhi],
) -> Strength {
    compute_strength_inner(year_gz, month_gz, day_gz, hour_gz, extras)
}

fn compute_strength(year_gz: GanZhi, month_gz: GanZhi, day_gz: GanZhi, hour_gz: GanZhi) -> Strength {
    compute_strength_inner(year_gz, month_gz, day_gz, hour_gz, &[])
}

fn compute_strength_inner(
    year_gz: GanZhi,
    month_gz: GanZhi,
    day_gz: GanZhi,
    hour_gz: GanZhi,
    extras: &[GanZhi],
) -> Strength {
    let dm = day_gz.stem;

    // 得令 = 月支十二长生分 + 月支藏干同党加成
    let stage = twelve_stage(dm, month_gz.branch) as usize;
    let mut got_ling = LING_TABLE[stage];
    for (i, &s) in hidden_stems(month_gz.branch).iter().enumerate() {
        if is_friendly_to_day_master(dm, s) {
            got_ling += [5, 3, 1][i.min(2)];
        }
    }
    got_ling = got_ling.min(30);

    // 得地 = 年/日/时三支 + extras 地支 藏干同党加权（本=9/中=5/余=3），月支已计入得令不重算。
    let mut got_di = 0u32;
    let di_branches = [year_gz.branch, day_gz.branch, hour_gz.branch];
    for &br in di_branches.iter().chain(extras.iter().map(|g| &g.branch)) {
        for (i, &s) in hidden_stems(br).iter().enumerate() {
            if is_friendly_to_day_master(dm, s) {
                got_di += [9, 5, 3][i.min(2)];
            }
        }
    }
    got_di = got_di.min(30);

    // 得势 = 年/月/时三干头 + extras 天干 同党各 +10（日主本身不算）。
    let mut got_shi = 0u32;
    let shi_stems = [year_gz.stem, month_gz.stem, hour_gz.stem];
    for &st in shi_stems.iter().chain(extras.iter().map(|g| &g.stem)) {
        if is_friendly_to_day_master(dm, st) {
            got_shi += 10;
        }
    }
    got_shi = got_shi.min(30);

    let raw = got_ling + got_di + got_shi;
    let score = (raw * 100 / 90).min(100);
    let level = if score >= 75 {
        "强"
    } else if score >= 60 {
        "偏强"
    } else if score >= 40 {
        "中和"
    } else if score >= 25 {
        "偏弱"
    } else {
        "弱"
    }
    .to_string();

    // 五行力量分布：天干 10、地支本气 12/中 6/余 3，月支 ×1.5。extras 干支与本命同质量级入。
    let mut wx = [0u32; 5];
    let stems = [year_gz.stem, month_gz.stem, day_gz.stem, hour_gz.stem];
    for &st in stems.iter().chain(extras.iter().map(|g| &g.stem)) {
        wx[stem_element(st).index()] += 10;
    }
    let branches = [year_gz.branch, month_gz.branch, day_gz.branch, hour_gz.branch];
    for (j, &br) in branches.iter().enumerate() {
        let is_month = j == 1;
        for (i, &s) in hidden_stems(br).iter().enumerate() {
            let base = [12u32, 6, 3][i.min(2)];
            let w = if is_month { base * 3 / 2 } else { base };
            wx[stem_element(s).index()] += w;
        }
    }
    for g in extras {
        for (i, &s) in hidden_stems(g.branch).iter().enumerate() {
            wx[stem_element(s).index()] += [12u32, 6, 3][i.min(2)];
        }
    }
    // 总权重必 ≥ 4*10=40（四个天干本身），整数除安全。
    let total: u32 = wx.iter().sum();
    let norm = |v: u32| (v * 100 + total / 2) / total;
    let wuxing = WuxingPower {
        wood: norm(wx[0]),
        fire: norm(wx[1]),
        earth: norm(wx[2]),
        metal: norm(wx[3]),
        water: norm(wx[4]),
    };

    Strength {
        score,
        level,
        got_ling,
        got_di,
        got_shi,
        wuxing,
    }
}

/// 排八字（独立入口：自行构造共享上下文 [`Moment`]）。
#[must_use]
pub fn compute(input: BirthInput) -> BaziChart {
    let m = Moment::new(
        input.year,
        input.month,
        input.day,
        input.hour,
        input.minute,
        input.tz,
    );
    compute_at(&m, input.gender)
}

/// 子时归属流派（影响 23：00–23：59 出生的日柱）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ZiHourMethod {
    /// **晚子（Late，主流）**：子时整体属次日，23-24 点 → 次日日柱。
    Late,
    /// **早子（Early，传统少数派）**：23-24 点仍属当日，称为「夜子」；0-1 点称「正子」次日。
    Early,
}

/// 年柱换岁流派（影响立春前/正月初一前出生的年柱）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum YearBreakMethod {
    /// **立春换年（主流）**：节气立春（太阳黄经 315°）为新年界。子平命理主流。
    LiChun,
    /// **春节换年（民间少数派）**：农历正月初一为新年界。民俗/择吉派偶用。
    SpringFestival,
}

/// 八字流派全集。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct BaziSchool {
    /// 子时归属流派。
    pub zi_hour: ZiHourMethod,
    /// 年柱换岁流派。
    pub year_break: YearBreakMethod,
}

impl Default for BaziSchool {
    fn default() -> Self {
        Self { zi_hour: ZiHourMethod::Late, year_break: YearBreakMethod::LiChun }
    }
}

/// 在已算好的共享上下文上排八字（指定子时流派，年柱仍用主流立春）。向后兼容。
#[must_use]
pub fn compute_at_with(m: &Moment, gender: Option<Gender>, zi: ZiHourMethod) -> BaziChart {
    compute_at_school(m, gender, BaziSchool { zi_hour: zi, year_break: YearBreakMethod::LiChun })
}

/// 在已算好的共享上下文上排八字（完整流派指定）。
#[must_use]
pub fn compute_at_school(m: &Moment, gender: Option<Gender>, school: BaziSchool) -> BaziChart {
    compute_at_impl(m, gender, school)
}

/// 在已算好的共享上下文 [`Moment`] 上排八字——供 DAG 引擎复用同一 `Moment`、零重算天文（默认 [`BaziSchool::default`]）。
#[must_use]
pub fn compute_at(m: &Moment, gender: Option<Gender>) -> BaziChart {
    compute_at_impl(m, gender, BaziSchool::default())
}

fn compute_at_impl(m: &Moment, gender: Option<Gender>, school: BaziSchool) -> BaziChart {
    let zi = school.zi_hour;
    let (jd, lam) = (m.jd_ut, m.sun_longitude);

    // 年柱：换岁流派——主流立春（节气黄经 315°）；少数派春节（农历正月初一）。
    let solar_year = match school.year_break {
        YearBreakMethod::LiChun => {
            let lichun = solar_term_jd(m.year, 315.0);
            if jd < lichun { m.year - 1 } else { m.year }
        }
        YearBreakMethod::SpringFestival => {
            // 农历正月初一（非闰）之前归前一年；之后（含）归本年。
            // m.lunar.month=1 且 m.lunar.day>=1 且 leap=false → 已到正月，本公历年成立。
            // 若 m.lunar.month=12 或 （month=1 day>=1 但 leap=true 跨闰），则尚未到正月初一，归前一年。
            let l = &m.lunar;
            if l.month == 1 && !l.leap && l.day >= 1 {
                m.year
            } else if l.month >= 11 || (l.month == 12) || (l.month == 1 && l.leap) {
                // 公历 1 月 1 日到农历正月初一之间（必落在公历 1-2 月）
                m.year - 1
            } else {
                m.year
            }
        }
    };
    let year_gz = year_ganzhi(solar_year);

    // 月柱：以「节」换月。s=0 → 寅月（立春起）。
    let s = ((lam - 315.0).rem_euclid(360.0) / 30.0).floor() as u8;
    let month_branch = (2 + s) % 12;
    let month_gz = GanZhi {
        stem: month_pillar_stem(year_gz.stem, month_branch),
        branch: month_branch,
    };

    // 日柱：共享上下文的民用日序 → 干支锚点递推
    // 子时流派：晚子（默认）= 23-24 点出生归次日；早子（传统少数派）= 仍归当日。
    let day_jdn = match zi {
        ZiHourMethod::Late if m.hour == 23 => m.civil_day + 1,
        _ => m.civil_day,
    };
    let day_gz = day_ganzhi(day_jdn);

    // 时柱：五鼠遁
    let hb = hour_branch(m.hour, m.minute);
    let hour_gz = GanZhi {
        stem: ((day_gz.stem % 5) * 2 + hb) % 10,
        branch: hb,
    };

    let dm = day_gz.stem;
    let lunar = m.lunar;
    let dayun = gender.map(|g| compute_dayun(jd, lam, year_gz.stem, g, month_gz));
    let strength = compute_strength(year_gz, month_gz, day_gz, hour_gz);
    let pattern = determine_pattern(year_gz, month_gz, day_gz, hour_gz);
    let yongshen = determine_yongshen(day_gz.stem, month_gz.branch, &strength);
    let three_houses = determine_three_houses(year_gz, month_gz, hb);

    BaziChart {
        input: BirthInput {
            year: m.year,
            month: m.month,
            day: m.day,
            hour: m.hour,
            minute: m.minute,
            tz: m.tz,
            gender,
        },
        lunar: LunarChart {
            year: lunar.year,
            month: lunar.month,
            leap: lunar.leap,
            day: lunar.day,
        },
        year: pillar(year_gz, dm, false, year_gz.branch, day_gz),
        month: pillar(month_gz, dm, false, year_gz.branch, day_gz),
        day: pillar(day_gz, dm, true, year_gz.branch, day_gz),
        hour: pillar(hour_gz, dm, false, year_gz.branch, day_gz),
        day_master: STEMS[dm as usize].to_string(),
        day_master_wuxing: stem_element(dm).name().to_string(),
        xunkong: {
            // 旬空：日柱所在 6 旬中，10 干配 12 支余下的 2 支。沿用 ganzhi 主干层 helper。
            let kong = mingli_ganzhi::xunkong(day_gz);
            [BRANCHES[kong[0] as usize].to_string(), BRANCHES[kong[1] as usize].to_string()]
        },
        strength,
        pattern,
        yongshen,
        three_houses,
        dayun,
    }
}

/// 大运：阳男阴女顺行、阴男阳女逆行；起运 = 到前/后一「节」的天数 ÷ 3 年。
fn compute_dayun(jd: f64, lam: f64, year_stem: u8, gender: Gender, month_gz: GanZhi) -> DaYun {
    let year_yang = year_stem.is_multiple_of(2); // 甲丙戊庚壬 为阳年
    let forward = match gender {
        Gender::Male => year_yang,
        Gender::Female => !year_yang,
    };

    // 「节」黄经 ≡ 15 (mod 30)。求紧邻的前/后一个节。
    let k = ((lam - 15.0) / 30.0).floor();
    let next_target = 15.0 + 30.0 * (k + 1.0);
    let prev_target = 15.0 + 30.0 * k;
    let next_jd =
        solar_term_time_near(jd + (next_target - lam).rem_euclid(360.0) / 0.98565, next_target);
    let prev_jd =
        solar_term_time_near(jd - (lam - prev_target).rem_euclid(360.0) / 0.98565, prev_target);

    let days = if forward { next_jd - jd } else { jd - prev_jd };
    let start_age_years = (days / 3.0).max(0.0);
    let start_age0 = start_age_years.round() as u32;

    let mut pillars = Vec::with_capacity(10);
    let m_idx = i32::from(month_gz.index());
    for i in 1..=10i32 {
        let idx = (if forward { m_idx + i } else { m_idx - i }).rem_euclid(60) as u8;
        pillars.push(LuckPillar {
            start_age: start_age0 + (i as u32 - 1) * 10,
            ganzhi: GanZhi::from_index(idx).to_string(),
        });
    }

    DaYun {
        forward,
        start_age_years: (start_age_years * 100.0).round() / 100.0,
        pillars,
    }
}

// ============================================================================
// Fortune：t 时刻运势切片（本命 + 大运 + 流年叠加旺衰 + 用神供给度） + 100 年时间序列
// 「拨杆 → 运势 → 用神供给」是旺衰 / 岁运叠加 / 用神在 t 时刻的统一聚合，
// 把「用神喜忌」从静态（出生即定的喜什么）升级为动态（t 时刻拿到多少 / 未来 100 年曲线）。
// ============================================================================

/// 吉凶判读（净增益分级）：由用神供给度计算 5 等级。
///
/// **算法**（基于命局所喜/所忌）：
/// `net = primary_pct + 0.5*secondary_pct − max_avoid_pct`
/// - 大吉：net ≥ +15（主用神远超忌神）
/// - 吉  ：+5 ≤ net < +15（主用神略胜）
/// - 平  ：-5 < net < +5（平衡）
/// - 凶  ：-15 < net ≤ -5（忌神略胜）
/// - 大凶：net ≤ -15（忌神远超）
#[derive(Debug, Clone, Serialize)]
pub struct Judgment {
    /// 吉凶等级字面（大吉/吉/平/凶/大凶）。
    pub level: String,
    /// 净增益分（=主用神 + 0.5*副用神 − 最高忌神，可正可负）。
    pub score: i32,
    /// 一句话判读（基于结构事实给出有利/不利说明）。
    pub summary: String,
}

/// 判读核心：由 （主供给， 副供给， 最高忌神供给） → Judgment。
fn judge_from_supplies(primary: u32, secondary: Option<u32>, max_avoid: u32) -> Judgment {
    let p = i32::try_from(primary).unwrap_or(0);
    let s = secondary.and_then(|v| i32::try_from(v).ok()).unwrap_or(0);
    let a = i32::try_from(max_avoid).unwrap_or(0);
    let net = p + s / 2 - a;
    let (level, summary) = if net >= 15 {
        (
            "大吉",
            format!("主用神 {p}% 远超忌神 {a}%，流年大运对命局所喜五行供给充足，利于发展、决策、行动。"),
        )
    } else if net >= 5 {
        (
            "吉",
            format!("主用神 {p}% 略胜忌神 {a}%，流年大运扶持有力，顺势而为有利。"),
        )
    } else if net > -5 {
        (
            "平",
            format!("主用神 {p}% 与忌神 {a}% 相当，流年大运无明显加持也无明显损耗，守成之时。"),
        )
    } else if net > -15 {
        (
            "凶",
            format!("忌神 {a}% 略胜主用神 {p}%，流年大运对命局所忌五行偏强，谨慎决策、避免冒进。"),
        )
    } else {
        (
            "大凶",
            format!("忌神 {a}% 远超主用神 {p}%，流年大运对命局压制明显，宜守不宜攻、稳健渡过。"),
        )
    };
    Judgment {
        level: level.to_string(),
        score: net,
        summary,
    }
}

/// 五行名 → `WuxingPower` 字段查询。未知名返回 0（防御性 — 调用方应只用 5 标准名）。
fn wuxing_pct_by_name(w: &WuxingPower, name: &str) -> u32 {
    match name {
        "木" => w.wood,
        "火" => w.fire,
        "土" => w.earth,
        "金" => w.metal,
        "水" => w.water,
        _ => 0,
    }
}

/// 从大运 timeline 按年龄挑活动步。`pillars[i]` 在 `[start_age_i, start_age_{i+1})` 内活动；
/// 末步对未来无截断（传统大运十步即百年覆盖）。
fn active_dayun_step(dayun: Option<&DaYun>, age_years: f64) -> Option<(usize, String)> {
    let d = dayun?;
    let mut chosen: Option<(usize, &LuckPillar)> = None;
    for (i, p) in d.pillars.iter().enumerate() {
        if f64::from(p.start_age) <= age_years {
            chosen = Some((i, p));
        }
    }
    chosen.map(|(i, p)| (i, p.ganzhi.clone()))
}

/// Fortune：t 时刻运势切片。
///
/// 把本命旺衰 / 命格 / 岁运叠加 / 用神/喜忌 在 t 时刻一次给齐，
/// 供 web Fortune 视图直接渲染「拨杆动 → 运层动」的运势画面。
///
/// **算法**：
/// 1. 本命盘 = `compute(natal_input)`（见 [`compute`]）— 出生切片（固定底图）。
/// 2. t 时刻盘 = [`compute_at`] 在 `(t_year,..,t_tz)` 的 Moment — t 流年/流月/流日/流时四柱。
/// 3. 当前大运步 = 从本命大运 timeline 按年龄挑（三日折一年起运 / 每步十年）。
/// 4. 运层旺衰 = [`compute_strength_with_extras`]（本命四柱， extras=[当前大运柱， t 流年柱]）。
/// 5. 用神供给度 = `yun_strength.wuxing[本命主用神五行] / [副用神] / [忌神...]`。
///
/// 主用神供给度高 = t 时刻拿到喜用多 = **吉**；忌神供给度高 = **凶**。
#[derive(Debug, Clone, Serialize)]
pub struct FortuneAt {
    /// 本命盘（出生切片，固定底图）。
    pub natal: BaziChart,
    /// t 时刻 bazi 盘（t 流年/流月/流日/流时四柱）。
    pub t_chart: BaziChart,
    /// t 时刻年龄（从 natal 出生时刻到 t_target 的浮点年）。
    pub age_years: f64,
    /// 当前活动大运步 index（本命大运 timeline `pillars[step]`）；无大运 → None。
    pub dayun_step: Option<usize>,
    /// 当前活动大运干支字面（同上）。
    pub dayun_ganzhi: Option<String>,
    /// t 时刻流年干支（取自 t_chart 年柱）。
    pub flow_year_ganzhi: String,
    /// 本命旺衰（等于 natal.strength）。
    pub ming_strength: Strength,
    /// 运层旺衰（本命 + 当前大运柱 + t 流年柱叠加）。
    pub yun_strength: Strength,
    /// 综合分差 = yun_strength.score − ming_strength.score（可正可负）。
    pub delta_score: i32,
    /// t 时刻主用神供给度 %（运层五行分布对本命主用神五行的占比）。
    pub primary_supply_pct: u32,
    /// t 时刻副用神供给度；调候法无副用神 → None。
    pub secondary_supply_pct: Option<u32>,
    /// t 时刻各忌神供给度，长度 = natal.yongshen.avoid_wuxing。
    pub avoid_supply_pcts: Vec<u32>,
    /// t 时刻吉凶判读（由 primary/secondary/avoid 供给度量化）。
    pub judgment: Judgment,
}

/// Fortune 入口：给定本命输入 + 目标时刻，聚合返回运势切片。
///
/// # Panics
///
/// 不会发生：本命四柱字面由内部 `compute()` 产出，必为合法干支，[`parse_ganzhi`] 解析永远成功。
#[must_use]
pub fn fortune_at(
    natal_input: BirthInput,
    t_year: i32,
    t_month: u32,
    t_day: u32,
    t_hour: u32,
    t_minute: u32,
    t_tz: f64,
) -> FortuneAt {
    let natal = compute(natal_input);
    let t_moment = Moment::new(t_year, t_month, t_day, t_hour, t_minute, t_tz);
    let t_chart = compute_at(&t_moment, natal_input.gender);

    // 年龄（浮点年）：简化用儒略日差 / 365.25。
    let birth_moment = Moment::new(
        natal_input.year, natal_input.month, natal_input.day,
        natal_input.hour, natal_input.minute, natal_input.tz,
    );
    let age_years = ((t_moment.jd_ut - birth_moment.jd_ut) / 365.25).max(0.0);

    let dayun_active = active_dayun_step(natal.dayun.as_ref(), age_years);
    let dayun_step = dayun_active.as_ref().map(|(i, _)| *i);
    let dayun_ganzhi = dayun_active.as_ref().map(|(_, gz)| gz.clone());

    let flow_year_ganzhi = t_chart.year.ganzhi.clone();

    // extras：本命四柱 + 当前大运柱 + t 流年柱（若解析成功）。
    let mut extras: Vec<GanZhi> = Vec::with_capacity(2);
    if let Some((_, ref gz_s)) = dayun_active
        && let Some(g) = parse_ganzhi(gz_s)
    {
        extras.push(g);
    }
    if let Some(g) = parse_ganzhi(&flow_year_ganzhi) { extras.push(g); }

    // 本命四柱 GanZhi（从 natal 重建，或用 t_chart 不行 — 必须用 natal 的）。
    let n_year = parse_ganzhi(&natal.year.ganzhi).expect("natal year_gz 应可解析");
    let n_month = parse_ganzhi(&natal.month.ganzhi).expect("natal month_gz 应可解析");
    let n_day = parse_ganzhi(&natal.day.ganzhi).expect("natal day_gz 应可解析");
    let n_hour = parse_ganzhi(&natal.hour.ganzhi).expect("natal hour_gz 应可解析");
    let yun_strength = compute_strength_with_extras(n_year, n_month, n_day, n_hour, &extras);

    let delta_score = i32::try_from(yun_strength.score).unwrap_or(0)
        - i32::try_from(natal.strength.score).unwrap_or(0);

    let primary_supply_pct = wuxing_pct_by_name(&yun_strength.wuxing, &natal.yongshen.primary_wuxing);
    let secondary_supply_pct = natal.yongshen.secondary_wuxing.as_ref()
        .map(|w| wuxing_pct_by_name(&yun_strength.wuxing, w));
    let avoid_supply_pcts: Vec<u32> = natal.yongshen.avoid_wuxing.iter()
        .map(|w| wuxing_pct_by_name(&yun_strength.wuxing, w))
        .collect();
    let max_avoid = avoid_supply_pcts.iter().copied().max().unwrap_or(0);
    let judgment = judge_from_supplies(primary_supply_pct, secondary_supply_pct, max_avoid);

    FortuneAt {
        ming_strength: natal.strength.clone(),
        natal,
        t_chart,
        age_years,
        dayun_step,
        dayun_ganzhi,
        flow_year_ganzhi,
        yun_strength,
        delta_score,
        primary_supply_pct,
        secondary_supply_pct,
        avoid_supply_pcts,
        judgment,
    }
}

/// 用神供给时间序列的一年点（供「100 年用神供给曲线」时序图）。
#[derive(Debug, Clone, Serialize)]
pub struct FortuneTimelinePoint {
    /// 年龄（整数岁，0..=max_age）。
    pub age: u32,
    /// 对应公历年（出生年 + age，以正月初一近似不细分）。
    pub year: i32,
    /// 该年流年干支。
    pub flow_year_ganzhi: String,
    /// 当前大运步 index。
    pub dayun_step: Option<usize>,
    /// 当前大运干支。
    pub dayun_ganzhi: Option<String>,
    /// 该年运层综合分(0..=100)。
    pub yun_score: u32,
    /// 主用神供给度 %。
    pub primary_supply_pct: u32,
    /// 副用神供给度 %；调候法无副用神 → None。
    pub secondary_supply_pct: Option<u32>,
    /// 该年最高忌神供给度 %（各忌神中 max，作单线展示便利）。
    pub avoid_supply_pct: u32,
    /// 该年吉凶判读（由 supply 度量化）。
    pub judgment: Judgment,
}

/// 扫描 `[0..=max_age]` 每年点的运势供给（主用神/副用神/忌神最高）。
///
/// **简化**：每年生日时刻锚定该年流年干支（出生月日同年内 1 个流年柱）；不细分流月/流日。
/// 大运按 `start_age` 整数比对取活动步。
///
/// # Panics
///
/// 不会发生：本命四柱字面由内部 `compute()` 产出，必为合法干支，[`parse_ganzhi`] 解析永远成功。
#[must_use]
pub fn fortune_supply_timeline(natal_input: BirthInput, max_age: u32) -> Vec<FortuneTimelinePoint> {
    let natal = compute(natal_input);
    let n_year = parse_ganzhi(&natal.year.ganzhi).expect("natal year_gz 应可解析");
    let n_month = parse_ganzhi(&natal.month.ganzhi).expect("natal month_gz 应可解析");
    let n_day = parse_ganzhi(&natal.day.ganzhi).expect("natal day_gz 应可解析");
    let n_hour = parse_ganzhi(&natal.hour.ganzhi).expect("natal hour_gz 应可解析");

    let primary_w = natal.yongshen.primary_wuxing.clone();
    let secondary_w = natal.yongshen.secondary_wuxing.clone();
    let avoid_w = natal.yongshen.avoid_wuxing.clone();

    (0..=max_age)
        .map(|age| {
            // 流年：用「每年回到出生时刻」近似采样 — Moment 取出生月/日/时，只换公历年。
            let target_year = natal_input.year + i32::try_from(age).unwrap_or(0);
            let m = Moment::new(
                target_year,
                natal_input.month,
                natal_input.day,
                natal_input.hour,
                natal_input.minute,
                natal_input.tz,
            );
            let flow_chart = compute_at(&m, None);
            let flow_year_gz = flow_chart.year.ganzhi.clone();

            let dayun_active = active_dayun_step(natal.dayun.as_ref(), f64::from(age));
            let dayun_step = dayun_active.as_ref().map(|(i, _)| *i);
            let dayun_ganzhi = dayun_active.as_ref().map(|(_, gz)| gz.clone());

            let mut extras: Vec<GanZhi> = Vec::with_capacity(2);
            if let Some((_, ref gz_s)) = dayun_active
                && let Some(g) = parse_ganzhi(gz_s)
            {
                extras.push(g);
            }
            if let Some(g) = parse_ganzhi(&flow_year_gz) { extras.push(g); }

            let strength = compute_strength_with_extras(n_year, n_month, n_day, n_hour, &extras);
            let primary = wuxing_pct_by_name(&strength.wuxing, &primary_w);
            let secondary = secondary_w.as_ref().map(|w| wuxing_pct_by_name(&strength.wuxing, w));
            let avoid = avoid_w.iter()
                .map(|w| wuxing_pct_by_name(&strength.wuxing, w))
                .max().unwrap_or(0);

            let judgment = judge_from_supplies(primary, secondary, avoid);
            FortuneTimelinePoint {
                age,
                year: target_year,
                flow_year_ganzhi: flow_year_gz,
                dayun_step,
                dayun_ganzhi,
                yun_score: strength.score,
                primary_supply_pct: primary,
                secondary_supply_pct: secondary,
                avoid_supply_pct: avoid,
                judgment,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- fortune_at / timeline 测试 ----------------------------------

    fn natal_1987() -> BirthInput {
        BirthInput { year: 1987, month: 9, day: 17, hour: 15, minute: 0, tz: 8.0, gender: Some(Gender::Male) }
    }

    #[test]
    fn fortune_at_aggregates_natal_and_t() {
        // 1987 长沙男（本命主用神=木官杀，忌火印+土比劫）；t=2026-06-16(age ~38.7)
        let f = fortune_at(natal_1987(), 2026, 6, 16, 10, 0, 8.0);
        // 本命旺衰恒等
        assert_eq!(f.ming_strength.score, f.natal.strength.score);
        // age 浮点合理
        assert!(f.age_years > 38.0 && f.age_years < 39.0, "age_years={}", f.age_years);
        // 流年丙午（2026年柱）与 t_chart 年柱一致
        assert_eq!(f.flow_year_ganzhi, f.t_chart.year.ganzhi);
        assert_eq!(f.flow_year_ganzhi, "丙午");
        // 大运：1987-09-17 男 → 阳年顺行，十步覆盖，age ~38 落在第 4-5 步附近（具体随起运岁数）
        assert!(f.dayun_step.is_some());
        assert!(f.dayun_ganzhi.is_some());
        // 用神供给度 ∈ [0, 100]
        assert!(f.primary_supply_pct <= 100);
        if let Some(s) = f.secondary_supply_pct { assert!(s <= 100); }
        for v in &f.avoid_supply_pcts { assert!(*v <= 100); }
        // 忌神条目数 = natal.yongshen.avoid_wuxing.len()
        assert_eq!(f.avoid_supply_pcts.len(), f.natal.yongshen.avoid_wuxing.len());
        // delta = yun.score - ming.score
        assert_eq!(
            f.delta_score,
            i32::try_from(f.yun_strength.score).unwrap() - i32::try_from(f.ming_strength.score).unwrap()
        );
    }

    #[test]
    fn fortune_at_t_chart_is_t_moment_bazi() {
        // t_chart 实际就是 t 时刻自起一盘 — 与从 t 算的本命 BaziChart 字节一致（确认共享层一次性算法路径）。
        let f = fortune_at(natal_1987(), 2026, 6, 16, 10, 0, 8.0);
        let standalone = compute(BirthInput {
            year: 2026, month: 6, day: 16, hour: 10, minute: 0, tz: 8.0, gender: Some(Gender::Male)
        });
        assert_eq!(f.t_chart.year.ganzhi, standalone.year.ganzhi);
        assert_eq!(f.t_chart.month.ganzhi, standalone.month.ganzhi);
        assert_eq!(f.t_chart.day.ganzhi, standalone.day.ganzhi);
        assert_eq!(f.t_chart.hour.ganzhi, standalone.hour.ganzhi);
    }

    #[test]
    fn fortune_timeline_covers_range_and_is_well_formed() {
        let timeline = fortune_supply_timeline(natal_1987(), 100);
        assert_eq!(timeline.len(), 101);
        assert_eq!(timeline[0].age, 0);
        assert_eq!(timeline[100].age, 100);
        // 每年公历 = 出生年 + age
        for p in &timeline {
            assert_eq!(p.year, 1987 + i32::try_from(p.age).unwrap());
            assert!(p.yun_score <= 100);
            assert!(p.primary_supply_pct <= 100);
            assert!(p.avoid_supply_pct <= 100);
            // 流年干支字面合法（2 个汉字）
            assert_eq!(p.flow_year_ganzhi.chars().count(), 2);
        }
        // 出生当年 age=0 流年应 = natal 年柱（都是 1987 → 丁卯）。
        assert_eq!(timeline[0].flow_year_ganzhi, "丁卯");
        // age=39 (2026) → 丙午；age=43 (2030) → 庚戌。
        assert_eq!(timeline[39].flow_year_ganzhi, "丙午");
        assert_eq!(timeline[43].flow_year_ganzhi, "庚戌");
    }

    #[test]
    fn fortune_timeline_dayun_step_monotone_non_decreasing() {
        // 大运按 start_age 递增，timeline 上 step index 应单调不减（或保持 None 期一致）。
        let timeline = fortune_supply_timeline(natal_1987(), 100);
        let mut prev: Option<usize> = None;
        for p in &timeline {
            if let (Some(prev_i), Some(cur_i)) = (prev, p.dayun_step) {
                assert!(cur_i >= prev_i, "dayun_step 应单调不减 prev={prev_i} cur={cur_i} at age={}", p.age);
            }
            if p.dayun_step.is_some() { prev = p.dayun_step; }
        }
    }

    #[test]
    fn judgment_threshold_boundaries() {
        // 大吉：net=+15+ (primary 30， avoid 15， net=15)
        let j = judge_from_supplies(30, None, 15);
        assert_eq!(j.level, "大吉");
        assert_eq!(j.score, 15);
        // 吉：net 5..15 (primary 20， avoid 15， net=5)
        let j = judge_from_supplies(20, None, 15);
        assert_eq!(j.level, "吉");
        assert_eq!(j.score, 5);
        // 平：net 0 (primary 15， avoid 15)
        let j = judge_from_supplies(15, None, 15);
        assert_eq!(j.level, "平");
        // 凶：net -5 (primary 10， avoid 15)
        let j = judge_from_supplies(10, None, 15);
        assert_eq!(j.level, "凶");
        assert_eq!(j.score, -5);
        // 大凶：net -15 (primary 0， avoid 15)
        let j = judge_from_supplies(0, None, 15);
        assert_eq!(j.level, "大凶");
        assert_eq!(j.score, -15);
        // 副用神计入：primary 10 + secondary 20 （折 0.5=10） - avoid 5 = +15 → 大吉
        let j = judge_from_supplies(10, Some(20), 5);
        assert_eq!(j.level, "大吉");
        assert_eq!(j.score, 15);
        // summary 非空且含百分比
        assert!(!j.summary.is_empty());
        assert!(j.summary.contains('%'));
    }

    #[test]
    fn fortune_at_carries_judgment_for_1987() {
        // 1987 长沙男 + 2026-06-16：主用神木 13%/副水 9%/忌火 33%
        // net = 13 + 9/2 - 33 = 13 + 4 - 33 = -16 → 大凶
        let f = fortune_at(natal_1987(), 2026, 6, 16, 10, 0, 8.0);
        assert_eq!(f.judgment.level, "大凶");
        assert_eq!(f.judgment.score, -16);
        assert!(f.judgment.summary.contains("33%"));
        assert!(f.judgment.summary.contains("宜守不宜攻"));
    }

    #[test]
    fn fortune_timeline_carries_judgment_each_point() {
        let timeline = fortune_supply_timeline(natal_1987(), 100);
        let levels: std::collections::HashSet<&str> = timeline.iter().map(|p| p.judgment.level.as_str()).collect();
        // 100 年里至少出现 2 个等级（命局不可能恒一）
        assert!(levels.len() >= 2, "timeline should cover multiple judgment levels: {levels:?}");
        // 每点判读 score 满足 5 等级阈值
        for p in &timeline {
            let net = p.judgment.score;
            match p.judgment.level.as_str() {
                "大吉" => assert!(net >= 15),
                "吉" => assert!((5..15).contains(&net)),
                "平" => assert!((-4..5).contains(&net)),
                "凶" => assert!((-14..-4).contains(&net)),
                "大凶" => assert!(net <= -15),
                other => panic!("unexpected level {other}"),
            }
        }
    }

    #[test]
    fn wuxing_pct_by_name_dispatch_all_five_plus_unknown() {
        let w = WuxingPower { wood: 10, fire: 20, earth: 30, metal: 40, water: 50 };
        assert_eq!(wuxing_pct_by_name(&w, "木"), 10);
        assert_eq!(wuxing_pct_by_name(&w, "火"), 20);
        assert_eq!(wuxing_pct_by_name(&w, "土"), 30);
        assert_eq!(wuxing_pct_by_name(&w, "金"), 40);
        assert_eq!(wuxing_pct_by_name(&w, "水"), 50);
        assert_eq!(wuxing_pct_by_name(&w, "未知"), 0);
    }

    /// 年柱换岁流派：2024-02-01（立春前一日）出生。
    /// 立春派：归 2023 癸卯；春节派：归 2023（春节 2024-02-10，出生在它之前 → 也归 2023）。
    /// 2024-02-09（春节前一日）：立春派 2024 甲辰（已过立春 2024-02-04）；春节派 2023（未到正月初一）。
    #[test]
    fn year_break_school_lichun_vs_springfestival() {
        let m = Moment::new(2024, 2, 9, 12, 0, 8.0);
        let chun = compute_at_school(&m, None, BaziSchool { zi_hour: ZiHourMethod::Late, year_break: YearBreakMethod::LiChun });
        let sf = compute_at_school(&m, None, BaziSchool { zi_hour: ZiHourMethod::Late, year_break: YearBreakMethod::SpringFestival });
        assert_eq!(chun.year.ganzhi, "甲辰", "立春派 2024-02-09 已过立春 → 甲辰");
        assert_eq!(sf.year.ganzhi, "癸卯", "春节派 2024-02-09 春节(02-10)未到 → 癸卯");
    }

    /// 子时流派校验：23：30 出生，晚子（主流）归次日日柱；早子（传统少数）归当日。
    /// 1990-06-15 日柱=辛亥；1990-06-16 日柱应为壬子（辛亥之次）。
    #[test]
    fn zi_hour_school_late_vs_early() {
        let m_2330 = Moment::new(1990, 6, 15, 23, 30, 8.0);
        let late = compute_at_with(&m_2330, None, ZiHourMethod::Late);
        let early = compute_at_with(&m_2330, None, ZiHourMethod::Early);
        // 早子：仍归当日（辛亥）
        assert_eq!(early.day.ganzhi, "辛亥", "Early Zi 日柱应为 1990-06-15 当日");
        // 晚子：归次日（壬子）
        assert_eq!(late.day.ganzhi, "壬子", "Late Zi 日柱应为次日 1990-06-16");
        // 非 23 点出生时，两派应一致
        let m_1430 = Moment::new(1990, 6, 15, 14, 30, 8.0);
        let a = compute_at_with(&m_1430, None, ZiHourMethod::Late);
        let b = compute_at_with(&m_1430, None, ZiHourMethod::Early);
        assert_eq!(a.day.ganzhi, b.day.ganzhi);
    }

    #[test]
    fn sample_1990_06_15_male() {
        let chart = compute(BirthInput {
            year: 1990,
            month: 6,
            day: 15,
            hour: 14,
            minute: 30,
            tz: 8.0,
            gender: Some(Gender::Male),
        });
        assert_eq!(chart.year.ganzhi, "庚午");
        assert_eq!(chart.month.ganzhi, "壬午");
        assert_eq!(chart.day.ganzhi, "辛亥");
        assert_eq!(chart.hour.ganzhi, "乙未");
        assert_eq!(chart.day_master, "辛");
        assert_eq!(chart.day_master_wuxing, "金");
        assert_eq!(chart.month.ten_god, "伤官"); // 壬 vs 日主辛
        assert_eq!(chart.day.ten_god, "日主");
        assert_eq!(
            (chart.lunar.year, chart.lunar.month, chart.lunar.day),
            (1990, 5, 23)
        );
        let dy = chart.dayun.as_ref().unwrap();
        assert!(dy.forward); // 庚午阳年男 → 顺行
        assert_eq!(dy.pillars.len(), 10);
        assert_eq!(dy.pillars[0].ganzhi, "癸未"); // 月柱壬午顺行下一步
    }

    #[test]
    fn dayun_reverse_for_yin_year_male() {
        // 1989 己巳（阴年）男 → 逆行
        let chart = compute(BirthInput {
            year: 1989,
            month: 6,
            day: 15,
            hour: 12,
            minute: 0,
            tz: 8.0,
            gender: Some(Gender::Male),
        });
        assert!(!chart.dayun.as_ref().unwrap().forward);
    }

    #[test]
    fn before_lichun_uses_prev_year() {
        // 1990-01-20 在立春(1990-02-04)前 → 年柱归 1989 己巳。
        let chart = compute(BirthInput {
            year: 1990,
            month: 1,
            day: 20,
            hour: 12,
            minute: 0,
            tz: 8.0,
            gender: None,
        });
        assert_eq!(chart.year.ganzhi, "己巳");
    }

    #[test]
    fn no_dayun_without_gender() {
        let chart = compute(BirthInput {
            year: 2000,
            month: 1,
            day: 1,
            hour: 0,
            minute: 0,
            tz: 8.0,
            gender: None,
        });
        assert!(chart.dayun.is_none());
    }

    #[test]
    fn xunkong_and_nayin_oracle() {
        // 1990-06-15 14：30 男 → 日柱辛亥。辛亥在甲辰旬 → 旬空寅卯；辛亥纳音=钗钏金（金）。
        let chart = compute(BirthInput {
            year: 1990, month: 6, day: 15, hour: 14, minute: 30, tz: 8.0,
            gender: Some(Gender::Male),
        });
        assert_eq!(chart.day.ganzhi, "辛亥");
        assert_eq!(chart.xunkong, ["寅".to_string(), "卯".to_string()]);
        assert_eq!(chart.day.nayin, "金"); // 钗钏金

        // 1987-09-17 15：00 男 → 四柱 丁卯 己酉 己巳 壬申。日柱己巳在甲子旬 → 旬空戌亥。
        let c2 = compute(BirthInput {
            year: 1987, month: 9, day: 17, hour: 15, minute: 0, tz: 8.0,
            gender: Some(Gender::Male),
        });
        assert_eq!(c2.year.ganzhi, "丁卯");
        assert_eq!(c2.day.ganzhi, "己巳");
        assert_eq!(c2.hour.ganzhi, "壬申");
        assert_eq!(c2.xunkong, ["戌".to_string(), "亥".to_string()]);
    }

    #[test]
    fn strength_oracle_1987_male_yin_earth() {
        // 1987-09-17 15：00 男 = 丁卯 己酉 己巳 壬申。日主己土。
        // 手算：
        //   得令（月酉）：己阴土在酉=长生 stage0 → 20；酉藏[辛]辛金=食伤非同党；got_ling=20。
        //   得地（卯/巳/申）：
        //     卯[乙] 七杀，非 → 0；
        //     巳[丙 庚 戊] 丙印（本+9） 庚伤(0) 戊劫（余+3） = 12；
        //     申[庚 壬 戊] 庚伤 壬财 戊劫（余+3） = 3；
        //     got_di = 0+12+3 = 15。
        //   得势（年丁/月己/时壬）：丁印(+10) 己比肩(+10) 壬财(0)=20。
        //   raw=20+15+20=55 → score=55*100/90=61 → 偏强。
        let c = compute(BirthInput {
            year: 1987, month: 9, day: 17, hour: 15, minute: 0, tz: 8.0,
            gender: Some(Gender::Male),
        });
        assert_eq!(c.day_master, "己");
        assert_eq!(c.strength.got_ling, 20, "酉=长生 20、月支辛伤非同党");
        assert_eq!(c.strength.got_di, 15, "巳丙印9+戊劫3 申戊劫3");
        assert_eq!(c.strength.got_shi, 20, "丁印10+己比10");
        assert_eq!(c.strength.score, 61);
        assert_eq!(c.strength.level, "偏强");
        // 五行分布合 100（整数 round 凑巧；允差 1）。
        let s = c.strength.wuxing;
        let sum = s.wood + s.fire + s.earth + s.metal + s.water;
        assert!((99..=101).contains(&sum), "wuxing 合 ≈ 100，实 {sum}");
        // 金最旺（月令酉×1.5 + 巳/申庚金）：应是最大项
        let max = [s.wood, s.fire, s.earth, s.metal, s.water].into_iter().max().unwrap();
        assert_eq!(s.metal, max, "酉月金最旺");
    }

    #[test]
    fn strength_oracle_1990_male_yin_metal_in_summer() {
        // 1990-06-15 14：30 男 = 庚午 壬午 辛亥 乙未。日主辛金，生于午月夏火克金。
        // 手算：
        //   得令（月午）：辛阴金在午=stage6 病 → 6；午藏[丁 己] 丁七杀(0) 己偏印（中+3） → 6+3=9。
        //   得地（午/亥/未）：
        //     午[丁 己] 丁七杀 己偏印（中+5） = 5
        //     亥[壬 甲] 壬伤 甲正财 = 0
        //     未[己 乙 丁] 己偏印（本+9） 乙偏财 丁七杀 = 9
        //     got_di = 5+0+9 = 14。
        //   得势（年庚/月壬/时乙）：庚劫财(+10) 壬伤 乙偏财 → 10。
        //   raw=9+14+10=33 → score=33*100/90=36 → 偏弱（辛金生夏天合理）。
        let c = compute(BirthInput {
            year: 1990, month: 6, day: 15, hour: 14, minute: 30, tz: 8.0,
            gender: Some(Gender::Male),
        });
        assert_eq!(c.day_master, "辛");
        assert_eq!(c.strength.got_ling, 9);
        assert_eq!(c.strength.got_di, 14);
        assert_eq!(c.strength.got_shi, 10);
        assert_eq!(c.strength.score, 36);
        assert_eq!(c.strength.level, "偏弱");
    }

    /// Female 大运：阴年女顺行 / 阳年女逆行。1990 庚午阳年女 → 逆行。
    #[test]
    fn dayun_female_gender() {
        let chart = compute(BirthInput {
            year: 1990, month: 6, day: 15, hour: 14, minute: 30, tz: 8.0,
            gender: Some(Gender::Female),
        });
        assert!(!chart.dayun.as_ref().unwrap().forward, "庚午阳年女 → 逆行");
    }

    /// 春节换年：春节后归本年（覆盖 L397 = month==1 day>=1 非闰分支）。
    /// 2024-02-15（春节 02-10 已过）非闰正月初六 → 春节派应归 2024 甲辰。
    #[test]
    fn year_break_springfestival_after_lunar_new_year() {
        let m = Moment::new(2024, 2, 15, 12, 0, 8.0);
        let sf = compute_at_school(&m, None, BaziSchool {
            zi_hour: ZiHourMethod::Late, year_break: YearBreakMethod::SpringFestival,
        });
        assert_eq!(sf.year.ganzhi, "甲辰", "春节(02-10)已过 → 春节派归 2024 甲辰");
    }

    /// 春节换年 fallback：既非正月初一在前、又非月>=11/=12/闰正月 → m.year。
    /// 实际上 2024-03-15（农历二月初六）中已到春节后，但 month=2，不命中前两条 → fallback L402。
    #[test]
    fn year_break_springfestival_fallback_branch() {
        let m = Moment::new(2024, 3, 15, 12, 0, 8.0);
        let sf = compute_at_school(&m, None, BaziSchool {
            zi_hour: ZiHourMethod::Late, year_break: YearBreakMethod::SpringFestival,
        });
        // 立春派与春节派此刻应一致（都在 2024 范围内），走 fallback m.year
        assert_eq!(sf.year.ganzhi, "甲辰");
    }

    #[test]
    fn strength_extras_empty_equals_natal() {
        // 空 extras 必须等价于本命旺衰，作为本命 chart 的回归校验。
        use mingli_ganzhi::parse_ganzhi;
        let c = compute(BirthInput {
            year: 1987, month: 9, day: 17, hour: 15, minute: 0, tz: 8.0,
            gender: Some(Gender::Male),
        });
        let y = parse_ganzhi(&c.year.ganzhi).unwrap();
        let mo = parse_ganzhi(&c.month.ganzhi).unwrap();
        let d = parse_ganzhi(&c.day.ganzhi).unwrap();
        let h = parse_ganzhi(&c.hour.ganzhi).unwrap();
        let no_extra = compute_strength_with_extras(y, mo, d, h, &[]);
        assert_eq!(no_extra.score, c.strength.score);
        assert_eq!(no_extra.got_ling, c.strength.got_ling);
        assert_eq!(no_extra.got_di, c.strength.got_di);
        assert_eq!(no_extra.got_shi, c.strength.got_shi);
    }

    #[test]
    fn strength_extras_help_pushes_score_up() {
        // 1987 己土 本命 = 偏强 61（得令20+得地15+得势20）。叠加「戊午」（戊=劫财+10、午藏丁己印+劫）：
        //   得地原15 + （午丁本印+9 + 己中劫+5） = 29（未封顶）；得势原20 + 戊劫+10 = 30（封顶）；得令20。
        //   raw = 79 → score = 79*100/90 = 87 → 强。
        use mingli_ganzhi::parse_ganzhi;
        let c = compute(BirthInput {
            year: 1987, month: 9, day: 17, hour: 15, minute: 0, tz: 8.0,
            gender: Some(Gender::Male),
        });
        let y = parse_ganzhi(&c.year.ganzhi).unwrap();
        let mo = parse_ganzhi(&c.month.ganzhi).unwrap();
        let d = parse_ganzhi(&c.day.ganzhi).unwrap();
        let h = parse_ganzhi(&c.hour.ganzhi).unwrap();
        let yun = compute_strength_with_extras(y, mo, d, h, &[parse_ganzhi("戊午").unwrap()]);
        assert_eq!(yun.got_di, 29);
        assert_eq!(yun.got_shi, 30, "得势封顶");
        assert_eq!(yun.score, 87);
        assert_eq!(yun.level, "强");
        assert!(yun.score > c.strength.score, "助党推升旺衰");
    }

    #[test]
    fn strength_extras_hostile_keeps_score_steady() {
        // 加纯敌党（如「壬寅」：壬财、寅藏甲丙戊→甲杀/丙印/戊劫）：
        //   实际寅有印+劫，会拉升 di，所以严格意义不是「纯敌」。改测「乙未」乙偏财 + 未己乙丁（己劫本+9、乙财、丁印余+3=12）。
        //   得地原15+12=27；得势原20+乙财0=20；得令20；raw=67→score=74 偏强（仍提升，因未支带印劫）。
        // 真正纯敌党：取「庚申」（庚=伤、申=庚伤本+壬财中+戊劫余3=3）：
        //   得地原15+3=18；得势原20+庚伤0=20；得令20；raw=58→score=64 偏强（微升）。
        //   关键性质：不论加什么 extras，得令永远不变 = 月令固定。
        use mingli_ganzhi::parse_ganzhi;
        let c = compute(BirthInput {
            year: 1987, month: 9, day: 17, hour: 15, minute: 0, tz: 8.0,
            gender: Some(Gender::Male),
        });
        let y = parse_ganzhi(&c.year.ganzhi).unwrap();
        let mo = parse_ganzhi(&c.month.ganzhi).unwrap();
        let d = parse_ganzhi(&c.day.ganzhi).unwrap();
        let h = parse_ganzhi(&c.hour.ganzhi).unwrap();
        let yun = compute_strength_with_extras(y, mo, d, h, &[parse_ganzhi("庚申").unwrap()]);
        assert_eq!(yun.got_ling, c.strength.got_ling, "得令固定取本命月支，extras 不改");
        assert_eq!(yun.got_di, 18);
        assert_eq!(yun.got_shi, 20);
        assert_eq!(yun.score, 64);
    }

    #[test]
    fn strength_score_bounds() {
        // 任意输入下，三栏都在 [0,30]、综合分都在 [0,100]、五行和约 100。
        for &(y, m, d, h) in &[
            (2024, 6, 21, 12), (1980, 1, 1, 0), (2000, 11, 11, 11),
            (1949, 10, 1, 15), (2030, 2, 4, 16),
        ] {
            let c = compute(BirthInput {
                year: y, month: m, day: d, hour: h, minute: 0, tz: 8.0, gender: None,
            });
            let s = &c.strength;
            assert!(s.got_ling <= 30);
            assert!(s.got_di <= 30);
            assert!(s.got_shi <= 30);
            assert!(s.score <= 100);
            let sum = s.wuxing.wood + s.wuxing.fire + s.wuxing.earth + s.wuxing.metal + s.wuxing.water;
            assert!((99..=101).contains(&sum));
        }
    }

    #[test]
    fn pattern_1987_yin_earth_anzang_shi_shen() {
        // 1987-09-17 15：00 男 = 丁卯 己酉 己巳 壬申。日主己土，月支酉（专气藏辛）。
        // 三干头（丁/己/壬）无辛 → 辛不透 → 暗藏取本气 → 辛（食神） → 暗食神格。
        let c = compute(BirthInput {
            year: 1987, month: 9, day: 17, hour: 15, minute: 0, tz: 8.0,
            gender: Some(Gender::Male),
        });
        let p = &c.pattern;
        assert_eq!(p.name, "暗食神格");
        assert_eq!(p.qi_stem, "辛");
        assert_eq!(p.qi_kind, "本气");
        assert!(!p.revealed);
        assert_eq!(p.revealed_in, None);
        assert_eq!(p.ten_god, "食神");
        assert!(!p.is_lu_ren);
    }

    #[test]
    fn pattern_1990_yin_metal_anzang_qisha() {
        // 1990-06-15 14：30 男 = 庚午 壬午 辛亥 乙未。日主辛，月支午（本丁/中己）。
        // 三干头（庚/壬/乙）无丁、无己 → 暗藏取本气丁（七杀） → 暗七杀格。
        let c = compute(BirthInput {
            year: 1990, month: 6, day: 15, hour: 14, minute: 30, tz: 8.0,
            gender: Some(Gender::Male),
        });
        assert_eq!(c.pattern.name, "暗七杀格");
        assert_eq!(c.pattern.qi_stem, "丁");
        assert_eq!(c.pattern.ten_god, "七杀");
        assert!(!c.pattern.revealed);
    }

    /// 直接构造 GanZhi 测纯算法（不经历法约束）——覆盖建禄/月刃/八正格透干的各分支。
    #[test]
    fn pattern_jianlu_when_main_qi_equals_day_master() {
        // 日主甲（0 阳木） + 月支寅（本气甲） → 同五行同阴阳 → 建禄格。
        let yr = GanZhi { stem: 9, branch: 0 };   // 任意
        let mo = GanZhi { stem: 2, branch: 2 };   // 月支寅
        let dy = GanZhi { stem: 0, branch: 4 };   // 日主甲
        let hr = GanZhi { stem: 5, branch: 11 };
        let p = determine_pattern(yr, mo, dy, hr);
        assert_eq!(p.name, "建禄格");
        assert!(p.is_lu_ren);
        assert_eq!(p.ten_god, "比肩");
    }

    #[test]
    fn pattern_yueren_when_main_qi_same_element_different_polarity() {
        // 日主甲（0 阳木） + 月支卯（本气乙阴木） → 同五行异阴阳 → 月刃格。
        let yr = GanZhi { stem: 9, branch: 0 };
        let mo = GanZhi { stem: 2, branch: 3 };
        let dy = GanZhi { stem: 0, branch: 4 };
        let hr = GanZhi { stem: 5, branch: 11 };
        let p = determine_pattern(yr, mo, dy, hr);
        assert_eq!(p.name, "月刃格");
        assert!(p.is_lu_ren);
        assert_eq!(p.ten_god, "劫财");
    }

    #[test]
    fn pattern_main_qi_revealed() {
        // 月支寅（本甲/中丙/余戊），日主己土。让本气甲在年柱透出 → 正官格（甲对己=正官）。
        let yr = GanZhi { stem: 0, branch: 0 }; // 年干 甲
        let mo = GanZhi { stem: 9, branch: 2 }; // 月干 癸（非月令藏干），月支寅
        let dy = GanZhi { stem: 5, branch: 11 }; // 日主 己
        let hr = GanZhi { stem: 3, branch: 7 }; // 时干 丁
        let p = determine_pattern(yr, mo, dy, hr);
        assert_eq!(p.name, "正官格");
        assert!(p.revealed);
        assert_eq!(p.qi_stem, "甲");
        assert_eq!(p.qi_kind, "本气");
        assert_eq!(p.revealed_in.as_deref(), Some("年柱"));
        assert_eq!(p.ten_god, "正官");
    }

    #[test]
    fn pattern_middle_qi_revealed_skips_main_qi() {
        // 月支寅（本甲/中丙/余戊），日主己。本气甲不透，中气丙在时柱透 → 正印格。
        let yr = GanZhi { stem: 3, branch: 5 }; // 丁巳
        let mo = GanZhi { stem: 9, branch: 2 }; // 癸寅（非历法合理，纯算法测试）
        let dy = GanZhi { stem: 5, branch: 11 }; // 己亥
        let hr = GanZhi { stem: 2, branch: 3 }; // 丙卯
        let p = determine_pattern(yr, mo, dy, hr);
        assert_eq!(p.name, "正印格");
        assert!(p.revealed);
        assert_eq!(p.qi_stem, "丙");
        assert_eq!(p.qi_kind, "中气");
        assert_eq!(p.revealed_in.as_deref(), Some("时柱"));
        assert_eq!(p.ten_god, "正印");
    }

    #[test]
    fn pattern_yu_qi_revealed_when_main_and_middle_unrevealed() {
        // 月支寅（本甲/中丙/余戊），日主庚金。让余气戊在月柱透 → 偏印格（戊对庚=偏印，土生金同阳）。
        let yr = GanZhi { stem: 1, branch: 5 }; // 乙巳
        let mo = GanZhi { stem: 4, branch: 2 }; // 戊寅
        let dy = GanZhi { stem: 6, branch: 11 }; // 庚亥
        let hr = GanZhi { stem: 9, branch: 3 }; // 癸卯
        let p = determine_pattern(yr, mo, dy, hr);
        assert_eq!(p.name, "偏印格");
        assert_eq!(p.qi_stem, "戊");
        assert_eq!(p.qi_kind, "余气");
        assert_eq!(p.revealed_in.as_deref(), Some("月柱"));
        assert_eq!(p.ten_god, "偏印");
    }

    #[test]
    fn yongshen_1987_male_yin_earth_strong() {
        // 1987-09-17 15：00 男 = 丁卯 己酉 己巳 壬申。日主己土 score 61 偏强。
        // 五行分布：木11/火20/土23/金32/水14。
        // 走身强宜耗：候选 官杀木(11)/财水(14)/食伤金(32) → 升序 木<水<金 → 主用神=木（官杀），副=水（财）。
        // 忌神 = 印火 + 比劫土。
        let c = compute(BirthInput {
            year: 1987, month: 9, day: 17, hour: 15, minute: 0, tz: 8.0,
            gender: Some(Gender::Male),
        });
        let y = &c.yongshen;
        assert_eq!(y.method, "扶抑 · 身强宜耗");
        assert_eq!(y.primary_wuxing, "木");
        assert_eq!(y.primary_role, "官杀");
        assert_eq!(y.secondary_wuxing.as_deref(), Some("水"));
        assert_eq!(y.secondary_role.as_deref(), Some("财"));
        assert_eq!(y.avoid_wuxing, vec!["火".to_string(), "土".to_string()]);
        assert!(y.reasoning.contains("耗身"));
    }

    #[test]
    fn yongshen_1990_male_yin_metal_weak() {
        // 1990-06-15 14：30 男 = 庚午 壬午 辛亥 乙未。日主辛金 score 36 偏弱。
        // 走身弱宜扶：印星（土）优先，比劫（金）副。忌神 = 官杀火 + 财木。
        let c = compute(BirthInput {
            year: 1990, month: 6, day: 15, hour: 14, minute: 30, tz: 8.0,
            gender: Some(Gender::Male),
        });
        let y = &c.yongshen;
        assert_eq!(y.method, "扶抑 · 身弱宜扶");
        assert_eq!(y.primary_wuxing, "土");
        assert_eq!(y.primary_role, "印星");
        assert_eq!(y.secondary_wuxing.as_deref(), Some("金"));
        assert_eq!(y.secondary_role.as_deref(), Some("比劫"));
        assert_eq!(y.avoid_wuxing, vec!["火".to_string(), "木".to_string()]);
        assert!(y.reasoning.contains("助身"));
    }

    /// 中和走调候：用纯算法构造一个 score≈50 的盘，看调候按月支取。
    #[test]
    fn yongshen_neutral_takes_tiao_hou() {
        use mingli_ganzhi::parse_ganzhi;
        // 手造 score=50 strength + 月支子（冬月） → 调候取火。
        let fake_str = Strength {
            score: 50, level: "中和".into(),
            got_ling: 15, got_di: 15, got_shi: 15,
            wuxing: WuxingPower { wood: 20, fire: 20, earth: 20, metal: 20, water: 20 },
        };
        // 日主己土 + 月支子 → 调候 寒月取火
        let y = determine_yongshen(5, 0, &fake_str);
        assert_eq!(y.method, "调候为主");
        assert_eq!(y.primary_wuxing, "火");
        assert_eq!(y.primary_role, "调候");
        assert!(y.secondary_wuxing.is_none());
        assert!(y.avoid_wuxing.is_empty());
        // 日主己土 + 月支午（燥月） → 取水
        let y2 = determine_yongshen(5, 6, &fake_str);
        assert_eq!(y2.primary_wuxing, "水");
        // 日主庚 + 月支寅（春木） → 取金
        let y3 = determine_yongshen(6, 2, &fake_str);
        assert_eq!(y3.primary_wuxing, "金");
        // 日主壬 + 月支申（秋金） → 取火
        let y4 = determine_yongshen(8, 8, &fake_str);
        assert_eq!(y4.primary_wuxing, "火");
        // 日主甲 + 月支辰（杂气） → 取日主同行（木）
        let y5 = determine_yongshen(0, 4, &fake_str);
        assert_eq!(y5.primary_wuxing, "木");
        // 校验：5 个月支分支都已覆盖，确认 parse_ganzhi 与本测试无关
        assert!(parse_ganzhi("甲子").is_some());
    }

    /// 反查五行关系正确性：印星生我、官杀克我。
    #[test]
    fn yongshen_role_inverses_correct() {
        use mingli_ganzhi::Element;
        // 印星 X.generates() == dm
        for dm in [Element::Wood, Element::Fire, Element::Earth, Element::Metal, Element::Water] {
            assert_eq!(yin_xing_of(dm).generates(), dm, "印星生我：{dm:?}");
            assert_eq!(guan_sha_of(dm).controls(), dm, "官杀克我：{dm:?}");
        }
    }

    #[test]
    fn true_solar_offset_changshanha_oracle() {
        // 长沙 1987-09-17 lon=112.94°E，tz=+8（标准经线 120°）。
        //   经度差 = (112.94 − 120) × 4 = −28.24 min
        //   EoT（9月17日） ≈ +6 min（Spencer 公式）
        //   合 ≈ −22 min（真太阳时较钟表早约 22 分钟）
        let off = true_solar_offset_minutes(112.94, 8.0, 1987, 9, 17);
        assert!(
            (-24.0..=-20.0).contains(&off),
            "长沙 1987-09-17 真太阳时差应在 [-24, -20] 分钟，实测 {off:.2}"
        );
    }

    #[test]
    fn true_solar_does_not_change_pillar_when_within_same_chen() {
        // 长沙 1987-09-17 15：00 钟表（未时）；真太阳时 ≈ 14：38，仍在未时(13-15)：
        //   等等，未时 13-15，15：00 已是申时(15-17)；真太阳 14：38 是未时。
        //   ⇒ 时柱按钟表 是申时，按真太阳是未时；时柱会变！
        // 重新设计：取真正不变的 case：15：30 钟表 → 真太阳 ≈ 15：08，两者都申时 → 时柱同。
        let with_solar = compute_with_true_solar(BirthInput {
            year: 1987, month: 9, day: 17, hour: 15, minute: 30, tz: 8.0,
            gender: Some(Gender::Male),
        }, 112.94);
        let no_solar = compute(BirthInput {
            year: 1987, month: 9, day: 17, hour: 15, minute: 30, tz: 8.0,
            gender: Some(Gender::Male),
        });
        // 钟表 15：30 申时 hour_branch=8；真太阳 ≈ 15：08 申时 hour_branch=8 → 时柱同。
        assert_eq!(no_solar.hour.ganzhi, with_solar.hour.ganzhi);
    }

    #[test]
    fn true_solar_changes_pillar_across_chen_boundary() {
        // 长沙 1987-09-17 钟表 15：00（申时起点），真太阳 ≈ 14：38（未时）：
        //   钟表 hour_branch=hour_branch(15，30)=8 申、真太阳 hour_branch(14，..)=7 未 → 时柱必变。
        let no_solar = compute(BirthInput {
            year: 1987, month: 9, day: 17, hour: 15, minute: 0, tz: 8.0,
            gender: Some(Gender::Male),
        });
        let with_solar = compute_with_true_solar(BirthInput {
            year: 1987, month: 9, day: 17, hour: 15, minute: 0, tz: 8.0,
            gender: Some(Gender::Male),
        }, 112.94);
        // 钟表时柱 = 壬申（已验）；真太阳时柱 = ？
        // hour_branch=7 未， day_stem=5（己）， stem=(5%5)*2+7=7 → 辛。时柱=辛未。
        assert_eq!(no_solar.hour.ganzhi, "壬申");
        assert_eq!(with_solar.hour.ganzhi, "辛未");
        // 其它三柱不变（同一日同一月同一年）
        assert_eq!(no_solar.year.ganzhi, with_solar.year.ganzhi);
        assert_eq!(no_solar.month.ganzhi, with_solar.month.ganzhi);
        assert_eq!(no_solar.day.ganzhi, with_solar.day.ganzhi);
    }

    #[test]
    fn true_solar_helpers_round_trip() {
        // 校验 day_of_year 闰年分支 + add_days_civil 跨月跨年。
        assert_eq!(day_of_year(2024, 3, 1), 61); // 闰年 1月31+2月29+1=61
        assert_eq!(day_of_year(2023, 3, 1), 60);
        assert_eq!(add_days_civil(2024, 1, 1, -1), (2023, 12, 31));
        assert_eq!(add_days_civil(2023, 12, 31, 1), (2024, 1, 1));
        assert_eq!(add_days_civil(2024, 2, 28, 2), (2024, 3, 1)); // 闰年 +2 跨 29
        assert_eq!(add_days_civil(2023, 2, 28, 2), (2023, 3, 2)); // 平年 28→3/2
    }

    #[test]
    fn three_houses_1987_oracle() {
        // 1987-09-17 15：00 男 = 丁卯 己酉 己巳 壬申。
        // 月支酉(9)，时支未（7，15：00 hour_branch=(15+1)/2=8 申？）。
        //   等等 hour_branch(15，0)=(16/2)=8 申。所以时支=申(8)。
        //   命宫支 = (9 - 8 + 12) % 12 = 1 → 丑；命宫干 = 五虎遁丁年丑月 = 癸（丁年寅起壬，寅卯辰巳午未申酉戌亥子丑 = 壬癸甲乙丙丁戊己庚辛壬癸，丑=癸） → 命宫=癸丑。
        //   身宫支 = (9 + 8) % 12 = 5 → 巳；身宫干 = 丁年寅起壬，巳=丙（壬癸甲乙丙） → 身宫=丙巳。
        //   胎元 = 己酉 +1+3 = 庚子。
        let c = compute(BirthInput {
            year: 1987, month: 9, day: 17, hour: 15, minute: 0, tz: 8.0,
            gender: Some(Gender::Male),
        });
        let t = &c.three_houses;
        // 丁年寅月起壬：寅壬卯癸辰甲巳乙午丙未丁申戊酉己戌庚亥辛子壬丑癸 — 命宫干=癸，身宫干=乙。
        assert_eq!(t.ming_gong, "癸丑");
        assert_eq!(t.shen_gong, "乙巳");
        assert_eq!(t.tai_yuan, "庚子");
    }

    #[test]
    fn three_houses_1990_oracle() {
        // 1990-06-15 14：30 男 = 庚午 壬午 辛亥 乙未。
        // 月支午(6)，时支未（7，14：30 hour_branch=(15/2)=7 未）。
        //   命宫支 = (6 - 7 + 12) % 12 = 11 → 亥；命宫干 = 庚年寅月起戊，寅卯辰巳午未申酉戌亥子丑 = 戊己庚辛壬癸甲乙丙丁戊己，亥=丁 → 命宫=丁亥。
        //   身宫支 = (6 + 7) % 12 = 1 → 丑；身宫干 = 庚年戊起，丑=己 → 身宫=己丑。
        //   胎元 = 壬午 +1+3 = 癸酉。
        let c = compute(BirthInput {
            year: 1990, month: 6, day: 15, hour: 14, minute: 30, tz: 8.0,
            gender: Some(Gender::Male),
        });
        assert_eq!(c.three_houses.ming_gong, "丁亥");
        assert_eq!(c.three_houses.shen_gong, "己丑");
        assert_eq!(c.three_houses.tai_yuan, "癸酉");
    }

    /// 命宫公式性质：寅月寅时 → 命宫支 = (2-2+12)%12 = 0 = 子；身宫支 = 4 = 辰。
    #[test]
    fn three_houses_ming_gong_property() {
        // 甲年寅月寅时（任意 day）：月支=寅(2)，时支=寅(2)。
        // 甲年寅月起丙：寅丙卯丁辰戊巳己午庚未辛申壬酉癸戌甲亥乙子丙丑丁。
        let yr = GanZhi { stem: 0, branch: 0 }; // 甲年
        let mo = GanZhi { stem: 2, branch: 2 }; // 丙寅
        let th = determine_three_houses(yr, mo, 2);
        // 命宫支 = (2-2+12)%12 = 0 → 子；命宫干 = 甲年子月 = 丙 → 命宫=丙子。
        assert_eq!(th.ming_gong, "丙子");
        // 身宫支 = (2+2)%12 = 4 → 辰；身宫干 = 甲年辰月 = 戊 → 身宫=戊辰。
        assert_eq!(th.shen_gong, "戊辰");
        // 胎元 = 月柱干+1=丁、支+3=巳 → 丁巳。
        assert_eq!(th.tai_yuan, "丁巳");
    }

    #[test]
    fn team_wuxing_average_oracle() {
        // 两人合盘：1987 长沙（木11火20土23金32水14）+ 1990 长沙（走 compute 后取实际值）。
        // 平均 = 两人逐项求和÷2。
        let a = compute(BirthInput {
            year: 1987, month: 9, day: 17, hour: 15, minute: 0, tz: 8.0,
            gender: Some(Gender::Male),
        });
        let b = compute(BirthInput {
            year: 1990, month: 6, day: 15, hour: 14, minute: 30, tz: 8.0,
            gender: Some(Gender::Male),
        });
        let team = team_wuxing_average(&[a.clone(), b.clone()]);
        assert_eq!(team.wood, u32::midpoint(a.strength.wuxing.wood, b.strength.wuxing.wood));
        assert_eq!(team.fire, u32::midpoint(a.strength.wuxing.fire, b.strength.wuxing.fire));
        assert_eq!(team.earth, u32::midpoint(a.strength.wuxing.earth, b.strength.wuxing.earth));
        assert_eq!(team.metal, u32::midpoint(a.strength.wuxing.metal, b.strength.wuxing.metal));
        assert_eq!(team.water, u32::midpoint(a.strength.wuxing.water, b.strength.wuxing.water));
    }

    #[test]
    fn team_wuxing_empty_is_zero() {
        let z = team_wuxing_average(&[]);
        assert_eq!(z.wood + z.fire + z.earth + z.metal + z.water, 0);
    }

    #[test]
    fn complement_and_team_extremes() {
        // 1987 wuxing： 木11 火20 土23 金32 水14。
        let a = compute(BirthInput {
            year: 1987, month: 9, day: 17, hour: 15, minute: 0, tz: 8.0,
            gender: Some(Gender::Male),
        });
        let wx = &a.strength.wuxing;
        // 对应主用神「木」（1987 偏强主用木），互补度 = 11（自给度低，需别人补）
        assert_eq!(complement_score(&a.yongshen.primary_wuxing, wx), wx.wood);
        // 极端最弱 = 木11、最旺 = 金32
        assert_eq!(team_weakest(wx), ("木".into(), wx.wood));
        assert_eq!(team_strongest(wx), ("金".into(), wx.metal));
        // 未知五行字符串 → 0
        assert_eq!(complement_score("xxx", wx), 0);
    }

    /// 1987-09-17 男 = 丁卯 己酉 己巳 壬申。日干己，年支卯。
    /// 各柱命中神煞（人工核校）：
    /// - 年柱卯：日干己 → 卯无日干锚命中（学堂/文昌均酉）；
    ///   年支卯 anchor 亥卯未组 → 桃花子/驿马巳/华盖未/将星卯 → 卯=将星 ✓
    /// - 月柱酉：日干己 → 命中 学堂（酉） + 文昌（酉）；年支卯 anchor → 酉非该组任一神煞 → 无；合 [学堂， 文昌]
    /// - 日柱巳：日干己 → 巳非任一日干锚位 → 无；年支卯 anchor → 巳=驿马；且非魁罡 → 合 [驿马]
    /// - 时柱申：日干己 → 无；年支卯 anchor → 申非该组任一 → 无 → 合 []
    #[test]
    fn shensha_1987_oracle() {
        let c = compute(BirthInput {
            year: 1987, month: 9, day: 17, hour: 15, minute: 0, tz: 8.0,
            gender: Some(Gender::Male),
        });
        assert_eq!(c.year.shensha, vec!["将星"]);
        // shensha_by_day_stem 内部顺序 = 羊刃/禄/文昌/红艳/学堂/词馆 → 文昌先于学堂
        assert_eq!(c.month.shensha, vec!["文昌", "学堂"]);
        assert_eq!(c.day.shensha, vec!["驿马"]);
        assert_eq!(c.hour.shensha, Vec::<String>::new());
    }

    /// 魁罡日柱触发：1980-09-13 22：00 男 → 看是不是庚辰/庚戌/壬辰/戊戌之一？
    /// 实际不知日柱，构造一个已知魁罡日：1984-04-29 （任查）。
    /// 改用 zi_hour 测试日柱 = 壬辰的样例。
    /// 1976-12-04 → 日柱壬辰？（查实际锚 2024-01-01=甲子 0，壬辰=28号）
    /// 简化：用合成 GanZhi 直接测 is_kuigang_day，主测「魁罡」字符串出现在 day.shensha 即可。
    #[test]
    fn shensha_kuigang_marker() {
        use mingli_ganzhi::is_kuigang_day;
        // 任意找一天日柱 = 庚戌（=06 戌 序号 47）？跑出来反测：
        // 1979-08-12 = JDN ？，日柱？
        // 直接 unit test ganzhi crate fn：
        assert!(is_kuigang_day(mingli_ganzhi::GanZhi { stem: 6, branch: 10 }));
        assert!(!is_kuigang_day(mingli_ganzhi::GanZhi { stem: 5, branch: 5 })); // 己巳
        // 1987 己巳日 → 非魁罡 → day.shensha 不含「魁罡」
        let c = compute(BirthInput {
            year: 1987, month: 9, day: 17, hour: 15, minute: 0, tz: 8.0,
            gender: Some(Gender::Male),
        });
        assert!(!c.day.shensha.contains(&"魁罡".to_string()));
    }

    #[test]
    fn hidden_stems_oracle() {
        // 1987-09-17：四柱 丁卯 己酉 己巳 壬申。
        let c = compute(BirthInput {
            year: 1987, month: 9, day: 17, hour: 15, minute: 0, tz: 8.0,
            gender: Some(Gender::Male),
        });
        let stems = |p: &Pillar| p.hidden.iter().map(|h| h.stem.clone()).collect::<Vec<_>>();
        assert_eq!(stems(&c.year), ["乙"]); // 卯藏乙
        assert_eq!(stems(&c.month), ["辛"]); // 酉藏辛
        assert_eq!(stems(&c.day), ["丙", "庚", "戊"]); // 巳藏丙庚戊
        assert_eq!(stems(&c.hour), ["庚", "壬", "戊"]); // 申藏庚壬戊
        // 支藏十神接线：日主己土，巳本气丙火生己土、阴阳异 → 正印。
        assert_eq!(c.day.hidden[0].ten_god, "正印");
        // 十二长生（日主己，阴干逆行，长生在酉）：年卯=病、月酉=长生、日巳=帝旺、时申=沐浴。
        assert_eq!(c.year.day_twelve, "病");
        assert_eq!(c.month.day_twelve, "长生");
        assert_eq!(c.day.day_twelve, "帝旺");
        assert_eq!(c.hour.day_twelve, "沐浴");
    }
}

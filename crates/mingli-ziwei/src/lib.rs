//! L3 叶：紫微斗数排盘。
//!
//! 确定性排盘：用 `mingli-astro` 农历换算 + `mingli-ganzhi` 干支 + `mingli-core` 的 Z₁₂ 群作用，
//! 定命宫/身宫、五行局、紫微星位置，再以固定位移安十四主星、布十二宫。
//! 起紫微算法对齐开源库 iztro 并经掌中诀多点验证。不含「释义」。
//! 闰月生人的「生月」取该月数字（闰月归本月），属已知简化。

#![allow(

    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap,
    reason = "十二宫/十四星均为 Z₁₂ 上的有界模运算，整数窄化安全"
)]

#[cfg(feature = "port")]
mod engine;
#[cfg(feature = "port")]
pub use engine::ZiweiEngine;

use mingli_astro::Moment;
use mingli_core::group::shift;
use mingli_ganzhi::{
    hour_branch, month_pillar_stem, nayin_element, year_ganzhi, Element, GanZhi, BRANCHES, STEMS,
};
#[cfg(feature = "serde")]
use serde::Serialize;

/// 十二宫名（自命宫起，逆时针即地支递减方向）。
pub(crate) const PALACE_NAMES: [&str; 12] = [
    "命宫", "兄弟", "夫妻", "子女", "财帛", "疾厄", "迁移", "交友", "官禄", "田宅", "福德", "父母",
];

pub mod limit;

/// 四化星流派。
///
/// 多源交叉验证仅确证两组分歧：庚干「太阴 vs 天府」化科（王亭之自述传授）、壬干「左辅 vs 天府」化科
/// （《紫微斗数全书》古本 vs 通行本）。戊/癸两干的派别分歧本次研究**未取得多源证据**，
/// 各派一致取通行表（在两派下完全一致）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum SihuaSchool {
    /// 通行版（中州/三合派，默认）：5 独立源完全一致(cnblogs/51xingli×2/vocus/wikipedia)。
    /// 庚=太阴化科、壬=左辅化科。
    #[default]
    Standard,
    /// 中州派（王亭之传授版）：主张左辅右弼属辅曜、**不入四化**，于是戊 = 太阳化科、
    /// 庚 = 天府化科、壬 = 天府化科；其余 7 干同通行版。
    ///
    /// 三干互为一体，不可只开其一（见 [`sihua_for`] 与本 crate 的 `SIHUA_ZHONGZHOU`）。
    /// 稳定 id 仍作 `"quanshu"`（历史沿用，不改以免破坏对外契约）。
    Quanshu,
}

impl SihuaSchool {
    /// 流派稳定 id（schools dropdown 用）。
    #[must_use]
    pub fn id(self) -> &'static str {
        match self {
            Self::Standard => "standard",
            Self::Quanshu => "quanshu",
        }
    }

    /// 从稳定 id 解析；未知 id → `None`。
    #[must_use]
    pub fn from_id(s: &str) -> Option<Self> {
        match s {
            "standard" => Some(Self::Standard),
            "quanshu" => Some(Self::Quanshu),
            _ => None,
        }
    }
}

/// 一个生年天干对应的四化星名（禄/权/科/忌）。
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct SihuaStars {
    /// 化禄星名。
    pub lu: &'static str,
    /// 化权星名。
    pub quan: &'static str,
    /// 化科星名。
    pub ke: &'static str,
    /// 化忌星名。
    pub ji: &'static str,
}

/// 通行版四化表（中州/三合派，默认）：10 天干 × 4 化星。
/// 索引 = stem id（甲=0 .. 癸=9）。
const SIHUA_STANDARD: [SihuaStars; 10] = [
    SihuaStars { lu: "廉贞", quan: "破军", ke: "武曲", ji: "太阳" }, // 甲
    SihuaStars { lu: "天机", quan: "天梁", ke: "紫微", ji: "太阴" }, // 乙
    SihuaStars { lu: "天同", quan: "天机", ke: "文昌", ji: "廉贞" }, // 丙
    SihuaStars { lu: "太阴", quan: "天同", ke: "天机", ji: "巨门" }, // 丁
    SihuaStars { lu: "贪狼", quan: "太阴", ke: "右弼", ji: "天机" }, // 戊
    SihuaStars { lu: "武曲", quan: "贪狼", ke: "天梁", ji: "文曲" }, // 己
    SihuaStars { lu: "太阳", quan: "武曲", ke: "太阴", ji: "天同" }, // 庚（通行）
    SihuaStars { lu: "巨门", quan: "太阳", ke: "文曲", ji: "文昌" }, // 辛
    SihuaStars { lu: "天梁", quan: "紫微", ke: "左辅", ji: "武曲" }, // 壬（通行）
    SihuaStars { lu: "破军", quan: "巨门", ke: "太阴", ji: "贪狼" }, // 癸
];

/// 中州派（王亭之）四化表：戊 = 太阳化科、庚 = 天府化科、壬 = 天府化科；其余 7 干同通行版。
///
/// 这三处不是三条孤立的异文，而是**同一条学理的三个后果**——该派主张左辅右弼属辅曜、
/// 不入四化，于是通行表里由右弼化科的戊、由左辅化科的壬都要换星，庚随之一并调整。
/// 王亭之原话：「戊干，通行作［右弼化科］；壬干，通行作［左弼化科］，然而［中州派］所传，
/// 左辅右弼却不化科」。因此**三干必须一起开**，只改其中一两处会让这个流派自相矛盾。
const SIHUA_ZHONGZHOU: [SihuaStars; 10] = [
    SIHUA_STANDARD[0],
    SIHUA_STANDARD[1],
    SIHUA_STANDARD[2],
    SIHUA_STANDARD[3],
    SihuaStars { lu: "贪狼", quan: "太阴", ke: "太阳", ji: "天机" }, // 戊（中州派：右弼 → 太阳）
    SIHUA_STANDARD[5],
    SihuaStars { lu: "太阳", quan: "武曲", ke: "天府", ji: "天同" }, // 庚（中州派）
    SIHUA_STANDARD[7],
    SihuaStars { lu: "天梁", quan: "紫微", ke: "天府", ji: "武曲" }, // 壬（中州派：左辅 → 天府）
    SIHUA_STANDARD[9],
];

/// 取生年天干在指定流派下的四化星名。`stem_id` ∈ 0..10（甲=0）。
#[must_use]
pub fn sihua_for(stem_id: u8, school: SihuaSchool) -> SihuaStars {
    let idx = (stem_id % 10) as usize;
    match school {
        SihuaSchool::Standard => SIHUA_STANDARD[idx],
        SihuaSchool::Quanshu => SIHUA_ZHONGZHOU[idx],
    }
}

/// 四化排盘结果（星名 + 落入宫位地支）。落宫由排盘扫 18 颗（十四主星 + 4 辅星）反查；
/// 若该化星不在前述 18 颗中（罕见），`*_branch` 为 `None`。
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct Sihua {
    /// 流派 id(`standard` / `quanshu`)。
    pub school_id: &'static str,
    /// 化禄星名。
    pub lu_star: &'static str,
    /// 化禄落入地支（若已安星）；否则 `None`。
    pub lu_branch: Option<String>,
    /// 化权星名。
    pub quan_star: &'static str,
    /// 化权落入地支；同上。
    pub quan_branch: Option<String>,
    /// 化科星名。
    pub ke_star: &'static str,
    /// 化科落入地支；同上。
    pub ke_branch: Option<String>,
    /// 化忌星名。
    pub ji_star: &'static str,
    /// 化忌落入地支；同上。
    pub ji_branch: Option<String>,
}

/// 性别。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
pub enum Gender {
    /// 男。
    Male,
    /// 女。
    Female,
}

/// 出生信息（排盘输入）。
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(Serialize))]
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
    /// 性别；命盘本体不依赖，保留以备大限/流年扩展。
    pub gender: Option<Gender>,
}

/// 一宫。
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct Palace {
    /// 宫名（命宫/兄弟/…/父母）。
    pub name: String,
    /// 宫位地支。
    pub branch: String,
    /// 宫干支。
    pub ganzhi: String,
    /// 落入本宫的主星。
    pub stars: Vec<String>,
    /// 是否命宫。
    pub is_ming: bool,
    /// 是否身宫。
    pub is_shen: bool,
}

/// 农历日期。
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
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

/// 一张紫微斗数命盘。
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct ZiweiChart {
    /// 排盘输入。
    pub input: BirthInput,
    /// 农历日期。
    pub lunar: LunarChart,
    /// 命宫地支。
    pub ming_branch: String,
    /// 大限盘（十年一宫）。性别缺省时为 `None`——顺逆由「年干阴阳 + 性别」定，缺一不可。
    pub major_limits: Option<limit::MajorLimits>,
    /// 身宫地支。
    pub shen_branch: String,
    /// 命宫干支。
    pub ming_ganzhi: String,
    /// 五行局名（如「土五局」）。
    pub wuxing_ju: String,
    /// 五行局数（2/3/4/5/6）。
    pub ju_number: u32,
    /// 紫微星所在地支。
    pub ziwei_branch: String,
    /// 天府星所在地支。
    pub tianfu_branch: String,
    /// 十二宫，按地支 子..亥 顺序排列。
    pub palaces: Vec<Palace>,
    /// 四化（生年天干→禄/权/科/忌四星 + 落宫地支）。
    pub sihua: Sihua,
}

/// 起紫微：由农历生日与五行局数，求紫微所在地支（子=0 约定）。
/// 算法对齐 iztro getStartIndex（内部寅=0，末尾 +2 转子=0）。
///
/// # Panics
///
/// `ju` 为 0 时 panic。五行局只有 2..=6 五种，0 不是合法局数。
#[must_use]
pub fn ziwei_branch(day: u32, ju: u32) -> u8 {
    // 补足数：让日数加到能被局数整除的最小非负增量。
    //
    // 从前这里是 `while rem != 0` 配两个哨兵（`offset = -1`、`rem = -1`），
    // 循环没有上限——变异扫描在里面留了三个**超时**：把 `+=` 改成 `*=`、
    // 把 `%` 改成 `/`，循环就再也退不出来，测试挂在那里而不是红。
    //
    // 上限本来就有，只是没写出来：连续 `ju` 个整数里必有一个是 `ju` 的倍数，
    // 所以补足数一定落在 `0..ju` 内。写成有界的查找之后，哨兵和那三个超时一起没了。
    let ju = i64::from(ju);
    let day = i64::from(day);
    let offset = (0..ju)
        .find(|o| (day + o) % ju == 0)
        .expect("连续 ju 个整数里必有 ju 的倍数");
    // 不必先取模 12：下面的 shift 用 rem_euclid(12) 归一，多这一步只是让
    // 「% 12」和「+ 12」变成同一个结果的两种写法（变异扫描据此报过一个漏网）。
    let quotient = (day + offset) / ju;
    // 寅=0 编号下：商定基准，补足数(offset)偶进奇退。
    let z = shift(quotient - 1, offset, 12, offset % 2 == 0);
    ((z + 2) % 12) as u8 // 转子=0
}

fn ju_from_element(e: Element) -> u32 {
    match e {
        Element::Water => 2, // 水二局
        Element::Wood => 3,  // 木三局
        Element::Metal => 4, // 金四局
        Element::Earth => 5, // 土五局
        Element::Fire => 6,  // 火六局
    }
}

fn ju_name(ju: u32) -> &'static str {
    match ju {
        2 => "水二局",
        3 => "木三局",
        4 => "金四局",
        5 => "土五局",
        _ => "火六局",
    }
}

/// 安十四主星 + 4 辅星到 12 宫（`zw` = 紫微地支，`tf` = 天府地支，`hb` = 生时支（子=0），`m` = 农历月）。
///
/// 紫微星系六颗（紫微/天机/太阳/武曲/天同/廉贞）逆行 + 天府星系八颗顺行（天府/太阴/贪狼/巨门/天相/天梁/七杀/破军）；
/// 4 辅星（文昌/文曲/左辅/右弼）按古典通行口诀：文昌 （10-时支） mod 12、文曲 （4+时支） mod 12、
/// 左辅 （4+月-1） mod 12、右弼 （10-（月-1）） mod 12。
fn arrange_stars(zw: u8, tf: u8, hb: i64, m: i64) -> [Vec<&'static str>; 12] {
    let mut stars: [Vec<&'static str>; 12] = Default::default();
    // 紫微星系（逆行）
    for (name, off) in [("紫微", 0), ("天机", -1), ("太阳", -3), ("武曲", -4), ("天同", -5), ("廉贞", -8)] {
        stars[(i32::from(zw) + off).rem_euclid(12) as usize].push(name);
    }
    // 天府星系（顺行）
    for (name, off) in [("天府", 0), ("太阴", 1), ("贪狼", 2), ("巨门", 3), ("天相", 4), ("天梁", 5), ("七杀", 6), ("破军", 10)] {
        stars[(i32::from(tf) + off).rem_euclid(12) as usize].push(name);
    }
    // 4 辅星
    stars[(10 - hb).rem_euclid(12) as usize].push("文昌");
    stars[(4 + hb).rem_euclid(12) as usize].push("文曲");
    stars[(4 + m - 1).rem_euclid(12) as usize].push("左辅");
    stars[(10 - (m - 1)).rem_euclid(12) as usize].push("右弼");
    stars
}

/// 由生年天干 + 流派 + 已布好的 12 宫，产生四化排盘结果（星名 + 落入地支）。
fn compute_sihua(stem_id: u8, school: SihuaSchool, palaces: &[Palace]) -> Sihua {
    let s = sihua_for(stem_id, school);
    let find_branch = |name: &str| -> Option<String> {
        palaces.iter().find(|p| p.stars.iter().any(|x| x == name)).map(|p| p.branch.clone())
    };
    Sihua {
        school_id: school.id(),
        lu_star: s.lu, lu_branch: find_branch(s.lu),
        quan_star: s.quan, quan_branch: find_branch(s.quan),
        ke_star: s.ke, ke_branch: find_branch(s.ke),
        ji_star: s.ji, ji_branch: find_branch(s.ji),
    }
}

/// 排紫微斗数命盘（独立入口：自行构造共享上下文 [`Moment`]）。默认 [`SihuaSchool::Standard`] 通行版四化。
#[must_use]
pub fn compute(input: BirthInput) -> ZiweiChart {
    compute_with(input, SihuaSchool::default())
}

/// 排紫微斗数命盘 + 指定四化流派（独立入口）。
#[must_use]
pub fn compute_with(input: BirthInput, school: SihuaSchool) -> ZiweiChart {
    let moment = Moment::new(
        input.year,
        input.month,
        input.day,
        input.hour,
        input.minute,
        input.tz,
    );
    compute_at_with(&moment, input.gender, school)
}

/// 在已算好的共享上下文 [`Moment`] 上排紫微——供 DAG 引擎复用同一 `Moment`、零重算历法。
/// 默认 [`SihuaSchool::Standard`] 通行版四化。
#[must_use]
pub fn compute_at(moment: &Moment, gender: Option<Gender>) -> ZiweiChart {
    compute_at_with(moment, gender, SihuaSchool::default())
}

/// 在共享上下文 [`Moment`] 上排紫微 + 指定四化流派。
#[must_use]
pub fn compute_at_with(moment: &Moment, gender: Option<Gender>, school: SihuaSchool) -> ZiweiChart {
    let lunar = moment.lunar;
    let hb = i64::from(hour_branch(moment.hour, moment.minute)); // 生时支 （子=0）
    let m = i64::from(lunar.month); // 生月（闰月按本月数字）

    // 寅(2)起正月顺数生月 → 生月宫；再起子时，命宫逆数生时、身宫顺数生时。
    let month_palace = shift(2, m - 1, 12, true);
    let ming = shift(month_palace, hb, 12, false) as u8;
    let shen = shift(month_palace, hb, 12, true) as u8;

    // 年干（农历年）→ 命宫天干（五虎遁）→ 纳音五行 → 五行局
    let year_gz = year_ganzhi(lunar.year);
    let ming_gz = GanZhi {
        stem: month_pillar_stem(year_gz.stem, ming),
        branch: ming,
    };
    let ju = ju_from_element(nayin_element(ming_gz));

    // 紫微 & 天府（天府以寅申轴对称：tf = （4 − 紫微） mod 12）
    let zw = ziwei_branch(lunar.day, ju);
    let tf = (4 - i32::from(zw)).rem_euclid(12) as u8;

    // 十四主星 + 4 辅星布盘
    let stars = arrange_stars(zw, tf, hb, m);

    // 十二宫名：命宫在 ming，兄弟在 ming-1 …（逆时针）。
    let mut palace_of: [&str; 12] = [""; 12];
    for (i, &pname) in PALACE_NAMES.iter().enumerate() {
        palace_of[shift(i64::from(ming), i as i64, 12, false) as usize] = pname;
    }

    let palaces: Vec<Palace> = (0..12u8)
        .map(|b| {
            let stem = month_pillar_stem(year_gz.stem, b);
            Palace {
                name: palace_of[b as usize].to_string(),
                branch: BRANCHES[b as usize].to_string(),
                ganzhi: format!("{}{}", STEMS[stem as usize], BRANCHES[b as usize]),
                stars: stars[b as usize].iter().map(|s| (*s).to_string()).collect(),
                is_ming: b == ming,
                is_shen: b == shen,
            }
        })
        .collect();

    let sihua = compute_sihua(year_gz.stem, school, &palaces);

    ZiweiChart {
        input: BirthInput {
            year: moment.year,
            month: moment.month,
            day: moment.day,
            hour: moment.hour,
            minute: moment.minute,
            tz: moment.tz,
            gender,
        },
        lunar: LunarChart {
            year: lunar.year,
            month: lunar.month,
            leap: lunar.leap,
            day: lunar.day,
        },
        ming_branch: BRANCHES[ming as usize].to_string(),
        major_limits: gender.map(|g| {
            limit::major_limits(ming, ju, year_gz.stem, matches!(g, Gender::Male))
        }),
        shen_branch: BRANCHES[shen as usize].to_string(),
        ming_ganzhi: ming_gz.to_string(),
        wuxing_ju: ju_name(ju).to_string(),
        ju_number: ju,
        ziwei_branch: BRANCHES[zw as usize].to_string(),
        tianfu_branch: BRANCHES[tf as usize].to_string(),
        palaces,
        sihua,
    }
}

#[cfg(test)]
mod tests;

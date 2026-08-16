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

mod engine;
pub use engine::ZiweiEngine;

use mingli_astro::Moment;
use mingli_core::group::shift;
use mingli_ganzhi::{
    hour_branch, month_pillar_stem, nayin_element, year_ganzhi, Element, GanZhi, BRANCHES, STEMS,
};
use serde::Serialize;

/// 十二宫名（自命宫起，逆时针即地支递减方向）。
const PALACE_NAMES: [&str; 12] = [
    "命宫", "兄弟", "夫妻", "子女", "财帛", "疾厄", "迁移", "交友", "官禄", "田宅", "福德", "父母",
];

/// 四化星流派。
///
/// 多源交叉验证仅确证两组分歧：庚干「太阴 vs 天府」化科（王亭之自述传授）、壬干「左辅 vs 天府」化科
/// （《紫微斗数全书》古本 vs 通行本）。戊/癸两干的派别分歧本次研究**未取得多源证据**，
/// 各派一致取通行表（在两派下完全一致）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SihuaSchool {
    /// 通行版（中州/三合派，默认）：5 独立源完全一致(cnblogs/51xingli×2/vocus/wikipedia)。
    /// 庚=太阴化科、壬=左辅化科。
    #[default]
    Standard,
    /// 全书本（王亭之传授版）：庚=天府化科（王亭之亲文）、壬=天府化科（《全书》古本）。其余 8 干同通行版。
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
#[derive(Debug, Clone, Copy, Serialize)]
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

/// 全书本四化表（王亭之版）：庚 = 天府化科；壬 = 天府化科；其余 8 干同通行版。
const SIHUA_QUANSHU: [SihuaStars; 10] = [
    SIHUA_STANDARD[0],
    SIHUA_STANDARD[1],
    SIHUA_STANDARD[2],
    SIHUA_STANDARD[3],
    SIHUA_STANDARD[4],
    SIHUA_STANDARD[5],
    SihuaStars { lu: "太阳", quan: "武曲", ke: "天府", ji: "天同" }, // 庚（全书本）
    SIHUA_STANDARD[7],
    SihuaStars { lu: "天梁", quan: "紫微", ke: "天府", ji: "武曲" }, // 壬（全书本）
    SIHUA_STANDARD[9],
];

/// 取生年天干在指定流派下的四化星名。`stem_id` ∈ 0..10（甲=0）。
#[must_use]
pub fn sihua_for(stem_id: u8, school: SihuaSchool) -> SihuaStars {
    let idx = (stem_id % 10) as usize;
    match school {
        SihuaSchool::Standard => SIHUA_STANDARD[idx],
        SihuaSchool::Quanshu => SIHUA_QUANSHU[idx],
    }
}

/// 四化排盘结果（星名 + 落入宫位地支）。落宫由排盘扫 18 颗（十四主星 + 4 辅星）反查；
/// 若该化星不在前述 18 颗中（罕见），`*_branch` 为 `None`。
#[derive(Debug, Clone, Serialize)]
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
    /// 性别；命盘本体不依赖，保留以备大限/流年扩展。
    pub gender: Option<Gender>,
}

/// 一宫。
#[derive(Debug, Clone, Serialize)]
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

/// 一张紫微斗数命盘。
#[derive(Debug, Clone, Serialize)]
pub struct ZiweiChart {
    /// 排盘输入。
    pub input: BirthInput,
    /// 农历日期。
    pub lunar: LunarChart,
    /// 命宫地支。
    pub ming_branch: String,
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
#[must_use]
pub fn ziwei_branch(day: u32, ju: u32) -> u8 {
    let mut offset: i64 = -1;
    let mut rem: i64 = -1;
    let mut quotient: i64 = 0;
    while rem != 0 {
        offset += 1;
        let d = i64::from(day) + offset;
        quotient = d / i64::from(ju);
        rem = d % i64::from(ju);
    }
    quotient %= 12;
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
mod tests {
    use super::*;

    // 起紫微多点验证（用 iztro + 掌中诀核对，子=0 编号）
    #[test]
    fn ziwei_position_points() {
        assert_eq!(ziwei_branch(23, 5), 8); // 申
        assert_eq!(ziwei_branch(23, 4), 6); // 午
        assert_eq!(ziwei_branch(28, 2), 3); // 卯
        assert_eq!(ziwei_branch(1, 2), 1); // 丑
        assert_eq!(ziwei_branch(1, 3), 4); // 辰
        assert_eq!(ziwei_branch(1, 4), 11); // 亥
        assert_eq!(ziwei_branch(1, 5), 6); // 午
        assert_eq!(ziwei_branch(2, 5), 11); // 亥
        assert_eq!(ziwei_branch(2, 6), 6); // 午
    }

    #[test]
    fn sample_1990_06_15() {
        const MAJOR: [&str; 14] = [
            "紫微","天机","太阳","武曲","天同","廉贞",
            "天府","太阴","贪狼","巨门","天相","天梁","七杀","破军",
        ];
        let chart = compute(BirthInput {
            year: 1990,
            month: 6,
            day: 15,
            hour: 14,
            minute: 30,
            tz: 8.0,
            gender: Some(Gender::Male),
        });
        assert_eq!(chart.ming_branch, "亥");
        assert_eq!(chart.shen_branch, "丑"); // 身宫在福德（丑）
        assert_eq!(chart.ming_ganzhi, "丁亥");
        assert_eq!(chart.wuxing_ju, "土五局");
        assert_eq!(chart.ju_number, 5);
        assert_eq!(chart.ziwei_branch, "申");
        assert_eq!(chart.tianfu_branch, "申");
        // 命宫（亥）主星应含巨门
        let ming = chart.palaces.iter().find(|p| p.is_ming).unwrap();
        assert!(
            ming.stars.iter().any(|s| s == "巨门"),
            "命宫主星应含巨门，实得 {:?}",
            ming.stars
        );
        // 十四主星 + 4 辅星（文昌/文曲/左辅/右弼） = 18 颗，无遗漏。
        let total: usize = chart.palaces.iter().map(|p| p.stars.len()).sum();
        assert_eq!(total, 18);
        // 单独校验 14 主星仍齐（过滤 4 辅星）。
        let major_count: usize = chart.palaces.iter()
            .flat_map(|p| p.stars.iter())
            .filter(|s| MAJOR.contains(&s.as_str())).count();
        assert_eq!(major_count, 14);
        // 十二宫名俱全
        assert!(chart.palaces.iter().any(|p| p.name == "福德"));
    }

    #[test]
    fn ju_mapping_all() {
        for (e, n, nm) in [
            (Element::Water, 2, "水二局"),
            (Element::Wood, 3, "木三局"),
            (Element::Metal, 4, "金四局"),
            (Element::Earth, 5, "土五局"),
            (Element::Fire, 6, "火六局"),
        ] {
            assert_eq!(ju_from_element(e), n);
            assert_eq!(ju_name(n), nm);
        }
    }

    #[test]
    fn aux_stars_1990_06_15_oracle() {
        // 1990-06-15 14：30 CST： 农历五月廿三 未时（时支=7）
        // 公式校验：文昌=(10-7)%12=3=卯、文曲=(4+7)%12=11=亥、左辅=(4+5-1)%12=8=申、右弼=(10-(5-1))%12=6=午。
        let chart = compute(BirthInput {
            year: 1990, month: 6, day: 15, hour: 14, minute: 30, tz: 8.0,
            gender: Some(Gender::Male),
        });
        let where_star = |s: &str| {
            chart.palaces.iter().find(|p| p.stars.iter().any(|x| x == s)).map(|p| p.branch.clone())
        };
        assert_eq!(where_star("文昌").as_deref(), Some("卯"));
        assert_eq!(where_star("文曲").as_deref(), Some("亥"));
        assert_eq!(where_star("左辅").as_deref(), Some("申"));
        assert_eq!(where_star("右弼").as_deref(), Some("午"));
        // 18 颗（十四主星 + 4 辅星）无遗漏。
        let total: usize = chart.palaces.iter().map(|p| p.stars.len()).sum();
        assert_eq!(total, 18);
    }

    #[test]
    fn sihua_1990_geng_standard_school_oracle() {
        // 1990 庚午年（年干=庚， stem_id=6）。通行版四化：太阳禄/武曲权/太阴科/天同忌。
        // 落宫（已知盘）：太阳=巳、武曲=辰、太阴=酉、天同=卯。
        let chart = compute(BirthInput {
            year: 1990, month: 6, day: 15, hour: 14, minute: 30, tz: 8.0,
            gender: Some(Gender::Male),
        });
        assert_eq!(chart.sihua.school_id, "standard");
        assert_eq!(chart.sihua.lu_star, "太阳");
        assert_eq!(chart.sihua.lu_branch.as_deref(), Some("巳"));
        assert_eq!(chart.sihua.quan_star, "武曲");
        assert_eq!(chart.sihua.quan_branch.as_deref(), Some("辰"));
        assert_eq!(chart.sihua.ke_star, "太阴");
        assert_eq!(chart.sihua.ke_branch.as_deref(), Some("酉"));
        assert_eq!(chart.sihua.ji_star, "天同");
        assert_eq!(chart.sihua.ji_branch.as_deref(), Some("卯"));
    }

    #[test]
    fn sihua_1990_geng_quanshu_school_oracle() {
        // 同 1990 庚午，全书本（王亭之版）：太阳禄/武曲权/天府科/天同忌。
        // 天府=申 → 化科分歧。其余三化同通行。
        let chart = compute_with(
            BirthInput { year: 1990, month: 6, day: 15, hour: 14, minute: 30, tz: 8.0, gender: Some(Gender::Male) },
            SihuaSchool::Quanshu,
        );
        assert_eq!(chart.sihua.school_id, "quanshu");
        assert_eq!(chart.sihua.lu_star, "太阳"); // 与通行同
        assert_eq!(chart.sihua.lu_branch.as_deref(), Some("巳"));
        assert_eq!(chart.sihua.quan_star, "武曲");
        assert_eq!(chart.sihua.quan_branch.as_deref(), Some("辰"));
        assert_eq!(chart.sihua.ke_star, "天府"); // 分歧：通行=太阴、全书=天府
        assert_eq!(chart.sihua.ke_branch.as_deref(), Some("申"));
        assert_eq!(chart.sihua.ji_star, "天同");
        assert_eq!(chart.sihua.ji_branch.as_deref(), Some("卯"));
    }

    #[test]
    fn sihua_table_only_diverges_at_geng_and_ren() {
        // 两派表只在庚(6)与壬(8)的化科上不同；其余 8 干两派全等。
        for stem_id in 0..10u8 {
            let s = sihua_for(stem_id, SihuaSchool::Standard);
            let q = sihua_for(stem_id, SihuaSchool::Quanshu);
            assert_eq!(s.lu, q.lu, "stem {stem_id}");
            assert_eq!(s.quan, q.quan, "stem {stem_id}");
            assert_eq!(s.ji, q.ji, "stem {stem_id}");
            if stem_id == 6 || stem_id == 8 {
                assert_ne!(s.ke, q.ke, "stem {stem_id} 科应分歧");
            } else {
                assert_eq!(s.ke, q.ke, "stem {stem_id} 科应一致");
            }
        }
    }

    #[test]
    fn compute_at_default_school_equals_standard() {
        let mo = Moment::new(1990, 6, 15, 14, 30, 8.0);
        let a = compute_at(&mo, Some(Gender::Male));
        let b = compute_at_with(&mo, Some(Gender::Male), SihuaSchool::Standard);
        assert_eq!(a.sihua.school_id, b.sihua.school_id);
        assert_eq!(a.sihua.ke_star, b.sihua.ke_star);
        assert_eq!(a.ming_branch, b.ming_branch);
    }

    #[test]
    fn sihua_school_id_roundtrip() {
        for s in [SihuaSchool::Standard, SihuaSchool::Quanshu] {
            assert_eq!(SihuaSchool::from_id(s.id()), Some(s));
        }
        assert_eq!(SihuaSchool::from_id("unknown"), None);
        assert_eq!(SihuaSchool::default(), SihuaSchool::Standard);
    }

    #[test]
    fn no_gender_ok() {
        let chart = compute(BirthInput {
            year: 2000,
            month: 1,
            day: 1,
            hour: 0,
            minute: 0,
            tz: 8.0,
            gender: None,
        });
        assert_eq!(chart.palaces.len(), 12);
    }
}

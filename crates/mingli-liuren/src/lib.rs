//! L3 叶（⟂ 横切 / 确定性）：大六壬起课。
//!
//! 大六壬把「时间」折成一个 `Z₁₂` 上的盘旋转，再由日干支取四课、发三传：
//!
//! 1. **天地盘**：地盘十二支固定顺布；天盘 = 地盘整体平移，偏移 `offset = (月将支 − 时支) mod 12`
//!    （「月将加占时」）。地盘第 `g` 宫之上神 = `(g + offset) mod 12`（[`heaven_plate`]）。
//! 2. **月将**：太阳过宫，每过一中气换将，随黄经递减（[`month_general_branch`]）；雨水后日躔亥=登明。
//! 3. **四课**：用天干寄宫（[`STEM_LODGING`]）取一课，层层取天盘上神得四课（[`four_courses`]）。
//! 4. **三传**：先判课式（九宗门），再取传。
//!
//! 验证：天地盘 + 四课校验古法工作例「亥将子时甲子日 → 四课 丑/子/亥/戌」。
//!
//! 九宗门里八门已取传：贼克 / 比用 / 遥克 / 伏吟 / 返吟（有克者）/ 昴星 / 别责 / 八专；
//! 涉害亦取传，但**取用法两派**（数不数「受克深浅」），见 [`SheHaiSchool`]。
//! 唯一仍留空的是返吟无克一路（井栏射等），取传细则未查。
//!
//! 三门的取传各自有一张**全表课数**可对账，这是本叶最硬的校验面：
//! 昴星恰 16 课（刚 4 柔 12）、别责恰 9 课（刚 3 柔 6）、八专恰 16 课（刚 6 柔 10），
//! 且八专里三传三字全同的「独足课」有且仅有一课。这些数字在《六壬大全》与《六壬粹言》
//! 两部彼此独立的书里各自被自报过。

#![allow(

    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::trivially_copy_pass_by_ref,
    reason = "盘位全在 Z₁₂（0..12）小范围内换算；Course/数组按引用传是为可读性，受控安全"
)]

mod engine;
pub use engine::LiurenEngine;

use mingli_astro::Moment;
use mingli_ganzhi::branch_element;
use serde::Serialize;

/// 十二月将名，按地支序索引（子=0…亥=11）。
pub const MONTH_GENERAL_NAMES: [&str; 12] = [
    "神后", "大吉", "功曹", "太冲", "天罡", "太乙", "胜光", "小吉", "传送", "从魁", "河魁", "登明",
];

/// 天干寄宫：天干（甲=0…癸=9）寄于某地支宫。四正（子午卯酉）不作寄宫，故丙戊同寄巳、丁己同寄未。
pub const STEM_LODGING: [u8; 10] = [2, 4, 5, 7, 5, 7, 8, 10, 11, 1];

/// 由太阳视黄经定月将地支（0..11）。每过一中气（黄经每 30°）月将递减；
/// `λ∈[0,30)`→戌(10)、`λ∈[330,360)`→亥(11)（雨水后日躔亥=登明）。
#[must_use]
pub fn month_general_branch(sun_longitude: f64) -> u8 {
    let s = (sun_longitude.rem_euclid(360.0) / 30.0).floor();
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "s∈0..12"
    )]
    let s = s as i64;
    ((10 - s).rem_euclid(12)) as u8
}

/// 天地盘偏移：`(月将支 − 时支) mod 12`（月将加占时）。
#[must_use]
pub fn plate_offset(month_general: u8, hour_branch: u8) -> u8 {
    (12 + month_general - hour_branch) % 12
}

/// 地盘第 `ground` 宫之上的天盘地支：`(ground + offset) mod 12`。
#[must_use]
pub fn heaven_plate(ground: u8, offset: u8) -> u8 {
    (ground + offset) % 12
}

/// 一课：下（地盘支）与上（天盘上神）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Course {
    /// 下神（地盘地支序 0..11）。
    pub down: u8,
    /// 上神（天盘地支序 0..11）。
    pub up: u8,
}

/// 起四课：一课=日干寄宫之上神；二课=一课上神之上神；三课=日支之上神；四课=三课上神之上神。
#[must_use]
pub fn four_courses(day_stem: u8, day_branch: u8, offset: u8) -> [Course; 4] {
    let c1d = STEM_LODGING[day_stem as usize];
    let c1u = heaven_plate(c1d, offset);
    let c2u = heaven_plate(c1u, offset);
    let c3u = heaven_plate(day_branch, offset);
    let c4u = heaven_plate(c3u, offset);
    [
        Course { down: c1d, up: c1u },
        Course { down: c1u, up: c2u },
        Course { down: day_branch, up: c3u },
        Course { down: c3u, up: c4u },
    ]
}

/// 三传课式（九宗门）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub enum Pattern {
    /// 重审（下贼上，取受贼之上神）。
    ZhongShen,
    /// 元首（上克下，取受克之上神）。
    YuanShou,
    /// 比用（克 ≥2，取与日干同阴阳之上神）。
    BiYong,
    /// 涉害（俱比/俱不比，数克深浅定，🟡 流派分歧不强编）。
    SheHai,
    /// 遥克·蒿矢（无上下克，天盘神克日）。
    HaoShi,
    /// 遥克·弹射（无上下克，日克天盘神）。
    TanShe,
    /// 昴星（无克无遥克，四课全，🟡 取传不强编）。
    MaoXing,
    /// 别责（四课不全，🟡 取传不强编）。
    BieZe,
    /// 八专（日干支同位，🟡 取传不强编）。
    BaZhuan,
    /// 伏吟（月将==时，天地同位）。
    FuYin,
    /// 返吟（天地相冲，offset==6）。
    FanYin,
}

impl Pattern {
    /// 课式中文名。
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Pattern::ZhongShen => "重审",
            Pattern::YuanShou => "元首",
            Pattern::BiYong => "比用",
            Pattern::SheHai => "涉害",
            Pattern::HaoShi => "蒿矢",
            Pattern::TanShe => "弹射",
            Pattern::MaoXing => "昴星",
            Pattern::BieZe => "别责",
            Pattern::BaZhuan => "八专",
            Pattern::FuYin => "伏吟",
            Pattern::FanYin => "返吟",
        }
    }
}

/// 一次大六壬起课的结果。
#[derive(Debug, Clone, Serialize)]
pub struct Cast {
    /// 日干（甲=0…癸=9）。
    pub day_stem: u8,
    /// 日支（子=0…亥=11）。
    pub day_branch: u8,
    /// 占时地支（子=0…亥=11）。
    pub hour_branch: u8,
    /// 月将地支（子=0…亥=11）。
    pub month_general: u8,
    /// 月将名。
    pub month_general_name: &'static str,
    /// 天地盘偏移。
    pub offset: u8,
    /// 天盘十二支：`heaven[g]` = 地盘 g 宫上神。
    pub heaven: [u8; 12],
    /// 四课。
    pub courses: [Course; 4],
    /// 三传课式。
    pub pattern: Pattern,
    /// 课式中文名。
    pub pattern_label: &'static str,
    /// 三传（初/中/末，地支序），仅在取传规则明确时给出；🟡 流派分歧的课式为 `None`。
    pub transmission: Option<[u8; 3]>,
}

/// 涉害的取用法两派，且**两派都不是抄错**——各有多源、各自点名对方。
///
/// - [`Classical`](SheHaiSchool::Classical)：古法。先数「受克深浅」，深者为用；深浅相等才按孟仲季。
///   《六壬大全》卷一歌诀「涉害行来本家止，路逢多克为用取」、卷七《课经》《袖中金》《观月经》、
///   《御定六壬直指》、《六壬粹言》卷一「此古法也」皆主此。本 crate 的六个古籍算例复算全中。
/// - [`ByPosition`](SheHaiSchool::ByPosition)：近法。不数深浅，直接孟 ＞ 仲 ＞ 季。
///   陈公献《六壬指南》系明言「涉害取法，只以孟仲季为准，**不以涉害深浅为义**，此《指南》所用之法，切记」；
///   《六壬粹言》卷一亦记「近来诸家，均未用之者」。
///
/// 默认取古法：它被算例直接支持，且近法只是它去掉第一步。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SheHaiSchool {
    /// 古法：先数深浅。
    #[default]
    Classical,
    /// 近法：只按孟仲季。
    ByPosition,
}

impl SheHaiSchool {
    /// 稳定 id。
    #[must_use]
    pub const fn id(self) -> &'static str {
        match self {
            Self::Classical => "classical",
            Self::ByPosition => "by_position",
        }
    }

    /// 由稳定 id 解析；未知 → `None`。
    #[must_use]
    pub fn from_id(s: &str) -> Option<Self> {
        match s {
            "classical" => Some(Self::Classical),
            "by_position" => Some(Self::ByPosition),
            _ => None,
        }
    }
}

/// 地盘位的孟仲季档次：孟（寅申巳亥）0 ＞ 仲（子午卯酉）1 ＞ 季（辰戌丑未）2。
///
/// **看的是天盘神所临的地盘位，不是天盘神自己**——《六壬粹言》的复等例把这一点钉死了：
/// 戊辰日一课子加巳、四课午加亥，子午本是仲，书却说「俱在孟位上」，因为巳、亥是孟。
fn meng_zhong_ji(ground: u8) -> u8 {
    match ground {
        2 | 8 | 5 | 11 => 0,
        0 | 6 | 3 | 9 => 1,
        _ => 2,
    }
}

/// 涉害「受克深浅」：天盘神 `up` 临地盘 `down`，自 `down` 的下一格顺行到 `up` 本家的前一格，
/// 沿途每遇一个克 `up` 的住户记一重。住户 = 该地支本身 ＋ 寄于该宫的天干。
///
/// 两处边界由古籍算例定死，都**不计**：起点（`down` 本身）与终点（`up` 的本家）。
/// 《观月经》甲辰日「子加辰……巳上戊土、未土、未上己土、前又戌土，共四重」——
/// 起点辰本身是土（克子水）却不在账上；「未土 ＋ 未上己土」分开记两重，故寄干单独计。
/// 《课经》甲午日「辰加寅，历卯木一重」——终点辰的寄干乙木若计就是两重，作一重故本家不计。
fn shehai_depth(course: &Course) -> u32 {
    let target = branch_element(course.up);
    let mut depth = 0;
    let mut g = (course.down + 1) % 12;
    while g != course.up {
        if branch_element(g).controls() == target {
            depth += 1;
        }
        for stem in 0..10u8 {
            if STEM_LODGING[stem as usize] == g
                && mingli_ganzhi::stem_element(stem).controls() == target
            {
                depth += 1;
            }
        }
        g = (g + 1) % 12;
    }
    depth
}

/// 四课「下位」的五行。
///
/// 一课写作「干上神／**日干**」——下位是日干本身，不是它的寄宫。这一条要紧：
/// 乙丁戊辛癸五干的寄宫地支五行与干不同（乙木寄辰土、丁火寄未土、戊土寄巳火、
/// 辛金寄戌土、癸水寄丑土），拿寄宫五行去判贼克，这五干的一课会判错，
/// 连带把课式判到别的门里去。二三四课的下位都是地支，照常取支五行。
fn down_element(idx: usize, c: &Course, day_stem: u8) -> mingli_ganzhi::Element {
    if idx == 0 {
        mingli_ganzhi::stem_element(day_stem)
    } else {
        branch_element(c.down)
    }
}

/// 下贼上：下位五行克上神五行。
fn down_controls_up(idx: usize, c: &Course, day_stem: u8) -> bool {
    down_element(idx, c, day_stem).controls() == branch_element(c.up)
}
/// 上克下：上神五行克下位五行。
fn up_controls_down(idx: usize, c: &Course, day_stem: u8) -> bool {
    branch_element(c.up).controls() == down_element(idx, c, day_stem)
}

/// 由初传地支「层层取天盘上神」得三传。
fn transmit_from(first: u8, offset: u8) -> [u8; 3] {
    let mid = heaven_plate(first, offset);
    let last = heaven_plate(mid, offset);
    [first, mid, last]
}

/// 天干阴阳：甲丙戊庚壬（偶）为阳。地支阴阳：子寅辰午申戌（偶）为阳。
fn stem_is_yang(stem: u8) -> bool {
    stem.is_multiple_of(2)
}
fn branch_is_yang(branch: u8) -> bool {
    branch.is_multiple_of(2)
}

/// 判课式并取传。
fn derive_transmission(
    courses: &[Course; 4],
    day_stem: u8,
    day_branch: u8,
    offset: u8,
    school: SheHaiSchool,
) -> (Pattern, Option<[u8; 3]>) {
    // 伏吟 / 返吟：先判天地盘几何。
    if offset == 0 {
        // 伏吟：阳日（自任）初传=干上神，阴日（自信）初传=支上神。中末层层取上神。
        let first = if stem_is_yang(day_stem) {
            courses[0].up
        } else {
            courses[2].up
        };
        return (Pattern::FuYin, Some(transmit_from(first, offset)));
    }
    if offset == 6 {
        // 返吟：有克走贼克类；中末传层层取上神（offset=6 即取冲，天然由 heaven_plate 实现）。
        if let Some(first) =
            single_kede(courses, true, day_stem).or_else(|| single_kede(courses, false, day_stem))
        {
            return (Pattern::FanYin, Some(transmit_from(first, offset)));
        }
        return (Pattern::FanYin, None); // 无克的井栏射等，🟡 不强编
    }

    // 贼克法：下贼上优先于上克下。
    let zei = courses.iter().enumerate().any(|(i, c)| down_controls_up(i, c, day_stem));
    let ke = courses.iter().enumerate().any(|(i, c)| up_controls_down(i, c, day_stem));

    if zei {
        return resolve_kede(courses, true, Pattern::ZhongShen, day_stem, offset, school);
    }
    if ke {
        return resolve_kede(courses, false, Pattern::YuanShou, day_stem, offset, school);
    }

    let day_lodging = STEM_LODGING[day_stem as usize];
    // 八专先于遥克判：《六壬粹言》卷一驳《订讹》——「伏吟课既无遥克之例，
    // 而八专何独有取于遥克耶？且既取遥克，则古来当不设独足一课矣」。
    // 独足课（己未日酉加未，三传酉酉酉）在 720 课里有且仅有一课，正是不取遥克才存在。
    if day_lodging == day_branch {
        // 阳日自干上神连本位顺数三位，阴日自四课上神连本位逆数三位；中末皆取干上神。
        // 「连本位」四源明证：卷一夹注「连本位数」、《课经》算例「干上阳神亥，顺数至丑」、
        // 《指南》今注「以丑为一，顺数至三」、《粹言》「连根顺数三神」。
        let first = if stem_is_yang(day_stem) {
            (courses[0].up + 2) % 12
        } else {
            (courses[3].up + 10) % 12
        };
        return (Pattern::BaZhuan, Some([first, courses[0].up, courses[0].up]));
    }

    // 无上下克：遥克。
    let day_elem = mingli_ganzhi::stem_element(day_stem);
    // 蒿矢：天盘神克日干（寄宫）。
    if let Some(first) = courses
        .iter()
        .map(|c| c.up)
        .find(|&u| branch_element(u).controls() == day_elem)
    {
        return (Pattern::HaoShi, Some(transmit_from(first, offset)));
    }
    // 弹射：日干克天盘神。
    if let Some(first) = courses
        .iter()
        .map(|c| c.up)
        .find(|&u| day_elem.controls() == branch_element(u))
    {
        return (Pattern::TanShe, Some(transmit_from(first, offset)));
    }

    // 别责：四课不全（去重后 < 4 课）。
    let distinct: std::collections::HashSet<(u8, u8)> =
        courses.iter().map(|c| (c.down, c.up)).collect();
    if distinct.len() < 4 {
        // 阳日「干合上头神」：取日干之合干的寄宫，用该宫的上神；
        // 阴日「支前三合」：取日支三合局前一位（支 + 4）**本身**发用，不再取其上神。
        // 中末皆取干上神。柔日取本身还是取其上神，《订讹》曾存疑，
        // 《六壬粹言》卷一裁定「仍以古法为是」（取本身），《课经》《指南》算例亦然。
        let first = if stem_is_yang(day_stem) {
            let partner_lodging = STEM_LODGING[((day_stem + 5) % 10) as usize];
            heaven_plate(partner_lodging, offset)
        } else {
            (day_branch + 4) % 12
        };
        return (Pattern::BieZe, Some([first, courses[0].up, courses[0].up]));
    }

    // 昴星：余下。阳日仰视地盘酉位之上神，阴日俯视天盘酉所临之地盘支。
    // 中末：**阳日先支后干、阴日先干后支**——《课经》给了理据「刚日本乎天者亲上，末传归干；
    // 柔日本乎地者亲下，末传归辰」。五源无异议。
    let (first, mid, last) = if stem_is_yang(day_stem) {
        (heaven_plate(9, offset), courses[2].up, courses[0].up)
    } else {
        ((9 + 12 - offset) % 12, courses[0].up, courses[2].up)
    };
    (Pattern::MaoXing, Some([first, mid, last]))
}

/// 单一克对：返回唯一受克之上神；多于一则 None（交给比用/涉害）。
fn single_kede(courses: &[Course; 4], zei: bool, day_stem: u8) -> Option<u8> {
    let ups: Vec<u8> = courses
        .iter()
        .enumerate()
        .filter(|(i, c)| {
            if zei {
                down_controls_up(*i, c, day_stem)
            } else {
                up_controls_down(*i, c, day_stem)
            }
        })
        .map(|(_, c)| c.up)
        .collect();
    if ups.len() == 1 {
        Some(ups[0])
    } else {
        None
    }
}

/// 贼克 / 比用 / 涉害的取传。`zei` 为真时取下贼上诸课，否则取上克下诸课。
fn resolve_kede(
    courses: &[Course; 4],
    zei: bool,
    base: Pattern,
    day_stem: u8,
    offset: u8,
    school: SheHaiSchool,
) -> (Pattern, Option<[u8; 3]>) {
    let hits: Vec<Course> = courses
        .iter()
        .enumerate()
        .filter(|(i, c)| {
            if zei {
                down_controls_up(*i, c, day_stem)
            } else {
                up_controls_down(*i, c, day_stem)
            }
        })
        .map(|(_, c)| *c)
        .collect();
    if hits.len() == 1 {
        return (base, Some(transmit_from(hits[0].up, offset)));
    }
    // 比用：取与日干同阴阳之上神。
    let yang = stem_is_yang(day_stem);
    let bi: Vec<Course> = hits.iter().copied().filter(|c| branch_is_yang(c.up) == yang).collect();
    if bi.len() == 1 {
        return (Pattern::BiYong, Some(transmit_from(bi[0].up, offset)));
    }
    // 俱比取比者，俱不比取全部克课 —— 这一组进涉害。
    let mut pool = if bi.is_empty() { hits } else { bi };

    if school == SheHaiSchool::Classical {
        // 第一层：受克深者为用。
        let deepest = pool.iter().map(shehai_depth).max().unwrap_or(0);
        pool.retain(|c| shehai_depth(c) == deepest);
    }
    // 第二层：孟 ＞ 仲 ＞ 季，按天盘神**所临的地盘位**判。
    let best = pool.iter().map(|c| meng_zhong_ji(c.down)).min().unwrap_or(2);
    pool.retain(|c| meng_zhong_ji(c.down) == best);
    // 第三层（复等 / 缀瑕）：阳日取干上神，阴日取支上神。
    let first = if pool.len() == 1 {
        pool[0].up
    } else {
        let prefer = if yang { courses[0].up } else { courses[2].up };
        pool.iter().map(|c| c.up).find(|&u| u == prefer).unwrap_or(pool[0].up)
    };
    (Pattern::SheHai, Some(transmit_from(first, offset)))
}

/// 在共享上下文 [`Moment`] 上起大六壬课（涉害取古法）。
#[must_use]
pub fn compute_at(m: &Moment) -> Cast {
    compute_at_with(m, SheHaiSchool::Classical)
}

/// 在共享上下文上起课，指定涉害流派。
#[must_use]
pub fn compute_at_with(m: &Moment, school: SheHaiSchool) -> Cast {
    let day = mingli_ganzhi::day_ganzhi(m.civil_day);
    let hb = mingli_ganzhi::hour_branch(m.hour, m.minute);
    let mg = month_general_branch(m.sun_longitude);
    let offset = plate_offset(mg, hb);
    let mut heaven = [0u8; 12];
    for (g, h) in heaven.iter_mut().enumerate() {
        #[allow(clippy::cast_possible_truncation, reason = "g∈0..12")]
        let gg = g as u8;
        *h = heaven_plate(gg, offset);
    }
    let courses = four_courses(day.stem, day.branch, offset);
    let (pattern, transmission) =
        derive_transmission(&courses, day.stem, day.branch, offset, school);
    Cast {
        day_stem: day.stem,
        day_branch: day.branch,
        hour_branch: hb,
        month_general: mg,
        month_general_name: MONTH_GENERAL_NAMES[mg as usize],
        offset,
        heaven,
        courses,
        pattern,
        pattern_label: pattern.label(),
        transmission,
    }
}

/// 由本地民用时刻起课（独立入口，构造 [`Moment`]）。
#[must_use]
pub fn compute(year: i32, month: u32, day: u32, hour: u32, minute: u32, tz: f64) -> Cast {
    compute_at(&Moment::new(year, month, day, hour, minute, tz))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn worked_example_hai_general_zi_hour_jiazi_day() {
        // 古法工作例：亥将(11)子时(0)甲子日（干甲=0，支子=0）。
        // offset = (11-0) mod 12 = 11。四课上神应为 丑(1)/子(0)/亥(11)/戌(10)。
        let offset = plate_offset(11, 0);
        assert_eq!(offset, 11);
        let courses = four_courses(0, 0, offset);
        // 甲寄寅(2)，寅上见丑(1)→一课。
        assert_eq!(courses[0], Course { down: 2, up: 1 });
        // 丑(1)上见子(0)→二课。
        assert_eq!(courses[1], Course { down: 1, up: 0 });
        // 日支子(0)上见亥(11)→三课。
        assert_eq!(courses[2], Course { down: 0, up: 11 });
        // 亥(11)上见戌(10)→四课。
        assert_eq!(courses[3], Course { down: 11, up: 10 });
    }

    #[test]
    fn month_general_from_sun_longitude() {
        // λ 刚过雨水(330)→亥(11)=登明；λ∈[0，30)→戌(10)；λ∈[300，330)→子(0)。
        assert_eq!(month_general_branch(331.0), 11);
        assert_eq!(MONTH_GENERAL_NAMES[month_general_branch(331.0) as usize], "登明");
        assert_eq!(month_general_branch(15.0), 10); // 河魁
        assert_eq!(month_general_branch(310.0), 0); // 神后
        // 全 360° 扫描：月将恒在 0..12。
        let mut x = 0.0;
        while x < 360.0 {
            assert!(month_general_branch(x) < 12);
            x += 5.0;
        }
    }

    #[test]
    fn lodging_table_no_four_cardinals() {
        // 四正（子0午6卯3酉9）不作寄宫。
        for &b in &STEM_LODGING {
            assert!(![0u8, 3, 6, 9].contains(&b), "寄宫不应落四正： {b}");
        }
        // 丙戊同寄巳(5)、丁己同寄未(7)。
        assert_eq!(STEM_LODGING[2], STEM_LODGING[4]); // 丙=戊=巳
        assert_eq!(STEM_LODGING[3], STEM_LODGING[5]); // 丁=己=未
    }

    #[test]
    fn heaven_plate_is_z12_rotation() {
        // 天盘是地盘的纯平移：12 宫各异、双射。
        for offset in 0..12u8 {
            let set: std::collections::HashSet<u8> = (0..12).map(|g| heaven_plate(g, offset)).collect();
            assert_eq!(set.len(), 12);
        }
    }

    #[test]
    fn transmission_valid_when_present() {
        // 扫描多日多时辰：凡给出三传者，三传皆合法地支、且中末传由层层取上神自洽。
        for day in 1..=28u32 {
            for hour in [0u32, 6, 12, 18] {
                let c = compute(2024, 3, day, hour, 30, 8.0);
                // 课式总有；课式名稳定。
                let _ = c.pattern;
                if let Some(t) = c.transmission {
                    assert!(t.iter().all(|&b| b < 12));
                    // 层层取上神的那几门：中传 = 初传上神、末传 = 中传上神。
                    // 昴星 / 别责 / 八专不走这条——它们的中末取干上神与支上神。
                    if !matches!(c.pattern, Pattern::MaoXing | Pattern::BieZe | Pattern::BaZhuan) {
                        assert_eq!(t[1], heaven_plate(t[0], c.offset));
                        assert_eq!(t[2], heaven_plate(t[1], c.offset));
                    }
                }
                // 天盘双射。
                let set: std::collections::HashSet<u8> = c.heaven.iter().copied().collect();
                assert_eq!(set.len(), 12);
            }
        }
    }

    #[test]
    fn fuyin_when_general_equals_hour() {
        // 月将==时 → offset 0 → 伏吟，且给出三传。
        let mg = month_general_branch(331.0); // 11
        // 选 hour==mg 使 offset==0。
        let c = compute_via(0, 0, mg, mg);
        assert_eq!(c.offset, 0);
        assert_eq!(c.pattern, Pattern::FuYin);
        assert!(c.transmission.is_some());
    }

    #[test]
    fn classification_covers_kede_and_yaoke() {
        // 构造覆盖：贼克类与遥克类都应在某些时辰出现。
        use std::collections::HashSet;
        let mut seen = HashSet::new();
        for day in 1..=60u32 {
            for hour in 0..24u32 {
                let c = compute(2024, 1, 1 + (day % 28), hour, 0, 8.0);
                seen.insert(c.pattern);
            }
        }
        // 至少应见到重审/元首（贼克）与一种遥克或特殊式。
        assert!(seen.contains(&Pattern::ZhongShen) || seen.contains(&Pattern::YuanShou));
        assert!(seen.len() >= 2);
    }

    /// 测试辅助：直接给 （日干，日支，月将，时支） 起课。
    pub(super) fn compute_via(stem: u8, branch: u8, mg: u8, hb: u8) -> Cast {
        compute_via_with(stem, branch, mg, hb, SheHaiSchool::Classical)
    }

    pub(super) fn compute_via_with(stem: u8, branch: u8, mg: u8, hb: u8, school: SheHaiSchool) -> Cast {
        let offset = plate_offset(mg, hb);
        let courses = four_courses(stem, branch, offset);
        let (pattern, transmission) = derive_transmission(&courses, stem, branch, offset, school);
        let mut heaven = [0u8; 12];
        for (g, h) in heaven.iter_mut().enumerate() {
            *h = heaven_plate(g as u8, offset);
        }
        Cast {
            day_stem: stem,
            day_branch: branch,
            hour_branch: hb,
            month_general: mg,
            month_general_name: MONTH_GENERAL_NAMES[mg as usize],
            offset,
            heaven,
            courses,
            pattern,
            pattern_label: pattern.label(),
            transmission,
        }
    }

    #[test]
    fn fanyin_when_opposite() {
        // offset==6（天地相冲）→ 返吟。
        let c = compute_via(0, 0, 6, 0); // mg=6,hb=0 → offset 6
        assert_eq!(c.offset, 6);
        assert_eq!(c.pattern, Pattern::FanYin);
    }

    #[test]
    fn special_patterns_have_none_transmission() {
        // 九宗门里只剩返吟无克一路（井栏射等）不强编三传，其余八门皆已取传。
        use std::collections::HashSet;
        let mut seen = HashSet::new();
        for stem in 0..10u8 {
            for branch in 0..12u8 {
                if stem % 2 != branch % 2 {
                    continue; // 干支阴阳须同，六十甲子
                }
                for mg in 0..12u8 {
                    for hb in 0..12u8 {
                        let c = compute_via(stem, branch, mg, hb);
                        seen.insert(c.pattern);
                        if c.transmission.is_none() {
                            assert_eq!(c.pattern, Pattern::FanYin, "只有返吟无克一路留空");
                        }
                    }
                }
            }
        }
        for p in [Pattern::SheHai, Pattern::MaoXing, Pattern::BieZe, Pattern::BaZhuan] {
            assert!(seen.contains(&p), "{} 应在全枚举里出现", p.label());
        }
    }

    #[test]
    fn full_scan_reaches_every_pattern_branch() {
        // 穷举 （日干×日支×月将×时支）=10×12×12×12=17280 组合，确保每条判定分支都被走到，
        // 并校验：贼克/比用/遥克/伏吟/返吟（有克）给三传；涉害/昴星/别责/八专给 None。
        use std::collections::HashSet;
        let mut patterns = HashSet::new();
        let mut fanyin_some = false;
        let mut fanyin_none = false;
        for stem in 0..10u8 {
            for branch in 0..12u8 {
                for mg in 0..12u8 {
                    for hb in 0..12u8 {
                        let c = compute_via(stem, branch, mg, hb);
                        patterns.insert(c.pattern);
                        match c.pattern {
                            Pattern::FanYin if c.transmission.is_some() => fanyin_some = true,
                            Pattern::FanYin => fanyin_none = true,
                            _ => {}
                        }
                    }
                }
            }
        }
        // 全部 11 种课式都可达（判定树无死分支）。
        for p in [
            Pattern::ZhongShen,
            Pattern::YuanShou,
            Pattern::BiYong,
            Pattern::SheHai,
            Pattern::HaoShi,
            Pattern::TanShe,
            Pattern::MaoXing,
            Pattern::BieZe,
            Pattern::BaZhuan,
            Pattern::FuYin,
            Pattern::FanYin,
        ] {
            assert!(patterns.contains(&p), "课式 {p:?} 不可达");
        }
        assert!(fanyin_some, "应存在有克返吟（给三传）");
        assert!(fanyin_none, "应存在无克返吟（不强编三传）");
    }

    #[test]
    fn deterministic() {
        let a = compute(2024, 6, 15, 14, 30, 8.0);
        let b = compute(2024, 6, 15, 14, 30, 8.0);
        assert_eq!(a.courses, b.courses);
        assert_eq!(a.pattern, b.pattern);
    }
}

#[cfg(test)]
mod course_census {
    use super::*;

    /// 枚举六十甲子 × 十二局，收集某课式的全部命中。
    fn census(want: Pattern) -> Vec<(u8, u8, u8, Cast)> {
        let mut out = Vec::new();
        for stem in 0..10u8 {
            for branch in 0..12u8 {
                if stem % 2 != branch % 2 {
                    continue;
                }
                for offset in 0..12u8 {
                    // 月将 = 时支 + offset，这里直接以时支 0 遍历 offset
                    let c = super::tests::compute_via(stem, branch, offset, 0);
                    if c.pattern == want {
                        out.push((stem, branch, offset, c));
                    }
                }
            }
        }
        out
    }

    /// 昴星恰 16 课，刚 4 柔 12 —— 两部独立的书各自自报过这两个数。
    ///
    /// 《六壬大全》卷一末〈补论〉「凡昴星止十六课」；
    /// 《六壬粹言》卷二「昴星仰视格……计四课」「昴星俯视格……计一十二课」。
    /// 课数对上，说明取传规则与作者脑中的规则是同一个——比任何单条口诀都硬。
    #[test]
    fn the_mao_xing_census_matches_what_two_books_each_reported() {
        let all = census(Pattern::MaoXing);
        let gang = all.iter().filter(|(s, ..)| s % 2 == 0).count();
        let rou = all.len() - gang;
        assert_eq!((all.len(), gang, rou), (16, 4, 12), "昴星应 16 课（刚 4 柔 12）");
    }

    /// 别责恰 9 课，刚 3 柔 6，且日辰清单与《六壬大全》卷一小注逐条对上。
    ///
    /// 小注原文：「戊辰、戊午、丙辰三刚日各一课，辛未二课，辛丑二课，丁酉、辛酉各一课」。
    #[test]
    fn the_bie_ze_census_matches_the_nine_days_listed_in_the_gloss() {
        let all = census(Pattern::BieZe);
        let gang = all.iter().filter(|(s, ..)| s % 2 == 0).count();
        assert_eq!((all.len(), gang, all.len() - gang), (9, 3, 6), "别责应 9 课（刚 3 柔 6）");
        // 干支组合逐一对表：丙辰、戊辰、戊午各一，辛未、辛丑各二，丁酉、辛酉各一
        let mut tally: std::collections::BTreeMap<(u8, u8), usize> = std::collections::BTreeMap::new();
        for (s, b, ..) in &all {
            *tally.entry((*s, *b)).or_default() += 1;
        }
        // (干, 支, 课数)：丙2辰4 · 戊4辰4 · 戊4午6 · 辛7未7 · 辛7丑1 · 丁3酉9 · 辛7酉9
        let want: Vec<((u8, u8), usize)> =
            vec![((2, 4), 1), ((3, 9), 1), ((4, 4), 1), ((4, 6), 1), ((7, 1), 2), ((7, 7), 2), ((7, 9), 1)];
        assert_eq!(tally.into_iter().collect::<Vec<_>>(), want, "别责的九课日辰应与小注一致");
    }

    /// 八专恰 16 课，刚 6 柔 10；癸丑日一课不入（四课皆有克）；独足课有且仅有一课。
    ///
    /// 《六壬粹言》卷二「顺数三神格……计六课」「逆数三神格……计十课」；
    /// 《课经》「八专日有五，除癸丑日俱有克」；
    /// 《御定六壬直指》「独脚课兮止一名」——三传三字全同者唯一。
    #[test]
    fn the_ba_zhuan_census_and_the_single_footed_course() {
        let all = census(Pattern::BaZhuan);
        let gang = all.iter().filter(|(s, ..)| s % 2 == 0).count();
        assert_eq!((all.len(), gang, all.len() - gang), (16, 6, 10), "八专应 16 课（刚 6 柔 10）");
        // 癸(9)丑(1) 一课不入
        assert!(!all.iter().any(|(s, b, ..)| (*s, *b) == (9, 1)), "癸丑日四课皆有克，不入八专");
        // 五个八专日里只有四日出现
        let days: std::collections::BTreeSet<(u8, u8)> = all.iter().map(|(s, b, ..)| (*s, *b)).collect();
        assert_eq!(days.into_iter().collect::<Vec<_>>(), vec![(0, 2), (3, 7), (5, 7), (6, 8)]);
        // 独足：三传三字全同，唯一
        let single_footed: Vec<_> = all
            .iter()
            .filter(|(.., c)| {
                c.transmission.is_some_and(|t| t[0] == t[1] && t[1] == t[2])
            })
            .collect();
        assert_eq!(single_footed.len(), 1, "独足课止一名");
        let (s, b, _, c) = single_footed[0];
        assert_eq!((*s, *b), (5, 7), "独足课是己未日");
        assert_eq!(c.transmission, Some([9, 9, 9]), "三传酉酉酉");
    }

    /// 涉害的「受克深浅」数法：六个古籍算例逐条复算。
    ///
    /// 两处边界由算例定死，都不计：起点（天盘神所临的地盘位）与终点（该神的本家）。
    /// 《观月经》甲辰日「子加辰……巳上戊土、未土、未上己土、前又戌土，共四重」——
    /// 起点辰本身是土（克子水）却不在账上；「未土 ＋ 未上己土」分两重记，故寄干单独计。
    /// 《课经》甲午日「辰加寅，历卯木一重」——终点辰的寄干乙木若计就是两重，作一重故本家不计。
    #[test]
    fn the_depth_count_reproduces_six_worked_examples() {
        // (下神, 上神, 古籍所记重数)
        const ORACLE: [(u8, u8, u32); 6] = [
            (2, 10, 2),  // 《观月经》甲辰日：戌加寅，历卯木、乙木二重
            (4, 0, 4),   // 同上：子加辰，历巳戊土、未土、未己土、戌土四重
            (3, 1, 1),   // 《课经》丁卯日：丑加卯，只历辰中乙木一重
            (1, 11, 5),  // 同上：亥加丑，历辰戊未己戌土五重
            (2, 4, 1),   // 《课经》甲午日缀瑕例：辰加寅，历卯木一重
            (6, 8, 1),   // 同上：申加午，历丁火一重
        ];
        for (down, up, want) in ORACLE {
            let got = shehai_depth(&Course { down, up });
            assert_eq!(got, want, "{down} 上 {up} 的重数应为 {want}，实得 {got}");
        }
    }

    /// 两派涉害并非同一件事：至少存在一课，古法与近法给出不同的初传。
    #[test]
    fn the_two_shehai_schools_are_not_the_same_rule() {
        let mut differ = 0;
        for stem in 0..10u8 {
            for branch in 0..12u8 {
                if stem % 2 != branch % 2 {
                    continue;
                }
                for offset in 0..12u8 {
                    let a = super::tests::compute_via_with(stem, branch, offset, 0, SheHaiSchool::Classical);
                    let b = super::tests::compute_via_with(stem, branch, offset, 0, SheHaiSchool::ByPosition);
                    if a.pattern == Pattern::SheHai && a.transmission != b.transmission {
                        differ += 1;
                    }
                }
            }
        }
        assert!(differ > 0, "两派若处处同解，就不该建成两个流派");
    }

    /// 流派 id 往返。
    #[test]
    fn shehai_school_id_roundtrip() {
        for s in [SheHaiSchool::Classical, SheHaiSchool::ByPosition] {
            assert_eq!(SheHaiSchool::from_id(s.id()), Some(s));
        }
        assert_eq!(SheHaiSchool::from_id("unknown"), None);
        assert_eq!(SheHaiSchool::default(), SheHaiSchool::Classical);
    }
}

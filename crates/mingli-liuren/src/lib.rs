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
//! 诚实边界（🟡）：三传的**贼克/比用/遥克/伏吟/返吟**取传规则明确、已实现并取传；而
//! **涉害（数克深浅的兜底）/昴星/别责/八专** 的取传细则随流派分歧、且本项目无权威三传校验工具，
//! 故只**判定课式**、不强编其三传（[`Cast::transmission`] 返回 `None`），把不确定显式暴露。

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

/// 下贼上：下神五行克上神五行。
fn down_controls_up(c: &Course) -> bool {
    branch_element(c.down).controls() == branch_element(c.up)
}
/// 上克下：上神五行克下神五行。
fn up_controls_down(c: &Course) -> bool {
    branch_element(c.up).controls() == branch_element(c.down)
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
        if let Some(first) = single_kede(courses, true).or_else(|| single_kede(courses, false)) {
            return (Pattern::FanYin, Some(transmit_from(first, offset)));
        }
        return (Pattern::FanYin, None); // 无克的井栏射等，🟡 不强编
    }

    // 贼克法：下贼上优先于上克下。
    let zei: Vec<u8> = courses
        .iter()
        .filter(|c| down_controls_up(c))
        .map(|c| c.up)
        .collect();
    let ke: Vec<u8> = courses
        .iter()
        .filter(|c| up_controls_down(c))
        .map(|c| c.up)
        .collect();

    if !zei.is_empty() {
        return resolve_kede(&zei, Pattern::ZhongShen, day_stem, offset);
    }
    if !ke.is_empty() {
        return resolve_kede(&ke, Pattern::YuanShou, day_stem, offset);
    }

    // 无上下克：遥克。
    let day_lodging = STEM_LODGING[day_stem as usize];
    let day_elem = branch_element(day_lodging);
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

    // 八专：日干支同位（寄宫==日支）。
    if day_lodging == day_branch {
        return (Pattern::BaZhuan, None); // 🟡 取传流派分歧
    }
    // 别责：四课不全（去重后 < 4 课）。
    let distinct: std::collections::HashSet<(u8, u8)> =
        courses.iter().map(|c| (c.down, c.up)).collect();
    if distinct.len() < 4 {
        return (Pattern::BieZe, None); // 🟡
    }
    // 昴星：余下。
    (Pattern::MaoXing, None) // 🟡
}

/// 单一克对：返回唯一受克之上神；多于一则 None（交给比用/涉害）。
fn single_kede(courses: &[Course; 4], zei: bool) -> Option<u8> {
    let ups: Vec<u8> = courses
        .iter()
        .filter(|c| if zei { down_controls_up(c) } else { up_controls_down(c) })
        .map(|c| c.up)
        .collect();
    if ups.len() == 1 {
        Some(ups[0])
    } else {
        None
    }
}

/// 贼克/比用/涉害的取传。`ups` 为受克之上神集合（非空）。
fn resolve_kede(ups: &[u8], base: Pattern, day_stem: u8, offset: u8) -> (Pattern, Option<[u8; 3]>) {
    if ups.len() == 1 {
        return (base, Some(transmit_from(ups[0], offset)));
    }
    // 比用：取与日干同阴阳之上神。
    let yang = stem_is_yang(day_stem);
    let bi: Vec<u8> = ups
        .iter()
        .copied()
        .filter(|&u| branch_is_yang(u) == yang)
        .collect();
    if bi.len() == 1 {
        return (Pattern::BiYong, Some(transmit_from(bi[0], offset)));
    }
    // 俱比 / 俱不比 → 涉害（🟡 兜底数克细则流派分歧，不强编取传）。
    (Pattern::SheHai, None)
}

/// 在共享上下文 [`Moment`] 上起大六壬课。
#[must_use]
pub fn compute_at(m: &Moment) -> Cast {
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
    let (pattern, transmission) = derive_transmission(&courses, day.stem, day.branch, offset);
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
                    // 中传 = 初传上神；末传 = 中传上神。
                    assert_eq!(t[1], heaven_plate(t[0], c.offset));
                    assert_eq!(t[2], heaven_plate(t[1], c.offset));
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
    fn compute_via(stem: u8, branch: u8, mg: u8, hb: u8) -> Cast {
        let offset = plate_offset(mg, hb);
        let courses = four_courses(stem, branch, offset);
        let (pattern, transmission) = derive_transmission(&courses, stem, branch, offset);
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
        // 涉害/昴星/别责/八专：诚实地不强编三传。逐一构造或扫描命中后断言 None。
        use std::collections::HashSet;
        let mut none_patterns = HashSet::new();
        for stem in 0..10u8 {
            for branch in 0..12u8 {
                for mg in 0..12u8 {
                    for hb in 0..12u8 {
                        let c = compute_via(stem, branch, mg, hb);
                        if matches!(
                            c.pattern,
                            Pattern::SheHai | Pattern::MaoXing | Pattern::BieZe | Pattern::BaZhuan
                        ) {
                            assert!(c.transmission.is_none(), "流派分歧课式应不强编三传");
                            none_patterns.insert(c.pattern);
                        }
                    }
                }
            }
        }
        // 全 17280 组合里这些课式确实出现（覆盖判定分支）。
        assert!(!none_patterns.is_empty());
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

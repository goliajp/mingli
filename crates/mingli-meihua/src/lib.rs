//! L3 叶（C 族中的确定性子类）：梅花易数·多起卦法。
//!
//! 梅花有多种起卦法（《梅花易数》卷一明列），本叶实现两类**纯模运算**法（可代码化、可校验）：
//!
//! - [`Method::Time`]（默认·确定性，归 A 族风格）：邵雍古法，
//!   - 上卦 = （年支数 + 月 + 日） mod 8（余 0 取 8）→ 先天八卦数 → 卦；
//!   - 下卦 = （年支数 + 月 + 日 + 时辰数） mod 8（余 0 取 8）；
//!   - 动爻 = （年支数 + 月 + 日 + 时辰数） mod 6（余 0 取 6）。
//!   - 年支数 = 农历年地支（子1…亥12）；月/日 = 农历月、日数；时辰数 = 子1…亥12。
//! - [`Method::Numbers`]（随机·种子起卦，归 C 族风格）：报数法（"先报一数为上卦，后报一数为下卦"）。
//!   两数 (a， b) 由种子拆解派生（高低 32 位）以与既有 C 族叶统一，
//!   - 上卦 = a mod 8；下卦 = b mod 8；动爻 = （a + b + 时辰数） mod 6。
//!
//! 由本卦定**互卦**（中爻卦）与**之卦**（动爻变），六爻代数全部复用主干 [`mingli_gua`]。
//!
//! 语域注：六十四卦名属🟡查表，与 [`mingli_gua`] 一致不在此硬编；卦以上/下卦名复合标识。

#![allow(

    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "梅花起卦全是模运算：年支/月/日/时辰均落在 1..=31 的小范围，窄化到 u8 受控安全"
)]

mod engine;
pub use engine::MeihuaEngine;

use mingli_astro::Moment;
use mingli_gua::{Hexagram, Trigram, TRIGRAM_XIANTIAN};
use serde::Serialize;

/// 起卦法（流派）。默认 [`Method::Time`]。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Method {
    /// 时间起卦法（邵雍古法）：以农历年支/月/日/时辰模运算定卦。确定性，不依赖种子。
    #[default]
    Time,
    /// 数字（报数）起卦法：以两个数 + 时辰模运算定卦。两数由种子拆解派生（高/低 32 位）。
    Numbers,
}

impl Method {
    /// 流派稳定 id（小写英数，用于 schools dropdown）。
    #[must_use]
    pub fn id(self) -> &'static str {
        match self {
            Self::Time => "time",
            Self::Numbers => "numbers",
        }
    }

    /// 从稳定 id 解析；未知 id 返回 `None`。
    #[must_use]
    pub fn from_id(s: &str) -> Option<Self> {
        match s {
            "time" => Some(Self::Time),
            "numbers" => Some(Self::Numbers),
            _ => None,
        }
    }
}

/// 一次梅花起卦的结果。
///
/// 时间法填 [`year_branch`](Self::year_branch)/[`month`](Self::month)/[`day`](Self::day)；
/// 数字法填 [`numbers`](Self::numbers)；[`hour_branch`](Self::hour_branch) 两法都填。
#[derive(Debug, Clone, Serialize)]
pub struct Cast {
    /// 起卦法稳定 id（"time" / "numbers"）。
    pub method_id: &'static str,
    /// 年支数（子1…亥12，时间法填，数字法 None）。
    pub year_branch: Option<u8>,
    /// 农历月数（时间法填，数字法 None）。
    pub month: Option<u8>,
    /// 农历日数（时间法填，数字法 None）。
    pub day: Option<u8>,
    /// 时辰数（子1…亥12，两法都填）。
    pub hour_branch: u8,
    /// 数字法两数（首=上卦，次=下卦，数字法填，时间法 None）。
    pub numbers: Option<(u32, u32)>,
    /// 本卦。
    pub primary: Hexagram,
    /// 本卦上/下卦名。
    pub primary_upper: &'static str,
    /// 见 [`Cast::primary_upper`]。
    pub primary_lower: &'static str,
    /// 动爻位（1=初爻…6=上爻）。
    pub moving_line: u8,
    /// 互卦（中爻卦）。
    pub mutual: Hexagram,
    /// 之卦（动爻变本卦）。
    pub changed: Hexagram,
    /// 之卦上/下卦名。
    pub changed_upper: &'static str,
    /// 见 [`Cast::changed_upper`]。
    pub changed_lower: &'static str,
    /// 本卦简称（乾/坤/.. — 三源校验，见 `mingli_gua::HEXAGRAM_NAMES`）。
    pub primary_name: &'static str,
    /// 本卦传统全名（乾为天/水雷屯/..）。
    pub primary_full_name: &'static str,
    /// 本卦文王卦序 1..=64。
    pub primary_king_wen: u8,
    /// 互卦简称。
    pub mutual_name: &'static str,
    /// 互卦传统全名。
    pub mutual_full_name: &'static str,
    /// 互卦文王卦序 1..=64。
    pub mutual_king_wen: u8,
    /// 之卦简称。
    pub changed_name: &'static str,
    /// 之卦传统全名。
    pub changed_full_name: &'static str,
    /// 之卦文王卦序 1..=64。
    pub changed_king_wen: u8,
}

/// 先天八卦数（1..=8）→ 八卦值（0..8），由 [`TRIGRAM_XIANTIAN`] 反查。
fn trigram_by_xiantian(n: u8) -> Trigram {
    let v = (0u8..8)
        .find(|&v| TRIGRAM_XIANTIAN[v as usize] == n)
        .unwrap_or(0);
    Trigram(v)
}

/// 把模 8 余数（0 视作 8）映射到八卦（先天数 1..=8）。
fn trigram_mod8(sum: u32) -> Trigram {
    let r = sum % 8;
    trigram_by_xiantian(if r == 0 { 8 } else { r as u8 })
}

/// 把模 6 余数（0 视作 6）映射到动爻位（1..=6）。
fn moving_line_mod6(sum: u32) -> u8 {
    let r = sum % 6;
    if r == 0 { 6 } else { r as u8 }
}

/// 小时（0..24）→ 时辰数（子1…亥12，每时辰二小时，子时含 23 与 0 时）。
#[must_use]
pub fn hour_to_branch(hour: u32) -> u8 {
    (hour.div_ceil(2) % 12) as u8 + 1
}

/// 农历年 → 年支数（子1…亥12）。
#[must_use]
pub fn year_to_branch(lunar_year: i32) -> u8 {
    ((lunar_year - 4).rem_euclid(12)) as u8 + 1
}

/// [`Cast`] 的输入元数据部分（method + 时间法字段 + 数字法字段）。
#[derive(Clone, Copy)]
struct CastHead {
    method_id: &'static str,
    year_branch: Option<u8>,
    month: Option<u8>,
    day: Option<u8>,
    hour_branch: u8,
    numbers: Option<(u32, u32)>,
}

/// 由本卦 + 动爻位 + 输入元数据组装 [`Cast`]（互卦/之卦/卦名公用尾段）。
fn finalize(head: CastHead, primary: Hexagram, moving_line: u8) -> Cast {
    let mask = 1u8 << (moving_line - 1);
    let changed = primary.changed(mask);
    let mutual = primary.mutual();
    Cast {
        method_id: head.method_id,
        year_branch: head.year_branch,
        month: head.month,
        day: head.day,
        hour_branch: head.hour_branch,
        numbers: head.numbers,
        primary,
        primary_upper: primary.upper().name(),
        primary_lower: primary.lower().name(),
        moving_line,
        mutual,
        changed,
        changed_upper: changed.upper().name(),
        changed_lower: changed.lower().name(),
        primary_name: primary.name(),
        primary_full_name: primary.full_name(),
        primary_king_wen: primary.king_wen(),
        mutual_name: mutual.name(),
        mutual_full_name: mutual.full_name(),
        mutual_king_wen: mutual.king_wen(),
        changed_name: changed.name(),
        changed_full_name: changed.full_name(),
        changed_king_wen: changed.king_wen(),
    }
}

/// 时间法（默认）：在共享上下文 [`Moment`] 上做梅花时间起卦（确定性）。
#[must_use]
pub fn compute_at(m: &Moment) -> Cast {
    let yb = year_to_branch(m.lunar.year);
    let month = m.lunar.month;
    let day = m.lunar.day;
    let hb = hour_to_branch(m.hour);

    let base = u32::from(yb) + month + day;
    let upper = trigram_mod8(base);
    let with_hour = base + u32::from(hb);
    let lower = trigram_mod8(with_hour);
    let primary = Hexagram::from_trigrams(upper, lower);

    let moving_line = moving_line_mod6(with_hour);

    finalize(
        CastHead {
            method_id: Method::Time.id(),
            year_branch: Some(yb),
            month: Some(month as u8),
            day: Some(day as u8),
            hour_branch: hb,
            numbers: None,
        },
        primary,
        moving_line,
    )
}

/// 数字法：以两数 `(a, b)` + 时辰起卦。
///
/// 上卦 = a mod 8；下卦 = b mod 8；动爻 = （a + b + 时辰数） mod 6。
#[must_use]
pub fn compute_numbers(a: u32, b: u32, m: &Moment) -> Cast {
    let hb = hour_to_branch(m.hour);
    let upper = trigram_mod8(a);
    let lower = trigram_mod8(b);
    let primary = Hexagram::from_trigrams(upper, lower);
    // 用 wrapping_add 防 u32 溢出（a/b 可能为 u32::MAX 量级）；模 6 不受影响。
    let sum = a.wrapping_add(b).wrapping_add(u32::from(hb));
    let moving_line = moving_line_mod6(sum);
    finalize(
        CastHead {
            method_id: Method::Numbers.id(),
            year_branch: None,
            month: None,
            day: None,
            hour_branch: hb,
            numbers: Some((a, b)),
        },
        primary,
        moving_line,
    )
}

/// 数字法（种子驱动）：两数从种子高/低 32 位拆解。与 C 族其余叶的 `effective_seed` 统一接口。
#[must_use]
pub fn compute_seeded(seed: u64, m: &Moment) -> Cast {
    let a = (seed >> 32) as u32;
    let b = (seed & 0xFFFF_FFFF) as u32;
    compute_numbers(a, b, m)
}

/// 在共享上下文 [`Moment`] 上按 `method` 起卦（schools 分发用）。
#[must_use]
pub fn compute_at_with(m: &Moment, method: Method, seed: u64) -> Cast {
    match method {
        Method::Time => compute_at(m),
        Method::Numbers => compute_seeded(seed, m),
    }
}

/// 由本地民用时刻起卦（时间法，独立入口，构造 [`Moment`]）。
#[must_use]
pub fn compute(year: i32, month: u32, day: u32, hour: u32, minute: u32, tz: f64) -> Cast {
    compute_at(&Moment::new(year, month, day, hour, minute, tz))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hour_branch_table() {
        // 子时含 23 与 0；丑=1/2；午=11/12。
        assert_eq!(hour_to_branch(23), 1);
        assert_eq!(hour_to_branch(0), 1);
        assert_eq!(hour_to_branch(1), 2);
        assert_eq!(hour_to_branch(2), 2);
        assert_eq!(hour_to_branch(11), 7); // 午
        assert_eq!(hour_to_branch(12), 7);
        // 12 个时辰全覆盖、皆在 1..=12。
        let set: std::collections::HashSet<u8> = (0..24).map(hour_to_branch).collect();
        assert_eq!(set.len(), 12);
        assert!(set.iter().all(|&b| (1..=12).contains(&b)));
    }

    #[test]
    fn year_branch_table() {
        assert_eq!(year_to_branch(2024), 5); // 甲辰，辰=5
        assert_eq!(year_to_branch(2020), 1); // 庚子，子=1
        assert_eq!(year_to_branch(2008), 1); // 戊子
        assert_eq!(year_to_branch(2019), 12); // 己亥，亥=12
    }

    #[test]
    fn xiantian_inverse_is_consistent() {
        // 先天数 1..8 ↔ 八卦值 双向自洽。
        for v in 0u8..8 {
            let n = TRIGRAM_XIANTIAN[v as usize];
            assert_eq!(trigram_by_xiantian(n).0, v);
        }
        assert_eq!(trigram_by_xiantian(1).name(), "乾"); // 先天乾1
        assert_eq!(trigram_by_xiantian(8).name(), "坤"); // 先天坤8
    }

    #[test]
    fn deterministic_and_structurally_sound() {
        let c = compute(2024, 6, 15, 14, 30, 8.0);
        // 确定性：再算一次相同。
        let c2 = compute(2024, 6, 15, 14, 30, 8.0);
        assert_eq!(c.primary, c2.primary);
        assert_eq!(c.changed, c2.changed);
        assert_eq!(c.method_id, "time");
        assert!(c.numbers.is_none());
        // 动爻在 1..=6；之卦 = 本卦仅在动爻位翻转。
        assert!((1..=6).contains(&c.moving_line));
        let bit = 1u8 << (c.moving_line - 1);
        assert_eq!(c.changed.0, c.primary.0 ^ bit);
        // 互卦 = gua 的中爻卦。
        assert_eq!(c.mutual, c.primary.mutual());
        // 卦名取自 gua 已核实 8 名。
        assert!(mingli_gua::TRIGRAM_NAMES.contains(&c.primary_upper));
    }

    /// 自洽检查：`compute_at` 与它自己的模运算一致。
    ///
    /// **这条不是 oracle**——两边用的是同一组函数，公式整体写错它照样绿。
    /// 它只保证「取出来的农历量确实是喂进公式的那几个」，真正的外部校验在
    /// [`guanmei_the_worked_example_from_the_source_text`]。
    #[test]
    fn worked_example_matches_formula() {
        let m = Moment::new(2024, 6, 15, 14, 30, 8.0);
        let c = compute_at(&m);
        let base = u32::from(c.year_branch.unwrap())
            + u32::from(c.month.unwrap())
            + u32::from(c.day.unwrap());
        let upper = super::trigram_mod8(base);
        let lower = super::trigram_mod8(base + u32::from(c.hour_branch));
        assert_eq!(c.primary, Hexagram::from_trigrams(upper, lower));
        assert_eq!(c.year_branch, Some(5));
        assert_eq!(c.hour_branch, hour_to_branch(14)); // 未时
    }

    /// 观梅占——《梅花易数》卷一自带的那个例子，逐步对上。
    ///
    /// 辰年十二月十七日申时：
    /// 上卦 5 + 12 + 17 = 34，34 mod 8 = 2 = 兑；
    /// 下卦 34 + 9 = 43，43 mod 8 = 3 = 离，合为**泽火革**；
    /// 动爻 43 mod 6 = 1，初爻动，之卦**泽山咸**。
    ///
    /// 年支数子 1 丑 2 …… 辰 5，时辰数同法申 9。取这个例子是因为它出自原书本身，
    /// 且四个中间量（34 / 2 / 43 / 3 / 1）都写在书里 —— 公式哪一步写反了都会在这里现形，
    /// 而上面那条自洽检查不会。
    ///
    /// 来源：例题与「以年月日数之和除以八，余数为上卦；年月日时数之和除以八，余数为下卦；
    /// 年月日时数之和除以六，余数取动爻」的表述在多处易学资料中一致复述（新浪 blog_4a66051f0101gwa2、
    /// 知乎 p/630773131 等均给出同一组数）。
    #[test]
    fn guanmei_the_worked_example_from_the_source_text() {
        let (year_branch, month, day, hour_branch) = (5u32, 12u32, 17u32, 9u32);
        let base = year_branch + month + day;
        assert_eq!(base, 34, "上卦之和");
        assert_eq!(base % 8, 2, "34 mod 8 = 2 = 兑");
        let with_hour = base + hour_branch;
        assert_eq!(with_hour, 43, "下卦之和");
        assert_eq!(with_hour % 8, 3, "43 mod 8 = 3 = 离");
        assert_eq!(super::moving_line_mod6(with_hour), 1, "43 mod 6 = 1，初爻动");

        let upper = super::trigram_mod8(base);
        let lower = super::trigram_mod8(with_hour);
        let primary = Hexagram::from_trigrams(upper, lower);
        assert_eq!(primary.full_name(), "泽火革", "上兑下离即泽火革");

        let changed = primary.changed(1 << (1 - 1)); // 初爻
        assert_eq!(changed.full_name(), "泽山咸", "初爻动，革之咸");
    }

    #[test]
    fn mod8_zero_maps_to_kun() {
        // 余 0 取 8 = 先天坤。
        assert_eq!(super::trigram_mod8(8).name(), "坤");
        assert_eq!(super::trigram_mod8(16).name(), "坤");
        assert_eq!(super::trigram_mod8(1).name(), "乾"); // 先天乾1
    }

    #[test]
    fn numbers_method_formula_5_7_at_chen_hour() {
        // 数字法 oracle：a=5， b=7， 辰时（7-9 时，时辰数=5）。
        // 上卦 = 5 mod 8 = 5 → 先天 5 = 巽；下卦 = 7 mod 8 = 7 → 先天 7 = 艮；
        // 动爻 = (5+7+5) mod 6 = 17 mod 6 = 5。
        let m = Moment::new(2024, 6, 15, 7, 30, 8.0);
        assert_eq!(hour_to_branch(7), 5); // 辰时（子1丑2寅3卯4辰5）
        let c = compute_numbers(5, 7, &m);
        assert_eq!(c.method_id, "numbers");
        assert_eq!(c.numbers, Some((5, 7)));
        assert_eq!(c.year_branch, None);
        assert_eq!(c.primary_upper, "巽"); // 先天 5
        assert_eq!(c.primary_lower, "艮"); // 先天 7
        assert_eq!(c.moving_line, 5);
        // 之卦 = 本卦五爻翻转。
        let mask = 1u8 << 4;
        assert_eq!(c.changed.0, c.primary.0 ^ mask);
        // 互卦 = gua 中爻卦。
        assert_eq!(c.mutual, c.primary.mutual());
    }

    #[test]
    fn numbers_method_zero_residue_maps_to_kun() {
        // a=8， b=16， 子时（时辰数=1）。上=坤(8 mod 8=0→8)、下=坤(16 mod 8=0→8)、
        // 动爻 = (8+16+1) mod 6 = 25 mod 6 = 1。
        let m = Moment::new(2024, 6, 15, 0, 30, 8.0);
        let c = compute_numbers(8, 16, &m);
        assert_eq!(c.primary_upper, "坤");
        assert_eq!(c.primary_lower, "坤");
        assert_eq!(c.moving_line, 1);
    }

    #[test]
    fn seeded_dispatch_is_reproducible_and_distinct_from_time() {
        let m = Moment::new(2024, 6, 15, 14, 30, 8.0);
        let seed: u64 = 0xDEAD_BEEF_CAFE_BABE;
        let a = compute_seeded(seed, &m);
        let b = compute_seeded(seed, &m);
        assert_eq!(a.primary, b.primary); // 同种子可复现
        assert_eq!(a.method_id, "numbers");
        assert_eq!(a.numbers, Some((0xDEAD_BEEF, 0xCAFE_BABE)));
        // compute_at_with 路由
        let via_with = compute_at_with(&m, Method::Numbers, seed);
        assert_eq!(via_with.primary, a.primary);
        let via_time = compute_at_with(&m, Method::Time, seed);
        assert_eq!(via_time.method_id, "time");
    }

    #[test]
    fn moving_line_helper_covers_both_branches() {
        // mod 6 = 0 → 6（上爻）；其余 1..=5 → 自身。
        assert_eq!(super::moving_line_mod6(0), 6);
        assert_eq!(super::moving_line_mod6(6), 6);
        assert_eq!(super::moving_line_mod6(12), 6);
        for r in 1u32..=5 {
            assert_eq!(super::moving_line_mod6(r), r as u8);
        }
    }

    #[test]
    fn method_id_roundtrip() {
        for m in [Method::Time, Method::Numbers] {
            assert_eq!(Method::from_id(m.id()), Some(m));
        }
        assert_eq!(Method::from_id("unknown"), None);
        assert_eq!(Method::default(), Method::Time);
    }
}

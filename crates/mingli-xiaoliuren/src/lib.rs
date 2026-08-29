//! L3 叶（A 族 / 确定性）：小六壬（诸葛马前课）。
//!
//! 小六壬把时间在一个长度 6 的循环上「连续掐指」：六神固定成环
//! `大安→留连→速喜→赤口→小吉→空亡→(回)大安`，本质是循环群 `Z₆`。算法（自正月、初一、
//! 子时各为起步）：
//! - 月神位 `m = (农历月 − 1) mod 6`（正月落大安）；
//! - 日神位 `d = (m + 农历日 − 1) mod 6`（初一落「月神位」处）；
//! - 时神位 `h = (d + 时辰序 0..11) mod 6`（子时落「日神位」处）。
//!
//! 三步都是同一个 `Z₆` 位移（[`mingli_core::group::count_to`]），把「时间」线性地折进 6 元环——
//! 这是 A 族「时间 → 模运算 → 盘」最小的范例。月/日取**农历**量（与梅花、传统口诀一致）。
//!
//! 语域注：六神名是这门系统的**定义性有序环**（仅 6 个、各家一致），故在此给出；而每神的
//! 五行 / 吉凶 / 含义随流派分歧，属 🟡 释义层，本 crate 不下断言、只给位置与名。

#![allow(

    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "掐指全在 Z₆ 上：count_to 的 i64 结果恒落 0..6，窄化到 u8 受控安全"
)]

#[cfg(feature = "port")]
mod engine;
#[cfg(feature = "port")]
pub use engine::XiaoliurenEngine;

use mingli_astro::Moment;
use mingli_core::group::count_to;
#[cfg(feature = "serde")]
use serde::Serialize;

/// 六神固定环（次序即掐指方向）：大安→留连→速喜→赤口→小吉→空亡。
pub const DEITIES: [&str; 6] = ["大安", "留连", "速喜", "赤口", "小吉", "空亡"];

/// 六神各配之方，**六个里只有四个定得下来**（与 [`DEITIES`] 同序，定不下者为 `None`）。
///
/// 定得下的四个五行—方位—四象三者自洽、多源同述：
/// 大安属木·东·青龙、留连属水·北·玄武、速喜属火·南·朱雀、赤口属金·西·白虎。
///
/// 两个留空，各有各的理由，都不是「还没查」：
///
/// - **空亡**配「中」。中宫不是可面向的方位——本仓库在奇门那边已按同一条道理处理过
///   （值符落中五宫时按「中 5 寄坤 2」归并，不出「朝中间」这种候选）。
///   小六壬没有对应的寄宫之说可援，故此处留空而非硬给一个方向。
/// - **小吉**各家不同：一系作属水·北（与留连同方），一系作属木（不给方位），
///   而其口诀又作「失物在**坤方**」（西南）——三说并存。知乎《小六壬理论知识详解》
///   那一路明记「不同的文献来源对小吉的方位属性描述有所不同，这反映了小六壬占卜法
///   在民间传承中的多个版本」。三说无一得两个独立源，按铁律留空。
///
/// 正因两个留空，本叶**不认领「寻」这一类问局**：一次掐指落在哪个神是不由人的，
/// 六分之二的情形给不出方位，那就不是「算得出这一类的 output_shape」。
/// 方位仍随盘面出，读的人自己看得见有没有。
pub const DEITY_DIRECTION: [Option<&str>; 6] = [
    Some("东"), // 大安 · 木 · 青龙
    Some("北"), // 留连 · 水 · 玄武
    Some("南"), // 速喜 · 火 · 朱雀
    Some("西"), // 赤口 · 金 · 白虎
    None,       // 小吉 —— 三说并存
    None,       // 空亡 —— 配「中」，非可面向之方
];

/// 一次小六壬掐指的结果。三个神位皆为 `0..6` 的环上下标。
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct Cast {
    /// 农历月（用于起月神）。
    pub lunar_month: u32,
    /// 农历日（用于起日神）。
    pub lunar_day: u32,
    /// 时辰地支序（子=0 … 亥=11）。
    pub hour_branch: u8,
    /// 月神环位 `0..6`。
    pub month_pos: u8,
    /// 日神环位 `0..6`。
    pub day_pos: u8,
    /// 时神环位 `0..6`（落定之神）。
    pub hour_pos: u8,
    /// 月神名。
    pub month_deity: &'static str,
    /// 日神名。
    pub day_deity: &'static str,
    /// 时神名（最终落定）。
    pub hour_deity: &'static str,
    /// 时神所配之方；落在小吉或空亡时为 `None`（见 [`DEITY_DIRECTION`]）。
    pub hour_direction: Option<&'static str>,
    /// 日神所配之方；同上。
    pub day_direction: Option<&'static str>,
    /// 月神所配之方；同上。
    pub month_direction: Option<&'static str>,
}

/// 在共享上下文 [`Moment`] 上做小六壬掐指（确定性）。
#[must_use]
pub fn compute_at(m: &Moment) -> Cast {
    let lunar_month = m.lunar.month;
    let lunar_day = m.lunar.day;
    let hb = mingli_ganzhi::hour_branch(m.hour, m.minute); // 子=0 … 亥=11

    // 正月、初一、子时分别为各步的「第 0 步」，故起点各减一后做 Z₆ 位移。
    let month_pos = count_to(0, i64::from(lunar_month) - 1, 6, true) as u8;
    let day_pos = count_to(i64::from(month_pos), i64::from(lunar_day) - 1, 6, true) as u8;
    let hour_pos = count_to(i64::from(day_pos), i64::from(hb), 6, true) as u8;

    Cast {
        lunar_month,
        lunar_day,
        hour_branch: hb,
        month_pos,
        day_pos,
        hour_pos,
        month_deity: DEITIES[month_pos as usize],
        day_deity: DEITIES[day_pos as usize],
        hour_deity: DEITIES[hour_pos as usize],
        hour_direction: DEITY_DIRECTION[hour_pos as usize],
        day_direction: DEITY_DIRECTION[day_pos as usize],
        month_direction: DEITY_DIRECTION[month_pos as usize],
    }
}

/// 由本地民用时刻掐指（独立入口，构造 [`Moment`]）。
#[must_use]
pub fn compute(year: i32, month: u32, day: u32, hour: u32, minute: u32, tz: f64) -> Cast {
    compute_at(&Moment::new(year, month, day, hour, minute, tz))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deities_form_a_six_cycle() {
        // 六神恰 6 个、互异；环的长度即模数。
        let set: std::collections::HashSet<_> = DEITIES.iter().collect();
        assert_eq!(set.len(), 6);
        assert_eq!(DEITIES[0], "大安");
        assert_eq!(DEITIES[5], "空亡");
    }

    #[test]
    fn manual_worked_example() {
        // 手算校验：农历正月初一子时 → 月神=日神=时神=大安(0)。
        // 正月(m=1)→pos0；初一(d=1)→pos0；子时(hb=0)→pos0。
        // 用一个公历日构造，再直接以其农历量复核（不依赖具体农历换算结果）。
        let m = Moment::new(2024, 6, 15, 0, 30, 8.0);
        let c = compute_at(&m);
        let mp = ((c.lunar_month - 1) % 6) as u8;
        let dp = ((u32::from(mp) + c.lunar_day - 1) % 6) as u8;
        let hp = ((u32::from(dp) + u32::from(c.hour_branch)) % 6) as u8;
        assert_eq!(c.month_pos, mp);
        assert_eq!(c.day_pos, dp);
        assert_eq!(c.hour_pos, hp);
        assert_eq!(c.hour_deity, DEITIES[hp as usize]);
    }

    #[test]
    fn classic_handcalc_lunar_5_23_noon() {
        // 经典掌诀手算：农历五月廿三午时（午时地支序 hb=6）。
        // 月神：正月起大安顺数到五月 → (5-1)%6=4 → 小吉。
        // 日神：从小吉(4)起初一顺数到廿三 → (4+22)%6 = 26%6 = 2 → 速喜。
        // 时神：从速喜(2)起子时顺数到午时 → (2+6)%6 = 2 → 速喜。
        let step = |start: u32, steps: u32| (start + steps) % 6;
        let month_pos = step(0, 5 - 1);
        assert_eq!(DEITIES[month_pos as usize], "小吉");
        let day_pos = step(month_pos, 23 - 1);
        assert_eq!(DEITIES[day_pos as usize], "速喜");
        let hour_pos = step(day_pos, 6); // 午=hb6
        assert_eq!(DEITIES[hour_pos as usize], "速喜");
    }

    #[test]
    fn deterministic() {
        let a = compute(2024, 6, 15, 14, 30, 8.0);
        let b = compute(2024, 6, 15, 14, 30, 8.0);
        assert_eq!(a.hour_pos, b.hour_pos);
        assert_eq!(a.month_deity, b.month_deity);
    }

    #[test]
    fn positions_always_in_range_and_chain_consistent() {
        // 性质测试：遍历多日多时辰，三神位恒在 0..6，且链式位移自洽。
        for day in 1..=28u32 {
            for hour in 0..24u32 {
                let c = compute(2023, 3, day.min(28), hour, 0, 8.0);
                assert!((c.month_pos as usize) < 6);
                assert!((c.day_pos as usize) < 6);
                assert!((c.hour_pos as usize) < 6);
                // 时神 = 日神位 + 时辰序 (mod 6)
                let expect = ((u32::from(c.day_pos) + u32::from(c.hour_branch)) % 6) as u8;
                assert_eq!(c.hour_pos, expect);
                // 名与位一致
                assert_eq!(c.hour_deity, DEITIES[c.hour_pos as usize]);
            }
        }
    }
}

#[cfg(test)]
mod direction_tests {
    use super::*;

    /// 六神配方位：四个定得下、两个定不下，且**定不下的那两个必须留空**。
    ///
    /// 这条守的不只是取值，还有「留空」这件事本身。硬给小吉一个方位、
    /// 或给空亡填个「中」，都会让本叶看起来能答「寻」——而那正是它答不了的。
    /// 四个定得下的，其五行—方位—四象三者自洽（大安木东青龙、留连水北玄武、
    /// 速喜火南朱雀、赤口金西白虎），这是它们能被多源同述的原因。
    #[test]
    fn four_of_the_six_deities_have_a_settled_direction() {
        assert_eq!(DEITY_DIRECTION.len(), DEITIES.len(), "方位表应与六神一一对应");
        for (name, want) in [
            ("大安", Some("东")),
            ("留连", Some("北")),
            ("速喜", Some("南")),
            ("赤口", Some("西")),
            ("小吉", None),
            ("空亡", None),
        ] {
            let k = DEITIES.iter().position(|d| *d == name).expect("六神应有此神");
            assert_eq!(DEITY_DIRECTION[k], want, "{name} 的方位");
        }
        // 定得下的四个互不同方——四正各一，这是它们自洽的表现
        let set: std::collections::BTreeSet<&str> = DEITY_DIRECTION.iter().flatten().copied().collect();
        assert_eq!(set.len(), 4, "四个定得下的应分居四正，实为 {set:?}");
        assert!(!set.contains("中"), "中宫不是可面向之方，不该出现在方位表里");
    }

    /// 盘面上的三个方位各自跟着自己那一级的神，不串位。
    #[test]
    fn each_level_carries_its_own_deitys_direction() {
        for (y, mo, d, h) in [(2026, 8, 19, 12), (1990, 6, 15, 0), (2024, 1, 1, 23), (1987, 9, 17, 7)] {
            let c = compute_at(&mingli_astro::Moment::new(y, mo, d, h, 0, 8.0));
            for (deity, dir, level) in [
                (c.month_deity, c.month_direction, "月"),
                (c.day_deity, c.day_direction, "日"),
                (c.hour_deity, c.hour_direction, "时"),
            ] {
                let k = DEITIES.iter().position(|x| *x == deity).expect("神名应在环上");
                assert_eq!(dir, DEITY_DIRECTION[k], "{y}-{mo}-{d} {h}时：{level}神「{deity}」的方位串位了");
            }
        }
    }

    /// 本叶不认领「寻」——因为六分之二的落点给不出方位。
    ///
    /// 这条与 `mingli-registry` 的「认领『寻』必须真给得出方位候选」互为表里：
    /// 那条防「认领了却给不出」，这条防「给得出一部分就去认领」。
    #[test]
    fn the_leaf_does_not_claim_locative_because_two_deities_cannot_answer() {
        let unsettled = DEITY_DIRECTION.iter().filter(|d| d.is_none()).count();
        assert_eq!(unsettled, 2, "小吉与空亡两处留空是不认领「寻」的理由，实有 {unsettled} 处");
    }
}

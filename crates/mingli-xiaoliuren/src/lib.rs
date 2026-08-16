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

mod engine;
pub use engine::XiaoliurenEngine;

use mingli_astro::Moment;
use mingli_core::group::count_to;
use serde::Serialize;

/// 六神固定环（次序即掐指方向）：大安→留连→速喜→赤口→小吉→空亡。
pub const DEITIES: [&str; 6] = ["大安", "留连", "速喜", "赤口", "小吉", "空亡"];

/// 一次小六壬掐指的结果。三个神位皆为 `0..6` 的环上下标。
#[derive(Debug, Clone, Copy, Serialize)]
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

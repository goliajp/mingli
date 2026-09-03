//! L3 叶（A 族 / 确定性）：缅甸 Mahabote（မဟာဘုတ်）本命核心数。
//!
//! Mahabote 的可计算内核是把出生日折成一个 `Z₇`：
//! `核心数 = (缅历年 − 星期) mod 7`，落入七宫（house）之一。本 crate 实现缅甸**本土算法**
//! （Yan Naing Aye / cool-emerald 算法，逐字核对、单测黄金向量自洽），不实现西方实践者那套
//! 另起炉灶的行星盘（两套算法互不兼容，混用即毒化）。
//!
//! - **缅历年** `my = ⌊(JDN − 0.5 − [`EPOCH_OFFSET`]) / [`TROPICAL_YEAR`]⌋`。
//! - **星期** `wd = (JDN + 2) mod 7`，编号 `0=Sat … 6=Fri`。
//! - **核心数** `(my − wd) mod 7`（取非负），索引 [`HOUSES`]。
//!
//! 另附**八天週 → 行星**：缅历一週八日（周三按正午拆 Mercury / Rahu）。这是高置信的固定属性。
//!
//! 诚实边界（🟡 不写）：七宫的英文含义配对、宫间关系（trine/square 的精确几何）在可达权威来源里
//! 没有自洽单一出处，故本 crate **只给宫名与核心数**，不下含义/关系断言。

#![allow(

    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap,
    reason = "核心数/星期经 rem_euclid 落 0..7，缅历年由 JDN(~2.5e6) floor 得；与 i64/usize 间换算受控"
)]

#[cfg(feature = "port")]
mod engine;
#[cfg(feature = "port")]
pub use engine::MahaboteEngine;

use mingli_astro::Moment;
#[cfg(feature = "serde")]
use serde::Serialize;

/// 缅历 0 ME 的 JDN 起点偏移（cool-emerald 算法常数）。
pub const EPOCH_OFFSET: f64 = 1_954_168.050_623;
/// 回归年长（= 1577917828/4320000 天，cool-emerald 算法常数）。
pub const TROPICAL_YEAR: f64 = 1_577_917_828.0 / 4_320_000.0;

/// 七宫名（约定 B，0-based，与本土公式 `(my−wd) mod 7` 配套）。
///
/// 注：索引约定必须与公式绑定；另有一套传统 1..7 约定配另一条公式，二者**不可交叉**。
pub const HOUSES: [&str; 7] = ["Binga", "Atun", "Yaza", "Adipati", "Marana", "Thike", "Puti"];

/// 星期名，编号 `0=Sat … 6=Fri`（`wd = (JDN+2) mod 7`）。
pub const WEEKDAYS: [&str; 7] = ["Sat", "Sun", "Mon", "Tue", "Wed", "Thu", "Fri"];

/// 八天週对应行星：日..六（周三拆早/晚）。Sun→Sun， Mon→Moon， …， Wed-AM→Mercury， Wed-PM→Rahu。
pub const PLANETS_8: [&str; 8] = [
    "Sun", "Moon", "Mars", "Mercury", "Rahu", "Jupiter", "Venus", "Saturn",
];

/// 由 JDN 算缅历年（floor 式）。
#[must_use]
pub fn myanmar_year(jdn: i64) -> i64 {
    #[allow(
        clippy::cast_precision_loss,
        clippy::cast_possible_truncation,
        reason = "JDN 量级 ~2.5e6，f64 精确表示；floor 后落 i64 受控"
    )]
    let my = ((jdn as f64 - 0.5 - EPOCH_OFFSET) / TROPICAL_YEAR).floor() as i64;
    my
}

/// 由 JDN 算星期，`0=Sat … 6=Fri`。
#[must_use]
pub fn weekday(jdn: i64) -> usize {
    (jdn + 2).rem_euclid(7) as usize
}

/// 八天週的行星下标 `0..8`：周三（`wd==4`）按 `before_noon` 拆 Mercury(3)/Rahu(4)，其余直接映射。
#[must_use]
pub fn planet8_index(wd: usize, before_noon: bool) -> usize {
    // wd: 0=Sat,1=Sun,2=Mon,3=Tue,4=Wed,5=Thu,6=Fri
    match wd {
        1 => 0, // Sun
        2 => 1, // Moon
        3 => 2, // Mars
        4 => {
            if before_noon {
                3 // Mercury
            } else {
                4 // Rahu
            }
        }
        5 => 5, // Jupiter
        6 => 6, // Venus
        _ => 7, // Sat → Saturn
    }
}

/// 一次 Mahabote 本命换算的结果。
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct Cast {
    /// 缅历年。
    pub myanmar_year: i64,
    /// 星期下标 `0=Sat … 6=Fri`。
    pub weekday_index: u8,
    /// 星期名。
    pub weekday: &'static str,
    /// 核心数 `0..7`。
    pub core: u8,
    /// 本命宫名。
    pub house: &'static str,
    /// 八天週行星名（周三按出生在午前/午后拆 Mercury/Rahu）。
    pub planet: &'static str,
}

/// 在共享上下文 [`Moment`] 上算 Mahabote（取其民用日序与出生时辰午前/午后）。
#[must_use]
pub fn compute_at(m: &Moment) -> Cast {
    let jdn = m.civil_day;
    let my = myanmar_year(jdn);
    let wd = weekday(jdn);
    let core = (my - wd as i64).rem_euclid(7) as usize;
    let planet = PLANETS_8[planet8_index(wd, m.hour < 12)];
    Cast {
        myanmar_year: my,
        #[allow(clippy::cast_possible_truncation, reason = "wd∈0..7，窄化安全")]
        weekday_index: wd as u8,
        weekday: WEEKDAYS[wd],
        #[allow(clippy::cast_possible_truncation, reason = "core∈0..7，窄化安全")]
        core: core as u8,
        house: HOUSES[core],
        planet,
    }
}

/// 由本地民用时刻算 Mahabote（独立入口，构造 [`Moment`]）。
#[must_use]
pub fn compute(year: i32, month: u32, day: u32, hour: u32, minute: u32, tz: f64) -> Cast {
    compute_at(&Moment::new(year, month, day, hour, minute, tz))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn golden_vector_2000_01_01() {
        // cool-emerald 逐字算例：2000-01-01 (JDN 2451545) → my=1361， Sat， 核心数 3 = Adipati。
        assert_eq!(mingli_astro::civil_day_number(2000, 1, 1), 2_451_545);
        assert_eq!(myanmar_year(2_451_545), 1361);
        assert_eq!(weekday(2_451_545), 0); // Sat
        let c = compute(2000, 1, 1, 9, 0, 6.5); // 缅甸 UTC+6：30
        assert_eq!(c.myanmar_year, 1361);
        assert_eq!(c.weekday, "Sat");
        assert_eq!(c.core, 3);
        assert_eq!(c.house, "Adipati");
        assert_eq!(c.planet, "Saturn");
    }

    #[test]
    fn weekday_matches_known_days() {
        // 2024-01-01 是周一；2000-01-01 是周六。
        assert_eq!(WEEKDAYS[weekday(mingli_astro::civil_day_number(2024, 1, 1))], "Mon");
        assert_eq!(WEEKDAYS[weekday(mingli_astro::civil_day_number(2000, 1, 1))], "Sat");
        // 一周内逐日推进，覆盖 7 个星期且互异。
        let base = mingli_astro::civil_day_number(2024, 1, 1);
        let set: std::collections::HashSet<_> = (0..7).map(|k| weekday(base + k)).collect();
        assert_eq!(set.len(), 7);
    }

    #[test]
    fn wednesday_splits_mercury_rahu() {
        // 找一个周三（wd==4），午前 Mercury、午后 Rahu。
        // 七天之内必遇周三。从前这里是没有上限的 `while`：把 `weekday` 换成常数，
        // 它就永远找不到，测试挂在这里而不是红——变异扫描里两个超时出自这一处。
        let base = (0..7)
            .map(|k| mingli_astro::civil_day_number(2024, 1, 1) + k)
            .find(|&d| weekday(d) == 4)
            .expect("七天之内必有一个周三");
        assert_eq!(weekday(base), 4);
        assert_eq!(planet8_index(4, true), 3); // Mercury
        assert_eq!(planet8_index(4, false), 4); // Rahu
        assert_eq!(PLANETS_8[3], "Mercury");
        assert_eq!(PLANETS_8[4], "Rahu");
        // 其余六日不分早晚。
        for wd in [0usize, 1, 2, 3, 5, 6] {
            assert_eq!(planet8_index(wd, true), planet8_index(wd, false));
        }

        // 每一天对应哪一颗，逐个钉住。
        //
        // 上面只问了「周三分不分早晚」，没问哪一天是哪颗星。于是把周日到周五那几条
        // match 分支逐个删掉、让它们落到兜底的 Saturn 上，五个变异体全部活着：
        // 「不分早晚」在全都变成土星之后照样成立。
        for (wd, idx, name) in [
            (0_usize, 7_usize, "Saturn"),
            (1, 0, "Sun"),
            (2, 1, "Moon"),
            (3, 2, "Mars"),
            (5, 5, "Jupiter"),
            (6, 6, "Venus"),
        ] {
            assert_eq!(planet8_index(wd, true), idx, "星期下标 {wd} 的行星位");
            assert_eq!(PLANETS_8[idx], name, "第 {idx} 位应是 {name}");
        }
    }

    /// 正午整点归午后：`hour < 12` 的那道界。
    ///
    /// 周三按午前午后拆 Mercury / Rahu，而全模块只有这一处读时刻。把 `<` 松成 `<=`，
    /// 只有生在正午整点的盘会变——此前没有一条测试取在那一刻，于是它活着。
    #[test]
    fn noon_itself_counts_as_afternoon() {
        // 找一个周三（同上，七天之内必有）。
        let wed = (0..7)
            .map(|k| mingli_astro::civil_day_number(2024, 1, 1) + k)
            .find(|&d| weekday(d) == 4)
            .expect("七天之内必有一个周三");
        let (y, mo, d) = (2024, 1, 1 + (wed - mingli_astro::civil_day_number(2024, 1, 1)));
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss, reason = "日在 1..=7")]
        let day = d as u32;
        let before = compute(y, 1, day, 11, 59, 6.5);
        let at_noon = compute(y, mo, day, 12, 0, 6.5);
        assert_eq!(before.planet, "Mercury", "午前应是 Mercury");
        assert_eq!(at_noon.planet, "Rahu", "正午整点应归午后，取 Rahu");
    }

    /// 缅历年号本身此前只在 2000-01-01 一个日期上验过，而它是个浮点历元折算，
    /// 差一年不会有任何地方察觉——核心数、宫、行星全跟着错。
    ///
    /// 这条与 `profile()` 里标着 Und 的「本命宫取法」无关：宫怎么落是单源未定的，
    /// 而缅历纪元是另一件事，有两源可依：
    ///
    /// 1. 缅历纪元为公元 638 年（历元 638-03-22 儒略历），故年号 = 公历年 − 638
    ///    <https://en.wikipedia.org/wiki/Burmese_calendar>
    /// 2. 缅甸新年（太阳入白羊）现落 4 月 16 或 17 日；2024 年新年日为 4 月 17 日
    ///    <https://en.wikipedia.org/wiki/Thingyan>
    ///
    /// 两源合起来给出一条可查的关系：跳变前 ME = 公历 − 639，跳变后 = 公历 − 638。
    /// 取 2 月 1 日与 8 月 1 日，两头都离四月的跳变足够远，不必知道跳变具体在哪天。
    /// 年号跳变落在哪一天，逐年钉住。
    ///
    /// 上面那条取 2 月与 8 月，两头都离跳变很远——于是把历元折算里的 `− 0.5`
    /// 写成 `+ 0.5`（整条界挪一天）也没人红。跳变日本身才是那半天唯一显形的地方。
    ///
    /// 实测（2026-09-03）：1900 年 4-16、1950 年 4-17、2000 年 4-16、2024 年 4-17、
    /// 2100 年 4-18。与本模块注释里「实测落在 4 月 16 或 17 日」一致；2100 年漂到 18，
    /// 是平回归年简化式与真实岁差的差，本模块的 `profile()` 已声明这一点。
    #[test]
    fn the_year_number_jumps_on_the_day_it_has_always_jumped() {
        for (year, want_day) in [(1900_i32, 16_u32), (1950, 17), (2000, 16), (2024, 17), (2100, 18)] {
            let jump = (2..=30_u32)
                .find(|&d| {
                    myanmar_year(mingli_astro::civil_day_number(year, 4, d))
                        != myanmar_year(mingli_astro::civil_day_number(year, 4, d - 1))
                })
                .expect("四月内应有且仅有一次跳变");
            assert_eq!(jump, want_day, "{year} 年的年号跳变日");
        }
    }

    #[test]
    fn the_year_number_follows_the_era_epoch_across_two_centuries() {
        for year in 1900..=2100i32 {
            let before = myanmar_year(mingli_astro::civil_day_number(year, 2, 1));
            assert_eq!(
                before,
                i64::from(year) - 639,
                "{year}-02-01 在缅甸新年之前，缅历年应为公历减 639"
            );
            let after = myanmar_year(mingli_astro::civil_day_number(year, 8, 1));
            assert_eq!(
                after,
                i64::from(year) - 638,
                "{year}-08-01 在缅甸新年之后，缅历年应为公历减 638"
            );
        }
    }

    /// 年号一年只跳一次，且跳在四月。
    ///
    /// 本模块用的是平回归年的简化式，跳变实测落在 4 月 16 或 17 日。维基条目称受岁差
    /// 影响，实际缅历的新年日在二十世纪是 4 月 15 或 16 日、十七世纪是 4 月 9 或 10 日——
    /// 即本模块在早年可能偏后一天。断言只取「落在四月」，不把简化式的精度当成事实。
    #[test]
    fn the_year_number_advances_once_a_year_in_april() {
        for year in 1900..=2100i32 {
            let jan = mingli_astro::civil_day_number(year, 1, 1);
            let next_jan = mingli_astro::civil_day_number(year + 1, 1, 1);
            let mut rollovers = Vec::new();
            for jdn in jan..next_jan {
                if myanmar_year(jdn) != myanmar_year(jdn - 1) {
                    rollovers.push(jdn);
                }
            }
            assert_eq!(rollovers.len(), 1, "{year} 年该只跳一次，实得 {}", rollovers.len());
            let offset = rollovers[0] - jan;
            let april_1 = mingli_astro::civil_day_number(year, 4, 1) - jan;
            let may_1 = mingli_astro::civil_day_number(year, 5, 1) - jan;
            assert!(
                (april_1..may_1).contains(&offset),
                "{year} 年的跳变该落在四月，实得距 1 月 1 日 {offset} 天"
            );
        }
    }

    /// 对外报出的那一组字段彼此对不对得上。
    ///
    /// 原先这里是一段扫两万天的循环，但它在测试内部自己算 `core`，再断言
    /// `core < 7`（由它自己那个 `rem_euclid(7)` 保证）与 `HOUSES[core] == HOUSES[core]`
    /// （自己等于自己），一次也没调用 `compute`。换成拿 `compute` 的输出互相对账：
    /// 宫名要取自它自己报的核心数，核心数要合它自己报的年与星期，行星要合八天週的规则。
    #[test]
    fn what_compute_reports_hangs_together() {
        for year in [1900i32, 1950, 2000, 2024, 2050] {
            for (month, day) in [(1u32, 1u32), (4, 16), (4, 17), (7, 15), (12, 31)] {
                for hour in [9u32, 15] {
                    let c = compute(year, month, day, hour, 0, 6.5);
                    let jdn = mingli_astro::civil_day_number(year, month, day);
                    assert_eq!(c.myanmar_year, myanmar_year(jdn));
                    assert_eq!(usize::from(c.weekday_index), weekday(jdn));
                    assert_eq!(c.weekday, WEEKDAYS[usize::from(c.weekday_index)]);
                    assert_eq!(
                        i64::from(c.core),
                        (c.myanmar_year - i64::from(c.weekday_index)).rem_euclid(7),
                        "{year}-{month:02}-{day:02}：核心数与它自己报的年、星期不符"
                    );
                    assert_eq!(
                        c.house,
                        HOUSES[usize::from(c.core)],
                        "{year}-{month:02}-{day:02}：宫名没取自它自己报的核心数"
                    );
                    assert_eq!(
                        c.planet,
                        PLANETS_8[planet8_index(usize::from(c.weekday_index), hour < 12)],
                        "{year}-{month:02}-{day:02} {hour} 时：行星不合八天週"
                    );
                }
            }
        }
        // 八天週行星名互异（含 Rahu 共 8 个）。
        let set: std::collections::HashSet<_> = PLANETS_8.iter().collect();
        assert_eq!(set.len(), 8);
    }

    #[test]
    fn name_tables_well_formed() {
        assert_eq!(HOUSES.len(), 7);
        assert_eq!(WEEKDAYS.len(), 7);
        assert_eq!(PLANETS_8.len(), 8);
        assert_eq!(HOUSES[3], "Adipati");
    }
}

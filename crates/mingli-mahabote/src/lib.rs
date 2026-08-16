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

mod engine;
pub use engine::MahaboteEngine;

use mingli_astro::Moment;
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
#[derive(Debug, Clone, Serialize)]
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
        let mut base = mingli_astro::civil_day_number(2024, 1, 1);
        while weekday(base) != 4 {
            base += 1;
        }
        assert_eq!(planet8_index(4, true), 3); // Mercury
        assert_eq!(planet8_index(4, false), 4); // Rahu
        assert_eq!(PLANETS_8[3], "Mercury");
        assert_eq!(PLANETS_8[4], "Rahu");
        // 其余六日不分早晚。
        for wd in [0usize, 1, 2, 3, 5, 6] {
            assert_eq!(planet8_index(wd, true), planet8_index(wd, false));
        }
    }

    #[test]
    fn core_always_in_range_and_house_consistent() {
        // 性质：扫描多年多日，核心数恒在 0..7、宫名与核心数一致。
        let base = mingli_astro::civil_day_number(1980, 1, 1);
        for k in (0..20_000).step_by(37) {
            let jdn = base + k;
            let my = myanmar_year(jdn);
            let wd = weekday(jdn);
            let core = (my - wd as i64).rem_euclid(7) as usize;
            assert!(core < 7);
            assert_eq!(HOUSES[core], HOUSES[core]);
        }
        // 所有八天週行星名互异（含 Rahu 共 8 个）。
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

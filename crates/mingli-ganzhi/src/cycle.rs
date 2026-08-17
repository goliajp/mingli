//! 六十甲子本体：干支表、日 / 年 / 月 / 时四柱的推法。
//!
//! 日柱以民用日序（JDN）递推，对天文零依赖。

use super::*;

/// 六十干支的循环周期（= `mingli_core::cyclic::cycle_period(&[10,12])`）。
pub const CYCLE: u8 = 60;

/// 日柱锚点：民用日序（JDN）2_460_311 = 公历 2024-01-01 = 甲子(#0)。
pub const DAY_ANCHOR_JDN: i64 = 2_460_311;

/// 十天干字面（甲=0 … 癸=9）。
pub const STEMS: [&str; 10] = ["甲", "乙", "丙", "丁", "戊", "己", "庚", "辛", "壬", "癸"];
/// 十二地支字面（子=0 … 亥=11）。
pub const BRANCHES: [&str; 12] = [
    "子", "丑", "寅", "卯", "辰", "巳", "午", "未", "申", "酉", "戌", "亥",
];

/// 一个干支组合：`stem` 天干 0..9（甲=0），`branch` 地支 0..11（子=0）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct GanZhi {
    /// 天干序号 0..9（甲=0）。
    pub stem: u8,
    /// 地支序号 0..11（子=0）。
    pub branch: u8,
}

impl GanZhi {
    /// 60 甲子序号 0..59（甲子=0）。
    #[must_use]
    pub fn index(&self) -> u8 {
        let mut n = i32::from(self.stem);
        while n % 12 != i32::from(self.branch) {
            n += 10;
        }
        (n % 60) as u8
    }
    /// 由 60 甲子序号 `n`（甲子=0）构造（对 `n` 取模，越界安全）。
    #[must_use]
    pub fn from_index(n: u8) -> Self {
        GanZhi {
            stem: n % 10,
            branch: n % 12,
        }
    }
    /// 天干字面。
    #[must_use]
    pub fn stem_str(&self) -> &'static str {
        STEMS[self.stem as usize]
    }
    /// 地支字面。
    #[must_use]
    pub fn branch_str(&self) -> &'static str {
        BRANCHES[self.branch as usize]
    }
}

impl std::fmt::Display for GanZhi {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}{}", self.stem_str(), self.branch_str())
    }
}

/// 日柱干支序号 0..59，输入为民用日序（JDN）。以 [`DAY_ANCHOR_JDN`] 为锚线性递推。
///
/// 注：八字「晚子时换日」传统不在此处处理；调用方按需在传入的 JDN 上 ±1。
#[must_use]
pub fn day_ganzhi_index(civil_day_jdn: i64) -> u8 {
    (civil_day_jdn - DAY_ANCHOR_JDN).rem_euclid(i64::from(CYCLE)) as u8
}

/// 日柱干支，输入为民用日序（JDN）。
#[must_use]
pub fn day_ganzhi(civil_day_jdn: i64) -> GanZhi {
    GanZhi::from_index(day_ganzhi_index(civil_day_jdn))
}

/// 年柱干支。`solar_year` 须为已按立春调整后的年份（八字）或农历年（紫微）。
#[must_use]
pub fn year_ganzhi(solar_year: i32) -> GanZhi {
    GanZhi {
        stem: (solar_year - 4).rem_euclid(10) as u8,
        branch: (solar_year - 4).rem_euclid(12) as u8,
    }
}

/// 五虎遁：给定年干，返回某地支宫位对应的天干（0..9）。寅(2) 为正月起点。
/// 用于月柱天干，以及紫微「命宫天干」。
#[must_use]
pub fn month_pillar_stem(year_stem: u8, branch: u8) -> u8 {
    let base = ((year_stem % 5) * 2 + 2) % 10; // 寅之干（甲己→丙…）
    let pos = (i32::from(branch) - 2).rem_euclid(12) as u8; // 距寅步数
    (base + pos) % 10
}

/// 时辰地支 0..11（子=0）。23：00–01：00 为子时。
#[must_use]
pub fn hour_branch(hour: u32, _minute: u32) -> u8 {
    (((hour + 1) % 24) / 2) as u8
}

/// 由字符串（"甲子"/"癸亥"等）解析干支。两字异常或不在表内返回 None。
#[must_use]
pub fn parse_ganzhi(s: &str) -> Option<GanZhi> {
    let mut it = s.chars();
    let st = it.next()?;
    let br = it.next()?;
    if it.next().is_some() {
        return None;
    }
    let stem = STEMS.iter().position(|&v| v.starts_with(st))?;
    let branch = BRANCHES.iter().position(|&v| v.starts_with(br))?;
    Some(GanZhi { stem: stem as u8, branch: branch as u8 })
}

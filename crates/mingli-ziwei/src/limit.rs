//! 大限与流年：紫微论运的两条主路。
//!
//! **大限**十年一宫，两源同述其三条规矩
//! （<https://iztro.com/learn/basis>、<https://zhuanlan.zhihu.com/p/718987833>）：
//!
//! 1. 起运岁 = 五行局数——水二局 2 岁起、木三 3、金四 4、土五 5、火六 6
//! 2. 第一大限**固定落在命宫**，此后每十年推进一宫
//! 3. 顺逆由「年干阴阳 + 性别」定：**阳男阴女顺行、阴男阳女逆行**
//!
//! 第 3 条的方向以地支序论：顺行即支序递增。十二宫名自命宫起**逆时针**排
//! （命/兄弟/夫妻/…/父母），故顺行经过的宫名是 命→父母→福德→田宅→官禄→交友…，
//! 逆行则是 命→兄弟→夫妻→子女→财帛→疾厄…——两源列的正是这两串。
//!
//! **流年**按太岁支入宫：某年的年支落在哪一宫，那一宫即该年的流年宫。
//! 这一条不涉顺逆也不涉性别，是十二支与十二宫的直接对位。
//!
//! 性别缺省时不出大限（与四柱大运同一处置：顺逆定不下就不给）。

use mingli_ganzhi::BRANCHES;
use serde::Serialize;

/// 一步大限（十年）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MajorLimit {
    /// 第几步（1 起）。
    pub step: u32,
    /// 起岁（含）。
    pub start_age: u32,
    /// 止岁（含）。
    pub end_age: u32,
    /// 所值宫的地支序（子=0）。
    pub branch_index: u8,
    /// 所值宫的地支字面。
    pub branch: &'static str,
    /// 所值宫名。
    pub palace: &'static str,
}

/// 大限盘：十二步走满一轮。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MajorLimits {
    /// 起运岁 = 五行局数。
    pub start_age: u32,
    /// 是否顺行（阳男阴女为真）。
    pub forward: bool,
    /// 十二步。
    pub steps: Vec<MajorLimit>,
}

/// 排大限。
///
/// `ming_branch` 命宫地支序（子=0）、`ju` 五行局数（2..=6）、
/// `year_stem` 年干序（甲=0）、`male` 性别。
#[must_use]
pub fn major_limits(ming_branch: u8, ju: u32, year_stem: u8, male: bool) -> MajorLimits {
    // 年干阴阳：甲丙戊庚壬（偶序）为阳
    let yang_year = year_stem.is_multiple_of(2);
    // 阳男阴女顺行、阴男阳女逆行
    let forward = yang_year == male;
    let steps = (0..12)
        .map(|i| {
            let offset = i32::try_from(i).unwrap_or(0);
            let delta = if forward { offset } else { -offset };
            let bi = u8::try_from((i32::from(ming_branch) + delta).rem_euclid(12)).unwrap_or(0);
            // 宫名自命宫起逆时针排，故宫名序 = (命宫支 − 本宫支) mod 12
            let pi = usize::from(
                u8::try_from((i32::from(ming_branch) - i32::from(bi)).rem_euclid(12)).unwrap_or(0),
            );
            MajorLimit {
                step: i + 1,
                start_age: ju + i * 10,
                end_age: ju + i * 10 + 9,
                branch_index: bi,
                branch: BRANCHES[usize::from(bi)],
                palace: crate::PALACE_NAMES[pi],
            }
        })
        .collect();
    MajorLimits { start_age: ju, forward, steps }
}

/// 某公历年的流年宫：太岁支入宫。
///
/// 年支序 = `(year − 4) mod 12`（甲子年为公元 4 年，子=0）。返回 (支序, 宫名)。
#[must_use]
pub fn annual_palace(ming_branch: u8, year: i32) -> (u8, &'static str) {
    let bi = u8::try_from((year - 4).rem_euclid(12)).unwrap_or(0);
    let pi = usize::from(u8::try_from((i32::from(ming_branch) - i32::from(bi)).rem_euclid(12)).unwrap_or(0));
    (bi, crate::PALACE_NAMES[pi])
}

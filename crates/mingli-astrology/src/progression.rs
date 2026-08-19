//! 二次推运（secondary progression）：**出生后第 N 天的天象代表人生第 N 年**。
//!
//! 这条「一日一年」的换算是西洋占星里推运的通行主法，两源同述：
//! Cafe Astrology《Secondary Progressions》与 Kepler College 图书馆
//! 《An Introduction to Secondary Progressions》都作「one day after birth equals one year of life」，
//! 查星历时数到出生后第 5 日即对应第 5 年。
//!
//! 它是**由出生时刻单独可导出的时间序列**——这一点决定了它能落在本层。
//! 行运（transit）要的是「本命 + 另一个当下时刻」两个入参，而端口层的 `cast(&Moment, &Query)`
//! 只给一个时刻；推运不需要第二个时刻，故本叶答「运」走推运这一路。
//!
//! 两条可独立验证的性质（两源都点了名，本 crate 的测试拿它们当 oracle）：
//! 推运太阳约 **1°/年**、推运月亮约 **13°/年**（故月亮每两三年换一座）。

use crate::{compute_planets, CrossAspect, PlanetPos, DEFAULT_ORB};
use serde::Serialize;

/// 一年的推运切片。
#[derive(Debug, Clone, Serialize)]
pub struct ProgressedYear {
    /// 岁数（0 = 出生当年）。
    pub age: u32,
    /// 该年的推运行星位置（= 出生后第 `age` 日的天象）。
    pub planets: Vec<PlanetPos>,
    /// 推运盘与本命盘之间的相位——「运」的着力处：推运星走到本命星的角上。
    pub to_natal: Vec<CrossAspect>,
}

/// 一生的推运时间序列。
#[derive(Debug, Clone, Serialize)]
pub struct Progression {
    /// 换算法的稳定 id。
    pub method: &'static str,
    /// 覆盖到第几岁（含）。
    pub max_age: u32,
    /// 相邻两格相差几岁。盘面出的是 10；传 1 得逐年。
    pub step: u32,
    /// 逐年切片。
    pub years: Vec<ProgressedYear>,
}

/// 由本命 JDE 推一条时间序列，每 `step` 岁一个切片。
///
/// `natal` 为本命行星位置（相位的另一端）。每个切片是出生后第 N 日的天象。
///
/// `step = 1` 得逐年全分辨率；「运」那条用例取 `step = 5`。
///
/// 本命盘上**一格都不出**。逐年那份（101 格 × 9 星 + 相位）曾直接挂在盘上，
/// 代价是每问一次本命盘就要付一遍百年星历——排一盘 35.6 ms、盘面 276 KB，
/// 而其余二十片叶合起来是 0.5 ms、33 KB。问本命盘的人并没有要一生的推运。
/// 这与四柱的处置一致：那一片的盘面出十步大运，逐年的供给时序在用例层另算。
#[must_use]
pub fn progression(natal_jde: f64, natal: &[PlanetPos], max_age: u32, step: u32) -> Progression {
    let step = step.max(1);
    let years = (0..=max_age)
        .step_by(step as usize)
        .map(|age| {
            // 一日一年：第 N 年 = 出生后第 N 日
            let planets = compute_planets(natal_jde + f64::from(age), None);
            let to_natal = crate::cross_aspects(&planets, natal, DEFAULT_ORB);
            ProgressedYear { age, planets, to_natal }
        })
        .collect();
    Progression { method: "secondary", max_age, step, years }
}


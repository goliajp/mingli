//! 格局：伏吟 / 反吟与三奇得使等可判定的盘面结构。
//!
//! 只出**结构性**判定（能由盘面直接读出的），不下吉凶断语。

use super::*;

/// 三吉门：开 · 休 · 生。
pub const JI_MEN: [&str; 3] = ["开门", "休门", "生门"];

/// 三奇：乙（日奇）· 丙（月奇）· 丁（星奇）。
pub const SAN_QI: [&str; 3] = ["乙", "丙", "丁"];

/// 一处「三奇临吉门」：某宫的天盘三奇与三吉门同宫。
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct QiGate {
    /// 宫号 1..=9。
    pub palace: u8,
    /// 天盘三奇之一。
    pub qi: &'static str,
    /// 同宫的吉门。
    pub gate: &'static str,
}

/// 盘面结构格局。
///
/// 这里只出**结构事实**（哪几处成立），不出吉凶断语——判读属释义层。
/// 本 crate 目前只收无流派争议的几条：伏吟 / 反吟（由旋转格数直接判定）与三奇临吉门。
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[allow(
    clippy::struct_excessive_bools,
    reason = "星/门 × 伏吟/反吟 是四条彼此独立、可同时成立的判定，摊平比塞进枚举更贴合盘面"
)]
pub struct Patterns {
    /// 星伏吟：天盘九星各归原宫（旋转 0 格）。
    pub star_fu_yin: bool,
    /// 星反吟：天盘九星各落原宫的对冲宫（旋转 4 格）。
    pub star_fan_yin: bool,
    /// 门伏吟：八门各归本位（旋转 0 格）。
    pub gate_fu_yin: bool,
    /// 门反吟：八门各落本位的对冲宫（旋转 4 格）。
    pub gate_fan_yin: bool,
    /// 干伏吟的宫号：该宫天盘干与地盘干相同。
    pub stem_fu_yin_palaces: Vec<u8>,
    /// 全盘伏吟：星 · 门 · 干三者俱伏。
    pub full_fu_yin: bool,
    /// 三奇临吉门的各处。
    pub qi_gates: Vec<QiGate>,
}

/// 圆周上相隔 4 格即对冲宫（坎 1 ↔ 离 9 · 坤 2 ↔ 艮 8 · 震 3 ↔ 兑 7 · 巽 4 ↔ 乾 6）。
const FAN_YIN_SHIFT: u8 = 4;

/// 判盘面格局（只出结构，不下断语）。
#[must_use]
pub fn patterns(earth: &[&'static str; 9], sky: &SkyPlate, gates: &GatePlate) -> Patterns {
    let stem_fu_yin_palaces: Vec<u8> = (1..=9u8)
        .filter(|&p| {
            let k = p as usize - 1;
            !sky.stems[k].is_empty() && sky.stems[k] == earth[k]
        })
        .collect();
    let star_fu_yin = sky.shift == 0;
    let gate_fu_yin = gates.shift == 0;
    let qi_gates = (1..=9u8)
        .filter_map(|p| {
            let k = p as usize - 1;
            let qi = SAN_QI.iter().find(|&&q| q == sky.stems[k])?;
            let gate = JI_MEN.iter().find(|&&g| g == gates.gates[k])?;
            Some(QiGate { palace: p, qi, gate })
        })
        .collect();
    Patterns {
        star_fu_yin,
        star_fan_yin: sky.shift == FAN_YIN_SHIFT,
        gate_fu_yin,
        gate_fan_yin: gates.shift == FAN_YIN_SHIFT,
        full_fu_yin: star_fu_yin && gate_fu_yin && !stem_fu_yin_palaces.is_empty(),
        stem_fu_yin_palaces,
        qi_gates,
    }
}

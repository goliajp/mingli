//! L3 叶（⟂ 横切 / 确定性）：太乙神数的可计算结构。
//!
//! 太乙数（三式之首）把一个**积年**（自上元累积的年数）折成太乙在八宫上的位置：
//!
//! - **太乙积年**：自「上元混沌甲子」累积。历元锚用《太乙金镜式经》：唐开元十二年（724 CE）=
//!   积年 `1_937_281`（[`accumulated_years`]）。
//! - **太乙行宫**：太乙**三年居一宫、不入中宫、二十四年转一周（八宫×3）、七十二年游三期**，
//!   阳遁顺行（一宫起）、阴遁逆行（九宫起）。每宫三年依次「理天 / 理地 / 理人」（三才）。
//!   宫由积年定：`r = 积年 mod 24`，宫序 `= ⌊r/3⌋`，三才 `= r mod 3`（[`taiyi_palace`]）。
//! - **阴阳遁**：冬至后阳遁、夏至后阴遁（由太阳视黄经定）。
//! - **十六神**：八正宫（八卦方位）+ 八间神（十二支中非四正者）共十六方位，是太乙盘的方位框架。
//!
//! 八宫复用洛书九宫（[`mingli_luoshu`]）的宫数↔八卦映射。
//!
//! 诚实边界（🟡）：
//! - 太乙落宫的**绝对相位**遵循「积年起一宫」的引文规则；精确到年的校验需权威排盘软件，暂记 🟡。
//! - **文昌 / 始击 / 主客算 / 主客大将参将 / 君臣民基 / 大游小游**等诸神起法源间分歧且无校验工具，
//!   本 crate **暂不实现**（先做确定的太乙行宫结构）。
//! - **十六神的古僻神名**（地主/阳德/…）仅得自单一（且转写有异）来源，作 [`SIXTEEN_GODS`] 暴露但标 🟡；
//!   indisputable 的十六**方位**（十二支+四维）见 [`SIXTEEN_DIRECTIONS`]。

#![allow(

    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "积年 mod 24 ∈ 0..24、step∈0..8、三才∈0..3，窄化到 u8/usize 受控安全"
)]

mod engine;
pub use engine::TaiyiEngine;

use mingli_astro::Moment;
use serde::Serialize;

/// 太乙积年历元锚：唐开元十二年 = 公元 724 年。
pub const TAIYI_EPOCH_YEAR: i64 = 724;
/// 该锚年的太乙积年（《太乙金镜式经》：上元混沌甲子 → 724 CE）。
pub const TAIYI_EPOCH_JINIAN: i64 = 1_937_281;

/// 太乙行宫的八宫（洛书宫数，顺行序，不入中五）：一宫起 1→2→3→4→6→7→8→9。
pub const PALACES_8: [u8; 8] = [1, 2, 3, 4, 6, 7, 8, 9];

/// 三才（太乙居一宫三年，依次理天/理地/理人）。
pub const SANCAI: [&str; 3] = ["理天", "理地", "理人"];

/// 十六方位（八正宫 + 八间神）：十二地支 + 四维卦，按罗盘自子顺时针。**indisputable 结构**。
pub const SIXTEEN_DIRECTIONS: [&str; 16] = [
    "子", "丑", "艮", "寅", "卯", "辰", "巽", "巳", "午", "未", "坤", "申", "酉", "戌", "乾", "亥",
];

/// 十六神古僻神名（与 [`SIXTEEN_DIRECTIONS`] 同序）。🟡 单源、转写有异、未校验，仅供参考。
pub const SIXTEEN_GODS: [&str; 16] = [
    "地主", "阳德", "和德", "吕申", "高丛", "太阳", "大炅", "大神", "大威", "天道", "大武", "武德",
    "太簇", "阴主", "阴德", "大义",
];

/// 由公历年算太乙积年（积年 = 历元积年 + （年 − 历元年））。
#[must_use]
pub fn accumulated_years(year: i64) -> i64 {
    TAIYI_EPOCH_JINIAN + (year - TAIYI_EPOCH_YEAR)
}

/// 是否阳遁：冬至后（太阳黄经 `λ∈[270,360)∪[0,90)`）阳遁，夏至后阴遁。
#[must_use]
pub fn is_yang_dun(sun_longitude: f64) -> bool {
    let l = sun_longitude.rem_euclid(360.0);
    // 阳遁段 = 冬至(270)→夏至(90)，即不在 [90，270)（夏至→冬至 阴遁段）内。
    !(90.0..270.0).contains(&l)
}

/// 太乙行宫结果。
#[derive(Debug, Clone, Copy, Serialize)]
pub struct TaiyiPalace {
    /// 八宫序 `0..8`（沿阴阳遁方向的第几步）。
    pub step: u8,
    /// 洛书宫数（1..9，必非 5）。
    pub palace: u8,
    /// 该宫八卦名（复用洛书九宫）。
    pub gua: &'static str,
    /// 入宫年数 `1..=3`（该宫已居第几年）。
    pub year_in_palace: u8,
    /// 三才（理天/理地/理人）。
    pub sancai: &'static str,
}

/// 由积年与阴阳遁定太乙宫。`r = 积年 mod 24`，宫序 `⌊r/3⌋`，三才 `r mod 3`；阳顺阴逆。
#[must_use]
pub fn taiyi_palace(jinian: i64, yang_dun: bool) -> TaiyiPalace {
    let r = jinian.rem_euclid(24);
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "r∈0..24，step∈0..8，sancai∈0..3，窄化受控"
    )]
    let step = (r / 3) as usize;
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss, reason = "见上")]
    let sancai = (r % 3) as usize;
    // 阳遁顺行（一宫起），阴遁逆行（九宫起）= 顺行序的镜像。
    let palace = if yang_dun {
        PALACES_8[step]
    } else {
        PALACES_8[7 - step]
    };
    TaiyiPalace {
        step: step as u8,
        palace,
        gua: mingli_luoshu::PALACE_NAME[palace as usize],
        year_in_palace: sancai as u8 + 1,
        sancai: SANCAI[sancai],
    }
}

/// 一次太乙起局（确定部分）的结果。
#[derive(Debug, Clone, Serialize)]
pub struct Cast {
    /// 公历年。
    pub year: i64,
    /// 太乙积年。
    pub jinian: i64,
    /// 是否阳遁。
    pub yang_dun: bool,
    /// 太乙行宫。
    pub taiyi: TaiyiPalace,
}

/// 在共享上下文 [`Moment`] 上起太乙局（确定部分）。
#[must_use]
pub fn compute_at(m: &Moment) -> Cast {
    let year = i64::from(m.year);
    let jinian = accumulated_years(year);
    let yang = is_yang_dun(m.sun_longitude);
    Cast {
        year,
        jinian,
        yang_dun: yang,
        taiyi: taiyi_palace(jinian, yang),
    }
}

/// 由本地民用日期起局（独立入口）。
#[must_use]
pub fn compute(year: i32, month: u32, day: u32, tz: f64) -> Cast {
    compute_at(&Moment::new(year, month, day, 12, 0, tz))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jinian_epoch_anchor() {
        // 《太乙金镜式经》：724 CE = 积年 1_937_281。
        assert_eq!(accumulated_years(724), 1_937_281);
        // 线性外推：相邻年差 1。
        assert_eq!(accumulated_years(725), 1_937_282);
        assert_eq!(accumulated_years(723), 1_937_280);
    }

    #[test]
    fn palaces_skip_center_and_are_eight() {
        assert_eq!(PALACES_8.len(), 8);
        assert!(!PALACES_8.contains(&5), "太乙不入中五");
        let set: std::collections::HashSet<u8> = PALACES_8.iter().copied().collect();
        assert_eq!(set.len(), 8);
        // 顺行序：一宫起，1→2→3→4→6→7→8→9。
        assert_eq!(PALACES_8[0], 1);
        assert_eq!(PALACES_8[7], 9);
    }

    #[test]
    fn taiyi_dwells_three_years_per_palace_cycle_24() {
        // 三年居一宫、廿四年转一周：同一阳遁下，每 3 年宫序 +1，24 年回到原宫。
        for base in 0..24i64 {
            let p0 = taiyi_palace(base, true);
            // 入宫年数 1..3，三才随之。
            assert!((1..=3).contains(&p0.year_in_palace));
            assert_eq!(p0.sancai, SANCAI[(p0.year_in_palace - 1) as usize]);
            // 同宫连居三年。
            let same = taiyi_palace(base - (base % 3), true);
            let same2 = taiyi_palace(base - (base % 3) + 2, true);
            assert_eq!(same.palace, same2.palace, "同宫三年应同宫");
            // 24 年后复位。
            assert_eq!(taiyi_palace(base + 24, true).palace, p0.palace);
        }
        // 八宫在 24 年内恰好各被走到一次。
        let set: std::collections::HashSet<u8> = (0..24).step_by(3).map(|r| taiyi_palace(r, true).palace).collect();
        assert_eq!(set.len(), 8);
    }

    #[test]
    fn yin_dun_is_mirror_of_yang() {
        // 阴遁逆行：九宫起，与阳遁同步序号镜像。
        assert_eq!(taiyi_palace(0, false).palace, 9); // 阴遁一步起九宫
        for r in 0..24i64 {
            let y = taiyi_palace(r, true);
            let n = taiyi_palace(r, false);
            assert_eq!(n.palace, PALACES_8[7 - y.step as usize]);
            // 太乙恒不入中五。
            assert_ne!(y.palace, 5);
            assert_ne!(n.palace, 5);
        }
    }

    #[test]
    fn yang_yin_dun_by_solar_term() {
        // 冬至(λ=270)后阳遁、夏至(λ=90)后阴遁。
        assert!(is_yang_dun(270.0)); // 冬至
        assert!(is_yang_dun(0.0)); // 春分仍阳遁段
        assert!(!is_yang_dun(90.0)); // 夏至
        assert!(!is_yang_dun(180.0)); // 秋分阴遁段
        // 全 360° 各半。
        let yang = (0..360).filter(|&d| is_yang_dun(f64::from(d))).count();
        assert_eq!(yang, 180);
    }

    #[test]
    fn sixteen_gods_framework() {
        assert_eq!(SIXTEEN_DIRECTIONS.len(), 16);
        assert_eq!(SIXTEEN_GODS.len(), 16);
        // 十六方位含十二地支 + 四维卦。
        for s in ["子", "午", "卯", "酉", "艮", "巽", "坤", "乾"] {
            assert!(SIXTEEN_DIRECTIONS.contains(&s), "缺方位 {s}");
        }
        assert_eq!(SIXTEEN_GODS[0], "地主"); // 子神
    }

    #[test]
    fn compute_is_deterministic_and_palace_valid() {
        let c = compute(2024, 6, 15, 8.0);
        assert_eq!(c.jinian, accumulated_years(2024));
        assert!((1..=9).contains(&c.taiyi.palace) && c.taiyi.palace != 5);
        assert_eq!(c.taiyi.gua, mingli_luoshu::PALACE_NAME[c.taiyi.palace as usize]);
        let c2 = compute(2024, 6, 15, 8.0);
        assert_eq!(c.taiyi.palace, c2.taiyi.palace);
        assert_eq!(c.taiyi.sancai, c2.taiyi.sancai);
    }
}

//! 三宫：命宫 / 身宫 / 胎元的子平传统公式。

use super::*;

/// 三宫：命宫 / 身宫 / 胎元。子平传统 DET 公式。
///
/// **算法（各家通行）**：
/// - **命宫** = （月支 − 时支） mod 12 → 月支起子时**逆数**生时；天干由五虎遁（年干 → 命宫支）。
/// - **身宫** = （月支 + 时支） mod 12 → 月支起子时**顺数**生时；天干同样由五虎遁定。
/// - **胎元** = 月柱干 +1，月柱支 +3（怀胎十月范畴的「受气宫」）。
///
/// 注：命宫流派分歧——子平用**节气月支**（本算法），紫微/三命通会用**农历月**（数字 1..12）。
/// 本 crate 站在子平视角，与 [`mingli_ziwei`](crate::compute_with_true_solar) 的农历月命宫
/// 在「月支 ≠ 农历月对应支」的节气切换日会差一格。
#[derive(Debug, Clone, Serialize)]
pub struct ThreeHouses {
    /// 命宫干支（如「丙寅」）。
    pub ming_gong: String,
    /// 身宫干支（如「庚辰」）。
    pub shen_gong: String,
    /// 胎元干支（如「庚子」）。
    pub tai_yuan: String,
}

/// 取三宫：命宫/身宫/胎元（通行 DET 公式）。
#[must_use]
pub fn determine_three_houses(year_gz: GanZhi, month_gz: GanZhi, hour_b: u8) -> ThreeHouses {
    let ming_b = (month_gz.branch + 12 - hour_b) % 12;
    let ming_s = month_pillar_stem(year_gz.stem, ming_b);
    let shen_b = (month_gz.branch + hour_b) % 12;
    let shen_s = month_pillar_stem(year_gz.stem, shen_b);
    let tai_s = (month_gz.stem + 1) % 10;
    let tai_b = (month_gz.branch + 3) % 12;
    let fmt = |s: u8, b: u8| format!("{}{}", STEMS[s as usize], BRANCHES[b as usize]);
    ThreeHouses {
        ming_gong: fmt(ming_s, ming_b),
        shen_gong: fmt(shen_s, shen_b),
        tai_yuan: fmt(tai_s, tai_b),
    }
}

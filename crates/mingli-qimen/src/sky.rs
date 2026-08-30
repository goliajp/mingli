//! 天盘：九星与三奇六仪随值符旋转。
//!
//! 值符自「旬首宫」旋到「时干宫」，整盘沿后天八卦圆周作一次刚体旋转。
//! 中宫不在圆周上，取通行的寄坤 2。

/// 九星按九宫原配（地盘初始，未旋转）：蓬芮冲辅禽心柱任英，对应宫 1..=9。
/// 主流通行版 — 中宫天禽 🟡 寄坤 2（古本派寄艮 8）。
///
/// 索引 0 = 占位（从 1 起用，与宫号对齐）。
pub const JIU_XING_PALACE: [&str; 10] = [
    "", "天蓬", "天芮", "天冲", "天辅", "天禽", "天心", "天柱", "天任", "天英",
];

/// 天盘旋转所走的圆周宫序：后天八卦顺时针一周 —— 坎 1 → 艮 8 → 震 3 → 巽 4 → 离 9 →
/// 坤 2 → 兑 7 → 乾 6。中 5 不在圆周上（寄坤 2）。
///
/// 注意与地盘的走宫方式相区别：地盘按**宫序号 1→9 线性**铺三奇六仪，天盘则是整盘沿这条
/// **圆周**刚体旋转。
pub const ORBIT: [u8; 8] = [1, 8, 3, 4, 9, 2, 7, 6];

/// 中宫寄坤 2（主流寄宫法）：符首或时干落中 5 时按坤 2 论。
///
/// 🟡 古本另有寄艮 8 一派，本 crate 取通行版。
#[must_use]
pub const fn lodged_palace(palace: u8) -> u8 {
    if palace == 5 { 2 } else { palace }
}

/// 宫号在圆周 [`ORBIT`] 上的下标（中 5 按寄宫算）。
pub(crate) fn orbit_index(palace: u8) -> usize {
    let p = lodged_palace(palace);
    ORBIT.iter().position(|&x| x == p).unwrap_or(0)
}

/// 天盘：九星与三奇六仪随值符整体旋转后的结果。
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct SkyPlate {
    /// 旋转格数（沿 [`ORBIT`] 顺时针 0..=7）：值符从「旬首宫」走到「时干宫」的位移。
    pub shift: u8,
    /// 天盘九星，`stars[k]` = 第 `k+1` 宫；中 5 宫为空串（天禽寄坤 2，与天芮同宫）。
    pub stars: [&'static str; 9],
    /// 天盘天干，`stems[k]` = 第 `k+1` 宫；中 5 宫为空串。
    pub stems: [&'static str; 9],
    /// 地盘中 5 之干（寄坤 2，随坤 2 一同旋转）。
    pub center_stem: &'static str,
    /// 中宫寄干在天盘上的落宫 1..=9。
    pub center_palace: u8,
}

/// 天盘旋转（主流转盘法）。
///
/// 值符星原在**旬首六仪所在宫**，随时干走到**实际值符干所在宫**；整盘（九星 + 三奇六仪）
/// 沿 [`ORBIT`] 作同一次刚体旋转。地盘中 5 之干寄坤 2，坤 2 转到哪它就跟到哪
/// （等价于「随天芮星走」）。
///
/// 多源校验：以两则古例复现——阳遁三局丙寅时（旬首戊震 3、时干丙坎 1）与
/// 阳遁一局庚午时（旬首戊坎 1、时干庚震 3），两例天盘九星各 8 宫全中。
#[must_use]
pub fn sky_rotation(earth: &[&'static str; 9], xun_yi_palace: u8, zhi_fu_palace: u8) -> SkyPlate {
    let from = orbit_index(xun_yi_palace);
    let to = orbit_index(zhi_fu_palace);
    let shift = (to + 8 - from) % 8;
    let mut stars = [""; 9];
    let mut stems = [""; 9];
    for (i, &target) in ORBIT.iter().enumerate() {
        let src = ORBIT[(i + 8 - shift) % 8];
        stars[target as usize - 1] = JIU_XING_PALACE[src as usize];
        stems[target as usize - 1] = earth[src as usize - 1];
    }
    SkyPlate {
        shift: shift as u8,
        stars,
        stems,
        center_stem: earth[4],
        center_palace: ORBIT[(orbit_index(2) + shift) % 8],
    }
}

//! 人盘：八门。
//!
//! 值使门自旬首宫按**宫序号线性**数过本旬时辰位次落宫，八门再沿后天八卦圆周同步旋转。

use super::*;

/// 八门本位，与 [`ORBIT`] 同序：休坎 1 · 生艮 8 · 伤震 3 · 杜巽 4 · 景离 9 · 死坤 2 · 惊兑 7 · 开乾 6。
///
/// 八门的原配次序恰与后天八卦圆周重合，所以八门旋转与九星旋转是同一种刚体位移。
pub const BA_MEN_ORBIT: [&str; 8] = ["休门", "生门", "伤门", "杜门", "景门", "死门", "惊门", "开门"];

/// 人盘八门：值使门与其落宫，以及旋转后的八门分布。
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct GatePlate {
    /// 值使门 —— 旬首六仪所在宫的本位门（本旬不变）。
    pub zhi_shi_gate: &'static str,
    /// 值使门落宫 1..=9（落中 5 时按寄坤 2 归并）。
    pub zhi_shi_palace: u8,
    /// 占事时辰在本旬中的位次 0..=9（甲子旬的甲子时为 0）。
    pub steps: u8,
    /// 八门沿 [`ORBIT`] 的旋转格数 0..=7。
    pub shift: u8,
    /// 八门分布，`gates[k]` = 第 `k+1` 宫；中 5 宫为空串（八门不入中宫）。
    pub gates: [&'static str; 9],
}

/// 值使门与八门旋转（主流转盘法）。
///
/// 与九星「随时干」不同，值使**随时辰**走：自旬首六仪所在宫起，按**宫序号线性**
/// 阳遁 +1 / 阴遁 −1（数宫时中 5 也占一位）数过本旬内的时辰位次，落点即值使门所在宫；
/// 落到中 5 则按寄坤 2 论。其余七门再自值使宫起沿 [`ORBIT`] 圆周顺布。
///
/// 校验：阳遁一局庚午时（旬首戊坎 1、庚午为甲子旬第 7 个时辰）→ 休门数至兑 7，
/// 八门 坎1伤 艮8杜 震3景 巽4死 离9惊 坤2开 兑7休 乾6生，8 宫全中。
#[must_use]
pub fn gate_plate(xun_yi_palace: u8, head_branch: u8, time_branch: u8, yang_dun: bool) -> GatePlate {
    let steps = (time_branch + 12 - head_branch) % 12;
    let from = lodged_palace(xun_yi_palace);
    let delta = i32::from(steps) * if yang_dun { 1 } else { -1 };
    let landed = (i32::from(from) - 1 + delta).rem_euclid(9) + 1;
    let zhi_shi_palace = lodged_palace(u8::try_from(landed).unwrap_or(2));
    let start = orbit_index(from);
    let shift = (orbit_index(zhi_shi_palace) + 8 - start) % 8;
    let mut gates = [""; 9];
    for (i, &target) in ORBIT.iter().enumerate() {
        gates[target as usize - 1] = BA_MEN_ORBIT[(i + 8 - shift) % 8];
    }
    GatePlate {
        zhi_shi_gate: BA_MEN_ORBIT[start],
        zhi_shi_palace,
        steps,
        shift: shift as u8,
        gates,
    }
}

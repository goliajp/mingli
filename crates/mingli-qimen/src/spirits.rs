//! 神盘：八神。
//!
//! 直符与值符同宫，其余七神阳顺阴逆布外八宫。第 5 / 6 位的**称谓**两系不一
//! （白虎 / 玄武 与 勾陈 / 朱雀），位序则一致，故两名并出。

use super::*;

/// 八神次序：值符 · 腾蛇 · 太阴 · 六合 · 白虎 · 玄武 · 九地 · 九天。
///
/// 🟡 第 5 / 6 位的**称谓**有两系：一系两遁通用「白虎 / 玄武」（本 crate 取此），
/// 另一系阳遁称「勾陈 / 朱雀」、阴遁才称「白虎 / 玄武」（见 [`BA_SHEN_YANG_ALT`]）。
/// 两系只是名字不同，**位序一致**，故结构确定、命名留待定夺。
pub const BA_SHEN: [&str; 8] =
    ["值符", "腾蛇", "太阴", "六合", "白虎", "玄武", "九地", "九天"];

/// 另一系在**阳遁**下对第 5 / 6 位的称谓（勾陈 / 朱雀），其余六神同 [`BA_SHEN`]。
pub const BA_SHEN_YANG_ALT: [&str; 8] =
    ["值符", "腾蛇", "太阴", "六合", "勾陈", "朱雀", "九地", "九天"];

/// 神盘八神。
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct SpiritPlate {
    /// 值符（直符）所在宫 1..=9 —— 八神的起点，随天盘值符走。
    pub start_palace: u8,
    /// 八神分布，`spirits[k]` = 第 `k+1` 宫；中 5 宫为空串（八神不入中宫）。
    pub spirits: [&'static str; 9],
    /// 同上，但第 5 / 6 位用另一系称谓（阳遁作勾陈 / 朱雀）——两系位序相同，只是名字不同。
    pub spirits_alt: [&'static str; 9],
}

/// 八神布列：直符与值符同宫，其余七神自值符宫起沿 [`ORBIT`] 阳遁顺时针、阴遁逆时针依次落宫。
///
/// 校验：起点坎 1 的阳遁盘 → 坎1值符 艮8腾蛇 震3太阴 巽4六合 离9白虎 坤2玄武 兑7九地 乾6九天，
/// 8 宫全中（公开教程例题）。
///
/// 起点取**天盘**旬首所在宫（即值符宫）；另有一路「地八神」以地盘旬首宫为起点，本 crate 不出。
#[must_use]
pub fn spirit_plate(zhi_fu_palace: u8, yang_dun: bool) -> SpiritPlate {
    let start_palace = lodged_palace(zhi_fu_palace);
    let start = orbit_index(start_palace);
    let mut spirits = [""; 9];
    let mut spirits_alt = [""; 9];
    for k in 0..8 {
        let i = if yang_dun { (start + k) % 8 } else { (start + 8 - k) % 8 };
        let p = ORBIT[i] as usize - 1;
        spirits[p] = BA_SHEN[k];
        spirits_alt[p] = if yang_dun { BA_SHEN_YANG_ALT[k] } else { BA_SHEN[k] };
    }
    SpiritPlate { start_palace, spirits, spirits_alt }
}

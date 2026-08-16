//! 团队合盘：N 人五行画像、最缺最旺项与互补供给度。

use super::*;

/// 团队五行画像：N 个本命盘五行向量平均 → 团队整体五行分布(%)。
/// 用于「合盘 / 团队」：看整支队伍五行结构是否均衡。
#[must_use]
pub fn team_wuxing_average(members: &[BaziChart]) -> WuxingPower {
    if members.is_empty() {
        return WuxingPower { wood: 0, fire: 0, earth: 0, metal: 0, water: 0 };
    }
    let n = members.len() as u32;
    let sum_w = members.iter().map(|c| c.strength.wuxing.wood).sum::<u32>();
    let sum_f = members.iter().map(|c| c.strength.wuxing.fire).sum::<u32>();
    let sum_e = members.iter().map(|c| c.strength.wuxing.earth).sum::<u32>();
    let sum_m = members.iter().map(|c| c.strength.wuxing.metal).sum::<u32>();
    let sum_wa = members.iter().map(|c| c.strength.wuxing.water).sum::<u32>();
    WuxingPower {
        wood: sum_w / n,
        fire: sum_f / n,
        earth: sum_e / n,
        metal: sum_m / n,
        water: sum_wa / n,
    }
}

/// 互补度：成员 j 的盘中，成员 i 主用神五行的占比(%)。值越高 = j 越能补 i 的缺。
///
/// 用于团队互补矩阵 N×N 的每一格。
#[must_use]
pub fn complement_score(i_yongshen_wuxing: &str, j_wuxing: &WuxingPower) -> u32 {
    match i_yongshen_wuxing {
        "木" => j_wuxing.wood,
        "火" => j_wuxing.fire,
        "土" => j_wuxing.earth,
        "金" => j_wuxing.metal,
        "水" => j_wuxing.water,
        _ => 0,
    }
}

/// 五行索引 0..5 → 中文（用于按团队画像取最弱/最旺项的字面）。
pub(crate) const WX_NAMES: [&str; 5] = ["木", "火", "土", "金", "水"];

/// 取团队五行最弱的项 (name， pct)。
#[must_use]
pub fn team_weakest(wx: &WuxingPower) -> (String, u32) {
    let arr = [wx.wood, wx.fire, wx.earth, wx.metal, wx.water];
    let (idx, &v) = arr.iter().enumerate().min_by_key(|&(_, &v)| v).unwrap_or((0, &0));
    (WX_NAMES[idx].to_string(), v)
}

/// 取团队五行最旺的项 (name， pct)。
#[must_use]
pub fn team_strongest(wx: &WuxingPower) -> (String, u32) {
    let arr = [wx.wood, wx.fire, wx.earth, wx.metal, wx.water];
    let (idx, &v) = arr.iter().enumerate().max_by_key(|&(_, &v)| v).unwrap_or((0, &0));
    (WX_NAMES[idx].to_string(), v)
}

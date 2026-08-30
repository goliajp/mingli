//! 把盘上的落点翻成方位。
//!
//! 十二支与方位是确定映射；哪一路取用为吉各家不同，那属释义，不在这里合成排名。

use crate::Cast;

/// 十二地支所指方位（子北起顺时针）：子北 · 丑寅东北 · 卯东 · 辰巳东南 · 午南 · 未申西南 · 酉西 · 戌亥西北。
pub const BRANCH_DIR: [&str; 12] =
    ["北", "东北", "东北", "东", "东南", "东南", "南", "西南", "西南", "西", "西北", "西北"];

/// 十二地支名（子起）。
pub const BRANCH_NAMES: [&str; 12] =
    ["子", "丑", "寅", "卯", "辰", "巳", "午", "未", "申", "酉", "戌", "亥"];

/// 由一课六壬抽出方位候选：三传各支所指之方。
///
/// 九宗门取传已全覆盖，故三传恒有；此处仍保留「无传则取四课上神」的退路，
/// 是为了万一将来新增课式而尚未定取传时，寻方位不至于整片空掉。
#[must_use]
pub fn bearings_of(cast: &Cast) -> Vec<mingli_contract::Bearing> {
    let mk = |element: String, branch: u8, note: String| mingli_contract::Bearing {
        leaf: "liuren",
        element,
        at: BRANCH_NAMES[branch as usize % 12].to_string(),
        direction: BRANCH_DIR[branch as usize % 12],
        note,
    };
    if let Some(tr) = cast.transmission {
        return tr
            .iter()
            .enumerate()
            .map(|(i, &b)| {
                mk(
                    ["初传", "中传", "末传"][i.min(2)].to_string(),
                    b,
                    format!("课式 {}", cast.pattern_label),
                )
            })
            .collect();
    }
    cast.courses
        .iter()
        .enumerate()
        .map(|(i, c)| {
            mk(
                format!("第{}课上神", i + 1),
                c.up,
                format!("课式 {} · 🟡 取传未出", cast.pattern_label),
            )
        })
        .collect()
}

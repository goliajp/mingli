//! 从盘上读方位：奇门这一侧。
//!
//! 「寻方位」要的候选是：值符宫、值使宫、三吉门所在宫、三奇所在宫。
//! 判据用本 crate 自己的 [`JI_MEN`] 与 [`SAN_QI`]，不在用例层另抄一份字面量。

use super::*;
use mingli_contract::Bearing;

/// 某宫的方位（复用洛书九宫方位；中 5 无方位）。
#[must_use]
pub fn direction_of(palace: u8) -> &'static str {
    mingli_luoshu::PALACE_DIR[(palace as usize).min(9)]
}

/// 某宫的字面（如「坎1」）。
#[must_use]
pub fn palace_label(cast: &Cast, palace: u8) -> String {
    let k = palace as usize - 1;
    cast.palace.get(k).map_or_else(String::new, |n| format!("{n}{palace}"))
}

/// 某宫的同宫结构：天盘星 + 旺衰 · 八门 · 八神 · 天盘干。判读的依据，本层不下断语。
#[must_use]
pub fn palace_note(cast: &Cast, palace: u8) -> String {
    let k = palace as usize - 1;
    format!(
        "{}{} · {} · {} · 天盘{}",
        cast.sky.stars[k], cast.star_vigor[k], cast.gates.gates[k], cast.spirits.spirits[k], cast.sky.stems[k]
    )
}

/// 中宫的宫号。它不在圆周上，星门神俱不入。
const CENTER: u8 = 5;
/// 中宫所寄之宫：坤 2。见本叶 `profile()` 的「天禽寄坤 2（两遁通用）」一条。
const JI_PALACE: u8 = 2;

/// 由一张奇门盘抽出全部方位候选。
#[must_use]
pub fn bearings_of(cast: &Cast) -> Vec<Bearing> {
    // 中 5 不在后天八卦圆周上，也不是可面向的方位；星门神一概不入中宫，
    // 于是「落中五」的候选会带着一条四段全空的附注与一个没法面向的「中」出门。
    // 本叶已把「中 5 寄坤 2」定为 Det（两源，见 `profile()`），值使门落中五时
    // `gates.rs` 正是这么归并的——方位候选照同一条办。落点字面把两端都写出来，
    // 不把「它本在中宫」这件事藏掉。
    let mk = |element: String, palace: u8| {
        let (at, dest) = if palace == CENTER {
            (format!("{}寄{}", palace_label(cast, CENTER), palace_label(cast, JI_PALACE)), JI_PALACE)
        } else {
            (palace_label(cast, palace), palace)
        };
        Bearing {
            leaf: "qimen",
            element,
            at,
            direction: direction_of(dest),
            note: palace_note(cast, dest),
        }
    };
    let mut out = vec![
        mk("值符".to_string(), cast.zhi_fu_palace),
        mk("值使".to_string(), cast.gates.zhi_shi_palace),
    ];
    for (k, gate) in cast.gates.gates.iter().enumerate() {
        if JI_MEN.contains(gate) {
            out.push(mk((*gate).to_string(), u8::try_from(k + 1).unwrap_or(1)));
        }
    }
    for (k, stem) in cast.sky.stems.iter().enumerate() {
        if SAN_QI.contains(stem) {
            out.push(mk(format!("{stem}奇"), u8::try_from(k + 1).unwrap_or(1)));
        }
    }
    // 中宫之干寄坤 2 随转，不在 sky.stems 里；若它恰是三奇，方位取其实际落宫
    if SAN_QI.contains(&cast.sky.center_stem) {
        out.push(mk(format!("{}奇（中宫寄）", cast.sky.center_stem), cast.sky.center_palace));
    }
    out
}

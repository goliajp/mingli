//! 旺相休囚死：以节气月令衡量九星与八门的五行。
//!
//! 《五行大义》通行五档——当令者旺、令所生者相、生令者休、克令者囚、令所克者死。

use super::*;

/// 九星五行，与宫号对齐（索引 0 占位）：蓬水 · 芮土 · 冲木 · 辅木 · 禽土 · 心金 · 柱金 · 任土 · 英火。
pub const JIU_XING_ELEMENT: [Element; 10] = [
    Element::Earth,
    Element::Water,
    Element::Earth,
    Element::Wood,
    Element::Wood,
    Element::Earth,
    Element::Metal,
    Element::Metal,
    Element::Earth,
    Element::Fire,
];

/// 五行在月令下的强弱五等级。
///
/// 取《五行大义》以来的通行判法：**当令者旺、令生者相、生令者休、克令者囚、令克者死**。
/// （另有一路以星为主体、含「废」的五等级说法，非通行，本 crate 不取。）
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum Vigor {
    /// 旺：与月令同五行，当令。
    Wang,
    /// 相：月令所生，次旺。
    Xiang,
    /// 休：生月令者，气已泄。
    Xiu,
    /// 囚：克月令者，反受制。
    Qiu,
    /// 死：月令所克，气尽。
    Si,
}

impl Vigor {
    /// 中文标签。
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Vigor::Wang => "旺",
            Vigor::Xiang => "相",
            Vigor::Xiu => "休",
            Vigor::Qiu => "囚",
            Vigor::Si => "死",
        }
    }
}

/// 由节气序号取节气月支 0..=11（0 = 子）。每个「节」开启一个月，故两气一支。
///
/// [`SOLAR_TERMS`] 自春分（黄经 0°）起排，立春在 21 位开寅月。
#[must_use]
pub fn month_branch_of_term(term_index: usize) -> u8 {
    let k = ((term_index + 3) % 24) / 2;
    u8::try_from((k + 2) % 12).unwrap_or(0)
}

/// 地支五行（寅卯木 · 巳午火 · 申酉金 · 亥子水 · 辰戌丑未土）。
#[must_use]
pub fn branch_element_of(branch: u8) -> Element {
    mingli_ganzhi::branch_element(branch)
}

/// 判某五行在给定月令下的旺相休囚死。
#[must_use]
pub fn vigor_of(subject: Element, month: Element) -> Vigor {
    if subject == month {
        Vigor::Wang
    } else if month.generates() == subject {
        Vigor::Xiang
    } else if subject.generates() == month {
        Vigor::Xiu
    } else if subject.controls() == month {
        Vigor::Qiu
    } else {
        Vigor::Si
    }
}

/// 由星名取其五行（未知星名返回 `None`）。
#[must_use]
pub fn star_element(name: &str) -> Option<Element> {
    (1..=9).find(|&p| JIU_XING_PALACE[p] == name).map(|p| JIU_XING_ELEMENT[p])
}

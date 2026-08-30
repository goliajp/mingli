//! L3 叶（C 族）：抽牌占卜（塔罗 / Lenormand / 卢恩等多种 Deck）。
//!
//! 共同机制 = **无放回抽取** + 可选**正逆位**：把一副 `deck_size` 张牌用可复现种子
//! （[`mingli_core::sampler`] 的 Fisher-Yates 洗牌）洗成一个置换，取前 `count` 张即得牌阵；
//! 允许逆位时每张牌再附一个可复现的方向位。不同占卜体系的差别只在牌副大小与是否用逆位——
//! 本 crate 把这一点显式表达为 [`Deck`] 流派枚举。
//!
//! 支持的 deck（流派）：
//! - **Tarot 78**（默认，Major 22 + Minor 56，允许逆位）
//! - **Tarot 大阿卡纳 22**（仅 Major Arcana，允许逆位）
//! - **Lenormand**（Petit Lenormand 36 张，传统**不用逆位**）
//! - **Elder Futhark**（24 卢恩字符，允许逆位；部分卢恩对称、逆位无意义留待释义层）
//! - **Younger Futhark**（维京时期 16 字符简化卢恩，允许逆位）
//!
//! **牌名表**：各 deck 完整牌名经多源校验入码，不凭记忆硬编。
//!
//! - **Tarot Major 22**：英文(en.wikipedia Major_Arcana)+ 中文通行译名(zh.wikipedia 大阿尔克那 +
//!   Biddy Tarot + Labyrinthos 三源一致)；**TarotOrder enum** 暴露 Rider-Waite-Smith(默认，8=Strength
//!   /11=Justice) vs Marseilles(8=Justice/11=Strength) 流派分歧 — Golden Dawn 占星机理 8↔Leo/11↔Libra。
//! - **Tarot Minor 56**：由 4 花色(Wands/Cups/Swords/Pentacles) × 14 等级(Ace， 2..10， Page， Knight， Queen， King)
//!   程序生成，无大查表；中文取 zh.wikipedia 小阿尔克那主流（Pentacles=钱币、Queen=王后）。
//! - **Lenormand Petit 36**：英文 4 源完全一致(globalspiritualstudies / learnlenormand / tarotwhisper / lenormand.tw)。
//! - **Elder Futhark 24** / **Younger Futhark 16**：英文 + Unicode 字符（BabelStone + en.wikipedia + Runic block 三源一致），
//!   中文译名多家拼写不稳标 🟡 不入码；只入古北欧名 + Unicode。

#![allow(
    clippy::wildcard_imports,
    reason = "叶内各模块以 `use super::*` 共享 crate 顶层的领域 import——这是把一张大盘拆成多文件的常规手法"
)]

#[cfg(feature = "port")]
mod engine;
#[cfg(feature = "port")]
pub use engine::TarotEngine;

use mingli_core::sampler::{shuffle, SplitMix64};
#[cfg(feature = "serde")]
use serde::Serialize;

pub mod decks;
pub mod names;
pub mod draw;
#[cfg(test)]
mod tests;

// 全部出口在 crate 根平铺——拆成多文件是内部组织，对外仍是一片叶。
pub use decks::*;
pub use names::*;
pub use draw::*;

//! 一片叶对自己的声明：属于哪个计算家族、哪些地方算得准、有哪些流派。
//!
//! 这一组是「供给侧」的自述——与 [`crate::intent`] 的需求侧对偶：
//! 那边说「有哪几类问局」，这边说「这片叶答得起什么、答到什么程度」。

use serde::Serialize;


/// 计算家族。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub enum Family {
    /// A 循环群 / CRT（时间→模运算）。
    Cyclic,
    /// B 角度量化（星历→黄经→分段）。
    Angular,
    /// C 抽样 / 二进制（熵→有限格）。
    Sampling,
    /// D 哈希环（字符串→数→约化）。
    Hashing,
    /// ⟂ 飞布 / 横切（群作用）。
    CrossCutting,
}

impl Family {
    /// 家族中文标签（承接层展示用）。
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Family::Cyclic => "循环群/CRT",
            Family::Angular => "角度量化",
            Family::Sampling => "抽样/二进制",
            Family::Hashing => "哈希环",
            Family::CrossCutting => "飞布/横切",
        }
    }
}

/// 确定性谱：标注一项计算是确定算的、随机可复现的、还是流派欠定。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Determinism {
    /// 🟢 确定：由 Rust 确定性算出（多校验权威值/已知量）。
    Det,
    /// 🎲 随机·种子可复现：抽样/起卦，给定种子可复现。
    Sto,
    /// 🟡 欠定：流派分歧 / 待权威校验 / 大查表未取证——引擎诚实留空而非臆造。
    Und,
}

impl Determinism {
    /// 中文标签。
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Determinism::Det => "确定",
            Determinism::Sto => "随机·种子可复现",
            Determinism::Und => "欠定",
        }
    }
}

/// 确定性谱的一项：某计算方面的确定性等级与说明。
#[derive(Debug, Clone, Copy, Serialize)]
pub struct DetItem {
    /// 计算方面（如「四柱」「Asc/MC」「三传」）。
    pub aspect: &'static str,
    /// 确定性等级。
    pub status: Determinism,
    /// 一句说明（校验依据 / 为何欠定）。
    pub note: &'static str,
}

/// 构造 [`DetItem`] 的简写。
#[must_use]
pub const fn d(aspect: &'static str, status: Determinism, note: &'static str) -> DetItem {
    DetItem { aspect, status, note }
}

/// 叶的一个流派。
#[derive(Debug, Clone, Copy, Serialize)]
pub struct SchoolItem {
    /// 流派稳定 id（代码内使用，小写英数）。
    pub id: &'static str,
    /// 显示名（承接层展示）。
    pub name: &'static str,
    /// 是否默认流派（每叶应恰有一个默认）。
    pub default: bool,
    /// 一句说明：差异点 / 校验依据 / 流派归属。
    pub note: &'static str,
}

/// 构造 [`SchoolItem`] 的简写。
#[must_use]
pub const fn s(id: &'static str, name: &'static str, default: bool, note: &'static str) -> SchoolItem {
    SchoolItem { id, name, default, note }
}

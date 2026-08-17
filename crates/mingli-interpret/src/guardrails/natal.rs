//! 本命解盘的护栏与读法提示。
//!
//! 与其余六套不同：本命盘吃的是 [`mingli_contract::LeafOutput`] 结构体而非 JSON 串，
//! 且要带上确定性谱与主体重映射，故它的组装不走 [`crate::Prompt`]。
//!
//! 「这片叶的字段各是什么意思」「换个主体怎么读」不在这里——那是各叶自己的领域知识，
//! 由 [`mingli_contract::CastingEngine::reading_notes`] 与 `subject_notes` 声明，本层原样转交。

use mingli_contract::Determinism;

/// 护栏系统指令（所有释义共享）。
pub const GUARDRAIL: &str = "你是术数释义助手。下面是【已由确定性引擎算好】的一片盘面。规则：\
1) 只解释，绝不修改、重算或新增任何数字与名称；\
2) 标 DET 的部分忠实转述其含义；标 UND（欠定） 的部分须明说『此处流派分歧、引擎诚实留空』，不要替它杜撰；\
3) **可以给出吉凶 / 喜忌 / 适合 / 不适合 / 有利 / 不利 等评估**，结合结构事实（强/弱、用神/忌神、格局、神煞、星曜组合）给出基于传统命理推断的建议；评估须有依据（简述基于哪几项结构），避免空泛断言；\
4) 250 字以内，简体中文，结尾标注「仅供研究与娱乐」。";

/// 确定性等级的中文标记。
pub(crate) fn det_mark(s: Determinism) -> &'static str {
    match s {
        Determinism::Det => "DET 确定",
        Determinism::Sto => "STO 随机·可复现",
        Determinism::Und => "UND 欠定🟡",
    }
}

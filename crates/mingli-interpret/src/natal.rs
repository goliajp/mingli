//! 本命解盘：由一片叶的输出组装带确定性谱与主体重映射的提示词。
//!
//! 与其余六套意图不同，本命吃的是 [`LeafOutput`] 结构体而不是 JSON 串，
//! 还要把确定性谱逐条铺进提示词里，故不走 [`crate::Prompt`] 的四段框。

use super::guardrails::natal::{det_mark, GUARDRAIL};
use super::{Interpretation, Interpreter};
pub use mingli_contract::Subject;

use mingli_contract::{CastingEngine, LeafOutput};

/// 由一片叶组装带护栏的释义提示词（确定性，可校验）。默认主体 = Person。
#[must_use]
pub fn build_prompt(e: &dyn CastingEngine, leaf: &LeafOutput) -> String {
    build_prompt_with_subject(e, leaf, Subject::Person)
}

/// 同 [`build_prompt`]，但显式指定主体类型；非 Person 时附加主体重映射段。
#[must_use]
pub fn build_prompt_with_subject(e: &dyn CastingEngine, leaf: &LeafOutput, subject: Subject) -> String {
    let mut s = String::with_capacity(512);
    s.push_str(GUARDRAIL);
    if subject != Subject::Person {
        s.push_str("\n【主体类型】本盘的主体不是「人」而是「");
        s.push_str(subject.cn());
        s.push_str("」 — 请按下方主体重映射读宫位/十神；计算层（干支/五行/旺衰/用神）不变，只换象义。");
    }
    s.push_str("\n\n【系统】");
    s.push_str(leaf.name);
    s.push('（');
    s.push_str(leaf.family_label);
    s.push_str(" 族）\n【确定性谱】\n");
    for it in leaf.profile {
        s.push_str("- [");
        s.push_str(det_mark(it.status));
        s.push_str("] ");
        s.push_str(it.aspect);
        s.push('：');
        s.push_str(it.note);
        s.push('\n');
    }
    s.push_str("【盘面 JSON】\n");
    s.push_str(&serde_json::to_string(&leaf.chart).unwrap_or_default());
    // 读法与主体重映射都问这片叶自己要——释义层不攒各叶的字段词典。
    if let Some(hint) = e.reading_notes() {
        s.push_str(hint);
    }
    if let Some(hint) = e.subject_notes(subject) {
        s.push_str(hint);
    }
    s
}

/// 用给定后端释义一片叶。
///
/// # Errors
/// 后端 `interpret` 失败时透传错误。
pub fn interpret_leaf(
    it: &dyn Interpreter,
    e: &dyn CastingEngine,
    leaf: &LeafOutput,
) -> std::io::Result<Interpretation> {
    interpret_leaf_with_subject(it, e, leaf, Subject::Person)
}

/// 释义一片叶，显式指定主体类型。
///
/// # Errors
/// 后端 `interpret` 失败时透传错误。
pub fn interpret_leaf_with_subject(
    it: &dyn Interpreter,
    e: &dyn CastingEngine,
    leaf: &LeafOutput,
    subject: Subject,
) -> std::io::Result<Interpretation> {
    let text = it.interpret(&build_prompt_with_subject(e, leaf, subject))?;
    Ok(Interpretation {
        leaf: leaf.id.to_string(),
        text,
        backend: it.backend(),
        kind: "INT",
    })
}

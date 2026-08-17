//! 本命解盘：由一片叶的输出组装带确定性谱与主体重映射的提示词。
//!
//! 与其余六套意图不同，本命吃的是 [`LeafOutput`] 结构体而不是 JSON 串，
//! 还要把确定性谱逐条铺进提示词里，故不走 [`crate::Prompt`] 的四段框。

use super::guardrails::natal::{det_mark, semantic_hints, subject_hints, GUARDRAIL};
use super::{Interpretation, Interpreter};
use mingli_contract::LeafOutput;

/// 主体类型：同一套四柱计算给不同主体读出不同象义。
///
/// **计算层完全 DET 同源**（干支/五行/十神/旺衰对任何主体一致）；
/// **只解读层换映射**。person 是默认；company/product/event 适配「物有时刻 → 八字」（择日的逆运算）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Subject {
    /// 人（默认）：传统人盘。年=祖根、月=父母青年、日=自身/配偶、时=子女晚年。
    Person,
    /// 公司/组织：年=创立根基/行业属性、月=成长环境/团队、日=主体/核心、时=前景/产出。
    Company,
    /// 物（有时刻发布的产品/建筑/开张）：同公司盘（择日的镜像）。
    Product,
    /// 事（已发生事件）：用于复盘事的性质与走向。
    Event,
}

impl Subject {
    /// 从字符串解析(`"person"/"company"/"product"/"event"`)。
    #[must_use]
    pub fn from_str_opt(s: &str) -> Option<Self> {
        match s {
            "person" | "人" => Some(Self::Person),
            "company" | "公司" => Some(Self::Company),
            "product" | "object" | "物" | "产品" => Some(Self::Product),
            "event" | "事" => Some(Self::Event),
            _ => None,
        }
    }
    /// 中文展示名。
    #[must_use]
    pub fn cn(self) -> &'static str {
        match self {
            Self::Person => "人",
            Self::Company => "公司/组织",
            Self::Product => "物/产品",
            Self::Event => "事/事件",
        }
    }
}

/// 由一片叶组装带护栏的释义提示词（确定性，可校验）。默认主体 = Person。
#[must_use]
pub fn build_prompt(leaf: &LeafOutput) -> String {
    build_prompt_with_subject(leaf, Subject::Person)
}

/// 同 [`build_prompt`]，但显式指定主体类型；非 Person 时附加主体重映射段。
#[must_use]
pub fn build_prompt_with_subject(leaf: &LeafOutput, subject: Subject) -> String {
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
    if let Some(hint) = semantic_hints(leaf.id) {
        s.push_str(hint);
    }
    if let Some(hint) = subject_hints(subject, leaf.id) {
        s.push_str(hint);
    }
    s
}

/// 用给定后端释义一片叶。
///
/// # Errors
/// 后端 `interpret` 失败时透传错误。
pub fn interpret_leaf(it: &dyn Interpreter, leaf: &LeafOutput) -> std::io::Result<Interpretation> {
    interpret_leaf_with_subject(it, leaf, Subject::Person)
}

/// 释义一片叶，显式指定主体类型。
///
/// # Errors
/// 后端 `interpret` 失败时透传错误。
pub fn interpret_leaf_with_subject(
    it: &dyn Interpreter,
    leaf: &LeafOutput,
    subject: Subject,
) -> std::io::Result<Interpretation> {
    let text = it.interpret(&build_prompt_with_subject(leaf, subject))?;
    Ok(Interpretation {
        leaf: leaf.id.to_string(),
        text,
        backend: it.backend(),
        kind: "INT",
    })
}

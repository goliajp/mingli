//! L4 释义层：把确定性引擎算好的盘面组装成**带护栏**的提示词，交给可替换的后端出文字。
//!
//! 本层不算命——一个数都不改、一个名都不换。它做的是两件事：
//!
//! 1. **护栏**：告诉释义者「只读盘面、绝不重算」，并把确定性谱里标 🟡 的部分原样交出去，
//!    要求它照实说「此处流派分歧、引擎诚实留空」而不是替引擎杜撰。
//! 2. **组装**：护栏 → 读法提示 → 盘面 JSON → 尾部提示，四段的框由 [`Prompt`] 统一负责。
//!
//! 七套意图分两类：本命解盘（[`natal`]，吃 [`mingli_contract::LeafOutput`] 结构体、要铺确定性谱与主体重映射）
//! 与六套吃 JSON 的（[`intents`]）。护栏正文一意图一文件，见 [`guardrails`]。
//!
//! 无 LLM 时有离线兜底 [`Template`]：不调任何模型，忠实把确定性谱转成一段话——
//! 诚实且可校验，测试就是拿它跑的。

pub mod guardrails;
pub mod intents;
pub mod natal;

mod backend;
mod prompt;
#[cfg(test)]
mod tests;

pub use backend::{Interpretation, Interpreter, Template};
pub use intents::{
    build_election_prompt, build_event_prompt, build_locative_prompt, build_mundane_prompt,
    build_synastry_prompt, build_team_prompt, interpret_election, interpret_event,
    interpret_locative, interpret_mundane, interpret_synastry, interpret_team,
};
pub use natal::{
    build_prompt, build_prompt_with_subject, interpret_leaf, interpret_leaf_with_subject, Subject,
};
pub use prompt::Prompt;

// 护栏常量在 crate 根平铺，保持既有引用路径不破。
pub use guardrails::election::ELECTION_GUARDRAIL;
pub use guardrails::event::EVENT_GUARDRAIL;
pub use guardrails::locative::LOCATIVE_GUARDRAIL;
pub use guardrails::mundane::MUNDANE_GUARDRAIL;
pub use guardrails::natal::GUARDRAIL;
pub use guardrails::synastry::SYNASTRY_GUARDRAIL;
pub use guardrails::team::TEAM_GUARDRAIL;

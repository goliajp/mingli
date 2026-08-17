//! 六套「吃 JSON」的意图：占事 / 择吉 / 寻方位 / 合盘 / 国运 / 团队。
//!
//! 每套只声明自己不同的三件事——护栏、读法提示、JSON 抬头——其余的框由
//! [`Prompt`] 统一负责；`interpret_*` 六个函数此前逐字同构，现在收成一个私有的 `finish`。

use super::guardrails::{election, event, locative, mundane, synastry, team};
use super::{Interpretation, Interpreter, Prompt};

/// 把一份提示词交给后端，包成 [`Interpretation`]。
///
/// 六个 `interpret_*` 此前的函数体逐字相同，只差 `leaf` 的字面量与所调的 builder。
fn finish(it: &dyn Interpreter, leaf: &str, prompt: &str) -> std::io::Result<Interpretation> {
    Ok(Interpretation {
        leaf: leaf.to_string(),
        text: it.interpret(prompt)?,
        backend: it.backend(),
        kind: "INT",
    })
}

/// 声明一套意图：护栏 + 可选读法提示 + JSON 抬头 → builder + interpreter 两个公开函数。
macro_rules! intent {
    (
        $(#[$meta:meta])*
        $id:literal, $build:ident, $interpret:ident,
        guardrail = $guardrail:expr,
        $(hints = $hints:expr,)?
        json = $header:literal
        $(, trailer = $trailer:expr)?
    ) => {
        $(#[$meta])*
        #[must_use]
        pub fn $build(json: &str) -> String {
            #[allow(unused_mut, reason = "hints / trailer 是可选段，不是每套意图都有")]
            let mut p = Prompt::new($guardrail);
            $( p = p.hints($hints); )?
            p = p.json($header, json);
            p = p.trailer("\n");
            $( p = p.trailer($trailer); )?
            p.render()
        }

        #[doc = concat!("释义一次「", $id, "」，返回 `Interpretation { leaf: \"", $id, "\" }`。")]
        ///
        /// # Errors
        ///
        /// 释义后端不可用时返回其 I/O 错误。
        pub fn $interpret(it: &dyn Interpreter, json: &str) -> std::io::Result<Interpretation> {
            finish(it, $id, &$build(json))
        }
    };
}

intent! {
    /// 组装占事释义提示词（护栏 + 读法 + 占事 JSON）。
    "event", build_event_prompt, interpret_event,
    guardrail = event::EVENT_GUARDRAIL,
    hints = event::event_hints(),
    json = "\n占事结果 JSON：\n"
}

intent! {
    /// 组装择吉释义提示词（护栏 + 读法 + 择吉 JSON）。
    "election", build_election_prompt, interpret_election,
    guardrail = election::ELECTION_GUARDRAIL,
    hints = election::election_hints(),
    json = "\n择吉结果 JSON：\n"
}

intent! {
    /// 组装寻方位释义提示词（护栏 + 读法 + 寻方位 JSON）。
    "locative", build_locative_prompt, interpret_locative,
    guardrail = locative::LOCATIVE_GUARDRAIL,
    hints = locative::locative_hints(),
    json = "\n寻方位结果 JSON：\n"
}

intent! {
    /// 组装合盘释义提示词。
    "synastry", build_synastry_prompt, interpret_synastry,
    guardrail = synastry::SYNASTRY_GUARDRAIL,
    json = "\n合盘结果 JSON：\n"
}

intent! {
    /// 组装国运释义提示词。
    "mundane", build_mundane_prompt, interpret_mundane,
    guardrail = mundane::MUNDANE_GUARDRAIL,
    hints = mundane::mundane_hints(),
    json = "\n国运结果 JSON：\n"
}

intent! {
    /// 由团队合盘结果（JSON 形式）组装团队释义提示词。
    ///
    /// 输入是 `/api/team` 端点返回的完整 JSON（含 members / team_wuxing /
    /// team_weakest / team_strongest / complement_matrix）。由调用方序列化好直接传入，
    /// 本函数不解析也不假设结构。
    ///
    /// 这一套的字段提示在 JSON **之后**，故走 trailer 而非 hints。
    "team", build_team_prompt, interpret_team,
    guardrail = team::TEAM_GUARDRAIL,
    json = "\n\n【团队合盘 JSON】\n",
    trailer = team::TEAM_FIELD_HINTS
}

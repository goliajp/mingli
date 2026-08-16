//! 释义用例：算出盘面 → 组装带护栏的提示词 → 交给可替换的后端。
//!
//! 后端实现（claude CLI / 离线模板 / 将来的 HTTP LLM）属于外层，由调用方注入；
//! 这里只负责「先算哪片叶、用哪种主体象义、失败回退到哪」这条编排。

use mingli_contract::{CastingEngine, Query};
use mingli_interpret::{Interpretation, Interpreter, Subject, Template};

/// 释义一片叶：先只算该叶（省去其余叶），再送释义后端；后端失败回退离线模板。
///
/// # Errors
///
/// 叶 id 不在注册表内，或连离线模板都失败时返回错误说明。
pub fn leaf(
    reg: &[Box<dyn CastingEngine>],
    backend: &dyn Interpreter,
    leaf_id: &str,
    q: &Query,
    subject: Subject,
) -> Result<Interpretation, String> {
    let leaf = mingli_engine::cast_one(reg, leaf_id, q).ok_or_else(|| format!("未知叶 {leaf_id}"))?;
    mingli_interpret::interpret_leaf_with_subject(backend, &leaf, subject)
        .or_else(|_| mingli_interpret::interpret_leaf_with_subject(&Template, &leaf, subject))
        .map_err(|e| format!("释义后端不可用：{e}"))
}

/// 释义团队合盘：同样是「后端失败回退离线模板」。
///
/// # Errors
///
/// 连离线模板都失败时返回错误说明。
pub fn team(backend: &dyn Interpreter, team_json: &str) -> Result<Interpretation, String> {
    mingli_interpret::interpret_team(backend, team_json)
        .or_else(|_| mingli_interpret::interpret_team(&Template, team_json))
        .map_err(|e| format!("释义后端不可用：{e}"))
}

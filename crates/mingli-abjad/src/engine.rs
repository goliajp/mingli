//! 本叶对 [`mingli_contract::WordEngine`] 的实现——字/词模态不吃出生时刻，
//! 只吃文字或笔画，因此走与 `CastingEngine` 平行的第二条契约。

use mingli_contract::{WordEngine, WordQuery};
use serde_json::Value;

/// 阿拉伯 abjad叶的字词入口。
#[derive(Debug, Default)]
pub struct AbjadEngine;

impl WordEngine for AbjadEngine {
    fn id(&self) -> &'static str {
        "abjad"
    }
    fn name(&self) -> &'static str {
        "阿拉伯 abjad"
    }
    fn compute(&self, q: &WordQuery) -> Result<Value, String> {
        let w = q.text.clone().unwrap_or_default();
        Ok(serde_json::json!({ "system": "abjad", "input": w, "result": crate::compute(&w) }))
    }
}

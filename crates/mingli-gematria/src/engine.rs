//! 本叶对 [`mingli_contract::WordEngine`] 的实现——字/词模态不吃出生时刻，
//! 只吃文字或笔画，因此走与 `CastingEngine` 平行的第二条契约。

use mingli_contract::{WordEngine, WordQuery};
use serde_json::Value;

/// 希伯来 gematria叶的字词入口。
#[derive(Debug, Default)]
pub struct GematriaEngine;

impl WordEngine for GematriaEngine {
    fn id(&self) -> &'static str {
        "gematria"
    }
    fn name(&self) -> &'static str {
        "希伯来 gematria"
    }
    fn compute(&self, q: &WordQuery) -> Result<Value, String> {
        let w = q.text.clone().unwrap_or_default();
        Ok(serde_json::json!({ "system": "gematria", "input": w, "result": crate::compute(&w) }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 适配器把本叶接到字词契约上：元数据齐备，输入齐备时能取值。
    #[test]
    fn adapter_is_wired_to_the_contract() {
        let e = GematriaEngine;
        assert!(!e.id().is_empty() && !e.name().is_empty());
        let q = WordQuery {
            text: Some("שלום".to_string()),
            surname: Some(vec![7]),
            given: Some(vec![16, 9]),
        };
        let v = e.compute(&q).expect("输入齐备应能取值");
        assert_eq!(v["system"], e.id());
    }
}

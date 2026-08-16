//! 本叶对 [`mingli_contract::WordEngine`] 的实现——字/词模态不吃出生时刻，
//! 只吃文字或笔画，因此走与 `CastingEngine` 平行的第二条契约。

use mingli_contract::{WordEngine, WordQuery};
use serde_json::Value;

/// 姓名五格叶的字词入口。
#[derive(Debug, Default)]
pub struct WugeEngine;

impl WordEngine for WugeEngine {
    fn id(&self) -> &'static str {
        "wuge"
    }
    fn name(&self) -> &'static str {
        "姓名五格"
    }
    fn compute(&self, q: &WordQuery) -> Result<Value, String> {
        let s = q.surname.clone().unwrap_or_default();
        let g = q.given.clone().unwrap_or_default();
        if s.is_empty() || g.is_empty() {
            return Err("姓与名笔画至少各一字".to_string());
        }
        Ok(serde_json::json!({ "system": "wuge", "surname": s, "given": g, "result": crate::five_grids(&s, &g) }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 适配器把本叶接到字词契约上：元数据齐备，输入齐备时能取值。
    #[test]
    fn adapter_is_wired_to_the_contract() {
        let e = WugeEngine;
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

//! 字词用例：把 `system` 字符串派发到对应的字词叶。
//!
//! 注册表由调用方注入——用例不认识装配根。

use mingli_contract::{WordEngine, WordQuery};
use serde_json::Value;

/// 按 `system` 找叶并取值。
///
/// # Errors
///
/// `system` 不在注册表内，或该叶所需输入不足时返回中文说明。
pub fn compute(reg: &[Box<dyn WordEngine>], system: &str, q: &WordQuery) -> Result<Value, String> {
    let e = reg
        .iter()
        .find(|e| e.id() == system)
        .ok_or_else(|| format!("未知字词系统 {system}"))?;
    e.compute(q)
}

#[cfg(test)]
mod tests {
    use super::*;
    use mingli_registry::word_registry;

    #[test]
    fn dispatches_by_system_and_reports_unknown() {
        let reg = word_registry();
        let q = WordQuery { text: Some("שלום".to_string()), ..WordQuery::default() };
        let v = compute(&reg, "gematria", &q).expect("gematria 应在注册表内");
        assert_eq!(v["system"], "gematria");
        assert!(compute(&reg, "nope", &q).is_err());
    }

    #[test]
    fn wuge_requires_both_stroke_lists() {
        let reg = word_registry();
        let empty = WordQuery::default();
        assert!(compute(&reg, "wuge", &empty).is_err());
        let ok = WordQuery { surname: Some(vec![7]), given: Some(vec![9, 9]), ..WordQuery::default() };
        assert_eq!(compute(&reg, "wuge", &ok).expect("笔画齐备应通过")["system"], "wuge");
    }
}

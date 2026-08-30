//! 本叶对 [`mingli_contract::WordEngine`] 的实现——字/词模态不吃出生时刻，
//! 只吃文字或笔画，因此走与 `CastingEngine` 平行的第二条契约。

use mingli_contract::{d, DetItem, Determinism, WordEngine, WordQuery};
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
    fn profile(&self) -> &'static [DetItem] {
        use Determinism::Det;
        const { &[
            d("22 本形字母值（Mispar Hechrachi）", Det, "1–9 / 10–90 / 100–400 三段，多源一致；五尾形取本形值"),
            d("Gadol / Siduri / Katan 三种变体值", Det, "Gadol 仅五尾形取 500–900；Siduri 取字母表序 1–22；Katan 逐字 mod 9（0 归 9）"),
            d("Katan Mispari（整词数字根）", Det, "对整词标准值反复求各位和至 1..=9，等价 1+(n−1) mod 9"),
            d("AtBash / AlBam 替换码", Det, "AtBash i↔23−i，AlBam 1..=11↔12..=22；替换后按标准值求和"),
            d("非希伯来字符处理", Det, "一律跳过，不参与求和；七种计法行为一致"),
        ] }
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
        assert!(!e.profile().is_empty(), "每片叶都要显式声明确定性谱");
        let v = e.compute(&q).expect("输入齐备应能取值");
        assert_eq!(v["system"], e.id());
    }
}

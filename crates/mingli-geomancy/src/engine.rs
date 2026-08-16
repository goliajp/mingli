//! 本叶对 [`mingli_contract::CastingEngine`] 的实现——把叶的领域计算适配成
//! 全树统一的排盘契约，并声明本叶的确定性边界与流派。

use mingli_contract::{d, effective_seed, CastingEngine, DetItem, Determinism, Family, Moment, Query};
use serde_json::Value;

/// 地占叶（C 族）。4 母图→盾牌图，种子可复现，法官恒为偶。
#[derive(Debug, Default)]
pub struct GeomancyEngine;

impl CastingEngine for GeomancyEngine {
    fn id(&self) -> &'static str {
        "geomancy"
    }
    fn name(&self) -> &'static str {
        "地占"
    }
    fn family(&self) -> Family {
        Family::Sampling
    }
    fn cast(&self, m: &Moment, q: &Query) -> Value {
        serde_json::to_value(crate::cast(effective_seed(m, q))).unwrap_or(Value::Null)
    }
    fn profile(&self) -> &'static [DetItem] {
        use Determinism::{Det, Sto, Und};
        const { &[
            d("四母→盾牌图", Sto, "种子可复现"),
            d("法官恒为偶", Det, "GF(2) 线性，穷举证于 core::gf2"),
            d("16 figure 名", Und, "查表待补，只显 GF(2) 四点结构"),
        ] }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 适配器把本叶接到统一契约上：元数据齐备、能出盘、确定性谱已声明、
    /// 有流派时恰有一个默认。
    #[test]
    fn adapter_is_wired_to_the_contract() {
        let e = GeomancyEngine;
        assert!(!e.id().is_empty() && !e.name().is_empty());
        let m = Moment::new(1990, 6, 15, 14, 30, 8.0);
        let q = Query::at(1990, 6, 15, 14, 30, 8.0);
        assert!(!e.cast(&m, &q).is_null(), "每片叶都应产出非空盘面");
        assert!(!e.profile().is_empty(), "每片叶都要显式声明确定性谱");
        let defaults = e.schools().iter().filter(|s| s.default).count();
        assert!(e.schools().is_empty() || defaults == 1, "有流派的叶应恰有一个默认");
        assert!(!e.family().label().is_empty());
    }
}

//! 本叶对 [`mingli_contract::CastingEngine`] 的实现——把叶的领域计算适配成
//! 全树统一的排盘契约，并声明本叶的确定性边界与流派。

use mingli_contract::{d, effective_seed, CastingEngine, DetItem, Determinism, Family, Moment, Query};
use serde_json::Value;

/// Ifá 叶（C 族）。双 figure→256 odu，种子可复现。
#[derive(Debug, Default)]
pub struct IfaEngine;

impl CastingEngine for IfaEngine {
    fn id(&self) -> &'static str {
        "ifa"
    }
    fn name(&self) -> &'static str {
        "Ifá"
    }
    fn family(&self) -> Family {
        Family::Sampling
    }
    fn cast(&self, m: &Moment, q: &Query) -> Value {
        serde_json::to_value(crate::cast(effective_seed(m, q))).unwrap_or(Value::Null)
    }
    fn profile(&self) -> &'static [DetItem] {
        use Determinism::{Sto, Und};
        const { &[
            d("双 figure→256 odu", Sto, "种子可复现；16×16 = (Z₂)⁴ 组合"),
            d("256 odu 名", Und, "查表（错一个毒整枝）待补，结构已建"),
        ] }
    }
}

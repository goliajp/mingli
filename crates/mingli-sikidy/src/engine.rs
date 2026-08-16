//! 本叶对 [`mingli_contract::CastingEngine`] 的实现——把叶的领域计算适配成
//! 全树统一的排盘契约，并声明本叶的确定性边界与流派。

use mingli_contract::{d, effective_seed, CastingEngine, DetItem, Determinism, Family, Moment, Query};
use serde_json::Value;

/// Sikidy 叶（C 族）。4 母列→16 列，种子可复现，C15 恒为偶。
#[derive(Debug, Default)]
pub struct SikidyEngine;

impl CastingEngine for SikidyEngine {
    fn id(&self) -> &'static str {
        "sikidy"
    }
    fn name(&self) -> &'static str {
        "Sikidy"
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
            d("四母→16 列", Sto, "种子可复现，同地占 GF(2) 代数"),
            d("创世者 C15 恒偶", Det, "与地占法官同一定理"),
            d("16 列名表", Und, "查表待补"),
        ] }
    }
}

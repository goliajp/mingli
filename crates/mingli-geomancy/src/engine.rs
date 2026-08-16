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

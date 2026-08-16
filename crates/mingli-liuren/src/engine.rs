//! 本叶对 [`mingli_contract::CastingEngine`] 的实现——把叶的领域计算适配成
//! 全树统一的排盘契约，并声明本叶的确定性边界与流派。

use mingli_contract::{d, CastingEngine, DetItem, Determinism, Family, Moment, Query};
use serde_json::Value;

/// 大六壬叶（⟂ 横切）。天地盘 + 四课 + 三传课式。
#[derive(Debug, Default)]
pub struct LiurenEngine;

impl CastingEngine for LiurenEngine {
    fn id(&self) -> &'static str {
        "liuren"
    }
    fn name(&self) -> &'static str {
        "大六壬"
    }
    fn family(&self) -> Family {
        Family::CrossCutting
    }
    fn cast(&self, m: &Moment, _q: &Query) -> Value {
        serde_json::to_value(crate::compute_at(m)).unwrap_or(Value::Null)
    }
    fn profile(&self) -> &'static [DetItem] {
        use Determinism::{Det, Und};
        const { &[
            d("天地盘·寄宫四课", Det, "月将加时 Z₁₂ 旋转，校验 亥将子时甲子日"),
            d("三传·贼克/比用/遥克/伏返", Det, "取传规则明确"),
            d("三传·涉害/昴星/别责/八专", Und, "取传流派分歧，诚实返 None 不强编"),
        ] }
    }
}

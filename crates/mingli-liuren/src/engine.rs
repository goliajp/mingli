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

#[cfg(test)]
mod tests {
    use super::*;

    /// 适配器把本叶接到统一契约上：元数据齐备、能出盘、确定性谱已声明、
    /// 有流派时恰有一个默认。
    #[test]
    fn adapter_is_wired_to_the_contract() {
        let e = LiurenEngine;
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

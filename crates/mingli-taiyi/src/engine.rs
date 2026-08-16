//! 本叶对 [`mingli_contract::CastingEngine`] 的实现——把叶的领域计算适配成
//! 全树统一的排盘契约，并声明本叶的确定性边界与流派。

use mingli_contract::{d, CastingEngine, DetItem, Determinism, Family, Moment, Query};
use serde_json::Value;

/// 太乙神数叶（⟂ 横切）。太乙积年 → 太乙行八宫（三年一宫·阳顺阴逆）+ 三才。
#[derive(Debug, Default)]
pub struct TaiyiEngine;

impl CastingEngine for TaiyiEngine {
    fn id(&self) -> &'static str {
        "taiyi"
    }
    fn name(&self) -> &'static str {
        "太乙神数"
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
            d("太乙行八宫·三才·积年", Det, "三年一宫·廿四年一周，积年锚《金镜式经》724=1937281"),
            d("文昌·始击·主客算·诸将", Und, "源间分歧，暂缺"),
            d("落宫绝对相位", Und, "遵引文规则，精校待权威排盘软件"),
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
        let e = TaiyiEngine;
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

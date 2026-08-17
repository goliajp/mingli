//! 本叶对 [`mingli_contract::CastingEngine`] 的实现——把叶的领域计算适配成
//! 全树统一的排盘契约，并声明本叶的确定性边界与流派。

use mingli_contract::{d, CastingEngine, DetItem, Determinism, Family, Moment, Query};
use serde_json::Value;

/// 巴厘 Pawukon 叶（A 族·多并行週）。210 上的十个 wewaran。
#[derive(Debug, Default)]
pub struct PawukonEngine;

impl CastingEngine for PawukonEngine {
    fn id(&self) -> &'static str {
        "pawukon"
    }
    fn name(&self) -> &'static str {
        "巴厘Pawukon"
    }
    fn family(&self) -> Family {
        Family::Cyclic
    }
    fn cast(&self, m: &Moment, _q: &Query) -> Value {
        serde_json::to_value(crate::compute_at(m)).unwrap_or(Value::Null)
    }
    fn profile(&self) -> &'static [DetItem] {
        use Determinism::Det;
        const { &[
            d("十週（简单/派生/卡日）", Det, "210=2·3·5·7，锚 day0=2020-07-05 校验 Galungan"),
            d("Pancawara/Saptawara urip 权重表", Det, "5 独立源逐值一致：Babad Bali 本地权威表 / en.wikipedia Pawukon / Reingold-Dershowitz 参考实现 / sakacalendar / balinese-date-js-lib"),
            d("Ekawara/Dwiwara 奇偶向", Det, "urip 之和为奇 → Luang + Pepet，为偶 → 无 Ekawara + Menga；6 独立源同向。唯一相反记载(sejarahharirayahindu)与 sastrabali 同文转载且自身 Eka/Dwi 互相矛盾，判为讹误"),
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
        let e = PawukonEngine;
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

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
        use Determinism::{Det, Und};
        const { &[
            d("十週（简单/派生/卡日）", Det, "210=2·3·5·7，锚 day0=2020-07-05 校验 Galungan"),
            d("Ekawara/Dwiwara 奇偶向", Und, "源间一处冲突，采信两个独立实现"),
        ] }
    }
}

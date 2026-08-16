//! 本叶对 [`mingli_contract::CastingEngine`] 的实现——把叶的领域计算适配成
//! 全树统一的排盘契约，并声明本叶的确定性边界与流派。

use mingli_contract::{d, CastingEngine, DetItem, Determinism, Family, Moment, Query};
use serde_json::Value;

/// 玛雅历叶（A 族·CRT）。Tzolkʼin 260 + Haab 365 + Long Count。
#[derive(Debug, Default)]
pub struct MayaEngine;

impl CastingEngine for MayaEngine {
    fn id(&self) -> &'static str {
        "maya"
    }
    fn name(&self) -> &'static str {
        "玛雅历"
    }
    fn family(&self) -> Family {
        Family::Cyclic
    }
    fn cast(&self, m: &Moment, _q: &Query) -> Value {
        serde_json::to_value(crate::compute_at(m)).unwrap_or(Value::Null)
    }
    fn profile(&self) -> &'static [DetItem] {
        use Determinism::Det;
        const { &[d("Tzolkʼin·Haab·Long Count", Det, "GMT 历元 584283，校验 0.0.0.0.0 与 2012-12-21 双锚")] }
    }
}

//! 本叶对 [`mingli_contract::CastingEngine`] 的实现——把叶的领域计算适配成
//! 全树统一的排盘契约，并声明本叶的确定性边界与流派。

use mingli_contract::{d, CastingEngine, DetItem, Determinism, Family, Moment, Query};
use serde_json::Value;

/// 缅甸 Mahabote 叶（A 族）。本命核心数 = （缅历年 − 星期） mod 7。
#[derive(Debug, Default)]
pub struct MahaboteEngine;

impl CastingEngine for MahaboteEngine {
    fn id(&self) -> &'static str {
        "mahabote"
    }
    fn name(&self) -> &'static str {
        "缅甸Mahabote"
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
            d("核心数·七宫·八天週行星", Det, "（缅历年−星期） mod 7，校验 2000-01-01=Adipati"),
            d("宫义·宫间关系", Und, "无自洽单源，不下断言"),
        ] }
    }
}

//! 本叶对 [`mingli_contract::CastingEngine`] 的实现——把叶的领域计算适配成
//! 全树统一的排盘契约，并声明本叶的确定性边界与流派。

use mingli_contract::{d, CastingEngine, DetItem, Determinism, Family, Moment, Query};
use serde_json::Value;

/// 藏历循环叶（A 族）。60 周期（元素×生肖）+ 年 mewa。
#[derive(Debug, Default)]
pub struct TibetanEngine;

impl CastingEngine for TibetanEngine {
    fn id(&self) -> &'static str {
        "tibetan"
    }
    fn name(&self) -> &'static str {
        "藏历循环"
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
            d("60 周期·年 mewa", Det, "5 元素×12 生肖；mewa 逆行，校验 2024=木阳龙·mewa3"),
            d("年 parkha", Und, "主流藏历无年卦（仅个人盘），故不输出"),
        ] }
    }
}

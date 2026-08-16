//! 本叶对 [`mingli_contract::CastingEngine`] 的实现——把叶的领域计算适配成
//! 全树统一的排盘契约，并声明本叶的确定性边界与流派。

use mingli_contract::{d, CastingEngine, DetItem, Determinism, Family, Moment, Query};
use serde_json::Value;

/// 小六壬叶（A 族·时间起课，确定性）。月→日→时辰在 Z₆ 上掐指。
#[derive(Debug, Default)]
pub struct XiaoliurenEngine;

impl CastingEngine for XiaoliurenEngine {
    fn id(&self) -> &'static str {
        "xiaoliuren"
    }
    fn name(&self) -> &'static str {
        "小六壬"
    }
    fn family(&self) -> Family {
        Family::Cyclic
    }
    fn cast(&self, m: &Moment, _q: &Query) -> Value {
        serde_json::to_value(crate::compute_at(m)).unwrap_or(Value::Null)
    }
    fn profile(&self) -> &'static [DetItem] {
        use Determinism::Det;
        const { &[d("六神掐指（月→日→时）", Det, "Z₆ 连续位移，六神为定义性有序环")] }
    }
}

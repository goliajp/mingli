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

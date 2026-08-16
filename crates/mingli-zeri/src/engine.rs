//! 本叶对 [`mingli_contract::CastingEngine`] 的实现——把叶的领域计算适配成
//! 全树统一的排盘契约，并声明本叶的确定性边界与流派。

use mingli_contract::{d, CastingEngine, DetItem, Determinism, Family, Moment, Query};
use serde_json::Value;

/// 择日叶（A 族）。建除十二神 + 二十八宿值日 + 彭祖百忌 + 天乙贵人。
#[derive(Debug, Default)]
pub struct ZeriEngine;

impl CastingEngine for ZeriEngine {
    fn id(&self) -> &'static str {
        "zeri"
    }
    fn name(&self) -> &'static str {
        "择日"
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
            d("建除十二神", Det, "日支−月建支 on Z₁₂"),
            d("二十八宿值日", Det, "连续 Z₂₈，偏移 11 跨 341 年 5 锚校验"),
            d("彭祖百忌（干句+支句）", Det, "《钦定协纪辨方书》/通胜多源口诀，22 句固定查表"),
            d("天乙贵人（双地支）", Det, "《三命通会》通行版『甲戊庚牛羊』口诀"),
            d("天乙贵人（《珞琭子赋》变体）", Und, "庚归虎马，无多源校验源，不入码"),
            d("其余神煞宜忌", Und, "随流派分歧，不下断言"),
        ] }
    }
}

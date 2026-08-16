//! 本叶对 [`mingli_contract::CastingEngine`] 的实现——把叶的领域计算适配成
//! 全树统一的排盘契约，并声明本叶的确定性边界与流派。

use mingli_contract::{d, effective_seed, s, CastingEngine, DetItem, Determinism, Family, Moment, Query, SchoolItem};
use serde_json::Value;

/// 梅花易数叶（时间起卦·确定性）。年支/月/日/时辰 mod8/mod6 → 卦，不用种子。
#[derive(Debug, Default)]
pub struct MeihuaEngine;

impl CastingEngine for MeihuaEngine {
    fn id(&self) -> &'static str {
        "meihua"
    }
    fn name(&self) -> &'static str {
        "梅花易数"
    }
    fn family(&self) -> Family {
        // 时间→模运算→卦，属 A 族（确定性循环），非随机抽样。
        Family::Cyclic
    }
    fn cast(&self, m: &Moment, q: &Query) -> Value {
        let method = crate::Method::from_id(q.school_of(self.id(), "time"))
            .unwrap_or_default();
        let cst = crate::compute_at_with(m, method, effective_seed(m, q));
        serde_json::to_value(cst).unwrap_or(Value::Null)
    }
    fn profile(&self) -> &'static [DetItem] {
        use Determinism::{Det, Sto};
        const { &[
            d("本/互/之卦·体用五行（时间法）", Det, "农历量 mod8/mod6，确定；同时刻同卦"),
            d("本/互/之卦（数字法）", Sto, "两数由种子高低 32 位派生，同种子可复现"),
            d("六十四卦名 + 文王序", Det, "三源校验，「二二相耦」定理穷举证明"),
        ] }
    }
    fn schools(&self) -> &'static [SchoolItem] {
        const { &[
            s("time", "时间起卦法", true, "邵雍古法：年支/月/日/时辰 mod8/mod6；确定性，同时刻同卦"),
            s("numbers", "数字（报数）法", false, "首数为上卦，次数为下卦，（首+次+时辰） mod6 为动爻；两数由种子拆解派生（C 族风格）"),
        ] }
    }
}

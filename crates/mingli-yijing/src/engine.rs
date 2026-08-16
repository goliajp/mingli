//! 本叶对 [`mingli_contract::CastingEngine`] 的实现——把叶的领域计算适配成
//! 全树统一的排盘契约，并声明本叶的确定性边界与流派。

use mingli_contract::{d, effective_seed, s, CastingEngine, DetItem, Determinism, Family, Moment, Query, SchoolItem};
use serde_json::Value;

/// 易经起卦叶（C 族）。三钱法、种子可复现。
#[derive(Debug, Default)]
pub struct YijingEngine;

impl CastingEngine for YijingEngine {
    fn id(&self) -> &'static str {
        "yijing"
    }
    fn name(&self) -> &'static str {
        "易经起卦"
    }
    fn family(&self) -> Family {
        Family::Sampling
    }
    fn cast(&self, m: &Moment, q: &Query) -> Value {
        let method = match q.school_of(self.id(), "three_coins") {
            "yarrow_stalks" => crate::Method::YarrowStalks,
            _ => crate::Method::ThreeCoins,
        };
        let cst = crate::cast(method, effective_seed(m, q));
        serde_json::to_value(cst).unwrap_or(Value::Null)
    }
    fn profile(&self) -> &'static [DetItem] {
        use Determinism::{Det, Sto};
        const { &[
            d("六爻·本卦/之卦", Sto, "种子可复现；两流派概率分布 1/8·3/8·3/8·1/8（三钱）、1/16·5/16·7/16·3/16（蓍草） 均校验"),
            d("六十四卦名 + 文王序", Det, "三源校验（ctext《序卦传》/zh.wiki/en.wiki），「二二相耦」定理（4 纯错对 + 28 综对）穷举证明"),
        ] }
    }
    fn schools(&self) -> &'static [SchoolItem] {
        const { &[
            s("three_coins", "三钱法", true, "三枚铜钱六掷；概率 6：7：8：9 = 1：3：3：1（老阴/少阳/少阴/老阳）"),
            s("yarrow_stalks", "蓍草法", false, "五十蓍策十八变（模拟分布）；概率 6：7：8：9 = 1：5：7：3"),
        ] }
    }
}

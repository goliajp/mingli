//! 本叶对 [`mingli_contract::CastingEngine`] 的实现——把叶的领域计算适配成
//! 全树统一的排盘契约，并声明本叶的确定性边界与流派。

use mingli_contract::{d, effective_seed, s, CastingEngine, DetItem, Determinism, Family, Intent, Moment, Principal, Query, SchoolItem};
use serde_json::Value;

/// 易经起卦叶（C 族）。三钱法、种子可复现。
#[derive(Debug, Default)]
pub struct YijingEngine;

/// 本次查询下的盘。
///
/// `cast` 与 `principal` 都从这里取：一个把它整份序列化，一个读它的一个字段。
/// 分出来是为了让后者不必去解前者产出的 JSON——字段改名时，读结构体会编译报错，解 JSON 不会。
fn chart(e: &YijingEngine, m: &Moment, q: &Query) -> crate::Cast {
    let method = match q.school_of(e.id(), "three_coins") {
        "yarrow_stalks" => crate::Method::YarrowStalks,
        _ => crate::Method::ThreeCoins,
    };
    crate::cast(method, effective_seed(m, q))
}

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
        serde_json::to_value(chart(self, m, q)).unwrap_or(Value::Null)
    }
    fn answers(&self) -> &'static [Intent] {
        &[Intent::Natal, Intent::Event]
    }
    fn principal(&self, m: &Moment, q: &Query) -> Option<Principal> {
        // 本卦下卦——内卦为主。
        let c = chart(self, m, q);
        Some(Principal { label: "下卦", value: c.primary_lower.to_string() })
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

#[cfg(test)]
mod tests {
    use super::*;

    /// 适配器把本叶接到统一契约上：元数据齐备、能出盘、确定性谱已声明、
    /// 有流派时恰有一个默认。
    #[test]
    fn adapter_is_wired_to_the_contract() {
        let e = YijingEngine;
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

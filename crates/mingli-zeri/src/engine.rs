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
            d("天乙贵人（双地支）", Det, "通行版口诀 5 源一致：《三命通会》卷三、《五行精纪》卷十三/卷十四、《珞琭子三命消息赋注》(徐子平)、《渊海子平·论日贵》"),
            d("天乙贵人「庚辛逢虎马」一系", Und, "坊间归给《珞琭子赋》不成立——徐子平注与昙莹注全文皆无此诀，徐注用例反是通行版；《渊海子平》全文亦无。唯一原始出处为唐·李筌《太白阴经》卷十『庚辛之日旦理胜光暮理功曹』，属六壬旦暮贵人体系，单源；且该系把甲戊合并作旦丑暮未，不能只挪庚一格移植"),
            d("其余神煞宜忌", Und, "随流派分歧，不下断言"),
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
        let e = ZeriEngine;
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

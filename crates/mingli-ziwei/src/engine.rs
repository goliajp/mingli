//! 本叶对 [`mingli_contract::CastingEngine`] 的实现——把叶的领域计算适配成
//! 全树统一的排盘契约，并声明本叶的确定性边界与流派。

use mingli_contract::{d, s, CastingEngine, DetItem, Determinism, Family, Moment, Query, SchoolItem};
use serde_json::Value;

/// 契约层性别 → 本叶性别。
fn leaf_gender(g: Option<mingli_contract::Gender>) -> Option<crate::Gender> {
    g.map(|x| match x {
        mingli_contract::Gender::Male => crate::Gender::Male,
        mingli_contract::Gender::Female => crate::Gender::Female,
    })
}

/// 紫微斗数叶。
#[derive(Debug, Default)]
pub struct ZiweiEngine;

impl CastingEngine for ZiweiEngine {
    fn id(&self) -> &'static str {
        "ziwei"
    }
    fn name(&self) -> &'static str {
        "紫微斗数"
    }
    fn family(&self) -> Family {
        Family::Cyclic
    }
    fn cast(&self, m: &Moment, q: &Query) -> Value {
        let school = crate::SihuaSchool::from_id(q.school_of(self.id(), "standard"))
            .unwrap_or_default();
        serde_json::to_value(crate::compute_at_with(m, leaf_gender(q.gender), school))
            .unwrap_or(Value::Null)
    }
    fn profile(&self) -> &'static [DetItem] {
        use Determinism::{Det, Und};
        const { &[
            d("十二宫·五行局·主星", Det, "Z₁₂ 群作用+五行局，校验 命宫亥·土五局·紫微申"),
            d("4 辅星（昌曲辅弼）", Det, "古典通行口诀（《紫微斗数·安文昌文曲星诀》+ 维基/iztro 实现双证），1990 庚午校验"),
            d("四化（禄/权/科/忌）", Det, "通行版 5 源完全一致；全书本（王亭之）庚/壬科星分歧 — 王亭之亲文+《全书》古本双证"),
            d("四化派（戊/癸）", Und, "戊/癸的派别分歧本次研究未获多源证据，两派统一取通行表；待权威钦天派文献再补"),
        ] }
    }
    fn schools(&self) -> &'static [SchoolItem] {
        const { &[
            s("standard", "通行版（中州/三合派）", true, "5 独立源完全一致(cnblogs/51xingli×2/vocus/wikipedia)；庚=太阴化科，壬=左辅化科"),
            s("quanshu", "全书本（王亭之版）", false, "庚=天府化科（王亭之亲文 haozh.com）；壬=天府化科（《全书》古本）；其余 8 干同通行版"),
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
        let e = ZiweiEngine;
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

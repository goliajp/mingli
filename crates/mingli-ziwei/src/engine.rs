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
            d("四化（禄/权/科/忌）", Det, "通行版 5 源完全一致；中州派(王亭之)在戊/庚/壬三干的化科上分歧，源自「辅弼不入四化」一条学理，三干同开"),
            d("戊干化科", Det, "通行作右弼 ≥7 独立源(《全书》原诀、维基全集全书两栏、梁若瑜飞星派、钦天门、星林学苑、紫微台、紫微杨)；中州派作太阳 3~4 独立源(王亭之两处亲文、九千飞星版本对照表、蓝天空)。两派并存，均已实现"),
            d("癸干化科", Det, "非分歧项——查过《全书》原诀「癸破巨阴贪狼停」及全集/闽派/北派河洛/占验门/钦天门/梁派飞星/中州派陆斌兆/中州派王亭之十家版本表，癸行逐字全同；多篇对照文亦明写「争议只在戊庚壬三干」"),
            d("庚干化科的底本出处", Und, "《紫微斗数全书》传本自身有异文：一转录本原诀作「庚日武府同」(天府科/天同忌)，另一版本对照表把「全书·闽派」栏记作「阳武同阴」(天同科/太阴忌)、而把天府科归中州派。本叶按中州派学理归入 quanshu 表，但《全书》底本究竟作何未定"),
        ] }
    }
    fn schools(&self) -> &'static [SchoolItem] {
        const { &[
            s("standard", "通行版（中州/三合派）", true, "5 独立源完全一致(cnblogs/51xingli×2/vocus/wikipedia)；庚=太阴化科，壬=左辅化科"),
            s("quanshu", "中州派（王亭之版）", false, "主张辅弼不入四化：戊=太阳化科、庚=天府化科、壬=天府化科；其余 7 干同通行版。三干同源于一条学理，不可只开其一。id 沿用历史命名"),
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

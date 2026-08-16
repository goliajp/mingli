//! 本叶对 [`mingli_contract::CastingEngine`] 的实现——把叶的领域计算适配成
//! 全树统一的排盘契约，并声明本叶的确定性边界与流派。

use mingli_contract::{d, s, CastingEngine, DetItem, Determinism, Family, Moment, Query, SchoolItem};
use serde_json::Value;

/// 西洋占星本命盘叶（B 族）。仅 `astrology` feature 开启时编译（连带 VSOP87 星历）。
#[derive(Debug, Default)]
pub struct AstrologyEngine;

impl CastingEngine for AstrologyEngine {
    fn id(&self) -> &'static str {
        "astrology"
    }
    fn name(&self) -> &'static str {
        "西洋占星"
    }
    fn family(&self) -> Family {
        Family::Angular
    }
    fn cast(&self, m: &Moment, q: &Query) -> Value {
        let geo = match (q.latitude, q.longitude) {
            (Some(latitude), Some(longitude)) => Some(crate::GeoLocation {
                latitude,
                longitude,
            }),
            _ => None,
        };
        let house_system = crate::HouseSystem::from_id(q.school_of(self.id(), "placidus"));
        serde_json::to_value(crate::compute_at(m, geo, house_system))
            .unwrap_or(Value::Null)
    }
    fn profile(&self) -> &'static [DetItem] {
        use Determinism::Det;
        const { &[
            d("行星落座·相位", Det, "VSOP87 视黄经，太阳校验 Meeus 0.02°"),
            d("月亮落座", Det, "ELP-2000/82 (astro crate)，校验 Meeus 47.a < 5″ 与 Diana(AA) < 0.2°"),
            d("Asc/MC", Det, "平恒星时+平交角，校验 Diana(AA) < 0.5°"),
            d("分宫制(Placidus/Koch/WholeSign/Equal/Porphyry)", Det, "Placidus/Koch 移植 swehouse.c+Diana 12 cusp<0.05°(pyswisseph oracle)；极区回落 Porphyry"),
        ] }
    }
    fn schools(&self) -> &'static [SchoolItem] {
        const { &[
            s("placidus", "Placidus 半弧三分", true, "占星圈业界默认；移植 Swiss swehouse.c；极区(|φ|≥66.5°)回落 Porphyry"),
            s("koch", "Koch 等赤经四分", false, "Walter Koch 1962；移植 Swiss swehouse.c 'K' case；极区(|φ|≥66.5°)回落 Porphyry"),
            s("whole_sign", "整宫制 Whole Sign", false, "古典占星与希腊占星派常用；一宫=一星座；极区可用"),
            s("equal", "Equal 等宫", false, "从上升起每 30° 一宫；MC 不作 10 宫尖；极区可用"),
            s("porphyry", "Porphyry 黄道三分", false, "1/10/4/7=Asc/MC/IC/DC；中间宫尖在黄道弧上三分；极区可用"),
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
        let e = AstrologyEngine;
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

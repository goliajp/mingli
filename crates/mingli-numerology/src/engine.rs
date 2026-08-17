//! 本叶对 [`mingli_contract::CastingEngine`] 的实现——把叶的领域计算适配成
//! 全树统一的排盘契约，并声明本叶的确定性边界与流派。

use mingli_contract::{d, s, CastingEngine, DetItem, Determinism, Family, Moment, Query, SchoolItem};
use serde_json::Value;

/// 数字学叶（D 族·哈希环）。日期生命灵数 + 生日数；给出姓名时附表达/灵魂/人格数（两套字母表）。
#[derive(Debug, Default)]
pub struct NumerologyEngine;

impl CastingEngine for NumerologyEngine {
    fn id(&self) -> &'static str {
        "numerology"
    }
    fn name(&self) -> &'static str {
        "数字学"
    }
    fn family(&self) -> Family {
        Family::Hashing
    }
    fn cast(&self, m: &Moment, q: &Query) -> Value {
        let method = match q.school_of(self.id(), "component") {
            "whole_sum" => crate::LifePathMethod::WholeSum,
            _ => crate::LifePathMethod::Component,
        };
        let cst = match &q.name {
            Some(name) => crate::compute_named_with(m, name, method),
            None => crate::compute_at_with(m, method),
        };
        serde_json::to_value(cst).unwrap_or(Value::Null)
    }
    fn profile(&self) -> &'static [DetItem] {
        use Determinism::{Det, Und};
        const { &[
            d("姓名数（双字母表并出）", Det, "Pythagorean/Chaldean 同时输出，无需选择"),
            d("生命灵数（可选 Component/WholeSum）", Det, "两派算法已实现并交叉校验；每次同时给出主+alt"),
            d("Y 元音归属（三说并出）", Det, "语境派『Y 紧邻元音则作辅音』4 独立源(Decoz/World Numerology、Token Rock、Felicia Bender、Crystal Logic)，可复现 Decoz 全部八条位置细则；『跟在元音后仍算元音』一支 2 独立源(Lyn's、Astrala)；『一律辅音』1 二手源(Bender 转述 Juno Jordan)。三读同时输出，不替调用方选边"),
            d("Y 归属的按音节条款", Und, "🟡 语境两派都还带一条『该音节里没有别的元音时 Y 算元音』(如 Bryan)，须分音节才能判；本叶无音节切分器，不实现也不假装实现"),
            d("W 是否可作元音", Und, "🟡 Matthew/Drew/Owen 一类里 W 算元音的说法仅 2 源(其一只有立场无规则)，且 Decoz 明确反对；强度不足，本叶一律作辅音"),
        ] }
    }
    fn schools(&self) -> &'static [SchoolItem] {
        const { &[
            s("component", "分量约化（Pythagorean 学派）", true, "y/m/d 各约化后求和再约化；现代数字学常用"),
            s("whole_sum", "全数字直加（Chaldean 学派）", false, "ymd 全数字平铺相加再约化；古典 Chaldean/Kabbalistic 派常用"),
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
        let e = NumerologyEngine;
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

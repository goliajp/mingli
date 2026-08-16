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
            d("Y 元音归属", Und, "Y 是否计入元音/辅音随细分流派，本叶按辅音处理"),
        ] }
    }
    fn schools(&self) -> &'static [SchoolItem] {
        const { &[
            s("component", "分量约化（Pythagorean 学派）", true, "y/m/d 各约化后求和再约化；现代数字学常用"),
            s("whole_sum", "全数字直加（Chaldean 学派）", false, "ymd 全数字平铺相加再约化；古典 Chaldean/Kabbalistic 派常用"),
        ] }
    }
}

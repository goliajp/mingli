//! 本叶对 [`mingli_contract::CastingEngine`] 的实现——把叶的领域计算适配成
//! 全树统一的排盘契约，并声明本叶的确定性边界与流派。

use mingli_contract::{d, CastingEngine, DetItem, Determinism, Family, Intent, Moment, Principal, Query};
use serde_json::Value;

/// 小六壬叶（A 族·时间起课，确定性）。月→日→时辰在 Z₆ 上掐指。
#[derive(Debug, Default)]
pub struct XiaoliurenEngine;

/// 本次查询下的盘。
///
/// `cast` 与 `principal` 都从这里取：一个把它整份序列化，一个读它的一个字段。
/// 分出来是为了让后者不必去解前者产出的 JSON——字段改名时，读结构体会编译报错，解 JSON 不会。
fn chart(_e: &XiaoliurenEngine, m: &Moment, _q: &Query) -> crate::Cast {
crate::compute_at(m)
}

impl CastingEngine for XiaoliurenEngine {
    fn id(&self) -> &'static str {
        "xiaoliuren"
    }
    fn name(&self) -> &'static str {
        "小六壬"
    }
    fn family(&self) -> Family {
        Family::Cyclic
    }
    fn cast(&self, m: &Moment, q: &Query) -> Value {
        serde_json::to_value(chart(self, m, q)).unwrap_or(Value::Null)
    }
    fn answers(&self) -> &'static [Intent] {
        // 只答「命」。「择」要的是按吉凶分档的候选日，本叶给的是某一时辰落在六神的哪一位，
        // 不是那个形态；「寻」要方位候选，而本叶没有实现 `bearings`——路由到它只会排一张盘、
        // 一个候选都不出。六神传统上确有方位之说，但那是「还没做」。
        &[Intent::Natal]
    }
    fn principal(&self, m: &Moment, q: &Query) -> Option<Principal> {
        // 时辰落在六神的哪一位。
        let c = chart(self, m, q);
        Some(Principal { label: "时神位", value: c.hour_pos.to_string() })
    }
    fn profile(&self) -> &'static [DetItem] {
        use Determinism::Det;
        const { &[d("六神掐指（月→日→时）", Det, "Z₆ 连续位移，六神为定义性有序环")] }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 适配器把本叶接到统一契约上：元数据齐备、能出盘、确定性谱已声明、
    /// 有流派时恰有一个默认。
    #[test]
    fn adapter_is_wired_to_the_contract() {
        let e = XiaoliurenEngine;
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

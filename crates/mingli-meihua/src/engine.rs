//! 本叶对 [`mingli_contract::CastingEngine`] 的实现——把叶的领域计算适配成
//! 全树统一的排盘契约，并声明本叶的确定性边界与流派。

use mingli_contract::{d, effective_seed, s, CastingEngine, DetItem, Determinism, Family, Intent, Moment, Principal, Query, SchoolItem};
use serde_json::Value;

/// 梅花易数叶（时间起卦·确定性）。年支/月/日/时辰 mod8/mod6 → 卦，不用种子。
#[derive(Debug, Default)]
pub struct MeihuaEngine;

/// 本次查询下的盘。
///
/// `cast` 与 `principal` 都从这里取：一个把它整份序列化，一个读它的一个字段。
/// 分出来是为了让后者不必去解前者产出的 JSON——字段改名时，读结构体会编译报错，解 JSON 不会。
fn chart(e: &MeihuaEngine, m: &Moment, q: &Query) -> crate::Cast {
    let method = crate::Method::from_id(q.school_of(e.id(), "time"))
        .unwrap_or_default();
    crate::compute_at_with(m, method, effective_seed(m, q))
}

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
        serde_json::to_value(chart(self, m, q)).unwrap_or(Value::Null)
    }
    fn reading_notes(&self) -> Option<&'static str> {
        Some("\n【字段语义提示（梅花易数，时间起卦）】\n\
            - 起卦三数：`year_branch` / `month` / `day` 相加取上卦、再加 `hour_branch` 取下卦与动爻。\
              `method_id` 是所用起卦法（时间 / 数字等），`numbers` 为数字起卦时的输入。\n\
            - 四卦一套：`primary_*` 本卦（事之现状）、`mutual_*` 互卦（事中之情）、\
              `changed_*` 之卦（事之归宿）。每套各带 `_name` 卦名、`_full_name` 全名、\
              `_king_wen` 通行序号、`_upper` / `_lower` 上下卦。\n\
            - `moving_line`：动爻（1..6，自下起）。动爻是本卦变之卦的那一爻，也是断事的着眼处。\n\
            - 体用之分不在盘面上：动爻所在之卦为用、另一卦为体，体用生克才是梅花的判据；\
              这一层要由读的人依上下卦自行判定。\n\
            - **读法**：先看本卦与动爻，再以互卦看中间过程，之卦收尾；三卦一线说完即可。")
    }
    fn answers(&self) -> &'static [Intent] {
        &[Intent::Natal, Intent::Event]
    }
    fn principal(&self, m: &Moment, q: &Query) -> Option<Principal> {
        // 本卦上卦。
        let c = chart(self, m, q);
        Some(Principal { label: "上卦", value: c.primary_upper.to_string() })
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

#[cfg(test)]
mod tests {
    use super::*;

    /// 适配器把本叶接到统一契约上：元数据齐备、能出盘、确定性谱已声明、
    /// 有流派时恰有一个默认。
    #[test]
    fn adapter_is_wired_to_the_contract() {
        let e = MeihuaEngine;
        assert!(!e.id().is_empty() && !e.name().is_empty());
        let m = Moment::new(1990, 6, 15, 14, 30, 8.0);
        let q = Query::at(1990, 6, 15, 14, 30, 8.0);
        assert!(!e.cast(&m, &q).is_null(), "每片叶都应产出非空盘面");
        assert!(!e.profile().is_empty(), "每片叶都要显式声明确定性谱");
        let defaults = e.schools().iter().filter(|s| s.default).count();
        assert!(e.schools().is_empty() || defaults == 1, "有流派的叶应恰有一个默认");
    }
}

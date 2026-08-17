//! 本叶对 [`mingli_contract::CastingEngine`] 的实现——把叶的领域计算适配成
//! 全树统一的排盘契约，并声明本叶的确定性边界与流派。

use mingli_contract::{d, CastingEngine, DetItem, Determinism, Family, Intent, Moment, Principal, Query};
use serde_json::Value;

/// 巴厘 Pawukon 叶（A 族·多并行週）。210 上的十个 wewaran。
#[derive(Debug, Default)]
pub struct PawukonEngine;

/// 本次查询下的盘。
///
/// `cast` 与 `principal` 都从这里取：一个把它整份序列化，一个读它的一个字段。
/// 分出来是为了让后者不必去解前者产出的 JSON——字段改名时，读结构体会编译报错，解 JSON 不会。
fn chart(_e: &PawukonEngine, m: &Moment, _q: &Query) -> crate::Cast {
crate::compute_at(m)
}

impl CastingEngine for PawukonEngine {
    fn id(&self) -> &'static str {
        "pawukon"
    }
    fn name(&self) -> &'static str {
        "巴厘Pawukon"
    }
    fn family(&self) -> Family {
        Family::Cyclic
    }
    fn cast(&self, m: &Moment, q: &Query) -> Value {
        serde_json::to_value(chart(self, m, q)).unwrap_or(Value::Null)
    }
    fn reading_notes(&self) -> Option<&'static str> {
        Some("\n【字段语义提示（巴厘 Pawukon 历，十週并行）】\n\
            - `wuku`：三十个七日週之一，210 日一轮，是本历的骨架。`day` 是週期内第几日。\n\
            - 十个并行週：`ekawara`(1) `dwiwara`(2) `triwara`(3) `caturwara`(4) `pancawara`(5)\
              `sadwara`(6) `saptawara`(7) `astawara`(8) `sangawara`(9) `dasawara`(10)。\
              **210 = 2·3·5·7**，故各週在 210 日上同时归位，这是它能并行的数理由来。\n\
            - `urip`：各週的数值权重之和，`dasawara` 等派生週由它推出。\n\
            - `ekawara` 可为 null：一日週不是每日都有，取决于 urip 的奇偶。\n\
            - 巴厘的节庆与择日主要看 wuku 与 pancawara / saptawara 的组合；\
              但具体宜忌属查表，本叶只出历日结构。\n\
            - **读法**：先说 wuku，再挑五日週与七日週，其余各週点到为止。")
    }
    fn answers(&self) -> &'static [Intent] {
        &[Intent::Natal]
    }
    fn principal(&self, m: &Moment, q: &Query) -> Option<Principal> {
        // 五日週——Pawukon 十週中最常用的一支。
        let c = chart(self, m, q);
        Some(Principal { label: "Pancawara", value: c.pancawara.to_string() })
    }
    fn profile(&self) -> &'static [DetItem] {
        use Determinism::Det;
        const { &[
            d("十週（简单/派生/卡日）", Det, "210=2·3·5·7，锚 day0=2020-07-05 校验 Galungan"),
            d("Pancawara/Saptawara urip 权重表", Det, "5 独立源逐值一致：Babad Bali 本地权威表 / en.wikipedia Pawukon / Reingold-Dershowitz 参考实现 / sakacalendar / balinese-date-js-lib"),
            d("Ekawara/Dwiwara 奇偶向", Det, "urip 之和为奇 → Luang + Pepet，为偶 → 无 Ekawara + Menga；6 独立源同向。唯一相反记载(sejarahharirayahindu)与 sastrabali 同文转载且自身 Eka/Dwi 互相矛盾，判为讹误"),
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
        let e = PawukonEngine;
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

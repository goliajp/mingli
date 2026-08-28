//! 本叶对 [`mingli_contract::CastingEngine`] 的实现——把叶的领域计算适配成
//! 全树统一的排盘契约，并声明本叶的确定性边界与流派。

use mingli_contract::{d, CastingEngine, DetItem, Determinism, Family, Intent, Moment, Principal, Query};
use serde_json::Value;

/// 玛雅历叶（A 族·CRT）。Tzolkʼin 260 + Haab 365 + Long Count。
#[derive(Debug, Default)]
pub struct MayaEngine;

/// 本次查询下的盘。
///
/// `cast` 与 `principal` 都从这里取：一个把它整份序列化，一个读它的一个字段。
/// 分出来是为了让后者不必去解前者产出的 JSON——字段改名时，读结构体会编译报错，解 JSON 不会。
fn chart(_e: &MayaEngine, m: &Moment, _q: &Query) -> crate::Cast {
crate::compute_at(m)
}

impl CastingEngine for MayaEngine {
    fn id(&self) -> &'static str {
        "maya"
    }
    fn name(&self) -> &'static str {
        "玛雅历"
    }
    fn family(&self) -> Family {
        Family::Cyclic
    }
    fn cast(&self, m: &Moment, q: &Query) -> Value {
        serde_json::to_value(chart(self, m, q)).unwrap_or(Value::Null)
    }
    fn reading_notes(&self) -> Option<&'static str> {
        Some("\n【字段语义提示（玛雅三套历法）】\n\
            - `tzolkin_number` / `tzolkin_name`：卓尔金历，13 数 × 20 名 = 260 日一轮。\
              数与名各自独立推进，这是中美洲历法的核心结构。`tzolkin_round` 是第几轮。\n\
            - `haab_day` / `haab_month`：哈布历，18 月 × 20 日 + 5 日（Wayeb）= 365 日。\n\
            - `long_count[5]`：长纪历，自 baktun 至 kin 五级（20 进制，唯 winal 取 18）。\n\
            - `jdn`：儒略日数，三套历法都由它折算，故彼此严格同步。\n\
            - 卓尔金与哈布合起来 52 年一循环（历法轮）；本盘只出日期，不出日名的吉凶象义——\
              那属查表且各地传统不一，本叶不收。\n\
            - **读法**：三套并列说清即可，重点在卓尔金的数与名。")
    }
    fn answers(&self) -> &'static [Intent] {
        &[Intent::Natal]
    }
    fn principal(&self, m: &Moment, q: &Query) -> Option<Principal> {
        // Tzolkʼin 的 13 数。
        let c = chart(self, m, q);
        Some(Principal { label: "Tzolkʼin 数", value: c.tzolkin_number.to_string() })
    }
    fn profile(&self) -> &'static [DetItem] {
        use Determinism::Det;
        const { &[d("Tzolkʼin·Haab·Long Count", Det, "GMT 历元 584283，校验 0.0.0.0.0 与 2012-12-21 双锚")] }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 适配器把本叶接到统一契约上：元数据齐备、能出盘、确定性谱已声明、
    /// 有流派时恰有一个默认。
    #[test]
    fn adapter_is_wired_to_the_contract() {
        let e = MayaEngine;
        assert!(!e.id().is_empty() && !e.name().is_empty());
        let m = Moment::new(1990, 6, 15, 14, 30, 8.0);
        let q = Query::at(1990, 6, 15, 14, 30, 8.0);
        assert!(!e.cast(&m, &q).is_null(), "每片叶都应产出非空盘面");
        assert!(!e.profile().is_empty(), "每片叶都要显式声明确定性谱");
        let defaults = e.schools().iter().filter(|s| s.default).count();
        assert!(e.schools().is_empty() || defaults == 1, "有流派的叶应恰有一个默认");
    }
}

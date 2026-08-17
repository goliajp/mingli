//! 本叶对 [`mingli_contract::CastingEngine`] 的实现——把叶的领域计算适配成
//! 全树统一的排盘契约，并声明本叶的确定性边界与流派。

use mingli_contract::{d, CastingEngine, DetItem, Determinism, Family, Intent, Moment, Principal, Query};
use serde_json::Value;

/// 藏历循环叶（A 族）。60 周期（元素×生肖）+ 年 mewa。
#[derive(Debug, Default)]
pub struct TibetanEngine;

/// 本次查询下的盘。
///
/// `cast` 与 `principal` 都从这里取：一个把它整份序列化，一个读它的一个字段。
/// 分出来是为了让后者不必去解前者产出的 JSON——字段改名时，读结构体会编译报错，解 JSON 不会。
fn chart(_e: &TibetanEngine, m: &Moment, _q: &Query) -> crate::Cast {
crate::compute_at(m)
}

impl CastingEngine for TibetanEngine {
    fn id(&self) -> &'static str {
        "tibetan"
    }
    fn name(&self) -> &'static str {
        "藏历循环"
    }
    fn family(&self) -> Family {
        Family::Cyclic
    }
    fn cast(&self, m: &Moment, q: &Query) -> Value {
        serde_json::to_value(chart(self, m, q)).unwrap_or(Value::Null)
    }
    fn answers(&self) -> &'static [Intent] {
        &[Intent::Natal]
    }
    fn principal(&self, m: &Moment, q: &Query) -> Option<Principal> {
        // 六十週期的十二生肖位。
        let c = chart(self, m, q);
        Some(Principal { label: "生肖", value: c.animal.to_string() })
    }
    fn profile(&self) -> &'static [DetItem] {
        use Determinism::{Det, Und};
        const { &[
            d("60 周期·年 mewa", Det, "5 元素×12 生肖；mewa 逆行，校验 2024=木阳龙·mewa3"),
            d("年 parkha（不存在，非未实现）", Und, "🟡 查证结论是主流藏历确实不给年份配卦，故本叶不出。原典《白琉璃》与洛钦《月光疏》（Gyurme Dorje 英译 2001）把八卦的用法列全为：babs-spar（按年龄性别推的现行卦）、skye-spar（由母亲年龄推的本命卦）、天/地/敌门、历日卦、时辰卫——无年卦。Berzin 原话「Except for in the Bon variation ... there are no transiting annual trigrams」；Janson《Tibetan Calendar Mathematics》Appendix E 的 E.1 年 / E.2 月无 Trigram 小节而 E.3 阴历日 / E.4 历日才有；Henning 的 tibcalendar 里 get_year_astro_data() 也只填 rabjung/animal/element/gender/sme_ba。苯教有年卦一说仅 Berzin 单源。未取到实体 lo tho 年历样本核对"),
            d(
                "历日 parkha",
                Det,
                "Janson《Tibetan Calendar Mathematics》E.4：`(JD + 2) amod 8`（其 Table 15 次第）。\
                 与元素 / 性别 / 生肖 / 数四者同为简单循环，周期 10 / 2 / 12 / 8 / 9",
            ),
            d(
                "阴历日 parkha 的公式",
                Det,
                "Janson E.10：`(D + 30(A−3)) amod 8`，A 取该月生肖序。原文另以枚举复述同一规则\
                 （寅午戌起 Li、卯未亥起 Zin、子辰申起 Kham、丑巳酉起 Da），两处互为校验，测试逐条对过。\
                 吃生肖而非月号是有意的——月序到生肖的映射 Phugpa 与 Tsurphu 不同",
            ),
            d(
                "阴历日 parkha 的取日（未实现）",
                Und,
                "🟡 公式已落 Det，但要用它得先有藏历的阴历日与月生肖，而本 crate 不做阴历推步\
                 （无闰月 / 缺日）。故 `lunar_day_parkha` 由调用方给出这两个数；\
                 要本 crate 自出，须先实现 Phugpa 的朔望推步，那是另一件事",
            ),
            d("个人 parkha 的「斜跳」", Und, "🟡 真分歧：《白琉璃》/《月光疏》与 tibastro 主张数到十要斜跳（count + ⌊count/11⌋×3 再 mod 8），Berzin 与 Erlewine 的描述则是纯 mod 8 无斜跳。两说各有来源，未取得裁定依据"),
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
        let e = TibetanEngine;
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

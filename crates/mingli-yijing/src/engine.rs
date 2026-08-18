//! 本叶对 [`mingli_contract::CastingEngine`] 的实现——把叶的领域计算适配成
//! 全树统一的排盘契约，并声明本叶的确定性边界与流派。

use mingli_contract::{d, effective_seed, s, CastingEngine, DetItem, Determinism, Family, Intent, Moment, Principal, Query, SchoolItem};
use serde_json::Value;

/// 易经起卦叶（C 族）。三钱法、种子可复现。
#[derive(Debug, Default)]
pub struct YijingEngine;

/// 本次查询下的盘。
///
/// `cast` 与 `principal` 都从这里取：一个把它整份序列化，一个读它的一个字段。
/// 分出来是为了让后者不必去解前者产出的 JSON——字段改名时，读结构体会编译报错，解 JSON 不会。
fn chart(e: &YijingEngine, m: &Moment, q: &Query) -> crate::Cast {
    let method = match q.school_of(e.id(), "three_coins") {
        "yarrow_stalks" => crate::Method::YarrowStalks,
        _ => crate::Method::ThreeCoins,
    };
    crate::cast(method, effective_seed(m, q))
}

impl CastingEngine for YijingEngine {
    fn id(&self) -> &'static str {
        "yijing"
    }
    fn name(&self) -> &'static str {
        "易经起卦"
    }
    fn family(&self) -> Family {
        Family::Sampling
    }
    fn cast(&self, m: &Moment, q: &Query) -> Value {
        serde_json::to_value(chart(self, m, q)).unwrap_or(Value::Null)
    }
    fn reading_notes(&self) -> Option<&'static str> {
        Some("\n【字段语义提示（易经起卦）】\n\
            - `method`：起卦法（三钱 / 蓍草）。两法的爻值概率分布不同——\
              三钱四值等概、蓍草老阴老阳偏少——故变爻多寡的期望不同，这是体系差异不是误差。\n\
            - `lines[6]`：六爻，自下（初爻）至上。值为 6 老阴、7 少阳、8 少阴、9 老阳；\
              **老阴老阳为变爻**，少阴少阳不变。\n\
            - `primary_*` 本卦、`resulting_*` 之卦：`_upper` / `_lower` 上下卦、`_name` 卦名、\
              `_king_wen` 通行序号。无变爻时之卦同本卦。\n\
            - `changing_mask`：变爻的位掩码（自初爻起，置位即该爻变）。变爻的多寡决定取何断辞\
              （一爻变看该爻、多爻变另有取法），取法各家不一，本盘只出结构不代为选择。\n\
            \
            - **读法**：先说本卦与变爻，再说之卦；卦爻辞属经文，本盘不出，要引须自行注明出处。")
    }
    fn answers(&self) -> &'static [Intent] {
        &[Intent::Natal, Intent::Event]
    }
    fn principal(&self, m: &Moment, q: &Query) -> Option<Principal> {
        // 本卦下卦——内卦为主。
        let c = chart(self, m, q);
        Some(Principal { label: "下卦", value: c.primary_lower.to_string() })
    }
    fn profile(&self) -> &'static [DetItem] {
        use Determinism::{Det, Sto};
        const { &[
            d(
                "两法的爻值分布",
                Det,
                "三钱 1/8·3/8·3/8·1/8 是 B(3,½) 的直接推论；蓍草 3/16·5/16·7/16·1/16 由四营十八变的\
                 分策规则决定。两组都可自行复算，不依赖任何一家的说法，测试各以八万次抽样对过。\
                 两法分布不同是体系差异——蓍草的变爻更偏老阳",
            ),
            d(
                "卦的代数（本卦 / 之卦 / 上下卦 / 文王序）",
                Det,
                "不在本叶：六十四卦格与文王序住在 mingli-gua，那里有锚点 oracle、双射校验与配对性质三重把关。\
                 本叶只负责把抽样结果落成六爻",
            ),

            d("六爻·本卦/之卦", Sto, "种子可复现；两流派概率分布 1/8·3/8·3/8·1/8（三钱）、1/16·5/16·7/16·3/16（蓍草） 均校验"),
            d("六十四卦名 + 文王序", Det, "三源校验（ctext《序卦传》/zh.wiki/en.wiki），「二二相耦」定理（4 纯错对 + 28 综对）穷举证明"),
        ] }
    }
    fn schools(&self) -> &'static [SchoolItem] {
        const { &[
            s("three_coins", "三钱法", true, "三枚铜钱六掷；概率 6：7：8：9 = 1：3：3：1（老阴/少阳/少阴/老阳）"),
            s("yarrow_stalks", "蓍草法", false, "五十蓍策十八变（模拟分布）；概率 6：7：8：9 = 1：5：7：3"),
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
        let e = YijingEngine;
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

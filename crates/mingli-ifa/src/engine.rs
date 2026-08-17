//! 本叶对 [`mingli_contract::CastingEngine`] 的实现——把叶的领域计算适配成
//! 全树统一的排盘契约，并声明本叶的确定性边界与流派。

use mingli_contract::{d, effective_seed, CastingEngine, DetItem, Determinism, Family, Intent, Moment, Principal, Query};
use serde_json::Value;

/// Ifá 叶（C 族）。双 figure→256 odu，种子可复现。
#[derive(Debug, Default)]
pub struct IfaEngine;

/// 本次查询下的盘。
///
/// `cast` 与 `principal` 都从这里取：一个把它整份序列化，一个读它的一个字段。
/// 分出来是为了让后者不必去解前者产出的 JSON——字段改名时，读结构体会编译报错，解 JSON 不会。
fn chart(_e: &IfaEngine, m: &Moment, q: &Query) -> crate::Odu {
crate::cast(effective_seed(m, q))
}

impl CastingEngine for IfaEngine {
    fn id(&self) -> &'static str {
        "ifa"
    }
    fn name(&self) -> &'static str {
        "Ifá"
    }
    fn family(&self) -> Family {
        Family::Sampling
    }
    fn cast(&self, m: &Moment, q: &Query) -> Value {
        serde_json::to_value(chart(self, m, q)).unwrap_or(Value::Null)
    }
    fn reading_notes(&self) -> Option<&'static str> {
        Some("\n【字段语义提示（西非约鲁巴 Ifá）】\n\
            - 一次占问出两个 figure，各四位：`right` / `right_name` / `right_marks` 与\
              `left` / `left_name` / `left_marks`。**约鲁巴传统先读右后读左**，\
              两者合成 256 odù 之一（`index` / `name`）。左右颠倒会得到另一个同样像样的 odù，故次序要紧。\n\
            - `*_marks[4]`：四行的单双画（true = 单画），自顶行起。\n\
            - `meji`：左右两 figure 相同时为真，即十六「主 odù」之一，传统上视为分量最重的一类。\n\
            - 🟡 十六主 odù 的排序无定本（Bascom 自列两套、另记二十一套在案）、\
              256 复合 odù 的名与经文三系拼写不同，故本盘按数值索引而非名次，且不发经文。见确定性谱。\n\
            - **读法**：说 odù 名与左右两 figure 即可；经文与断辞本盘不出，不要代拟。")
    }
    fn answers(&self) -> &'static [Intent] {
        &[Intent::Natal, Intent::Event]
    }
    fn principal(&self, m: &Moment, q: &Query) -> Option<Principal> {
        // 左半 figure——Ifá 先读右后读左，左为二。
        let c = chart(self, m, q);
        Some(Principal { label: "左 figure", value: c.left.to_string() })
    }
    fn profile(&self) -> &'static [DetItem] {
        use Determinism::{Det, Sto, Und};
        const { &[
            d("双 figure→256 odu", Sto, "八次二进制投掷由种子派生（同种子同 odù 可复现）；右四次成右半、左四次成左半，16×16 = (Z₂)⁴ 得 256 组合"),
            d("16 主 odù 的名与点阵", Det, "Bascom《Ifa Divination》(1969) Table 1(p.4) 与 Table 3(p.48) 为一手；en.wikipedia《Ifá》的 Yoruba 与 Fon 两张表、Commons《The Meji Odus》图、跨系统对照表四处逐条相符"),
            d("行序与左右序", Det, "Bascom Fig. 2 面板 A(p.41) 把八次投掷的编号直接画在八个位置上：顶行右→顶行左→次行右→…；正文 p.40-41 明记右(ọ̀tún)为长、复合名右名在前，并警告反过来是另一个 odù。ifa-odu.com 独立表述一致；Fon/贝宁 Fa 传统同理由"),
            d("16 主 odù 的排序", Und, "🟡 无定本。Bascom Table 3 自己就并列 A.Ifẹ̀ 与 B.Southwestern Yoruba 两套(差异在第 5–8 与 11–14 位)，p.47 记明另有二十一套排序在案。故本叶按数值索引而非名次排，不发布任何「第 N 号 odù」"),
            d("256 复合 odù 的名与经文", Und, "🟡 尼日利亚 / 古巴 Lucumí / 贝宁三系拼写系统性不同(Ogbe-Iwori 在古巴作 Ogbe Weñe)，缩合形式因 lineage 而异；未取得多源一致的全表。本叶只按「右名 + 左名」拼出复合名，不发经文"),
            d("0/1 整数编码", Und, "🟡 传统只写单画 / 双画，不写数值。本叶取「置位 = 单画、bit0 = 顶行」为内部表示；可引用的原始写法是 Bascom 的四位 1/2 串(见 `bascom_notation`)"),
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
        let e = IfaEngine;
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

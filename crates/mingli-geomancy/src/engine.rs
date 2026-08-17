//! 本叶对 [`mingli_contract::CastingEngine`] 的实现——把叶的领域计算适配成
//! 全树统一的排盘契约，并声明本叶的确定性边界与流派。

use mingli_contract::{d, effective_seed, CastingEngine, DetItem, Determinism, Family, Moment, Query};
use serde_json::Value;

/// 地占叶（C 族）。4 母图→盾牌图，种子可复现，法官恒为偶。
#[derive(Debug, Default)]
pub struct GeomancyEngine;

impl CastingEngine for GeomancyEngine {
    fn id(&self) -> &'static str {
        "geomancy"
    }
    fn name(&self) -> &'static str {
        "地占"
    }
    fn family(&self) -> Family {
        Family::Sampling
    }
    fn cast(&self, m: &Moment, q: &Query) -> Value {
        serde_json::to_value(crate::cast(effective_seed(m, q))).unwrap_or(Value::Null)
    }
    fn profile(&self) -> &'static [DetItem] {
        use Determinism::{Det, Sto, Und};
        const { &[
            d("四母→盾牌图", Sto, "四母图由种子随机起（SplitMix64，同种子同盘可复现）；其余十二图由母图经 GF(2) 转置与 XOR 完全决定，无二次随机"),
            d("法官恒为偶", Det, "GF(2) 线性，穷举证于 core::gf2"),
            d("16 图名与点阵", Det, "三源一致：Unicode 提案 L2/23-218(已入 Unicode 17.0，给出编码规则原句) · Princeton「Medieval Geomancy」图版(Martin of Spain《De geomantia》英译) · en.wikipedia《Geomantic figures》(其偶/奇各 8、对称 4、进出各 6 四张分类表与点阵全维度自洽)"),
            d("行星 / 星座归属", Und, "🟡 Puer 与 Puella 的归属两派相反：Martin of Spain 从 Moerbeke/Cremona 作 Puer=金星双子、Puella=火星天秤；Agrippa 与现代主流作 Puer=火星白羊、Puella=金星天秤。两派的名↔点阵映射一致，只有星占字段冲突"),
            d("阿拉伯名", Und, "🟡 名集 15/16 两源相符(Savage-Smith & Smith 1980 据大英博物馆 13 世纪铜盘 · The Digital Ambler)，但同一图常有多个并行名(ʿuqla 亦作 thikāf、inkis 亦作 mankūs/nākis/rakīza 等)，且 Puer 一图两源给出不同名(jawdala / faraḥ)；单值入库会失真"),
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
        let e = GeomancyEngine;
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

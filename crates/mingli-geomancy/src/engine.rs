//! 本叶对 [`mingli_contract::CastingEngine`] 的实现——把叶的领域计算适配成
//! 全树统一的排盘契约，并声明本叶的确定性边界与流派。

use mingli_contract::{d, effective_seed, CastingEngine, DetItem, Determinism, Family, Intent, Moment, Principal, Query};
use serde_json::Value;

/// 地占叶（C 族）。4 母图→盾牌图，种子可复现，法官恒为偶。
#[derive(Debug, Default)]
pub struct GeomancyEngine;

/// 本次查询下的盘。
///
/// `cast` 与 `principal` 都从这里取：一个把它整份序列化，一个读它的一个字段。
/// 分出来是为了让后者不必去解前者产出的 JSON——字段改名时，读结构体会编译报错，解 JSON 不会。
fn chart(_e: &GeomancyEngine, m: &Moment, q: &Query) -> crate::Reading {
crate::cast(effective_seed(m, q))
}

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
        serde_json::to_value(chart(self, m, q)).unwrap_or(Value::Null)
    }
    fn reading_notes(&self) -> Option<&'static str> {
        Some("\n【字段语义提示（地占 ʿilm al-raml）】\n\
            - 十六图形各由四行点数组成，每行一点或两点，故 `*_marks` 是四个布尔（true = 单点）。\n\
            - 盾牌四层：`mothers[4]` 四母（种子所生）→ `daughters[4]` 四女（母之转置）\
              → `nieces[4]` 四侄（两两 XOR）→ `witnesses[2]` 两证 → `judge` 法官。\
              **整套是 GF(2) 上的逐层异或**，故结构完全确定，随机只在四母。\n\
            - `judge` 恒为偶图形（四行点数之和为偶）——这是异或的奇偶守恒定理，不是巧合，\
              可作盘面自检：法官若为奇，必是算错。\n\
            - `names`：各位置的图名（`mothers` / `daughters` / `nieces` / `witnesses` / `judge` 各一组拉丁名），与上面的点阵一一对应。🟡 阿拉伯名同一图常有多个并行名，本盘不强选，见确定性谱。\n\
            \
            - **读法**：法官为结论、两证为左右势，四母交代起因；说这三层即可，不必列全十六图。")
    }
    fn answers(&self) -> &'static [Intent] {
        &[Intent::Natal, Intent::Event]
    }
    fn principal(&self, m: &Moment, q: &Query) -> Option<Principal> {
        // 法官图形——盾牌的结论位。
        let c = chart(self, m, q);
        Some(Principal { label: "法官", value: c.judge.to_string() })
    }
    fn profile(&self) -> &'static [DetItem] {
        use Determinism::{Det, Sto, Und};
        const { &[
            d("四母→盾牌图", Sto, "四母图由种子随机起（SplitMix64，同种子同盘可复现）；其余十二图由母图经 GF(2) 转置与 XOR 完全决定，无二次随机"),
            d("法官恒为偶", Det, "GF(2) 线性，穷举证于 core::gf2"),
            d(
                "法官与 sikidy「创世者」是同一个量",
                Det,
                "两系同源于阿拉伯 ʿilm al-raml，本仓两片叶用的是**同一套 GF(2) 构造**：\
                 同一次取机导出同一组四母，于是本叶的「法官」与 sikidy 的「创世者」逐样本相同。\
                 720 样本实测 NMI = 1.0000，跨叶分析层有一条守卫钉住这一对。\
                 后果：同一次取机下同时问这两片，拿到的是同一组数换个命名，不是两份独立判断",
            ),
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
    }
}

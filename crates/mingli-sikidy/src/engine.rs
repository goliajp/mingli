//! 本叶对 [`mingli_contract::CastingEngine`] 的实现——把叶的领域计算适配成
//! 全树统一的排盘契约，并声明本叶的确定性边界与流派。

use mingli_contract::{d, effective_seed, CastingEngine, DetItem, Determinism, Family, Intent, Moment, Principal, Query};
use serde_json::Value;

/// Sikidy 叶（C 族）。4 母列→16 列，种子可复现，C15 恒为偶。
#[derive(Debug, Default)]
pub struct SikidyEngine;

/// 本次查询下的盘。
///
/// `cast` 与 `principal` 都从这里取：一个把它整份序列化，一个读它的一个字段。
/// 分出来是为了让后者不必去解前者产出的 JSON——字段改名时，读结构体会编译报错，解 JSON 不会。
fn chart(_e: &SikidyEngine, m: &Moment, q: &Query) -> crate::Reading {
crate::cast(effective_seed(m, q))
}

impl CastingEngine for SikidyEngine {
    fn id(&self) -> &'static str {
        "sikidy"
    }
    fn name(&self) -> &'static str {
        "Sikidy"
    }
    fn family(&self) -> Family {
        Family::Sampling
    }
    fn cast(&self, m: &Moment, q: &Query) -> Value {
        serde_json::to_value(chart(self, m, q)).unwrap_or(Value::Null)
    }
    fn answers(&self) -> &'static [Intent] {
        &[Intent::Natal, Intent::Event]
    }
    fn principal(&self, m: &Moment, q: &Query) -> Option<Principal> {
        // 创世者列。
        let c = chart(self, m, q);
        Some(Principal { label: "创世者", value: c.seer.to_string() })
    }
    fn profile(&self) -> &'static [DetItem] {
        use Determinism::{Det, Sto, Und};
        const { &[
            d("四母→16 列", Sto, "四母列由种子随机起（同种子同盘可复现）；其余十二列由母列经转置与 XOR 树完全决定，与地占共用同一套 GF(2) 代数"),
            d("创世者 C15 恒偶", Det, "与地占法官同一定理"),
            d("16 列的角色与编号", Det, "三源交叉印证(Ascher 1997 Historia Mathematica · Chemillier 2007 L'Homme · Dahle-Sibree 1892 Folk-Lore)；Dahle 用马语列名写的八条生成规则与本叶公式逐条对上。本叶用 Ascher 生成序编号，创世者列在生成序为第 15、空间序为第 12——英文维基条目内部混用两套，勿照抄"),
            d("第 6 与第 14 列的语义", Und, "🟡 三源三说：第 6 作 the bad intentions / abily(奴隶) / Marìna；第 14 作 the people / saily / Mpànontàny(发问者)。不硬选一说，两处留空"),
            d("16 个图（四行点阵）的马语名", Und, "🟡 Sibree 1892 自己就并列了 Hova / Sakalava / 东非阿拉伯商人三套互不相同的命名(如同一图作 Jamà/Asombòla/Asombòla)，未取得第二个独立来源"),
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
        let e = SikidyEngine;
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

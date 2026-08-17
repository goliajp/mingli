//! 本叶对 [`mingli_contract::CastingEngine`] 的实现——把叶的领域计算适配成
//! 全树统一的排盘契约，并声明本叶的确定性边界与流派。

use mingli_contract::{d, CastingEngine, DetItem, Determinism, Family, Intent, Moment, Principal, Query};
use serde_json::Value;

/// 缅甸 Mahabote 叶（A 族）。本命核心数 = （缅历年 − 星期） mod 7。
#[derive(Debug, Default)]
pub struct MahaboteEngine;

/// 本次查询下的盘。
///
/// `cast` 与 `principal` 都从这里取：一个把它整份序列化，一个读它的一个字段。
/// 分出来是为了让后者不必去解前者产出的 JSON——字段改名时，读结构体会编译报错，解 JSON 不会。
fn chart(_e: &MahaboteEngine, m: &Moment, _q: &Query) -> crate::Cast {
crate::compute_at(m)
}

impl CastingEngine for MahaboteEngine {
    fn id(&self) -> &'static str {
        "mahabote"
    }
    fn name(&self) -> &'static str {
        "缅甸Mahabote"
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
        // 本命核心数所落之宫。
        let c = chart(self, m, q);
        Some(Principal { label: "本命宫", value: c.house.to_string() })
    }
    fn profile(&self) -> &'static [DetItem] {
        use Determinism::{Det, Und};
        const { &[
            d("核心数·七宫·八天週行星", Det, "（缅历年−星期） mod 7，校验 2000-01-01=Adipati"),
            d("宫间关系", Und, "🟡 真单源。Grand Trine / Minor Trine / Square / Core / Cardinal Points 五套几何只见于 Barbara Cameron《MaHaBote, the Little Key》一脉：其学生 Sage Asita 的教学页与所附五图、荷兰 DIRAH 函授课、Scribd 两份转抄——四家英文名逐字相同、示例盘同构，判为同源。缅语侧查过缅文维基《မဟာဘုတ်》（只给三行盘面与顺时针盘序、不涉关系）与六个缅甸开源实现（一律只算宫位），一条都没有。另注：Cameron 讲的「友敌生克」是**行星之间**，不是宫之间"),
            d("七宫含义", Und, "🟡 两系互证的只有三宫：Adipati（领袖 / 善言辞）、Atun（声誉 / 勤勉）、Marana（极端 / 无中间地带）。**Thike 一宫两说相反**——Cameron 作 House of Wealth，而缅文 zatas.ts 的 သိုက်ဖွား 条通篇讲缺钱负债劳而无获（သိုက် 字面即「埋在地下的宝藏」）。Binga 与 Yaza 两系交集过小，缅文维基则不给任何含义。故整体不出"),
            d("盘面几何与吉凶二分", Det, "缅文维基的三行 wikitable（顶 အဓိပတိ；中 အထွန်း|သိုက်|ရာဇ；底 မရဏ|ဘင်္ဂ|ပုတိ）与 Cameron 盘图（顶 7；中 3|4|5；底 2|1|6，上两排绿底、底排橙底）逐格重合，两条源流互不相干。吉凶二分另有巴利词源独立佐证：bhaṅga 坏灭 / maraṇa 死 / pūti 腐归凶，rāja 王 / adhipati 主宰 / htun 光耀 / thike 埋藏之宝归吉"),
            d("HOUSES 是索引序不是盘序", Det, "本叶的 HOUSES 配的是 (缅历年−星期) mod 7 的索引序，与盘面顺时针序差「每步 +2」（HOUSES[k] == BOARD[(2k) mod 7]，七项已验）。现只输出本命宫名故无碍；若日后要出整张盘或宫位坐标，直接拿它当盘序会错位"),
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
        let e = MahaboteEngine;
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

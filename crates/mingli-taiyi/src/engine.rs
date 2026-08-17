//! 本叶对 [`mingli_contract::CastingEngine`] 的实现——把叶的领域计算适配成
//! 全树统一的排盘契约，并声明本叶的确定性边界与流派。

use mingli_contract::{d, CastingEngine, DetItem, Determinism, Family, Moment, Query};
use serde_json::Value;

/// 太乙神数叶（⟂ 横切）。太乙积年 → 太乙行八宫（三年一宫·阳顺阴逆）+ 三才。
#[derive(Debug, Default)]
pub struct TaiyiEngine;

impl CastingEngine for TaiyiEngine {
    fn id(&self) -> &'static str {
        "taiyi"
    }
    fn name(&self) -> &'static str {
        "太乙神数"
    }
    fn family(&self) -> Family {
        Family::CrossCutting
    }
    fn cast(&self, m: &Moment, _q: &Query) -> Value {
        serde_json::to_value(crate::compute_at(m)).unwrap_or(Value::Null)
    }
    fn profile(&self) -> &'static [DetItem] {
        use Determinism::{Det, Und};
        const { &[
            d("太乙行八宫·三才·积年", Det, "三年一宫·廿四年一周，积年锚《金镜式经》724=1937281"),
            d("文昌（主目）", Det, "《太乙金镜式经》卷一「推天目所在法」：积年以十八去之，命起武德顺行十六神，遇阴德、大武重留一算；阴遁命起吕申、重留和德大炅（《太乙统宗宝鉴》卷一）。两双计位各占两算故周法十八"),
            d("计神", Det, "阳遁起寅、阴遁起申，逆行十二辰，不入四维。两部原典均载"),
            d("始击（客目）", Det, "《金镜式经》卷二「以計神加和徳宫，求文昌所臨宫」；《统宗》卷二「計神既加和徳之宫，却視天上文昌所臨之下，而為始擊神也」。即 始击 =（文昌 + 和德 − 计神）mod 16"),
            d("主算 / 客算 / 大将 / 参将", Det, "自目位顺行累加沿途正宫宫数至太乙宫止：起点正宫计其宫数、间神计 1，间神不累加，终点太乙宫不计入（《金镜式经》卷二第八条 +《统宗》卷二第七条）；「去十用零」整十者以九去之，参将 = 大将三因后再去十（《统宗》卷二第八条）。诸将可落中五，太乙自身不游中五"),
            d("太乙九宫配法", Det, "乾 1 · 离 2 · 艮 3 · 震 4 · 中 5 · 兑 6 · 坤 7 · 坎 8 · 巽 9，**与洛书不同**（洛书一宫是坎）。三源一致，且由算例反证：局 11 客算得 4 只在此配法下成立"),
            d("校验", Det, "两则纪年实例全字段吻合且出自两部不同的书：唐天復二年（局 11，太乙 4 宫，文昌高丛，始击阳德，客算 4）与秦二世二年（局 55，太乙 3 宫，文昌武德，始击和德）；三个纪年锚点（开元十二年局 49 / 天復二年局 11 / 秦二世二年局 55）在同一 72 局环上自洽"),
            d("144 局立成表全表校验", Und, "🟡 《太乙金镜式经》卷三载《陽局立成》《隂局立成》各 72 局逐局给全字段，是原典自带的黄金校验集；本叶尚未逐格录入，目前只以两则纪年实例 + 结构不变量把关"),
            d("定计目（及定算 / 定大将 / 定参将）", Und, "🟡 只见《太乙统宗宝鉴》卷二第九条；《太乙金镜式经》自称「運式之儀有八」，无此条。两书的真实差异就在这一项，单源不实现"),
            d("君臣民基 / 大游小游 / 四神 / 十精等其余诸神", Und, "🟡 现只做了二目一系（文昌 / 始击 → 主客算 → 大将 / 参将），这一系两部原典明载且有纪年实例可校。其余诸神的起法未查，既未确认多源也未确认分歧——是「没查」而不是「查了定不下」，两者性质不同，此处如实记前者"),
            d("落宫绝对相位", Det, "由三个纪年锚点交叉锁定，见上"),
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
        let e = TaiyiEngine;
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

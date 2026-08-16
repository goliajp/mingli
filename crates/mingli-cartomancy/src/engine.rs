//! 本叶对 [`mingli_contract::CastingEngine`] 的实现——把叶的领域计算适配成
//! 全树统一的排盘契约，并声明本叶的确定性边界与流派。

use mingli_contract::{d, effective_seed, s, CastingEngine, DetItem, Determinism, Family, Moment, Query, SchoolItem};
use serde_json::Value;

/// 抽牌叶（C 族）。schools 暴露五种 deck：塔罗 78 / 大阿卡纳 22 / Lenormand 36 /
/// Elder Futhark 24 / Younger Futhark 16。统一三张牌阵，种子可复现。
#[derive(Debug, Default)]
pub struct TarotEngine;

impl CastingEngine for TarotEngine {
    fn id(&self) -> &'static str {
        "tarot"
    }
    fn name(&self) -> &'static str {
        "塔罗"
    }
    fn family(&self) -> Family {
        Family::Sampling
    }
    fn cast(&self, m: &Moment, q: &Query) -> Value {
        // `tarot_full_marseilles` / `tarot_major_marseilles` 复合 id 同时编码 deck + Tarot 流派。
        let school = q.school_of(self.id(), "tarot_full");
        let (deck_id, order) = match school {
            "tarot_full_marseilles" => ("tarot_full", crate::TarotOrder::Marseilles),
            "tarot_major_marseilles" => ("tarot_major", crate::TarotOrder::Marseilles),
            id => (id, crate::TarotOrder::RiderWaite),
        };
        let deck = crate::Deck::from_id(deck_id)
            .unwrap_or(crate::Deck::TarotFull);
        serde_json::to_value(crate::draw_deck_with_order(deck, order, 3, effective_seed(m, q)))
            .unwrap_or(Value::Null)
    }
    fn schools(&self) -> &'static [SchoolItem] {
        const {
            &[
                s("tarot_full", "塔罗 78 (RWS)", true, "Rider-Waite-Smith 1909：8=Strength/11=Justice；Major 22 + Minor 56，允许逆位"),
                s("tarot_full_marseilles", "塔罗 78 (Marseilles)", false, "Tarot de Marseille 传统：8=Justice/11=Strength；牌副同 RWS"),
                s("tarot_major", "塔罗大阿卡纳 22 (RWS)", false, "仅 Major Arcana，RWS 顺序"),
                s("tarot_major_marseilles", "塔罗大阿卡纳 22 (Marseilles)", false, "仅 Major，Marseilles 顺序（8/11 互换）"),
                s("lenormand", "Petit Lenormand 36", false, "传统不用逆位；Hechtel 1799 The Game of Hope 标准"),
                s("elder_futhark", "Elder Futhark 卢恩 24", false, "古日耳曼/古英，允许逆位；BabelStone Runic block U+16A0-U+16FF"),
                s("younger_futhark", "Younger Futhark 卢恩 16", false, "维京时期 Long-branch 简化卢恩，允许逆位"),
            ]
        }
    }
    fn profile(&self) -> &'static [DetItem] {
        use Determinism::Sto;
        const { &[
            d("抽牌·正逆位", Sto, "无放回 Fisher–Yates 置换 + 逆位 bit（Lenormand 除外），种子可复现"),
            d("牌副大小", Sto, "由 schools 选择：78/22/36/24/16 五种 deck × Tarot RWS/Marseilles 两派 = 7 组合"),
            d("牌名·中文译名·Unicode 字符", Sto, "多源校验入码：Tarot Major（en.wiki+zh.wiki+Biddy 3源）/Minor（花色×等级生成）/Lenormand（4源）/Futhark（BabelStone+Runic block 2源）"),
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
        let e = TarotEngine;
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

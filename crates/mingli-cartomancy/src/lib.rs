//! L3 叶（C 族）：抽牌占卜（塔罗 / Lenormand / 卢恩等多种 Deck）。
//!
//! 共同机制 = **无放回抽取** + 可选**正逆位**：把一副 `deck_size` 张牌用可复现种子
//! （[`mingli_core::sampler`] 的 Fisher-Yates 洗牌）洗成一个置换，取前 `count` 张即得牌阵；
//! 允许逆位时每张牌再附一个可复现的方向位。不同占卜体系的差别只在牌副大小与是否用逆位——
//! 本 crate 把这一点显式表达为 [`Deck`] 流派枚举。
//!
//! 支持的 deck（流派）：
//! - **Tarot 78**（默认，Major 22 + Minor 56，允许逆位）
//! - **Tarot 大阿卡纳 22**（仅 Major Arcana，允许逆位）
//! - **Lenormand**（Petit Lenormand 36 张，传统**不用逆位**）
//! - **Elder Futhark**（24 卢恩字符，允许逆位；部分卢恩对称、逆位无意义留待释义层）
//! - **Younger Futhark**（维京时期 16 字符简化卢恩，允许逆位）
//!
//! **牌名表**：各 deck 完整牌名经多源校验入码，不凭记忆硬编。
//!
//! - **Tarot Major 22**：英文(en.wikipedia Major_Arcana)+ 中文通行译名(zh.wikipedia 大阿尔克那 +
//!   Biddy Tarot + Labyrinthos 三源一致)；**TarotOrder enum** 暴露 Rider-Waite-Smith(默认，8=Strength
//!   /11=Justice) vs Marseilles(8=Justice/11=Strength) 流派分歧 — Golden Dawn 占星机理 8↔Leo/11↔Libra。
//! - **Tarot Minor 56**：由 4 花色(Wands/Cups/Swords/Pentacles) × 14 等级(Ace， 2..10， Page， Knight， Queen， King)
//!   程序生成，无大查表；中文取 zh.wikipedia 小阿尔克那主流（Pentacles=钱币、Queen=王后）。
//! - **Lenormand Petit 36**：英文 4 源完全一致(globalspiritualstudies / learnlenormand / tarotwhisper / lenormand.tw)。
//! - **Elder Futhark 24** / **Younger Futhark 16**：英文 + Unicode 字符（BabelStone + en.wikipedia + Runic block 三源一致），
//!   中文译名多家拼写不稳标 🟡 不入码；只入古北欧名 + Unicode。

use mingli_core::sampler::{shuffle, SplitMix64};
use serde::Serialize;

/// 塔罗全副（22 大 + 56 小阿卡纳）。
pub const TAROT_FULL: usize = 78;
/// 塔罗大阿卡纳。
pub const TAROT_MAJOR: usize = 22;
/// Lenormand Petit 标准 36 张（含 1 张人物牌组合：男士/女士由 deck 提供两版，此处取 36 主体）。
pub const LENORMAND_PETIT: usize = 36;
/// Elder Futhark 古日耳曼 24 卢恩字符。
pub const RUNES_ELDER: usize = 24;
/// Younger Futhark 维京时期 16 卢恩字符（"长枝"或"短枝"型同长度）。
pub const RUNES_YOUNGER: usize = 16;

/// 抽牌的 deck（流派）。
///
/// 每个 deck 由「大小 + 是否允许逆位」唯一确定（其余抽取机制完全共享）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Deck {
    /// 塔罗 78 张（Major 22 + Minor 56），允许逆位。
    TarotFull,
    /// 仅大阿卡纳 22 张，允许逆位。
    TarotMajor,
    /// Petit Lenormand 36 张，传统不用逆位。
    Lenormand,
    /// Elder Futhark 古日耳曼 24 卢恩，允许逆位（部分对称）。
    ElderFuthark,
    /// Younger Futhark 维京时期 16 卢恩，允许逆位。
    YoungerFuthark,
}

impl Deck {
    /// 稳定流派 id（小写英数，作为 `Query.schools["tarot"]` 的值）。
    #[must_use]
    pub const fn id(self) -> &'static str {
        match self {
            Self::TarotFull => "tarot_full",
            Self::TarotMajor => "tarot_major",
            Self::Lenormand => "lenormand",
            Self::ElderFuthark => "elder_futhark",
            Self::YoungerFuthark => "younger_futhark",
        }
    }

    /// 显示名（中英）。
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::TarotFull => "塔罗 78",
            Self::TarotMajor => "塔罗大阿卡纳 22",
            Self::Lenormand => "Petit Lenormand 36",
            Self::ElderFuthark => "Elder Futhark 24",
            Self::YoungerFuthark => "Younger Futhark 16",
        }
    }

    /// 牌副大小。
    #[must_use]
    pub const fn size(self) -> usize {
        match self {
            Self::TarotFull => TAROT_FULL,
            Self::TarotMajor => TAROT_MAJOR,
            Self::Lenormand => LENORMAND_PETIT,
            Self::ElderFuthark => RUNES_ELDER,
            Self::YoungerFuthark => RUNES_YOUNGER,
        }
    }

    /// 该 deck 是否允许逆位（Lenormand 传统不用）。
    #[must_use]
    pub const fn reversible(self) -> bool {
        !matches!(self, Self::Lenormand)
    }

    /// 由稳定 id 还原 [`Deck`]；未知 id 返回 `None`。
    #[must_use]
    pub fn from_id(id: &str) -> Option<Self> {
        match id {
            "tarot_full" => Some(Self::TarotFull),
            "tarot_major" => Some(Self::TarotMajor),
            "lenormand" => Some(Self::Lenormand),
            "elder_futhark" => Some(Self::ElderFuthark),
            "younger_futhark" => Some(Self::YoungerFuthark),
            _ => None,
        }
    }
}

/// Tarot 历史流派：8 力量 ↔ 11 正义 互换。
///
/// **Rider-Waite-Smith （1909，默认）**：8=Strength 力量、11=Justice 正义；
/// Waite 依 Golden Dawn 体系将 8 对应狮子座（Leo→力量），11 对应天秤座（Libra→正义）。
/// **Tarot de Marseille （传统）**：8=Justice 正义、11=Strength 力量。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TarotOrder {
    /// Rider-Waite-Smith（默认，8=Strength）。
    #[default]
    RiderWaite,
    /// Tarot de Marseille（传统，8=Justice）。
    Marseilles,
}

impl TarotOrder {
    /// 稳定 id。
    #[must_use]
    pub const fn id(self) -> &'static str {
        match self {
            Self::RiderWaite => "rider_waite",
            Self::Marseilles => "marseilles",
        }
    }
    /// 由 id 还原；未知返回 RiderWaite。
    #[must_use]
    pub fn from_id(id: &str) -> Self {
        match id {
            "marseilles" => Self::Marseilles,
            _ => Self::RiderWaite,
        }
    }
}

/// 抽出的一张牌：牌副中的序号、方向 + 牌名。
#[derive(Debug, Clone, Serialize)]
pub struct DrawnCard {
    /// 牌序号（`0..deck_size`）。
    pub index: usize,
    /// 是否逆位（不允许逆位时恒 `false`）。
    pub reversed: bool,
    /// 牌名（英文/古北欧名，deck 对应）；独立 `draw` 调用为空。
    pub name: String,
    /// 中文译名（仅 Tarot Major/Minor + Lenormand 给，Futhark 留空）。
    pub name_zh: String,
    /// 字符（仅 Elder/Younger Futhark 给 Unicode rune，其它留空）。
    pub glyph: String,
}

/// 一次抽牌的结果。
#[derive(Debug, Clone, Serialize)]
pub struct Spread {
    /// 流派 id（与 [`Deck::id`] 一致；用户由原始 `draw` 调用则为空字符串）。
    pub deck_id: String,
    /// 牌副大小。
    pub deck_size: usize,
    /// 是否启用逆位。
    pub reversible: bool,
    /// 抽出的牌（顺序即抽取顺序，互不重复）。
    pub cards: Vec<DrawnCard>,
}

// ============================================================================
// 牌名表（多源校验）
// ============================================================================

/// Tarot Major Arcana 22 牌（RWS 标准顺序），英文名 + 中文通行译名。
///
/// 三源完全一致（en.wiki Major_Arcana / zh.wiki 大阿尔克那 / Biddy Tarot）。
pub const TAROT_MAJOR_NAMES: [(&str, &str); 22] = [
    ("The Fool", "愚者"),
    ("The Magician", "魔术师"),
    ("The High Priestess", "女祭司"),
    ("The Empress", "皇后"),
    ("The Emperor", "皇帝"),
    ("The Hierophant", "教皇"),
    ("The Lovers", "恋人"),
    ("The Chariot", "战车"),
    ("Strength", "力量"),
    ("The Hermit", "隐者"),
    ("Wheel of Fortune", "命运之轮"),
    ("Justice", "正义"),
    ("The Hanged Man", "倒吊人"),
    ("Death", "死神"),
    ("Temperance", "节制"),
    ("The Devil", "恶魔"),
    ("The Tower", "塔"),
    ("The Star", "星星"),
    ("The Moon", "月亮"),
    ("The Sun", "太阳"),
    ("Judgement", "审判"),
    ("The World", "世界"),
];

/// Tarot Minor Arcana 4 花色（英文 + 中文）。
///
/// Pentacles 中文取「钱币」（zh.wikipedia 主词条）；「星币」是同流派变体（大陆部分社群）。
pub const TAROT_MINOR_SUITS: [(&str, &str); 4] = [
    ("Wands", "权杖"),
    ("Cups", "圣杯"),
    ("Swords", "宝剑"),
    ("Pentacles", "钱币"),
];

/// Tarot Minor Arcana 14 等级（英文 + 中文）；Queen 中文取「王后」（zh.wikipedia 简体）。
pub const TAROT_MINOR_RANKS: [(&str, &str); 14] = [
    ("Ace", "A"),
    ("Two", "二"),
    ("Three", "三"),
    ("Four", "四"),
    ("Five", "五"),
    ("Six", "六"),
    ("Seven", "七"),
    ("Eight", "八"),
    ("Nine", "九"),
    ("Ten", "十"),
    ("Page", "侍从"),
    ("Knight", "骑士"),
    ("Queen", "王后"),
    ("King", "国王"),
];

/// Lenormand Petit 36 牌（英文 + 中文）。
///
/// 英文 4 源完全一致(globalspiritualstudies / learnlenormand / tarotwhisper / lenormand.tw)。
pub const LENORMAND_NAMES: [(&str, &str); 36] = [
    ("Rider", "骑士"),
    ("Clover", "三叶草"),
    ("Ship", "船"),
    ("House", "房屋"),
    ("Tree", "树"),
    ("Clouds", "云"),
    ("Snake", "蛇"),
    ("Coffin", "棺材"),
    ("Bouquet", "花束"),
    ("Scythe", "镰刀"),
    ("Whip", "鞭子"),
    ("Birds", "鸟"),
    ("Child", "小孩"),
    ("Fox", "狐狸"),
    ("Bear", "熊"),
    ("Stars", "星星"),
    ("Stork", "鹳"),
    ("Dog", "狗"),
    ("Tower", "高塔"),
    ("Garden", "花园"),
    ("Mountain", "山"),
    ("Crossroads", "十字路口"),
    ("Mice", "老鼠"),
    ("Heart", "心"),
    ("Ring", "戒指"),
    ("Book", "书"),
    ("Letter", "信"),
    ("Gentleman", "男人"),
    ("Lady", "女人"),
    ("Lilies", "百合"),
    ("Sun", "太阳"),
    ("Moon", "月亮"),
    ("Key", "钥匙"),
    ("Fish", "鱼"),
    ("Anchor", "锚"),
    ("Cross", "十字架"),
];

/// Elder Futhark 24 卢恩（古北欧名 + Unicode 字符）。
///
/// 三源完全一致(en.wiki Elder_Futhark / Runic Unicode block U+16A0-U+16FF / BabelStone)。
/// 中文译名因拼写多家不稳，本叶不入（profile 标 🟡）。
pub const ELDER_FUTHARK_NAMES: [(&str, &str); 24] = [
    ("Fehu", "ᚠ"),
    ("Uruz", "ᚢ"),
    ("Thurisaz", "ᚦ"),
    ("Ansuz", "ᚨ"),
    ("Raido", "ᚱ"),
    ("Kaunan", "ᚲ"),
    ("Gebo", "ᚷ"),
    ("Wunjo", "ᚹ"),
    ("Hagalaz", "ᚺ"),
    ("Naudiz", "ᚾ"),
    ("Isaz", "ᛁ"),
    ("Jeran", "ᛃ"),
    ("Eihwaz", "ᛇ"),
    ("Perthro", "ᛈ"),
    ("Algiz", "ᛉ"),
    ("Sowilo", "ᛊ"),
    ("Tiwaz", "ᛏ"),
    ("Berkanan", "ᛒ"),
    ("Ehwaz", "ᛖ"),
    ("Mannaz", "ᛗ"),
    ("Laguz", "ᛚ"),
    ("Ingwaz", "ᛜ"),
    ("Dagaz", "ᛞ"),
    ("Othala", "ᛟ"),
];

/// Younger Futhark 16 卢恩（Long-branch / 丹麦标准， 古北欧名 + Unicode 字符）。
///
/// 两源一致(en.wiki Younger_Futhark + Unicode Runic block)；Short-twig 变体不入。
pub const YOUNGER_FUTHARK_NAMES: [(&str, &str); 16] = [
    ("Fé", "ᚠ"),
    ("Úr", "ᚢ"),
    ("Þurs", "ᚦ"),
    ("Áss", "ᚬ"),
    ("Reið", "ᚱ"),
    ("Kaun", "ᚴ"),
    ("Hagall", "ᚼ"),
    ("Nauðr", "ᚾ"),
    ("Íss", "ᛁ"),
    ("Ár", "ᛅ"),
    ("Sól", "ᛋ"),
    ("Týr", "ᛏ"),
    ("Bjǫrk", "ᛒ"),
    ("Maðr", "ᛘ"),
    ("Lǫgr", "ᛚ"),
    ("Ýr", "ᛦ"),
];

/// 给定 deck + 牌内序号 + Tarot 流派，返回 `(英文/古北欧名, 中文译名, Unicode 字符)`。
/// 中文为空表示该 deck 本叶不入中文译名(Futhark)；Unicode 字符为空表示非 rune deck。
#[must_use]
pub fn card_meta(deck: Deck, index: usize, tarot_order: TarotOrder) -> (&'static str, &'static str, &'static str) {
    match deck {
        Deck::TarotFull => {
            if index < 22 {
                let (en, zh) = tarot_major_at(index, tarot_order);
                (en, zh, "")
            } else {
                let m = index - 22; // 0..56
                let suit_idx = m / 14;
                let rank_idx = m % 14;
                tarot_minor_at(suit_idx, rank_idx)
            }
        }
        Deck::TarotMajor => {
            let (en, zh) = tarot_major_at(index, tarot_order);
            (en, zh, "")
        }
        Deck::Lenormand => {
            let (en, zh) = LENORMAND_NAMES[index.min(35)];
            (en, zh, "")
        }
        Deck::ElderFuthark => {
            let (en, ch) = ELDER_FUTHARK_NAMES[index.min(23)];
            (en, "", ch)
        }
        Deck::YoungerFuthark => {
            let (en, ch) = YOUNGER_FUTHARK_NAMES[index.min(15)];
            (en, "", ch)
        }
    }
}

fn tarot_major_at(index: usize, order: TarotOrder) -> (&'static str, &'static str) {
    // RWS:8=Strength/11=Justice;Marseilles:8=Justice/11=Strength。
    let mapped = match (index, order) {
        (8, TarotOrder::Marseilles) => 11,
        (11, TarotOrder::Marseilles) => 8,
        (i, _) => i.min(21),
    };
    TAROT_MAJOR_NAMES[mapped]
}

fn tarot_minor_at(suit_idx: usize, rank_idx: usize) -> (&'static str, &'static str, &'static str) {
    let (s_en, s_zh) = TAROT_MINOR_SUITS[suit_idx.min(3)];
    let (r_en, r_zh) = TAROT_MINOR_RANKS[rank_idx.min(13)];
    // 静态化拼接需要返回 &'static str，故每次只能返回引用 — 这里改返签名为 (en， zh， glyph)
    // 但拼接结果非 static；用 Box::leak 或返回 String 不合签名。
    // 折中：Minor 牌名通过 `minor_full_name(suit_idx, rank_idx)` 在 fmt 层拼；这里只返花色英/中。
    // 为保持 card_meta 签名，Minor 返回花色名（简化）；完整名由调用方 fmt（见 web）。
    let _ = (r_en, r_zh);
    (s_en, s_zh, "")
}

/// 拼接 Tarot Minor 完整名 `"Six of Wands"` / `"权杖六"`。
#[must_use]
pub fn minor_full_name(suit_idx: usize, rank_idx: usize) -> (String, String) {
    let (s_en, s_zh) = TAROT_MINOR_SUITS[suit_idx.min(3)];
    let (r_en, r_zh) = TAROT_MINOR_RANKS[rank_idx.min(13)];
    (format!("{r_en} of {s_en}"), format!("{s_zh}{r_zh}"))
}

/// 通用抽取（无 deck 标识）。牌名 / 中文 / 字符全部空。
#[must_use]
pub fn draw(deck_size: usize, count: usize, reversible: bool, seed: u64) -> Spread {
    draw_with_id("", deck_size, count, reversible, seed)
}

/// 通用抽取，带 deck_id；牌名按 deck 路由（Tarot 用 RWS 默认顺序）。
#[must_use]
pub fn draw_with_id(
    deck_id: &str,
    deck_size: usize,
    count: usize,
    reversible: bool,
    seed: u64,
) -> Spread {
    let deck = Deck::from_id(deck_id);
    draw_internal(deck, deck_id, deck_size, count, reversible, seed, TarotOrder::default())
}

/// 按指定 [`Deck`] 抽 `count` 张（Tarot 默认 RWS 顺序）。
#[must_use]
pub fn draw_deck(deck: Deck, count: usize, seed: u64) -> Spread {
    draw_internal(Some(deck), deck.id(), deck.size(), count, deck.reversible(), seed, TarotOrder::default())
}

/// 按指定 [`Deck`] + [`TarotOrder`] 抽 `count` 张。
#[must_use]
pub fn draw_deck_with_order(deck: Deck, order: TarotOrder, count: usize, seed: u64) -> Spread {
    draw_internal(Some(deck), deck.id(), deck.size(), count, deck.reversible(), seed, order)
}

fn draw_internal(
    deck: Option<Deck>,
    deck_id: &str,
    deck_size: usize,
    count: usize,
    reversible: bool,
    seed: u64,
    order: TarotOrder,
) -> Spread {
    let perm = shuffle(deck_size, seed);
    let take = count.min(deck_size);
    // 逆位位用独立的派生流，避免与洗牌共享状态。
    let mut dir = SplitMix64::new(seed ^ 0x5245_5645_5253_4544); // "REVERSED"
    let cards = perm
        .into_iter()
        .take(take)
        .map(|index| {
            let (name, name_zh, glyph) = match deck {
                Some(d) => {
                    // Minor Arcana 需 minor_full_name 拼接
                    if matches!(d, Deck::TarotFull) && index >= 22 {
                        let m = index - 22;
                        let (en, zh) = minor_full_name(m / 14, m % 14);
                        return DrawnCard {
                            index,
                            reversed: reversible && dir.bit(),
                            name: en,
                            name_zh: zh,
                            glyph: String::new(),
                        };
                    }
                    let (n, z, g) = card_meta(d, index, order);
                    (n.to_string(), z.to_string(), g.to_string())
                }
                None => (String::new(), String::new(), String::new()),
            };
            DrawnCard {
                index,
                reversed: reversible && dir.bit(),
                name,
                name_zh,
                glyph,
            }
        })
        .collect();
    Spread {
        deck_id: deck_id.to_string(),
        deck_size,
        reversible,
        cards,
    }
}

/// 抽塔罗全副 `count` 张（含逆位）。等价于 [`draw_deck`] 加 [`Deck::TarotFull`]。
#[must_use]
pub fn tarot(count: usize, seed: u64) -> Spread {
    draw_deck(Deck::TarotFull, count, seed)
}

/// 抽 Elder Futhark 卢恩 `count` 枚（含逆位）。等价于 [`draw_deck`] 加 [`Deck::ElderFuthark`]。
#[must_use]
pub fn runes(count: usize, seed: u64) -> Spread {
    draw_deck(Deck::ElderFuthark, count, seed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    const ALL_DECKS: [Deck; 5] = [
        Deck::TarotFull,
        Deck::TarotMajor,
        Deck::Lenormand,
        Deck::ElderFuthark,
        Deck::YoungerFuthark,
    ];

    #[test]
    fn deck_sizes_match_canonical_values() {
        // 多源公认大小：塔罗 78=22+56、Lenormand 36、Elder Futhark 24、Younger Futhark 16。
        assert_eq!(Deck::TarotFull.size(), 78);
        assert_eq!(Deck::TarotMajor.size(), 22);
        assert_eq!(Deck::Lenormand.size(), 36);
        assert_eq!(Deck::ElderFuthark.size(), 24);
        assert_eq!(Deck::YoungerFuthark.size(), 16);
        // Tarot Full = Major + Minor(56) 数学关系（Minor 4 花色 × 14 张 = 56）。
        assert_eq!(Deck::TarotFull.size(), Deck::TarotMajor.size() + 4 * 14);
    }

    #[test]
    fn lenormand_traditionally_no_reversal() {
        assert!(!Deck::Lenormand.reversible());
        for d in [
            Deck::TarotFull,
            Deck::TarotMajor,
            Deck::ElderFuthark,
            Deck::YoungerFuthark,
        ] {
            assert!(d.reversible(), "{} 应可逆位", d.name());
        }
    }

    #[test]
    fn id_roundtrip_for_all_decks() {
        for d in ALL_DECKS {
            assert_eq!(Deck::from_id(d.id()), Some(d));
        }
        assert_eq!(Deck::from_id("unknown"), None);
        assert_eq!(Deck::from_id(""), None);
    }

    #[test]
    fn deck_ids_unique() {
        let ids: HashSet<&str> = ALL_DECKS.iter().map(|d| d.id()).collect();
        assert_eq!(ids.len(), ALL_DECKS.len());
    }

    #[test]
    fn deck_names_distinct_and_nonempty() {
        let names: HashSet<&str> = ALL_DECKS.iter().map(|d| d.name()).collect();
        assert_eq!(names.len(), ALL_DECKS.len());
        for d in ALL_DECKS {
            assert!(!d.name().is_empty());
        }
    }

    #[test]
    fn draw_deck_carries_deck_id_and_traits() {
        for d in ALL_DECKS {
            let s = draw_deck(d, 3, 2024);
            assert_eq!(s.deck_id, d.id());
            assert_eq!(s.deck_size, d.size());
            assert_eq!(s.reversible, d.reversible());
            assert_eq!(s.cards.len(), 3.min(d.size()));
            if !d.reversible() {
                assert!(s.cards.iter().all(|c| !c.reversed));
            }
        }
    }

    #[test]
    fn deterministic_given_seed() {
        for d in ALL_DECKS {
            let a = draw_deck(d, 5.min(d.size()), 2024);
            let b = draw_deck(d, 5.min(d.size()), 2024);
            let av: Vec<_> = a.cards.iter().map(|c| (c.index, c.reversed)).collect();
            let bv: Vec<_> = b.cards.iter().map(|c| (c.index, c.reversed)).collect();
            assert_eq!(av, bv);
        }
        // 不同种子大概率不同（任意一个 deck 即可）
        let c = tarot(10, 7);
        let d = tarot(10, 2024);
        assert_ne!(c.cards[0].index, d.cards[0].index);
    }

    #[test]
    fn draw_is_without_replacement_all_decks() {
        for deck in ALL_DECKS {
            for seed in 0..50u64 {
                let s = draw_deck(deck, deck.size(), seed);
                let set: HashSet<usize> = s.cards.iter().map(|c| c.index).collect();
                assert_eq!(set.len(), deck.size(), "deck={} seed={seed} 重复或缺", deck.name());
                assert!(s.cards.iter().all(|c| c.index < deck.size()));
            }
        }
    }

    #[test]
    fn count_clamped_to_deck() {
        let s = draw_deck(Deck::YoungerFuthark, 100, 1);
        assert_eq!(s.cards.len(), RUNES_YOUNGER);
        let full: HashSet<usize> = s.cards.iter().map(|c| c.index).collect();
        assert_eq!(full.len(), RUNES_YOUNGER);
    }

    #[test]
    fn legacy_draw_keeps_no_deck_id() {
        let s = draw(TAROT_FULL, 3, true, 0);
        assert_eq!(s.deck_id, ""); // 不指定 deck 时为空
        assert_eq!(s.cards.len(), 3);
    }

    #[test]
    fn reversed_bits_reproducible_and_both_occur() {
        let s = tarot(40, 12345);
        let again = tarot(40, 12345);
        let r1: Vec<_> = s.cards.iter().map(|c| c.reversed).collect();
        let r2: Vec<_> = again.cards.iter().map(|c| c.reversed).collect();
        assert_eq!(r1, r2);
        assert!(r1.iter().any(|&x| x) && r1.iter().any(|&x| !x));
    }

    #[test]
    fn empty_draw() {
        for d in ALL_DECKS {
            let s = draw_deck(d, 0, 5);
            assert!(s.cards.is_empty());
        }
    }

    #[test]
    fn runes_helper_uses_elder_futhark() {
        let s = runes(3, 2024);
        assert_eq!(s.deck_id, "elder_futhark");
        assert_eq!(s.deck_size, RUNES_ELDER);
    }

    // ============ 牌名表测试 ============

    /// 4 个 const 表大小与 deck size 一致。
    #[test]
    fn name_tables_lengths_match_deck_sizes() {
        assert_eq!(TAROT_MAJOR_NAMES.len(), 22);
        assert_eq!(TAROT_MINOR_SUITS.len(), 4);
        assert_eq!(TAROT_MINOR_RANKS.len(), 14);
        assert_eq!(LENORMAND_NAMES.len(), Deck::Lenormand.size());
        assert_eq!(ELDER_FUTHARK_NAMES.len(), Deck::ElderFuthark.size());
        assert_eq!(YOUNGER_FUTHARK_NAMES.len(), Deck::YoungerFuthark.size());
    }

    /// 关键 oracle：Tarot Major 头尾 + 8/11 流派分歧。
    #[test]
    fn tarot_major_rws_vs_marseilles_swap() {
        // RWS 默认
        assert_eq!(tarot_major_at(0, TarotOrder::RiderWaite).0, "The Fool");
        assert_eq!(tarot_major_at(21, TarotOrder::RiderWaite).0, "The World");
        assert_eq!(tarot_major_at(8, TarotOrder::RiderWaite).0, "Strength");
        assert_eq!(tarot_major_at(11, TarotOrder::RiderWaite).0, "Justice");
        // Marseilles 颠倒
        assert_eq!(tarot_major_at(8, TarotOrder::Marseilles).0, "Justice");
        assert_eq!(tarot_major_at(11, TarotOrder::Marseilles).0, "Strength");
        // 其余 20 张两派一致
        for i in 0..22 {
            if i == 8 || i == 11 {
                continue;
            }
            assert_eq!(
                tarot_major_at(i, TarotOrder::RiderWaite),
                tarot_major_at(i, TarotOrder::Marseilles),
                "Major {i} 在两派应一致"
            );
        }
    }

    /// Tarot Minor 全 56 牌：`minor_full_name` 在 (suit， rank) ∈ 4×14 全覆盖且唯一。
    #[test]
    fn tarot_minor_56_full_names_unique() {
        let mut names: HashSet<String> = HashSet::new();
        for s in 0..4 {
            for r in 0..14 {
                let (en, zh) = minor_full_name(s, r);
                assert!(en.contains("of"), "Minor en 缺 of： {en}");
                assert!(!zh.is_empty());
                assert!(names.insert(en.clone()), "Minor 英文名重复： {en}");
            }
        }
        assert_eq!(names.len(), 56);
        // 抽样 oracle
        assert_eq!(minor_full_name(0, 0), ("Ace of Wands".into(), "权杖A".into()));
        assert_eq!(minor_full_name(3, 13), ("King of Pentacles".into(), "钱币国王".into()));
        assert_eq!(minor_full_name(1, 10), ("Page of Cups".into(), "圣杯侍从".into()));
    }

    /// Tarot 22 名唯一 + 中文名唯一 + 非空。
    #[test]
    fn tarot_major_names_unique() {
        let en: HashSet<&str> = TAROT_MAJOR_NAMES.iter().map(|p| p.0).collect();
        let zh: HashSet<&str> = TAROT_MAJOR_NAMES.iter().map(|p| p.1).collect();
        assert_eq!(en.len(), 22);
        assert_eq!(zh.len(), 22);
    }

    /// Lenormand 36 名英唯一 + 中文唯一 + 非空。
    #[test]
    fn lenormand_names_unique() {
        let en: HashSet<&str> = LENORMAND_NAMES.iter().map(|p| p.0).collect();
        let zh: HashSet<&str> = LENORMAND_NAMES.iter().map(|p| p.1).collect();
        assert_eq!(en.len(), 36);
        assert_eq!(zh.len(), 36);
    }

    /// Elder Futhark 24 + Younger Futhark 16 古北欧名唯一 + Unicode 字符在 Runic block(U+16A0..U+16FF)。
    #[test]
    fn futhark_glyphs_in_runic_block() {
        for &(name, glyph) in &ELDER_FUTHARK_NAMES {
            assert!(!name.is_empty());
            assert_eq!(glyph.chars().count(), 1, "Elder {name} 字符应 1 字");
            let c = glyph.chars().next().unwrap() as u32;
            assert!(
                (0x16A0..=0x16FF).contains(&c),
                "Elder {name} 字符 U+{c:04X} 越 Runic block"
            );
        }
        for &(name, glyph) in &YOUNGER_FUTHARK_NAMES {
            assert!(!name.is_empty());
            assert_eq!(glyph.chars().count(), 1, "Younger {name} 字符应 1 字");
            let c = glyph.chars().next().unwrap() as u32;
            assert!((0x16A0..=0x16FF).contains(&c), "Younger {name} 越 Runic block");
        }
        // 古北欧名唯一
        let elder: HashSet<&str> = ELDER_FUTHARK_NAMES.iter().map(|p| p.0).collect();
        let younger: HashSet<&str> = YOUNGER_FUTHARK_NAMES.iter().map(|p| p.0).collect();
        assert_eq!(elder.len(), 24);
        assert_eq!(younger.len(), 16);
    }

    /// `card_meta` 对每 deck 全索引非 panic + 英文名非空。
    #[test]
    fn card_meta_full_coverage() {
        for d in ALL_DECKS {
            for idx in 0..d.size() {
                let (en, _zh, _g) = card_meta(d, idx, TarotOrder::RiderWaite);
                if matches!(d, Deck::TarotFull) && idx >= 22 {
                    // Minor 由 card_meta 只给花色名（简化）；全名走 minor_full_name
                    continue;
                }
                assert!(!en.is_empty(), "{d:?} idx={idx} 英文名空");
            }
        }
    }

    /// `draw_deck` 输出的 DrawnCard 带英文名(Tarot/Lenormand)或 Unicode 字符(Futhark)。
    #[test]
    fn drawn_cards_carry_names() {
        // Tarot Major：取 3 张应每张有英文 + 中文
        let s = draw_deck(Deck::TarotMajor, 3, 12345);
        for c in &s.cards {
            assert!(!c.name.is_empty(), "Tarot Major 牌名空");
            assert!(!c.name_zh.is_empty(), "Tarot Major 中文名空");
            assert!(c.glyph.is_empty());
        }
        // Lenormand：英文 + 中文 + 不可逆位
        let s = draw_deck(Deck::Lenormand, 5, 12345);
        for c in &s.cards {
            assert!(!c.name.is_empty() && !c.name_zh.is_empty());
            assert!(c.glyph.is_empty());
            assert!(!c.reversed, "Lenormand 不应逆位");
        }
        // Elder Futhark：英文 + Unicode 字符 + 无中文
        let s = draw_deck(Deck::ElderFuthark, 4, 12345);
        for c in &s.cards {
            assert!(!c.name.is_empty());
            assert!(c.name_zh.is_empty(), "Futhark 不入中文");
            assert!(!c.glyph.is_empty(), "Futhark 应有 Unicode 字符");
        }
    }

    /// `draw_deck_with_order` 在同 seed 下 Tarot 8/11 牌名按流派切换。
    #[test]
    fn draw_with_marseilles_swaps_8_11() {
        // 抽全 22 大牌，在两派下应只在 8/11 牌名不同
        let rws = draw_deck_with_order(Deck::TarotMajor, TarotOrder::RiderWaite, 22, 42);
        let tdm = draw_deck_with_order(Deck::TarotMajor, TarotOrder::Marseilles, 22, 42);
        // 抽出 index 相同（同 seed）
        for (a, b) in rws.cards.iter().zip(tdm.cards.iter()) {
            assert_eq!(a.index, b.index);
            if a.index == 8 || a.index == 11 {
                assert_ne!(a.name, b.name, "8/11 应在两派下不同");
            } else {
                assert_eq!(a.name, b.name);
            }
        }
    }

    /// `TarotOrder::from_id` roundtrip。
    #[test]
    fn tarot_order_roundtrip() {
        assert_eq!(TarotOrder::from_id("rider_waite"), TarotOrder::RiderWaite);
        assert_eq!(TarotOrder::from_id("marseilles"), TarotOrder::Marseilles);
        assert_eq!(TarotOrder::from_id("unknown"), TarotOrder::RiderWaite);
        assert_eq!(TarotOrder::default(), TarotOrder::RiderWaite);
        assert_eq!(TarotOrder::RiderWaite.id(), "rider_waite");
        assert_eq!(TarotOrder::Marseilles.id(), "marseilles");
    }

    /// `draw` （无 deck） 与 `tarot_minor_at` （内部） 退化路径覆盖。
    #[test]
    fn anonymous_draw_has_empty_names() {
        let s = draw(20, 3, true, 99);
        for c in &s.cards {
            assert!(c.name.is_empty() && c.name_zh.is_empty() && c.glyph.is_empty());
        }
        // tarot_minor_at 内部 helper(card_meta 走 TarotFull idx>=22 通过 minor_full_name 而非 tarot_minor_at；
        // 这里直接调以覆盖)
        let (s_en, s_zh, _) = tarot_minor_at(0, 0);
        assert_eq!(s_en, "Wands");
        assert_eq!(s_zh, "权杖");
    }
}

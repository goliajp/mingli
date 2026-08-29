//! 抽取：无放回洗牌取前 N 张，可选正逆位。种子决定一切，同种子必同牌阵。

use super::*;

/// 抽出的一张牌：牌副中的序号、方向 + 牌名。
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
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
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
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

pub(crate) fn draw_internal(
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

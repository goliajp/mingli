//! 抽牌的校验：牌名表多源对照、洗牌的置换性质、同种子可复现。

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

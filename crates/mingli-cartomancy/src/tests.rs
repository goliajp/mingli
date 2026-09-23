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

/// Elder Futhark 24：名与字符**逐条对顺序**。
///
/// 原先只验了「字符落在 Runic 区块 U+16A0..U+16FF」——那连两个卢恩对调都看不出来。
/// 而顺序在这里是实义的：这套字母表按三个 ætt（各八个）排，抽签的 index → 卢恩
/// 这条映射错一位，整副就错了位。
///
/// 参照 en.wikipedia《Elder Futhark》的 ætt 表逐条对过：24 个字符与顺序全同，
/// 名字 22 个逐字相同，两个属异名而字符不变——第 14 个本表作 Perthro、维基作 Peordh，
/// 第 24 个本表作 Othala、维基作 Othalan。
#[test]
fn elder_futhark_follows_the_three_aettir() {
const ORDER: [(&str, &str); 24] = [
    ("Fehu", "ᚠ"), ("Uruz", "ᚢ"), ("Thurisaz", "ᚦ"), ("Ansuz", "ᚨ"),
    ("Raido", "ᚱ"), ("Kaunan", "ᚲ"), ("Gebo", "ᚷ"), ("Wunjo", "ᚹ"),
    ("Hagalaz", "ᚺ"), ("Naudiz", "ᚾ"), ("Isaz", "ᛁ"), ("Jeran", "ᛃ"),
    ("Eihwaz", "ᛇ"), ("Perthro", "ᛈ"), ("Algiz", "ᛉ"), ("Sowilo", "ᛊ"),
    ("Tiwaz", "ᛏ"), ("Berkanan", "ᛒ"), ("Ehwaz", "ᛖ"), ("Mannaz", "ᛗ"),
    ("Laguz", "ᛚ"), ("Ingwaz", "ᛜ"), ("Dagaz", "ᛞ"), ("Othala", "ᛟ"),
];
for (i, (name, glyph)) in ORDER.iter().enumerate() {
    assert_eq!(
        ELDER_FUTHARK_NAMES[i], (*name, *glyph),
        "第 {} 个应为「{name} {glyph}」，实为「{} {}」",
        i + 1,
        ELDER_FUTHARK_NAMES[i].0,
        ELDER_FUTHARK_NAMES[i].1,
    );
}
// 三个 ætt 各八个，是这套字母表的结构
assert_eq!(ELDER_FUTHARK_NAMES.len() % 8, 0, "24 应能分成三个各八个的 ætt");
assert_eq!(ELDER_FUTHARK_NAMES.len() / 8, 3);
// 「Futhark」这个名字取自前六个卢恩的首音：f-u-th-a-r-k
let initials: Vec<char> = ELDER_FUTHARK_NAMES[..6].iter().filter_map(|p| p.0.chars().next()).collect();
assert_eq!(initials, ['F', 'U', 'T', 'A', 'R', 'K'], "前六个的首字母应拼出 FUThARK");
}

/// Lenormand 36 名：**逐条对牌序**，不是只验唯一性。
///
/// 顺序本身就是内容——牌是按 1..36 固定编号的，index → 牌名这条映射错一位，
/// 整副牌就全错了位，而「36 个名字互不相同」对此完全无感（原先只验了这个）。
///
/// 参照 en.wikipedia《Lenormand cards》的编号表逐条对过，34 张字面相同或属其所列异体
/// （Trefoil/Clover、Cloud/Clouds 这类）。第 28、29 两张本表作 Gentleman / Lady，
/// 维基作 Man / Woman——两套名并行且都有据：本叶所引的 globalspiritualstudies
/// 正作「The Lady · Lenormand Card 29」，两名同指人事象征牌（significator）。
#[test]
fn lenormand_names_follow_the_numbered_order() {
// 1..36，与 en.wikipedia 的编号表逐条对应
const ORDER: [&str; 36] = [
    "Rider", "Clover", "Ship", "House", "Tree", "Clouds", "Snake", "Coffin", "Bouquet",
    "Scythe", "Whip", "Birds", "Child", "Fox", "Bear", "Stars", "Stork", "Dog",
    "Tower", "Garden", "Mountain", "Crossroads", "Mice", "Heart", "Ring", "Book", "Letter",
    "Gentleman", "Lady", "Lilies", "Sun", "Moon", "Key", "Fish", "Anchor", "Cross",
];
for (i, want) in ORDER.iter().enumerate() {
    assert_eq!(
        LENORMAND_NAMES[i].0, *want,
        "第 {} 张应为「{want}」，实为「{}」",
        i + 1,
        LENORMAND_NAMES[i].0,
    );
}
// 中文译名一一配齐且互不相同
let zh: HashSet<&str> = LENORMAND_NAMES.iter().map(|p| p.1).collect();
assert_eq!(zh.len(), 36, "中文译名应 36 个互不相同");
assert!(LENORMAND_NAMES.iter().all(|p| !p.1.is_empty()), "中文译名不许留空");
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
let (s_en, s_zh, _) = tarot_minor_at(0);
assert_eq!(s_en, "Wands");
assert_eq!(s_zh, "权杖");
}

/// 七十八张塔罗的花色分界与牌名，逐张钉住。
///
/// 大阿卡纳 0..21、小阿卡纳 22..77 每十四张一花色。此前没有一条测试逐张读过 `card_meta`，
/// 于是分界处的 `index < 22`（松成 `<=` 会让第 22 张变成「世界」）、
/// 花色分割 `(index − 22) / 14`（改成乘或取模就全乱）都无人过问。
#[test]
fn the_seventy_eight_cards_fall_into_their_arcana_and_suits() {
    use std::collections::BTreeSet;
    // 大阿卡纳两端与分界。
    for (i, want) in [(0_usize, "The Fool"), (21, "The World")] {
        let (en, _, _) = card_meta(Deck::TarotFull, i, TarotOrder::RiderWaite);
        assert_eq!(en, want, "第 {i} 张");
    }
    // 小阿卡纳：每十四张一花色，四色依次。
    for (suit, want) in [(0_usize, "Wands"), (1, "Cups"), (2, "Swords"), (3, "Pentacles")] {
        for k in 0..14_usize {
            let i = 22 + suit * 14 + k;
            let (en, _, _) = card_meta(Deck::TarotFull, i, TarotOrder::RiderWaite);
            assert_eq!(en, want, "第 {i} 张应属 {want}");
        }
    }
    // 第 22 张是小阿卡纳的第一张，不是大阿卡纳的第 22 张——分界就在这里。
    let (twenty_two, _, _) = card_meta(Deck::TarotFull, 22, TarotOrder::RiderWaite);
    assert_eq!(twenty_two, "Wands", "第 22 张已进小阿卡纳");
    let (twenty_one, _, _) = card_meta(Deck::TarotFull, 21, TarotOrder::RiderWaite);
    assert_ne!(twenty_one, twenty_two, "第 21 与第 22 张应分属两个阿卡纳");
    // 大阿卡纳二十二张互不相同。
    let majors: BTreeSet<&str> = (0..22_usize)
        .map(|i| card_meta(Deck::TarotFull, i, TarotOrder::RiderWaite).0)
        .collect();
    assert_eq!(majors.len(), 22, "大阿卡纳应有二十二个不同的名字");
}

/// 固定种子的一次抽牌，逐张钉住。
///
/// 现有的抽牌测试问的是「同种子同结果」「不放回」「张数被截断」「逆位两种都出现过」——
/// 这些性质在整副牌重新洗过、逆位流换一条种子之后照样成立。于是 `seed ^ 常量`
/// 改成 `|` 或 `&`、`reversible && dir.bit()` 改成 `||`，都没有测试红。
///
/// 值由当前实现算出，钉的是转写：它答的是「洗牌与逆位的取流动过没有」。
#[test]
fn a_fixed_seed_deals_the_same_ten_cards() {
    let sp = draw_deck_with_order(Deck::TarotFull, TarotOrder::RiderWaite, 6, 20_240_301);
    let got: Vec<(usize, bool, &str)> =
        sp.cards.iter().map(|c| (c.index, c.reversed, c.name.as_str())).collect();
    assert_eq!(
        got,
        vec![
            (12_usize, false, "The Hanged Man"),
            (10, true, "Wheel of Fortune"),
            (51, true, "Two of Swords"),
            (1, true, "The Magician"),
            (38, true, "Three of Cups"),
            (45, true, "Ten of Cups"),
        ],
        "固定种子抽出的牌变了——洗牌或逆位的取流动过了"
    );

    // 不可逆的牌组一张都不许翻。Lenormand 传统上不用逆位。
    let no_rev = draw_deck_with_order(Deck::Lenormand, TarotOrder::RiderWaite, 8, 20_240_301);
    assert!(
        no_rev.cards.iter().all(|c| !c.reversed),
        "不可逆的牌组出现了逆位——`reversible && …` 被写成了「或」"
    );
}


//! 牌名表：各 deck 的完整牌名，逐条多源校验后入码，不凭记忆硬编。

use super::*;

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

pub(crate) fn tarot_major_at(index: usize, order: TarotOrder) -> (&'static str, &'static str) {
    // RWS:8=Strength/11=Justice;Marseilles:8=Justice/11=Strength。
    let mapped = match (index, order) {
        (8, TarotOrder::Marseilles) => 11,
        (11, TarotOrder::Marseilles) => 8,
        (i, _) => i.min(21),
    };
    TAROT_MAJOR_NAMES[mapped]
}

pub(crate) fn tarot_minor_at(suit_idx: usize, rank_idx: usize) -> (&'static str, &'static str, &'static str) {
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

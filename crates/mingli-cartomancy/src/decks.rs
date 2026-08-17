//! 牌副与流派：一副牌有多大、允不允许逆位、8/11 两张牌怎么排。

use super::*;

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

//! 一课的结果形状：课式（九宗门）、盘面、以及涉害的两派取用。

use crate::Course;
#[cfg(feature = "serde")]
use serde::Serialize;

/// 三传课式（九宗门）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum Pattern {
    /// 重审（下贼上，取受贼之上神）。
    ZhongShen,
    /// 元首（上克下，取受克之上神）。
    YuanShou,
    /// 比用（克 ≥2，取与日干同阴阳之上神）。
    BiYong,
    /// 涉害（俱比/俱不比，数克深浅定，🟡 流派分歧不强编）。
    SheHai,
    /// 遥克·蒿矢（无上下克，天盘神克日）。
    HaoShi,
    /// 遥克·弹射（无上下克，日克天盘神）。
    TanShe,
    /// 昴星（无克无遥克，四课全，🟡 取传不强编）。
    MaoXing,
    /// 别责（四课不全，🟡 取传不强编）。
    BieZe,
    /// 八专（日干支同位，🟡 取传不强编）。
    BaZhuan,
    /// 伏吟（月将==时，天地同位）。
    FuYin,
    /// 返吟（天地相冲，offset==6）。
    FanYin,
}

impl Pattern {
    /// 课式中文名。
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Pattern::ZhongShen => "重审",
            Pattern::YuanShou => "元首",
            Pattern::BiYong => "比用",
            Pattern::SheHai => "涉害",
            Pattern::HaoShi => "蒿矢",
            Pattern::TanShe => "弹射",
            Pattern::MaoXing => "昴星",
            Pattern::BieZe => "别责",
            Pattern::BaZhuan => "八专",
            Pattern::FuYin => "伏吟",
            Pattern::FanYin => "返吟",
        }
    }
}

/// 一次大六壬起课的结果。
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct Cast {
    /// 日干（甲=0…癸=9）。
    pub day_stem: u8,
    /// 日支（子=0…亥=11）。
    pub day_branch: u8,
    /// 占时地支（子=0…亥=11）。
    pub hour_branch: u8,
    /// 月将地支（子=0…亥=11）。
    pub month_general: u8,
    /// 月将名。
    pub month_general_name: &'static str,
    /// 天地盘偏移。
    pub offset: u8,
    /// 天盘十二支：`heaven[g]` = 地盘 g 宫上神。
    pub heaven: [u8; 12],
    /// 四课。
    pub courses: [Course; 4],
    /// 三传课式。
    pub pattern: Pattern,
    /// 课式中文名。
    pub pattern_label: &'static str,
    /// 三传（初/中/末，地支序），仅在取传规则明确时给出；🟡 流派分歧的课式为 `None`。
    pub transmission: Option<[u8; 3]>,
}

/// 涉害的取用法两派，且**两派都不是抄错**——各有多源、各自点名对方。
///
/// - [`Classical`](SheHaiSchool::Classical)：古法。先数「受克深浅」，深者为用；深浅相等才按孟仲季。
///   《六壬大全》卷一歌诀「涉害行来本家止，路逢多克为用取」、卷七《课经》《袖中金》《观月经》、
///   《御定六壬直指》、《六壬粹言》卷一「此古法也」皆主此。本 crate 的六个古籍算例复算全中。
/// - [`ByPosition`](SheHaiSchool::ByPosition)：近法。不数深浅，直接孟 ＞ 仲 ＞ 季。
///   陈公献《六壬指南》系明言「涉害取法，只以孟仲季为准，**不以涉害深浅为义**，此《指南》所用之法，切记」；
///   《六壬粹言》卷一亦记「近来诸家，均未用之者」。
///
/// 默认取古法：它被算例直接支持，且近法只是它去掉第一步。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum SheHaiSchool {
    /// 古法：先数深浅。
    #[default]
    Classical,
    /// 近法：只按孟仲季。
    ByPosition,
}

impl SheHaiSchool {
    /// 稳定 id。
    #[must_use]
    pub const fn id(self) -> &'static str {
        match self {
            Self::Classical => "classical",
            Self::ByPosition => "by_position",
        }
    }

    /// 由稳定 id 解析；未知 → `None`。
    #[must_use]
    pub fn from_id(s: &str) -> Option<Self> {
        match s {
            "classical" => Some(Self::Classical),
            "by_position" => Some(Self::ByPosition),
            _ => None,
        }
    }
}
